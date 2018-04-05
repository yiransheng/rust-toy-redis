use std::mem;
use super::resp::{self, ProtocolError, RespProtocol, StringValue};

#[derive(Debug)]
pub enum Command {
    SET {
        key: StringValue,
        value: StringValue,
    },
    GET {
        key: StringValue,
    },
    DEL {
        keys: Vec<StringValue>,
    },
}

impl Command {
    pub fn try_from_protocol(proto: RespProtocol) -> resp::Result<Self> {
        match proto {
            RespProtocol::Array(mut protos) => {
                let len = protos.len();
                if len == 0 {
                    return Err(ProtocolError::ParseError);
                }
                let keyword = mem::replace(&mut protos[0], RespProtocol::Null);
                let keyword = keyword.as_bytes()?;

                match keyword {
                    b"GET" => {
                        if len == 2 {
                            let key = mem::replace(&mut protos[1], RespProtocol::Null);
                            let key = key.try_into_string_value()?;
                            Ok(Command::GET { key })
                        } else {
                            Err(ProtocolError::ParseError)
                        }
                    }
                    b"SET" => {
                        if len == 3 {
                            let key = mem::replace(&mut protos[1], RespProtocol::Null);
                            let key = key.try_into_string_value()?;
                            let value = mem::replace(&mut protos[2], RespProtocol::Null);
                            let value = value.try_into_string_value()?;
                            Ok(Command::SET { key, value })
                        } else {
                            Err(ProtocolError::ParseError)
                        }
                    }
                    b"DEL" => {
                        if len >= 2 {
                            let keys: Vec<StringValue> = protos[1..]
                                .iter_mut()
                                .filter_map(|p| p.take().try_into_string_value().ok())
                                .collect();
                            Ok(Command::DEL { keys })
                        } else {
                            Err(ProtocolError::ParseError)
                        }
                    }
                    _ => Err(ProtocolError::ParseError),
                }
            }
            _ => Err(ProtocolError::ParseError),
        }
    }
}
