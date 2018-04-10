use btoi::{btoi, ParseIntegerError};
use bytes::Bytes;
use std::convert::{AsRef, From};
use std::str;

use super::traits::{end_line_crlf, AnyByte, DecodeBytes, DecodeError, ExpectByte, LineSafeByte};

#[inline]
fn check_bulk<'b>() -> impl DecodeBytes<'b, Output = usize> {
    ExpectByte::new(b'$')
        .and(LineSafeByte.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_(end_line_crlf)
        .and_then(|n| AnyByte.repeat_(n))
        .and_(end_line_crlf)
        .count_bytes()
}
#[inline]
fn parse_bulk_str<'b>() -> impl DecodeBytes<'b, Output = &'b str> {
    ExpectByte::new(b'$')
        .and(LineSafeByte.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_(end_line_crlf)
        .and_then(|n| {
            AnyByte
                .repeat_(n)
                .parse_slice(|s| str::from_utf8(s).unwrap())
        })
        .and_(end_line_crlf)
}

#[inline]
pub fn check_array<'b>() -> impl DecodeBytes<'b, Output = usize> {
    ExpectByte::new(b'*')
        .and(LineSafeByte.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_then_(|_| end_line_crlf)
        .and_then(|n| check_bulk().repeat_(n))
        .count_bytes()
}
#[inline]
fn parse_array_str<'b>() -> impl DecodeBytes<'b, Output = Vec<&'b str>> {
    ExpectByte::new(b'*')
        .and(LineSafeByte.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_then_(|_| end_line_crlf)
        .and_then(|n| parse_bulk_str().repeat(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_bulk() {
        let b = check_bulk();
        let input = b"$3\r\nfoo\r\n";

        assert_eq!(b.decode_all(&input[..]), Ok(input.len()));
    }
    #[test]
    fn test_check_array() {
        let a = check_array();
        let pa = parse_array_str();
        let input = b"*3\r\n$3\r\nfoo\r\n$4\r\nbars\r\n$1\r\nx\r\n";

        println!("{:?}", pa.decode_all(&input[..]));

        assert_eq!(a.decode_all(&input[..]), Ok(input.len()));
        assert_eq!(false, true);
    }
}
