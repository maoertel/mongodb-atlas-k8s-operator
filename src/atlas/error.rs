use std::fmt::Debug;

use kuberator::error::Error as KubeError;
use reqwest::StatusCode;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Kubernetes reported error: {0}")]
    K8s(#[from] kube::Error),
    #[error("serde_json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Unexpected response from MongoDB Atlas API: {status}. Message: {message}.")]
    Api { status: StatusCode, message: String },
    #[error("Atlas user {user_id} not found in organization {org_id}")]
    AtlasUserNotFound { user_id: String, org_id: String },
    #[error("Status object not set yet")]
    StatusObjectNotSet,
}

impl From<Error> for KubeError {
    fn from(error: Error) -> KubeError {
        KubeError::Anyhow(anyhow::anyhow!(error))
    }
}
