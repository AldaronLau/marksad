use std::io;

use crate::Warning;

#[derive(Debug)]
pub enum Error<'a> {
    Io(io::Error),
    Warning(Warning<'a>),
}
