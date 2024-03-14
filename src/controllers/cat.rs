use crate::Cat;
use kube::api::WatchEvent;
use log::info;

pub fn handle_cat(event: WatchEvent<Cat>) {
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
