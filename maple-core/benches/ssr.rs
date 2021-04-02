#![allow(non_snake_case)]

use criterion::{criterion_group, criterion_main, Criterion};
use maple_core::prelude::*;

pub fn bench(c: &mut Criterion) {
    c.bench_function("ssr_small", |b| {
        b.iter(|| {
            fn App<G: GenericNode>() -> TemplateResult<G> {
                template! {
                    div(class="my-container") {
                        p { "Hello World!" }
                    }
                }
            }

            let _ssr = render_to_string(|| template! { App() });
        })
    });

    c.bench_function("ssr_medium", |b| {
        b.iter(|| {
            fn ListItem<G: GenericNode>(value: i32) -> TemplateResult<G> {
                template! {
                    p {
                        span(class="placeholder")
                        i { (value) }
                        button(class="delete") {
                            i(class="delete-icon")
                        }
                    }
                }
            }

            fn App<G: GenericNode>() -> TemplateResult<G> {
                let values = Signal::new((0i32..=10).collect::<Vec<_>>());

                template! {
                    div(class="my-container") {
                        Indexed(IndexedProps {
                            iterable: values.handle(),
                            template: |x| template! {
                                ListItem(x)
                            }
                        })
                    }
                }
            }

            let _ssr = render_to_string(|| template! { App() });
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.05 /* noisy CI */);
    targets = bench
}
criterion_main!(benches);
