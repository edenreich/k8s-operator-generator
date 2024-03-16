//! Main library entry point for openapi_client implementation.

#![allow(unused_imports)]

use async_trait::async_trait;
use futures::{future, Stream, StreamExt, TryFutureExt, TryStreamExt};
use hyper::server::conn::Http;
use hyper::service::Service;
use log::info;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::{Has, XSpanIdString};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use tokio::net::TcpListener;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{Ssl, SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use openapi_client::models;

/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub async fn create(addr: &str, https: bool) {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new();

    let service = MakeService::new(server);

    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    #[allow(unused_mut)]
    let mut service =
        openapi_client::server::context::MakeAddContext::<_, EmptyContext>::new(
            service
        );

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()).expect("Failed to create SSL Acceptor");

            // Server authentication
            ssl.set_private_key_file("examples/server-key.pem", SslFiletype::PEM).expect("Failed to set private key");
            ssl.set_certificate_chain_file("examples/server-chain.pem").expect("Failed to set certificate chain");
            ssl.check_private_key().expect("Failed to check private key");

            let tls_acceptor = ssl.build();
            let tcp_listener = TcpListener::bind(&addr).await.unwrap();

            loop {
                if let Ok((tcp, _)) = tcp_listener.accept().await {
                    let ssl = Ssl::new(tls_acceptor.context()).unwrap();
                    let addr = tcp.peer_addr().expect("Unable to get remote address");
                    let service = service.call(addr);

                    tokio::spawn(async move {
                        let tls = tokio_openssl::SslStream::new(ssl, tcp).map_err(|_| ())?;
                        let service = service.await.map_err(|_| ())?;

                        Http::new()
                            .serve_connection(tls, service)
                            .await
                            .map_err(|_| ())
                    });
                }
            }
        }
    } else {
        // Using HTTP
        hyper::server::Server::bind(&addr).serve(service).await.unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server{marker: PhantomData}
    }
}


use openapi_client::{
    Api,
    CatsGetResponse,
    CatsIdDeleteResponse,
    CatsIdGetResponse,
    CatsIdPutResponse,
    CatsPostResponse,
    DogsGetResponse,
    DogsIdDeleteResponse,
    DogsIdGetResponse,
    DogsIdPutResponse,
    DogsPostResponse,
};
use openapi_client::server::MakeService;
use std::error::Error;
use swagger::ApiError;

#[async_trait]
impl<C> Api<C> for Server<C> where C: Has<XSpanIdString> + Send + Sync
{
    /// Get all cats
    async fn cats_get(
        &self,
        context: &C) -> Result<CatsGetResponse, ApiError>
    {
        info!("cats_get() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    /// Delete a cat by ID
    async fn cats_id_delete(
        &self,
        id: String,
        context: &C) -> Result<CatsIdDeleteResponse, ApiError>
    {
        info!("cats_id_delete(\"{}\") - X-Span-ID: {:?}", id, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    /// Get a cat by ID
    async fn cats_id_get(
        &self,
        id: String,
        context: &C) -> Result<CatsIdGetResponse, ApiError>
    {
        info!("cats_id_get(\"{}\") - X-Span-ID: {:?}", id, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    /// Update a cat by ID
    async fn cats_id_put(
        &self,
        id: String,
        cat: models::Cat,
        context: &C) -> Result<CatsIdPutResponse, ApiError>
    {
        info!("cats_id_put(\"{}\", {:?}) - X-Span-ID: {:?}", id, cat, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    /// Create a new cat
    async fn cats_post(
        &self,
        cat: models::Cat,
        context: &C) -> Result<CatsPostResponse, ApiError>
    {
        info!("cats_post({:?}) - X-Span-ID: {:?}", cat, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    /// Get all dogs
    async fn dogs_get(
        &self,
        context: &C) -> Result<DogsGetResponse, ApiError>
    {
        info!("dogs_get() - X-Span-ID: {:?}", context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    /// Delete a dog by ID
    async fn dogs_id_delete(
        &self,
        id: String,
        context: &C) -> Result<DogsIdDeleteResponse, ApiError>
    {
        info!("dogs_id_delete(\"{}\") - X-Span-ID: {:?}", id, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    /// Get a dog by ID
    async fn dogs_id_get(
        &self,
        id: String,
        context: &C) -> Result<DogsIdGetResponse, ApiError>
    {
        info!("dogs_id_get(\"{}\") - X-Span-ID: {:?}", id, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    /// Update a dog by ID
    async fn dogs_id_put(
        &self,
        id: String,
        dog: models::Dog,
        context: &C) -> Result<DogsIdPutResponse, ApiError>
    {
        info!("dogs_id_put(\"{}\", {:?}) - X-Span-ID: {:?}", id, dog, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

    /// Create a new dog
    async fn dogs_post(
        &self,
        dog: models::Dog,
        context: &C) -> Result<DogsPostResponse, ApiError>
    {
        info!("dogs_post({:?}) - X-Span-ID: {:?}", dog, context.get().0.clone());
        Err(ApiError("Generic failure".into()))
    }

}
