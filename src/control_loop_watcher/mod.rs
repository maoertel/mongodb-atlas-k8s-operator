pub mod error;

use std::sync::Arc;
use std::time::Duration;

use futures::stream::StreamExt;
use kube::runtime::controller::Action;
use kube::runtime::watcher::Config;
use kube::runtime::Controller;
use kube::Api;
use kube::Resource;
use kube::ResourceExt;

use crate::atlas::AtlasUserContext;
use crate::control_loop_watcher::error::Error;
use crate::control_loop_watcher::error::Result;
use crate::crd::AtlasUser;

pub struct AtlasUserReconciler {
    crd_api: Api<AtlasUser>,
    atlas_context: Arc<AtlasUserContext>,
}

enum AtlasUserAction {
    Create,
    Delete,
    NoOp,
}

impl AtlasUserReconciler {
    pub fn new(k8s_client: kube::Client, atlas_context: Arc<AtlasUserContext>) -> Self {
        AtlasUserReconciler {
            crd_api: Api::all(k8s_client),
            atlas_context,
        }
    }

    pub async fn start(self) {
        Controller::new(self.crd_api, Config::default())
            .run(Self::reconcile, Self::on_error, self.atlas_context)
            .for_each(|reconciliation_result| async move {
                match reconciliation_result {
                    Ok(atlas_user_resource) => {
                        log::info!("Reconciliation successful. Resource: {atlas_user_resource:?}");
                    }
                    Err(reconciliation_err) => {
                        log::error!("Reconciliation error: {reconciliation_err:?}");
                    }
                }
            })
            .await;
    }

    async fn reconcile(atlas_user: Arc<AtlasUser>, atlas_context: Arc<AtlasUserContext>) -> Result<Action> {
        // TODO Validation: Check if the namespace exists in the cluster.
        let namespace = atlas_user.namespace().ok_or(Error::UserInputError({
            "Expected AtlasUser resource to be namespaced. Can't deploy to an unknown namespace.".to_owned()
        }))?;

        match determine_action(&atlas_user) {
            AtlasUserAction::Create => {
                log::info!("Creating user in Atlas: {atlas_user:?}");
                atlas_context.handle_creation(&atlas_user, &namespace).await?;
                Ok(Action::requeue(Duration::from_secs(10)))
            }
            AtlasUserAction::Delete => {
                log::info!("Deleting user in Atlas: {atlas_user:?}");
                atlas_context.handle_deletion(&atlas_user, &namespace).await?;
                Ok(Action::await_change())
            }
            AtlasUserAction::NoOp => {
                log::info!("No action required for AtlasUser: {atlas_user:?}");
                Ok(Action::requeue(Duration::from_secs(10)))
            }
        }
    }

    /// Unused argument `_context`: Context Data "injected" automatically by kube-rs.
    pub fn on_error(atlas_user: Arc<AtlasUser>, error: &Error, _context: Arc<AtlasUserContext>) -> Action {
        log::error!("Reconciliation error:\n{error:?}.\n{atlas_user:?}");
        Action::requeue(Duration::from_secs(5))
    }
}

fn determine_action(atlas_user: &AtlasUser) -> AtlasUserAction {
    if atlas_user.meta().deletion_timestamp.is_some() {
        return AtlasUserAction::Delete;
    }

    if atlas_user
        .meta()
        .finalizers
        .as_ref()
        .map_or(true, |finalizers| finalizers.is_empty())
    {
        return AtlasUserAction::Create;
    }

    AtlasUserAction::NoOp
}
