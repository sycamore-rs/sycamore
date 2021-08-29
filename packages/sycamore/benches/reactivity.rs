use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sycamore::prelude::*;
use sycamore::rx::{map_indexed, map_keyed};

pub fn bench(c: &mut Criterion) {
    c.bench_function("reactivity_signals", |b| {
        b.iter(|| {
            let state = Signal::new(black_box(0));

            for _i in 0..1000 {
                state.set(*state.get() + 1);
            }
        });
    });

    c.bench_function("reactivity_effects", |b| {
        b.iter(|| {
            let state = Signal::new(black_box(0));
            create_effect(cloned!((state) => move || {
                let double = *state.get() * 2;
                black_box(double);
            }));

            for _i in 0..1000 {
                state.set(*state.get() + 1);
            }
        });
    });

    c.bench_function("reactivity_map_indexed", |b| {
        b.iter(|| {
            let v = Signal::new((0..100).collect());
            let mut mapped = map_indexed(v.handle(), |x| *x * 2);
            mapped();

            v.set((100..200).collect());
            mapped();
        });
    });

    c.bench_function("reactivity_map_keyed", |b| {
        b.iter(|| {
            let v = Signal::new((0..100).collect());
            let mut mapped = map_keyed(v.handle(), |x| *x * 2, |x| *x);
            mapped();

            v.set((100..200).collect());
            mapped();
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.05 /* noisy CI */);
    targets = bench
}
criterion_main!(benches);
