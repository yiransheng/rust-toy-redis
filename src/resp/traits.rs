use bytes::Bytes;
use std::mem;
use std::convert::{AsRef, From, Into};

#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    Incomplete,
    Fail,
}

pub trait DecodeBytes: Sized {
    type Output;

    fn decode(&self, bytes: &[u8]) -> Result<(usize, Self::Output), DecodeError>;

    fn unwrap_fail<T>(self) -> UnwrapFail<Self>
    where
        Self::Output: Into<Option<T>>,
    {
        UnwrapFail { src: self.into() }
    }

    fn count(self) -> BytesConsumed<Self> {
        BytesConsumed { src: self }
    }
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Output) -> B,
    {
        Map { src: self, f }
    }
    fn map_slice<B, F>(self, f: F) -> MapSlice<Self, F>
    where
        F: Fn(&[u8]) -> B,
    {
        MapSlice { src: self, f }
    }
    fn and_then<B, F>(self, f: F) -> FlatMap<Self, F>
    where
        B: DecodeBytes,
        F: Fn(Self::Output) -> B,
    {
        FlatMap { src: self, f }
    }
    fn and_then_<B, F>(self, f: F) -> FlatMap_<Self, F>
    where
        B: DecodeBytes,
        F: Fn(&Self::Output) -> B,
    {
        FlatMap_ { src: self, f }
    }
    fn or<B: DecodeBytes<Output = Self::Output>>(self, other: B) -> Alternative<Self, B> {
        Alternative { a: self, b: other }
    }
    fn many_(self) -> Many_<Self> {
        Many_ { one: self }
    }
    fn many(self) -> Many<Self> {
        Many { one: self }
    }
    fn repeat(self, n: u64) -> Repeat<Self> {
        Repeat { one: self, n }
    }
    fn repeat_(self, n: u64) -> Repeat_<Self> {
        Repeat_ { one: self, n }
    }
}

pub struct UnwrapFail<D> {
    src: D,
}
impl<T, D> DecodeBytes for UnwrapFail<D>
where
    D: DecodeBytes<Output = Option<T>>,
{
    type Output = T;

    #[inline]
    fn decode(&self, bytes: &[u8]) -> Result<(usize, T), DecodeError> {
        let (consumed, r) = self.src.decode(bytes)?;

        match r {
            Some(x) => Ok((consumed, x)),
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
    fn decode(&self, bytes: &[u8]) -> Result<(usize, usize), DecodeError> {
        let (consumed, _) = self.src.decode(bytes)?;

        Ok((consumed, consumed))
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
    fn decode(&self, bytes: &[u8]) -> Result<(usize, B), DecodeError> {
        let (consumed, x) = self.src.decode(bytes)?;
        let f = &self.f;

        Ok((consumed, f(x)))
    }
}

pub struct MapSlice<D, F> {
    src: D,
    f: F,
}
impl<B, D: DecodeBytes, F> DecodeBytes for MapSlice<D, F>
where
    F: Fn(&[u8]) -> B,
{
    type Output = B;

    #[inline]
    fn decode(&self, bytes: &[u8]) -> Result<(usize, B), DecodeError> {
        let (consumed, _) = self.src.decode(bytes)?;
        let slice = &bytes[0..consumed];
        let f = &self.f;

        Ok((consumed, f(slice)))
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
    fn decode(&self, bytes: &[u8]) -> Result<(usize, B::Output), DecodeError> {
        let (consumed, x) = self.src.decode(bytes)?;
        let f = &self.f;

        let next = f(x);
        let bytes_len = bytes.len();
        if bytes_len > consumed {
            let (next_consumed, o) = next.decode(&bytes[consumed..])?;
            Ok((consumed + next_consumed, o))
        } else {
            Err(DecodeError::Incomplete)
        }
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
    fn decode(&self, bytes: &[u8]) -> Result<(usize, D::Output), DecodeError> {
        let (consumed, x) = self.src.decode(bytes)?;
        let f = &self.f;

        let next = f(&x);
        let bytes_len = bytes.len();
        if bytes_len > consumed {
            let (next_consumed, o) = next.decode(&bytes[consumed..])?;
            Ok((consumed + next_consumed, x))
        } else {
            Err(DecodeError::Incomplete)
        }
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
    fn decode(&self, bytes: &[u8]) -> Result<(usize, A::Output), DecodeError> {
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
    fn decode(&self, bytes: &[u8]) -> Result<(usize, ()), DecodeError> {
        let total_len = bytes.len();
        let mut total_consumed = 0;
        loop {
            let bytes = &bytes[total_consumed..];
            match self.one.decode(bytes) {
                Ok((consumed, _)) => {
                    total_consumed += consumed;
                    if total_len <= total_consumed {
                        return Err(DecodeError::Incomplete);
                    }
                }
                Err(DecodeError::Incomplete) => return Err(DecodeError::Incomplete),
                _ => return Ok((total_consumed, ())),
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
    fn decode(&self, bytes: &[u8]) -> Result<(usize, Vec<D::Output>), DecodeError> {
        let total_len = bytes.len();
        let mut total_consumed = 0;
        let mut results = vec![];
        loop {
            let bytes = &bytes[total_consumed..];
            match self.one.decode(bytes) {
                Ok((consumed, v)) => {
                    total_consumed += consumed;
                    if total_len > total_consumed {
                        results.push(v)
                    } else {
                        return Err(DecodeError::Incomplete);
                    }
                }
                Err(DecodeError::Incomplete) => return Err(DecodeError::Incomplete),
                _ => return Ok((total_consumed, results)),
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
    fn decode(&self, bytes: &[u8]) -> Result<(usize, Vec<D::Output>), DecodeError> {
        let total_len = bytes.len();
        let mut total_consumed = 0;
        let mut results = vec![];
        for _ in 0..self.n {
            let bytes = &bytes[total_consumed..];
            match self.one.decode(bytes) {
                Ok((consumed, v)) => {
                    total_consumed += consumed;
                    if total_len >= total_consumed {
                        results.push(v);
                    } else {
                        return Err(DecodeError::Incomplete);
                    }
                }
                Err(DecodeError::Incomplete) => return Err(DecodeError::Incomplete),
                _ => return Err(DecodeError::Fail),
            }
        }
        Ok((total_consumed, results))
    }
}
pub struct Repeat_<D> {
    one: D,
    n: u64,
}
impl<D: DecodeBytes> DecodeBytes for Repeat_<D> {
    type Output = ();

    #[inline]
    fn decode(&self, bytes: &[u8]) -> Result<(usize, ()), DecodeError> {
        let total_len = bytes.len();
        let mut total_consumed = 0;
        for _ in 0..self.n {
            let bytes = &bytes[total_consumed..];
            match self.one.decode(bytes) {
                Ok((consumed, v)) => {
                    total_consumed += consumed;
                    if total_len < total_consumed {
                        return Err(DecodeError::Incomplete);
                    }
                }
                Err(DecodeError::Incomplete) => return Err(DecodeError::Incomplete),
                _ => return Err(DecodeError::Fail),
            }
        }
        Ok((total_consumed, ()))
    }
}

pub enum Never {}
pub struct Fail;

impl DecodeBytes for Fail {
    type Output = Never;

    #[inline]
    fn decode(&self, bytes: &[u8]) -> Result<(usize, Never), DecodeError> {
        Err(DecodeError::Fail)
    }
}

pub struct Halt;

impl DecodeBytes for Halt {
    type Output = Never;

    #[inline]
    fn decode(&self, bytes: &[u8]) -> Result<(usize, Never), DecodeError> {
        Err(DecodeError::Incomplete)
    }
}
