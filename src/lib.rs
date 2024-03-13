use futures_util::stream::StreamExt;
use kube::api::{Api, WatchEvent, WatchParams};
use log::{error, info};
use tokio::time::{sleep, Duration};

pub async fn watch_resource<T>(
    api: Api<T>,
    watch_params: WatchParams,
    handler: fn(WatchEvent<T>),
) -> anyhow::Result<()>
where
    T: Clone + core::fmt::Debug + serde::de::DeserializeOwned + 'static,
{
    let mut stream = api.watch(&watch_params, "0").await?.boxed();
    loop {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => handler(event),
                Err(e) => error!("Error watching resource: {:?}", e),
            }
        }
        sleep(Duration::from_secs(1)).await;
        stream = api.watch(&watch_params, "0").await?.boxed();
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    serde::Deserialize,
    serde::Serialize,
    schemars::JsonSchema,
    kube::CustomResource,
)]
#[kube(
    group = "example.com",
    version = "v1",
    kind = "Cat",
    plural = "cats",
    namespaced
)]
pub struct CatSpec {
    id: String,
    name: String,
    breed: String,
    age: u32,
}

pub fn handle_cat_event(event: WatchEvent<Cat>) {
    match event {
        WatchEvent::Added(resource) => {
            info!("{} Added: {:?}", "Cat", resource.metadata.name);
        }
        WatchEvent::Modified(resource) => {
            info!("{} Modified: {:?}", "Cat", resource.metadata.name);
        }
        WatchEvent::Deleted(resource) => {
            info!("{} Deleted: {:?}", "Cat", resource.metadata.name);
        }
        _ => {}
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    serde::Deserialize,
    serde::Serialize,
    schemars::JsonSchema,
    kube::CustomResource,
)]
#[kube(
    group = "example.com",
    version = "v1",
    kind = "Dog",
    plural = "dogs",
    namespaced
)]
pub struct DogSpec {
    id: String,
    name: String,
    breed: String,
    age: u32,
}

pub fn handle_dog_event(event: WatchEvent<Dog>) {
    match event {
        WatchEvent::Added(resource) => {
            info!("{} Added: {:?}", "Dog", resource.metadata.name);
        }
        WatchEvent::Modified(resource) => {
            info!("{} Modified: {:?}", "Dog", resource.metadata.name);
        }
        WatchEvent::Deleted(resource) => {
            info!("{} Deleted: {:?}", "Dog", resource.metadata.name);
        }
        _ => {}
    }
}
