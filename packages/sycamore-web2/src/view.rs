use crate::*;

/// Represents a view tree.
pub struct View<T = HtmlNode> {
    pub(crate) nodes: Vec<T>,
}

impl<T> Default for View<T> {
    fn default() -> Self {
        View { nodes: Vec::new() }
    }
}

impl<T> View<T> {
    /// Create a new view with a single node.
    pub fn node(node: T) -> Self {
        View { nodes: vec![node] }
    }
}
impl<T> From<Vec<View<T>>> for View<T> {
    fn from(nodes: Vec<View<T>>) -> Self {
        View {
            nodes: nodes.into_iter().flat_map(|v| v.nodes).collect(),
        }
    }
}

// Implement `From` for all tuples of types that implement `Into<View<HtmlNode>>`.
macro_rules! impl_from_tuple {
    ($($name:ident),*) => {
        paste::paste! {
            impl<$($name),*> From<($($name,)*)> for View<HtmlNode>
            where
                $($name: Into<View<HtmlNode>>),*
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
