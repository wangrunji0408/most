# Codes for MO's Trading

[MO's Trading]（莫队交易赛） is an online contest for high frequency trading.

This repo contains my code written in Rust, which got 6th out of 10 in the finals.

The code is special optimized for x86_64 CPU with AVX512, but is expected to run on all platforms since it uses Rust's [portable SIMD].

[MO's Trading]: ./txt/most.txt
[portable SIMD]: https://doc.rust-lang.org/nightly/std/simd/index.html

## Run

Start mock server:
```sh
python3 src/server.py
```

Start program:
```sh
RUST_LOG=info cargo run --release
```

Run micro benchmarks:
```sh
cargo bench
```
