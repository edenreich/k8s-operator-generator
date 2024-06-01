use anyhow::{Context, Result};
use futures::StreamExt;
use kube::api::{Api, PostParams, Resource};
use thiserror::Error;

use kube_runtime::{controller::Action, watcher, Controller};
use log::{error, info, warn};
use std::{sync::Arc, time::Duration};

use openapi::{
    apis::{
        cats_api::{create_cat, delete_cat_by_id, get_cat_by_id, update_cat_by_id},
        configuration::Configuration,
    },
    models::Cat as CatDto,
};

use crate::types::cat::{Cat, CatSpec, CatStatus};
use crate::{add_finalizer, create_condition, remove_finalizer, update_status};

const REQUEUE_AFTER_IN_SEC: u64 = 30;
const API_URL: &str = "http://localhost:8080";
const API_USER_AGENT: &str = "k8s-operator";

fn convert_uuid_to_string(uuid: Option<uuid::Uuid>) -> Option<String> {
    match uuid {
        Some(uuid) => Some(uuid.to_string()),
        None => None,
    }
}

fn convert_string_to_uuid(uuid: Option<String>) -> Option<uuid::Uuid> {
    match uuid {
        Some(uuid) => match uuid::Uuid::parse_str(&uuid) {
            Ok(uuid) => Some(uuid),
            Err(_) => None,
        },
        None => None,
    }
}

fn convert_kube_type_to_dto(cat: Cat) -> CatDto {
    let uuid = match cat.status {
        Some(status) => convert_string_to_uuid(status.uuid),
        None => None,
    };
    CatDto {
        uuid,
        name: cat.spec.name,
        breed: cat.spec.breed,
        age: cat.spec.age,
    }
}

fn convert_dto_to_kube_type(cat: CatDto) -> CatSpec {
    CatSpec {
        name: cat.name,
        breed: cat.breed,
        age: cat.age,
    }
}

struct ExtraArgs {
    kube_client: Api<Cat>,
}

#[derive(Debug, Error)]
enum OperatorError {
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    // #[error("Failed to process event: {0}")]
    // FailedToProcessEvent(#[source] kube::Error),
    #[error("Failed to delete a cat: {0}")]
    FailedToDeleteCat(#[source] anyhow::Error),
    // #[error("Failed to update a cat: {0}")]
    // FailedToUpdateCat(#[source] anyhow::Error),
    // #[error("Failed to create a cat: {0}")]
    // FailedToCreateCat(#[source] anyhow::Error),
    // #[error("Failed to get a cat: {0}")]
    // FailedToGetCat(#[source] anyhow::Error),
    #[error("Failed to update status: {0}")]
    FailedToUpdateStatus(#[source] anyhow::Error),
    // #[error("Failed to remove finalizer: {0}")]
    // FailedToRemoveFinalizer(#[source] anyhow::Error),
    // #[error("Failed to add finalizer: {0}")]
    // FailedToAddFinalizer(#[source] anyhow::Error),
    // #[error("Failed to check for drift: {0}")]
    // FailedToCheckForDrift(#[source] anyhow::Error),
}

pub async fn handle(kube_client: Api<Cat>) -> Result<()> {
    info!("Starting the controller");
    let controller = Controller::new(kube_client.clone(), watcher::Config::default());

    let extra_args = Arc::new(ExtraArgs {
        kube_client: kube_client.clone(),
    });

    info!("Running the controller");
    controller
        .run(reconcile, error_policy, extra_args)
        .for_each(|res| async {
            match res {
                Ok(action) => info!("Reconciliation was successful, action: {:?}", action),
                Err(e) => error!("Error reconciling: {:?}", e),
            }
        })
        .await;

    info!("Cat Controller has stopped");
    Ok(())
}

async fn reconcile(cat: Arc<Cat>, ctx: Arc<ExtraArgs>) -> Result<Action, OperatorError> {
    let kube_client = ctx.kube_client.clone();
    let mut cat = cat.as_ref().clone();
    let uuid = match cat.clone().status {
        Some(status) => status.uuid.unwrap_or_default(),
        None => "".to_string(),
    };

    // Add default stauts if it's missing
    if cat.status.is_none() {
        add_default_status(&kube_client, &mut cat).await?;
    }

    // If the resource was marked for deletion, we need to delete it
    if cat.meta().deletion_timestamp.is_some() {
        handle_delete(&kube_client, &mut cat, &uuid).await?;
    }

    // If uuid is empty, we need to create a new resource
    if uuid.is_empty() {
        let condition = create_condition(
            "Creating",
            "ProgressingCreating",
            "Creating the resource",
            "Resource is being created",
            cat.meta().generation,
        );
        if let Some(status) = cat.clone().status.as_mut() {
            status.conditions.push(condition);
            status.observed_generation = cat.meta().generation;
        }
        update_status(&kube_client, cat.clone()).await?;

        handle_create(&kube_client, &mut cat.clone()).await?;
    } else {
        // If the resource was updated in kubernetes, we need to update the remote resource
        if cat.meta().generation != cat.status.as_ref().unwrap().observed_generation {
            handle_update(&kube_client, &mut cat, &uuid).await?;
        }
    }

    check_for_drift(&kube_client, &mut cat).await?;

    Ok(Action::requeue(Duration::from_secs(REQUEUE_AFTER_IN_SEC)))
}

async fn get_client_config() -> Result<Configuration> {
    let config = Configuration {
        base_path: API_URL.to_string(),
        client: reqwest::Client::new(),
        user_agent: Some(API_USER_AGENT.to_string()),
        bearer_access_token: Some(std::env::var("ACCESS_TOKEN").unwrap_or_default()),
        ..Default::default()
    };
    Ok(config)
}

async fn add_default_status(kube_client: &Api<Cat>, cat: &mut Cat) -> Result<(), OperatorError> {
    let status = CatStatus {
        conditions: vec![],
        uuid: None,
        observed_generation: Some(0),
    };
    cat.status = Some(status);
    update_status(kube_client, cat.clone())
        .await
        .map_err(OperatorError::FailedToUpdateStatus)
}

pub async fn check_for_drift(kube_client: &Api<Cat>, cat: &mut Cat) -> Result<()> {
    let dto = convert_kube_type_to_dto(cat.clone());
    let uuid = convert_uuid_to_string(dto.uuid).unwrap();
    let config = get_client_config().await?;

    if dto.uuid.is_none() {
        warn!("Cat has no status, cannot get by id or check for drift. Skipping...");
        return Ok(());
    }

    match get_cat_by_id(&config, &uuid).await {
        Ok(dto) => {
            let remote_cat = convert_dto_to_kube_type(dto);
            if remote_cat != cat.spec {
                let current_cat_dto = convert_kube_type_to_dto(cat.clone());
                warn!("Cat has drifted remotely, sending an update to remote...");
                match update_cat_by_id(&config, &uuid, current_cat_dto).await {
                    Ok(_) => {
                        info!("Cat updated successfully");
                        let condition = create_condition(
                            "Updated",
                            "AvailableUpdated",
                            "Updated the resource",
                            "Resource has been updated",
                            cat.meta().generation,
                        );
                        let mut cat_clone = cat.clone();
                        if let Some(status) = cat_clone.status.as_mut() {
                            status.conditions.push(condition);
                            status.observed_generation = cat.meta().generation;
                        }
                        update_status(kube_client, cat_clone).await?
                    }
                    Err(e) => {
                        error!("Failed to update Cat: {:?}", e);
                        return Err(anyhow::anyhow!("Failed to update cat: {:?}", e));
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to get Cat: {:?}", e);
            return Err(anyhow::anyhow!("Failed to get cat: {:?}", e));
        }
    }

    Ok(())
}

fn error_policy(_resource: Arc<Cat>, error: &OperatorError, _ctx: Arc<ExtraArgs>) -> Action {
    error!("Error processing event: {:?}", error);
    Action::requeue(Duration::from_secs(REQUEUE_AFTER_IN_SEC))
}

async fn handle_delete(
    kube_client: &Api<Cat>,
    cat: &mut Cat,
    uuid: &str,
) -> Result<(), OperatorError> {
    let config = get_client_config().await?;
    if uuid.is_empty() {
        warn!("Cat has no status, cannot delete by id. Skipping...");
        return Ok(());
    }

    if let Err(e) = delete_cat_by_id(&config, uuid).await {
        error!("Failed to delete cat: {:?}", e);
        return Err(OperatorError::FailedToDeleteCat(e.into()));
    }

    remove_finalizer(cat, kube_client.clone()).await?;
    info!("Successfully deleted cat");
    Ok(())
}

pub async fn handle_update(kubernetes_api: &Api<Cat>, cat: &mut Cat, uuid: &str) -> Result<()> {
    let dto = convert_kube_type_to_dto(cat.clone());
    let config = get_client_config().await?;

    if uuid.is_empty() {
        return Err(anyhow::anyhow!("uuid is empty"));
    }

    update_cat_by_id(&config, uuid, dto)
        .await
        .context("Failed to update a cat by id")?;

    let cat_name = cat.metadata.name.as_deref().unwrap_or_default();
    kubernetes_api
        .replace(cat_name, &PostParams::default(), cat)
        .await
        .context("Failed to update a cat by id")?;

    info!("Updated a cat by id went successfully");
    Ok(())
}

pub async fn handle_create(kube_client: &Api<Cat>, cat: &mut Cat) -> Result<(), anyhow::Error> {
    let dto = convert_kube_type_to_dto(cat.clone());
    let config = get_client_config().await?;

    match create_cat(&config, dto.clone()).await {
        Ok(remote_cat) => match remote_cat.uuid {
            Some(uuid) => {
                let uuid = convert_uuid_to_string(Some(uuid)).unwrap();
                add_finalizer(cat, kube_client.clone()).await?;
                let condition = create_condition(
                    "Created",
                    "AvailableCreated",
                    "Created the resource",
                    "Resource has been created",
                    cat.meta().generation,
                );
                let mut cat_clone = cat.clone();
                if let Some(status) = cat_clone.status.as_mut() {
                    status.conditions.push(condition);
                    status.uuid = Some(uuid);
                    status.observed_generation = cat.meta().generation;
                }
                update_status(kube_client, cat_clone).await
            }
            None => {
                warn!("Remote cat has no uuid, cannot update status");
                Ok(())
            }
        },
        Err(e) => {
            error!("Failed to create a new cat: {:?}", e);
            Err(anyhow::anyhow!("Failed to create a new cat: {:?}", e))
        }
    }
}
