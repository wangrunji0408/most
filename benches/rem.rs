#![feature(portable_simd)]
#![feature(core_intrinsics)]

use criterion::*;
use most::{U128x8, U192x8, U192};

criterion_group!(benches, bench, u128x8, u192x8);
criterion_main!(benches);

fn u128x8(c: &mut Criterion) {
    let mut x = U128x8::ZERO;
    let y = U128x8::MAX;
    let mut m = Default::default();
    c.bench_function("u128x8/add", |b| b.iter(|| x = x + y));
    c.bench_function("u128x8/sub", |b| b.iter(|| x = x - y));
    c.bench_function("u128x8/lanes_gt", |b| b.iter(|| m = x.lanes_gt(y)));
    c.bench_function("u128x8/sub_on_ge", |b| b.iter(|| x = x.sub_on_ge(y)));
    c.bench_function("u128x8/mul10", |b| b.iter(|| x = x.mul10_add(1)));
    black_box(m);
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
    c.bench_function("rem_u128_x8", |b| {
        let mut f = [0x12345678_u128; 8];
        b.iter(|| {
            for f in &mut f {
                *f = rem_u128(*f, M2);
            }
        })
    });
    c.bench_function("rem_u128_simd8", |b| {
        let mut f = U128x8::splat(0x12345678);
        b.iter(|| f = rem_u128x8_m2(f))
    });
    c.bench_function("rem_u192_x8", |b| {
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
    c.bench_function("rem_u192_simd8", |b| {
        let mut f = U192x8::MAX;
        b.iter(|| f = rem_u192x8_m3(f))
    });
    c.bench_function("task1", |b| {
        let mut f1 = [0u64; N];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f1 {
                *f = (*f * 10 + x as u64) % M1;
            }
        })
    });
    c.bench_function("task1_32x2", |b| {
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
    c.bench_function("task1_32x2_simd", |b| {
        use std::simd::u32x16;
        let mut f1 = [u32x16::default(); N / 16];
        let mut f2 = [u32x16::default(); N / 16];
        let x = 3u8;
        b.iter(|| {
            for (f1, f2) in f1.iter_mut().zip(f2.iter_mut()) {
                *f1 = (*f1 * u32x16::splat(10) + u32x16::splat(x as _)) % u32x16::splat(M1_1);
                *f2 = (*f2 * u32x16::splat(10) + u32x16::splat(x as _)) % u32x16::splat(M1_2);
            }
        })
    });
    c.bench_function("task1_simd", |b| {
        use std::simd::u64x8;
        let mut f1 = [u64x8::splat(0); N / 8];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f1 {
                let ff = *f * u64x8::splat(10) + u64x8::splat(x as u64);
                let ff = ff.min(ff - u64x8::splat(M1 * 4));
                let ff = ff.min(ff - u64x8::splat(M1 * 4));
                let ff = ff.min(ff - u64x8::splat(M1 * 2));
                let ff = ff.min(ff - u64x8::splat(M1 * 1));
                *f = ff;
            }
        })
    });
    c.bench_function("task2", |b| {
        let mut f2 = [0u128; N];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f2 {
                let ff = *f * 10 + x as u128;
                *f = rem_u128(ff, M2);
            }
        })
    });
    c.bench_function("task2_simd", |b| {
        let mut f2 = [U128x8::default(); N / 8];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f2 {
                let ff = rem_u128x8_m2(f.mul10_add(x as _));
                *f = ff;
            }
        })
    });
    c.bench_function("task3", |b| {
        let mut f3 = [U192::ZERO; N];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f3 {
                let ff = rem_u192_m3((*f << 1) + (*f << 3) + x);
                *f = ff;
            }
        })
    });
    c.bench_function("task3_simd", |b| {
        let mut f3 = [U192x8::ZERO; N];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f3 {
                let ff = rem_u192x8_m3(f.mul10_add(x as _));
                *f = ff;
            }
        })
    });
    c.bench_function("task4", |b| {
        let mut f4 = [(0u128, 0u128, 0u128); N];
        let x = 3u8;
        b.iter(|| {
            for (f2, f3, f7) in &mut f4 {
                let ff2 = (*f2 * 10 + x as u128) & ((1 << 75) - 1);
                let ff3 = rem_u128(*f3 * 10 + x as u128, M4_3);
                let ff7 = rem_u128(*f7 * 10 + x as u128, M4_7);
                (*f2, *f3, *f7) = (ff2, ff3, ff7);
            }
        })
    });
    c.bench_function("task4_simd", |b| {
        let mut f4 = [(U128x8::default(), U128x8::default(), U128x8::default()); N / 8];
        let x = 3u8;
        b.iter(|| {
            for (f2, f3, f7) in &mut f4 {
                let ff2 = f2.mul10_add(x as _) & U128x8::splat((1 << 75) - 1);
                let ff3 = rem_u128x8_m4_3(f3.mul10_add(x as _));
                let ff7 = rem_u128x8_m4_7(f7.mul10_add(x as _));
                (*f2, *f3, *f7) = (ff2, ff3, ff7);
            }
        })
    });
    c.bench_function("u192_load_store", |b| {
        let mut f3 = [M3; N];
        b.iter(|| {
            for f in &mut f3 {
                *f = black_box(*f);
            }
        })
    });
    c.bench_function("u128_load_store", |b| {
        let mut f2 = [0u128; N];
        b.iter(|| {
            for f in &mut f2 {
                *f = black_box(*f);
            }
        })
    });
    c.bench_function("u128x3_load_store", |b| {
        let mut f4 = [(0u128, 0u128, 0u128); N];
        b.iter(|| {
            for f in &mut f4 {
                *f = black_box(*f);
            }
        })
    });
}

const N: usize = 512;
const M1: u64 = 20220209192254;
const M1_1: u32 = 2 * 3588061;
const M1_2: u32 = 23 * 122509;
const M2: u128 = 104648257118348370704723099;
const M3: U192 = U192([0x32b9c8672a627dd5, 0x959989af0854b90, 0x14e1878814c9d]);
const M4_3: u128 = 717897987691852588770249;
const M4_7: u128 = 1341068619663964900807;

#[inline]
fn rem_u128x8_m2(mut x: U128x8) -> U128x8 {
    const MX4: U128x8 = U128x8::splat(M2 * 4);
    const MX2: U128x8 = U128x8::splat(M2 * 2);
    const MX1: U128x8 = U128x8::splat(M2 * 1);
    x = x.sub_on_ge(MX4);
    x = x.sub_on_ge(MX4);
    x = x.sub_on_ge(MX2);
    x = x.sub_on_ge(MX1);
    x
}

#[inline]
fn rem_u192x8_m3(mut x: U192x8) -> U192x8 {
    const MX4: U192x8 = U192x8::splat(M3.mul(4).0);
    const MX2: U192x8 = U192x8::splat(M3.mul(2).0);
    const MX1: U192x8 = U192x8::splat(M3.0);
    x = x.sub_on_ge(MX4);
    x = x.sub_on_ge(MX4);
    x = x.sub_on_ge(MX2);
    x = x.sub_on_ge(MX1);
    x
}

#[inline]
fn rem_u128x8_m4_3(mut x: U128x8) -> U128x8 {
    const MX4: U128x8 = U128x8::splat(M4_3 * 4);
    const MX2: U128x8 = U128x8::splat(M4_3 * 2);
    const MX1: U128x8 = U128x8::splat(M4_3 * 1);
    x = x.sub_on_ge(MX4);
    x = x.sub_on_ge(MX4);
    x = x.sub_on_ge(MX2);
    x = x.sub_on_ge(MX1);
    x
}
#[inline]
fn rem_u128x8_m4_7(mut x: U128x8) -> U128x8 {
    const MX4: U128x8 = U128x8::splat(M4_7 * 4);
    const MX2: U128x8 = U128x8::splat(M4_7 * 2);
    const MX1: U128x8 = U128x8::splat(M4_7 * 1);
    x = x.sub_on_ge(MX4);
    x = x.sub_on_ge(MX4);
    x = x.sub_on_ge(MX2);
    x = x.sub_on_ge(MX1);
    x
}

#[inline]
fn rem_u128(x: u128, m: u128) -> u128 {
    if x >= m * 5 {
        if x >= m * 7 {
            if x >= m * 9 {
                x - m * 9
            } else if x >= m * 8 {
                x - m * 8
            } else {
                x - m * 7
            }
        } else {
            if x >= m * 6 {
                x - m * 6
            } else {
                x - m * 5
            }
        }
    } else {
        if x >= m * 2 {
            if x >= m * 4 {
                x - m * 4
            } else if x >= m * 3 {
                x - m * 3
            } else {
                x - m * 2
            }
        } else {
            if x >= m * 1 {
                x - m * 1
            } else {
                x
            }
        }
    }
}

#[inline]
fn rem_u192_m3(x: U192) -> U192 {
    const M3S: [U192; 10] = [
        M3.mul(0),
        M3.mul(1),
        M3.mul(2),
        M3.mul(3),
        M3.mul(4),
        M3.mul(5),
        M3.mul(6),
        M3.mul(7),
        M3.mul(8),
        M3.mul(9),
    ];
    if x >= M3S[5] {
        if x >= M3S[7] {
            if x >= M3S[9] {
                x - M3S[9]
            } else if x >= M3S[8] {
                x - M3S[8]
            } else {
                x - M3S[7]
            }
        } else {
            if x >= M3S[6] {
                x - M3S[6]
            } else {
                x - M3S[5]
            }
        }
    } else {
        if x >= M3S[2] {
            if x >= M3S[4] {
                x - M3S[4]
            } else if x >= M3S[3] {
                x - M3S[3]
            } else {
                x - M3S[2]
            }
        } else {
            if x >= M3S[1] {
                x - M3S[1]
            } else {
                x
            }
        }
    }
}
