use anyhow::{Context, Result};
use futures::StreamExt;
use kube::api::{Api, PostParams, Resource};
use thiserror::Error;

use kube_runtime::{controller::Action, watcher, Controller};
use log::{error, info, warn};
use std::{sync::Arc, time::Duration};

use openapi::{
    apis::{
        configuration::Configuration,
        dogs_api::{create_dog, delete_dog_by_id, get_dog_by_id, update_dog_by_id},
    },
    models::Dog as DogDto,
};

use crate::types::dog::{Dog, DogSpec, DogStatus};
use crate::{add_finalizer, create_condition, remove_finalizer, update_status};

const REQUEUE_AFTER_IN_SEC: u64 = 30;
const API_URL: &str = "http://localhost:8080";
const API_USER_AGENT: &str = "k8s-operator";

fn convert_kube_type_to_dto(dog: Dog) -> DogDto {
    let uuid = match dog.status {
        Some(status) => status.uuid,
        None => None,
    };
    DogDto {
        uuid,
        name: dog.spec.name,
        breed: dog.spec.breed,
        age: dog.spec.age,
    }
}

fn convert_dto_to_kube_type(dog: DogDto) -> DogSpec {
    DogSpec {
        name: dog.name,
        breed: dog.breed,
        age: dog.age,
    }
}

struct ExtraArgs {
    kube_client: Api<Dog>,
}

#[derive(Debug, Error)]
enum OperatorError {
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    // #[error("Failed to process event: {0}")]
    // FailedToProcessEvent(#[source] kube::Error),
    #[error("Failed to delete a dog: {0}")]
    FailedToDeleteDog(#[source] anyhow::Error),
    // #[error("Failed to update a dog: {0}")]
    // FailedToUpdateDog(#[source] anyhow::Error),
    // #[error("Failed to create a dog: {0}")]
    // FailedToCreateDog(#[source] anyhow::Error),
    // #[error("Failed to get a dog: {0}")]
    // FailedToGetDog(#[source] anyhow::Error),
    #[error("Failed to update status: {0}")]
    FailedToUpdateStatus(#[source] anyhow::Error),
    // #[error("Failed to remove finalizer: {0}")]
    // FailedToRemoveFinalizer(#[source] anyhow::Error),
    // #[error("Failed to add finalizer: {0}")]
    // FailedToAddFinalizer(#[source] anyhow::Error),
    // #[error("Failed to check for drift: {0}")]
    // FailedToCheckForDrift(#[source] anyhow::Error),
}

pub async fn handle(kube_client: Api<Dog>) -> Result<()> {
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

    info!("Dog Controller has stopped");
    Ok(())
}

async fn reconcile(dog: Arc<Dog>, ctx: Arc<ExtraArgs>) -> Result<Action, OperatorError> {
    let kube_client = ctx.kube_client.clone();
    let mut dog = dog.as_ref().clone();
    let uuid = match dog.clone().status {
        Some(status) => status.uuid.unwrap_or_default(),
        None => "".to_string(),
    };

    // Add default stauts if it's missing
    if dog.status.is_none() {
        add_default_status(&kube_client, &mut dog).await?;
    }

    // If the resource was marked for deletion, we need to delete it
    if dog.meta().deletion_timestamp.is_some() {
        handle_delete(&kube_client, &mut dog, &uuid).await?;
    }

    // If uuid is empty, we need to create a new resource
    if uuid.is_empty() {
        let condition = create_condition(
            "Creating",
            "ProgressingCreating",
            "Creating the resource",
            "Resource is being created",
            dog.meta().generation,
        );
        if let Some(status) = dog.clone().status.as_mut() {
            status.conditions.push(condition);
            status.observed_generation = dog.meta().generation;
        }
        update_status(&kube_client, dog.clone()).await?;

        handle_create(&kube_client, &mut dog.clone()).await?;
    } else {
        // If the resource was updated in kubernetes, we need to update the remote resource
        if dog.meta().generation != dog.status.as_ref().unwrap().observed_generation {
            handle_update(&kube_client, &mut dog, &uuid).await?;
        }
    }

    check_for_drift(&kube_client, &mut dog).await?;

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

async fn add_default_status(kube_client: &Api<Dog>, dog: &mut Dog) -> Result<(), OperatorError> {
    let status = DogStatus {
        conditions: vec![],
        uuid: None,
        observed_generation: Some(0),
    };
    dog.status = Some(status);
    update_status(kube_client, dog.clone())
        .await
        .map_err(OperatorError::FailedToUpdateStatus)
}

pub async fn check_for_drift(kube_client: &Api<Dog>, dog: &mut Dog) -> Result<()> {
    let dto = convert_kube_type_to_dto(dog.clone());
    let uuid = dto.uuid.clone().unwrap_or_default();
    let config = get_client_config().await?;

    if dto.uuid.is_none() {
        warn!("Dog has no status, cannot get by id or check for drift. Skipping...");
        return Ok(());
    }

    match get_dog_by_id(&config, &uuid).await {
        Ok(dto) => {
            let remote_dog = convert_dto_to_kube_type(dto);
            if remote_dog != dog.spec {
                let current_dog_dto = convert_kube_type_to_dto(dog.clone());
                warn!("Dog has drifted remotely, sending an update to remote...");
                match update_dog_by_id(&config, &uuid, current_dog_dto).await {
                    Ok(_) => {
                        info!("Dog updated successfully");
                        let condition = create_condition(
                            "Updated",
                            "AvailableUpdated",
                            "Updated the resource",
                            "Resource has been updated",
                            dog.meta().generation,
                        );
                        let mut dog_clone = dog.clone();
                        if let Some(status) = dog_clone.status.as_mut() {
                            status.conditions.push(condition);
                            status.observed_generation = dog.meta().generation;
                        }
                        update_status(kube_client, dog_clone).await?
                    }
                    Err(e) => {
                        error!("Failed to update Dog: {:?}", e);
                        return Err(anyhow::anyhow!("Failed to update dog: {:?}", e));
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to get Dog: {:?}", e);
            return Err(anyhow::anyhow!("Failed to get dog: {:?}", e));
        }
    }

    Ok(())
}

fn error_policy(_resource: Arc<Dog>, error: &OperatorError, _ctx: Arc<ExtraArgs>) -> Action {
    error!("Error processing event: {:?}", error);
    Action::requeue(Duration::from_secs(REQUEUE_AFTER_IN_SEC))
}

async fn handle_delete(
    kube_client: &Api<Dog>,
    dog: &mut Dog,
    uuid: &str,
) -> Result<(), OperatorError> {
    let config = get_client_config().await?;
    if uuid.is_empty() {
        warn!("Dog has no status, cannot delete by id. Skipping...");
        return Ok(());
    }

    if let Err(e) = delete_dog_by_id(&config, uuid).await {
        error!("Failed to delete dog: {:?}", e);
        return Err(OperatorError::FailedToDeleteDog(e.into()));
    }

    remove_finalizer(dog, kube_client.clone()).await?;
    info!("Successfully deleted dog");
    Ok(())
}

pub async fn handle_update(kubernetes_api: &Api<Dog>, dog: &mut Dog, uuid: &str) -> Result<()> {
    let dto = convert_kube_type_to_dto(dog.clone());
    let config = get_client_config().await?;

    if uuid.is_empty() {
        return Err(anyhow::anyhow!("uuid is empty"));
    }

    update_dog_by_id(&config, uuid, dto)
        .await
        .context("Failed to update a dog by id")?;

    let dog_name = dog.metadata.name.as_deref().unwrap_or_default();
    kubernetes_api
        .replace(dog_name, &PostParams::default(), dog)
        .await
        .context("Failed to update a dog by id")?;

    info!("Updated a dog by id went successfully");
    Ok(())
}

pub async fn handle_create(kube_client: &Api<Dog>, dog: &mut Dog) -> Result<(), anyhow::Error> {
    let dto = convert_kube_type_to_dto(dog.clone());
    let config = get_client_config().await?;

    match create_dog(&config, dto.clone()).await {
        Ok(remote_dog) => match remote_dog.uuid {
            Some(uuid) => {
                add_finalizer(dog, kube_client.clone()).await?;
                let condition = create_condition(
                    "Created",
                    "AvailableCreated",
                    "Created the resource",
                    "Resource has been created",
                    dog.meta().generation,
                );
                let mut dog_clone = dog.clone();
                if let Some(status) = dog_clone.status.as_mut() {
                    status.conditions.push(condition);
                    status.uuid = Some(uuid);
                    status.observed_generation = dog.meta().generation;
                }
                update_status(kube_client, dog_clone).await
            }
            None => {
                warn!("Remote dog has no uuid, cannot update status");
                Ok(())
            }
        },
        Err(e) => {
            error!("Failed to create a new dog: {:?}", e);
            Err(anyhow::anyhow!("Failed to create a new dog: {:?}", e))
        }
    }
}
