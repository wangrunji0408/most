#![feature(portable_simd)]

use criterion::*;
use most::*;
use std::simd::{u32x16, u64x8};

criterion_group!(benches, bench, u128x8, u192x8);
criterion_main!(benches);

fn u128x8(c: &mut Criterion) {
    let mut x = U128x8::ZERO;
    let y = U128x8::MAX;
    c.bench_function("u128x8/sub_on_ge", |b| b.iter(|| x = x.sub_on_ge(y)));
    c.bench_function("u128x8/mul10", |b| b.iter(|| x = x.mul10_add(1)));
}

fn u192x8(c: &mut Criterion) {
    let mut x = U192x8::ZERO;
    let y = U192x8::MAX;
    let mut m = Default::default();
    c.bench_function("u192x8/add", |b| b.iter(|| x = x + y));
    c.bench_function("u192x8/sub", |b| b.iter(|| x = x - y));
    c.bench_function("u192x8/lanes_gt", |b| b.iter(|| m = x.lanes_gt(y)));
    c.bench_function("u192x8/sub_on_ge", |b| b.iter(|| x = x.sub_on_ge(y)));
    c.bench_function("u192x8/mul10", |b| b.iter(|| x = x.mul10_add(1)));
    black_box(m);
}

fn bench(c: &mut Criterion) {
    c.bench_function("rem/128x8/scalar", |b| {
        let mut f = [0x12345678_u128; 8];
        b.iter(|| {
            for f in &mut f {
                *f = rem_u128(*f, M2);
            }
        })
    });
    c.bench_function("rem/128x8/simd", |b| {
        let mut f = U128x8::splat(0x12345678);
        b.iter(|| f = f.rem10(M2))
    });
    c.bench_function("rem/192x8/scalar", |b| {
        let mut f = [
            M3,
            M3.mul(2),
            M3.mul(3),
            M3.mul(4),
            M3.mul(5),
            M3.mul(6),
            M3.mul(7),
            M3.mul(8),
        ];
        b.iter(|| {
            for f in &mut f {
                *f = rem_u192_m3(*f);
            }
        })
    });
    c.bench_function("rem/192x8/simd", |b| {
        let mut f = U192x8::MAX;
        b.iter(|| f = rem_u192x8_m3(f))
    });
    c.bench_function("m1/64/scalar", |b| {
        let mut f1 = [0u64; N];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f1 {
                *f = (*f * 10 + x as u64) % M1;
            }
        })
    });
    c.bench_function("m1/32x2/scalar", |b| {
        let mut f1 = [0u32; N];
        let mut f2 = [0u32; N];
        let x = 3u8;
        b.iter(|| {
            for (f1, f2) in f1.iter_mut().zip(f2.iter_mut()) {
                *f1 = (*f1 * 10 + x as u32) % M1_1;
                *f2 = (*f2 * 10 + x as u32) % M1_2;
            }
        })
    });
    c.bench_function("m1/32x2/simd", |b| {
        let mut f1 = [(u32x16::default(), u32x16::default()); N / 16];
        let x = 3u8;
        b.iter(|| {
            for (f1, f2) in f1.iter_mut() {
                let ff1 = rem_u32x16(*f1 * u32x16::splat(10) + u32x16::splat(x as _), M1_1);
                let ff2 = rem_u32x16(*f2 * u32x16::splat(10) + u32x16::splat(x as _), M1_2);
                (*f1, *f2) = (ff1, ff2);
            }
        })
    });
    c.bench_function("m1/32/simd", |b| {
        let mut f1 = [u32x16::default(); N / 16];
        let x = 3u8;
        b.iter(|| {
            for f1 in f1.iter_mut() {
                let ff1 = rem_u32x16(*f1 * u32x16::splat(10) + u32x16::splat(x as _), M1_2);
                *f1 = ff1;
            }
        })
    });
    c.bench_function("m1/64/simd", |b| {
        let mut f1 = [u64x8::splat(0); N / 8];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f1 {
                *f = rem_u64x8(*f * u64x8::splat(10) + u64x8::splat(x as u64), M1);
            }
        })
    });
    c.bench_function("m2/128/scalar", |b| {
        let mut f2 = [0u128; N];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f2 {
                let ff = *f * 10 + x as u128;
                *f = rem_u128(ff, M2);
            }
        })
    });
    c.bench_function("m2/128/simd", |b| {
        let mut f2 = [U128x8::default(); N / 8];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f2 {
                let ff = f.mul10_add(x as _).rem10(M2);
                *f = ff;
            }
        })
    });
    c.bench_function("m3/192/scalar", |b| {
        let mut f3 = [U192::ZERO; N];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f3 {
                let ff = rem_u192_m3((*f << 1) + (*f << 3) + x);
                *f = ff;
            }
        })
    });
    c.bench_function("m3/192/simd", |b| {
        let mut f3 = [U192x8::ZERO; N];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f3 {
                let ff = rem_u192x8_m3(f.mul10_add(x as _));
                *f = ff;
            }
        })
    });
    c.bench_function("m3/64x3/simd", |b| {
        let mut f3 = [(u64x8::default(), u64x8::default(), u64x8::default()); N / 8];
        let x = 3u8;
        b.iter(|| {
            for (f1, f2, f3) in &mut f3 {
                *f1 = rem_u64x8(*f1 * u64x8::splat(10) + u64x8::splat(x as _), M3_1);
                *f2 = rem_u64x8(*f2 * u64x8::splat(10) + u64x8::splat(x as _), M3_2);
                *f3 = rem_u64x8(*f3 * u64x8::splat(10) + u64x8::splat(x as _), M3_3);
            }
        })
    });
    c.bench_function("m4/128x3/scalar", |b| {
        let mut f4 = [(0u128, 0u128, 0u128); N];
        let x = 3u8;
        b.iter(|| {
            for (f2, f3, f7) in &mut f4 {
                let ff2 = rem_u128(*f2 * 10 + x as u128, M4_11);
                let ff3 = rem_u128(*f3 * 10 + x as u128, M4_3);
                let ff7 = rem_u128(*f7 * 10 + x as u128, M4_7);
                (*f2, *f3, *f7) = (ff2, ff3, ff7);
            }
        })
    });
    c.bench_function("m4/128x3/simd", |b| {
        let mut f4 = [(U128x8::default(), U128x8::default(), U128x8::default()); N / 8];
        let x = 3u8;
        b.iter(|| {
            for (f2, f3, f7) in &mut f4 {
                let ff2 = f2.mul10_add(x as _).rem10(M4_11);
                let ff3 = f3.mul10_add(x as _).rem10(M4_3);
                let ff7 = f7.mul10_add(x as _).rem10(M4_7);
                (*f2, *f3, *f7) = (ff2, ff3, ff7);
            }
        })
    });
    c.bench_function("m3/check", |b| {
        let digits: Vec<u8> = (0..N).map(|i| i as u8 % 10).collect();
        b.iter(|| {
            let mut f2 = 0;
            let mut f3 = 0;
            for &x in &digits {
                f2 = (f2 * 10 + x as u64) % M3_2;
                f3 = (f3 * 10 + x as u64) % M3_3;
            }
            f2 == 0 && f3 == 0
        })
    });
    c.bench_function("m4/check", |b| {
        let digits: Vec<u8> = (0..N).map(|i| i as u8 % 10).collect();
        b.iter(|| {
            let mut f1 = 0;
            let mut f2 = 0;
            let mut f3 = 0;
            for &x in &digits {
                f1 = rem_u128(f1 * 10 + x as u128, M4_3);
                f2 = rem_u128(f2 * 10 + x as u128, M4_7);
                f3 = rem_u128(f3 * 10 + x as u128, M4_11);
            }
            f1 == 0 && f2 == 0 && f3 == 0
        })
    });
    c.bench_function("load_store/u192", |b| {
        let mut f3 = [M3; N];
        b.iter(|| {
            for f in &mut f3 {
                *f = black_box(*f);
            }
        })
    });
    c.bench_function("load_store/u128", |b| {
        let mut f2 = [0u128; N];
        b.iter(|| {
            for f in &mut f2 {
                *f = black_box(*f);
            }
        })
    });
    c.bench_function("load_store/u128x3", |b| {
        let mut f4 = [(0u128, 0u128, 0u128); N];
        b.iter(|| {
            for f in &mut f4 {
                *f = black_box(*f);
            }
        })
    });
}
