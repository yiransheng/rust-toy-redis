use std::io;

use bytes::{Bytes, BytesMut};
use bytes_decoder::{Decode, DecodeError};

use tokio_io::codec::{Decoder, Encoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;

use super::redis_value::RedisValue;
use super::resp::decode::{check_array, decode_array};
use super::resp::Arguments;

pub struct RedisCodec;

pub struct RedisProto;

impl Decoder for RedisCodec {
    type Item = Arguments<Bytes>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        let checker = check_array();
        let bytes = buf.as_ref();

        let n_bytes = checker.decode_(bytes);

        match n_bytes {
            Err(DecodeError::Incomplete) => return Ok(None),
            Err(DecodeError::Fail) => io_fail!(InvalidData, "RESP decode Error"),
            Ok(consumed) => {
                let bytes = buf.split_to(consumed).as_ref();
                // checker ensures decoding will succeed
                let args = decode_array(bytes).unwrap();
                Ok(Some(args))
            }
        }
    }
}

impl Encoder for RedisCodec {
    type Item = RedisValue;
    type Error = io::Error;

    fn encode(&mut self, msg: RedisValue, buf: &mut BytesMut) -> io::Result<()> {
        msg.encode(buf);

        Ok(())
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for RedisProto {
    type Request = Arguments<Bytes>;
    type Response = RedisValue;

    type Transport = Framed<T, RedisCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(RedisCodec))
    }
}
