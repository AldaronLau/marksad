use std::result;

use crate::decode::Error;

/// `Result` type alias for convenience
pub type Result<'a, T = (), E = Error<'a>> = result::Result<T, E>;
