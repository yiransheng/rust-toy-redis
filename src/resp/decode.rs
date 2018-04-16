use btoi::{btoi, ParseIntegerError};
use std::convert::{AsRef, From};
use std::str;

use bytes::Bytes;
use bytes_decoder::primitives::*;
use bytes_decoder::{Decode, DecodeError};

use super::command::Arguments;

#[inline]
fn check_bulk<'b>() -> impl Decode<'b, Output = usize> {
    let end_line_crlf: BytesExact = BytesExact::new("\r\n".as_bytes());
    Byte::new(b'$')
        .and(ByteLineSafe.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_(end_line_crlf)
        .and_then(|n| ByteAny.repeat_(n))
        .and_(end_line_crlf)
        .bytes_consumed()
}

#[inline]
pub fn check_array<'b>() -> impl Decode<'b, Output = usize> {
    Byte::new(b'*')
        .and(ByteLineSafe.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_then_(|_| BytesExact::new("\r\n".as_bytes()))
        .and_then(|n| check_bulk().repeat_(n))
        .bytes_consumed()
}

#[inline]
fn decode_bulk<'b>() -> impl Decode<'b, Output = &'b [u8]> {
    let end_line_crlf: BytesExact = BytesExact::new("\r\n".as_bytes());
    Byte::new(b'$')
        .and(ByteLineSafe.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_(end_line_crlf)
        .and_then(|n| ByteAny.repeat_(n))
        .and_(end_line_crlf)
        .to_consumed_slice()
}
#[inline]
pub fn decode_array<'b>(bytes: &'b [u8]) -> Result<Arguments<Bytes>, DecodeError> {
    let end_line_crlf: BytesExact = BytesExact::new("\r\n".as_bytes());

    let decoder = Byte::new(b'*')
        .and(ByteLineSafe.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_then_(|_| BytesExact::new("\r\n".as_bytes()))
        .and_then(|n| {
            decode_bulk().reduce_repeat::<Arguments<&'b [u8]>, _>(n, |args, s| args.append(s))
        });

    let args = decoder.decode_exact(bytes)?;

    Ok(args.to_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_bulk_str<'b>() -> impl Decode<'b, Output = &'b str> {
        let end_line_crlf: BytesExact = BytesExact::new("\r\n".as_bytes());
        Byte::new(b'$')
            .and(ByteLineSafe.many_().parse_slice(btoi))
            .filter_map(|x| x.ok())
            .and_(end_line_crlf)
            .and_then(|n| {
                ByteAny
                    .repeat_(n)
                    .parse_slice(|s| str::from_utf8(s).unwrap())
            })
            .and_(end_line_crlf)
    }
    fn parse_array_str<'b>() -> impl Decode<'b, Output = Vec<&'b str>> {
        let end_line_crlf: BytesExact = BytesExact::new("\r\n".as_bytes());
        Byte::new(b'*')
            .and(ByteLineSafe.many_().parse_slice(btoi))
            .filter_map(|x| x.ok())
            .and_then_(|_| BytesExact::new("\r\n".as_bytes()))
            .and_then(|n| parse_bulk_str().repeat(n))
    }

    #[test]
    fn test_decode_bulk() {
        let checker = check_bulk();
        let parser = parse_bulk_str();

        let input = b"$3\r\nfoo\r\n";

        assert_eq!(checker.decode_exact(&input[..]), Ok(input.len()));
        assert_eq!(parser.decode_exact(&input[..]), Ok("foo"));
    }

    #[test]
    fn test_decode_array() {
        let checker = check_array();
        let parser = parse_array_str();

        let input = b"*3\r\n$3\r\nfoo\r\n$4\r\nbars\r\n$1\r\nx\r\n";

        assert_eq!(checker.decode_exact(&input[..]), Ok(input.len()));
        assert_eq!(
            parser.decode_exact(&input[..]),
            Ok(vec!["foo", "bars", "x"])
        );
    }
}
