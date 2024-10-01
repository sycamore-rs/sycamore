//! A few handy utilities.

use crate::*;

/// A trait that is implemented for reactive data that can be tracked, such as [`Signal`].
///
/// Also implemented for tuples containing `Trackable`s.
pub trait Trackable {
    /// Track the data reactively.
    fn _track(&self);
}

impl<T> Trackable for Signal<T> {
    fn _track(&self) {
        self.track();
    }
}

impl<T> Trackable for ReadSignal<T> {
    fn _track(&self) {
        self.track();
    }
}

macro_rules! impl_trackable_deps_for_tuple {
    ($($T:tt),*) => {
        paste::paste! {
            impl<$($T,)*> Trackable for ($($T,)*)
            where
                $($T: Trackable,)*
            {
                fn _track(&self) {
                    let ($([<$T:lower>],)*) = self;
                    $(
                        [<$T:lower>]._track();
                    )*
                }
            }
        }
    }
}

impl_trackable_deps_for_tuple!(A);
impl_trackable_deps_for_tuple!(A, B);
impl_trackable_deps_for_tuple!(A, B, C);
impl_trackable_deps_for_tuple!(A, B, C, D);
impl_trackable_deps_for_tuple!(A, B, C, D, E);
impl_trackable_deps_for_tuple!(A, B, C, D, E, F);
impl_trackable_deps_for_tuple!(A, B, C, D, E, F, G);
impl_trackable_deps_for_tuple!(A, B, C, D, E, F, G, H);
impl_trackable_deps_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_trackable_deps_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_trackable_deps_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_trackable_deps_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

/// A helper function for making dependencies explicit.
///
/// # Params
/// * `deps` - A list of signals/memos that are tracked. This can be a single signal or it can be a
///   tuple of signals.
/// * `f` - The callback function.
///
/// # Example
/// ```
/// # use sycamore_reactive::*;
/// # create_root(|| {
/// let state = create_signal(0);
///
/// create_effect(on(state, move || {
///     println!("State changed. New state value = {}", state.get());
/// }));
/// // Prints "State changed. New state value = 0"
///
/// state.set(1);
/// // Prints "State changed. New state value = 1"
/// # });
/// ```
pub fn on<T>(
    deps: impl Trackable + 'static,
    mut f: impl FnMut() -> T + 'static,
) -> impl FnMut() -> T + 'static {
    move || {
        deps._track();
        f()
    }
}
