//! A rust library to communicate with the
//! SOEHNLE Terminal 3005 (via RS232).

use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

/// A message received from the terminal.
#[derive(Debug, Clone)]
pub struct Message {
    pub status: Status,
    pub id: u8,
    pub value: f32,
}

/// Balance status.
#[derive(Debug, Clone)]
pub struct Status {
    pub under_load: bool,
    pub over_load: bool,
    pub standstill: bool,
    pub empty_message: bool,
}

impl FromStr for Message {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        if (s.len() > 27) || (s.len() < 7) {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }
        if !s.is_ascii() {
            return Err(Error::new(ErrorKind::InvalidData, "None-ASCII str"));
        }
        let (status, tail) = s.split_at(4);
        let (id, netto) = tail.split_at(2);
        let v = netto
            .replace("N", "")
            .replace("kg", "")
            .replace(" ", "")
            .replace(",", ".");

        Ok(Message {
            status: Status::from_str(status)?,
            id: id.replace("W", "")
                .parse()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid balance ID"))?,
            value: v.trim()
                .parse()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid balance ID"))?,
        })
    }
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
    fn parse_value_from_message() {
        assert_eq!(
            Message::from_str("000101N        3,1 kg").unwrap().value,
            3.1
        );
        assert_eq!(
            Message::from_str("000101N    -   3,7 kg").unwrap().value,
            -3.7
        );
        assert_eq!(
            Message::from_str("000101N       -3,9 kg").unwrap().value,
            -3.9
        );
        assert_eq!(
            Message::from_str("000000N 0123456,78kg").unwrap().value,
            123456.78
        );
    }

    #[test]
    fn parse_id_from_message() {
        assert!(Message::from_str("0000XXN    -1000,0 kg").is_err());

        assert_eq!(Message::from_str("000003N    -1000,0 kg").unwrap().id, 3);
        assert_eq!(Message::from_str("000000N    -1000,0 kg").unwrap().id, 0);
        assert_eq!(Message::from_str("000099N    -1000,0 kg").unwrap().id, 99);
        assert_eq!(Message::from_str("0000W9N    -1000,0 kg").unwrap().id, 9);
    }

    #[test]
    fn parse_message_from_bad_str() {
        assert!(Message::from_str("").is_err());
        assert!(Message::from_str("Foo x").is_err());
        assert!(Message::from_str("0000W1").is_err());
        assert!(Message::from_str("None Sense").is_err());
        assert!(Message::from_str("000�ۿ3,9 kg").is_err());
    }

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
