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

fn convert_kube_type_to_dto(cat: Cat) -> CatDto {
    let uuid = match cat.status {
        Some(status) => status.uuid,
        None => None,
    };
    CatDto {
        uuid: uuid,
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
    config: Configuration,
}

#[derive(Debug, Error)]
enum OperatorError {
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error("Failed to process event: {0}")]
    FailedToProcessEvent(#[source] kube::Error),
    #[error("Failed to delete a cat: {0}")]
    FailedToDeleteCat(#[source] anyhow::Error),
    #[error("Failed to update a cat: {0}")]
    FailedToUpdateCat(#[source] anyhow::Error),
    #[error("Failed to create a cat: {0}")]
    FailedToCreateCat(#[source] anyhow::Error),
    #[error("Failed to get a cat: {0}")]
    FailedToGetCat(#[source] anyhow::Error),
    #[error("Failed to update status: {0}")]
    FailedToUpdateStatus(#[source] anyhow::Error),
    #[error("Failed to remove finalizer: {0}")]
    FailedToRemoveFinalizer(#[source] anyhow::Error),
    #[error("Failed to add finalizer: {0}")]
    FailedToAddFinalizer(#[source] anyhow::Error),
    #[error("Failed to check for drift: {0}")]
    FailedToCheckForDrift(#[source] anyhow::Error),
}

pub async fn handle(kube_client: Api<Cat>, config: Configuration) -> Result<()> {
    info!("Starting the controller");
    let controller = Controller::new(kube_client.clone(), watcher::Config::default());

    let extra_args = Arc::new(ExtraArgs {
        kube_client: kube_client.clone(),
        config: config,
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
    let config = &ctx.config;
    let kube_client = ctx.kube_client.clone();
    let cat = cat.as_ref();

    // Add default stauts if it's missing
    if cat.status.is_none() {
        let status = CatStatus {
            conditions: vec![],
            uuid: None,
            observed_generation: Some(0),
        };
        let mut cat_clone = cat.clone();
        cat_clone.status = Some(status);
        update_status(kube_client.clone(), cat_clone).await?;
    }

    // If the resource was marked for deletion, we need to delete it
    if cat.meta().deletion_timestamp.is_some() {
        if let Err(e) =
            handle_delete_cat_by_id(&config, &mut cat.clone(), kube_client.clone()).await
        {
            error!("Failed to delete cat: {:?}", e);
            return Err(OperatorError::FailedToDeleteCat(e));
        }
        return Ok(Action::requeue(Duration::from_secs(REQUEUE_AFTER_IN_SEC)));
    }

    // If the resource has no remote reference, meaning it's a new resource, so we need to create it
    // Otherwise, we need to check for drift
    match cat.clone().status.unwrap().uuid {
        Some(_) => {
            check_for_drift(&config, kube_client.clone(), &mut cat.clone()).await?;
            Ok(Action::requeue(Duration::from_secs(REQUEUE_AFTER_IN_SEC)))
        }
        None => {
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
            update_status(kube_client.clone(), cat.clone()).await?;
            handle_create_cat(&config, cat.clone(), kube_client).await?;
            Ok(Action::requeue(Duration::from_secs(REQUEUE_AFTER_IN_SEC)))
        }
    }
}

pub async fn check_for_drift(
    config: &Configuration,
    kubernetes_api: Api<Cat>,
    cat: &mut Cat,
) -> Result<()> {
    let dto = convert_kube_type_to_dto(cat.clone());
    let uuid = dto.uuid.clone().unwrap_or_default();

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
                        return update_status(kubernetes_api.clone(), cat_clone).await;
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

pub async fn handle_delete_cat_by_id(
    config: &Configuration,
    cat: &mut Cat,
    kubernetes_api: Api<Cat>,
) -> Result<()> {
    let dto = convert_kube_type_to_dto(cat.clone());
    let uuid = dto.uuid.clone().unwrap_or_default();

    if uuid.is_empty() {
        warn!("Cat has no uuid, cannot delete a cat by id. Skipping...");
        return Ok(());
    }

    delete_cat_by_id(&config, &uuid)
        .await
        .context("Failed to delete a cat by id")?;

    remove_finalizer(cat, kubernetes_api.clone()).await?;
    let condition = create_condition(
        "Deleted",
        "UnavailableDeleted",
        "Deleted the resource",
        "Resource has has deleted",
        cat.meta().generation,
    );
    let mut cat_clone = cat.clone();
    if let Some(status) = cat_clone.status.as_mut() {
        status.conditions.push(condition);
        status.observed_generation = cat.meta().generation;
    }
    return update_status(kubernetes_api.clone(), cat_clone).await;
}

pub async fn handle_update_cat_by_id(
    config: &Configuration,
    cat: &mut Cat,
    kubernetes_api: Api<Cat>,
) -> Result<()> {
    let dto = convert_kube_type_to_dto(cat.clone());
    let uuid = match dto.uuid.clone() {
        Some(uuid) => uuid,
        None => {
            warn!("Cat has no status, cannot update by id. Skipping...");
            return Ok(());
        }
    };

    update_cat_by_id(&config, &uuid, dto)
        .await
        .context("Failed to update a cat by id")?;

    let cat_name = cat.metadata.name.as_deref().unwrap_or_default();
    kubernetes_api
        .replace(cat_name, &PostParams::default(), &cat)
        .await
        .context("Failed to update a cat by id")?;

    info!("Updated a cat by id went successfully");
    Ok(())
}

pub async fn handle_create_cat(
    config: &Configuration,
    cat: Cat,
    kubernetes_api: Api<Cat>,
) -> Result<(), anyhow::Error> {
    let dto = convert_kube_type_to_dto(cat.clone());

    match create_cat(&config, dto.clone()).await {
        Ok(remote_cat) => match remote_cat.uuid {
            Some(uuid) => {
                let mut cat = cat;
                add_finalizer(&mut cat, kubernetes_api.clone()).await?;
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
                return update_status(kubernetes_api.clone(), cat_clone).await;
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

fn error_policy(_resource: Arc<Cat>, error: &OperatorError, _ctx: Arc<ExtraArgs>) -> Action {
    error!("Error processing event: {:?}", error);
    Action::requeue(Duration::from_secs(REQUEUE_AFTER_IN_SEC))
}
