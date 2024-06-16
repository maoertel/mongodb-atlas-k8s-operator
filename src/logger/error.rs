use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;

use log::SetLoggerError;
use log4rs::config::runtime::ConfigErrors;

use crate::impl_from_error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Config(ConfigErrors),
    SetLogger(SetLoggerError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(e) => std::fmt::Display::fmt(e, f),
            Error::SetLogger(e) => std::fmt::Display::fmt(e, f),
        }
    }
}

impl_from_error!(ConfigErrors => Error, Config);
impl_from_error!(SetLoggerError => Error, SetLogger);
