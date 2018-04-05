use std::str::{self, FromStr};
use std::string;
use std::io::{self, BufRead, Error, ErrorKind};
use std::result;
use std::ops::Deref;
use std::convert::{From, Into};

const CRLF: &'static str = "\r\n";

#[derive(Debug)]
pub enum ProtocolError {
    BadBytes,
    ParseError,
    TypeError,
    IoError(io::Error),
}

impl From<io::Error> for ProtocolError {
    fn from(err: io::Error) -> ProtocolError {
        ProtocolError::IoError(err)
    }
}

pub type Result<T> = result::Result<T, ProtocolError>;

#[derive(Debug, Eq, PartialEq)]
pub struct SimpleBytes {
    // Wrapper aroung Vec<u8> ensures bytes stored
    // does not contain CR nor LF
    bytes: Vec<u8>,
}

impl SimpleBytes {
    pub fn from_bytes<B: Into<Vec<u8>>>(bytes: B) -> Result<Self> {
        let bytes = bytes.into();
        let CR = '\r' as u8;
        let LF = '\n' as u8;

        for b in &bytes {
            if *b == CR || *b == LF {
                return Err(ProtocolError::BadBytes);
            }
        }

        Ok(SimpleBytes { bytes })
    }
    pub fn read_from<R: BufRead>(reader: &mut R) -> Result<Self> {
        let mut buffer: Vec<u8> = Vec::new();
        let _ = read_until_crlf(reader, &mut buffer)?;
        // remove CLRF line endings
        buffer.pop();
        buffer.pop();

        Ok(SimpleBytes { bytes: buffer })
    }
}
impl Into<Vec<u8>> for SimpleBytes {
    fn into(self) -> Vec<u8> {
        self.bytes
    }
}
impl Deref for SimpleBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.bytes
    }
}

fn read_until_crlf<R: BufRead>(reader: &mut R, buffer: &mut Vec<u8>) -> Result<usize> {
    let CR = '\r' as u8;
    let LF = '\n' as u8;

    let length = reader.read_until(LF, buffer)?;

    if length >= 2 && buffer[length - 2] == CR {
        Ok(length)
    } else {
        Err(ProtocolError::BadBytes)
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct StringValue(Vec<u8>);

impl<'a> From<&'a [u8]> for StringValue {
    fn from(slice: &[u8]) -> StringValue {
        StringValue(slice.to_vec())
    }
}

impl Deref for StringValue {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0[..]
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RespProtocol {
    SimpleString(SimpleBytes),
    Error(SimpleBytes),
    Integer(SimpleBytes),
    Null,
    BulkString(Vec<u8>),
    Array(Vec<RespProtocol>),
}

impl RespProtocol {
    pub fn ok() -> Self {
        RespProtocol::SimpleString(SimpleBytes { bytes: "Ok".into() })
    }
    pub fn from_integer<N: Into<i64>>(n: N) -> Self {
        let n = n.into();
        let number = format!("{}", n).into_bytes();
        RespProtocol::Integer(SimpleBytes { bytes: number })
    }
    pub fn as_bytes(&self) -> Result<&[u8]> {
        use self::RespProtocol::*;

        match self {
            &SimpleString(ref bytes) => Ok(&bytes.bytes[..]),
            &BulkString(ref bytes) => Ok(&bytes[..]),
            _ => Err(ProtocolError::TypeError),
        }
    }
    pub fn from_protocol_error(e: ProtocolError) -> Self {
        match e {
            ProtocolError::BadBytes => RespProtocol::Error(SimpleBytes {
                bytes: "BadBytes".into(),
            }),
            ProtocolError::ParseError => RespProtocol::Error(SimpleBytes {
                bytes: "ParseError".into(),
            }),
            ProtocolError::TypeError => RespProtocol::Error(SimpleBytes {
                bytes: "TypeError".into(),
            }),
            ProtocolError::IoError(_) => RespProtocol::Error(SimpleBytes {
                bytes: "IoError".into(),
            }),
        }
    }
    pub fn try_into_string_value(self) -> Result<StringValue> {
        use self::RespProtocol::*;

        match self {
            SimpleString(bytes) => Ok(StringValue(bytes.into())),
            BulkString(bytes) => Ok(StringValue(bytes)),
            _ => Err(ProtocolError::TypeError),
        }
    }
    pub fn into_bytes(self) -> Vec<u8> {
        use self::RespProtocol::*;

        match self {
            SimpleString(ref bytes) => {
                let mut v = vec!['+' as u8];
                v.extend_from_slice(bytes);
                v.extend_from_slice(CRLF.as_bytes());
                v
            }
            Error(ref bytes) => {
                let mut v = vec!['-' as u8];
                v.extend_from_slice(bytes);
                v.extend_from_slice(CRLF.as_bytes());
                v
            }
            Integer(ref bytes) => {
                let mut v = vec![':' as u8];
                v.extend_from_slice(bytes);
                v.extend_from_slice(CRLF.as_bytes());
                v
            }
            Null => "$-1\r\n".into(),
            BulkString(ref bytes) => {
                let mut v = vec!['$' as u8];
                let len = bytes.len().to_string();
                v.extend_from_slice(len.as_bytes());
                v.extend_from_slice(CRLF.as_bytes());
                v.extend_from_slice(bytes);
                v.extend_from_slice(CRLF.as_bytes());
                v
            }
            Array(xs) => {
                let mut v = vec!['*' as u8];
                let len = xs.len().to_string();
                v.extend_from_slice(len.as_bytes());
                v.extend_from_slice(CRLF.as_bytes());
                v.extend(xs.into_iter().flat_map(RespProtocol::into_bytes));
                v
            }
        }
    }
}

impl Into<Vec<u8>> for RespProtocol {
    fn into(self) -> Vec<u8> {
        self.into_bytes()
    }
}
impl From<StringValue> for RespProtocol {
    fn from(v: StringValue) -> RespProtocol {
        let StringValue(xs) = v;
        if xs.len() == 0 {
            RespProtocol::Null
        } else {
            RespProtocol::BulkString(xs)
        }
    }
}
impl<'a> From<&'a StringValue> for RespProtocol {
    fn from(v: &StringValue) -> RespProtocol {
        let xs = &v;
        if xs.len() == 0 {
            RespProtocol::Null
        } else {
            RespProtocol::BulkString(xs.to_vec())
        }
    }
}
impl From<Option<StringValue>> for RespProtocol {
    fn from(v: Option<StringValue>) -> RespProtocol {
        v.map(RespProtocol::from).unwrap_or(RespProtocol::Null)
    }
}
impl<'a> From<Option<&'a StringValue>> for RespProtocol {
    fn from(v: Option<&StringValue>) -> RespProtocol {
        v.map(RespProtocol::from).unwrap_or(RespProtocol::Null)
    }
}
impl<E> From<result::Result<StringValue, E>> for RespProtocol {
    fn from(v: result::Result<StringValue, E>) -> RespProtocol {
        v.map(RespProtocol::from).unwrap_or(RespProtocol::Null)
    }
}

impl string::ToString for RespProtocol {
    fn to_string(&self) -> String {
        use self::RespProtocol::*;

        match self {
            &SimpleString(ref s) => {
                let s = str::from_utf8(s).unwrap();
                format!("+{}{}", s, CRLF)
            }
            &Error(ref s) => {
                let s = str::from_utf8(s).unwrap();
                format!("-{}{}", s, CRLF)
            }
            &Integer(ref s) => {
                let s = str::from_utf8(s).unwrap();
                format!(":{}{}", s, CRLF)
            }
            &Null => "$-1\r\n".to_string(),
            &BulkString(ref s) => {
                let l = s.len();
                let s = str::from_utf8(s).unwrap();
                format!("${}{}{}{}", l, CRLF, s, CRLF)
            }
            &Array(ref xs) => {
                let l = xs.len();
                let mut ret: String = format!("*{}{}", l, CRLF);
                for x in xs {
                    ret.push_str(&x.to_string());
                }
                ret
            }
        }
    }
}

fn read_line_ending<R: BufRead>(reader: &mut R) -> io::Result<()> {
    let mut line_endings = [0u8; 2];
    let _ = reader.read_exact(&mut line_endings)?;

    if line_endings == *CRLF.as_bytes() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Other,
            "Invalid line ending, needs CRLF",
        ))
    }
}

pub fn read_protocol<R: BufRead>(reader: &mut R) -> Result<RespProtocol> {
    let mut header: Vec<u8> = Vec::new();

    let length = read_until_crlf(reader, &mut header)?;

    if length < 3 {
        return Err(ProtocolError::ParseError);
    }

    let prefix = header[0] as char;

    match prefix {
        '+' => SimpleBytes::from_bytes(&header[1..length - 2]).map(RespProtocol::SimpleString),
        '-' => SimpleBytes::from_bytes(&header[1..length - 2]).map(RespProtocol::Error),
        ':' => SimpleBytes::from_bytes(&header[1..length - 2]).map(RespProtocol::Integer),
        '$' => {
            let len =
                str::from_utf8(&header[1..length - 2]).map_err(|_| ProtocolError::ParseError)?;

            let len = isize::from_str(len).map_err(|_| ProtocolError::ParseError)?;

            if len == -1 {
                return Ok(RespProtocol::Null);
            } else if len < 0 {
                return Err(ProtocolError::ParseError);
            }

            let mut bulk_string_buffer = vec![0u8; len as usize];
            reader
                .read_exact(&mut bulk_string_buffer)
                .map_err(ProtocolError::IoError)?;

            // Read remaining CRLF
            read_line_ending(reader).map_err(ProtocolError::IoError)?;

            Ok(RespProtocol::BulkString(bulk_string_buffer))
        }
        '*' => {
            let len =
                str::from_utf8(&header[1..length - 2]).map_err(|_| ProtocolError::ParseError)?;

            let len = usize::from_str(len).map_err(|_| ProtocolError::ParseError)?;
            let items: Result<Vec<RespProtocol>> =
                (0..len).map(|_| read_protocol(&mut *reader)).collect();

            items.map(RespProtocol::Array)
        }
        _ => Err(ProtocolError::ParseError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;
    use std::io::{BufReader, Read};

    #[test]
    fn test_simple_bytes() {
        let bytes_ok = "asfasfasf".as_bytes();
        let simple_bytes = SimpleBytes::from_bytes(bytes_ok);
        let bytes_err_1 = "asdfaf\r".as_bytes();
        let bytes_err_2 = "asd\nfaf".as_bytes();
        let bytes_err_3 = "asdfa\r\n".as_bytes();

        assert_matches!(simple_bytes, Ok(_));
        assert_matches!(
            SimpleBytes::from_bytes(bytes_err_1),
            Err(ProtocolError::BadBytes)
        );
        assert_matches!(
            SimpleBytes::from_bytes(bytes_err_2),
            Err(ProtocolError::BadBytes)
        );
        assert_matches!(
            SimpleBytes::from_bytes(bytes_err_3),
            Err(ProtocolError::BadBytes)
        );
    }

    #[test]
    fn test_read_to_string_ok() {
        let ok_tests: Vec<&str> = vec![
            "+Ok\r\n",
            "-MESSAGE error happenend\r\n",
            ":12\r\n",
            "$6\r\nfoobar\r\n",
            "$8\r\nfoo\r\nbar\r\n",
            "$-1\r\n",
            "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n",
        ];
        for s in ok_tests {
            let mut reader = BufReader::new(StringReader::new(s));
            let protocol = read_protocol(&mut reader).unwrap();
            let string_val = protocol.to_string();

            let mut rest = Vec::new();
            let rest_len = reader.read_to_end(&mut rest).unwrap();

            assert_eq!(&string_val, s);
            // consumed all input
            assert_eq!(rest_len, 0);

            let bytes = protocol.into_bytes();
            assert_eq!(string_val.as_bytes(), &bytes[..]);
        }
    }
    //TODO: test expected ParseError
}
