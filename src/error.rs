use std::fmt;
use std::fmt::{Debug, Formatter};

use crate::{atlas, logger};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    K8s(kube::Error),
    Logger(logger::error::Error),
    Atlas(atlas::error::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::K8s(error) => write!(f, "K8s error: {error}"),
            Error::Logger(error) => write!(f, "Logger error: {error}"),
            Error::Atlas(error) => write!(f, "Atlas error: {error}"),
        }
    }
}

#[macro_export]
macro_rules! impl_from_error {
    ($source:ty => $target:ty, $variant:ident) => {
        impl From<$source> for $target {
            fn from(error: $source) -> Self {
                <$target>::$variant(error)
            }
        }
    };
}

impl_from_error!(kube::Error => Error, K8s);
impl_from_error!(logger::error::Error => Error, Logger);
impl_from_error!(atlas::error::Error => Error, Atlas);
