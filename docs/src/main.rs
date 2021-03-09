#![allow(non_snake_case)]

mod header;
mod usage;

use maple_core::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    let root = template! {
        main {
            header::Header()

            usage::Usage()
        }
    };

    render(root);
}
