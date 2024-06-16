use std::fmt;
use std::fmt::Formatter;

use crate::impl_from_error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidHeader(reqwest::header::InvalidHeaderValue),
    Http(reqwest::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidHeader(e) => std::fmt::Display::fmt(e, f),
            Error::Http(e) => std::fmt::Display::fmt(e, f),
        }
    }
}

macro_rules! impl_from_for_http_client_error {
    ($error:ty, $variant:ident) => {
        impl_from_error!($error => Error, $variant);
    }
}

impl_from_for_http_client_error!(reqwest::header::InvalidHeaderValue, InvalidHeader);
impl_from_for_http_client_error!(reqwest::Error, Http);
