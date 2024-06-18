use crate::atlas;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    KubeError {
        #[from]
        source: kube::Error,
    },
    #[error("Invalid AtlasUser CRD: {0}")]
    UserInputError(String),
    #[error("Atlas reported error: {source}")]
    Atlas {
        #[from]
        source: atlas::error::Error,
    },
}
