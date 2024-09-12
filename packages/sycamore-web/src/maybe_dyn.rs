use std::borrow::Cow;

use wasm_bindgen::JsValue;

/// Represents a value that can be either static or dynamic.
///
/// This is useful for cases where you want to accept a value that can be either static or dynamic,
/// such as in component props.
///
/// A [`MaybeDyn`] value can be created from a static value or a closure that returns the value by
/// using the [`From`] trait.
pub enum MaybeDyn<T: Into<Self>> {
    Static(T),
    Dynamic(Box<dyn FnMut() -> T>),
}

impl<T: Into<Self>> MaybeDyn<T> {
    /// Evaluate the value by consuming itself.
    pub fn evaluate(self) -> T {
        match self {
            Self::Static(value) => value,
            Self::Dynamic(mut f) => f(),
        }
    }
}

impl<T: Into<Self>, F: FnMut() -> U + 'static, U: Into<T>> From<F> for MaybeDyn<T> {
    fn from(mut f: F) -> Self {
        Self::Dynamic(Box::new(move || f().into()))
    }
}

macro_rules! impl_from_maybe_dyn {
    ($struct:ty => $($ty:ty),*) => {
        $(
            impl From<$ty> for $struct {
                fn from(value: $ty) -> Self {
                    Self::Static(value.into())
                }
            }
        )*
    };
}

macro_rules! impl_into_self {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for MaybeDyn<$ty> {
                fn from(value: $ty) -> Self {
                    Self::Static(value)
                }
            }
        )*
    };
}

/// A possibly dynamic string value.
pub type MaybeDynString = MaybeDyn<Cow<'static, str>>;
impl_from_maybe_dyn!(MaybeDynString => &'static str, String);

/// A possibly dynamic boolean value.
pub type MaybeDynBool = MaybeDyn<bool>;

/// A possibly dynamic [`JsValue`].
pub type MaybeDynJsValue = MaybeDyn<JsValue>;
impl_from_maybe_dyn!(
    MaybeDynJsValue =>
    String,
    bool,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64
);
impl_into_self!(Cow<'static, str>, bool, JsValue);

impl_into_self!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);
