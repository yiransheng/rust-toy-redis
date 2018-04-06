use bytes::{Bytes, BytesMut};
use std::convert::{AsRef, From, Into};
use std::str::{self, FromStr};
use std::mem;

type Range = ::std::ops::Range<usize>;

#[derive(Debug)]
pub enum Value<T> {
    SimpleString(T),
    ErrorString(T),
    IntegerString(T),
    BulkString(T),
    Nil,
}

impl<'a, T: Into<&'a [u8]>> Into<&'a [u8]> for Value<T> {
    fn into(self) -> &'a [u8] {
        static EMPTY: [u8; 0] = [];
        match self {
            Value::SimpleString(v) => v.into(),
            Value::ErrorString(v) => v.into(),
            Value::IntegerString(v) => v.into(),
            Value::BulkString(v) => v.into(),
            Value::Nil => &EMPTY,
        }
    }
}
impl<T> Value<T> {
    fn take(&mut self) -> Self {
        mem::replace(self, Value::Nil)
    }
}

#[derive(Debug)]
pub enum Node<T> {
    Leaf(Value<T>),
    Open(usize),
    Close,
}

#[derive(Debug)]
pub struct RedisValue {
    nodes: Vec<Node<Bytes>>,
}
impl RedisValue {
    #[inline]
    fn from_value(v: Value<Bytes>) -> Self {
        RedisValue {
            nodes: vec![Node::Leaf(v)],
        }
    }
}

#[derive(Debug)]
enum Values {
    One(Value<Range>),
    Many(Vec<Node<Range>>),
}
#[derive(Debug)]
enum DecodeError {
    Failed,
    Incomplete,
}

// number of bytes consumed, Values
type Decoded = (usize, Values);
type DecodeResult = ::std::result::Result<Decoded, DecodeError>;

fn decode_values_from_slice(src: &[u8]) -> DecodeResult {
    let len = src.len();

    if len < 4 {
        // needs at least prefix + '\r\n'
        // prefix = + | - | : | $ | *
        return Err(DecodeError::Incomplete);
    }
    // find \n position
    if let Some(n) = src.iter().position(|b| *b == b'\n') {
        // requires CLRF ending
        if n <= 2 || src[n - 1] != b'\r' {
            return Err(DecodeError::Failed);
        }
        match src[0] {
            b'*' => {
                let array_len = str::from_utf8(&src[1..n - 1]).map_err(|_| DecodeError::Failed)?;
                let array_len = usize::from_str(array_len).map_err(|_| DecodeError::Failed)?;

                if len > n + 1 {
                    // more bytes avaiable
                    let mut nodes: Vec<Node<Range>> = Vec::with_capacity(32);
                    let mut index: usize = n + 1;
                    nodes.push(Node::Open(array_len));
                    for _ in 0..array_len {
                        // decode one
                        let (consumed, result) = decode_values_from_slice(&src[index..])?;
                        match result {
                            Values::One(value) => nodes.push(Node::Leaf(value)),
                            Values::Many(mut inner_nodes) => for n in inner_nodes.drain(..) {
                                nodes.push(n);
                            },
                        }
                        index = index + consumed;
                    }
                    nodes.push(Node::Close);
                    Ok((index, Values::Many(nodes)))
                } else {
                    // not enough bytes
                    Err(DecodeError::Incomplete)
                }
            }
            _ => decode_one(src),
        }
    } else {
        Err(DecodeError::Incomplete)
    }
}

fn decode_one(src: &[u8]) -> DecodeResult {
    let len = src.len();

    if len < 4 {
        // needs at least prefix + '\r\n'
        // prefix = + | - | : | $ | *
        return Err(DecodeError::Incomplete);
    }
    // find \n position
    if let Some(n) = src.iter().position(|b| *b == b'\n') {
        // requires CLRF ending
        if n <= 2 || src[n - 1] != b'\r' {
            return Err(DecodeError::Failed);
        }
        match src[0] {
            b'+' => {
                let value = Value::SimpleString(1..n - 1);
                Ok((n + 1, Values::One(value)))
            }
            b'-' => {
                let value = Value::ErrorString(1..n - 1);
                Ok((n + 1, Values::One(value)))
            }
            b':' => {
                let value = Value::IntegerString(1..n - 1);
                Ok((n + 1, Values::One(value)))
            }
            b'$' => {
                let bulk_len = str::from_utf8(&src[1..n - 1]).map_err(|_| DecodeError::Failed)?;
                let bulk_len = isize::from_str(bulk_len).map_err(|_| DecodeError::Failed)?;

                // Nil
                if bulk_len == -1 {
                    Ok((n + 1, Values::One(Value::Nil)))
                // Negative length other than -1
                } else if bulk_len < 0 {
                    Err(DecodeError::Failed)
                } else {
                    let bulk_len = bulk_len as usize;
                    // prefix(n+1) + bulk_len + 2 bytes CRLF
                    if len >= bulk_len + n + 3 {
                        let ending = &src[n + bulk_len + 1..n + bulk_len + 3];
                        if ending != b"\r\n" {
                            Err(DecodeError::Failed)
                        } else {
                            let value = Value::BulkString(n + 1..n + bulk_len + 1);
                            // n + 1 prefix, bulk_len bytes, 2 bytes line ending
                            Ok((n + bulk_len + 3, Values::One(value)))
                        }
                    } else {
                        // bulk string not ready
                        Err(DecodeError::Incomplete)
                    }
                }
            }
            b'*' => decode_values_from_slice(&src[n + 1..]),
            _ => Err(DecodeError::Failed),
        }
    } else {
        Err(DecodeError::Incomplete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_ok() {
        let ok_tests: Vec<&str> = vec![
            "+Ok\r\n",
            "-MESSAGE error happenend\r\n",
            ":12\r\n",
            "$6\r\nfoobar\r\n",
            "$8\r\nfoo\r\nbar\r\n",
            "$-1\r\n",
            "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n",
        ];
        for raw in &ok_tests {
            let result = decode_values_from_slice(raw.as_bytes());
            let (consumed, r) = result.unwrap();
            assert_eq!(consumed, raw.as_bytes().len());
        }
    }

    #[test]
    fn test_decode_incomplete() {
        let incomplete_tests: Vec<&str> = vec![
            "+Ok\r",
            "-MESSAGE error happ",
            ":",
            "$6\r\n",
            "$8\r\nfoo\r",
            "$-1\r",
            "*2\r\n$3\r\nfoo\r\n",
        ];
        for raw in &incomplete_tests {
            let result = decode_values_from_slice(raw.as_bytes());
            assert_matches!(result, Err(DecodeError::Incomplete));
        }
    }
}
