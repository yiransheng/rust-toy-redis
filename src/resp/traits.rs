use bytes::BytesMut;
use std::convert::Into;

#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    Incomplete,
    Fail,
}

pub trait DecodeBytes: Sized {
    type Output;

    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError>;

    #[inline]
    fn decode_<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<Self::Output, DecodeError> {
        let (_, out) = self.decode(bytes)?;
        Ok(out)
    }
    #[inline]
    fn decode_all<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<Self::Output, DecodeError> {
        let (remainder, out) = self.decode(bytes)?;
        if remainder.len() == 0 {
            Ok(out)
        } else {
            Err(DecodeError::Fail)
        }
    }
    #[inline]
    fn decode_buf(&self, buf: &mut BytesMut) -> Result<Option<Self::Output>, ::std::io::Error> {
        let result;
        let consumed: usize;

        {
            // borrow of buf inside here
            let out = self.decode(buf.as_ref());
            match out {
                Ok((remainder, out)) => {
                    consumed = buf.len() - remainder.len();
                    result = Ok(Some(out));
                }
                Err(DecodeError::Incomplete) => return Ok(None),
                _ => return Err(io_error!(InvalidData, "RESP DecodeError")),
            }
        }

        buf.advance(consumed);
        result
    }
    #[inline]
    fn count_bytes(self) -> BytesConsumed<Self> {
        BytesConsumed { src: self }
    }
    #[inline]
    fn filter_map<T, F>(self, f: F) -> FilterMap<Self, F>
    where
        F: Fn(Self::Output) -> Option<T>,
    {
        FilterMap { src: self, f }
    }
    #[inline]
    fn filter<B, F>(self, f: F) -> Filter<Self, F>
    where
        F: Fn(&Self::Output) -> bool,
    {
        Filter { src: self, f }
    }
    #[inline]
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Output) -> B,
    {
        Map { src: self, f }
    }
    #[inline]
    fn parse_slice<B, F>(self, f: F) -> ParseSlice<Self, F>
    where
        F: Fn(&[u8]) -> B,
    {
        ParseSlice { src: self, f }
    }
    #[inline]
    fn and_then<B, F>(self, f: F) -> FlatMap<Self, F>
    where
        B: DecodeBytes,
        F: Fn(Self::Output) -> B,
    {
        FlatMap { src: self, f }
    }
    #[inline]
    fn and_then_<B, F>(self, f: F) -> FlatMap_<Self, F>
    where
        B: DecodeBytes,
        F: Fn(&Self::Output) -> B,
    {
        FlatMap_ { src: self, f }
    }
    #[inline]
    fn or<B: DecodeBytes<Output = Self::Output>>(self, other: B) -> Alternative<Self, B> {
        Alternative { a: self, b: other }
    }
    #[inline]
    fn many_(self) -> Many_<Self> {
        Many_ { one: self }
    }
    #[inline]
    fn many(self) -> Many<Self> {
        Many { one: self }
    }
    #[inline]
    fn repeat(self, n: u64) -> Repeat<Self> {
        Repeat { one: self, n }
    }
    #[inline]
    fn repeat_(self, n: u64) -> Repeat_<Self> {
        Repeat_ { one: self, n }
    }
}

pub struct FilterMap<D, F> {
    src: D,
    f: F,
}
impl<T, D, F> DecodeBytes for FilterMap<D, F>
where
    D: DecodeBytes,
    F: Fn(D::Output) -> Option<T>,
{
    type Output = T;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], T), DecodeError> {
        let (remainder, r) = self.src.decode(bytes)?;
        let f = &self.f;

        match f(r) {
            Some(x) => Ok((remainder, x)),
            _ => Err(DecodeError::Fail),
        }
    }
}

pub struct BytesConsumed<D> {
    src: D,
}
impl<D: DecodeBytes> DecodeBytes for BytesConsumed<D> {
    type Output = usize;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], usize), DecodeError> {
        let total_len = bytes.len();
        let (remainder, _) = self.src.decode(bytes)?;

        Ok((remainder, total_len - remainder.len()))
    }
}

pub struct Filter<D, F> {
    src: D,
    f: F,
}
impl<D, F> DecodeBytes for Filter<D, F>
where
    D: DecodeBytes,
    F: Fn(&D::Output) -> bool,
{
    type Output = D::Output;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError> {
        let (remainder, x) = self.src.decode(bytes)?;
        let f = &self.f;

        if f(&x) {
            Ok((remainder, x))
        } else {
            Err(DecodeError::Fail)
        }
    }
}

// Functor
pub struct Map<D, F> {
    src: D,
    f: F,
}

impl<B, D: DecodeBytes, F> DecodeBytes for Map<D, F>
where
    F: Fn(D::Output) -> B,
{
    type Output = B;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], B), DecodeError> {
        let (remainder, x) = self.src.decode(bytes)?;
        let f = &self.f;

        Ok((remainder, f(x)))
    }
}
pub struct ParseSlice<D, F> {
    src: D,
    f: F,
}
impl<B, D: DecodeBytes, F> DecodeBytes for ParseSlice<D, F>
where
    F: Fn(&[u8]) -> B,
{
    type Output = B;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], B), DecodeError> {
        let total_len = bytes.len();
        let (remainder, _) = self.src.decode(bytes)?;
        let slice = &bytes[..(total_len - remainder.len())];
        let f = &self.f;

        Ok((remainder, f(slice)))
    }
}
// Monad
pub struct FlatMap<D, F> {
    src: D,
    f: F,
}

impl<B: DecodeBytes, D: DecodeBytes, F> DecodeBytes for FlatMap<D, F>
where
    F: Fn(D::Output) -> B,
{
    type Output = B::Output;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], B::Output), DecodeError> {
        let (remainder, x) = self.src.decode(bytes)?;
        let f = &self.f;

        let next = f(x);
        let (next_remainder, o) = next.decode(remainder)?;
        Ok((next_remainder, o))
    }
}
pub struct FlatMap_<D, F> {
    src: D,
    f: F,
}

impl<B: DecodeBytes, D: DecodeBytes, F> DecodeBytes for FlatMap_<D, F>
where
    F: Fn(&D::Output) -> B,
{
    type Output = D::Output;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], D::Output), DecodeError> {
        let (remainder, x) = self.src.decode(bytes)?;
        let f = &self.f;

        let next = f(&x);

        let (next_remainder, _) = next.decode(remainder)?;
        Ok((next_remainder, x))
    }
}
// Alternative
pub struct Alternative<A, B> {
    a: A,
    b: B,
}
impl<A, B> DecodeBytes for Alternative<A, B>
where
    A: DecodeBytes,
    B: DecodeBytes<Output = A::Output>,
{
    type Output = A::Output;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], A::Output), DecodeError> {
        match self.a.decode(bytes) {
            Err(DecodeError::Fail) => self.b.decode(bytes),
            x @ _ => x,
        }
    }
}
pub struct Many_<D> {
    one: D,
}
impl<D: DecodeBytes> DecodeBytes for Many_<D> {
    type Output = ();

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], ()), DecodeError> {
        let mut bytes = bytes;
        loop {
            match self.one.decode(bytes) {
                Ok((remainder, _)) => {
                    bytes = remainder;
                }
                Err(DecodeError::Incomplete) => return Err(DecodeError::Incomplete),
                _ => return Ok((bytes, ())),
            }
        }
    }
}
pub struct Many<D> {
    one: D,
}
impl<D: DecodeBytes> DecodeBytes for Many<D> {
    type Output = Vec<D::Output>;

    #[inline]
    fn decode<'a, 'b>(
        &'a self,
        bytes: &'b [u8],
    ) -> Result<(&'b [u8], Vec<D::Output>), DecodeError> {
        let mut results = vec![];
        let mut bytes = bytes;
        loop {
            match self.one.decode(bytes) {
                Ok((remainder, v)) => {
                    results.push(v);
                    bytes = remainder;
                }
                Err(DecodeError::Incomplete) => return Err(DecodeError::Incomplete),
                _ => return Ok((bytes, results)),
            }
        }
    }
}
pub struct Repeat<D> {
    one: D,
    n: u64,
}
impl<D: DecodeBytes> DecodeBytes for Repeat<D> {
    type Output = Vec<D::Output>;

    #[inline]
    fn decode<'a, 'b>(
        &'a self,
        bytes: &'b [u8],
    ) -> Result<(&'b [u8], Vec<D::Output>), DecodeError> {
        let mut results = vec![];
        let mut bytes = bytes;
        for _ in 0..self.n {
            match self.one.decode(bytes) {
                Ok((remainder, v)) => {
                    results.push(v);
                    bytes = remainder;
                }
                Err(DecodeError::Incomplete) => return Err(DecodeError::Incomplete),
                _ => return Err(DecodeError::Fail),
            }
        }
        Ok((bytes, results))
    }
}
pub struct Repeat_<D> {
    one: D,
    n: u64,
}
impl<D: DecodeBytes> DecodeBytes for Repeat_<D> {
    type Output = ();

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], ()), DecodeError> {
        let mut bytes = bytes;
        for _ in 0..self.n {
            match self.one.decode(bytes) {
                Ok((remainder, _)) => {
                    bytes = remainder;
                }
                Err(DecodeError::Incomplete) => return Err(DecodeError::Incomplete),
                _ => return Err(DecodeError::Fail),
            }
        }
        Ok((bytes, ()))
    }
}
pub enum Never {}
pub struct Fail;

impl DecodeBytes for Fail {
    type Output = Never;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Never), DecodeError> {
        Err(DecodeError::Fail)
    }
}

pub struct Halt;

impl DecodeBytes for Halt {
    type Output = Never;

    #[inline]
    fn decode<'a, 'b>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Never), DecodeError> {
        Err(DecodeError::Incomplete)
    }
}
