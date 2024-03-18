use futures_util::stream::StreamExt;
use k8s_openapi::api::core::v1::{Event, EventSource, ObjectReference};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use k8s_openapi::chrono;
use kube::api::{Api, ObjectMeta, Patch, PatchParams, PostParams, WatchEvent, WatchParams};
use kube::core::CustomResourceExt;
use kube::{Resource, ResourceExt};
use log::{debug, error, info};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use tokio::time::{sleep, Duration};

pub mod types;

pub mod controllers;

pub async fn watch_resource<T>(
    kubernetes_api: Api<T>,
    watch_params: WatchParams,
    handler: fn(WatchEvent<T>, Api<T>),
) -> anyhow::Result<()>
where
    T: Clone + core::fmt::Debug + DeserializeOwned + 'static,
{
    let mut stream = kubernetes_api.watch(&watch_params, "0").await?.boxed();
    loop {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => handler(event, kubernetes_api.clone()),
                Err(e) => error!("Error watching resource: {:?}", e),
            }
        }
        sleep(Duration::from_secs(1)).await;
        stream = kubernetes_api.watch(&watch_params, "0").await?.boxed();
    }
}

pub async fn add_finalizer<T>(resource: &mut T, kubernetes_api: Api<T>)
where
    T: Clone
        + Serialize
        + DeserializeOwned
        + Resource
        + CustomResourceExt
        + core::fmt::Debug
        + 'static,
{
    let finalizer = String::from(format!("finalizers.{}", "example.com"));
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
    match kubernetes_api
        .patch(&resource_name, &patch_params, &patch)
        .await
    {
        Ok(_) => debug!("Finalizer added successfully"),
        Err(e) => debug!("Failed to add finalizer: {:?}", e),
    };
}

pub async fn remove_finalizer<T>(resource: &mut T, kubernetes_api: Api<T>)
where
    T: Clone
        + Serialize
        + DeserializeOwned
        + Resource
        + CustomResourceExt
        + core::fmt::Debug
        + 'static,
{
    let finalizer = String::from(format!("finalizers.{}", "example.com"));
    if let Some(finalizers) = &mut resource.meta_mut().finalizers {
        if finalizers.contains(&finalizer) {
            finalizers.retain(|f| f != &finalizer);
            let patch = json ! ({ "metadata" : { "finalizers" : finalizers } });
            let patch = Patch::Merge(&patch);
            let patch_params = PatchParams {
                field_manager: resource.meta_mut().name.clone(),
                ..Default::default()
            };
            match kubernetes_api
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

pub async fn add_event<T>(kind: String, resource: &mut T, reason: &str, from: &str, message: &str)
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
    let kubernetes_api: Api<Event> = Api::namespaced(kube_client.clone(), &namespace);
    let resource_ref = ObjectReference {
        kind: Some(kind),
        namespace: resource.namespace().clone(),
        name: Some(resource.meta().name.clone().unwrap()),
        uid: resource.uid().clone(),
        ..Default::default()
    };
    let timestamp = chrono::Utc::now().to_rfc3339();
    let event_name = format!("{}-{}", resource.meta().name.clone().unwrap(), timestamp);
    let new_event = Event {
        metadata: ObjectMeta {
            name: Some(event_name),
            ..Default::default()
        },
        involved_object: resource_ref,
        reason: Some(reason.into()),
        message: Some(message.into()),
        type_: Some("Normal".into()),
        source: Some(EventSource {
            component: Some(String::from(from)),
            ..Default::default()
        }),
        first_timestamp: Some(Time(chrono::Utc::now())),
        last_timestamp: Some(Time(chrono::Utc::now())),
        ..Default::default()
    };
    match kubernetes_api
        .create(&PostParams::default(), &new_event)
        .await
    {
        Ok(_) => debug!("Event added successfully"),
        Err(e) => debug!("Failed to add event: {:?}", e),
    };
}

pub async fn change_status<T>(resource: &mut T, kubernetes_api: Api<T>, field: &str, value: String)
where
    T: Clone
        + Serialize
        + DeserializeOwned
        + Resource
        + CustomResourceExt
        + core::fmt::Debug
        + 'static,
{
    let name = resource.meta().name.clone().unwrap();
    let mut resource_json: serde_json::Value =
        serde_json::to_value(&resource).expect("Failed to serialize resource");
    resource_json["status"][field] = serde_json::json!(value);
    let new_resource: T =
        serde_json::from_value(resource_json).expect("Failed to deserialize resource");
    let resource_bytes = serde_json::to_vec(&new_resource).expect("Failed to serialize resource");
    match kubernetes_api
        .replace_status(&name, &PostParams::default(), resource_bytes)
        .await
    {
        Ok(_) => info!("Status updated successfully for {}", name),
        Err(e) => info!("Failed to update status for {}: {:?}", name, e),
    };
}
