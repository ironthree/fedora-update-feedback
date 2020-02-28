#![warn(missing_docs)]
#![warn(clippy::result_unwrap_used)]
#![warn(clippy::option_unwrap_used)]

//! This crate contains helper functionality that's used by the `fedora-update-feedback` binary.
//! It's contents are probably not useful for external use. But if something turns out to be
//! generally useful, it can be upstreamed into either the [`fedora`][fedora-rs] or [`bodhi`][bodhi]
//! crates.
//!
//! [fedora-rs]: https://crates.io/crates/fedora
//! [bodhi-rs]: https://crates.io/crates/bodhi

use std::cmp::PartialEq;
use std::fmt::Display;

mod config;
pub use config::*;

mod ignore;
pub use ignore::*;

mod input;
pub use input::*;

mod output;
pub use output::*;

mod parse;
pub use parse::*;

mod query;
pub use query::*;

mod sysinfo;
pub use sysinfo::*;

/// This struct encapsulates a parsed NVR string.
#[derive(Debug, PartialEq)]
pub struct NVR {
    /// name of the package
    pub n: String,
    /// version of the package
    pub v: String,
    /// release of the package
    pub r: String,
}

impl Display for NVR {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.n, self.v, self.r)
    }
}
