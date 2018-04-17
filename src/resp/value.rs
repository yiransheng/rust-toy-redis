use bytes::{BufMut, Bytes, BytesMut};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::mem;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Okay,
    Status(String),
    Int(i64),
    Data(Vec<u8>),
    Array(Vec<Value>),
}

impl Value {
    pub fn encoding_len(&self) -> usize {
        use self::Value::*;

        match *self {
            // $-1\r\n
            Nil => 5,
            // +Ok\r\n
            Okay => 5,
            Status(ref s) => s.as_bytes().len() + 3,
            Int(n) => count_digits(n) + 3,
            Data(ref xs) => {
                // $3\r\nfoo\r\n
                let n = xs.len();
                count_digits(n as i64) + n + 5
            }
            Array(ref xs) => {
                let n = xs.len();
                let data_len: usize = xs.iter().map(|v| v.encoding_len()).sum();
                3 + count_digits(n as i64) + data_len
            }
        }
    }

    pub fn encoding_iter(&self) -> EncodeIter {
        use self::Value::*;

        let mut queue = VecDeque::new();
        let cursor;

        match *self {
            Array(ref vs) => {
                for v in vs {
                    queue.push_back(v);
                }
                cursor = EncodeItem::Prefix(b'*', vs.len());
            }
            _ => {
                queue.push_back(self);
                cursor = EncodeItem::Done;
            }
        }

        EncodeIter {
            cursor,
            values: queue,
        }
    }

    fn as_encode_item(&self) -> EncodeItem {
        use self::Value::*;

        match *self {
            Nil => EncodeItem::Static(b"$-1\r\n"),
            Okay => EncodeItem::Static(b"+Ok\r\n"),
            Status(_) => EncodeItem::Enclosed(b'-', None, self),
            Int(n) => EncodeItem::Enclosed(b':', None, self),
            Data(ref xs) => EncodeItem::Enclosed(b'$', Some(xs.len()), self),
            Array(ref vs) => EncodeItem::Prefix(b'*', vs.len()),
        }
    }
    fn as_value_slice(&self) -> Cow<[u8]> {
        use self::Value::*;

        match *self {
            Status(ref s) => Cow::Borrowed(s.as_bytes()),
            Int(n) => Cow::Owned(format!("{}", n).into_bytes()),
            Data(ref xs) => Cow::Borrowed(&xs[..]),
            _ => Cow::Borrowed(b""),
        }
    }
}

#[derive(Debug)]
pub enum EncodeItem<'a> {
    Done,
    Static(&'static [u8]),
    Prefix(u8, usize),
    Enclosed(u8, Option<usize>, &'a Value),
}
impl<'a> EncodeItem<'a> {
    pub fn encode(self, buf: &mut BytesMut) {
        match self {
            EncodeItem::Static(s) => buf.put(s),
            EncodeItem::Prefix(p, c) => {
                buf.put(p);
                buf.put(format!("{}\r\n", c));
            }
            EncodeItem::Enclosed(p, c, v) => {
                buf.put(p);
                if let Some(c) = c {
                    buf.put(format!("{}", c));
                    buf.put("\r\n");
                }
                buf.put(v.as_value_slice().as_ref());
                buf.put("\r\n");
            }
            _ => {}
        }
    }
    pub fn encoding_len(&self) -> usize {
        match *self {
            EncodeItem::Static(s) => s.len(),
            EncodeItem::Prefix(p, c) => 3 + count_digits(c as i64),
            EncodeItem::Enclosed(p, c, v) => {
                let mut n = 1;
                if let Some(c) = c {
                    n = n + count_digits(c as i64);
                    n = n + 2;
                }
                n = n + v.as_value_slice().len();
                n = n + 2;
                n
            }
            _ => 0,
        }
    }
}

pub struct EncodeIter<'a> {
    cursor: EncodeItem<'a>,
    values: VecDeque<&'a Value>,
}
impl<'a> Iterator for EncodeIter<'a> {
    type Item = EncodeItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_item()
    }
}
impl<'a> EncodeIter<'a> {
    #[inline]
    fn pop_item(&mut self) -> Option<EncodeItem<'a>> {
        match self.cursor {
            EncodeItem::Done => {}
            _ => {
                let current = mem::replace(&mut self.cursor, EncodeItem::Done);
                return match current {
                    EncodeItem::Done => None,
                    x => Some(x),
                };
            }
        }

        let next_value = self.values.pop_front();

        if let Some(value) = next_value {
            match value {
                &Value::Array(ref vs) => {
                    let n = vs.len();
                    for i in 0..n {
                        let j = n - i - 1;
                        self.values.push_front(&vs[j]);
                    }
                }
                _ => {}
            }
            Some(value.as_encode_item())
        } else {
            None
        }
    }
}

fn count_digits(mut v: i64) -> usize {
    // negative sign
    let mut result = if v < 0 { 2 } else { 1 };
    v = v.abs();
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
    fn test_encode() {
        let values = Value::Array(vec![
            Value::Okay,
            Value::Okay,
            Value::Array(vec![Value::Nil]),
            Value::Int(32),
            Value::Array(vec![]),
        ]);
        let bulk_string = Value::Data(b"hello world!".to_vec());

        let value = Value::Array(vec![
            bulk_string,
            values,
            Value::Status("err".to_string()),
            Value::Nil,
        ]);

        let mut buf = BytesMut::with_capacity(value.encoding_len());

        let expected = b"*4\r\n$12\r\nhello world!\r\n*5\r\n+Ok\r\n+Ok\r\n*1\r\n$-1\r\n:32\r\n*0\r\n-err\r\n$-1\r\n";

        for item in value.encoding_iter() {
            item.encode(&mut buf);
        }

        assert_eq!(buf.as_ref(), &expected[..]);
    }
}
