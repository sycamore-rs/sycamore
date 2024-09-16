mod app;
cfg_ssr_item!(
    mod server;
);

use sycamore::prelude::*;
use sycamore::web::cfg_ssr_item;

#[cfg_ssr]
#[tokio::main]
async fn main() {
    server::start().await;
}

#[cfg_not_ssr]
fn main() {
    console_error_panic_hook::set_once();

    let document = document();
    sycamore::hydrate_to(app::Main, &document);
}
