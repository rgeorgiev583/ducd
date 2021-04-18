use log::error;
use std::{fmt::Display, result::Result};

pub(crate) fn log_error<T, E: Display>(result: Result<T, E>) {
    if let Err(err) = result {
        error!("{}", err);
    }
}
