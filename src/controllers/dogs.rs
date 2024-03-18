use crate::add_event;
use crate::add_finalizer;
use crate::change_status;
use crate::remove_finalizer;
use crate::types::dog::Dog;
use kube::api::Api;
use kube::api::WatchEvent;
use kube::Resource;
use log::error;
use log::info;
use openapi::apis::configuration::Configuration;
use openapi::apis::dogs_api::create_dog;
use openapi::apis::dogs_api::delete_dog_by_id;
use openapi::apis::dogs_api::update_dog_by_id;
use openapi::models::Dog as DogDto;

fn convert_to_dto(dog: Dog) -> DogDto {
    let _uuid = match dog.status {
        Some(status) => status.uuid,
        None => None,
    };
    DogDto {
        uuid: _uuid,
        name: dog.spec.name,
        breed: dog.spec.breed,
        age: dog.spec.age,
    }
}

pub async fn handle(event: WatchEvent<Dog>, kubernetes_api: Api<Dog>) {
    let kind = Dog::kind(&());
    let kind_str = kind.to_string();
    let config = &Configuration {
        base_path: "http://localhost:8080".to_string(),
        user_agent: None,
        client: reqwest::Client::new(),
        ..Configuration::default()
    };
    match event {
        WatchEvent::Added(mut dog) => {
            handle_added(config, kind_str, &mut dog, kubernetes_api).await
        }
        WatchEvent::Modified(mut dog) => {
            handle_modified(config, kind_str, &mut dog, kubernetes_api).await
        }
        WatchEvent::Bookmark(bookmark) => {
            info!("dog Bookmark: {:?}", bookmark.metadata.resource_version);
            return;
        }
        _ => {
            info!("Dog Unknown event {:?}", event);
            return;
        }
    };
}

pub async fn handle_added(
    config: &Configuration,
    kind_str: String,
    dog: &mut Dog,
    kubernetes_api: Api<Dog>,
) {
    if dog.metadata.deletion_timestamp.is_some() {
        handle_deleted(config, kind_str, dog, kubernetes_api).await;
        return;
    }
    if dog.status.is_none() {
        dog.status = Some(Default::default());
    }
    let model = dog.clone();
    let name = dog.metadata.name.clone().unwrap();
    let dto = convert_to_dto(model);
    if dto.uuid.is_some() {
        info!("{} {} already exists", kind_str, name);
        return;
    }
    add_finalizer(dog, kubernetes_api.clone()).await;
    match create_dog(config, dto).await {
        Ok(resp) => {
            info!("{} {} created", kind_str, name);
            change_status(dog, kubernetes_api.clone(), "uuid", resp.uuid.unwrap()).await;
            add_event(kind_str, dog, "Normal", "dog", "dog created").await;
        }
        Err(e) => {
            error!("Failed to create {} {}: {:?}", kind_str, name, e);
            remove_finalizer(dog, kubernetes_api.clone()).await;
        }
    };
}

pub async fn handle_modified(
    config: &Configuration,
    kind_str: String,
    dog: &mut Dog,
    kubernetes_api: Api<Dog>,
) {
    if dog.metadata.deletion_timestamp.is_some() {
        handle_deleted(config, kind_str, dog, kubernetes_api).await;
        return;
    }
    if dog.status.is_none() {
        dog.status = Some(Default::default());
    }
    let model = dog.clone();
    let name = dog.metadata.name.clone().unwrap();
    let dto = convert_to_dto(model);
    if dto.uuid.is_none() {
        info!("{} {} does not exist", kind_str, name);
        return;
    }
    let dto_clone = dto.clone();
    match update_dog_by_id(config, &dto.uuid.unwrap(), dto_clone).await {
        Ok(_) => {
            let msg = format!("{} {} updated", kind_str.clone(), name);
            info!("{}", msg);
            add_event(kind_str.clone(), dog, "Normal", &kind_str.clone(), &msg).await;
        }
        Err(e) => {
            let msg = format!("Failed to update {} {}: {:?}", kind_str.clone(), name, e);
            error!("{}", msg);
            add_event(kind_str.clone(), dog, "Error", &kind_str.clone(), &msg).await;
        }
    };
}

pub async fn handle_deleted(
    config: &Configuration,
    kind_str: String,
    dog: &mut Dog,
    _kubernetes_api: Api<Dog>,
) {
    let name = dog.metadata.name.clone().unwrap();
    match delete_dog_by_id(config, &dog.metadata.name.clone().unwrap()).await {
        Ok(_) => {
            info!("{} {} deleted", kind_str, name);
            add_event(kind_str, dog, "Normal", "dog", "dog deleted").await;
        }
        Err(e) => {
            error!("Failed to delete {} {}: {:?}", kind_str, name, e);
            add_event(
                kind_str,
                dog,
                "Error",
                "dog",
                "Failed to delete {} {} remotely",
            )
            .await;
        }
    };
}
