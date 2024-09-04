//! This module contains the [`View`] struct which represents a view tree.

use std::any::{Any, TypeId};
use std::fmt;

use smallvec::{smallvec, SmallVec};

use crate::*;

/// Represents a view tree.
pub struct View<T> {
    /// The nodes in the view tree.
    pub(crate) nodes: SmallVec<[T; 1]>,
}

impl<T> View<T> {
    /// Create a new blank view.
    pub fn new() -> Self {
        Self {
            nodes: SmallVec::new(),
        }
    }

    /// Create a new view with a single node.
    pub fn from_node(node: T) -> Self {
        Self {
            nodes: smallvec![node],
        }
    }
}

impl<T> Default for View<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Debug for View<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("View").finish()
    }
}

/// We cannot provide a general implementation for `T: ViewNode` since we also define From for
/// tuuples. Hence, only implement it for [`HtmlNode`].
impl From<HtmlNode> for View<HtmlNode> {
    fn from(node: HtmlNode) -> Self {
        View::from_node(node)
    }
}

impl<T> From<Vec<View<T>>> for View<T> {
    fn from(nodes: Vec<View<T>>) -> Self {
        View {
            nodes: nodes.into_iter().flat_map(|v| v.nodes).collect(),
        }
    }
}

impl<T> From<Option<View<T>>> for View<T> {
    fn from(node: Option<View<T>>) -> Self {
        node.unwrap_or_default()
    }
}

macro_rules! impl_view_from {
    ($($ty:ty),*) => {
        $(
            impl<T: ViewHtmlNode> From<$ty> for View<T> {
                fn from(t: $ty) -> Self {
                    View::from_node(T::create_text_node(t.into()))
                }
            }
        )*
    }
}

macro_rules! impl_view_from_to_string {
    ($($ty:ty),*) => {
        $(
            impl<T: ViewHtmlNode> From<$ty> for View<T> {
                fn from(t: $ty) -> Self {
                    View::from_node(T::create_text_node(t.to_string().into()))
                }
            }
        )*
    }
}

impl_view_from!(&'static str, String, Cow<'static, str>);
impl_view_from_to_string!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

impl<T, F: FnMut() -> U + 'static, U: Into<View<T>> + Any + 'static> From<F> for View<T> {
    fn from(mut f: F) -> Self {
        // Specialize for U = String.
        if TypeId::of::<U>() == TypeId::of::<String>() {
            render_dynamic_text(move || {
                (&f() as &dyn Any).downcast_ref::<String>().unwrap().clone()
            })
        } else {
            render_dynamic_view(f)
        }
    }
}
// Implement `From` for all tuples of types that implement `Into<View<U>>`.
macro_rules! impl_from_tuple {
    ($($name:ident),*) => {
        paste::paste! {
            impl<U, $($name),*> From<($($name,)*)> for View<U>
            where
                $($name: Into<View<U>>),*
            {
                fn from(t: ($($name,)*)) -> Self {
                    let ($([<$name:lower>]),*) = t;
                    #[allow(unused_mut)]
                    let mut nodes = SmallVec::new();
                    $(
                        nodes.extend([<$name:lower>].into().nodes);
                    )*
                    View { nodes }
                }
            }
        }
    };
}

impl_from_tuple!();
impl_from_tuple!(A, B);
impl_from_tuple!(A, B, C);
impl_from_tuple!(A, B, C, D);
impl_from_tuple!(A, B, C, D, E);
impl_from_tuple!(A, B, C, D, E, F);
impl_from_tuple!(A, B, C, D, E, F, G);
impl_from_tuple!(A, B, C, D, E, F, G, H);
impl_from_tuple!(A, B, C, D, E, F, G, H, I);
impl_from_tuple!(A, B, C, D, E, F, G, H, I, J);
