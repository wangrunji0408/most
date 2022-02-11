use criterion::*;
use num_bigint::BigUint;

fn num_bigint(c: &mut Criterion) {
    c.bench_function("parse512", |b| {
        let s: String = (0..512).map(|_| '1').collect();
        b.iter(|| {
            black_box(s.parse::<BigUint>().unwrap());
        })
    });
    c.bench_function("512/54", |b| {
        let n = (0..512)
            .map(|_| '1')
            .collect::<String>()
            .parse::<BigUint>()
            .unwrap();
        let m = "125000000000000064750000000000009507500000000000294357"
            .parse::<BigUint>()
            .unwrap();
        b.iter(|| black_box(&n % &m))
    });
}

criterion_group!(benches, num_bigint);
criterion_main!(benches);
