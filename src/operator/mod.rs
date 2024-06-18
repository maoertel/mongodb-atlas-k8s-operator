pub mod atlasuser;
pub mod error;

use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use futures::stream::StreamExt;
use futures::Future;
use kube::runtime::controller::Action;
use kube::runtime::watcher::Config;
use kube::runtime::Controller;
use kube::Api;
use kube::Resource;
use serde::de::DeserializeOwned;

use crate::operator::atlasuser::AtlasUserReconciler;
use crate::operator::error::Error;
use crate::operator::error::Result;

pub trait Reconcile<Crd, Ctx>: Sized
where
    Crd: Clone + Resource + DeserializeOwned + Debug + Send + Sync + 'static,
    Crd::DynamicType: Default + Eq + Hash + Clone + Debug + Unpin,
{
    fn reconcile(crd: Arc<Crd>, context: Arc<Ctx>) -> impl Future<Output = Result<Action>> + Send + 'static;
    fn error_policy(crd: Arc<Crd>, error: &Error, context: Arc<Ctx>) -> Action;
    fn destruct(self) -> (Api<Crd>, Config, Arc<Ctx>);

    #[allow(async_fn_in_trait)]
    async fn start(self) {
        let (crd_api, config, context) = self.destruct();
        Controller::new(crd_api, config)
            .run(Self::reconcile, Self::error_policy, context)
            .for_each(|reconciliation_result| async move {
                match reconciliation_result {
                    Ok(resource) => {
                        log::info!("Reconciliation successful. Resource: {resource:?}");
                    }
                    Err(error) => {
                        log::error!("Reconciliation error: {error:?}");
                    }
                }
            })
            .await;
    }
}

pub struct Operator {
    atlas_user_reconciler: AtlasUserReconciler,
}

impl Operator {
    pub fn new(atlas_user_reconciler: AtlasUserReconciler) -> Self {
        Operator { atlas_user_reconciler }
    }

    pub async fn run(self) {
        self.atlas_user_reconciler.start().await;
    }
}
