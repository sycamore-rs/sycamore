use std::{borrow::Cow, rc::Rc};

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
pub enum MaybeDyn<T: 'static> {
    /// A static value.
    Static(T),
    /// A dynamic value backed by a signal.
    Signal(ReadSignal<T>),
    /// A derived dynamic value.
    Derived(Rc<dyn Fn() -> Self>),
}

impl<T: 'static> MaybeDyn<T> {
    /// Get the value by consuming itself. Unlike [`get_clone`], this method avoids a clone if we
    /// are just storing a static value.
    pub fn evaluate(self) -> T where T: Clone {
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
    pub fn get(&mut self) -> T
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
    pub fn get_clone(&mut self) -> T
    where
        T: Clone,
    {
        match self {
            Self::Static(value) => value.clone(),
            Self::Signal(value) => value.get_clone(),
            Self::Derived(f) => f().evaluate(),
        }
    }
}

/// A macro that makes it easy to write implementations for `IntoMaybeDyn*` traits.
///
/// This will generate the trait and implement it for `T`, [`ReadSignal<T>`], [`Signal<T>`], and
/// `Fn() -> U` where `U` implements the trait. Furthermore, you can specify additional types that
/// can be converted using their [`From`] implementations.
/// 
/// For more information, check out the docs for [`MaybeDyn`].
///
/// # Example
///
/// ```
/// # use sycamore_reactive::*;
/// struct
/// ```
#[macro_export]
macro_rules! trait_into_maybe_dyn {
    ($vis:vis trait $trait:ident, $ty:ty $(; $($from:ty),*)?) => {
        #[doc = concat!("A trait for converting a [`", stringify!($ty), "`] into a [`MaybeDyn`].")]
        $vis trait $trait {
            #[doc = "Convert the value into a [`MaybeDyn`]."]
            fn into_maybe_dyn(self) -> $crate::MaybeDyn<$ty>;
        }

        impl $trait for $ty {
            fn into_maybe_dyn(self) -> $crate::MaybeDyn<$ty> {
                $crate::MaybeDyn::Static(self)
            }
        }

        impl $trait for $crate::ReadSignal<$ty> {
            fn into_maybe_dyn(self) -> $crate::MaybeDyn<$ty> {
                $crate::MaybeDyn::Signal(self)
            }
        }

        impl $trait for $crate::Signal<$ty> {
            fn into_maybe_dyn(self) -> $crate::MaybeDyn<$ty> {
                $crate::MaybeDyn::Signal(*self)
            }
        }

        impl<F: ::std::ops::Fn() -> U + 'static, U: $trait> $trait for F {
            fn into_maybe_dyn(self) -> $crate::MaybeDyn<$ty> {
                $crate::MaybeDyn::Derived(Rc::new(move || self().into_maybe_dyn()))
            }
        }

        $($(
            impl $trait for $from {
                fn into_maybe_dyn(self) -> $crate::MaybeDyn<$ty> {
                    $crate::MaybeDyn::Static(self.into())
                }
            }
        )*)?
    }
}

trait_into_maybe_dyn!(pub trait IntoMaybeDynBool, bool);

trait_into_maybe_dyn!(pub trait IntoMaybeDynCowStr, Cow<'static, str>; &'static str, String);

trait_into_maybe_dyn!(pub trait IntoMaybeDynF32, f32);
trait_into_maybe_dyn!(pub trait IntoMaybeDynF64, f64);

trait_into_maybe_dyn!(pub trait IntoMaybeDynI8, i8);
trait_into_maybe_dyn!(pub trait IntoMaybeDynI16, i16);
trait_into_maybe_dyn!(pub trait IntoMaybeDynI32, i32);
trait_into_maybe_dyn!(pub trait IntoMaybeDynI64, i64);
trait_into_maybe_dyn!(pub trait IntoMaybeDynI128, i128);
trait_into_maybe_dyn!(pub trait IntoMaybeDynISize, isize);
trait_into_maybe_dyn!(pub trait IntoMaybeDynU8, u8);
trait_into_maybe_dyn!(pub trait IntoMaybeDynU16, u16);
trait_into_maybe_dyn!(pub trait IntoMaybeDynU32, u32);
trait_into_maybe_dyn!(pub trait IntoMaybeDynU64, u64);
trait_into_maybe_dyn!(pub trait IntoMaybeDynU128, u128);
trait_into_maybe_dyn!(pub trait IntoMaybeDynUSize, usize);
