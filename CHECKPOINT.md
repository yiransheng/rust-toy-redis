# Checkpoint

version: 0.1.1



## Design

There's little design considerations in the current state of the codebase, mostly poking and tweaking things as I went, and trying to get things compile.



*  Central type for the application is `RespProtocol` in resp mod, an enum sufficiently representing all encodings of redis protocol:

  * Simple String: "+Ok\r\n"

  * Error: "-ERR\r\n"

  * Integer: ":12\r\n"

  * BulkString: "$6\r\nfoobar\r\n"

  * BulkString Null: "$-1\r\n"

  * Array: 

    ```
    *3\r\n
    $3\r\n
    foo\r\n
    $-1\r\n
    $3\r\n
    bar\r\n
    ```

* A struct `StringValue` is used as value types, constructed from `RespProtocol` 's `try_into_string_value` - getting an `Ok<StringValue>` only for SimpleString and BulkString

* Storage is an `Arc<RwLock<HashMap<StringValue, StringValue>>>`, it was surprisingly easy to get it working without compile errors

* Error handling is primitive but I tried to use as little `unwrap` as possible in application code, only one error type is used:

  ```rust
  enum ProtocolError {
      BadBytes,
      ParseError,
      TypeError,
      IoError(io::Error),
  }
  ```

* Almost all functions return `Result<T, ProtocolError>`. 

  * Note: found out a neat way to use `?` syntax
  * Given a function `fn f(..) -> Result<T,  ProtocolError>`, `expr?` works if:
    * `expr: io::Result<T>` and,
    * `ProtocolError` implements `From<io::Error>` (trivially done)

* For each client connection, a new thread is spawned, not a very scalable solution, but at least it allows mult-threading and concurrently connections

* Each such thread handles non-io `ProtocolError` returned from every stage of operation by writing back an `RespProtocol::Error` message, which could be understood by `redis-cli` - not descriptive at all but keeps the connection alive

* The thread breaks its `loop` and completes if any `IoError` is encountered, logging it (disconnected client returns a "broken pipe" error from OS(Linux))

* Parsing bytes into `RespProtocol` replies on its input to be `BufRead` (I was using `read_until`), which means underlying `TcpStream` had to be cloned and wrapped in `BufReader`. There are many areas I know can be done better, but this is a major one - will need to research a better solution

* In factor, the `RespProtocol` abstraction itself is questionable, some thoughts needs to be put into it for the next iteration

* Easy next steps:

  * Use `bytes` crate, found out about its existence after staring the project - hopefully it can alleviate a lot of pains dealing with `Vec<u8>` 
  * Moving into `tokio` universe and async io - which will probably trash most of `RespProtocol` - as its key mechanism is centered around `BufRead`, whereas `tokio` uses `AsyncRead` and `Codec` etc. Don't understand all of it yet, but should be existing
  * After that maybe poke around a bit in lower level stuff in `mio` directly



## Application

`cargo run` (runs on default 6379 port, no CLI args yet)

### Supported Commands

* `SET key value`
* `GET key`
* `DEL k1, k2, k3 ...`

Running `redis-cli` to test basics:

```
127.0.0.1:6379> SET a b
Ok
127.0.0.1:6379> SET a c d
(error) ParseError
127.0.0.1:6379> GET a
"b"
127.0.0.1:6379> DEL a
(integer) 1
```

