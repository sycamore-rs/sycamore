#![allow(non_snake_case)]

use maple_core::prelude::*;

fn App<G: GenericNode>() -> TemplateResult<G> {
    let counter = Signal::new(0);

    create_effect(cloned!((counter) => move || {
        log::info!("Counter value: {}", *counter.get());
    }));

    let increment = cloned!((counter) => move |_| counter.set(*counter.get() + 1));

    let reset = cloned!((counter) => move |_| counter.set(0));

    // template! {
    //     div {
    //         "Counter demo"
    //         p(class="value") {
    //             "Value: "
    //             (counter.get())
    //         }
    //         button(class="increment", on:click=increment) {
    //             "Increment"
    //         }
    //         button(class="reset", on:click=reset) {
    //             "Reset"
    //         }
    //     }
    // }
    ::maple_core::TemplateResult::new({
        let element = ::maple_core::internal::element("div");
        ::maple_core::internal::append_static_text(&element, &"Counter demo");
        ::maple_core::internal::append(&element, &{
            let element = ::maple_core::internal::element("p");
            ::maple_core::internal::attr(
                &element,
                "class",
                ::std::boxed::Box::new(move || {
                    let res = format!("value");
                    res
                }),
            );
            ::maple_core::internal::append_static_text(&element, &"Value: ");
            ::maple_core::internal::append_render(
                &element,
                ::std::boxed::Box::new(move || ::std::boxed::Box::new(counter.get())),
            );
            element
        });
        ::maple_core::internal::append(&element, &{
            let element = ::maple_core::internal::element("button");
            ::maple_core::internal::attr(
                &element,
                "class",
                ::std::boxed::Box::new(move || {
                    let res = format!("increment");
                    res
                }),
            );
            ::maple_core::internal::event(&element, "click", ::std::boxed::Box::new(increment));
            ::maple_core::internal::append_static_text(&element, &"Increment");
            element
        });
        ::maple_core::internal::append(&element, &{
            let element = ::maple_core::internal::element("button");
            ::maple_core::internal::attr(
                &element,
                "class",
                ::std::boxed::Box::new(move || {
                    let res = format!("reset");
                    res
                }),
            );
            ::maple_core::internal::event(&element, "click", ::std::boxed::Box::new(reset));
            ::maple_core::internal::append_static_text(&element, &"Reset");
            element
        });
        element
    })
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
