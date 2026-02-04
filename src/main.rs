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
use kuberator::cache::CachingStrategy;
use kuberator::cache::StaticApiProvider;
use kuberator::k8s::K8sRepository;
use kuberator::Reconcile;
use tokio::signal::unix::SignalKind;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::atlas::AtlasUserContext;
use crate::atlas::AtlasUserRepository;
use crate::cli::Cli;
use crate::config::Config;
use crate::crd::AtlasUser;
use crate::error::Result;
use crate::operator::AtlasUserReconciler;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    let config = Config::from_file(&cli.config_path)?;

    let atlas_repo = Arc::new(AtlasUserRepository::new(cli.access_token.into())?);

    let k8s_client = Client::try_default().await?;
    let namespaces = cli.namespaces.unwrap_or_else(|| vec!["default".to_string()]);
    let api_provider = StaticApiProvider::<AtlasUser>::new(k8s_client.clone(), &namespaces, CachingStrategy::Adhoc);
    let k8s_repo = Arc::new(K8sRepository::new(api_provider));

    let context = Arc::new(AtlasUserContext::new(atlas_repo, k8s_repo, config.atlas_user));

    let crd_api: Api<AtlasUser> = Api::all(k8s_client);

    let reconciler = AtlasUserReconciler::new(crd_api, context);

    info!("Starting the MongoDB Atlas Kubernetes Operator");

    reconciler.start(Some(graceful_shutdown())).await;

    info!("Operator shut down gracefully");

    Ok(())
}

async fn graceful_shutdown() {
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => info!("Received SIGINT, shutting down"),
        _ = sigterm.recv() => info!("Received SIGTERM, shutting down"),
    }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
