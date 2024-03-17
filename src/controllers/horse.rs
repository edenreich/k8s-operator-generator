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
use log::warn;
use openapi_client::apis::configuration::Configuration;
use openapi_client::apis::default_api::horses_id_delete;
use openapi_client::apis::default_api::horses_id_put;
use openapi_client::apis::default_api::horses_post;
use openapi_client::models::Horse as HorseDto;

fn convert_to_dto(horse_resource: Horse) -> HorseDto {
    let _uuid = match horse_resource.status {
        Some(status) => status.uuid,
        None => None,
    };
    todo!("Convert the resource to a DTO");
}

pub async fn handle_horse(event: WatchEvent<Horse>, api: Api<Horse>) {
    let kind = Horse::kind(&());
    let kind_str = kind.to_string();
    let config = &Configuration {
        base_path: "http://localhost:8080".to_string(),
        user_agent: None,
        client: reqwest::Client::new(),
        ..Configuration::default()
    };
    match event {
        WatchEvent::Added(mut horse) => handle_added(config, kind_str, &mut horse, api).await,
        WatchEvent::Modified(mut horse) => handle_modified(config, kind_str, &mut horse, api).await,
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

async fn handle_added(
    config: &Configuration,
    kind_str: String,
    horse: &mut Horse,
    api: Api<Horse>,
) {
    if horse.metadata.deletion_timestamp.is_some() {
        handle_deleted(config, kind_str, horse, api).await;
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
    add_finalizer(horse, api.clone()).await;
    match horses_post(config, dto).await {
        Ok(resp) => {
            info!("{} {} created successfully", kind_str, name);
            add_event(
                kind_str.clone(),
                horse,
                "Normal".into(),
                kind_str.clone(),
                format!("{} {} created remotely", kind_str, name),
            )
            .await;
            if let (Some(_status), Some(uuid)) = (horse.status.as_mut(), resp.uuid.clone()) {
                change_status(horse, api.clone(), "uuid", uuid).await;
            } else {
                warn!("Failed to retrieve uuid from response");
            }
        }
        Err(e) => {
            error!("Failed to add {} {}: {:?}", kind_str, name, e);
            add_event(
                kind_str.clone(),
                horse,
                "Error".into(),
                kind_str.clone(),
                format!("Failed to create {} {} remotely", kind_str, name),
            )
            .await;
        }
    }
}

async fn handle_modified(
    config: &Configuration,
    kind_str: String,
    horse: &mut Horse,
    api: Api<Horse>,
) {
    if horse.metadata.deletion_timestamp.is_some() {
        handle_deleted(config, kind_str, horse, api).await;
        return;
    }
    let model = horse.clone();
    let dto = convert_to_dto(model);
    let name = horse.metadata.name.clone().unwrap();
    if let Some(ref uuid) = dto.uuid.clone() {
        match horses_id_put(config, uuid.as_str(), dto).await {
            Ok(_) => {
                info!("{} {} modified successfully", kind_str, name);
                add_event(
                    kind_str.clone(),
                    horse,
                    "Normal".into(),
                    kind_str.clone(),
                    format!("{} {} modified remotely", kind_str, name),
                )
                .await;
            }
            Err(e) => {
                error!("Failed to update {} {}: {:?}", kind_str, name, e);
                add_event(
                    kind_str.clone(),
                    horse,
                    "Error".into(),
                    kind_str.clone(),
                    format!("Failed to update {} {} remotely", kind_str, name),
                )
                .await;
            }
        }
    } else {
        error!("{} {} has no id", kind_str, name,);
        add_event(
            kind_str.clone(),
            horse,
            "Error".into(),
            kind_str.clone(),
            format!("Failed to update {} {}", kind_str, name),
        )
        .await;
    }
}

async fn handle_deleted(
    config: &Configuration,
    kind_str: String,
    horse: &mut Horse,
    api: Api<Horse>,
) {
    let model = horse.clone();
    let dto = convert_to_dto(model);
    let name = horse.metadata.name.clone().unwrap();
    if let Some(uuid) = dto.uuid.clone() {
        match horses_id_delete(config, uuid.as_str()).await {
            Ok(_res) => {
                info!("{} {} deleted successfully", kind_str, name);
                add_event(
                    kind_str.clone(),
                    horse,
                    "Normal".into(),
                    kind_str.clone(),
                    format!("{} {} deleted remotely", kind_str, name),
                )
                .await;
                remove_finalizer(horse, api.clone()).await;
            }
            Err(e) => {
                error!("Failed to delete {} {}: {:?}", kind_str, name, e);
                add_event(
                    kind_str.clone(),
                    horse,
                    "Error".into(),
                    kind_str.clone(),
                    format!("Failed to delete {} {} remotely", kind_str, name),
                )
                .await;
            }
        }
    } else {
        error!(
            "{} {} has no id",
            kind_str,
            horse.metadata.name.clone().unwrap()
        );
        add_event(
            kind_str.clone(),
            horse,
            "Error".into(),
            kind_str.clone(),
            format!("Failed to delete {} {}", kind_str, name),
        )
        .await;
    }
}
