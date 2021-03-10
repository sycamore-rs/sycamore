use criterion::{black_box, criterion_group, criterion_main, Criterion};
use maple_core::prelude::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("reactivity signals run get/set 1000x", |b| {
        b.iter(|| {
            let state = Signal::new(black_box(0));

            for _i in 0..1000 {
                let value = state.get();
                state.set(*value + 1);
            }
        })
    });

    c.bench_function("reactivity run effects 1000x", |b| {
        b.iter(|| {
            let state = Signal::new(black_box(0));
            create_effect(cloned!((state) => move || {
                let _double = *state.get() * 2;
            }));

            for _i in 0..1000 {
                state.set(*state.get() + 1);
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
