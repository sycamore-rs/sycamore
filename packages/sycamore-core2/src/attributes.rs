//! General utilities for working with attributes.

use std::marker::PhantomData;

use paste::paste;
use sycamore_reactive::Scope;

use crate::generic_node::GenericNode;

/// An attribute that can be applied to a node.
/// These can be implemented only for a specific type that implements [`GenericNode`], rather than
/// for all types that implement [`GenericNode`].
pub trait ApplyAttr<'a, G: GenericNode, T> {
    fn apply(self, cx: Scope<'a>, el: &G, value: T);
}

/// An attribute that can be applied dynamically to a node.
pub trait ApplyAttrDyn<'a, G: GenericNode, T> {
    fn apply_dyn(self, cx: Scope<'a>, el: &G, value: Box<dyn FnMut() -> T + 'a>);
}

/// A list of attributes.
pub trait AttrList<'a, G: GenericNode, S> {
    /// Apply all the attributes in the list to the element.
    fn apply_all(self, cx: Scope<'a>, el: &G);
}

macro_rules! impl_attr_list_for_tuple {
    ($($name:ident),*) => {
        paste! {
            #[allow(unused_variables, unused_parens, non_snake_case)]
            impl<'a, G: GenericNode, $([<S $name>],)* $($name: AttrItem<'a, G, [<S $name>]>),*> AttrList<'a, G, ($([<S $name>],)*)>
                for ($($name),*)
            {
                fn apply_all(self, cx: Scope<'a>, el: &G) {
                    #[allow(unused_variables)]
                    let ( $([<attr_ $name>]),* ) = self;
                    $( [<attr_ $name>].apply(cx, el); )*
                }
            }
        }
    };
}

impl<'a, G: GenericNode, S, T: AttrItem<'a, G, S>> AttrList<'a, G, (S,)> for (T,) {
    fn apply_all(self, cx: Scope<'a>, el: &G) {
        let (attr,) = self;
        attr.apply(cx, el);
    }
}

impl_attr_list_for_tuple!();
impl_attr_list_for_tuple!(T1, T2);
impl_attr_list_for_tuple!(T1, T2, T3);
impl_attr_list_for_tuple!(T1, T2, T3, T4);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_attr_list_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

/// An item in an [`AttrList`].
pub trait AttrItem<'a, G: GenericNode, S> {
    fn apply(self, cx: Scope<'a>, el: &G);
}

impl<'a, T, G: GenericNode, Attr: ApplyAttr<'a, G, T>> AttrItem<'a, G, ()> for (Attr, T) {
    fn apply(self, cx: Scope<'a>, el: &G) {
        self.0.apply(cx, el, self.1);
    }
}

impl<'a, T, G: GenericNode, Attr: ApplyAttrDyn<'a, G, T>, F: FnMut() -> T + 'a>
    AttrItem<'a, G, ((),)> for (Attr, F)
{
    fn apply(self, cx: Scope<'a>, el: &G) {
        self.0.apply_dyn(cx, el, Box::new(self.1));
    }
}

impl<'a, G: GenericNode, S, T: AttrList<'a, G, S>> AttrItem<'a, G, ((), S)> for T {
    fn apply(self, cx: Scope<'a>, el: &G) {
        self.apply_all(cx, el);
    }
}
