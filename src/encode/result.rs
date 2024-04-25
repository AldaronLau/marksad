use std::result;

use crate::encode::Error;

/// `Result` type alias for convenience
pub type Result<T = (), E = Error> = result::Result<T, E>;
