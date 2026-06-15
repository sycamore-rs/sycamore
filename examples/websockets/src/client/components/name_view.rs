use sycamore::prelude::*;

use crate::client::TrigramConnection;

#[component]
pub fn NameView() -> View {
    let trigram = use_context::<TrigramConnection>().trigram;

    view! {
        div(class = "name") {
            (trigram.with(
                |tri_opt| tri_opt.map_or(
                    String::new(),
                    |tri| tri.name(),
                )
            ))
        }
    }
}
