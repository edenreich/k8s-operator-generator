use crate::add_event;
use crate::add_finalizer;
use crate::remove_finalizer;
use crate::Cat;
use kube::api::Api;
use kube::api::WatchEvent;
use kube::Resource;
use log::error;
use log::info;
use openapi_client::apis::configuration::Configuration;
use openapi_client::apis::default_api::cats_id_delete;
use openapi_client::apis::default_api::cats_id_put;
use openapi_client::apis::default_api::cats_post;
use openapi_client::models::Cat as CatDto;

fn convert_to_dto(cat_resource: Cat) -> CatDto {
    CatDto {
        id: cat_resource.spec.id,
        name: cat_resource.spec.name,
        breed: cat_resource.spec.breed,
        age: cat_resource.spec.age,
    }
}

pub async fn handle_cat(event: WatchEvent<Cat>, api: Api<Cat>) {
    let kind = Cat::kind(&());
    let kind_str = kind.to_string();
    let config = &Configuration {
        base_path: "http://localhost:8080".to_string(),
        user_agent: None,
        client: reqwest::Client::new(),
        ..Configuration::default()
    };
    match event {
        WatchEvent::Added(mut cat) => handle_added(config, kind_str, &mut cat, api).await,
        WatchEvent::Modified(mut cat) => handle_modified(config, kind_str, &mut cat, api).await,
        WatchEvent::Deleted(mut cat) => handle_deleted(config, kind_str, &mut cat, api).await,
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

async fn handle_added(config: &Configuration, kind_str: String, cat: &mut Cat, api: Api<Cat>) {
    if cat.metadata.deletion_timestamp.is_some() {
        return;
    }
    let model = cat.clone();
    let dto = convert_to_dto(model);
    add_finalizer(cat, api.clone()).await;
    match cats_post(config, dto).await {
        Ok(_) => {
            info!("{} added successfully", kind_str);
            add_event(
                kind_str.clone(),
                cat,
                "Normal".into(),
                kind_str.clone(),
                format!("{} added remotely", kind_str),
            )
            .await;
        }
        Err(e) => {
            error!("Failed to add {:?}: {:?}", cat, e);
            add_event(
                kind_str.clone(),
                cat,
                "Error".into(),
                kind_str.clone(),
                format!("Failed to create {} remotely", kind_str),
            )
            .await;
        }
    }
}

async fn handle_modified(config: &Configuration, kind_str: String, cat: &mut Cat, api: Api<Cat>) {
    let model = cat.clone();
    let dto = convert_to_dto(model);
    if let Some(ref id) = dto.id.clone() {
        match cats_id_put(config, id.as_str(), dto).await {
            Ok(_) => {
                info!("{} modified successfully", kind_str);
                add_event(
                    kind_str.clone(),
                    cat,
                    "Normal".into(),
                    kind_str.clone(),
                    format!("{} modified remotely", kind_str),
                )
                .await;
            }
            Err(e) => {
                error!("Failed to update {:?}: {:?}", cat, e);
                add_event(
                    kind_str.clone(),
                    cat,
                    "Error".into(),
                    kind_str.clone(),
                    format!("Failed to update {} remotely", kind_str),
                )
                .await;
            }
        }
    } else {
        error!(
            "{} {} has no id",
            kind_str,
            cat.metadata.name.clone().unwrap()
        );
    }
}

async fn handle_deleted(config: &Configuration, kind_str: String, cat: &mut Cat, api: Api<Cat>) {
    let model = cat.clone();
    let dto = convert_to_dto(model);
    if let Some(id) = dto.id.clone() {
        match cats_id_delete(config, id.as_str()).await {
            Ok(res) => {
                info!("{} deleted successfully", kind_str);
                add_event(
                    kind_str.clone(),
                    cat,
                    "Normal".into(),
                    kind_str.clone(),
                    format!("{} deleted remotely", kind_str),
                )
                .await;
                remove_finalizer(cat, api.clone()).await;
            }
            Err(e) => {
                error!("Failed to delete {:?}: {:?}", cat, e);
                add_event(
                    kind_str.clone(),
                    cat,
                    "Error".into(),
                    kind_str.clone(),
                    format!("Failed to delete {} remotely", kind_str),
                )
                .await;
            }
        }
    } else {
        error!(
            "{} {} has no id",
            kind_str,
            cat.metadata.name.clone().unwrap()
        );
    }
}
