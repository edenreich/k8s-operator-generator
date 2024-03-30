use env_logger::Env;
use k8s_operator::{crds_exist, watch_resource};
use kube::{
    api::{Api, WatchParams},
    Client,
};
use log::{error, info};
use openapi::apis::configuration::Configuration;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

    info!("Starting the operator...");

    let client = Client::try_default().await?;

    let watch_params = WatchParams::default().timeout(10);

    if !crds_exist(client.clone(), "example.com").await? {
        error!("No CRD's found from the group. Please install the CRD's first. Exiting...");
        return Ok(());
    }

    let config = Arc::new(Configuration {
        base_path: "http://localhost:8080".to_string(),
        user_agent: None,
        client: reqwest::Client::new(),
        ..Configuration::default()
    });

    tokio::spawn(watch_resource::<k8s_operator::types::cat::Cat>(
        Arc::clone(&config),
        Api::default_namespaced(client.clone()).clone(),
        watch_params.clone(),
        |config, event, kubernetes_api| {
            tokio::spawn(k8s_operator::controllers::cats::handle(
                config,
                event,
                kubernetes_api,
            ));
        },
    ));

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
