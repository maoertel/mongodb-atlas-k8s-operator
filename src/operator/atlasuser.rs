use std::sync::Arc;
use std::time::Duration;

use kube::runtime::controller::Action;
use kube::runtime::watcher::Config;
use kube::Api;
use kube::Resource;
use kube::ResourceExt;

use crate::atlas::AtlasUserContext;
use crate::crd::AtlasUser;
use crate::operator::error::Error;
use crate::operator::error::Result;
use crate::operator::Reconcile;

pub struct AtlasUserReconciler {
    crd_api: Api<AtlasUser>,
    atlas_context: Arc<AtlasUserContext>,
}

impl AtlasUserReconciler {
    pub fn new(k8s_client: kube::Client, atlas_context: Arc<AtlasUserContext>) -> Self {
        AtlasUserReconciler {
            crd_api: Api::all(k8s_client),
            atlas_context,
        }
    }
}

enum AtlasUserAction {
    Create,
    Delete,
    NoOp,
}

impl Reconcile<AtlasUser, AtlasUserContext> for AtlasUserReconciler {
    async fn reconcile(atlas_user: Arc<AtlasUser>, context: Arc<AtlasUserContext>) -> Result<Action> {
        let namespace = atlas_user.namespace().ok_or(Error::UserInputError({
            "Expected AtlasUser resource to be namespaced. Can't deploy to an unknown namespace.".to_owned()
        }))?;

        match validate_change(&atlas_user) {
            AtlasUserAction::Create => {
                log::info!("Creating user in Atlas: {atlas_user:?}");
                context.handle_creation(&atlas_user, &namespace).await?;
                Ok(Action::requeue(Duration::from_secs(10)))
            }
            AtlasUserAction::Delete => {
                log::info!("Deleting user in Atlas: {atlas_user:?}");
                context.handle_deletion(&atlas_user, &namespace).await?;
                Ok(Action::await_change())
            }
            AtlasUserAction::NoOp => {
                log::info!("No action required for AtlasUser: {atlas_user:?}");
                Ok(Action::requeue(Duration::from_secs(10)))
            }
        }
    }

    /// Unused argument `_context`: Context Data "injected" automatically by kube-rs.
    fn error_policy(atlas_user: Arc<AtlasUser>, error: &Error, _context: Arc<AtlasUserContext>) -> Action {
        log::error!("Reconciliation error:\n{error:?}.\n{atlas_user:?}");
        Action::requeue(Duration::from_secs(5))
    }

    fn destruct(self) -> (Api<AtlasUser>, Config, Arc<AtlasUserContext>) {
        (self.crd_api, Config::default(), self.atlas_context)
    }
}

fn validate_change(atlas_user: &AtlasUser) -> AtlasUserAction {
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
