use criterion::*;
use most2::*;

criterion_group!(benches, bench);
criterion_main!(benches);

fn bench(c: &mut Criterion) {
    c.bench_function("m1/32", |b| {
        let mut state = M1Data::default();
        let x = 3u8;
        b.iter(|| state.push(x))
    });
}
