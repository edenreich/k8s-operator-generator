use crate::Dog;
use kube::api::WatchEvent;
use log::info;

pub fn handle_dog(event: WatchEvent<Dog>) {
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
