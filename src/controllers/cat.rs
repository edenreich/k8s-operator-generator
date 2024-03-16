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
use openapi_client::apis::default_api::cats_get;
use openapi_client::apis::default_api::cats_id_delete;
use openapi_client::apis::default_api::cats_id_get;
use openapi_client::apis::default_api::cats_id_put;
use openapi_client::apis::default_api::cats_post;
use openapi_client::models::Cat as CatDto;

pub async fn handle_cat(event: WatchEvent<Cat>, api: Api<Cat>) {
    let kind = Cat::kind(&());
    let kind_str = kind.to_string();
    let config = &Configuration {
        base_path: "http://localhost:8080".to_string(),
        user_agent: None,
        client: reqwest::Client::new(),
        basic_auth: todo!(),
        oauth_access_token: todo!(),
        bearer_access_token: todo!(),
        api_key: todo!(),
    };
    let (mut cat, event_type) = match event {
        WatchEvent::Added(mut cat) => {
            if cat.metadata.deletion_timestamp.is_none() {
                add_finalizer(&mut cat, api.clone()).await;
                let dto = convert_to_dto(cat);
                cats_post(config, dto).await;
            } else {
                let dto = convert_to_dto(cat);
                if let Some(id) = dto.id {
                    cats_id_delete(config, id.as_str()).await;
                    remove_finalizer(&mut cat, api.clone()).await;
                } else {
                    error!(
                        "{} {} has no id",
                        kind_str,
                        cat.metadata.name.clone().unwrap()
                    );
                }
            }
            (cat, "Added")
        }
        WatchEvent::Modified(mut cat) => {
            let dto = convert_to_dto(cat);
            if let Some(id) = dto.id {
                cats_id_put(config, id.as_str(), dto).await;
            } else {
                error!(
                    "{} {} has no id",
                    kind_str,
                    cat.metadata.name.clone().unwrap()
                );
            }
            (cat, "Modified")
        }
        WatchEvent::Deleted(mut cat) => {
            let dto = convert_to_dto(cat);
            if let Some(id) = dto.id {
                cats_id_delete(config, id.as_str()).await;
                remove_finalizer(&mut cat, api.clone()).await;
            } else {
                error!(
                    "{} {} has no id",
                    kind_str,
                    cat.metadata.name.clone().unwrap()
                );
            }
            (cat, "Deleted")
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
    add_event(
        kind_str.clone(),
        &mut cat,
        event_type.into(),
        kind_str.clone(),
        format!("Cat Resource {} Remotely", event_type),
    )
    .await;
    info!(
        "Cat {}: {:?} {:?}",
        event_type, cat.metadata.name, cat.metadata.finalizers
    );
}

fn convert_to_dto(cat_resource: Cat) -> CatDto {
    CatDto {
        id: cat_resource.spec.id,
        name: cat_resource.spec.name,
        breed: cat_resource.spec.breed,
        age: cat_resource.spec.age,
    }
}
