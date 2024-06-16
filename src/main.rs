pub mod atlas;
pub mod control_loop_watcher;
mod crd;
pub mod error;
pub mod logger;

use std::sync::Arc;

use kube::Client;

use crate::atlas::client::AtlasClient;
use crate::atlas::AtlasUserContext;
use crate::control_loop_watcher::AtlasUserReconciler;
use crate::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;

    let public_key = String::from("ATLAS_PUBLIC_KEY");
    let private_key = String::from("ATLAS_PRIVATE_KEY");

    let atlas_client = AtlasClient::new(reqwest::Client::new(), public_key, private_key)?;
    let k8s_client = Client::try_default().await?;
    let atlas_context = Arc::new(AtlasUserContext::new(atlas_client, k8s_client));

    let k8s_client = Client::try_default().await?;
    let atlas_user_reconciler = AtlasUserReconciler::new(k8s_client, atlas_context);

    log::info!("Starting the AtlasUser operator.");
    atlas_user_reconciler.start().await;

    Ok(())
}
