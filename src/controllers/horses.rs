use crate::{
    add_event, add_finalizer, change_status, remove_finalizer, types::horse::Horse,
    types::horse::HorseSpec,
};
use kube::{
    api::{Api, WatchEvent},
    Resource,
};
use log::{error, info, warn};
use openapi::apis::configuration::Configuration;
use openapi::apis::horses_api::create_horse;
use openapi::apis::horses_api::delete_horse_by_id;
use openapi::apis::horses_api::get_horse_by_id;
use openapi::apis::horses_api::update_horse_by_id;
use openapi::models::Horse as HorseDto;
use std::sync::Arc;

fn convert_kube_type_to_dto(horse: Horse) -> HorseDto {
    let uuid = match horse.status {
        Some(status) => status.uuid,
        None => None,
    };
    HorseDto {
        uuid: uuid,
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
) {
    let kind = Horse::kind(&());
    let kind_str = kind.to_string();
    match event {
        WatchEvent::Added(mut horse) => {
            handle_added(&config, kind_str, &mut horse, kubernetes_api).await
        }
        WatchEvent::Modified(mut horse) => {
            handle_modified(&config, kind_str, &mut horse, kubernetes_api).await
        }
        WatchEvent::Bookmark(bookmark) => {
            info!("Horse Bookmark: {:?}", bookmark.metadata.resource_version);
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
        info!(
            "{} {} status is None",
            kind_str,
            horse.metadata.name.clone().unwrap()
        );
        horse.status = Some(Default::default());
    }
    let model = horse.clone();
    let name = horse.metadata.name.clone().unwrap();
    let dto = convert_kube_type_to_dto(model);
    if dto.uuid.is_some() {
        info!("{} {} already exists", kind_str, name);
        check_for_drift(config.clone(), horse.clone(), kubernetes_api.clone())
            .await
            .unwrap();
        return;
    }
    add_finalizer(horse, kubernetes_api.clone()).await;
    match create_horse(config, dto).await {
        Ok(resp) => {
            let msg = format!("{} {} created", kind_str.clone(), name);
            info!("{}", msg);
            change_status(horse, kubernetes_api.clone(), "uuid", resp.uuid.unwrap()).await;
            add_event(kind_str, horse, "Normal", "horse", &msg).await;
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
    let dto = convert_kube_type_to_dto(model);
    if dto.uuid.is_none() {
        info!("{} {} does not exist", kind_str, name);
        return;
    }
    let uuid_clone = dto.uuid.clone().unwrap();
    match get_horse_by_id(config, &uuid_clone).await {
        Ok(current_horse) => {
            if dto != current_horse {
                warn!(
                    "Drift detected for {} {}. Desired: {:?}, Current: {:?}",
                    kind_str, name, dto, current_horse
                );
                match update_horse_by_id(config, &uuid_clone, dto).await {
                    Ok(_) => {
                        let msg = format!("{} {} updated", kind_str.clone(), name);
                        info!("{}", msg);
                        add_event(kind_str.clone(), horse, "Normal", &kind_str.clone(), &msg).await;
                    }
                    Err(e) => {
                        let msg =
                            format!("Failed to update {} {}: {:?}", kind_str.clone(), name, e);
                        error!("{}", msg);
                        add_event(kind_str.clone(), horse, "Error", &kind_str.clone(), &msg).await;
                    }
                };
            }
        }
        Err(e) => {
            error!(
                "Failed to get current state of {} {}: {:?}",
                kind_str, name, e
            );
        }
    };
}

pub async fn handle_deleted(
    config: &Configuration,
    kind_str: String,
    horse: &mut Horse,
    kubernetes_api: Api<Horse>,
) {
    let name = horse.metadata.name.clone().unwrap();

    let uuid = match horse.status.clone() {
        Some(status) => status.uuid,
        None => None,
    }
    .unwrap();

    match delete_horse_by_id(config, &uuid).await {
        Ok(_) => {
            let msg = format!("{} {} deleted", kind_str, name);
            info!("{}", msg);
            add_event(kind_str, horse, "Normal", "horse", &msg).await;
            remove_finalizer(horse, kubernetes_api.clone()).await;
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

pub async fn check_for_drift(
    config: Configuration,
    horse: Horse,
    kubernetes_api: Api<Horse>,
) -> Result<bool, kube::Error> {
    let kind = Horse::kind(&());
    let kind_str = kind.to_string();
    let horse_clone = horse.clone();
    let dto = convert_kube_type_to_dto(horse_clone);
    if dto.uuid.is_none() {
        info!(
            "{} {} does not exist",
            kind_str,
            horse.metadata.name.clone().unwrap()
        );
        return Ok(false);
    }
    let uuid_clone = dto.uuid.clone().unwrap();
    match get_horse_by_id(&config, &uuid_clone).await {
        Ok(remote_horse) => {
            if dto != remote_horse {
                warn!(
                    "Drift detected for {} {}. Desired: {:?}, Current: {:?}",
                    kind_str,
                    horse.metadata.name.clone().unwrap(),
                    dto,
                    remote_horse
                );
                let remote_horse_spec = convert_dto_to_kube_type(remote_horse.clone());

                if horse.spec != remote_horse_spec {
                    handle_modified(&config, kind_str, &mut horse.clone(), kubernetes_api).await;
                }
                return Ok(true);
            }
        }
        Err(e) => {
            error!(
                "Failed to get current state of {} {}: {:?}",
                kind_str,
                horse.metadata.name.clone().unwrap(),
                e
            );
        }
    };
    Ok(false)
}
