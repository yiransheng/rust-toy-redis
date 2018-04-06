use bytes::Bytes;
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
