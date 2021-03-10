//! Definition of `cloned!` macro. Proc-macros are defined in the separate `maple-core-macro` crate.

/// Utility macro for cloning all the arguments and expanding the expression.
/// 
/// Temporary workaround for [Rust RFC #2407](https://github.com/rust-lang/rfcs/issues/2407).
///
/// # Example
/// ```
/// use maple_core::prelude::*;
/// 
/// let state = Signal::new(0);
///
/// create_effect(cloned!((state) => move || {
///    state.get();
/// }));
///
/// // state still accessible outside of the effect
/// let _ = state.get();
/// ```
#[macro_export]
macro_rules! cloned {
    (($($arg:ident),*) => $e:expr) => {{
        // clone all the args
        $( let $arg = ::std::clone::Clone::clone(&$arg); )*

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
