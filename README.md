# rusty-toy-redis

A Learning project to write a redis server.

See [PLANS.md](./PLANS.md) for ideas I wanted to try.

See [CHECKPOINT.md](./CHECKPOINT.md) for current status of the code.

## Supported Commands

Only `SET`, `GET` and `DEL`

## Networking Protocol

Full RESP protocol is implemented, should be able to fool `redis-cli`

## Benchmark

```
#> redis-benchmark -t set,get -n 100000 -q

# On my computer
#> SET: 149476.83 requests per second
#> GET: 153374.23 requests per second
```

Rust is pretty amazing, achieving this level of performance for some beginner-level code.



For comparison, same benchmark run on actual `redis` server:

```
#> SET: 162337.66 requests per second
#> GET: 165289.25 requests per second
```

(A naive version that handles each connection with a new thread I had [previously](https://github.com/yiransheng/rust-toy-redis/tree/0.1.1) completely choked on 100000 connection benchmark above; Yeah, tokio..)



Also run the same benchmark on `simpledb`, a python based redis compatible [key-value server](https://github.com/coleifer/simpledb); for a sense of how much low level languages matter in terms of perf:

```
#> SET: 25374.27 requests per second
#> GET: 28129.39 requests per second
```

