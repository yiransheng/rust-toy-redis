# A Toy Redis Server in Rust



## Steps

* [x] Write a parser for [redis protocol](https://redis.io/topics/protocol)
* [x] Hack `main` to accept connection and return "Ok" always (no threads/concurrency), using raw `TcpStream`
  * [x] Should fool `redis-cli`
* [ ] Run the above in a loop
* [ ] Spawn a thread for each connection (maybe use mpsc channels?)
* [ ] Parse Commands (only SET and GET) from Resp Array
  * [ ] Make a `HashMap` based storage, and handle `SET` and `GET`
  * [ ] Maybe generalize to parse more commands
* [ ] Somehow use tokio event loops
* [ ] Re-write protocols in tokio