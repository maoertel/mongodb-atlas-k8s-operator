pub mod atlas;
pub mod cli;
pub mod config;
pub mod crd;
pub mod error;
pub mod k8s;
pub mod operator;

use std::sync::Arc;

use clap::Parser;
use kube::Api;
use kube::Client;
use kuberator::cache::{CachingStrategy, StaticApiProvider};
use kuberator::k8s::K8sRepository;
use kuberator::Reconcile;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::atlas::{AtlasUserContext, AtlasUserRepository};
use crate::cli::Cli;
use crate::config::Config;
use crate::crd::AtlasUser;
use crate::error::Result;
use crate::operator::AtlasUserReconciler;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    // Load config from file if provided, otherwise use defaults
    let config = match &cli.config_path {
        Some(path) => Config::from_file(path)?,
        None => Config::default(),
    };

    // Create Atlas repository with bearer token
    let atlas_repo = Arc::new(AtlasUserRepository::new(cli.access_token.into())?);

    // Create K8s client and API
    let k8s_client = Client::try_default().await?;

    // Get namespaces to watch (default to all namespaces)
    let namespaces = cli.namespaces.clone().unwrap_or_else(|| vec!["default".to_string()]);

    // Create StaticApiProvider for caching API instances
    let api_provider = StaticApiProvider::<AtlasUser>::new(k8s_client.clone(), &namespaces, CachingStrategy::Adhoc);

    // Create K8s repository
    let k8s_repo = Arc::new(K8sRepository::new(api_provider));

    // Create context
    let context = Arc::new(AtlasUserContext::new(atlas_repo, k8s_repo, config.atlas_user));

    // Create API for the reconciler
    let crd_api: Api<AtlasUser> = Api::all(k8s_client);

    // Create reconciler
    let reconciler = AtlasUserReconciler::new(crd_api, context);

    info!("Starting the MongoDB Atlas Kubernetes Operator");

    // Start the reconciler (no graceful shutdown trigger)
    reconciler.start::<std::future::Pending<()>>(None).await;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
