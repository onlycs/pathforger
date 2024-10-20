use nt_client::publish::NewPublisherError;
use std::io;
use std::{backtrace::Backtrace, panic::Location};
use tokio::sync::broadcast::error::RecvError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PhotonWorkerError {
    #[error("At {location}: Networktables failed to recieve:\n{source}")]
    NTError {
        #[from]
        source: RecvError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: Networktables failed to publish:\n{source}")]
    NTPublishError {
        #[from]
        source: NewPublisherError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: IO error:\n{source}")]
    IOError {
        #[from]
        source: io::Error,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}
