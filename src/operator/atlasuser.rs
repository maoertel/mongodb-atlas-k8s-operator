use std::sync::Arc;

use kube::runtime::watcher::Config;
use kube::Api;
use kuberator::cache::StaticApiProvider;
use kuberator::k8s::K8sRepository;
use kuberator::Reconcile;

use crate::atlas::AtlasUserContext;
use crate::crd::AtlasUser;

/// Reconciler for AtlasUser resources
pub struct AtlasUserReconciler {
    crd_api: Api<AtlasUser>,
    context: Arc<AtlasUserContext>,
}

impl AtlasUserReconciler {
    pub fn new(crd_api: Api<AtlasUser>, context: Arc<AtlasUserContext>) -> Self {
        AtlasUserReconciler { crd_api, context }
    }
}

impl
    Reconcile<
        AtlasUser,
        AtlasUserContext,
        K8sRepository<AtlasUser, StaticApiProvider<AtlasUser>>,
        StaticApiProvider<AtlasUser>,
    > for AtlasUserReconciler
{
    fn destruct(self) -> (Api<AtlasUser>, Config, Arc<AtlasUserContext>) {
        (self.crd_api, Config::default(), self.context)
    }
}
