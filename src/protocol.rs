use std::io;

use bytes::{Bytes, BytesMut};
use bytes_decoder::{Decode, DecodeError};

use tokio_io::codec::{Decoder, Encoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::pipeline::ServerProto;

use super::resp::decode::{check_array, decode_array};
use super::resp::{Arguments, Value};

pub struct RedisCodec;

pub struct RedisProto;

impl Decoder for RedisCodec {
    type Item = Arguments<Bytes>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        let n_bytes: Result<usize, _>;

        {
            let bytes = buf.as_ref();
            let checker = check_array();

            n_bytes = checker.decode_(bytes);
        }

        let consumed: usize;

        match n_bytes {
            Err(DecodeError::Incomplete) => return Ok(None),
            Err(DecodeError::Fail) => io_fail!(InvalidData, "RESP decode Error"),
            Ok(n) => {
                consumed = n;
            }
        }

        let bytes = buf.split_to(consumed);
        let bytes = bytes.as_ref();
        // checker ensures decoding will succeed
        let args = decode_array(bytes).unwrap();
        Ok(Some(args))
    }
}

impl Encoder for RedisCodec {
    type Item = Value;
    type Error = io::Error;

    fn encode(&mut self, msg: Value, buf: &mut BytesMut) -> io::Result<()> {
        buf.reserve(msg.encoding_len());

        for item in msg.encoding_iter() {
            item.encode(buf);
        }

        Ok(())
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for RedisProto {
    type Request = Arguments<Bytes>;
    type Response = Value;

    type Transport = Framed<T, RedisCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(RedisCodec))
    }
}
