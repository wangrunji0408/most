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
    c.bench_function("m2/128", |b| {
        let mut state = M2Data::default();
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
    c.bench_function("m4/32", |b| {
        let mut state = M4Data::default();
        let x = 3u8;
        b.iter(|| {
            state.prepare_nop();
            for _ in 0..100 {
                state.push(x);
            }
        })
    });
    c.bench_function("m1234", |b| {
        let mut s1 = M1Data::default();
        let mut s2 = M2Data::default();
        let mut s3 = M3Data::default();
        let mut s4 = M4Data::default();
        let x = 3u8;
        b.iter(|| {
            s1.prepare_nop();
            s2.prepare_nop();
            s3.prepare_nop();
            s4.prepare_nop();
            for _ in 0..100 {
                s1.push(x);
                s2.push(x);
                s3.push(x);
                s4.push(x);
            }
        })
    });
}
