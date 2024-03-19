use crate::{add_event, add_finalizer, change_status, remove_finalizer, types::cat::Cat};
use kube::{
    api::{Api, WatchEvent},
    Resource,
};
use log::{error, info};
use openapi::apis::cats_api::create_cat;
use openapi::apis::cats_api::delete_cat_by_id;
use openapi::apis::cats_api::update_cat_by_id;
use openapi::apis::configuration::Configuration;
use openapi::models::Cat as CatDto;
use std::sync::Arc;

fn convert_to_dto(cat: Cat) -> CatDto {
    let _uuid = match cat.status {
        Some(status) => status.uuid,
        None => None,
    };
    // CatDto {
    //     uuid: uuid,
    // }
    todo!("Implement the mapping for cats")
}

pub async fn handle(config: Arc<Configuration>, event: WatchEvent<Cat>, kubernetes_api: Api<Cat>) {
    let kind = Cat::kind(&());
    let kind_str = kind.to_string();
    match event {
        WatchEvent::Added(mut cat) => {
            handle_added(&config, kind_str, &mut cat, kubernetes_api).await
        }
        WatchEvent::Modified(mut cat) => {
            handle_modified(&config, kind_str, &mut cat, kubernetes_api).await
        }
        WatchEvent::Bookmark(bookmark) => {
            info!("Cat Bookmark: {:?}", bookmark.metadata.resource_version);
            return;
        }
        _ => {
            info!("Cat Unknown event {:?}", event);
            return;
        }
    };
}

pub async fn handle_added(
    config: &Configuration,
    kind_str: String,
    cat: &mut Cat,
    kubernetes_api: Api<Cat>,
) {
    if cat.metadata.deletion_timestamp.is_some() {
        handle_deleted(config, kind_str, cat, kubernetes_api).await;
        return;
    }
    if cat.status.is_none() {
        cat.status = Some(Default::default());
    }
    let model = cat.clone();
    let name = cat.metadata.name.clone().unwrap();
    let dto = convert_to_dto(model);
    if dto.uuid.is_some() {
        info!("{} {} already exists", kind_str, name);
        return;
    }
    add_finalizer(cat, kubernetes_api.clone()).await;
    match create_cat(config, dto).await {
        Ok(resp) => {
            info!("{} {} created", kind_str, name);
            change_status(cat, kubernetes_api.clone(), "uuid", resp.uuid.unwrap()).await;
            add_event(kind_str, cat, "Normal", "cat", "Cat created").await;
        }
        Err(e) => {
            error!("Failed to create {} {}: {:?}", kind_str, name, e);
            remove_finalizer(cat, kubernetes_api.clone()).await;
        }
    };
}

pub async fn handle_modified(
    config: &Configuration,
    kind_str: String,
    cat: &mut Cat,
    kubernetes_api: Api<Cat>,
) {
    if cat.metadata.deletion_timestamp.is_some() {
        handle_deleted(config, kind_str, cat, kubernetes_api).await;
        return;
    }
    if cat.status.is_none() {
        cat.status = Some(Default::default());
    }
    let model = cat.clone();
    let name = cat.metadata.name.clone().unwrap();
    let dto = convert_to_dto(model);
    if dto.uuid.is_none() {
        info!("{} {} does not exist", kind_str, name);
        return;
    }
    let dto_clone = dto.clone();
    match update_cat_by_id(config, &dto.uuid.unwrap(), dto_clone).await {
        Ok(_) => {
            let msg = format!("{} {} updated", kind_str.clone(), name);
            info!("{}", msg);
            add_event(kind_str.clone(), cat, "Normal", &kind_str.clone(), &msg).await;
        }
        Err(e) => {
            let msg = format!("Failed to update {} {}: {:?}", kind_str.clone(), name, e);
            error!("{}", msg);
            add_event(kind_str.clone(), cat, "Error", &kind_str.clone(), &msg).await;
        }
    };
}

pub async fn handle_deleted(
    config: &Configuration,
    kind_str: String,
    cat: &mut Cat,
    _kubernetes_api: Api<Cat>,
) {
    let name = cat.metadata.name.clone().unwrap();
    match delete_cat_by_id(config, &cat.metadata.name.clone().unwrap()).await {
        Ok(_) => {
            info!("{} {} deleted", kind_str, name);
            add_event(kind_str, cat, "Normal", "cat", "Cat deleted").await;
        }
        Err(e) => {
            error!("Failed to delete {} {}: {:?}", kind_str, name, e);
            add_event(
                kind_str,
                cat,
                "Error",
                "cat",
                "Failed to delete {} {} remotely",
            )
            .await;
        }
    };
}
