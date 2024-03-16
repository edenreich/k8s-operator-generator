#![allow(missing_docs, trivial_casts, unused_variables, unused_mut, unused_imports, unused_extern_crates, non_camel_case_types)]
#![allow(unused_imports, unused_attributes)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::disallowed_names)]

use async_trait::async_trait;
use futures::Stream;
use std::error::Error;
use std::task::{Poll, Context};
use swagger::{ApiError, ContextWrapper};
use serde::{Serialize, Deserialize};

type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub const BASE_PATH: &str = "";
pub const API_VERSION: &str = "1.0.0";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CatsGetResponse {
    /// OK
    OK
    (Vec<models::Cat>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CatsIdDeleteResponse {
    /// No content
    NoContent
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum CatsIdGetResponse {
    /// OK
    OK
    (models::Cat)
    ,
    /// Not found
    NotFound
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum CatsIdPutResponse {
    /// OK
    OK
    (models::Cat)
    ,
    /// Not found
    NotFound
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum CatsPostResponse {
    /// Created
    Created
    (models::Cat)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DogsGetResponse {
    /// OK
    OK
    (Vec<models::Dog>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DogsIdDeleteResponse {
    /// No content
    NoContent
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum DogsIdGetResponse {
    /// OK
    OK
    (models::Dog)
    ,
    /// Not found
    NotFound
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum DogsIdPutResponse {
    /// OK
    OK
    (models::Dog)
    ,
    /// Not found
    NotFound
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DogsPostResponse {
    /// Created
    Created
    (models::Dog)
}

/// API
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait Api<C: Send + Sync> {
    fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    /// Get all cats
    async fn cats_get(
        &self,
        context: &C) -> Result<CatsGetResponse, ApiError>;

    /// Delete a cat by ID
    async fn cats_id_delete(
        &self,
        id: String,
        context: &C) -> Result<CatsIdDeleteResponse, ApiError>;

    /// Get a cat by ID
    async fn cats_id_get(
        &self,
        id: String,
        context: &C) -> Result<CatsIdGetResponse, ApiError>;

    /// Update a cat by ID
    async fn cats_id_put(
        &self,
        id: String,
        cat: models::Cat,
        context: &C) -> Result<CatsIdPutResponse, ApiError>;

    /// Create a new cat
    async fn cats_post(
        &self,
        cat: models::Cat,
        context: &C) -> Result<CatsPostResponse, ApiError>;

    /// Get all dogs
    async fn dogs_get(
        &self,
        context: &C) -> Result<DogsGetResponse, ApiError>;

    /// Delete a dog by ID
    async fn dogs_id_delete(
        &self,
        id: String,
        context: &C) -> Result<DogsIdDeleteResponse, ApiError>;

    /// Get a dog by ID
    async fn dogs_id_get(
        &self,
        id: String,
        context: &C) -> Result<DogsIdGetResponse, ApiError>;

    /// Update a dog by ID
    async fn dogs_id_put(
        &self,
        id: String,
        dog: models::Dog,
        context: &C) -> Result<DogsIdPutResponse, ApiError>;

    /// Create a new dog
    async fn dogs_post(
        &self,
        dog: models::Dog,
        context: &C) -> Result<DogsPostResponse, ApiError>;

}

/// API where `Context` isn't passed on every API call
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait ApiNoContext<C: Send + Sync> {

    fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    /// Get all cats
    async fn cats_get(
        &self,
        ) -> Result<CatsGetResponse, ApiError>;

    /// Delete a cat by ID
    async fn cats_id_delete(
        &self,
        id: String,
        ) -> Result<CatsIdDeleteResponse, ApiError>;

    /// Get a cat by ID
    async fn cats_id_get(
        &self,
        id: String,
        ) -> Result<CatsIdGetResponse, ApiError>;

    /// Update a cat by ID
    async fn cats_id_put(
        &self,
        id: String,
        cat: models::Cat,
        ) -> Result<CatsIdPutResponse, ApiError>;

    /// Create a new cat
    async fn cats_post(
        &self,
        cat: models::Cat,
        ) -> Result<CatsPostResponse, ApiError>;

    /// Get all dogs
    async fn dogs_get(
        &self,
        ) -> Result<DogsGetResponse, ApiError>;

    /// Delete a dog by ID
    async fn dogs_id_delete(
        &self,
        id: String,
        ) -> Result<DogsIdDeleteResponse, ApiError>;

    /// Get a dog by ID
    async fn dogs_id_get(
        &self,
        id: String,
        ) -> Result<DogsIdGetResponse, ApiError>;

    /// Update a dog by ID
    async fn dogs_id_put(
        &self,
        id: String,
        dog: models::Dog,
        ) -> Result<DogsIdPutResponse, ApiError>;

    /// Create a new dog
    async fn dogs_post(
        &self,
        dog: models::Dog,
        ) -> Result<DogsPostResponse, ApiError>;

}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync> where Self: Sized
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
         ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    /// Get all cats
    async fn cats_get(
        &self,
        ) -> Result<CatsGetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().cats_get(&context).await
    }

    /// Delete a cat by ID
    async fn cats_id_delete(
        &self,
        id: String,
        ) -> Result<CatsIdDeleteResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().cats_id_delete(id, &context).await
    }

    /// Get a cat by ID
    async fn cats_id_get(
        &self,
        id: String,
        ) -> Result<CatsIdGetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().cats_id_get(id, &context).await
    }

    /// Update a cat by ID
    async fn cats_id_put(
        &self,
        id: String,
        cat: models::Cat,
        ) -> Result<CatsIdPutResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().cats_id_put(id, cat, &context).await
    }

    /// Create a new cat
    async fn cats_post(
        &self,
        cat: models::Cat,
        ) -> Result<CatsPostResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().cats_post(cat, &context).await
    }

    /// Get all dogs
    async fn dogs_get(
        &self,
        ) -> Result<DogsGetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().dogs_get(&context).await
    }

    /// Delete a dog by ID
    async fn dogs_id_delete(
        &self,
        id: String,
        ) -> Result<DogsIdDeleteResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().dogs_id_delete(id, &context).await
    }

    /// Get a dog by ID
    async fn dogs_id_get(
        &self,
        id: String,
        ) -> Result<DogsIdGetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().dogs_id_get(id, &context).await
    }

    /// Update a dog by ID
    async fn dogs_id_put(
        &self,
        id: String,
        dog: models::Dog,
        ) -> Result<DogsIdPutResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().dogs_id_put(id, dog, &context).await
    }

    /// Create a new dog
    async fn dogs_post(
        &self,
        dog: models::Dog,
        ) -> Result<DogsPostResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().dogs_post(dog, &context).await
    }

}


#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::Service;

#[cfg(feature = "server")]
pub mod context;

pub mod models;

#[cfg(any(feature = "client", feature = "server"))]
pub(crate) mod header;
