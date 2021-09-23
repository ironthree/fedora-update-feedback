use std::cmp::PartialEq;
use std::fmt::Display;

/// This struct encapsulates a parsed NVR string.
#[derive(Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
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
