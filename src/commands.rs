use std::result;
use super::redis_value::{Node, Value};

pub type Result<T> = result::Result<T, ParseError>;

#[derive(Debug)]
pub enum Cmd<T> {
    SET { key: T, value: T },
    GET { key: T },
    DEL { keys: Vec<T> },
}

#[derive(Debug)]
pub enum ParseError {
    Unexpected,
    UnknownCmd,
    ExtraValues,
    EmptyNodes,
    NilError,
    UnknownError,
}

enum ParserState<T> {
    Start,
    Started(usize),
    ParseSET(Option<T>, Option<T>),
    ParseGET(Option<T>),
    ParseDEL(usize, Vec<T>),
    Done(Cmd<T>),
    Error(ParseError),
}

impl<T> ParserState<Value<T>>
where
    T: AsRef<[u8]>,
{
    fn next_node(self, node: Node<T>) -> Self {
        use self::ParserState::*;

        match self {
            Start => match node {
                Node::Open(x) if x >= 2 => Started(x),
                _ => Error(ParseError::EmptyNodes),
            },
            Started(n_items) => match node {
                Node::Leaf(v) => {
                    let keyword: &[u8] = v.as_slice();
                    match keyword {
                        b"SET" if n_items == 3 => ParseSET(None, None),
                        b"GET" if n_items == 2 => ParseGET(None),
                        b"DEL" => ParseDEL(n_items - 1, Vec::with_capacity(n_items - 1)),
                        _ => Error(ParseError::UnknownCmd),
                    }
                }
                _ => Error(ParseError::Unexpected),
            },
            ParseGET(None) => match node {
                Node::Leaf(v) => ParseGET(Some(v)),
                _ => Error(ParseError::Unexpected),
            },
            ParseGET(Some(v)) => match &v {
                &Value::Nil => Error(ParseError::NilError),
                _ => match node {
                    Node::Close => Done(Cmd::GET { key: v }),
                    _ => Error(ParseError::ExtraValues),
                },
            },
            ParseSET(None, None) => match node {
                Node::Leaf(v) => ParseSET(Some(v), None),
                _ => Error(ParseError::Unexpected),
            },
            ParseSET(Some(k), None) => match node {
                Node::Leaf(v) => ParseSET(Some(k), Some(v)),
                _ => Error(ParseError::Unexpected),
            },
            ParseSET(Some(k), Some(v)) => match node {
                Node::Close => match (&k, &v) {
                    (&Value::Nil, _) => Error(ParseError::NilError),
                    (_, &Value::Nil) => Error(ParseError::NilError),
                    _ => Done(Cmd::SET { key: k, value: v }),
                },
                _ => Error(ParseError::Unexpected),
            },
            ParseDEL(n_expected, mut xs) => {
                if n_expected > 0 {
                    match node {
                        Node::Leaf(v) => {
                            xs.push(v);
                            ParseDEL(n_expected - 1, xs)
                        }
                        _ => Error(ParseError::Unexpected),
                    }
                } else {
                    match node {
                        Node::Close => Done(Cmd::DEL { keys: xs }),
                        _ => Error(ParseError::ExtraValues),
                    }
                }
            }
            Error(e) => Error(e),
            Done(_) => Error(ParseError::ExtraValues),
            _ => Error(ParseError::UnknownError),
        }
    }
}

pub fn parse_command<T: AsRef<[u8]>, I: IntoIterator<Item = Node<T>>>(
    iter: I,
) -> Result<Cmd<Value<T>>> {
    let mut state = ParserState::Start;

    for node in iter.into_iter() {
        state = state.next_node(node);
        match state {
            ParserState::Error(e) => return Err(e),
            _ => (),
        }
    }

    match state {
        ParserState::Done(cmd) => Ok(cmd),
        ParserState::Error(err) => Err(err),

        _ => Err(ParseError::UnknownError),
    }
}
