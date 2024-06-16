pub mod error;

use kube::api::Patch;
use kube::api::PatchParams;
use kube::Api;
use kube::Client;
use kube::ResourceExt;
use serde_json::json;
use serde_json::Value;

use crate::atlas::error::Result;
use crate::crd::AtlasUser;

pub struct AtlasUserContext {
    k8s_client: Client,
}

enum FinalizerAction {
    Add,
    Remove,
}

impl AtlasUserContext {
    pub fn new(k8s_client: Client) -> Self {
        AtlasUserContext { k8s_client }
    }

    pub async fn handle_creation(&self, atlas_user: &AtlasUser, namespace: &str) -> Result<AtlasUser> {
        // Create the user in Atlas
        self.finalizer(FinalizerAction::Add, atlas_user, namespace).await
    }

    pub async fn handle_deletion(&self, atlas_user: &AtlasUser, namespace: &str) -> Result<AtlasUser> {
        // Remove a user in Atlas
        self.finalizer(FinalizerAction::Remove, atlas_user, namespace).await
    }

    async fn finalizer(&self, action: FinalizerAction, atlas_user: &AtlasUser, namespace: &str) -> Result<AtlasUser> {
        let api = Api::namespaced(self.k8s_client.to_owned(), namespace);
        let name = atlas_user.name_unchecked();
        let finalizer = json!({
            "metadata": {
                "finalizers":  match action {
                    FinalizerAction::Add => json! { ["atlasusers.moertel.com/finalizer"] },
                    FinalizerAction::Remove => Value::Null,
                }
            }
        });
        let patch: Patch<&Value> = Patch::Merge(&finalizer);

        Ok(api.patch(&name, &PatchParams::default(), &patch).await?)
    }
}
