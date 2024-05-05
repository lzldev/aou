use crate::{
  request::{HeaderParseError, RequestHeaderParser},
  utils::range_from_subslice,
};
use anyhow::anyhow;

use super::{RequestHead, RequestHeaders, VecOffset};

#[derive(Debug)]
pub struct RequestParser {
  pub buf: Vec<u8>,
  head: RequestHead,
  headers: RequestHeaders,
  body: VecOffset,
}

#[derive(Debug)]
pub enum ParserState {
  Start {
    read_until: Option<usize>,
  },
  Head {
    cursor: usize,
    read_until: usize,
    head: RequestHead,
  },
  Headers {
    cursor: usize,
    read_until: usize,
    head: RequestHead,
    headers: RequestHeaders,
  },
  Body {
    cursor: usize,
    read_until: usize,
    head: RequestHead,
    headers: RequestHeaders,
    body: VecOffset,
  },
}

impl ParserState {
  pub fn read_until(&self) -> usize {
    match self {
      ParserState::Start { read_until } => read_until.unwrap_or(0),
      ParserState::Head { read_until, .. } => *read_until,
      ParserState::Headers { read_until, .. } => *read_until,
      ParserState::Body { read_until, .. } => *read_until,
    }
  }
}

#[derive(Debug)]
pub enum RequestParseResponse {
  Success(RequestParser),
  Incomplete((Vec<u8>, ParserState)),
}

impl RequestParser {
  pub fn parse_request(
    buf: Vec<u8>,
    state: ParserState,
  ) -> Result<RequestParseResponse, anyhow::Error> {
    let buf_len = buf.len();
    let mut offset: usize = 0;
    let mut lines = buf.split(|b| b == &b'\n');

    let head = match RequestHead::from_split_iter(&mut lines, &buf) {
      Ok((size, head)) => {
        offset = offset + size;
        head
      }
      Err(_) => {
        return Ok(RequestParseResponse::Incomplete((
          buf,
          ParserState::Start {
            read_until: Some(buf_len),
          },
        )));
      }
    };

    let headers = match RequestHeaderParser::parse_headers(&buf, lines) {
      Ok((size, headers)) => {
        offset = offset + size;
        headers
      }
      Err(HeaderParseError::Incomplete) => {
        return Ok(RequestParseResponse::Incomplete((
          buf,
          ParserState::Head {
            cursor: offset,
            read_until: buf_len,
            head,
          },
        )))
      }
      Err(HeaderParseError::Invalid) => return Err(anyhow!("Invalid Headers")),
    };

    let buf_len = buf.len();

    debug_assert!(
      offset <= buf_len,
      "Buf:{:#?}\nOffset larger than buffer size : Offset {offset} : Buf {buf_len} Headers:{}",
      String::from_utf8_lossy(buf.as_slice()),
      headers.len()
    );

    let body = &buf[offset..];
    let body = range_from_subslice(&buf, body);

    let req = RequestParser {
      buf,
      head,
      headers,
      body,
    };

    Ok(RequestParseResponse::Success(req))
  }
}

#[cfg(test)]
mod unit_tests {
  use crate::request::{ParserState, RequestParseResponse, RequestParser};
  #[tokio::test]
  async fn invalid_http_version() {
    let buf =
      b"GET / HTTP/1.0\r\nHost: localhost:3000\r\nThe empty line before the body is missing";

    let parse = RequestParser::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      parse.is_err(),
      "Parse should return a Invalid HTTP Version Error Result"
    );
  }

  #[tokio::test]
  async fn invalid_header_termination() {
    let buf =
      b"GET / HTTP/1.1\r\nHost: localhost:3000\r\nThe empty line before the body is missing";

    let parse = RequestParser::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      parse.is_ok(),
      "Parse should not error since it's not sure it's the end of the headers."
    ); //Check the spec for spaces in header names

    let parse = parse.unwrap();

    let is_incomplete = match parse {
      RequestParseResponse::Incomplete(_) => true,
      RequestParseResponse::Success(_) => false,
    };

    assert!(is_incomplete, "Parser should return a Incomplete result");
  }
}
