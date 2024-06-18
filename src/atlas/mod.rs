pub mod client;
pub mod error;
pub mod user;

use kube::api::Patch;
use kube::api::PatchParams;
use kube::Api;
use kube::Client;
use kube::ResourceExt;
use serde_json::json;
use serde_json::Value;

use crate::atlas::client::AtlasClient;
use crate::atlas::error::Result;
use crate::crd::AtlasUser;

pub struct AtlasUserContext {
    atlas_client: AtlasClient,
    k8s_client: Client,
}

enum FinalizerAction {
    Add,
    Remove,
}

impl AtlasUserContext {
    pub fn new(atlas_client: AtlasClient, k8s_client: Client) -> Self {
        AtlasUserContext {
            atlas_client,
            k8s_client,
        }
    }

    pub async fn handle_creation(&self, atlas_user: &AtlasUser, namespace: &str) -> Result<AtlasUser> {
        self.atlas_client.create_atlas_user(atlas_user).await?;
        self.finalizer(FinalizerAction::Add, atlas_user, namespace).await
    }

    pub async fn handle_deletion(&self, atlas_user: &AtlasUser, namespace: &str) -> Result<AtlasUser> {
        // TODO: As there is no endpoint to delete a user in Atlas, we have to notify about the
        // deletion of the user, so that he can be removed manually from Atlas UI.
        self.finalizer(FinalizerAction::Remove, atlas_user, namespace).await
    }

    async fn finalizer(&self, action: FinalizerAction, atlas_user: &AtlasUser, namespace: &str) -> Result<AtlasUser> {
        let api = Api::namespaced(self.k8s_client.to_owned(), namespace);
        let name = atlas_user.name_unchecked();
        let finalizer = match action {
            FinalizerAction::Add => json! { ["atlasusers.moertel.com/finalizer"] },
            FinalizerAction::Remove => Value::Null,
        };
        let finalizer = json!({ "metadata": { "finalizers": finalizer } });

        let patch = Patch::Merge(&finalizer);

        Ok(api.patch(&name, &PatchParams::default(), &patch).await?)
    }
}
