use std::io;

use crate::decode::Warning;

/// An I/O error or Markdown decoding warning
#[derive(Debug)]
pub enum Error<'a> {
    /// I/O or invalid UTF-8 error
    Io(io::Error),
    /// Recoverable warning parsing the markdown
    Warning(Warning<'a>),
}
