use std::mem;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Okay,
    Status(String),
    Int(i64),
    Data(Vec<u8>),
    Array(Vec<Value>),
}

impl Value {
    pub fn n_bytes(&self) -> usize {
        use self::Value::*;

        match *self {
            // $-1\r\n
            Nil => 5,
            // +Ok\r\n
            Okey => 5,
            Status(ref s) => s.as_bytes().len(),
            Int(n) => count_digits(n),
            Data(ref xs) => xs.len(),
            Array(ref xs) => xs.iter().map(|v| v.n_bytes()).sum(),
        }
    }

    pub fn encoding_iter(&self) -> ValueIter {
        use self::Value::*;

        match *self {
            Nil => ValueIter::Simple("$-1\r\n".as_bytes()),
            Okey => ValueIter::Simple("+Ok\r\n".as_bytes()),
            Status(ref s) => ValueIter::Simple(s.as_bytes()),
            Int(n) => ValueIter::Simple(format!("{}", n).as_bytes()),
            Data(ref xs) => {
                let prefix = format!("${}\r\n", xs.len()).as_bytes();
                ValueIter::Prefixed(prefix, &xs[..])
            }
            Array(ref vs) => {
                if vs.len() == 0 {
                    ValueIter::Simple("*0\r\n".as_bytes())
                } else {
                    ValueIter::Array {
                        curr: &ValueIter::Simple(format!("*{}\r\n", vs.len()).as_bytes()),
                        values: &vs[..],
                    }
                }
            }
        }
    }
}

pub enum ValueIter<'a> {
    Done,
    Simple(&'a [u8]),
    Prefixed(&'a [u8], &'a [u8]),
    Array {
        curr: &'a ValueIter<'a>,
        values: &'a [Value],
    },
}

impl<'a> Iterator for ValueIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        use self::ValueIter::*;

        let iter = mem::replace(self, ValueIter::Done);
        let ret;

        match iter {
            Done => ret = None,
            Simple(s) => ret = Some(s),
            Prefixed(p, s) => {
                ret = Some(p);
                mem::replace(self, ValueIter::Simple(s));
            }
            Array {
                mut curr,
                mut values,
            } => {
                match curr {
                    Done => {
                        if values.len() == 0 {
                            // Attention: early return here
                            return None;
                        }
                        curr = &values[0].encoding_iter();
                        values = &values[1..];
                    }
                    _ => {}
                }

                ret = curr.next();
                mem::replace(self, ValueIter::Array { curr, values });
            }
        }

        ret
    }
}

fn count_digits(mut v: i64) -> usize {
    // negative sign
    let mut result = if v < 0 { 2 } else { 1 };
    let v = v.abs();
    loop {
        if v < 10 {
            return result;
        }
        if v < 100 {
            return result + 1;
        }
        if v < 1000 {
            return result + 2;
        }
        if v < 10000 {
            return result + 3;
        }

        v /= 10000;
        result += 4;
    }
}
