use criterion::*;
use most::U256;
use primitive_types::U256 as PU256;

fn u256(c: &mut Criterion) {
    let mut x = PU256::from(u128::MAX);
    let mut y = PU256::from(2);
    c.bench_function("pt-u256/add", |b| b.iter(|| x = x + y));
    c.bench_function("pt-u256/sub", |b| b.iter(|| x = x - y));
    c.bench_function("pt-u256/sll", |b| b.iter(|| y = x << 3));
    c.bench_function("pt-u256/mul10", |b| b.iter(|| y = x * 10));
    c.bench_function("pt-u256/mul10-sll", |b| b.iter(|| y = (x << 1) + (x << 3)));
    c.bench_function("pt-u256/gt", |b| b.iter(|| x > y));

    let mut x = U256([1, 0, 0, 0]);
    let mut y = U256([2, 0, 0, 0]);
    c.bench_function("my-u256/add", |b| b.iter(|| x = x + y));
    c.bench_function("my-u256/sub", |b| b.iter(|| x = x - y));
    c.bench_function("my-u256/sll", |b| b.iter(|| x = x << 3));
    c.bench_function("my-u256/mul10-sll", |b| b.iter(|| x = (x << 1) + (x << 3)));
    c.bench_function("my-u256/gt", |b| b.iter(|| x > y));

    const N: usize = 256;
    c.bench_function("pt-u256/m3", |b| {
        const M3: PU256 = PU256([0x32b9c8672a627dd5, 0x959989af0854b90, 0x14e1878814c9d, 0x0]);
        let mut f2 = [PU256::from(0); N];
        let x = 3u8;
        let mut m3s = vec![PU256::from(0)];
        for i in 1..10 {
            m3s.push(m3s[i - 1] + M3);
        }
        b.iter(|| {
            for f in &mut f2 {
                let ff = *f * 10 + x;
                let idx = m3s.partition_point(|m| &ff >= m);
                *f = ff - m3s[idx - 1];
            }
        })
    });
    c.bench_function("my-u256/m3", |b| {
        const M3: U256 = U256([0x32b9c8672a627dd5, 0x959989af0854b90, 0x14e1878814c9d, 0x0]);
        let mut f2 = [U256::default(); N];
        let x = 3u8;
        let mut m3s = vec![U256::default()];
        for i in 1..10 {
            m3s.push(m3s[i - 1] + M3);
        }
        b.iter(|| {
            for f in &mut f2 {
                let ff = (*f << 1) + (*f << 3) + x;
                let idx = m3s.partition_point(|m| &ff >= m);
                *f = ff - m3s[idx - 1];
            }
        })
    });
}

criterion_group!(benches, u256);
criterion_main!(benches);
