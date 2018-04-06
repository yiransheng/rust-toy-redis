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
        let empty = [];
        match self {
            Value::SimpleString(v) => v.into(),
            Value::ErrorString(v) => v.into(),
            Value::IntegerString(v) => v.into(),
            Value::BulkString(v) => v.into(),
            Value::Nil => &empty,
        }
    }
}

#[derive(Debug)]
pub enum Node<T> {
    Leaf(Value<T>),
    Open(usize),
    Close,
}

pub struct ArrayTree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Value<T> {
    fn take(&mut self) -> Self {
        mem::replace(self, Value::Nil)
    }
}
