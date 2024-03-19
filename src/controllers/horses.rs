use crate::add_event;
use crate::add_finalizer;
use crate::change_status;
use crate::remove_finalizer;
use crate::types::horse::Horse;
use kube::api::Api;
use kube::api::WatchEvent;
use kube::Resource;
use log::error;
use log::info;
use openapi::apis::configuration::Configuration;
use openapi::apis::horses_api::create_horse;
use openapi::apis::horses_api::delete_horse_by_id;
use openapi::apis::horses_api::update_horse_by_id;
use openapi::models::Horse as HorseDto;

fn convert_to_dto(horse: Horse) -> HorseDto {
    let uuid = match horse.status {
        Some(status) => status.uuid,
        None => None,
    };
    HorseDto { uuid: uuid }
}

pub async fn handle(event: WatchEvent<Horse>, kubernetes_api: Api<Horse>) {
    let kind = Horse::kind(&());
    let kind_str = kind.to_string();
    let config = &Configuration {
        base_path: "http://localhost:8080".to_string(),
        user_agent: None,
        client: reqwest::Client::new(),
        ..Configuration::default()
    };
    match event {
        WatchEvent::Added(mut horse) => {
            handle_added(config, kind_str, &mut horse, kubernetes_api).await
        }
        WatchEvent::Modified(mut horse) => {
            handle_modified(config, kind_str, &mut horse, kubernetes_api).await
        }
        WatchEvent::Bookmark(bookmark) => {
            info!("horse Bookmark: {:?}", bookmark.metadata.resource_version);
            return;
        }
        _ => {
            info!("Horse Unknown event {:?}", event);
            return;
        }
    };
}

pub async fn handle_added(
    config: &Configuration,
    kind_str: String,
    horse: &mut Horse,
    kubernetes_api: Api<Horse>,
) {
    if horse.metadata.deletion_timestamp.is_some() {
        handle_deleted(config, kind_str, horse, kubernetes_api).await;
        return;
    }
    if horse.status.is_none() {
        horse.status = Some(Default::default());
    }
    let model = horse.clone();
    let name = horse.metadata.name.clone().unwrap();
    let dto = convert_to_dto(model);
    if dto.uuid.is_some() {
        info!("{} {} already exists", kind_str, name);
        return;
    }
    add_finalizer(horse, kubernetes_api.clone()).await;
    match create_horse(config, dto).await {
        Ok(resp) => {
            info!("{} {} created", kind_str, name);
            change_status(horse, kubernetes_api.clone(), "uuid", resp.uuid.unwrap()).await;
            add_event(kind_str, horse, "Normal", "horse", "horse created").await;
        }
        Err(e) => {
            error!("Failed to create {} {}: {:?}", kind_str, name, e);
            remove_finalizer(horse, kubernetes_api.clone()).await;
        }
    };
}

pub async fn handle_modified(
    config: &Configuration,
    kind_str: String,
    horse: &mut Horse,
    kubernetes_api: Api<Horse>,
) {
    if horse.metadata.deletion_timestamp.is_some() {
        handle_deleted(config, kind_str, horse, kubernetes_api).await;
        return;
    }
    if horse.status.is_none() {
        horse.status = Some(Default::default());
    }
    let model = horse.clone();
    let name = horse.metadata.name.clone().unwrap();
    let dto = convert_to_dto(model);
    if dto.uuid.is_none() {
        info!("{} {} does not exist", kind_str, name);
        return;
    }
    let dto_clone = dto.clone();
    match update_horse_by_id(config, &dto.uuid.unwrap(), dto_clone).await {
        Ok(_) => {
            let msg = format!("{} {} updated", kind_str.clone(), name);
            info!("{}", msg);
            add_event(kind_str.clone(), horse, "Normal", &kind_str.clone(), &msg).await;
        }
        Err(e) => {
            let msg = format!("Failed to update {} {}: {:?}", kind_str.clone(), name, e);
            error!("{}", msg);
            add_event(kind_str.clone(), horse, "Error", &kind_str.clone(), &msg).await;
        }
    };
}

pub async fn handle_deleted(
    config: &Configuration,
    kind_str: String,
    horse: &mut Horse,
    _kubernetes_api: Api<Horse>,
) {
    let name = horse.metadata.name.clone().unwrap();
    match delete_horse_by_id(config, &horse.metadata.name.clone().unwrap()).await {
        Ok(_) => {
            info!("{} {} deleted", kind_str, name);
            add_event(kind_str, horse, "Normal", "horse", "horse deleted").await;
        }
        Err(e) => {
            error!("Failed to delete {} {}: {:?}", kind_str, name, e);
            add_event(
                kind_str,
                horse,
                "Error",
                "horse",
                "Failed to delete {} {} remotely",
            )
            .await;
        }
    };
}
