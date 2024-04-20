use std::result;

use crate::Error;

pub type Result<'a, T = (), E = Error<'a>> = result::Result<T, E>;
