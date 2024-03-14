use crate::Dog;
use kube::api::WatchEvent;
use log::info;

pub fn handle_dog(event: WatchEvent<Dog>) {
    match event {
        WatchEvent::Added(resource) => {
            info!("{} Added: {:?}", "Dog", resource.metadata.name);
            todo!("TODO: Implement event handling");
        }
        WatchEvent::Modified(resource) => {
            info!("{} Modified: {:?}", "Dog", resource.metadata.name);
            todo!("TODO: Implement event handling");
        }
        WatchEvent::Deleted(resource) => {
            info!("{} Deleted: {:?}", "Dog", resource.metadata.name);
            todo!("TODO: Implement event handling");
        }
        _ => {}
    }
}
