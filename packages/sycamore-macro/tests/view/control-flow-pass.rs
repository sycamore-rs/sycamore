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

        // Dynamic if/else: an explicit closure opts into reactivity.
        let _: View = view! {
            div {
                if move || random_bool() {
                    p {}
                } else {
                    p {}
                }
            }
        };
        // Dynamic if only
        let _: View = view! {
            div {
                if move || random_bool() {
                    p {}
                }
            }
        };
        // Dynamic if/else/else if
        let _: View = view! {
            div {
                if move || random_bool() {
                    p {}
                } else if move || random_bool() {
                    p {}
                } else {
                    p {}
                }
            }
        };
        // Dynamic match
        let _: View = view! {
            div {
                match move || random_number() {
                    1 => p {},
                    2 => p {},
                    _ => p {},
                }
            }
        };

        let signal = create_signal(true);
        let read_signal = create_memo(move || signal.get());
        let maybe_static = MaybeDyn::Static(true);
        let maybe_signal = MaybeDyn::from(signal);
        let maybe_derived: MaybeDyn<bool> = MaybeDyn::from(move || signal.get());
        let named_condition = move || signal.get();
        struct CustomCondition(bool);
        impl From<CustomCondition> for MaybeDyn<bool> {
            fn from(value: CustomCondition) -> Self {
                Self::Static(value.0)
            }
        }

        // Ordinary conditions accept the supported Into<MaybeDyn<bool>> forms.
        let _: View = view! {
            div {
                if signal { p {} }
                if read_signal { p {} }
                if maybe_static { p {} }
                if maybe_signal { p {} }
                if maybe_derived { p {} }
                if named_condition { p {} }
                if CustomCondition(true) { p {} }
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

        // If let
        let _: View = view! {
            div {
                if let Some(x) = Some(1) {
                    p { (x.to_string()) }
                } else {
                    p {}
                }
            }
        };
        // Static match with guards
        let _: View = view! {
            div {
                match random_number() {
                    n if n < 4 => p {},
                    _ => p {},
                }
            }
        };
    });
}

fn main() {}
