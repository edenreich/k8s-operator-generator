use env_logger::Env;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use k8s_operator::watch_resource;
use kube::{
    api::{Api, ListParams, WatchParams},
    Client,
};
use log::{error, info};
use tokio::time::{sleep, Duration};

async fn check_any_crd_from_group(client: Client, group: &str) -> anyhow::Result<bool> {
    let crds: Api<CustomResourceDefinition> = Api::all(client);
    let lp = ListParams::default();
    let crd_list = crds.list(&lp).await?;

    Ok(crd_list.items.iter().any(|crd| crd.spec.group == group))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

    info!("Starting the operator...");

    let client = Client::try_default().await?;

    let watch_params = WatchParams::default().timeout(10);

    if !check_any_crd_from_group(client.clone(), "example.com").await? {
        error!("No CRD's found from the group. Please install the CRD's first. Exiting...");
        return Ok(());
    }

    tokio::spawn(watch_resource::<k8s_operator::Cat>(
        Api::default_namespaced(client.clone()).clone(),
        watch_params.clone(),
        k8s_operator::controllers::cat::handle_cat,
    ));
    tokio::spawn(watch_resource::<k8s_operator::Dog>(
        Api::default_namespaced(client.clone()).clone(),
        watch_params.clone(),
        k8s_operator::controllers::dog::handle_dog,
    ));

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
