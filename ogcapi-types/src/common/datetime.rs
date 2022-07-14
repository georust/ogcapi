use std::str::FromStr;
use std::{cmp::Ordering, fmt};

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Datetime {
    Datetime(DateTime<Utc>),
    Interval {
        from: IntervalDatetime,
        to: IntervalDatetime,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum IntervalDatetime {
    Datetime(DateTime<Utc>),
    Open,
}

impl FromStr for IntervalDatetime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            ".." | "" => IntervalDatetime::Open,
            d => IntervalDatetime::Datetime(DateTime::parse_from_rfc3339(d)?.into()),
        })
    }
}

impl fmt::Display for IntervalDatetime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntervalDatetime::Datetime(d) => {
                write!(f, "{}", d.to_rfc3339_opts(SecondsFormat::Secs, true))
            }
            IntervalDatetime::Open => write!(f, ".."),
        }
    }
}

impl fmt::Display for Datetime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Datetime::Datetime(datetime) => {
                write!(f, "{}", datetime.to_rfc3339_opts(SecondsFormat::Secs, true))
            }
            Datetime::Interval { from, to } => write!(f, "{}/{}", from, to),
        }
    }
}

impl FromStr for Datetime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('/') && !["../..", "../", "/..", "/"].contains(&s.trim()) {
            let mut datetimes = s.trim().splitn(2, '/');

            let from = datetimes.next().unwrap_or_default().parse()?;
            let to = datetimes.next().unwrap_or_default().parse()?;

            Ok(Datetime::Interval { from, to })
        } else {
            Ok(Datetime::Datetime(DateTime::parse_from_rfc3339(s)?.into()))
        }
    }
}

impl PartialOrd for IntervalDatetime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            IntervalDatetime::Datetime(d) => match other {
                IntervalDatetime::Datetime(o) => d.partial_cmp(o),
                IntervalDatetime::Open => Some(Ordering::Less),
            },
            IntervalDatetime::Open => Some(Ordering::Less),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Datetime;
    use std::str::FromStr;

    #[test]
    fn parse_datetime() {
        let datetime_str = "2018-02-12T23:20:52Z";
        let datetime = Datetime::from_str(datetime_str).unwrap();
        assert_eq!(format!("{:#}", datetime), datetime_str)
    }

    #[test]
    fn parse_intervals() {
        let interval_str = "2018-02-12T00:00:00Z/2018-03-18T12:31:12Z";
        let datetime = Datetime::from_str(interval_str).unwrap();
        assert_eq!(format!("{:#}", datetime), interval_str);

        let interval_str = "2018-02-12T00:00:00Z/..";
        let datetime = Datetime::from_str(interval_str).unwrap();
        assert_eq!(format!("{:#}", datetime), interval_str);

        let interval_str = "../2018-03-18T12:31:12Z";
        let datetime = Datetime::from_str(interval_str).unwrap();
        assert_eq!(format!("{:#}", datetime), interval_str)
    }
}
