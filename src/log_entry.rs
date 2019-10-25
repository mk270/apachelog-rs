use pom::DataInput;
use pom::parser::*;
use chrono::NaiveDateTime;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

pub struct ApacheLog {
    pub ip_address: String,
    identd: String,
    username: String,
    time: NaiveDateTime,
    request: String,
    status_code: i64,
    size: Option<i64>,
    referrer: String,
    user_agent: String
}

pub fn producer(line: &str) -> Result<ApacheLog, Error> {
    let parser =
        dotted_quad() +
        word() +
        word() +
        space() * bracketed() +
        space() * quoted() +
        word() +
        word() +
        space() * quoted() +
        space() * quoted();
    let mut input = DataInput::new(line.as_ref());

    let output = parser.parse(&mut input);

    if let Ok(((((((((ip_address, identd), username), time), request),
                  status_code), raw_size), referrer), user_agent)) = output {

        let size = match i64::from_str(&raw_size) {
            Ok(parse_size) => Some(parse_size),
            _ => None
        };

        return Ok(ApacheLog {
            ip_address: ip_address,
            identd: identd,
            username: username,
            time: NaiveDateTime::parse_from_str(
                &time, "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            request: request,
            status_code: i64::from_str(&status_code).unwrap(),
            size: size,
            referrer: referrer,
            user_agent: user_agent,
        });
    }

    Err(Error::from(ErrorKind::InvalidData))
}

// doesn't handle 
fn dotted_quad<'a>() -> Parser<'a, u8, String> {
    (one_of(b"0123456789").repeat(0..) + sym(b'.') +
     one_of(b"0123456789").repeat(0..) + sym(b'.') +
     one_of(b"0123456789").repeat(0..) + sym(b'.') +
     one_of(b"0123456789").repeat(0..))
        .collect().convert(String::from_utf8)
}

fn space<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t\r\n").repeat(0..).discard()
}

fn word<'a>() -> Parser<'a, u8, String> {
    space() * none_of(b" ").repeat(0..).convert(String::from_utf8)
}

fn bracketed<'a>() -> Parser<'a, u8, String> {
    (sym(b'[') * none_of(b"]").repeat(0..) - sym(b']').discard())
        .convert(String::from_utf8)
}

fn quoted<'a>() -> Parser<'a, u8, String> {
    (sym(b'"') * none_of(b"\"").repeat(0..) - sym(b'"').discard())
        .convert(String::from_utf8)
}

