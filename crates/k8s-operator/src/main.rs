use anyhow::{Context, Result};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{Api, PostParams},
    runtime::wait::{await_condition, conditions},
    Client as KubeClient, CustomResourceExt,
};
use log::{error, info, warn};
use tokio::time::timeout;
use warp::Filter;

async fn deploy_crd(
    kube_client: Api<CustomResourceDefinition>,
    crd: CustomResourceDefinition,
) -> Result<()> {
    let crd_name = crd
        .metadata
        .name
        .clone()
        .unwrap_or_else(|| String::from("Unnamed CRD"));
    info!("Deploying CRD: {}", crd_name);

    let result = kube_client.create(&PostParams::default(), &crd).await;

    match result {
        core::result::Result::Ok(_) => info!("Successfully created CRD: {}", crd_name),
        Err(kube::Error::Api(ae)) if ae.code == 409 => {
            if kube_client
                .replace(&crd_name, &PostParams::default(), &crd)
                .await
                .is_ok()
            {
                info!("Successfully updated CRD: {}", crd_name);
            } else {
                warn!("Failed to update CRD, already exists: {}", crd_name);
            }
        }
        Err(_) => error!("Failed to create CRD: {}", crd_name),
    }

    Ok(())
}

async fn wait_for_crd(kube_client: Api<CustomResourceDefinition>, crd_name: &str) -> Result<()> {
    info!(
        "Waiting for the api-server to accept the CRD of {}...",
        crd_name
    );

    let establish = await_condition(
        kube_client.clone(),
        crd_name,
        conditions::is_crd_established(),
    );
    let _ = timeout(std::time::Duration::from_secs(10), establish).await?;
    info!("CRD of {} is established.", crd_name);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    info!("Starting operator...");

    let kube_client = KubeClient::try_default().await?;
    let kube_client_api: Api<CustomResourceDefinition> = Api::all(kube_client.clone());

    if std::env::var("INSTALL_CRDS")
        .unwrap_or_default()
        .to_lowercase()
        == "true"
    {
        info!("INSTALL_CRDS is set to true. Deploying CRDs...");

        let crds = vec![k8s_operator::types::cat::Cat::crd()];

        for crd in crds {
            deploy_crd(kube_client_api.clone(), crd).await?;
        }
    }

    let controllers_crds = vec![format!("cats.example.com")];
    for controller_crd in controllers_crds {
        if let Err(e) = wait_for_crd(kube_client_api.clone(), &controller_crd).await {
            error!("Error waiting for CRD {}: {}", &controller_crd, e);
        }
    }

    let controllers = vec![k8s_operator::controllers::cats::handle];
    for controller in controllers {
        let _ = controller(Api::namespaced(kube_client.clone(), "default")).await;
    }

    tokio::spawn(async {
        let liveness_route = warp::path!("healthz")
            .map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));

        let readiness_route = warp::path!("readyz")
            .map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));

        let health_routes = liveness_route.or(readiness_route);

        warp::serve(health_routes).run(([0, 0, 0, 0], 8000)).await;
    });

    tokio::signal::ctrl_c()
        .await
        .context("Failed to listen for Ctrl+C")?;
    info!("Termination signal received. Shutting down.");

    Ok(())
}
