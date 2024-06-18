use std::fmt::Debug;

use kube::runtime::finalizer;
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
    #[error("Diqwest error: {0}")]
    Diqwest(#[from] diqwest::error::Error),
    #[error("Unexpected response from MongoDB Atlas API: {status}. Message: {message}.")]
    Api { status: StatusCode, message: String },
    #[error(transparent)]
    Finalizer(#[from] FinalizerError),
}

#[derive(ThisError, Debug)]
pub enum FinalizerError {
    #[error("Failed to apply object, error: {0}")]
    ApplyFailed(String),
    #[error("Failed to clean up object: {0}")]
    CleanupFailed(String),
    #[error("Failed to add finalizer: {0}")]
    AddFinalizer(kube::Error),
    #[error("Failed to remove finalizer: {0}")]
    RemoveFinalizer(kube::Error),
    #[error("Object has no name")]
    UnnamedObject,
    #[error("Invalid finalizer")]
    InvalidFinalizer,
}

impl Error {
    pub(crate) fn from<K: std::error::Error + 'static>(e: finalizer::Error<K>) -> Self {
        match e {
            finalizer::Error::ApplyFailed(e) => {
                Error::Finalizer(FinalizerError::ApplyFailed(format!("Failed to apply object: {e}")))
            }
            finalizer::Error::CleanupFailed(e) => {
                Error::Finalizer(FinalizerError::CleanupFailed(format!("Failed to apply object: {e}")))
            }
            finalizer::Error::AddFinalizer(e) => Error::Finalizer(FinalizerError::AddFinalizer(e)),
            finalizer::Error::RemoveFinalizer(e) => Error::Finalizer(FinalizerError::RemoveFinalizer(e)),
            finalizer::Error::UnnamedObject => Error::Finalizer(FinalizerError::UnnamedObject),
            finalizer::Error::InvalidFinalizer => Error::Finalizer(FinalizerError::InvalidFinalizer),
        }
    }
}
