use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sycamore::reactive::*;

pub fn bench(c: &mut Criterion) {
    c.bench_function("reactivity_signals", |b| {
        b.iter(|| {
            create_scope_immediate(|cx| {
                let state = cx.create_signal(0);

                for _i in 0..1000 {
                    state.set(*state.get() + 1);
                }
            });
        });
    });

    c.bench_function("reactivity_effects", |b| {
        b.iter(|| {
            create_scope_immediate(|cx| {
                let state = cx.create_signal(0);
                cx.create_effect(|| {
                    let double = *state.get() * 2;
                    black_box(double);
                });
                for _i in 0..1000 {
                    state.set(*state.get() + 1);
                }
            });
        });
    });

    c.bench_function("reactivity_map_indexed", |b| {
        b.iter(|| {
            create_scope_immediate(|cx| {
                let v = cx.create_signal((0..100).collect());
                let mapped = cx.map_indexed(v, |_, x| x * 2);
                mapped.track();

                v.set((100..200).collect());
                mapped.track();
            });
        });
    });

    c.bench_function("reactivity_map_keyed", |b| {
        b.iter(|| {
            create_scope_immediate(|cx| {
                let v = cx.create_signal((0..100).collect());
                let mapped = cx.map_keyed(v, |_, x| x * 2, |x| *x);
                mapped.track();

                v.set((100..200).collect());
                mapped.track();
            });
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.05 /* noisy CI */);
    targets = bench
}
criterion_main!(benches);
