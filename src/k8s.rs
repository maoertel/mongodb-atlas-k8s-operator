use kuberator::cache::StaticApiProvider;
use kuberator::k8s::K8sRepository;

use crate::crd::AtlasUser;

/// Type alias for the AtlasUser Kubernetes repository using StaticApiProvider
pub type AtlasUserK8sRepo = K8sRepository<AtlasUser, StaticApiProvider<AtlasUser>>;
