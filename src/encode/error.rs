use std::io;

/// An I/O error
#[derive(Debug)]
pub enum Error {
    /// I/O or invalid UTF-8 error
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}
