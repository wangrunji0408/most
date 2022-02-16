use criterion::*;
use most::U128x8;
use most::U192;

fn u256(c: &mut Criterion) {
    c.bench_function("rem_u128", |b| {
        let mut f = [0x12345678_u128; 8];
        b.iter(|| {
            for f in &mut f {
                *f = rem_u128(*f, M2);
            }
        })
    });
    c.bench_function("rem_u128_simd", |b| {
        let mut f = U128x8::splat(0x12345678);
        b.iter(|| f = rem_u128x8(f))
    });
    c.bench_function("rem_u192", |b| {
        let mut f = U192::ZERO;
        b.iter_batched(
            || {
                f = rem_u192_m3((f << 1) + (f << 3) + 1);
                f
            },
            |f| rem_u192_m3(f),
            BatchSize::SmallInput,
        )
    });
    c.bench_function("task2", |b| {
        let mut f2 = [0u128; N];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f2 {
                let ff = (*f << 1) + (*f << 3) + x as u128;
                *f = rem_u128(ff, M2);
            }
        })
    });
    c.bench_function("task2_simd", |b| {
        let mut f2 = [U128x8::default(); N / 8];
        let x = 3u8;
        b.iter(|| {
            for f in &mut f2 {
                let ff = (*f << 1) + (*f << 3) + x;
                let ff = rem_u128x8(ff);
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
    c.bench_function("task4", |b| {
        let mut f4 = [(0u128, 0u128, 0u128); N];
        let x = 3u8;
        b.iter(|| {
            for (f2, f3, f7) in &mut f4 {
                let ff2 = ((*f2 << 1) + (*f2 << 3) + x as u128) & ((1 << 75) - 1);
                let ff3 = rem_u128((*f3 << 1) + (*f3 << 3) + x as u128, M4_3);
                let ff7 = rem_u128((*f7 << 1) + (*f7 << 3) + x as u128, M4_7);
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

criterion_group!(benches, u256);
criterion_main!(benches);

const N: usize = 512;
const M2: u128 = 104648257118348370704723099;
const M3: U192 = U192([0x32b9c8672a627dd5, 0x959989af0854b90, 0x14e1878814c9d]);
const M4_3: u128 = 717897987691852588770249;
const M4_7: u128 = 1341068619663964900807;

#[inline]
fn rem_u128x8(mut x: U128x8) -> U128x8 {
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
