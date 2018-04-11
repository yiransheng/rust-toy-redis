use btoi::{btoi, ParseIntegerError};
use bytes::Bytes;
use std::convert::{AsRef, From};
use std::str;

use super::traits::{any_byte, end_line_crlf, line_safe_byte, match_byte, DecodeBytes, DecodeError};

#[inline]
fn check_bulk<'b>() -> impl DecodeBytes<'b, Output = usize> {
    match_byte(b'$')
        .and(line_safe_byte.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_(end_line_crlf)
        .and_then(|n| any_byte.repeat_(n))
        .and_(end_line_crlf)
        .count_bytes()
}

#[inline]
pub fn check_array<'b>() -> impl DecodeBytes<'b, Output = usize> {
    match_byte(b'*')
        .and(line_safe_byte.many_().parse_slice(btoi))
        .filter_map(|x| x.ok())
        .and_then_(|_| end_line_crlf)
        .and_then(|n| check_bulk().repeat_(n))
        .count_bytes()
}

#[cfg(test)]
mod tests {
    use super::super::traits::fail;
    use super::*;

    fn parse_bulk_str<'b>() -> impl DecodeBytes<'b, Output = &'b str> {
        match_byte(b'$')
            .and(line_safe_byte.many_().parse_slice(btoi))
            .filter_map(|x| x.ok())
            .and_(end_line_crlf)
            .and_then(|n| {
                any_byte
                    .repeat_(n)
                    .parse_slice(|s| str::from_utf8(s).unwrap())
            })
            .and_(end_line_crlf)
    }
    fn parse_array_str<'b>() -> impl DecodeBytes<'b, Output = Vec<&'b str>> {
        match_byte(b'*')
            .and(line_safe_byte.many_().parse_slice(btoi))
            .filter_map(|x| x.ok())
            .and_then_(|_| end_line_crlf)
            .and_then(|n| parse_bulk_str().repeat(n))
    }

    #[derive(Debug, Eq, PartialEq)]
    enum Nested<T> {
        One(T),
        Many(Vec<Nested<T>>),
    }
    /*
     *
     *     fn parse_many<'b, T, D>(one: fn() -> D) -> impl DecodeBytes<'b, Output = Nested<T>>
     *     where
     *         D: DecodeBytes<'b, Output = T>,
     *     {
     *         match_byte(b'*')
     *             .and(line_safe_byte.many_().parse_slice(btoi))
     *             .filter_map::<u64, _>(|x| x.ok())
     *             .and_(end_line_crlf)
     *             .and_then(move |n| parse_nested(one).repeat(n))
     *             .map(Nested::Many)
     *     }
     *
     *     fn parse_nested<'b, T, D>(one: fn() -> D) -> impl DecodeBytes<'b, Output = Nested<T>>
     *     where
     *         D: DecodeBytes<'b, Output = T>,
     *     {
     *         one().map(Nested::One).or(parse_many(one))
     *     }
     */

    #[test]
    fn test_decode_bulk() {
        let checker = check_bulk();
        let parser = parse_bulk_str();

        let input = b"$3\r\nfoo\r\n";

        assert_eq!(checker.decode_all(&input[..]), Ok(input.len()));
        assert_eq!(parser.decode_all(&input[..]), Ok("foo"));
    }

    #[test]
    fn test_decode_array() {
        let checker = check_array();
        let parser = parse_array_str();

        let input = b"*3\r\n$3\r\nfoo\r\n$4\r\nbars\r\n$1\r\nx\r\n";

        assert_eq!(checker.decode_all(&input[..]), Ok(input.len()));
        assert_eq!(parser.decode_all(&input[..]), Ok(vec!["foo", "bars", "x"]));
    }

    /*
     *     #[test]
     *     fn test_decoded_nested() {
     *         let parser = parse_nested(parse_bulk_str);
     *         let input = b"*2\r\n$4\r\nhead\r\n*2\r\n$2\r\nok\r\n$3\r\nerr\r\n";
     *
     *         println!("{:?}", parser.decode_all(&input[..]));
     *
     *         assert_eq!(true, false);
     *     }
     */
}
