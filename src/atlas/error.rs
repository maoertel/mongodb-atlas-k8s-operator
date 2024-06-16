use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;

use crate::impl_from_error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    K8s(kube::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::K8s(e) => std::fmt::Display::fmt(e, f),
        }
    }
}

impl_from_error!(kube::Error => Error, K8s);
