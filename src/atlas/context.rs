use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use kube::runtime::controller::Action;
use kuberator::cache::StaticApiProvider;
use kuberator::error::Error as KubeError;
use kuberator::error::Result as KubeResult;
use kuberator::k8s::K8sRepository;
use kuberator::{Context, Finalize, ObserveGeneration, TryResource};
use tracing::{info, warn};

use crate::atlas::error::Error;
use crate::atlas::repository::AtlasUserRepository;
use crate::atlas::user_request::UserRequest;
use crate::config::AtlasUserConfig;
use crate::crd::{AtlasUser, AtlasUserStatus, UserOrgMembershipStatus};
use crate::k8s::AtlasUserK8sRepo;

const FINALIZER: &str = "atlasusers.moertel.com/finalizer";

/// Context for reconciling AtlasUser resources
pub struct AtlasUserContext {
    atlas_repo: Arc<AtlasUserRepository>,
    k8s_repo: Arc<AtlasUserK8sRepo>,
    config: AtlasUserConfig,
}

impl AtlasUserContext {
    pub fn new(atlas_repo: Arc<AtlasUserRepository>, k8s_repo: Arc<AtlasUserK8sRepo>, config: AtlasUserConfig) -> Self {
        Self {
            atlas_repo,
            k8s_repo,
            config,
        }
    }

    /// Determines if the resource needs to be updated based on generation
    fn needs_update(&self, atlas_user: &AtlasUser) -> bool {
        let current_gen = atlas_user.metadata.generation.unwrap_or(0);
        let observed_gen = atlas_user
            .status
            .as_ref()
            .and_then(|s| s.observed_generation)
            .unwrap_or(0);
        current_gen > observed_gen
    }

    /// Invites a new user to Atlas
    async fn invite_user(&self, atlas_user: Arc<AtlasUser>) -> KubeResult<Action> {
        let (name, namespace) = (atlas_user.try_name()?.to_string(), atlas_user.try_namespace()?);
        let spec = &atlas_user.spec;

        info!(name = %name, namespace = %namespace, username = %spec.username, "Inviting new user to Atlas");

        let request = UserRequest::for_invite(spec);
        let response = self.atlas_repo.invite_atlas_user(&spec.org_id, &request).await?;

        // Update status with the new user ID and membership status
        let mut status = atlas_user.status.clone().unwrap_or_default();
        status.user_id = Some(response.id);
        status.membership_status = Some(response.org_membership_status);
        status.error = None;
        status.with_observed_gen(&atlas_user.metadata);

        self.k8s_repo.update_status(&atlas_user, status).await?;

        Ok(Action::requeue(self.config.requeue_duration))
    }

    /// Updates an existing user in Atlas
    async fn update_user(&self, atlas_user: Arc<AtlasUser>, user_id: &str) -> KubeResult<Action> {
        let (name, namespace) = (atlas_user.try_name()?.to_string(), atlas_user.try_namespace()?);
        let spec = &atlas_user.spec;

        info!(name = %name, namespace = %namespace, user_id = %user_id, "Updating user in Atlas");

        let request = UserRequest::for_update(spec);
        let response = self
            .atlas_repo
            .update_atlas_user(&spec.org_id, user_id, &request)
            .await?;

        // Update status
        let mut status = atlas_user.status.clone().unwrap_or_default();
        status.membership_status = Some(response.org_membership_status);
        status.error = None;
        status.with_observed_gen(&atlas_user.metadata);

        self.k8s_repo.update_status(&atlas_user, status).await?;

        Ok(Action::requeue(self.config.requeue_duration))
    }

    /// Syncs the status from Atlas to the K8s resource
    async fn sync_status(&self, atlas_user: Arc<AtlasUser>, user_id: &str) -> KubeResult<Action> {
        let (name, namespace) = (atlas_user.try_name()?.to_string(), atlas_user.try_namespace()?);
        let spec = &atlas_user.spec;

        info!(name = %name, namespace = %namespace, "Syncing user status from Atlas");

        match self.atlas_repo.get_atlas_user(&spec.org_id, user_id).await {
            Ok(response) => {
                let mut status = atlas_user.status.clone().unwrap_or_default();
                status.membership_status = Some(response.org_membership_status);
                status.error = None;

                self.k8s_repo.update_status(&atlas_user, status).await?;
            }
            Err(Error::AtlasUserNotFound { .. }) => {
                // User was deleted externally, clear the user_id
                warn!(name = %name, namespace = %namespace, "User not found in Atlas, clearing status");
                let status = AtlasUserStatus {
                    error: Some("User was deleted externally from Atlas".to_string()),
                    ..Default::default()
                };

                self.k8s_repo.update_status(&atlas_user, status).await?;
            }
            Err(e) => return Err(e.into()),
        }

        Ok(Action::requeue(self.config.requeue_duration))
    }
}

#[async_trait]
impl Context<AtlasUser, AtlasUserK8sRepo, StaticApiProvider<AtlasUser>> for AtlasUserContext {
    fn k8s_repository(&self) -> Arc<K8sRepository<AtlasUser, StaticApiProvider<AtlasUser>>> {
        Arc::clone(&self.k8s_repo)
    }

    fn finalizer(&self) -> &'static str {
        FINALIZER
    }

    async fn handle_apply(&self, atlas_user: Arc<AtlasUser>) -> KubeResult<Action> {
        let (name, namespace) = (atlas_user.try_name()?.to_string(), atlas_user.try_namespace()?);
        let current_gen = atlas_user.metadata.generation.unwrap_or(1);

        // Check if we have a user_id from previous reconciliation
        let user_id = atlas_user
            .status
            .as_ref()
            .and_then(|s| s.user_id.as_ref())
            .map(|id| id.to_string());

        let needs_update = self.needs_update(&atlas_user);

        match (user_id, needs_update, current_gen) {
            (Some(user_id), true, _) => {
                // User exists and spec changed -> update
                self.update_user(atlas_user, &user_id).await
            }
            (Some(user_id), false, _) => {
                // User exists and spec unchanged -> sync status
                self.sync_status(atlas_user, &user_id).await
            }
            (None, _, 1) => {
                // New resource (generation == 1 and no user_id) -> invite
                self.invite_user(atlas_user).await
            }
            (None, _, _) => {
                // No user_id but generation > 1, try to find by username
                let spec = &atlas_user.spec;
                info!(name = %name, namespace = %namespace, "Looking up user by username");

                match self
                    .atlas_repo
                    .find_atlas_user_by_username(&spec.org_id, &spec.username)
                    .await?
                {
                    Some(response) => {
                        // Found the user, update status and proceed
                        let mut status = atlas_user.status.clone().unwrap_or_default();
                        status.user_id = Some(response.id.clone());
                        status.membership_status = Some(response.org_membership_status);
                        status.error = None;
                        status.with_observed_gen(&atlas_user.metadata);

                        self.k8s_repo.update_status(&atlas_user, status).await?;

                        Ok(Action::requeue(Duration::from_secs(1))) // Requeue immediately to process with user_id
                    }
                    None => {
                        // User doesn't exist in Atlas, create new
                        self.invite_user(atlas_user).await
                    }
                }
            }
        }
    }

    async fn handle_cleanup(&self, atlas_user: Arc<AtlasUser>) -> KubeResult<Action> {
        let (name, namespace) = (atlas_user.try_name()?.to_string(), atlas_user.try_namespace()?);

        if !self.config.safe_to_delete {
            info!(
                name = %name,
                namespace = %namespace,
                "safe_to_delete is false, skipping Atlas user deletion"
            );
            return Ok(Action::await_change());
        }

        let user_id = atlas_user
            .status
            .as_ref()
            .and_then(|s| s.user_id.as_ref())
            .ok_or_else(|| KubeError::UserInput("Cannot delete user without user_id in status".to_string()))?;

        info!(name = %name, namespace = %namespace, user_id = %user_id, "Deleting user from Atlas");

        self.atlas_repo
            .delete_atlas_user_from_org(&atlas_user.spec.org_id, user_id)
            .await?;

        // Update status to deleted
        let mut status = atlas_user.status.clone().unwrap_or_default();
        status.membership_status = Some(UserOrgMembershipStatus::Deleted);
        status.error = None;

        self.k8s_repo.update_status(&atlas_user, status).await?;

        Ok(Action::await_change())
    }
}
