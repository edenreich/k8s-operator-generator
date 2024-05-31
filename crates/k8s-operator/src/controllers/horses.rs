use anyhow::{Context, Result};
use kube::api::{Api, PostParams, Resource, WatchEvent};
use log::{error, info, warn};
use std::sync::Arc;

use openapi::{
    apis::{
        configuration::Configuration,
        horses_api::{
            create_horse, delete_horse_by_id, get_horse_by_id, update_horse_by_id,
        },
    },
    models::Horse as HorseDto,
};

use crate::types::horse::{Horse, HorseSpec, HorseStatus};
use crate::{add_finalizer, create_condition, remove_finalizer, update_status};

fn convert_kube_type_to_dto(horse: Horse) -> HorseDto {
    let uuid = match horse.status {
        Some(status) => status.uuid,
        None => None,
    };
    HorseDto {
        uuid,
        name: horse.spec.name,
        breed: horse.spec.breed,
        age: horse.spec.age,
    }
}

fn convert_dto_to_kube_type(horse: HorseDto) -> HorseSpec {
    HorseSpec {
        name: horse.name,
        breed: horse.breed,
        age: horse.age,
    }
}

pub async fn handle(
    config: Arc<Configuration>,
    event: WatchEvent<Horse>,
    kubernetes_api: Api<Horse>,
) -> anyhow::Result<()> {
    match event {
        WatchEvent::Added(mut horse) => {
            // Add default stauts if it's missing
            if horse.status.is_none() {
                horse.status = Some(HorseStatus::default());
            };

            // If the resource was marked for deletion, we need to delete it
            if horse.metadata.deletion_timestamp.is_some() {
                let condition = create_condition(
                    "Deleting",
                    "ProgressingDeletion",
                    "Deleting the resource",
                    "Resource is being deleted",
                    horse.meta().generation,
                );
                let mut horse_clone = horse.clone();
                if let Some(status) = horse_clone.status.as_mut() {
                    status.conditions.push(condition);
                    status.observed_generation = horse.meta().generation;
                }
                update_status(kubernetes_api.clone(), horse_clone).await?;
                return handle_delete_horse_by_id(&config, &mut horse, kubernetes_api).await;
            }

            // If the resource has no remote reference, meaning it's a new resource, so we need to create it
            // Otherwise, we need to check for drift
            match horse.clone().status.unwrap().uuid {
                Some(_) => {
                    check_for_drift(&config, kubernetes_api.clone(), &mut horse).await
                }
                None => {
                    let condition = create_condition(
                        "Creating",
                        "ProgressingCreating",
                        "Creating the resource",
                        "Resource is being created",
                        horse.meta().generation,
                    );
                    let mut horse_clone = horse.clone();
                    if let Some(status) = horse_clone.status.as_mut() {
                        status.conditions.push(condition);
                        status.observed_generation = horse.meta().generation;
                    }
                    update_status(kubernetes_api.clone(), horse_clone).await?;
                    handle_create_horse(&config, &mut horse, kubernetes_api).await
                }
            }
        }
        WatchEvent::Modified(mut horse) => {
            let condition = create_condition(
                "Updating",
                "ProgressingUpdating",
                "Updating the resource",
                "Resource is being updated",
                horse.meta().generation,
            );
            let mut horse_clone = horse.clone();
            if let Some(status) = horse_clone.status.as_mut() {
                status.conditions.push(condition);
                status.observed_generation = horse.meta().generation;
            }
            update_status(kubernetes_api.clone(), horse_clone).await?;
            handle_update_horse_by_id(&config, &mut horse, kubernetes_api).await
        }
        WatchEvent::Bookmark(bookmark) => {
            info!("Horse Bookmark: {:?}", bookmark.metadata.resource_version);
            Ok(())
        }
        _ => {
            info!("Horse {:?}", event);
            Ok(())
        }
    }
}

pub async fn check_for_drift(
    config: &Configuration,
    kubernetes_api: Api<Horse>,
    horse: &mut Horse,
) -> Result<()> {
    let dto = convert_kube_type_to_dto(horse.clone());
    let uuid = dto.uuid.clone().unwrap_or_default();

    if dto.uuid.is_none() {
        warn!("Horse has no status, cannot get by id or check for drift. Skipping...");
        return Ok(());
    }

    match get_horse_by_id(config, &uuid).await {
        Ok(dto) => {
            let remote_horse = convert_dto_to_kube_type(dto);
            if remote_horse != horse.spec {
                let current_horse_dto = convert_kube_type_to_dto(horse.clone());
                warn!("Horse has drifted remotely, sending an update to remote...");
                match update_horse_by_id(config, &uuid, current_horse_dto).await {
                    Ok(_) => {
                        info!("Horse updated successfully");
                        let condition = create_condition(
                            "Updated",
                            "AvailableUpdated",
                            "Updated the resource",
                            "Resource has been updated",
                            horse.meta().generation,
                        );
                        let mut horse_clone = horse.clone();
                        if let Some(status) = horse_clone.status.as_mut() {
                            status.conditions.push(condition);
                            status.observed_generation = horse.meta().generation;
                        }
                        return update_status(kubernetes_api.clone(), horse_clone).await;
                    }
                    Err(e) => {
                        error!("Failed to update Horse: {:?}", e);
                        return Err(anyhow::anyhow!("Failed to update horse: {:?}", e));
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to get Horse: {:?}", e);
            return Err(anyhow::anyhow!("Failed to get horse: {:?}", e));
        }
    }

    Ok(())
}

pub async fn handle_delete_horse_by_id(
    config: &Configuration,
    horse: &mut Horse,
    kubernetes_api: Api<Horse>,
) -> Result<()> {
    let dto = convert_kube_type_to_dto(horse.clone());
    let uuid = dto.uuid.clone().unwrap_or_default();

    if uuid.is_empty() {
        warn!("Horse has no uuid, cannot delete a horse by id. Skipping...");
        return Ok(());
    }

    delete_horse_by_id(config, &uuid)
        .await
        .context("Failed to delete a horse by id")?;

    remove_finalizer(horse, kubernetes_api.clone()).await?;
    let condition = create_condition(
        "Deleted",
        "UnavailableDeleted",
        "Deleted the resource",
        "Resource has has deleted",
        horse.meta().generation,
    );
    let mut horse_clone = horse.clone();
    if let Some(status) = horse_clone.status.as_mut() {
        status.conditions.push(condition);
        status.observed_generation = horse.meta().generation;
    }
    update_status(kubernetes_api.clone(), horse_clone).await
}

pub async fn handle_update_horse_by_id(
    config: &Configuration,
    horse: &mut Horse,
    kubernetes_api: Api<Horse>,
) -> Result<()> {
    let dto = convert_kube_type_to_dto(horse.clone());
    let uuid = match dto.uuid.clone() {
        Some(uuid) => uuid,
        None => {
            warn!("Horse has no status, cannot update by id. Skipping...");
            return Ok(());
        }
    };

    update_horse_by_id(config, &uuid, dto)
        .await
        .context("Failed to update a horse by id")?;

    let horse_name = horse.metadata.name.as_deref().unwrap_or_default();
    kubernetes_api
        .replace(horse_name, &PostParams::default(), horse)
        .await
        .context("Failed to update a horse by id")?;

    info!("Updated a horse by id went successfully");
    Ok(())
}

pub async fn handle_create_horse(
    config: &Configuration,
    horse: &mut Horse,
    kubernetes_api: Api<Horse>,
) -> Result<(), anyhow::Error> {
    let dto = convert_kube_type_to_dto(horse.clone());

    match create_horse(config, dto.clone()).await {
        Ok(remote_horse) => match remote_horse.uuid {
            Some(uuid) => {
                add_finalizer(horse, kubernetes_api.clone()).await?;
                let condition = create_condition(
                    "Created",
                    "AvailableCreated",
                    "Created the resource",
                    "Resource has been created",
                    horse.meta().generation,
                );
                let mut horse_clone = horse.clone();
                if let Some(status) = horse_clone.status.as_mut() {
                    status.conditions.push(condition);
                    status.uuid = Some(uuid);
                    status.observed_generation = horse.meta().generation;
                }
                update_status(kubernetes_api.clone(), horse_clone).await
            }
            None => {
                warn!("Remote horse has no uuid, cannot update status");
                Ok(())
            }
        },
        Err(e) => {
            error!("Failed to create a new horse: {:?}", e);
            Err(anyhow::anyhow!("Failed to create a new horse: {:?}", e))
        }
    }
}
