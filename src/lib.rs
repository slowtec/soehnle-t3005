//! A rust library to communicate with the
//! SOEHNLE Terminal 3005 (via RS232).

use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

const ACK: u8 = 0x06;
const NAK: u8 = 0x15;

/// A message received from the terminal
/// (at the moment only `30XX EDV Standard` is supported).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Message {
    pub status: Status,
    pub id: u8,
    pub value: f32,
}

/// A Command/Query response.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Response {
    Ack,
    Nak,
    Message(Message),
}

/// Balance status.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Status {
    pub under_load: bool,
    pub over_load: bool,
    pub standstill: bool,
    pub empty_message: bool,
}

/// Balance command
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Tare,
    ClearTare,
    SetTare(u32),
}

/// Command or Query with ACK
pub struct WithAck<T>(T);

/// Balance query command
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Query {
    Once,
    OnceOnChange,
}

pub trait ToAsciiString {
    fn to_ascii_string(&self) -> Result<String>;
}

impl Command {
    pub fn with_ack(self) -> WithAck<Command> {
        WithAck(self)
    }
}

impl ToAsciiString for Command {
    fn to_ascii_string(&self) -> Result<String> {
        use self::Command::*;
        let string = match *self {
            Tare => "<T>".into(),
            ClearTare => "<TC>".into(),
            SetTare(val) => {
                if val > 9999999 {
                    return Err(Error::new(ErrorKind::InvalidInput, "Invalid tare value"));
                }
                format!("<T{:07}>", val)
            }
        };
        Ok(string)
    }
}

impl ToAsciiString for WithAck<Command> {
    fn to_ascii_string(&self) -> Result<String> {
        use self::Command::*;
        let string = match self.0 {
            Tare => "<t>".into(),
            ClearTare => "<tC>".into(),
            SetTare(val) => {
                if val > 9999999 {
                    return Err(Error::new(ErrorKind::InvalidInput, "Invalid tare value"));
                }
                format!("<t{:07}>", val)
            }
        };
        Ok(string)
    }
}

impl Query {
    pub fn with_ack(self) -> WithAck<Query> {
        WithAck(self)
    }
}

impl ToAsciiString for Query {
    fn to_ascii_string(&self) -> Result<String> {
        use self::Query::*;
        let string = match *self {
            Once => "<A>".into(),
            OnceOnChange => "<B>".into(),
        };
        Ok(string)
    }
}

impl ToAsciiString for WithAck<Query> {
    fn to_ascii_string(&self) -> Result<String> {
        use self::Query::*;
        let string = match self.0 {
            Once => "<a>".into(),
            OnceOnChange => "<b>".into(),
        };
        Ok(string)
    }
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
            id: id
                .replace("W", "")
                .parse()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid balance ID"))?,
            value: v
                .trim()
                .parse()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid balance ID"))?,
        })
    }
}

impl FromStr for Response {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.len() < 1 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid message length"));
        }
        if s.as_bytes()[0] == ACK {
            return Ok(Response::Ack);
        }
        if s.as_bytes()[0] == NAK {
            return Ok(Response::Nak);
        }
        let msg = Message::from_str(s)?;
        Ok(Response::Message(msg))
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
        assert_eq!(
            Message::from_str("001101N     -0,001 kg ").unwrap().value,
            -0.001
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
    fn parse_response() {
        assert!(Response::from_str("").is_err());
        assert_eq!(
            Response::from_str(::std::str::from_utf8(&[0x06]).unwrap()).unwrap(),
            Response::Ack
        );
        assert_eq!(
            Response::from_str(::std::str::from_utf8(&[0x15]).unwrap()).unwrap(),
            Response::Nak
        );
        assert_eq!(
            Response::from_str("0000W9N    -1000,0 kg").unwrap(),
            Response::Message(Message {
                id: 9,
                status: Status {
                    empty_message: false,
                    over_load: false,
                    under_load: false,
                    standstill: false,
                },
                value: -1000.0
            })
        );
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

    #[test]
    fn command_to_ascii_string() {
        assert_eq!(Command::Tare.to_ascii_string().unwrap(), "<T>");
        assert_eq!(Command::ClearTare.to_ascii_string().unwrap(), "<TC>");
        assert_eq!(Command::SetTare(0).to_ascii_string().unwrap(), "<T0000000>");
        assert_eq!(
            Command::SetTare(9999999).to_ascii_string().unwrap(),
            "<T9999999>"
        );
        assert_eq!(
            Command::SetTare(1234567).to_ascii_string().unwrap(),
            "<T1234567>"
        );
        assert!(Command::SetTare(99999999).to_ascii_string().is_err());
    }

    #[test]
    fn command_with_ack_to_ascii_string() {
        assert_eq!(Command::Tare.with_ack().to_ascii_string().unwrap(), "<t>");
        assert_eq!(
            Command::ClearTare.with_ack().to_ascii_string().unwrap(),
            "<tC>"
        );
        assert_eq!(
            Command::SetTare(0).with_ack().to_ascii_string().unwrap(),
            "<t0000000>"
        );
        assert_eq!(
            Command::SetTare(9999999)
                .with_ack()
                .to_ascii_string()
                .unwrap(),
            "<t9999999>"
        );
        assert_eq!(
            Command::SetTare(1234567)
                .with_ack()
                .to_ascii_string()
                .unwrap(),
            "<t1234567>"
        );
        assert!(
            Command::SetTare(99999999)
                .with_ack()
                .to_ascii_string()
                .is_err()
        );
    }

    #[test]
    fn query_to_ascii_string() {
        assert_eq!(Query::Once.to_ascii_string().unwrap(), "<A>");
        assert_eq!(Query::OnceOnChange.to_ascii_string().unwrap(), "<B>");
    }

    #[test]
    fn query_with_ack_to_ascii_string() {
        assert_eq!(Query::Once.with_ack().to_ascii_string().unwrap(), "<a>");
        assert_eq!(
            Query::OnceOnChange.with_ack().to_ascii_string().unwrap(),
            "<b>"
        );
    }
}
