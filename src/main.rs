use futures_util::stream::StreamExt;
use k8s_operator::{Cat, Dog};
use kube::{
    api::{Api, WatchEvent, WatchParams},
    Client,
};
use tokio::time::{sleep, Duration};
use env_logger::Env;
use log::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::default()
        .default_filter_or("info");
    env_logger::init_from_env(env);

    info!("Starting the operator...");

    let client = Client::try_default().await?;

    let cats: Api<Cat> = Api::default_namespaced(client.clone());
    let dogs: Api<Dog> = Api::default_namespaced(client.clone());

    let watch_params = WatchParams::default().timeout(10);

    tokio::spawn(watch_resource::<Cat>(cats.clone(), watch_params.clone(), handle_cat_event));
    tokio::spawn(watch_resource::<Dog>(dogs.clone(), watch_params.clone(), handle_dog_event));

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}

async fn watch_resource<T>(
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

fn handle_cat_event(event: WatchEvent<Cat>) {
    match event {
        WatchEvent::Added(cat) => {
            info!("Cat Added: {:?}", cat.metadata.name);
            // 1. Check if the external Cat has already been created, if not, create it
            // 2. if the Cat has already been created, check if the external Cat is the same as the internal Cat
            // 3. If the external Cat is different, trigger a modified event
        }
        WatchEvent::Modified(cat) => {
            info!("Cat Modified: {:?}", cat.metadata.name);
            // 1. Compare the external Cat resource with the Cat that was modified
            // 2. If the cat was modified, make the API call to update the cat
        }
        WatchEvent::Deleted(cat) => {
            info!("Cat Deleted: {:?}", cat.metadata.name);
            // 1. Check if the cat has already been deleted, if not, delete it
        }
        _ => {}
    }
}

fn handle_dog_event(event: WatchEvent<Dog>) {
    match event {
        WatchEvent::Added(dog) => {
            info!("Dog Added: {:?}", dog.metadata.name);
            // 1. Check if the external Dog has already been created, if not, create it
            // 2. if the Dog has already been created, check if the external Dog is the same as the internal Dog
            // 3. If the external Dog is different, trigger a modified event
        }
        WatchEvent::Modified(dog) => {
            info!("Dog Modified: {:?}", dog.metadata.name);
            // 1. Compare the external Dog resource with the Dog that was modified
            // 2. If the Dog was modified, make the API call to update the Dog
        }
        WatchEvent::Deleted(dog) => {
            info!("Dog Deleted: {:?}", dog.metadata.name);
            // 1. Check if the Dog has already been deleted, if not, delete it
        }
        _ => {}
    }
}
