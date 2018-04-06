use bytes::{Bytes, BytesMut};
use std::convert::{From, Into};
use std::mem;

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
    values: BytesMut,
}

impl RedisValue {
    fn new() -> Self {
        RedisValue {
            nodes: Vec::new(),
            values: BytesMut::new(),
        }
    }
}
