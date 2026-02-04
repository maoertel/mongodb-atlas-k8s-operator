use std::fmt::Debug;

use thiserror::Error as ThisError;

use crate::{atlas, config};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Kubernetes reported error: {0}")]
    K8s(#[from] kube::Error),
    #[error("Atlas reported error: {0}")]
    Atlas(#[from] atlas::error::Error),
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
}
