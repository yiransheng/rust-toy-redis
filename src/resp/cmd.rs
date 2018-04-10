use btoi::{btoi, ParseIntegerError};
use bytes::Bytes;
use std::convert::{AsRef, From};
use std::mem;

use super::traits::{end_line_crlf, AnyByte, DecodeBytes, DecodeError, ExpectByte};

struct SafeByte;

/*
 * impl DecodeBytes for SafeByte {
 *     type Output = ();
 *
 *     fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError> {
 *         if bytes.len() == 0 {
 *             return Err(DecodeError::Incomplete);
 *         }
 *
 *         match bytes[0] {
 *             b'\r' => Err(DecodeError::Fail),
 *             b'\n' => Err(DecodeError::Fail),
 *             _ => Ok((&bytes[1..], ())),
 *         }
 *     }
 * }
 *
 * fn take_slice<'a>(bytes: &'a [u8]) -> Result<&'a [u8], DecodeError> {
 *     let sx: &[u8];
 *     let (r, x) = SafeByte
 *         .many_()
 *         .parse_slice::<&'a [u8], _>(|s| unsafe { &*(s as *const [u8]) })
 *         .decode(bytes)?;
 *
 *     Ok(x)
 * }
 *
 * fn check_bulk() -> impl DecodeBytes<Output = usize> {
 *     ExpectByte::new(b'$')
 *         .and_then(|_| {
 *             SafeByte
 *                 .many_()
 *                 .parse_slice(|s| btoi(s))
 *                 .filter_map::<u64, _>(|x| x.ok())
 *                 .and_then_(|_| end_line_crlf)
 *                 .and_then(|n| AnyByte.repeat_(n))
 *                 .and_then_(|_| end_line_crlf)
 *         })
 *         .count_bytes()
 * }
 * fn parse_bulk() -> impl DecodeBytes<Output = String> {
 *     ExpectByte::new(b'$')
 *         .and(SafeByte.many_().parse_slice(btoi))
 *         .filter_map(|x| x.ok())
 *         .and_then_(|_| end_line_crlf)
 *         .and_then(|n| {
 *             AnyByte
 *                 .repeat_(n)
 *                 .parse_slice(|s| String::from_utf8(s.to_vec()).unwrap())
 *         })
 *         .and_then_(|_| end_line_crlf)
 * }
 *
 * pub fn check_array() -> impl DecodeBytes<Output = usize> {
 *     ExpectByte::new(b'*')
 *         .and_then(|_| {
 *             SafeByte
 *                 .many_()
 *                 .parse_slice(|s| btoi(s))
 *                 .filter_map(|x| x.ok())
 *                 .and_then_(|_| end_line_crlf)
 *                 .and_then(|n| {
 *                     let bulk = check_bulk();
 *                     bulk.repeat_(n)
 *                 })
 *         })
 *         .count_bytes()
 * }
 * fn parse_array() -> impl DecodeBytes<Output = Vec<String>> {
 *     ExpectByte::new(b'*').and_then(|_| {
 *         SafeByte
 *             .many_()
 *             .parse_slice(|s| btoi(s))
 *             .filter_map(|x| x.ok())
 *             .and_then_(|_| end_line_crlf)
 *             .and_then(|n| {
 *                 let bulk = parse_bulk();
 *                 bulk.repeat(n)
 *             })
 *     })
 * }
 *
 * #[cfg(test)]
 * mod tests {
 *     use super::*;
 *
 *     #[test]
 *     fn test_check_bulk() {
 *         let b = check_bulk();
 *         let input = b"$3\r\nfoo\r\n";
 *
 *         assert_eq!(b.decode_all(&input[..]), Ok(input.len()));
 *     }
 *     #[test]
 *     fn test_check_array() {
 *         let a = check_array();
 *         let pa = parse_array();
 *         let input = b"*3\r\n$3\r\nfoo\r\n$4\r\nbars\r\n$1\r\nx\r\n";
 *
 *         println!("{:?}", pa.decode_all(&input[..]));
 *
 *         assert_eq!(a.decode_all(&input[..]), Ok(input.len()));
 *         assert_eq!(false, true);
 *     }
 * }
 */
