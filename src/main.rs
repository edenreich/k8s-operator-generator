use env_logger::Env;
use k8s_operator::watch_resource;
use kube::{
    api::{Api, WatchParams},
    Client,
};
use log::info;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

    info!("Starting the operator...");

    let client = Client::try_default().await?;

    let watch_params = WatchParams::default().timeout(10);

    tokio::spawn(watch_resource::<k8s_operator::Cat>(
        Api::default_namespaced(client.clone()).clone(),
        watch_params.clone(),
        k8s_operator::handle_cat_event,
    ));
    tokio::spawn(watch_resource::<k8s_operator::Dog>(
        Api::default_namespaced(client.clone()).clone(),
        watch_params.clone(),
        k8s_operator::handle_dog_event,
    ));

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
