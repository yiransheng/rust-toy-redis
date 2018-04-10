use bytes::Bytes;
use btoi::{btoi, ParseIntegerError};
use std::mem;
use std::convert::{AsRef, From};

use super::traits::{DecodeBytes, DecodeError};

struct ExpectByte {
    byte: u8,
}
impl ExpectByte {
    fn new(byte: u8) -> Self {
        ExpectByte { byte }
    }
}

impl DecodeBytes for ExpectByte {
    type Output = u8;

    fn decode(&self, bytes: &[u8]) -> Result<(usize, u8), DecodeError> {
        if bytes.len() == 0 {
            return Err(DecodeError::Incomplete);
        }

        if bytes[0] == self.byte {
            Ok((1, self.byte))
        } else {
            Err(DecodeError::Fail)
        }
    }
}

const lineEnd: ExpectBytes = ExpectBytes { bytes: b"\r\n" };

struct ExpectBytes {
    bytes: &'static [u8],
}

impl DecodeBytes for ExpectBytes {
    type Output = &'static [u8];

    fn decode(&self, bytes: &[u8]) -> Result<(usize, Self::Output), DecodeError> {
        let expected_bytes = self.bytes;
        let expected_len = expected_bytes.len();
        if bytes.len() < expected_len {
            return Err(DecodeError::Incomplete);
        }

        if &bytes[0..expected_len] == expected_bytes {
            Ok((expected_len, self.bytes))
        } else {
            Err(DecodeError::Fail)
        }
    }
}

struct SafeByte;

impl DecodeBytes for SafeByte {
    type Output = ();

    fn decode(&self, bytes: &[u8]) -> Result<(usize, ()), DecodeError> {
        if bytes.len() == 0 {
            return Err(DecodeError::Incomplete);
        }

        match bytes[0] {
            b'\r' => Err(DecodeError::Fail),
            b'\n' => Err(DecodeError::Fail),
            _ => Ok((1, ())),
        }
    }
}
struct AnyByte;

impl DecodeBytes for AnyByte {
    type Output = ();

    fn decode(&self, bytes: &[u8]) -> Result<(usize, ()), DecodeError> {
        if bytes.len() == 0 {
            return Err(DecodeError::Incomplete);
        }

        Ok((1, ()))
    }
}

impl Into<DecodeError> for ParseIntegerError {
    fn into(self) -> DecodeError {
        DecodeError::Fail
    }
}

fn check_bulk() -> impl DecodeBytes<Output = ()> {
    ExpectByte::new(b'$').and_then(|_| {
        SafeByte
            .many_()
            .map_slice(|s| btoi(s).ok())
            .unwrap_fail::<u64>()
            .and_then_(|_| lineEnd)
            .and_then(|n| AnyByte.repeat_(n))
            .and_then_(|_| lineEnd)
    })
}
fn parse_bulk() -> impl DecodeBytes<Output = String> {
    ExpectByte::new(b'$').and_then(|_| {
        SafeByte
            .many_()
            .map_slice(|s| btoi(s).ok())
            .unwrap_fail::<u64>()
            .and_then_(|_| lineEnd)
            .and_then(|n| {
                AnyByte
                    .repeat_(n)
                    .map_slice(|s| String::from_utf8(s.to_vec()).unwrap())
            })
            .and_then_(|_| lineEnd)
    })
}

fn check_array() -> impl DecodeBytes<Output = ()> {
    ExpectByte::new(b'*').and_then(|_| {
        SafeByte
            .many_()
            .map_slice(|s| btoi(s).ok())
            .unwrap_fail::<u64>()
            .and_then_(|_| lineEnd)
            .and_then(|n| {
                let bulk = check_bulk();
                bulk.repeat_(n)
            })
    })
}
fn parse_array() -> impl DecodeBytes<Output = Vec<String>> {
    ExpectByte::new(b'*').and_then(|_| {
        SafeByte
            .many_()
            .map_slice(|s| btoi(s).ok())
            .unwrap_fail::<u64>()
            .and_then_(|_| lineEnd)
            .and_then(|n| {
                let bulk = parse_bulk();
                bulk.repeat(n)
            })
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_bulk() {
        let b = check_bulk();
        let input = b"$3\r\nfoo\r\n";

        assert_eq!(b.decode(&input[..]), Ok((input.len(), ())));
    }
    #[test]
    fn test_check_array() {
        let a = check_array();
        let pa = parse_array();
        let input = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";

        println!("{:?}", pa.decode(&input[..]));

        assert_eq!(a.decode(&input[..]), Ok((input.len() + 1, ())));
    }
}
