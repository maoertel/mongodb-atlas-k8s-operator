use std::fmt::Debug;

use thiserror::Error as ThisError;

use crate::{atlas, http_client, logger};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Kubernetes reported error: {0}")]
    K8s(#[from] kube::Error),
    #[error("Logger reported error: {0}")]
    Logger(#[from] logger::error::Error),
    #[error("Atlas reported error: {0}")]
    Atlas(#[from] atlas::error::Error),
    #[error("HTTP client reported error: {0}")]
    Http(#[from] http_client::error::Error),
}
