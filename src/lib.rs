use futures_util::stream::StreamExt;
use kube::api::{Api, WatchEvent, WatchParams};
use log::error;
use tokio::time::{sleep, Duration};

pub mod controllers;

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
    id: Option<String>,
    name: String,
    breed: String,
    age: u32,
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
    id: Option<String>,
    name: String,
    breed: String,
    age: u32,
}
