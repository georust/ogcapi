mod bbox;
mod conformance;
mod datetime;
mod exception;
mod landingpage;
mod link;
mod mediatype;

pub use bbox::Bbox;
pub use conformance::Conformance;
pub use datetime::Datetime;
pub use exception::Exception;
pub use landingpage::LandingPage;
pub use link::{Link, LinkRel, Links};
pub use mediatype::MediaType;

use std::{fmt, str};

pub struct ListParam(Vec<String>);

impl fmt::Display for ListParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

impl str::FromStr for ListParam {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ListParam(
            s.split(',').map(|s| s.trim().to_owned()).collect(),
        ))
    }
}

impl From<&[&str]> for ListParam {
    fn from(l: &[&str]) -> Self {
        ListParam(l.iter().map(|s| s.trim().to_string()).collect())
    }
}
