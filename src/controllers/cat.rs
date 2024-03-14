use crate::Cat;
use kube::api::WatchEvent;
use log::info;

pub fn handle_cat(event: WatchEvent<Cat>) {
    match event {
        WatchEvent::Added(resource) => {
            info!("{} Added: {:?}", "Cat", resource.metadata.name);
            todo!("TODO: Implement event handling");
        }
        WatchEvent::Modified(resource) => {
            info!("{} Modified: {:?}", "Cat", resource.metadata.name);
            todo!("TODO: Implement event handling");
        }
        WatchEvent::Deleted(resource) => {
            info!("{} Deleted: {:?}", "Cat", resource.metadata.name);
            todo!("TODO: Implement event handling");
        }
        _ => {}
    }
}
