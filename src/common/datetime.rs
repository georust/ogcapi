use chrono::{DateTime, FixedOffset, SecondsFormat};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Datetime {
    Datetime(DateTime<FixedOffset>),
    Interval(Interval),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interval {
    from: IntervalDatetime,
    to: IntervalDatetime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum IntervalDatetime {
    DateTime(DateTime<FixedOffset>),
    Open,
}

impl FromStr for IntervalDatetime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            ".." => IntervalDatetime::Open,
            _ => IntervalDatetime::DateTime(DateTime::parse_from_rfc3339(s)?),
        })
    }
}

impl fmt::Display for Datetime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Datetime::Datetime(datetime) => {
                write!(f, "{}", datetime.to_rfc3339_opts(SecondsFormat::Secs, true))
            }
            Datetime::Interval(interval) => write!(
                f,
                "{from}/{to}",
                from = match interval.from {
                    IntervalDatetime::DateTime(d) => d.to_rfc3339_opts(SecondsFormat::Secs, true),
                    IntervalDatetime::Open => "..".to_owned(),
                },
                to = match interval.to {
                    IntervalDatetime::DateTime(d) => d.to_rfc3339_opts(SecondsFormat::Secs, true),
                    IntervalDatetime::Open => "..".to_owned(),
                }
            ),
        }
    }
}

impl FromStr for Datetime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("/") {
            let datetimes: Vec<&str> = s.split("/").collect();
            Ok(Datetime::Interval(Interval {
                from: IntervalDatetime::from_str(datetimes[0])?,
                to: IntervalDatetime::from_str(datetimes[1])?,
            }))
        } else {
            Ok(Datetime::Datetime(DateTime::parse_from_rfc3339(s)?))
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
