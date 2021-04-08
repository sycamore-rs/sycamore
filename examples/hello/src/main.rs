use maple_core::prelude::*;

#[component(App<G>)]
fn app() -> TemplateResult<G> {
    let name = Signal::new(String::new());
    let name2 = name.clone();
    let checked = Signal::new(false);
    let checked2 = checked.clone();

    template! {
        div {
            h1 {
                "Hello "
                ({if *create_selector(cloned!((name) => move || !name.get().is_empty())).get() {
                    cloned!((name) => template! {
                        span { (name.get()) }
                    })
                } else {
                    template! { span { "World" } }
                }})
                "!"
            }

            input(bind:value=name2)
            input(type="checkbox", bind:checked=checked)
            (checked2.get())
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    render(|| template! { App() });
}
