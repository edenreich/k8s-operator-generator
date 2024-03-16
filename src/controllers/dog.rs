use crate::add_event;
use crate::add_finalizer;
use crate::remove_finalizer;
use crate::Dog;
use kube::api::Api;
use kube::api::WatchEvent;
use kube::Resource;
use log::error;
use log::info;
use openapi_client::apis::configuration::Configuration;
use openapi_client::apis::default_api::dogs_get;
use openapi_client::apis::default_api::dogs_id_delete;
use openapi_client::apis::default_api::dogs_id_get;
use openapi_client::apis::default_api::dogs_id_put;
use openapi_client::apis::default_api::dogs_post;
use openapi_client::models::Dog as DogDto;

pub async fn handle_dog(event: WatchEvent<Dog>, api: Api<Dog>) {
    let kind = Dog::kind(&());
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
    let (mut dog, event_type) = match event {
        WatchEvent::Added(mut dog) => {
            if dog.metadata.deletion_timestamp.is_none() {
                add_finalizer(&mut dog, api.clone()).await;
                let dto = convert_to_dto(dog);
                dogs_post(config, dto).await;
            } else {
                let dto = convert_to_dto(dog);
                if let Some(id) = dto.id {
                    dogs_id_delete(config, id.as_str()).await;
                    remove_finalizer(&mut dog, api.clone()).await;
                } else {
                    error!(
                        "{} {} has no id",
                        kind_str,
                        dog.metadata.name.clone().unwrap()
                    );
                }
            }
            (dog, "Added")
        }
        WatchEvent::Modified(mut dog) => {
            let dto = convert_to_dto(dog);
            if let Some(id) = dto.id {
                dogs_id_put(config, id.as_str(), dto).await;
            } else {
                error!(
                    "{} {} has no id",
                    kind_str,
                    dog.metadata.name.clone().unwrap()
                );
            }
            (dog, "Modified")
        }
        WatchEvent::Deleted(mut dog) => {
            let dto = convert_to_dto(dog);
            if let Some(id) = dto.id {
                dogs_id_delete(config, id.as_str()).await;
                remove_finalizer(&mut dog, api.clone()).await;
            } else {
                error!(
                    "{} {} has no id",
                    kind_str,
                    dog.metadata.name.clone().unwrap()
                );
            }
            (dog, "Deleted")
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
        &mut dog,
        event_type.into(),
        kind_str.clone(),
        format!("Cat Resource {} Remotely", event_type),
    )
    .await;
    info!(
        "Cat {}: {:?} {:?}",
        event_type, dog.metadata.name, dog.metadata.finalizers
    );
}

fn convert_to_dto(dog_resource: Dog) -> DogDto {
    todo!("Convert the resource to a DTO");
}
