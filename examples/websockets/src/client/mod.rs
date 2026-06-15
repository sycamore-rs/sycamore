mod components {
    pub mod trigram_view;
    pub mod name_view;
}
mod trigram_connection;

use sycamore::prelude::*;

use components::{name_view::NameView, trigram_view::TrigramView};
use trigram_connection::TrigramConnection;

// the entry point for the client part of the example
pub fn main() {
    // set the console error panic hook
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    sycamore::render(|| {
        let (trigram_connection, connection_status) = TrigramConnection::new(
            "websocket"
        );
        if let Err(error) = connection_status {
            console_log!(
                "{error} (while opening WebSocket connection)"
            );
        }
        provide_context::<TrigramConnection>(trigram_connection);

        view! {
            TrigramView {}
            NameView {}
        }
    });
}