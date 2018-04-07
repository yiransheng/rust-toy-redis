# Checkpoint

version: 0.1.2

[Previously...](https://github.com/yiransheng/rust-toy-redis/blob/0.1.1/CHECKPOINT.md)



## Design

* Replaced everything with tokio stack
* Rewrote the value type(s), `RespProtocol` -> `RedisValue`

Overall, writing a tokio service is surprisingly easy and the experience is pretty nice - once I got a good grasp of underlying layers of abstraction (`Codec`, `Proto` and `Service`).  `protocol.rs` and `service.rs` is very short and clean now.



Unfortunately, the domain logic (RESP protocol) is still quite messy and more contrived than necessary (`redis_value.rs`). A major factor that prompted to for a rewrite here is to avoid nested `Vec`s. The type I used was:

```rust
#[derive(Debug, Eq, PartialEq)]
pub enum RespProtocol {
    SimpleString(SimpleBytes),
    Error(SimpleBytes),
    Integer(SimpleBytes),
    Null,
    BulkString(Vec<u8>),
    Array(Vec<RespProtocol>),
}
```

It's a recursive type: note the `Array(Vec<RespProtocol>)` variant. If we group all non-array variants together and consider them `Primitive`, this `enum` is conceptually an nested list or an S-expression-like structure with the atoms being `Primitive`s. Another way to think about it is a nested JSON array like:

```json
[
    [
        "+OK\r\n",
        ":12\r\n",
        [],
    ]
    "$3\r\nfoo\r\n",
]
```

A kind of JSON tokenizer might represent this as:

```
[
    { type: 'array_open' },
    { type: 'array_open' },
    { type: 'primitive', value: "+OK\r\n" },
    { type: 'primitive', value: ":12\r\n" },
    { type: 'array_open' },
    { type: 'array_close' },
    { type: 'array_close' },
    { type: 'primitive', value: "$3\r\nfoo\r\n" },
    { type: 'array_close' },
]
```

The old representation just represent it as nested `Vec`s - which is simple and straightforward, but I had a thought that I could achieve a more compact representation in terms of memory layout. The new idea is to take recursive part out, and have the follow `enum` for basic values: 

```rust
pub enum Value<T> {
    SimpleString(T),
    ErrorString(T),
    IntegerString(T),
    BulkString(T),
    Nil,
}
```

For an array type, I wanted to have its data packed densely in a single buffer - with each sub-slice corresponding to one of the simple value variant (`Value::Nil` would require a zero-lengthed sub-slice). Of course, some sort of metadata type needs to be stored as well to make sense of the flat buffer. It was at this point, I decided on the "tokenizer-like" approach above, to store RESP arrays as `Vec` of tokens:

```rust
#[derive(Debug)]
pub struct RedisValue {
    pub nodes: Vec<Node<Bytes>>,
}
#[derive(Debug)]
pub enum Node<T> {
    Leaf(Value<T>),
    Open(usize),
    Close,
}
```

The `Bytes` struct from `bytes` create is an owned type but can share a same underlying buffer - which is ref counted under the hood (`Arc`). So all the leaf nodes in a `RedisValue` can have their `Bytes` backed by the same continues region in memory. The `Open` "token" also stores how many items in the current array - to make calculate the encoding size of `RedisValue` easier. 



Unfortunately, this shifts the burden of parsing to the consumer of `RedisValue`s ,for example, I had to write a parser to get a `Command` out of `RedisValue`. Also, the decoding part ended up quite complicated. I am not even entirely sure the implementation is correct (the application it self still only handles 'SET' 'GET' and 'DEL', which don't utilize nested arrays at all). I wrote some basic unit tests for decoding - but there were so many intricate bytes manipulation in and out of `BytesMut` buffer handled over by `tokio`, the cod e really wasn't very pleasant to work with.



I might revert back to the old recursive type for its simplicity...



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

