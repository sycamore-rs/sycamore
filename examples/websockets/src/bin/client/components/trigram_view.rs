use sycamore::prelude::*;

use crate::trigram_connection::TrigramConnection;

#[component]
pub fn TrigramView() -> View {
    view! {
        div(class = "trigram") {
            LineView::<0> {}
            LineView::<1> {}
            LineView::<2> {}
        }
    }
}

#[component]
fn LineView<const N: u8>() -> View {
    const {
        assert!(N < 3);
    }

    let trigram_connection = use_context::<TrigramConnection>();
    let broken = create_selector(move || {
        trigram_connection.trigram.with(
            |tri_opt| tri_opt.map_or(
                "",
                |tri| if tri.broken::<N>() { " broken" } else { " unbroken" },
            )
        )
    });

    view! {
        div(
            class = format!("line{broken}"),
            on:click = move |_| trigram_connection.flip(N),
        )
    }
}
