use bytes::{BufMut, Bytes, BytesMut};
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
impl<T> Value<T> {
    fn map<R, F>(self, f: F) -> Value<R>
    where
        F: FnOnce(T) -> R,
    {
        match self {
            Value::SimpleString(x) => Value::SimpleString(f(x)),
            Value::ErrorString(x) => Value::ErrorString(f(x)),
            Value::IntegerString(x) => Value::IntegerString(f(x)),
            Value::BulkString(x) => Value::BulkString(f(x)),
            Value::Nil => Value::Nil,
        }
    }

    pub fn as_ref(&self) -> Value<&T> {
        match self {
            &Value::SimpleString(ref x) => Value::SimpleString(x),
            &Value::ErrorString(ref x) => Value::ErrorString(x),
            &Value::IntegerString(ref x) => Value::IntegerString(x),
            &Value::BulkString(ref x) => Value::BulkString(x),
            _ => Value::Nil,
        }
    }
    pub fn into_option(self) -> Option<T> {
        match self {
            Value::SimpleString(x) => Some(x),
            Value::ErrorString(x) => Some(x),
            Value::IntegerString(x) => Some(x),
            Value::BulkString(x) => Some(x),
            Value::Nil => None,
        }
    }
    pub fn as_option(&self) -> Option<&T> {
        self.as_ref().into_option()
    }
    pub fn iter(&self) -> ValueIter<T> {
        ValueIter {
            value: self.as_ref().into_option(),
        }
    }
}

pub struct ValueIter<'a, T: 'a> {
    value: Option<&'a T>,
}
impl<'a, T> Iterator for ValueIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.value.take()
    }
}
impl<T> Value<T> {
    #[allow(dead_code)]
    pub fn take(&mut self) -> Self {
        mem::replace(self, Value::Nil)
    }
}
impl<T: AsRef<[u8]>> Value<T> {
    pub fn as_slice(&self) -> &[u8] {
        static EMPTY: [u8; 0] = [];
        match self {
            &Value::SimpleString(ref v) => v.as_ref(),
            &Value::ErrorString(ref v) => v.as_ref(),
            &Value::IntegerString(ref v) => v.as_ref(),
            &Value::BulkString(ref v) => v.as_ref(),
            &Value::Nil => &EMPTY,
        }
    }
}
impl Value<Bytes> {
    pub fn from_slice(s: &[u8]) -> Self {
        if s.len() > 0 {
            Value::BulkString(Bytes::from(s))
        } else {
            Value::Nil
        }
    }
    pub fn from_integer<N: Into<i64>>(n: N) -> Self {
        let n = n.into();
        let number = format!("{}", n).into_bytes();
        Value::IntegerString(Bytes::from(number))
    }
    pub fn from_error(e: &str) -> Self {
        Value::ErrorString(Bytes::from(e))
    }
    pub fn size(&self) -> usize {
        let content_len = self.as_option().map_or(0, |b| b.len());
        match self {
            &Value::Nil => "$-1\r\n".len(),
            // $ (1 byte) + usize_size(content_len) + \r\n (2) + contents + \r\n (2)
            &Value::BulkString(_) => 1 + usize_string_size(content_len) + 2 + content_len + 2,
            // + (1 byte) + content_len + \r\n (2 bytes)
            _ => 1 + content_len + 2,
        }
    }
    // may panic assume buf has enough capacity (thus should not make this fn public)
    fn encode(&self, buf: &mut BytesMut) {
        match self {
            &Value::Nil => {
                buf.put("$-1\r\n");
            }
            &Value::SimpleString(ref b) => {
                buf.put(b'+');
                buf.extend_from_slice(&b[..]);
                buf.put("\r\n");
            }
            &Value::ErrorString(ref b) => {
                buf.put(b'-');
                buf.extend_from_slice(&b[..]);
                buf.put("\r\n");
            }
            &Value::IntegerString(ref b) => {
                buf.put(b':');
                buf.extend_from_slice(&b[..]);
                buf.put("\r\n");
            }
            &Value::BulkString(ref b) => {
                buf.put(b'$');
                buf.put(format!("{}", b.len()));
                buf.put("\r\n");
                buf.extend_from_slice(&b[..]);
                buf.put("\r\n");
            }
        }
    }
}

#[derive(Debug)]
pub enum Node<T> {
    Leaf(Value<T>),
    Open(usize),
    Close,
}
impl<T> Node<T> {
    #[allow(dead_code)]
    fn map<R, F>(self, f: F) -> Node<R>
    where
        F: FnOnce(T) -> R,
    {
        match self {
            Node::Leaf(v) => Node::Leaf(v.map(f)),
            Node::Open(n) => Node::Open(n),
            Node::Close => Node::Close,
        }
    }
    fn value_iter(&self) -> ValueIter<T> {
        match self {
            &Node::Leaf(ref x) => x.iter(),
            _ => ValueIter { value: None },
        }
    }
}

#[derive(Debug)]
pub struct RedisValue {
    pub nodes: Vec<Node<Bytes>>,
}
impl RedisValue {
    pub fn ok() -> Self {
        RedisValue {
            nodes: vec![Node::Leaf(Value::SimpleString(Bytes::from("Ok")))],
        }
    }
    pub fn from_value(v: Value<Bytes>) -> Self {
        RedisValue {
            nodes: vec![Node::Leaf(v)],
        }
    }
    pub fn size(&self) -> usize {
        self.nodes
            .iter()
            .map(|node| {
                match node {
                    &Node::Open(n) => usize_string_size(n) + 3, // '*' + <n item> | \r\n
                    &Node::Close => 0,
                    &Node::Leaf(ref v) => v.size(),
                }
            })
            .sum()
    }
    pub fn decode<B: AsRef<[u8]>>(buf: &B) -> Result<Option<(usize, Self)>, ()> {
        let buf = buf.as_ref();
        let result = decode_values_from_slice(buf);
        match result {
            Err(DecodeError::Incomplete) => Ok(None),
            Err(DecodeError::Failed) => Err(()),
            Ok((consumed, values)) => {
                let n_bytes = values.byte_count();
                match values {
                    Values::One(value) => {
                        // copy from buffer
                        let value = value.map(|r| Bytes::from(&buf[r]));
                        Ok(Some((
                            consumed,
                            RedisValue {
                                nodes: vec![Node::Leaf(value)],
                            },
                        )))
                    }
                    Values::Many(mut nodes) => {
                        // will copy bytes from input buf into this
                        // Leaf nodes's Bytes will share memory in this
                        // compat Bytes object with exactly enough bytes
                        let mut bytes = Bytes::with_capacity(n_bytes);
                        let nodes = nodes
                            .drain(..)
                            .map(|node| match node {
                                Node::Leaf(value) => {
                                    let rng = value.as_ref().into_option();
                                    if let Some(rng) = rng {
                                        let rng = rng.clone();
                                        let start = bytes.len();
                                        let end = start + (rng.end - rng.start);

                                        bytes.extend_from_slice(&buf[rng]);
                                        let leaf_bytes = bytes.slice(start, end);
                                        // clone here does not allocate, I think..
                                        Node::Leaf(value.as_ref().map(|_| leaf_bytes.clone()))
                                    } else {
                                        Node::Leaf(Value::Nil)
                                    }
                                }
                                Node::Open(count) => Node::Open(count),
                                Node::Close => Node::Close,
                            })
                            .collect();
                        Ok(Some((consumed, RedisValue { nodes })))
                    }
                }
            }
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) {
        buf.reserve(self.size());
        for node in &self.nodes {
            match node {
                &Node::Open(n) => {
                    buf.put(b'*');
                    buf.put(format!("{}", n));
                    buf.put("\r\n");
                }
                &Node::Leaf(ref v) => {
                    v.encode(buf);
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
enum Values {
    One(Value<Range>),
    Many(Vec<Node<Range>>),
}
impl Values {
    fn byte_count(&self) -> usize {
        match self {
            &Values::One(ref v) => v.iter().map(|r| r.end - r.start).sum(),
            &Values::Many(ref nodes) => nodes
                .iter()
                .flat_map(Node::value_iter)
                .map(|r| r.end - r.start)
                .sum(),
        }
    }
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
                            Values::One(value) => {
                                let value = value.map(|rng| (rng.start + index..rng.end + index));
                                nodes.push(Node::Leaf(value));
                            }
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

fn usize_string_size(mut v: usize) -> usize {
    let mut result = 1;
    loop {
        if v < 10 {
            return result;
        }
        if v < 100 {
            return result + 1;
        }
        if v < 1000 {
            return result + 2;
        }
        if v < 10000 {
            return result + 3;
        }

        v /= 10000;
        result += 4;
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
            let (consumed, _) = result.unwrap();
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

    #[test]
    fn test_decode_from_buffer() {
        let buf = Bytes::from("*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
        let _redis_val = RedisValue::decode(&buf);

        //TODO: imple Eq for RedisValue
        assert_eq!(true, true);
    }
}
