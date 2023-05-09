use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use sycamore::reactive::*;

pub fn bench(c: &mut Criterion) {
    c.bench_function("reactivity_signals new", |b| {
        let root = sycamore_reactive3::create_root(|cx| {
            b.iter(|| {
                let child_scope = sycamore_reactive3::create_child_scope(cx, |cx| {
                    let state = sycamore_reactive3::signals::create_signal(cx, 0);

                    for _i in 0..1000 {
                        state.set(state.get() + 1);
                    }
                });

                child_scope.dispose();
            });
        });
        root.dispose();
    });

    c.bench_function("reactivity_signals", |b| {
        b.iter(|| {
            create_scope_immediate(|cx| {
                let state = create_signal(cx, 0);

                for _i in 0..1000 {
                    state.set(*state.get() + 1);
                }
            });
        });
    });

    c.bench_function("reactivity_effects", |b| {
        b.iter(|| {
            create_scope_immediate(|cx| {
                let state = create_signal(cx, 0);
                create_effect(cx, || {
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
                let v = create_signal(cx, (0..100).collect());
                let mapped = map_indexed(cx, v, |_, x| x * 2);
                mapped.track();

                v.set((100..200).collect());
                mapped.track();
            });
        });
    });

    c.bench_function("reactivity_map_keyed", |b| {
        b.iter(|| {
            create_scope_immediate(|cx| {
                let v = create_signal(cx, (0..100).collect());
                let mapped = map_keyed(cx, v, |_, x| x * 2, |x| *x);
                mapped.track();

                v.set((100..200).collect());
                mapped.track();
            });
        });
    });

    c.bench_function("reactivity_context_deeply_nested", |b| {
        b.iter_batched(
            || {
                let trigger = create_rc_signal(());
                let trigger_clone = trigger.clone();
                create_scope_immediate(move |cx| {
                    let state = create_signal(cx, 0i32);
                    provide_context_ref(cx, state);

                    fn create_nested_child_scopes(cx: Scope, depth: usize, cb: impl FnOnce(Scope)) {
                        if depth == 0 {
                            cb(cx);
                            return;
                        }

                        create_child_scope(cx, |cx| {
                            provide_context::<i32>(cx, 0i32);
                            create_nested_child_scopes(cx, depth - 1, cb);
                        });
                    }

                    create_nested_child_scopes(cx, 100, |cx| {
                        create_effect(cx, move || {
                            trigger.track();
                            let state: &Signal<i32> = use_context(cx);
                            black_box(state);
                        });
                    });
                });
                trigger_clone
            },
            |trigger| trigger.set(()),
            BatchSize::SmallInput,
        );
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.05 /* noisy CI */);
    targets = bench
}
criterion_main!(benches);
