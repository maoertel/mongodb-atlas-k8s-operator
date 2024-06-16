use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;

use reqwest::StatusCode;

use crate::impl_from_error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    K8s(kube::Error),
    Json(serde_json::Error),
    Reqwest(reqwest::Error),
    Diqwest(diqwest::error::Error),
    Api { status: StatusCode, message: String },
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::K8s(e) => std::fmt::Display::fmt(e, f),
            Error::Json(e) => std::fmt::Display::fmt(e, f),
            Error::Reqwest(e) => std::fmt::Display::fmt(e, f),
            Error::Diqwest(e) => std::fmt::Display::fmt(e, f),
            Error::Api { status, message } => write!(
                f,
                "Unexpected response from MongoDB Atlas API: {status}. Message: {message}."
            ),
        }
    }
}

impl_from_error!(kube::Error => Error, K8s);
impl_from_error!(serde_json::Error => Error, Json);
impl_from_error!(reqwest::Error => Error, Reqwest);
impl_from_error!(diqwest::error::Error => Error, Diqwest);
