//! This module contains the [`View`] struct which represents a view tree.

use std::fmt;

/// Represents a view tree.
pub struct View<T> {
    pub(crate) nodes: Vec<T>,
}

impl<T> View<T> {
    /// Create a new blank view.
    pub fn new() -> Self {
        View { nodes: Vec::new() }
    }

    /// Create a new view with a single node.
    pub fn node(node: T) -> Self {
        View { nodes: vec![node] }
    }
}

impl<T> Default for View<T> {
    fn default() -> Self {
        View { nodes: Vec::new() }
    }
}

impl<T> fmt::Debug for View<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("View").finish()
    }
}

impl<T> From<Vec<View<T>>> for View<T> {
    fn from(nodes: Vec<View<T>>) -> Self {
        View {
            nodes: nodes.into_iter().flat_map(|v| v.nodes).collect(),
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
                    let mut nodes = Vec::new();
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
