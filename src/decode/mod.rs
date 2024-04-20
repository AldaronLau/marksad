//! Markdown file decoding

mod error;
mod result;
mod warning;

pub use self::{
    error::Error,
    result::Result,
    warning::{Warning, WarningKind},
};
