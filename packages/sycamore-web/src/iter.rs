//! Iteration utility components.
//!
//! Iteration can be either _"keyed"_ or _"non keyed"_ by using the [`Keyed`] or [`Indexed`] utility
//! components respectively.

#![allow(non_snake_case)]

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;

use sycamore_macro::{component, Props};
use wasm_bindgen::prelude::*;

use crate::*;

/// Props for [`Keyed`].
#[derive(Props)]
pub struct KeyedProps<T, K, U, List, F, Key>
where
    List: Accessor<Vec<T>> + 'static,
    F: Fn(T) -> U + 'static,
    Key: Fn(&T) -> K + 'static,
{
    list: List,
    view: F,
    key: Key,
    #[prop(default)]
    _phantom: std::marker::PhantomData<(T, K, U)>,
}

/// Keyed iteration.
///
/// Use this instead of directly rendering an array of [`View`]s.
/// Using this will minimize re-renders instead of re-rendering every view node on every
/// state change.
///
/// For non keyed iteration, see [`Indexed`].
///
/// # Example
///
/// ```
/// # use sycamore::prelude::*;
/// #[derive(Clone, PartialEq)]
/// struct AnimalInfo {
///     // The name of the animal.
///     name: &'static str,
///     // An unique id to identify the animal.
///     id: u32,
/// }
///
/// # fn App() -> View {
/// let animals = create_signal(vec![
///     AnimalInfo { name: "Dog", id: 1 },
///     AnimalInfo { name: "Cat", id: 2 },
///     AnimalInfo { name: "Fish", id: 3 },
/// ]);
/// view! {
///     ul {
///         Keyed(
///             list=animals,
///             view=|animal| view! {
///                 li { (animal.name) }
///             },
///             key=|animal| animal.id,
///         )
///     }
/// }
/// # }
/// ```
#[component]
pub fn Keyed<T, K, U, List, F, Key>(props: KeyedProps<T, K, U, List, F, Key>) -> View
where
    T: PartialEq + Clone + 'static,
    K: Hash + Eq + 'static,
    U: Into<View>,
    List: Accessor<Vec<T>> + 'static,
    F: Fn(T) -> U + 'static,
    Key: Fn(&T) -> K + 'static,
{
    let KeyedProps {
        list, view, key, ..
    } = props;

    let start = HtmlNode::create_marker_node();
    let start_node = start.as_web_sys().clone();
    let end = HtmlNode::create_marker_node();
    let end_node = end.as_web_sys().clone();

    //create_effect_initial(move || {
    //    let nodes = map_keyed(list, move |x| view(x).into(), key);
    //    (Box::new(move || {}), (start, end).into())
    //})
    todo!()

    //let initial_view = Rc::new(RefCell::new(Some(vec![])));
    //let nodes = map_keyed(
    //    list,
    //    {
    //        let initial_view = initial_view.clone();
    //        move |x| {
    //            let view = view(x).into();
    //            let node = view.as_web_sys();
    //            if let Some(initial_view) = initial_view.borrow_mut().as_mut() {
    //                initial_view.push(view);
    //            } else {
    //                DomRenderer.render_view_detatched(view);
    //            }
    //            node
    //        }
    //    },
    //    key,
    //);
    //let mut initial_nodes = nodes.with(|x| x.iter().flatten().cloned().collect::<Vec<_>>());
    //let mut prev_nodes: Vec<web_sys::Node> = Vec::new();
    //create_effect(move || {
    //    // Flatten nodes.
    //    let nodes = nodes.with(|x| x.iter().flatten().cloned().collect::<Vec<_>>());
    //
    //    if let Some(end_marker_node) = end_node.get() {
    //        // This will only run if this is the first time we are updating the list.
    //        if prev_nodes.is_empty() {
    //            prev_nodes = initial_nodes
    //                .drain(..)
    //                .map(|x| x.get().unwrap().clone())
    //                .collect();
    //            prev_nodes.push(end_marker_node.clone());
    //        }
    //        let parent = end_marker_node.parent_node().unwrap();
    //        let mut nodes = nodes
    //            .iter()
    //            .map(|x| x.get().unwrap().clone())
    //            .collect::<Vec<_>>();
    //
    //        nodes.push(end_marker_node.clone());
    //
    //        reconcile_fragments(&parent, &mut prev_nodes, &nodes);
    //        prev_nodes = nodes;
    //    }
    //});
}

/// Props for [`Keyed`].
#[derive(Props)]
pub struct IndexedProps<T, U, List, F>
where
    List: Accessor<Vec<T>> + 'static,
    F: Fn(T) -> U + 'static,
{
    list: List,
    view: F,
    #[prop(default)]
    _phantom: std::marker::PhantomData<(T, U)>,
}

/// Non keyed iteration (or keyed by index).
///
/// Use this instead of directly rendering an array of
/// [`View`]s. Using this will minimize re-renders instead of re-rendering every single
/// node on every state change.
///
/// For keyed iteration, see [`Keyed`].
///
/// # Example
/// ```
/// # use sycamore::prelude::*;
/// # fn App() -> View {
/// let fib = create_signal(vec![0, 1, 1, 2, 3, 5, 8]);
/// view! {
///     ul {
///         Indexed(
///             list=fib,
///             view=|x| view! {
///                 li { (x) }
///             },
///         )
///     }
/// }
/// # }
/// ```
#[component]
pub fn Indexed<T, U, List, F>(props: IndexedProps<T, U, List, F>) -> View
where
    T: PartialEq + Clone + 'static,
    U: Into<View>,
    List: Accessor<Vec<T>> + 'static,
    F: Fn(T) -> U + 'static,
{
    let IndexedProps { list, view, .. } = props;

    //let end_marker = HtmlNode::create_marker_node();
    //let end_marker_node = end_marker.as_web_sys().clone();
    //
    //let initial_view = Rc::new(RefCell::new(Some(vec![])));
    //let nodes = map_indexed(list, {
    //    let initial_view = initial_view.clone();
    //    move |x| {
    //        let view = view(x).into();
    //        let node = view.as_web_sys();
    //        if let Some(initial_view) = initial_view.borrow_mut().as_mut() {
    //            initial_view.push(view);
    //        } else {
    //            DomRenderer.render_view_detatched(view);
    //        }
    //        node
    //    }
    //});
    //let mut initial_nodes = nodes.with(|x| x.iter().flatten().cloned().collect::<Vec<_>>());
    //let mut prev_nodes: Vec<web_sys::Node> = Vec::new();
    //create_effect(move || {
    //    // Flatten nodes.
    //    let nodes = nodes.with(|x| x.iter().flatten().cloned().collect::<Vec<_>>());
    //
    //    // This will only run if this is the first time we are updating the list.
    //    if prev_nodes.is_empty() {
    //        prev_nodes = initial_nodes
    //            .drain(..)
    //            .map(|x| x.get().unwrap().clone())
    //            .collect();
    //        prev_nodes.push(end_marker_node.clone());
    //    }
    //    let parent = end_marker_node.parent_node().unwrap();
    //    let mut nodes = nodes
    //        .iter()
    //        .map(|x| x.get().unwrap().clone())
    //        .collect::<Vec<_>>();
    //
    //    nodes.push(end_marker_node.clone());
    //
    //    reconcile_fragments(&parent, &mut prev_nodes, &nodes);
    //    prev_nodes = nodes;
    //});
    //
    //(initial_view.take().unwrap(), end_marker).into()
    todo!()
}

#[wasm_bindgen]
extern "C" {
    /// Extend [`web_sys::Node`] type with an id field. This is used to make `Node` hashable from
    /// Rust.
    #[wasm_bindgen(extends = web_sys::Node)]
    pub(super) type NodeWithId;
    #[wasm_bindgen(method, getter, js_name = "$id")]
    pub fn node_id(this: &NodeWithId) -> Option<usize>;
    #[wasm_bindgen(method, setter, js_name = "$id")]
    pub fn set_node_id(this: &NodeWithId, id: usize);
}

/// A wrapper around [`web_sys::Node`] that implements `Hash` and `Eq`.
struct HashableNode<'a>(&'a NodeWithId, usize);

impl<'a> HashableNode<'a> {
    thread_local! {
        static NEXT_ID: Cell<usize> = const { Cell::new(0) };
    }

    fn new(node: &'a web_sys::Node) -> Self {
        let node = node.unchecked_ref::<NodeWithId>();
        let id = if let Some(id) = node.node_id() {
            id
        } else {
            Self::NEXT_ID.with(|cell| {
                let id = cell.get();
                cell.set(id + 1);
                node.set_node_id(id);
                id
            })
        };
        Self(node, id)
    }
}

impl<'a> PartialEq for HashableNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<'a> Eq for HashableNode<'a> {}

impl<'a> Hash for HashableNode<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

impl Deref for HashableNode<'_> {
    type Target = NodeWithId;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Reconciles an array of nodes.
///
/// # Params
/// * `parent` - The parent node under which all other nodes are (direct) children.
/// * `a` - The current/existing nodes that are to be diffed.
/// * `b` - The new nodes that are to be inserted. After the reconciliation, all the nodes in `b`
///   should be inserted under `parent`.
///
/// # Panics
/// Panics if `a.is_empty()`. Append nodes instead.
fn reconcile_fragments(parent: &web_sys::Node, a: &mut [web_sys::Node], b: &[web_sys::Node]) {
    debug_assert!(!a.is_empty(), "a cannot be empty");

    // Sanity check: make sure all nodes in a are children of parent.
    #[cfg(debug_assertions)]
    {
        for (i, node) in a.iter().enumerate() {
            if node.parent_node().as_ref() != Some(parent) {
                panic!("node {i} in existing nodes Vec is not a child of parent. node = {node:#?}",);
            }
        }
    }

    let b_len = b.len();
    let mut a_end = a.len();
    let mut b_end = b_len;
    let mut a_start = 0;
    let mut b_start = 0;
    let mut map = None::<HashMap<HashableNode, usize>>;

    // Last node in a.
    let after = a[a_end - 1].next_sibling();

    while a_start < a_end || b_start < b_end {
        if a_end == a_start {
            // Append.
            let node = if b_end < b_len {
                if b_start != 0 {
                    b[b_start - 1].next_sibling()
                } else {
                    Some(b[b_end - b_start].clone())
                }
            } else {
                after.clone()
            };

            for new_node in &b[b_start..b_end] {
                parent.insert_before(new_node, node.as_ref()).unwrap();
            }
            b_start = b_end;
        } else if b_end == b_start {
            // Remove.
            for node in &a[a_start..a_end] {
                if map.is_none() || !map.as_ref().unwrap().contains_key(&HashableNode::new(node)) {
                    parent.remove_child(node).unwrap();
                }
            }
            a_start = a_end;
        } else if a[a_start] == b[b_start] {
            // Common prefix.
            a_start += 1;
            b_start += 1;
        } else if a[a_end - 1] == b[b_end - 1] {
            // Common suffix.
            a_end -= 1;
            b_end -= 1;
        } else if a[a_start] == b[b_end - 1] && b[b_start] == a[a_end - 1] {
            // Swap backwards.
            let node = a[a_end - 1].next_sibling();
            parent
                .insert_before(&b[b_start], a[a_start].next_sibling().as_ref())
                .unwrap();
            parent.insert_before(&b[b_end - 1], node.as_ref()).unwrap();
            a_start += 1;
            b_start += 1;
            a_end -= 1;
            b_end -= 1;
            a[a_end] = b[b_end].clone();
        } else {
            // Fallback to map.
            if map.is_none() {
                let tmp = b[b_start..b_end]
                    .iter()
                    .enumerate()
                    .map(|(i, g)| (HashableNode::new(g), i))
                    .collect();
                map = Some(tmp);
            }
            let map = map.as_ref().unwrap();

            if let Some(&index) = map.get(&HashableNode::new(&a[a_start])) {
                if b_start < index && index < b_end {
                    let mut i = a_start;
                    let mut sequence = 1;
                    let mut t;

                    while i + 1 < a_end && i + 1 < b_end {
                        i += 1;
                        t = map.get(&HashableNode::new(&a[i])).copied();
                        if t != Some(index + sequence) {
                            break;
                        }
                        sequence += 1;
                    }

                    if sequence > index - b_start {
                        let node = &a[a_start];
                        while b_start < index {
                            parent.insert_before(&b[b_start], Some(node)).unwrap();
                            b_start += 1;
                        }
                    } else {
                        parent.replace_child(&a[a_start], &b[b_start]).unwrap();
                        a_start += 1;
                        b_start += 1;
                    }
                } else {
                    a_start += 1;
                }
            } else {
                parent.remove_child(&a[a_start]).unwrap();
                a_start += 1;
            }
        }
    }

    // Sanity check: make sure all nodes in b are children of parent after reconciliation.
    #[cfg(debug_assertions)]
    {
        for (i, node) in b.iter().enumerate() {
            if node.parent_node().as_ref() != Some(parent) {
                panic!(
                    "node {i} in new nodes Vec is not a child of parent after reconciliation. node = {node:#?}",
                );
            }
        }
    }
}

/// Get all nodes between `start` and `end`.
///
/// If `end` is before `start`, all nodes after `start` will be returned.
fn get_nodes_between(start: &web_sys::Node, end: &web_sys::Node) -> Vec<web_sys::Node> {
    let parent = start.parent_node().unwrap();
    debug_assert_eq!(
        parent,
        end.parent_node().unwrap(),
        "parents of `start` and `end` do not match"
    );

    let mut nodes = Vec::new();

    let mut next = start.next_sibling();
    while let Some(current) = next {
        let tmp = current.next_sibling();
        if &current == end {
            break;
        } else {
            nodes.push(current);
        }
        next = tmp;
    }

    nodes
}
