//! A rust library to communicate with the
//! SOEHNLE Terminal 3005 (via RS232).

use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Message {
    pub status: Status,
    pub nr: u8,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct Status {
    pub under_load: bool,
    pub over_load: bool,
    pub standstill: bool,
    pub empty_message: bool,
}

impl FromStr for Status {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 4 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }
        if !s.is_ascii() {
            return Err(Error::new(ErrorKind::InvalidData, "Non-ASCII str"));
        }
        let (under_load, tail) = s.split_at(1);
        let (over_load, tail) = tail.split_at(1);
        let (standstill, tail) = tail.split_at(1);
        let (empty_message, _) = tail.split_at(1);

        Ok(Status {
            under_load: bool_from_str(under_load)?,
            over_load: bool_from_str(over_load)?,
            standstill: bool_from_str(standstill)?,
            empty_message: bool_from_str(empty_message)?,
        })
    }
}

fn bool_from_str(s: &str) -> Result<bool> {
    match s {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            "Could not parse boolean",
        )),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_bool_str() {
        assert_eq!(bool_from_str("1").unwrap(), true);
        assert_eq!(bool_from_str("0").unwrap(), false);
        assert!(bool_from_str("").is_err());
        assert!(bool_from_str("x").is_err());
        assert!(bool_from_str("o").is_err());
        assert!(bool_from_str("O").is_err());
        assert!(bool_from_str("l").is_err());
    }

    #[test]
    fn parse_status() {
        let s = Status::from_str("0000").unwrap();
        assert_eq!(s.under_load, false);
        assert_eq!(s.over_load, false);
        assert_eq!(s.standstill, false);
        assert_eq!(s.empty_message, false);

        let s = Status::from_str("1000").unwrap();
        assert_eq!(s.under_load, true);
        assert_eq!(s.over_load, false);
        assert_eq!(s.standstill, false);
        assert_eq!(s.empty_message, false);

        let s = Status::from_str("0100").unwrap();
        assert_eq!(s.under_load, false);
        assert_eq!(s.over_load, true);
        assert_eq!(s.standstill, false);
        assert_eq!(s.empty_message, false);

        let s = Status::from_str("0010").unwrap();
        assert_eq!(s.under_load, false);
        assert_eq!(s.over_load, false);
        assert_eq!(s.standstill, true);
        assert_eq!(s.empty_message, false);

        let s = Status::from_str("0001").unwrap();
        assert_eq!(s.under_load, false);
        assert_eq!(s.over_load, false);
        assert_eq!(s.standstill, false);
        assert_eq!(s.empty_message, true);
    }

    #[test]
    fn parse_status_from_bad_str() {
        assert!(Status::from_str("").is_err());
        assert!(Status::from_str("� ").is_err());
        assert!(Status::from_str("�ۿ�").is_err());
    }
}
