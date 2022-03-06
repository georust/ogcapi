use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Datetime {
    Datetime(DateTime<Utc>),
    Interval {
        from: IntervalDatetime,
        to: IntervalDatetime,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum IntervalDatetime {
    Datetime(DateTime<Utc>),
    Open,
}

impl FromStr for IntervalDatetime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            ".." => IntervalDatetime::Open,
            _ => IntervalDatetime::Datetime(DateTime::parse_from_rfc3339(s)?.into()),
        })
    }
}

impl fmt::Display for Datetime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Datetime::Datetime(datetime) => {
                write!(f, "{}", datetime.to_rfc3339_opts(SecondsFormat::Secs, true))
            }
            Datetime::Interval { from, to } => write!(
                f,
                "{from}/{to}",
                from = match from {
                    IntervalDatetime::Datetime(d) => d.to_rfc3339_opts(SecondsFormat::Secs, true),
                    IntervalDatetime::Open => "..".to_owned(),
                },
                to = match to {
                    IntervalDatetime::Datetime(d) => d.to_rfc3339_opts(SecondsFormat::Secs, true),
                    IntervalDatetime::Open => "..".to_owned(),
                }
            ),
        }
    }
}

impl FromStr for Datetime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('/') {
            let datetimes: Vec<&str> = s.split('/').collect();
            Ok(Datetime::Interval {
                from: IntervalDatetime::from_str(datetimes[0])?,
                to: IntervalDatetime::from_str(datetimes[1])?,
            })
        } else {
            Ok(Datetime::Datetime(DateTime::parse_from_rfc3339(s)?.into()))
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
