use kube::api::{Api, PostParams, Resource, WatchEvent};
use log::{error, info, warn};
use std::sync::Arc;

use openapi::{
    apis::{
        cats_api::{create_cat, delete_cat_by_id, get_cat_by_id, get_cats, update_cat_by_id},
        configuration::Configuration,
    },
    models::Cat as CatDto,
};

use crate::types::cat::{Cat, CatSpec, CatStatus};
use crate::{add_event, add_finalizer, create_condition, remove_finalizer, update_status};

fn convert_kube_type_to_dto(cat: Cat) -> CatDto {
    let uuid = match cat.status {
        Some(status) => status.uuid,
        None => None,
    };
    CatDto {
        uuid: uuid,
        name: cat.spec.name,
        breed: cat.spec.breed,
        age: cat.spec.age,
    }
}

fn convert_dto_to_kube_type(cat: CatDto) -> CatSpec {
    CatSpec {
        name: cat.name,
        breed: cat.breed,
        age: cat.age,
    }
}

pub async fn handle(config: Arc<Configuration>, event: WatchEvent<Cat>, kubernetes_api: Api<Cat>) {
    match event {
        WatchEvent::Added(mut cat) => {
            // Add default stauts if it's missing
            if cat.status.is_none() {
                cat.status = Some(CatStatus::default());
            };

            // If the resource was marked for deletion, we need to delete it
            if cat.metadata.deletion_timestamp.is_some() {
                let condition = create_condition(
                    "Deleting",
                    "ProgressingDeletion",
                    "Deleting the resource",
                    "Resource is being deleted",
                    cat.meta().generation,
                )
                .await;
                let mut cat_clone = cat.clone();
                if let Some(status) = cat_clone.status.as_mut() {
                    status.conditions.push(condition);
                    status.observed_generation = cat.meta().generation;
                }
                update_status(kubernetes_api.clone(), cat_clone).await;
                handle_delete_cat_by_id(&config, &mut cat, kubernetes_api).await;
                return;
            }

            // If the resource has no remote reference, meaning it's a new resource, so we need to create it
            // Otherwise, we need to check for drift
            match cat.clone().status.unwrap().uuid {
                Some(_) => {
                    check_for_drift(&config, kubernetes_api.clone(), &mut cat).await;
                    return;
                }
                None => {
                    let condition = create_condition(
                        "Creating",
                        "ProgressingCreating",
                        "Creating the resource",
                        "Resource is being created",
                        cat.meta().generation,
                    )
                    .await;
                    let mut cat_clone = cat.clone();
                    if let Some(status) = cat_clone.status.as_mut() {
                        status.conditions.push(condition);
                        status.observed_generation = cat.meta().generation;
                    }
                    update_status(kubernetes_api.clone(), cat_clone).await;
                    handle_create_cat(&config, &mut cat, kubernetes_api).await;
                    return;
                }
            }
        }
        WatchEvent::Modified(mut cat) => {
            let condition = create_condition(
                "Updating",
                "ProgressingUpdating",
                "Updating the resource",
                "Resource is being updated",
                cat.meta().generation,
            )
            .await;
            let mut cat_clone = cat.clone();
            if let Some(status) = cat_clone.status.as_mut() {
                status.conditions.push(condition);
                status.observed_generation = cat.meta().generation;
            }
            update_status(kubernetes_api.clone(), cat_clone).await;
            handle_update_cat_by_id(&config, &mut cat, kubernetes_api).await;
        }
        WatchEvent::Bookmark(bookmark) => {
            info!("Cat Bookmark: {:?}", bookmark.metadata.resource_version);
            return;
        }
        _ => {
            info!("Cat {:?}", event);
            return;
        }
    };
}

pub async fn check_for_drift(config: &Configuration, kubernetes_api: Api<Cat>, cat: &mut Cat) {
    let dto = convert_kube_type_to_dto(cat.clone());
    let uuid = dto.uuid.clone().unwrap_or_default();

    if dto.uuid.is_none() {
        warn!("Cat has no status, cannot get by id or check for drift. Skipping...");
        return;
    }

    match get_cat_by_id(&config, &uuid).await {
        Ok(dto) => {
            let remote_cat = convert_dto_to_kube_type(dto);
            if remote_cat != cat.spec {
                let current_cat_dto = convert_kube_type_to_dto(cat.clone());
                warn!("Cat has drifted remotely, sending an update to remote...");
                match update_cat_by_id(&config, &uuid, current_cat_dto).await {
                    Ok(_) => {
                        info!("Cat updated successfully");
                        let condition = create_condition(
                            "Updated",
                            "AvailableUpdated",
                            "Updated the resource",
                            "Resource has been updated",
                            cat.meta().generation,
                        )
                        .await;
                        let mut cat_clone = cat.clone();
                        if let Some(status) = cat_clone.status.as_mut() {
                            status.conditions.push(condition);
                            status.observed_generation = cat.meta().generation;
                        }
                        update_status(kubernetes_api.clone(), cat_clone).await;
                    }
                    Err(e) => {
                        error!("Failed to update Cat: {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to get Cat: {:?}", e);
        }
    }
}

pub async fn handle_delete_cat_by_id(
    config: &Configuration,
    cat: &mut Cat,
    kubernetes_api: Api<Cat>,
) {
    let dto = convert_kube_type_to_dto(cat.clone());
    let uuid = dto.uuid.clone().unwrap_or_default();

    if uuid.is_empty() {
        warn!("Cat has no uuid, cannot delete a cat by id. Skipping...");
        return;
    }

    if let Err(e) = delete_cat_by_id(&config, &uuid).await {
        error!("Failed to delete a cat by id: {:?}", e);
        return;
    }

    remove_finalizer(cat, kubernetes_api.clone()).await;
    let condition = create_condition(
        "Deleted",
        "UnavailableDeleted",
        "Deleted the resource",
        "Resource has has deleted",
        cat.meta().generation,
    )
    .await;
    let mut cat_clone = cat.clone();
    if let Some(status) = cat_clone.status.as_mut() {
        status.conditions.push(condition);
        status.observed_generation = cat.meta().generation;
    }
    update_status(kubernetes_api.clone(), cat_clone).await;
}

pub async fn handle_update_cat_by_id(
    config: &Configuration,
    cat: &mut Cat,
    kubernetes_api: Api<Cat>,
) {
    let dto = convert_kube_type_to_dto(cat.clone());
    let uuid = match dto.uuid.clone() {
        Some(uuid) => uuid,
        None => {
            warn!("Cat has no status, cannot update by id. Skipping...");
            return;
        }
    };

    if let Err(e) = update_cat_by_id(&config, &uuid, dto).await {
        error!("Failed to update a cat by id: {:?}", e);
        return;
    }

    let cat_name = cat.metadata.name.as_deref().unwrap_or_default();
    match kubernetes_api
        .replace(cat_name, &PostParams::default(), &cat)
        .await
    {
        Ok(_) => info!("update a cat by id went successfully"),
        Err(e) => error!("Failed to update a cat by id: {:?}", e),
    }
}

pub async fn handle_create_cat(config: &Configuration, cat: &mut Cat, kubernetes_api: Api<Cat>) {
    let dto = convert_kube_type_to_dto(cat.clone());

    match create_cat(&config, dto.clone()).await {
        Ok(remote_cat) => match remote_cat.uuid {
            Some(uuid) => {
                add_finalizer(cat, kubernetes_api.clone()).await;
                let condition = create_condition(
                    "Created",
                    "AvailableCreated",
                    "Created the resource",
                    "Resource has been created",
                    cat.meta().generation,
                )
                .await;
                let mut cat_clone = cat.clone();
                if let Some(status) = cat_clone.status.as_mut() {
                    status.conditions.push(condition);
                    status.uuid = Some(uuid);
                    status.observed_generation = cat.meta().generation;
                }
                update_status(kubernetes_api.clone(), cat_clone).await;
            }
            None => {
                warn!("Remote cat has no uuid, cannot update status");
            }
        },
        Err(e) => {
            error!("Failed to create a new cat: {:?}", e);
        }
    }
}
