mod components {
    pub mod name_view;
    pub mod trigram_view;
}
mod trigram_connection;

use components::name_view::NameView;
use components::trigram_view::TrigramView;
use sycamore::prelude::*;
use trigram_connection::TrigramConnection;

pub fn main() {
    // set the console error panic hook
    console_error_panic_hook::set_once();

    sycamore::render(|| {
        let (trigram_connection, connection_status) = TrigramConnection::new("websocket");
        if let Err(error) = connection_status {
            console_log!("{error} (while opening WebSocket connection)");
        }
        provide_context::<TrigramConnection>(trigram_connection);

        view! {
            TrigramView {}
            NameView {}
        }
    });
}
