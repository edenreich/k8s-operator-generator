use crate::Dog;
use crate::{add_event, add_finalizer, remove_finalizer};
use kube::api::{Api, WatchEvent};
use log::info;

pub async fn handle_dog(event: WatchEvent<Dog>, api: Api<Dog>) {
    match event {
        WatchEvent::Added(mut dog) => {
            if dog.metadata.deletion_timestamp.is_some() {
                info!(
                    "{} Sending API call to delete the remote resource and wait for response: {:?}",
                    "Dog", dog.metadata.name
                );
                remove_finalizer(&mut dog, api.clone()).await;
            } else {
                add_finalizer(&mut dog, api.clone()).await;
                info!(
                    "{} Added: {:?} {:?}",
                    "Dog", dog.metadata.name, dog.metadata.finalizers
                )
            }
        }
        WatchEvent::Modified(dog) => {
            info!(
                "{} Modified: {:?} {:?}",
                "Dog", dog.metadata.name, dog.metadata.finalizers
            );
        }
        WatchEvent::Deleted(dog) => {
            info!(
                "{} Deleted: {:?} {:?}",
                "Dog", dog.metadata.name, dog.metadata.finalizers
            );
        }
        _ => {
            info!("{} Unknown event", "Dog");
        }
    }
}
