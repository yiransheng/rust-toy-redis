#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    Incomplete,
    Fail,
}

fn _assert_is_object_safe(_: &DecodeBytes<Output = ()>) {}

/// An interface for building decoders operating on a borrowed bytes slice.
///
/// Inspired by [monadic parser combinators] common in Haskell, and tries to follow
/// conventions established by [`std::iter::Iterator`].
///
/// This trait is generic over lifetime `'b`, which represents the lifetime
/// of borrowed `&[u8]` its `decode` method operates on. This allows associated
/// `Output` type to be references of this lifetime, for example produce a
/// sub-slice of the input as output.
///
/// [monadic parser combinators]: http://www.cs.nott.ac.uk/~pszgmh/monparsing.pdf
pub trait DecodeBytes<'b> {
    /// The type of value produced by a decoder
    type Output;

    /// Decode a sequence of bytes, return a tuple of (remaining bytes, output) if successful.
    ///
    /// # Examples
    ///
    /// Basic usages:
    ///
    /// ```
    /// let input = b"hello, world";
    /// let decoder = match_bytes(b"hello"); // implements DecodeBytes
    ///
    /// assert_eq!(
    ///     decoder.decode(&input[..]),
    ///     Ok((&b", world"[..], &b"hello"[..]))
    /// );
    ///
    /// ```
    ///
    /// #Errors
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError>;

    /// Decode bytes, discard remainder and only return [Ok(`Self::Output`)] if successful.
    #[inline]
    fn decode_<'a>(&'a self, bytes: &'b [u8]) -> Result<Self::Output, DecodeError> {
        let (_, out) = self.decode(bytes)?;
        Ok(out)
    }

    /// Similar to [`decode_`], but fails if input bytes is not consumed entirely.
    #[inline]
    fn decode_all<'a>(&'a self, bytes: &'b [u8]) -> Result<Self::Output, DecodeError> {
        let (remainder, out) = self.decode(bytes)?;
        if remainder.len() == 0 {
            Ok(out)
        } else {
            Err(DecodeError::Fail)
        }
    }

    /// Creates an DecodeBytes whose output is the number of bytes consumed by `self`.
    ///
    /// Created DecodeBytes behaves exactly like the current one, accept/fail identically
    /// on the same input - except it will report number of bytes consumed and discard
    /// the result of `self` when running on some inputs.
    ///
    /// # Examples
    ///
    ///
    /// ```
    /// let input = b"foo";
    /// let decoder = match_bytes(b"foo").count_bytes();
    ///
    /// assert_eq!(
    ///     decoder.decode_all(&input[..]),
    ///     Ok(3)
    /// );
    ///
    /// ```
    #[inline]
    fn count_bytes(self) -> BytesConsumed<Self>
    where
        Self: Sized,
    {
        BytesConsumed { src: self }
    }
    #[inline]
    fn filter_map<T, F>(self, f: F) -> FilterMap<Self, F>
    where
        F: Fn(Self::Output) -> Option<T>,
        Self: Sized,
    {
        FilterMap { src: self, f }
    }
    #[inline]
    fn filter<B, F>(self, f: F) -> Filter<Self, F>
    where
        F: Fn(&Self::Output) -> bool,
        Self: Sized,
    {
        Filter { src: self, f }
    }
    #[inline]
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Output) -> B,
        Self: Sized,
    {
        Map { src: self, f }
    }
    #[inline]
    fn parse_slice<B, F>(self, f: F) -> ParseSlice<Self, F>
    where
        F: Fn(&'b [u8]) -> B,
        Self: Sized,
    {
        ParseSlice { src: self, f }
    }
    #[inline]
    fn to_consumed_slice(self) -> ToSlice<Self>
    where
        Self: Sized,
    {
        ToSlice { src: self }
    }
    #[inline]
    fn and_then<B, F>(self, f: F) -> FlatMap<Self, F>
    where
        B: DecodeBytes<'b>,
        F: Fn(Self::Output) -> B,
        Self: Sized,
    {
        FlatMap { src: self, f }
    }
    #[inline]
    fn and_then_<B, F>(self, f: F) -> FlatMap_<Self, F>
    where
        B: DecodeBytes<'b>,
        F: Fn(&Self::Output) -> B,
        Self: Sized,
    {
        FlatMap_ { src: self, f }
    }
    #[inline]
    fn and<B: DecodeBytes<'b>>(self, snd: B) -> AndNext<Self, B>
    where
        Self: Sized,
    {
        AndNext { fst: self, snd }
    }
    #[inline]
    fn and_<B: DecodeBytes<'b>>(self, snd: B) -> AndNext_<Self, B>
    where
        Self: Sized,
    {
        AndNext_ { fst: self, snd }
    }
    #[inline]
    fn or<B: DecodeBytes<'b, Output = Self::Output>>(self, other: B) -> Alternative<Self, B>
    where
        Self: Sized,
    {
        Alternative { a: self, b: other }
    }
    #[inline]
    fn or_else<B, F>(self, f: F) -> OrElse<Self, F>
    where
        B: DecodeBytes<'b, Output = Self::Output>,
        F: Fn() -> B,
        Self: Sized,
    {
        OrElse { a: self, f }
    }
    #[inline]
    fn many_(self) -> Many_<Self>
    where
        Self: Sized,
    {
        Many_ { one: self }
    }
    #[inline]
    fn many(self) -> Many<Self>
    where
        Self: Sized,
    {
        Many { one: self }
    }
    #[inline]
    fn repeat(self, n: u64) -> Repeat<Self>
    where
        Self: Sized,
    {
        Repeat { one: self, n }
    }
    #[inline]
    fn repeat_(self, n: u64) -> Repeat_<Self>
    where
        Self: Sized,
    {
        Repeat_ { one: self, n }
    }
}

pub struct BytesConsumed<D> {
    src: D,
}
impl<'b, D: DecodeBytes<'b>> DecodeBytes<'b> for BytesConsumed<D> {
    type Output = usize;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], usize), DecodeError> {
        let total_len = bytes.len();
        let (remainder, _) = self.src.decode(bytes)?;

        Ok((remainder, total_len - remainder.len()))
    }
}

pub struct Filter<D, F> {
    src: D,
    f: F,
}
impl<'b, D, F> DecodeBytes<'b> for Filter<D, F>
where
    D: DecodeBytes<'b>,
    F: Fn(&D::Output) -> bool,
{
    type Output = D::Output;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError> {
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

impl<'b, B, D: DecodeBytes<'b>, F> DecodeBytes<'b> for Map<D, F>
where
    F: Fn(D::Output) -> B,
{
    type Output = B;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], B), DecodeError> {
        let (remainder, x) = self.src.decode(bytes)?;
        let f = &self.f;

        Ok((remainder, f(x)))
    }
}

pub struct FilterMap<D, F> {
    src: D,
    f: F,
}
impl<'b, T, D, F> DecodeBytes<'b> for FilterMap<D, F>
where
    D: DecodeBytes<'b>,
    F: Fn(D::Output) -> Option<T>,
{
    type Output = T;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], T), DecodeError> {
        let (remainder, r) = self.src.decode(bytes)?;
        let f = &self.f;

        match f(r) {
            Some(x) => Ok((remainder, x)),
            _ => Err(DecodeError::Fail),
        }
    }
}

pub struct ToSlice<D> {
    src: D,
}
impl<'b, D: DecodeBytes<'b>> DecodeBytes<'b> for ToSlice<D> {
    type Output = &'b [u8];

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError> {
        let total_len = bytes.len();
        let (remainder, _) = self.src.decode(bytes)?;
        let slice = &bytes[..(total_len - remainder.len())];

        Ok((remainder, slice))
    }
}

pub struct ParseSlice<D, F> {
    src: D,
    f: F,
}
impl<'b, B, D: DecodeBytes<'b>, F> DecodeBytes<'b> for ParseSlice<D, F>
where
    F: Fn(&'b [u8]) -> B,
{
    type Output = B;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], B), DecodeError> {
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

impl<'b, B: DecodeBytes<'b>, D: DecodeBytes<'b>, F> DecodeBytes<'b> for FlatMap<D, F>
where
    F: Fn(D::Output) -> B,
{
    type Output = B::Output;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], B::Output), DecodeError> {
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

impl<'b, B: DecodeBytes<'b>, D: DecodeBytes<'b>, F> DecodeBytes<'b> for FlatMap_<D, F>
where
    F: Fn(&D::Output) -> B,
{
    type Output = D::Output;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], D::Output), DecodeError> {
        let (remainder, x) = self.src.decode(bytes)?;
        let f = &self.f;

        let next = f(&x);

        let (next_remainder, _) = next.decode(remainder)?;
        Ok((next_remainder, x))
    }
}

pub struct AndNext<A, B> {
    fst: A,
    snd: B,
}
impl<'b, A: DecodeBytes<'b>, B: DecodeBytes<'b>> DecodeBytes<'b> for AndNext<A, B> {
    type Output = B::Output;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError> {
        let (remainder, _) = self.fst.decode(bytes)?;

        self.snd.decode(remainder)
    }
}
pub struct AndNext_<A, B> {
    fst: A,
    snd: B,
}
impl<'b, A: DecodeBytes<'b>, B: DecodeBytes<'b>> DecodeBytes<'b> for AndNext_<A, B> {
    type Output = A::Output;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError> {
        let (remainder, fst_x) = self.fst.decode(bytes)?;

        let (remainder, _) = self.snd.decode(remainder)?;

        Ok((remainder, fst_x))
    }
}

// Alternative
pub struct Alternative<A, B> {
    a: A,
    b: B,
}
impl<'b, A, B> DecodeBytes<'b> for Alternative<A, B>
where
    A: DecodeBytes<'b>,
    B: DecodeBytes<'b, Output = A::Output>,
{
    type Output = A::Output;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], A::Output), DecodeError> {
        match self.a.decode(bytes) {
            Err(DecodeError::Fail) => self.b.decode(bytes),
            x @ _ => x,
        }
    }
}

// Lazy Alternative
pub struct OrElse<A, F> {
    a: A,
    f: F,
}
impl<'b, A, B, F> DecodeBytes<'b> for OrElse<A, F>
where
    A: DecodeBytes<'b>,
    B: DecodeBytes<'b, Output = A::Output>,
    F: Fn() -> B,
{
    type Output = A::Output;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], A::Output), DecodeError> {
        let f = &self.f;
        match self.a.decode(bytes) {
            Err(DecodeError::Fail) => f().decode(bytes),
            x @ _ => x,
        }
    }
}

pub struct Many_<D> {
    one: D,
}
impl<'b, D: DecodeBytes<'b>> DecodeBytes<'b> for Many_<D> {
    type Output = ();

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], ()), DecodeError> {
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
impl<'b, D: DecodeBytes<'b>> DecodeBytes<'b> for Many<D> {
    type Output = Vec<D::Output>;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Vec<D::Output>), DecodeError> {
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
impl<'b, D: DecodeBytes<'b>> DecodeBytes<'b> for Repeat<D> {
    type Output = Vec<D::Output>;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Vec<D::Output>), DecodeError> {
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
impl<'b, D: DecodeBytes<'b>> DecodeBytes<'b> for Repeat_<D> {
    type Output = ();

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], ()), DecodeError> {
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
pub struct fail;

impl<'b> DecodeBytes<'b> for fail {
    type Output = Never;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Never), DecodeError> {
        Err(DecodeError::Fail)
    }
}

pub struct Halt;

impl<'b> DecodeBytes<'b> for Halt {
    type Output = Never;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Never), DecodeError> {
        Err(DecodeError::Incomplete)
    }
}

pub struct ExpectByte(u8);

#[inline]
pub fn match_byte(byte: u8) -> ExpectByte {
    ExpectByte(byte)
}

impl<'b> DecodeBytes<'b> for ExpectByte {
    type Output = u8;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], u8), DecodeError> {
        if bytes.len() == 0 {
            return Err(DecodeError::Incomplete);
        }

        if bytes[0] == self.0 {
            Ok((&bytes[1..], self.0))
        } else {
            Err(DecodeError::Fail)
        }
    }
}

pub const end_line: ExpectByte = ExpectByte(b'\n');
pub const end_line_crlf: ExpectBytes = ExpectBytes { bytes: b"\r\n" };

pub struct ExpectBytes {
    bytes: &'static [u8],
}
#[inline]
pub fn match_bytes(bytes: &'static [u8]) -> ExpectBytes {
    ExpectBytes { bytes }
}

impl<'b> DecodeBytes<'b> for ExpectBytes {
    type Output = &'static [u8];

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError> {
        let expected_bytes = self.bytes;
        let expected_len = expected_bytes.len();
        if bytes.len() < expected_len {
            return Err(DecodeError::Incomplete);
        }

        if &bytes[0..expected_len] == expected_bytes {
            Ok((&bytes[expected_len..], self.bytes))
        } else {
            Err(DecodeError::Fail)
        }
    }
}

pub struct line_safe_byte;

impl<'b> DecodeBytes<'b> for line_safe_byte {
    type Output = ();

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError> {
        if bytes.len() == 0 {
            return Err(DecodeError::Incomplete);
        }

        match bytes[0] {
            b'\r' => Err(DecodeError::Fail),
            b'\n' => Err(DecodeError::Fail),
            _ => Ok((&bytes[1..], ())),
        }
    }
}

pub struct any_byte;

impl<'b> DecodeBytes<'b> for any_byte {
    type Output = u8;

    #[inline]
    fn decode<'a>(&'a self, bytes: &'b [u8]) -> Result<(&'b [u8], Self::Output), DecodeError> {
        if bytes.len() == 0 {
            return Err(DecodeError::Incomplete);
        }

        Ok((&bytes[1..], bytes[0]))
    }
}
