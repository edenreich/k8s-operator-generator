use anyhow::{Context, Result};
use kube::api::{Api, PostParams, Resource, WatchEvent};
use log::{error, info, warn};
use std::sync::Arc;

use openapi::{
    apis::{
        configuration::Configuration,
        dogs_api::{create_dog, delete_dog_by_id, get_dog_by_id, update_dog_by_id},
    },
    models::Dog as DogDto,
};

use crate::types::dog::{Dog, DogSpec, DogStatus};
use crate::{add_finalizer, create_condition, remove_finalizer, update_status};

fn convert_kube_type_to_dto(dog: Dog) -> DogDto {
    let uuid = match dog.status {
        Some(status) => status.uuid,
        None => None,
    };
    DogDto {
        uuid,
        name: dog.spec.name,
        breed: dog.spec.breed,
        age: dog.spec.age,
    }
}

fn convert_dto_to_kube_type(dog: DogDto) -> DogSpec {
    DogSpec {
        name: dog.name,
        breed: dog.breed,
        age: dog.age,
    }
}

pub async fn handle(
    config: Arc<Configuration>,
    event: WatchEvent<Dog>,
    kubernetes_api: Api<Dog>,
) -> anyhow::Result<()> {
    match event {
        WatchEvent::Added(mut dog) => {
            // Add default stauts if it's missing
            if dog.status.is_none() {
                dog.status = Some(DogStatus::default());
            };

            // If the resource was marked for deletion, we need to delete it
            if dog.metadata.deletion_timestamp.is_some() {
                let condition = create_condition(
                    "Deleting",
                    "ProgressingDeletion",
                    "Deleting the resource",
                    "Resource is being deleted",
                    dog.meta().generation,
                );
                let mut dog_clone = dog.clone();
                if let Some(status) = dog_clone.status.as_mut() {
                    status.conditions.push(condition);
                    status.observed_generation = dog.meta().generation;
                }
                update_status(kubernetes_api.clone(), dog_clone).await?;
                return handle_delete_dog_by_id(&config, &mut dog, kubernetes_api).await;
            }

            // If the resource has no remote reference, meaning it's a new resource, so we need to create it
            // Otherwise, we need to check for drift
            match dog.clone().status.unwrap().uuid {
                Some(_) => {
                    check_for_drift(&config, kubernetes_api.clone(), &mut dog).await
                }
                None => {
                    let condition = create_condition(
                        "Creating",
                        "ProgressingCreating",
                        "Creating the resource",
                        "Resource is being created",
                        dog.meta().generation,
                    );
                    let mut dog_clone = dog.clone();
                    if let Some(status) = dog_clone.status.as_mut() {
                        status.conditions.push(condition);
                        status.observed_generation = dog.meta().generation;
                    }
                    update_status(kubernetes_api.clone(), dog_clone).await?;
                    handle_create_dog(&config, &mut dog, kubernetes_api).await
                }
            }
        }
        WatchEvent::Modified(mut dog) => {
            let condition = create_condition(
                "Updating",
                "ProgressingUpdating",
                "Updating the resource",
                "Resource is being updated",
                dog.meta().generation,
            );
            let mut dog_clone = dog.clone();
            if let Some(status) = dog_clone.status.as_mut() {
                status.conditions.push(condition);
                status.observed_generation = dog.meta().generation;
            }
            update_status(kubernetes_api.clone(), dog_clone).await?;
            handle_update_dog_by_id(&config, &mut dog, kubernetes_api).await
        }
        WatchEvent::Bookmark(bookmark) => {
            info!("Dog Bookmark: {:?}", bookmark.metadata.resource_version);
            Ok(())
        }
        _ => {
            info!("Dog {:?}", event);
            Ok(())
        }
    }
}

pub async fn check_for_drift(
    config: &Configuration,
    kubernetes_api: Api<Dog>,
    dog: &mut Dog,
) -> Result<()> {
    let dto = convert_kube_type_to_dto(dog.clone());
    let uuid = dto.uuid.clone().unwrap_or_default();

    if dto.uuid.is_none() {
        warn!("Dog has no status, cannot get by id or check for drift. Skipping...");
        return Ok(());
    }

    match get_dog_by_id(config, &uuid).await {
        Ok(dto) => {
            let remote_dog = convert_dto_to_kube_type(dto);
            if remote_dog != dog.spec {
                let current_dog_dto = convert_kube_type_to_dto(dog.clone());
                warn!("Dog has drifted remotely, sending an update to remote...");
                match update_dog_by_id(config, &uuid, current_dog_dto).await {
                    Ok(_) => {
                        info!("Dog updated successfully");
                        let condition = create_condition(
                            "Updated",
                            "AvailableUpdated",
                            "Updated the resource",
                            "Resource has been updated",
                            dog.meta().generation,
                        );
                        let mut dog_clone = dog.clone();
                        if let Some(status) = dog_clone.status.as_mut() {
                            status.conditions.push(condition);
                            status.observed_generation = dog.meta().generation;
                        }
                        return update_status(kubernetes_api.clone(), dog_clone).await;
                    }
                    Err(e) => {
                        error!("Failed to update Dog: {:?}", e);
                        return Err(anyhow::anyhow!("Failed to update dog: {:?}", e));
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to get Dog: {:?}", e);
            return Err(anyhow::anyhow!("Failed to get dog: {:?}", e));
        }
    }

    Ok(())
}

pub async fn handle_delete_dog_by_id(
    config: &Configuration,
    dog: &mut Dog,
    kubernetes_api: Api<Dog>,
) -> Result<()> {
    let dto = convert_kube_type_to_dto(dog.clone());
    let uuid = dto.uuid.clone().unwrap_or_default();

    if uuid.is_empty() {
        warn!("Dog has no uuid, cannot delete a dog by id. Skipping...");
        return Ok(());
    }

    delete_dog_by_id(config, &uuid)
        .await
        .context("Failed to delete a dog by id")?;

    remove_finalizer(dog, kubernetes_api.clone()).await?;
    let condition = create_condition(
        "Deleted",
        "UnavailableDeleted",
        "Deleted the resource",
        "Resource has has deleted",
        dog.meta().generation,
    );
    let mut dog_clone = dog.clone();
    if let Some(status) = dog_clone.status.as_mut() {
        status.conditions.push(condition);
        status.observed_generation = dog.meta().generation;
    }
    update_status(kubernetes_api.clone(), dog_clone).await
}

pub async fn handle_update_dog_by_id(
    config: &Configuration,
    dog: &mut Dog,
    kubernetes_api: Api<Dog>,
) -> Result<()> {
    let dto = convert_kube_type_to_dto(dog.clone());
    let uuid = match dto.uuid.clone() {
        Some(uuid) => uuid,
        None => {
            warn!("Dog has no status, cannot update by id. Skipping...");
            return Ok(());
        }
    };

    update_dog_by_id(config, &uuid, dto)
        .await
        .context("Failed to update a dog by id")?;

    let dog_name = dog.metadata.name.as_deref().unwrap_or_default();
    kubernetes_api
        .replace(dog_name, &PostParams::default(), dog)
        .await
        .context("Failed to update a dog by id")?;

    info!("Updated a dog by id went successfully");
    Ok(())
}

pub async fn handle_create_dog(
    config: &Configuration,
    dog: &mut Dog,
    kubernetes_api: Api<Dog>,
) -> Result<(), anyhow::Error> {
    let dto = convert_kube_type_to_dto(dog.clone());

    match create_dog(config, dto.clone()).await {
        Ok(remote_dog) => match remote_dog.uuid {
            Some(uuid) => {
                add_finalizer(dog, kubernetes_api.clone()).await?;
                let condition = create_condition(
                    "Created",
                    "AvailableCreated",
                    "Created the resource",
                    "Resource has been created",
                    dog.meta().generation,
                );
                let mut dog_clone = dog.clone();
                if let Some(status) = dog_clone.status.as_mut() {
                    status.conditions.push(condition);
                    status.uuid = Some(uuid);
                    status.observed_generation = dog.meta().generation;
                }
                update_status(kubernetes_api.clone(), dog_clone).await
            }
            None => {
                warn!("Remote dog has no uuid, cannot update status");
                Ok(())
            }
        },
        Err(e) => {
            error!("Failed to create a new dog: {:?}", e);
            Err(anyhow::anyhow!("Failed to create a new dog: {:?}", e))
        }
    }
}
