use kube::api::{Api, PostParams, WatchEvent};
use log::{error, info, warn};
use std::sync::Arc;

use openapi::{
    apis::{
        cats_api::{create_cat, delete_cat_by_id, get_cat_by_id, get_cats, update_cat_by_id},
        configuration::Configuration,
    },
    models::Cat as CatDto,
};

use crate::types::cat::{Cat, CatSpec};
use crate::{add_event, add_finalizer, change_status, remove_finalizer};

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
            // If the resource has no status, meaning it's a new resource, so we need to create it
            if cat.status.is_none() {
                handle_create_cat(&config, &mut cat, kubernetes_api).await;
                return;
            }

            // If the resource was marked for deletion, we need to delete it
            if cat.metadata.deletion_timestamp.is_some() {
                handle_delete_cat_by_id(&config, &mut cat, kubernetes_api).await;
                return;
            }

            // If the resource does exist, we check for drift and update the remote resource if necessary
            check_for_drift(&config, &mut cat).await;
        }
        WatchEvent::Modified(mut cat) => {
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

pub async fn handle_create_cat(config: &Configuration, cat: &mut Cat, kubernetes_api: Api<Cat>) {
    let dto = convert_kube_type_to_dto(cat.clone());
    match create_cat(&config, dto.clone()).await {
        Ok(remote_cat) => {
            add_finalizer(cat, kubernetes_api.clone()).await;
            change_status(
                cat,
                kubernetes_api.clone(),
                "uuid",
                remote_cat.uuid.unwrap(),
            )
            .await;
        }
        Err(e) => {
            error!("Failed to create a new cat: {:?}", e);
        }
    }
}

pub async fn handle_delete_cat_by_id(
    config: &Configuration,
    cat: &mut Cat,
    kubernetes_api: Api<Cat>,
) {
    let dto = convert_kube_type_to_dto(cat.clone());
    if dto.uuid.is_none() {
        warn!("Cat has no status, cannot delete by id");
        return;
    }
    let uuid = dto.uuid.as_ref().unwrap().clone();
    match delete_cat_by_id(&config, &uuid).await {
        Ok(_) => {
            remove_finalizer(cat, kubernetes_api.clone()).await;
        }
        Err(e) => {
            error!("Failed to delete a cat by id: {:?}", e);
        }
    }
}

pub async fn handle_update_cat_by_id(
    config: &Configuration,
    cat: &mut Cat,
    kubernetes_api: Api<Cat>,
) {
    let dto = convert_kube_type_to_dto(cat.clone());
    if dto.uuid.is_none() {
        warn!("Cat has no status, cannot put by id");
        return;
    }
    let uuid = dto.uuid.as_ref().unwrap().clone();
    match update_cat_by_id(&config, &uuid, dto.clone()).await {
        Ok(_) => {
            match kubernetes_api
                .replace(
                    cat.metadata.name.as_deref().unwrap_or_default(),
                    &PostParams::default(),
                    &cat,
                )
                .await
            {
                Ok(_) => {
                    info!("update a cat by id went successfully");
                }
                Err(e) => {
                    error!("Failed to update a cat by id: {:?}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to update a cat by id: {:?}", e);
        }
    }
}

pub async fn check_for_drift(config: &Configuration, cat: &mut Cat) {
    let dto = convert_kube_type_to_dto(cat.clone());
    if dto.uuid.is_none() {
        warn!("Cat has no status, cannot get by id or check for drift. Skipping...");
        return;
    }

    let uuid = dto.uuid.as_ref().unwrap().clone();
    match get_cat_by_id(&config, &dto.uuid.unwrap()).await {
        Ok(dto) => {
            let remote_cat = convert_dto_to_kube_type(dto);
            if remote_cat != cat.spec {
                let current_cat_dto = convert_kube_type_to_dto(cat.clone());
                warn!("Cat has drifted remotely, sending an update to remote...");
                match update_cat_by_id(&config, &uuid, current_cat_dto).await {
                    Ok(_) => {
                        info!("Cat updated successfully");
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
