use std::io;

use bytes::BytesMut;

use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Decoder, Encoder, Framed};
use tokio_proto::pipeline::ServerProto;

use super::redis_value::RedisValue;

pub struct RedisCodec;

pub struct RedisProto;

impl Decoder for RedisCodec {
    type Item = RedisValue;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<RedisValue>, io::Error> {
        RedisValue::decode(&*buf)
            .map(|redis_val| {
                match redis_val {
                    Some((consumed, x)) => {
                        // This is super Important!
                        //
                        // For a tokio Codec, returning Ok<Some<Item>> alone
                        // is not sufficient to tell the framework this Frame
                        // is Completed.
                        //
                        // There's a reason decode takes a &mut BytesMute, I
                        // guess, the Frame completes only if the buffer is
                        // drained fully, so it seems.
                        buf.advance(consumed);
                        Some(x)
                    }
                    None => None,
                }
            })
            .map_err(|_| io_error!(InvalidData, "RESP decode Error"))
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
    type Request = RedisValue;
    type Response = RedisValue;

    type Transport = Framed<T, RedisCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(RedisCodec))
    }
}
