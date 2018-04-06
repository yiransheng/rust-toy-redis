// return some io::Error
macro_rules! io_fail {
    ($kind:ident, $msg:expr) => {
        return Err(::std::io::Error::new(::std::io::ErrorKind::$kind, $msg))
    }
}

// convert Result<T, E> to Result<T, ()>
macro_rules! forget_err {
    ($expr:expr) => {
        $expr.map_err(|_| ())?
    }
}
