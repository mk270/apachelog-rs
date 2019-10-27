use pom::DataInput;
use pom::parser::*;
use chrono::NaiveDateTime;
use std::io::{Error, ErrorKind};
use std::str::FromStr;
use std::fmt;

pub struct ApacheLog {
    source: String,
    timestamp: NaiveDateTime,
    method: String,
    uri: String,
    status_code: i64,
    content_length: Option<i64>,
    referrer: String,
    user_agent: String
}

impl fmt::Display for ApacheLog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n {}\n {} {}\n {}\n {}\n {}\n {}\n", 
               self.timestamp,
               self.source, self.method, self.uri,
               self.status_code,
               self.content_length.unwrap_or(0),
               self.referrer, self.user_agent)
    }
}

pub fn parse_string(s: &str) -> Result<ApacheLog, Error> {
    let p =
        dotted_quad() +
        word() +
        word() +
        space() * bracketed() +
        space() * quoted() +
        word() +
        word() +
        space() * quoted() +
        space() * quoted();

    let mut data = DataInput::new(s.as_ref());
    let result = p.parse(&mut data);

    if let Ok(((((((((source,
                      _identuser),
                     _authuser),
                    timestamp),
                   request),
                  status_code),
                 raw_content_length),
                referrer),
               user_agent)) = result {

        let content_length = match i64::from_str(&raw_content_length) {
            Ok(parse_size) => Some(parse_size),
            _ => None
        };

        let split = request.split(" ");
        let words = split.collect::<Vec<&str>>();

        let (method, uri, _http_version) = match words[..] {
            [m, u, v] => { (m.to_uppercase(), u, Some(v)) }
            [m, u]    => { (m.to_uppercase(), u, None)    }
            _  => { return Err(Error::from(ErrorKind::InvalidData)) }
        };

        return Ok(ApacheLog {
            source: source,
            timestamp: NaiveDateTime::parse_from_str(
                &timestamp, "%d/%b/%Y:%H:%M:%S %z").unwrap(),
            method: method,
            uri: uri.to_string(),
            status_code: i64::from_str(&status_code).unwrap(),
            content_length: content_length,
            referrer: referrer,
            user_agent: user_agent,
        });
    } else {
        Err(Error::from(ErrorKind::InvalidData))
    }
}

// doesn't handle leading zeros, values > 255, etc
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

