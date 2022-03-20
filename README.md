# Codes for MO's Trading

[MO's Trading]（莫队交易赛） is an online contest for high frequency trading.

This repo contains my code written in Rust, which got 6th out of 10 in the finals ([board]).

The code is special optimized for x86_64 CPU with AVX512, but is expected to run on all platforms since it uses Rust's [portable SIMD].

[MO's Trading]: ./txt/most.txt
[board]: ./txt/board.txt
[portable SIMD]: https://doc.rust-lang.org/nightly/std/simd/index.html

## Related Links

- [金枪鱼之夜：高频交易与计算机科学](https://tuna.moe/event/2022/high-frequency-trading/)
- [代码优化卷翻天：莫队交易赛复盘](https://zhuanlan.zhihu.com/p/470766162)

## Run

Start mock server:
```sh
python3 most/src/server.py
```

Start program:
```sh
RUST_LOG=info cargo run --release
```

Run micro benchmarks:
```sh
cargo bench
```

## UEFI App

I also built an UEFI application for the round 2.

This solution bypasses the Linux kernel network stack and uses the UEFI stack to reduce latency.

```sh
cd most-uefi
make build
```
