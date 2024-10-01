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
/// You can create a `MaybeDyn` directly by using one of the enum variants. However, most of the
/// time, you probably want to use one of the `IntoMaybeDyn*` traits, e.g. [`IntoMaybeDynBool`].
///
/// The reason why we need different traits for every type is because we don't have specialization
/// in Rust (_yet_). However, we want to implement this trait for both `T` and functions that
/// return `T`. To work around this, we cannot implement this generically but must do so for every
/// type.
///
/// To make it slightly easier to use these traits for arbitrary types, you can use the
/// [`trait_into_maybe_dyn!`] macro to automatically generate the right implementations.
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
    /// Get the value by consuming itself. Unlike [`get_clone`], this method avoids a clone if we
    /// are just storing a static value.
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
/// impl From<i32> for MyType {
///     fn from(_: i32) -> Self {
///         todo!();
///     }
/// }
///
/// // You can also list additional types that can be converted to `MaybeDyn<MyType>`.
/// impl_into_maybe_dyn!(MyType; i32);
/// ```
#[macro_export]
macro_rules! impl_into_maybe_dyn {
    ($ty:ty $(; $($from:ty),*)?) => {
        impl From<$ty> for $crate::MaybeDyn<$ty> {
            fn from(val: $ty) -> Self {
                MaybeDyn::Static(val)
            }
        }

        $(
            $(
                impl From<$from> for $crate::MaybeDyn<$ty> {
                    fn from(val: $from) -> Self {
                        MaybeDyn::Static(val.into())
                    }
                }
            )*
        )?
    };
}

impl_into_maybe_dyn!(bool);
impl_into_maybe_dyn!(Cow<'static, str>; &'static str, String);

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
