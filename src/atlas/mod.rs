pub mod client;
pub mod error;
pub mod user;

use std::sync::Arc;
use std::time::Duration;

use kube::runtime::controller::Action;
use kube::runtime::finalizer;
use kube::runtime::finalizer::Event;
use kube::Api;
use kube::Client;

use crate::atlas::client::AtlasClient;
use crate::atlas::error::Error;
use crate::atlas::error::Result;
use crate::crd::AtlasUser;

pub struct AtlasUserContext {
    atlas_client: AtlasClient,
    api: Api<AtlasUser>,
}

impl AtlasUserContext {
    pub fn new(atlas_client: AtlasClient, k8s_client: Client) -> Self {
        let api: Api<AtlasUser> = Api::all(k8s_client);
        AtlasUserContext { atlas_client, api }
    }

    pub async fn handle_creation(&self, atlas_user: Arc<AtlasUser>, namespace: &str) -> Result<Action> {
        self.atlas_client.create_atlas_user(&atlas_user).await?;
        self.finalize(atlas_user, namespace).await
    }

    pub async fn handle_deletion(&self, atlas_user: Arc<AtlasUser>, namespace: &str) -> Result<Action> {
        // TODO: As there is no endpoint to delete a user in Atlas, we have to notify about the
        // deletion of the user, so that he can be removed manually from Atlas UI.
        self.finalize(atlas_user, namespace).await
    }

    async fn finalize(&self, atlas_user: Arc<AtlasUser>, _namespace: &str) -> Result<Action> {
        finalizer(
            &self.api,
            "atlasusers.moertel.com/finalizer",
            atlas_user,
            Self::reconcile,
        )
        .await
        .map_err(Error::from)
    }

    async fn reconcile(event: Event<AtlasUser>) -> Result<Action> {
        match event {
            Event::Apply(_atlas_user) => Ok(Action::requeue(Duration::from_secs(10))),
            Event::Cleanup(_atlas_user) => Ok(Action::await_change()),
        }
    }
}
