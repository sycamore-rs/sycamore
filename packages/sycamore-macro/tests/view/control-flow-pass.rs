use sycamore::prelude::*;

fn compile_pass() {
    let _ = create_root(|| {
        let bool_value = true;
        // Static if/else
        let _: View = view! {
            div {
                if bool_value {
                    p {}
                } else {
                    p {}
                }
            }
        };
        // Static if only
        let _: View = view! {
            div {
                if bool_value {
                    p {}
                }
            }
        };
        // Static if/else/else if
        let _: View = view! {
            div {
                if bool_value {
                    p {}
                } else if false {
                    p {}
                } else {
                    p {}
                }
            }
        };
        // Static match
        let _: View = view! {
            div {
                match 1 {
                    1 => p {},
                    2 => p {},
                    _ => p {},
                }
            }
            div {
                match bool_value {
                    true => p {},
                    false => p {},
                }
            }
        };

        fn random_bool() -> bool {
            // Chosen by fair coin toss, guaranteed to be random.
            true
        }

        fn random_number() -> u32 {
            // Chosen by fair coin toss, guaranteed to be random.
            4
        }

        // Dynamic if/else
        let _: View = view! {
            div {
                if random_bool() {
                    p {}
                } else {
                    p {}
                }
            }
        };
        // Dynamic if only
        let _: View = view! {
            div {
                if random_bool() {
                    p {}
                }
            }
        };
        // Dynamic if/else/else if
        let _: View = view! {
            div {
                if random_bool() {
                    p {}
                } else if random_bool() {
                    p {}
                } else {
                    p {}
                }
            }
        };
        // Dynamic match
        let _: View = view! {
            div {
                match random_number() {
                    1 => p {},
                    2 => p {},
                    _ => p {},
                }
            }
        };

        // Empty if/else
        let _: View = view! {
            div {
                if true {} else {}
            }
        };
        // Empty match arms
        let _: View = view! {
            div {
                match 1 {
                    1 => {},
                    2 => {},
                    _ => {},
                }
            }
        };
    });
}

fn main() {}
