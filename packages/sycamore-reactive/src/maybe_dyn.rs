use std::borrow::Cow;
use std::rc::Rc;

use crate::*;

/// Represents a value that can be either static or dynamic.
///
/// This is useful for cases where you want to accept a value that can be either static or dynamic,
/// such as in component props.
///
/// A [`MaybeDyn`] value can be created from a static value or a closure that returns the value by
/// using the [`From`] trait.
///
/// # Creating a `MaybeDyn`
///
/// You can create a `MaybeDyn` from a static value by using the [`MaybeDyn::Static`] variant.
/// However, most of the times, you probably want to use the implementation of the `From<U>` trait
/// for `MaybeDyn<T>`.
///
/// This trait is already implemented globally for signals and closures that return `T`. However,
/// we cannot provide a blanket implementation for all types `T` to convert into `MaybeDyn<T>`
/// because of specialization. Instead, we can only implement it for specific types.
#[derive(Clone)]
pub enum MaybeDyn<T>
where
    T: Into<Self> + 'static,
{
    /// A static value.
    Static(T),
    /// A dynamic value backed by a signal.
    Signal(ReadSignal<T>),
    /// A derived dynamic value.
    Derived(Rc<dyn Fn() -> Self>),
}

impl<T: Into<Self> + 'static> MaybeDyn<T> {
    /// Get the value by consuming itself. Unlike [`get_clone`](Self::get_clone), this method avoids
    /// a clone if we are just storing a static value.
    pub fn evaluate(self) -> T
    where
        T: Clone,
    {
        match self {
            Self::Static(value) => value,
            Self::Signal(signal) => signal.get_clone(),
            Self::Derived(f) => f().evaluate(),
        }
    }

    /// Get the value by copying it.
    ///
    /// If the type does not implement [`Copy`], consider using [`get_clone`](Self::get_clone)
    /// instead.
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        match self {
            Self::Static(value) => *value,
            Self::Signal(value) => value.get(),
            Self::Derived(f) => f().evaluate(),
        }
    }

    /// Get the value by cloning it.
    ///
    /// If the type implements [`Copy`], consider using [`get`](Self::get) instead.
    pub fn get_clone(&self) -> T
    where
        T: Clone,
    {
        match self {
            Self::Static(value) => value.clone(),
            Self::Signal(value) => value.get_clone(),
            Self::Derived(f) => f().evaluate(),
        }
    }

    /// Track the reactive dependencies, if it is dynamic.
    pub fn track(&self) {
        match self {
            Self::Static(_) => {}
            Self::Signal(signal) => signal.track(),
            Self::Derived(f) => f().track(),
        }
    }

    /// Tries to get the value statically or returns `None` if value is dynamic.
    pub fn as_static(&self) -> Option<&T> {
        match self {
            Self::Static(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Into<Self>> From<ReadSignal<T>> for MaybeDyn<T> {
    fn from(val: ReadSignal<T>) -> Self {
        MaybeDyn::Signal(val)
    }
}

impl<T: Into<Self>> From<Signal<T>> for MaybeDyn<T> {
    fn from(val: Signal<T>) -> Self {
        MaybeDyn::Signal(*val)
    }
}

// TODO: add #[diagnostic::do_not_recommend] when it is stablised.
impl<F, U, T: Into<Self>> From<F> for MaybeDyn<T>
where
    F: Fn() -> U + 'static,
    U: Into<MaybeDyn<T>>,
{
    fn from(f: F) -> Self {
        MaybeDyn::Derived(Rc::new(move || f().into()))
    }
}

/// A macro that makes it easy to write implementations for `Into<MaybeDyn<T>>`.
///
/// Because of Rust orphan rules, you can only implement `Into<MaybeDyn<T>>` for types that are
/// defined in the current crate. To work around this limitation, the newtype pattern can be used.
///
/// # Example
///
/// ```
/// # use sycamore_reactive::*;
///
/// struct MyType;
///
/// struct OtherType;
///
/// impl From<OtherType> for MyType {
///     fn from(_: OtherType) -> Self {
///         todo!();
///     }
/// }
///
/// // You can also list additional types that can be converted to `MaybeDyn<MyType>`.
/// impl_into_maybe_dyn!(MyType; OtherType);
/// ```
#[macro_export]
macro_rules! impl_into_maybe_dyn {
    ($ty:ty $(; $($from:ty),*)?) => {
        impl From<$ty> for $crate::MaybeDyn<$ty> {
            fn from(val: $ty) -> Self {
                MaybeDyn::Static(val)
            }
        }

        $crate::impl_into_maybe_dyn_with_convert!($ty; Into::into $(; $($from),*)?);
    };
}

/// Create `From<U>` implementations for `MaybeDyn<T>` for a list of types.
///
/// Usually, you would use the [`impl_into_maybe_dyn!`] macro instead of this macro.
#[macro_export]
macro_rules! impl_into_maybe_dyn_with_convert {
    ($ty:ty; $convert:expr $(; $($from:ty),*)?) => {
        $(
            $(
                impl From<$from> for $crate::MaybeDyn<$ty> {
                    fn from(val: $from) -> Self {
                        MaybeDyn::Static($convert(val))
                    }
                }

                impl From<$crate::ReadSignal<$from>> for $crate::MaybeDyn<$ty> {
                    fn from(val: $crate::ReadSignal<$from>) -> Self {
                        MaybeDyn::Derived(Rc::new(move || MaybeDyn::Static($convert(val.get_clone()))))
                    }
                }

                impl From<$crate::Signal<$from>> for $crate::MaybeDyn<$ty> {
                    fn from(val: $crate::Signal<$from>) -> Self {
                        // Call the implementation for `ReadSignal<$from>`.
                        (*val).into()
                    }
                }
            )*
        )?
    };
}

impl_into_maybe_dyn!(Cow<'static, str>; &'static str, String);
impl_into_maybe_dyn_with_convert!(
    Option<Cow<'static, str>>; |x| Some(Into::into(x));
    Cow<'static, str>, &'static str, String
);
impl_into_maybe_dyn_with_convert!(
    Option<Cow<'static, str>>; |x| Option::map(x, Into::into);
    Option<&'static str>, Option<String>
);

impl_into_maybe_dyn!(bool);

impl_into_maybe_dyn!(f32);
impl_into_maybe_dyn!(f64);

impl_into_maybe_dyn!(i8);
impl_into_maybe_dyn!(i16);
impl_into_maybe_dyn!(i32);
impl_into_maybe_dyn!(i64);
impl_into_maybe_dyn!(i128);
impl_into_maybe_dyn!(isize);
impl_into_maybe_dyn!(u8);
impl_into_maybe_dyn!(u16);
impl_into_maybe_dyn!(u32);
impl_into_maybe_dyn!(u64);
impl_into_maybe_dyn!(u128);
impl_into_maybe_dyn!(usize);

impl<T> From<Option<T>> for MaybeDyn<Option<T>> {
    fn from(val: Option<T>) -> Self {
        MaybeDyn::Static(val)
    }
}

impl<T> From<Vec<T>> for MaybeDyn<Vec<T>> {
    fn from(val: Vec<T>) -> Self {
        MaybeDyn::Static(val)
    }
}

#[cfg(feature = "wasm-bindgen")]
impl_into_maybe_dyn!(
    wasm_bindgen::JsValue;
    &'static str, String, bool, f32, f64, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maybe_dyn_static() {
        let value = MaybeDyn::<i32>::from(123);
        assert!(value.as_static().is_some());
        assert_eq!(value.get(), 123);
        assert_eq!(value.get_clone(), 123);
        assert_eq!(value.evaluate(), 123);
    }

    #[test]
    fn maybe_dyn_signal() {
        let _ = create_root(move || {
            let signal = create_signal(123);
            let value = MaybeDyn::<i32>::from(signal);
            assert!(value.as_static().is_none());
            assert_eq!(value.get(), 123);
            assert_eq!(value.get_clone(), 123);
            assert_eq!(value.evaluate(), 123);
        });
    }

    #[test]
    fn maybe_dyn_derived() {
        let value = MaybeDyn::<i32>::from(|| 123);
        assert!(value.as_static().is_none());
        assert_eq!(value.get(), 123);
        assert_eq!(value.get_clone(), 123);
        assert_eq!(value.evaluate(), 123);
    }
}
