use criterion::*;
use most2::*;

criterion_group!(benches, bench);
criterion_main!(benches);

fn bench(c: &mut Criterion) {
    c.bench_function("m1/32", |b| {
        let mut state = M1Data::default();
        let x = 3u8;
        b.iter(|| {
            state.prepare_nop();
            for _ in 0..100 {
                state.push(x);
            }
        })
    });
    c.bench_function("m3/64", |b| {
        let mut state = M3Data::default();
        let x = 3u8;
        b.iter(|| {
            state.prepare_nop();
            for _ in 0..100 {
                state.push(x);
            }
        })
    });
}
