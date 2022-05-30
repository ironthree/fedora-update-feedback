use std::cmp::PartialEq;
use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

use crate::parse::parse_nvr;

/// This struct encapsulates a parsed NVR string.
#[derive(Debug, Eq, PartialEq)]
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

impl TryFrom<&str> for NVR {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (n, v, r) = parse_nvr(value)?;
        Ok(NVR {
            n: n.to_owned(),
            v: v.to_owned(),
            r: r.to_owned(),
        })
    }
}

impl FromStr for NVR {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::try_from(s)
    }
}
