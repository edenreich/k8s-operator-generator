use futures_util::stream::StreamExt;
use k8s_openapi::api::core::v1::{Event, ObjectReference};
use kube::api::{Api, Patch, PatchParams, PostParams, WatchEvent, WatchParams};
use kube::core::CustomResourceExt;
use kube::{CustomResource, Resource, ResourceExt};
use log::{debug, error, info};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::{sleep, Duration};

pub mod controllers;

pub async fn watch_resource<T>(
    api: Api<T>,
    watch_params: WatchParams,
    handler: fn(WatchEvent<T>, Api<T>),
) -> anyhow::Result<()>
where
    T: Clone + core::fmt::Debug + DeserializeOwned + 'static,
{
    let mut stream = api.watch(&watch_params, "0").await?.boxed();
    loop {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => handler(event, api.clone()),
                Err(e) => error!("Error watching resource: {:?}", e),
            }
        }
        sleep(Duration::from_secs(1)).await;
        stream = api.watch(&watch_params, "0").await?.boxed();
    }
}

pub async fn add_finalizer<T>(resource: &mut T, api: Api<T>)
where
    T: Clone
        + Serialize
        + DeserializeOwned
        + Resource
        + CustomResourceExt
        + core::fmt::Debug
        + 'static,
{
    let finalizer = String::from("finalizers.example.com");
    let finalizers = resource.meta_mut().finalizers.get_or_insert_with(Vec::new);
    if finalizers.contains(&finalizer) {
        debug!("Finalizer already exists");
        return;
    }
    finalizers.push(finalizer);
    let resource_name = resource.meta_mut().name.clone().unwrap();
    let resource_clone = resource.clone();
    let patch = Patch::Merge(&resource_clone);
    let patch_params = PatchParams {
        field_manager: resource.meta_mut().name.clone(),
        ..Default::default()
    };
    match api.patch(&resource_name, &patch_params, &patch).await {
        Ok(_) => debug!("Finalizer added successfully"),
        Err(e) => debug!("Failed to add finalizer: {:?}", e),
    };
}

pub async fn remove_finalizer<T>(resource: &mut T, api: Api<T>)
where
    T: Clone
        + Serialize
        + DeserializeOwned
        + Resource
        + CustomResourceExt
        + core::fmt::Debug
        + 'static,
{
    let finalizer = String::from("finalizers.example.com");
    if let Some(finalizers) = &mut resource.meta_mut().finalizers {
        if finalizers.contains(&finalizer) {
            finalizers.retain(|f| f != &finalizer);
            let patch = json ! ({ "metadata" : { "finalizers" : finalizers } });
            let patch = Patch::Merge(&patch);
            let patch_params = PatchParams {
                field_manager: resource.meta_mut().name.clone(),
                ..Default::default()
            };
            match api
                .patch(
                    &resource.clone().meta_mut().name.clone().unwrap(),
                    &patch_params,
                    &patch,
                )
                .await
            {
                Ok(_) => debug!("Finalizer removed successfully"),
                Err(e) => debug!("Failed to remove finalizer: {:?}", e),
            };
        }
    }
}

pub async fn add_event<T>(kind: String, resource: &mut T, event: String)
where
    T: CustomResourceExt
        + Clone
        + Serialize
        + DeserializeOwned
        + Resource
        + core::fmt::Debug
        + 'static,
{
    let kube_client = kube::Client::try_default().await.unwrap();
    let namespace = resource.namespace().clone().unwrap_or_default();
    let api: Api<Event> = Api::namespaced(kube_client.clone(), &namespace);
    let resource_ref = ObjectReference {
        kind: Some(kind),
        namespace: resource.namespace().clone(),
        name: Some(resource.meta().name.clone().unwrap()),
        uid: resource.uid().clone(),
        ..Default::default()
    };
    let new_event = Event {
        metadata: Default::default(),
        involved_object: resource_ref,
        reason: Some("ExampleReason".into()),
        message: Some(event.into()),
        type_: Some("Normal".into()),
        ..Default::default()
    };
    match api.create(&PostParams::default(), &new_event).await {
        Ok(_) => debug!("Event added successfully"),
        Err(e) => debug!("Failed to add event: {:?}", e),
    };
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, CustomResource)]
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

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, CustomResource)]
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
