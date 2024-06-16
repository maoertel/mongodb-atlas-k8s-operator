pub mod atlas;
pub mod cli;
pub mod crd;
pub mod error;
pub mod http_client;
pub mod logger;
pub mod reconciler;

use std::sync::Arc;

use clap::Parser;
use kube::Client;

use crate::atlas::client::AtlasClient;
use crate::atlas::client::ATLAS_API_CONTENT_TYPE;
use crate::atlas::AtlasUserContext;
use crate::cli::Cli;
use crate::error::Result;
use crate::reconciler::AtlasUserReconciler;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;

    let Cli {
        public_key,
        private_key,
    } = Cli::parse();

    let http_client = http_client::accepts(ATLAS_API_CONTENT_TYPE)?;
    let atlas_client = AtlasClient::new(http_client, public_key, private_key)?;

    let k8s_client = Client::try_default().await?;
    let atlas_context = Arc::new(AtlasUserContext::new(atlas_client, k8s_client));

    let k8s_client = Client::try_default().await?;
    let atlas_user_reconciler = AtlasUserReconciler::new(k8s_client, atlas_context);

    log::info!("Starting the AtlasUser operator.");
    atlas_user_reconciler.start().await;

    Ok(())
}
