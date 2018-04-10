// return some io::Error
macro_rules! io_error {
    ($kind:ident, $msg:expr) => {
        ::std::io::Error::new(::std::io::ErrorKind::$kind, $msg)
    };
}

macro_rules! io_fail {
    ($kind:ident, $msg:expr) => {
        return Err(::std::io::Error::new(::std::io::ErrorKind::$kind, $msg));
    };
}
