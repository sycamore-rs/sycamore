#[cfg(feature = "dom")]
pub mod dom_node;
#[cfg(feature = "ssr")]
pub mod ssr_node;

#[cfg(feature = "dom")]
pub use dom_node::*;
#[cfg(feature = "ssr")]
pub use ssr_node::*;

use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use web_sys::Event;

use crate::prelude::*;

pub type EventListener = dyn Fn(Event);

/// Abstraction over a rendering backend (e.g. [`DomNode`] or [`SsrNode`]).
pub trait GenericNode: Debug + Clone + PartialEq + Eq + 'static {
    fn element(tag: &str) -> Self;
    fn text_node(text: &str) -> Self;
    fn fragment() -> Self;
    fn marker() -> Self;
    fn set_attribute(&self, name: &str, value: &str);
    fn append_child(&self, child: &Self);
    fn insert_before_self(&self, new_node: &Self);
    fn insert_child_before(&self, newNode: &Self, referenceNode: Option<&Self>);
    fn remove_child(&self, child: &Self);
    fn replace_child(&self, old: &Self, new: &Self);
    fn insert_sibling_before(&self, child: &Self);
    fn parent_node(&self) -> Option<Self>;
    fn next_sibling(&self) -> Option<Self>;

    /// TODO: remove node on Drop.
    fn remove_self(&self);

    /// Add a [`EventListener`] to the event name.
    fn event(&self, name: &str, handler: Box<EventListener>);
    fn update_text(&self, text: &str);

    /// Append an item that implements [`Render`] and automatically updates the DOM inside an effect.
    fn append_render(&self, child: Box<dyn Fn() -> Box<dyn Render<Self>>>) {
        let parent = self.clone();

        let node = create_effect_initial(cloned!((parent) => move || {
            let node = RefCell::new(child().render());

            let effect = cloned!((node) => move || {
                let new_node = child().update_node(&parent, &node.borrow());
                *node.borrow_mut() = new_node;
            });

            (Rc::new(effect), node)
        }));

        parent.append_child(&node.borrow());
    }
}
