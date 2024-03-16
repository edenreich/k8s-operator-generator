use crate::{add_event, Cat};
use crate::{add_finalizer, remove_finalizer};
use kube::api::{Api, WatchEvent};
use kube::Resource;
use log::info;
use pets_api_client::apis::default_api::DefaultApi;

pub async fn handle_cat(event: WatchEvent<Cat>, api: Api<Cat>, client: &DefaultApi) {
    let kind = Cat::kind(&());
    let kind_str = kind.to_string();

    let (mut cat, event_type) = match event {
        WatchEvent::Added(mut cat) => {
            if cat.metadata.deletion_timestamp.is_none() {
                add_finalizer(&mut cat, api.clone()).await;
                client.create_cat(&cat).await;
            } else {
                client.delete_cat(&cat).await;
                remove_finalizer(&mut cat, api.clone()).await;
            }
            (cat, "Added")
        }
        WatchEvent::Modified(mut cat) => {
            client.update_cat(&cat).await;
            (cat, "Modified")
        }
        WatchEvent::Deleted(mut cat) => {
            client.delete_cat(&cat).await;
            (cat, "Deleted")
        }
        WatchEvent::Bookmark(bookmark) => {
            info!("Cat Bookmark: {:?}", bookmark.metadata.resource_version);
            return;
        }
        _ => {
            info!("Cat Unknown event {:?}", event);
            return;
        }
    };

    add_event(
        kind_str.clone(),
        &mut cat,
        event_type.into(),
        kind_str.clone(),
        format!("Cat Resource {} Remotely", event_type),
    )
    .await;

    info!(
        "Cat {}: {:?} {:?}",
        event_type, cat.metadata.name, cat.metadata.finalizers
    );
}
