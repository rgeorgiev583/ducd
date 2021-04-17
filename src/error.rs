use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    WalkdirError(walkdir::Error),
    HotwatchError(hotwatch::Error),
    VarlinkError(varlink::Error),
    DucdError(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "{}", err),
            Error::WalkdirError(err) => write!(f, "{}", err),
            Error::HotwatchError(err) => write!(f, "{}", err),
            Error::VarlinkError(err) => write!(f, "{}", err),
            Error::DucdError(err) => write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Self {
        Error::WalkdirError(err)
    }
}

impl From<hotwatch::Error> for Error {
    fn from(err: hotwatch::Error) -> Self {
        Error::HotwatchError(err)
    }
}

impl From<varlink::Error> for Error {
    fn from(err: varlink::Error) -> Self {
        Error::VarlinkError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
