use maple_core::prelude::*;

fn compile_pass() {
    template! { p };
    template! { custom-element };

    template! { p() };
    template! { custom-element() };

    template! { p(class="my-class") };
    template! { p(class="my-class", id="my-id") };

    template! { button(class="my-btn", on:click=|| {}) };
}

fn main() {}
