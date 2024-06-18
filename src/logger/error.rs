use std::fmt::Debug;

use log::SetLoggerError;
use log4rs::config::runtime::ConfigErrors;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigErrors),
    #[error("Logger error: {0}")]
    SetLogger(#[from] SetLoggerError),
}
