use bytes::{Bytes, BytesMut};
use std::default::Default;
use std::iter::FromIterator;
use std::mem;

#[derive(Debug)]
pub enum Cmd<T> {
    SET { key: T, value: T },
    GET { key: T },
    DEL { keys: Arguments<T> },
}
impl Cmd<Bytes> {
    pub fn from_args(args: Arguments<Bytes>) -> Option<Self> {
        let cmd = args.first();
        match cmd {
            Some(arg) if arg.as_ref() == b"GET" => {
                if let Arguments::Two((_, ref key)) = args {
                    Some(Cmd::GET { key: key.clone() })
                } else {
                    None
                }
            }
            Some(arg) if arg.as_ref() == b"DEL" => {
                let keys: Arguments<Bytes> = args.iter().skip(1).map(|key| key.clone()).collect();
                Some(Cmd::DEL { keys })
            }
            Some(arg) if arg.as_ref() == b"SET" => {
                if let Arguments::Three((_, ref key, ref value)) = args {
                    Some(Cmd::SET {
                        key: key.clone(),
                        value: value.clone(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Arguments<Arg> {
    NoArgs,
    One(Arg),
    // use tuples for these variants, because we cannot move
    // value out of arrays; (T, T) vs [T; 2]
    Two((Arg, Arg)),
    Three((Arg, Arg, Arg)),
    More(Vec<Arg>),
}
impl<Arg> Arguments<Arg> {
    pub fn iter(&self) -> ArgumentsIter<Arg> {
        ArgumentsIter {
            args: self,
            index: 0,
        }
    }
    pub fn first(&self) -> Option<&Arg> {
        match *self {
            Arguments::NoArgs => None,
            Arguments::One(ref arg) => Some(arg),
            Arguments::Two((ref arg0, _)) => Some(arg0),
            Arguments::Three((ref arg0, _, _)) => Some(arg0),
            Arguments::More(ref xs) => xs.get(0),
        }
    }
    pub fn map<T, F>(&self, f: F) -> Arguments<T>
    where
        F: Fn(&Arg) -> T,
    {
        match *self {
            Arguments::NoArgs => Arguments::NoArgs,
            Arguments::One(ref arg) => Arguments::One(f(arg)),
            Arguments::Two((ref arg0, ref arg1)) => Arguments::Two((f(arg0), f(arg1))),
            Arguments::Three((ref arg0, ref arg1, ref arg2)) => {
                Arguments::Three((f(arg0), f(arg1), f(arg2)))
            }
            Arguments::More(ref xs) => {
                let ys = xs.iter().map(f).collect();
                Arguments::More(ys)
            }
        }
    }
    pub fn n_args(&self) -> usize {
        match *self {
            Arguments::NoArgs => 0,
            Arguments::One(_) => 1,
            Arguments::Two(_) => 2,
            Arguments::Three(_) => 3,
            Arguments::More(ref xs) => xs.len(),
        }
    }
    pub fn push(&mut self, arg: Arg) {
        let this = mem::replace(self, Arguments::NoArgs);
        let this = Self::append(this, arg);
        mem::replace(self, this);
    }
    pub fn append(self, arg: Arg) -> Self {
        match self {
            Arguments::NoArgs => Arguments::One(arg),
            Arguments::One(arg0) => Arguments::Two((arg0, arg)),
            Arguments::Two((arg0, arg1)) => Arguments::Three((arg0, arg1, arg)),
            Arguments::Three((arg0, arg1, arg2)) => Arguments::More(vec![arg0, arg1, arg2, arg]),
            Arguments::More(mut xs) => {
                xs.push(arg);
                Arguments::More(xs)
            }
        }
    }
}

impl<'a> Arguments<&'a [u8]> {
    pub fn n_bytes(&self) -> usize {
        self.iter().map(|s| s.len()).sum()
    }

    pub fn to_bytes(&self) -> Arguments<Bytes> {
        let n = self.n_bytes();
        let mut bytes = Bytes::with_capacity(n);
        let args: Arguments<_> = self.iter()
            .scan((0, 0), |(_start, end), s| {
                bytes.extend_from_slice(s);
                Some((*end, *end + s.len()))
            })
            .collect();

        args.map(|(start, end)| bytes.slice(*start, *end))
    }
}

impl<Arg> Default for Arguments<Arg> {
    fn default() -> Self {
        Arguments::NoArgs
    }
}
impl<Arg> FromIterator<Arg> for Arguments<Arg> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Arg>,
    {
        iter.into_iter()
            .fold(Self::default(), |args, arg| args.append(arg))
    }
}

pub struct ArgumentsIter<'a, Arg: 'a> {
    args: &'a Arguments<Arg>,
    index: usize,
}
impl<'a, Arg: 'a> Iterator for ArgumentsIter<'a, Arg> {
    type Item = &'a Arg;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        match *self.args {
            Arguments::NoArgs => None,
            Arguments::One(ref arg) if index == 0 => {
                self.index = index + 1;
                Some(arg)
            }
            Arguments::Two((ref arg0, ref arg1)) if index < 2 => {
                self.index = index + 1;
                Some([arg0, arg1][index])
            }
            Arguments::Three((ref arg0, ref arg1, ref arg2)) if index < 3 => {
                self.index = index + 1;
                Some([arg0, arg1, arg2][index])
            }
            Arguments::More(ref xs) => {
                self.index = index + 1;
                if index < xs.len() {
                    Some(&xs[index])
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
