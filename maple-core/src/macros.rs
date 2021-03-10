//! Definition of `cloned!` macro. Proc-macros are defined in the separate `maple-core-macro` crate.

#[macro_export]
macro_rules! cloned {
    (($($arg:ident),*) => $e:expr) => {{
        // clone all the args
        $( let $arg = ::std::clone::Clone::clone(&$arg); )*

        // return the closure using the cloned Rc
        $e
    }};
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn cloned() {
        let state = Signal::new(0);

        let _x = cloned!((state) => state);

        // state still accessible because it was cloned instead of moved
        let _ = state.get();
    }

    #[test]
    fn cloned_closure() {
        let state = Signal::new(0);

        create_effect(cloned!((state) => move || {
            state.get();
        }));

        // state still accessible outside of the effect
        let _ = state.get();
    }
}
