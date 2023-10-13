//! Utilities for rendering nodes.

use std::rc::Rc;

use hashbrown::HashMap;
use sycamore_reactive::*;

use crate::generic_node::GenericNode;
use crate::view::{View, ViewType};

/// Insert a [`GenericNode`] under `parent` at the specified `marker`. If `initial` is `Some(_)`,
/// `initial` will be replaced with the new inserted node.
///
/// # Params
/// * `parent` - The parent node to insert `accessor` under.
/// * `accessor` - The [`View`] to be inserted.
/// * `initial` - An optional initial node that is already inserted into the DOM.
/// * `marker` - An optional marker node. If `marker` is `Some(_)`, `accessor` will be inserted
///   directly before `marker`. If `marker` is `None`, `accessor` will be appended at the end of
///   `parent`.
/// * `multi` - A boolean flag indicating whether the node to be inserted is the only child of
///   `parent`. Setting this to `false` will enable an optimization when clearing the node. Even if
///   the node to be inserted is the only child of `parent`, `multi` can still be set to `true` but
///   forgoes the optimization.
pub fn insert<G: GenericNode>(
    parent: &G,
    accessor: View<G>,
    initial: Option<View<G>>,
    marker: Option<&G>,
    multi: bool,
) {
    insert_expression(
        parent,
        &accessor,
        initial.map(View::flatten),
        marker,
        false,
        multi,
    );
}

/// Implementaiton detail of [`insert`].
fn insert_expression<G: GenericNode>(
    parent: &G,
    value: &View<G>,
    mut current: Option<Vec<G>>,
    marker: Option<&G>,
    unwrap_fragment: bool,
    multi: bool,
) {
    match &value.inner {
        ViewType::Node(node) => {
            if let Some(current) = current {
                clean_children(parent, current, marker, Some(node), multi);
            } else if marker.is_none() {
                parent.append_child(node);
            } else {
                parent.insert_child_before(node, marker);
            }

            // The new node should be inserted into parent.
            debug_assert_eq!(node.parent_node().as_ref(), Some(parent));
        }
        ViewType::Dyn(f) => {
            let parent = parent.clone();
            let marker = marker.cloned();
            let f = *f;
            create_effect(move || {
                let value = f.get_clone();
                insert_expression(
                    &parent,
                    &value,
                    current.clone(),
                    marker.as_ref(),
                    false,
                    multi,
                );
                current = Some(value.flatten());
            });
        }
        ViewType::Fragment(fragment) => {
            let mut v = Vec::new();
            // normalize_incoming_fragment will subscribe to all dynamic nodes in the function so as
            // to trigger the create_effect when the view changes.
            let dynamic = normalize_incoming_fragment(&mut v, fragment.as_ref(), unwrap_fragment);
            if dynamic {
                let parent = parent.clone();
                let marker = marker.cloned();
                create_effect(move || {
                    let value = View::new_fragment(v.clone());
                    // This will call normalize_incoming_fragment again, but this time with the
                    // unwrap_fragment arg set to true.
                    insert_expression(
                        &parent,
                        &value,
                        current.clone(),
                        marker.as_ref(),
                        true,
                        false,
                    );
                    current = Some(value.flatten()); // TODO: do not perform unnecessary flattenin
                });
            } else {
                let v = v
                    .into_iter()
                    .map(|x| match x.inner {
                        ViewType::Node(node) => node,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>();

                if v.is_empty() && current.is_some() && !multi {
                    // Fast path when new array is empty.
                    clean_children(parent, Vec::new(), None, None, false);
                } else {
                    match current {
                        Some(mut current) => match &mut current[..] {
                            [node] => {
                                reconcile_fragments(parent, std::slice::from_mut(node), &v);
                            }
                            fragment => {
                                if fragment.is_empty() {
                                    append_nodes(parent, v, marker);
                                } else {
                                    reconcile_fragments(parent, fragment, &v);
                                }
                            }
                        },
                        None => append_nodes(parent, v, marker),
                    }
                }
            }
        }
    }
}

/// Cleans the children specified by `current` from `parent`.
///
/// # Params
/// * `parent` - The parent node from which to clean the children.
/// * `current` - A [`Vec`] of [`GenericNode`]s that are to be removed.
/// * `marker` - If `marker` is `None`, all the nodes from `parent` are removed regardless of
///   `current`. This behavior will likely change in the future.
/// * `replacement` - An optional replacement node for the removed nodes.
/// * `multi` - If set to `false`, this will use a more efficient `innerText = ""` to clear instead
///   of removing child nodes one by one.
pub fn clean_children<G: GenericNode>(
    parent: &G,
    current: Vec<G>,
    _marker: Option<&G>,
    replacement: Option<&G>,
    multi: bool,
) {
    if !multi {
        parent.update_inner_text("".into());
        if let Some(replacement) = replacement {
            parent.append_child(replacement);
        }
    } else {
        debug_assert!(!current.is_empty());
        for node in current {
            if node.parent_node().as_ref() == Some(parent) {
                if let Some(replacement) = replacement {
                    parent.replace_child(&node, replacement);
                } else {
                    parent.remove_child(&node);
                }
            } else {
                // debug_assert_eq!(node.parent_node().as_ref(), Some(parent), "{node:?}");
                // FIXME
                if let Some(replacement) = replacement {
                    parent.append_child(replacement);
                }
            }
        }
    }
}

/// Appends all the nodes in `fragment` to `parent` behind `marker`.
pub fn append_nodes<G: GenericNode>(parent: &G, fragment: Vec<G>, marker: Option<&G>) {
    for node in fragment {
        parent.insert_child_before(&node, marker);
    }
}

/// Normalizes a `&[View<G>]` into a `Vec<View<G>>`.
///
/// Returns whether the normalized `Vec<View<G>>` is dynamic (and should therefore be rendered in an
/// effect).
///
/// # Params
/// * `buf` - The `Vec` to write the output to.
/// * `fragment` - The `&[View<G>]` to normalize.
/// * `unwrap` - If `true`, unwraps the `fragment` without setting `dynamic` to true. In most cases,
///   this should be `false`.
pub fn normalize_incoming_fragment<G: GenericNode>(
    buf: &mut Vec<View<G>>,
    fragment: &[View<G>],
    unwrap: bool,
) -> bool {
    let mut dynamic = false;

    for view in fragment {
        match &view.inner {
            ViewType::Node(_) => buf.push(view.clone()),
            ViewType::Dyn(f) => {
                if unwrap {
                    let mut value = f.get_clone();
                    while let ViewType::Dyn(f) = &value.inner {
                        value = f.get_clone();
                    }
                    let fragment: Rc<[View<G>]> = match &value.inner {
                        ViewType::Node(_) => Rc::new([value]),
                        ViewType::Fragment(fragment) => Rc::clone(fragment),
                        _ => unreachable!(),
                    };
                    dynamic = normalize_incoming_fragment(buf, &fragment, false) || dynamic;
                } else {
                    buf.push(view.clone());
                    dynamic = true;
                }
            }
            ViewType::Fragment(fragment) => {
                dynamic = normalize_incoming_fragment(buf, fragment, false) || dynamic;
            }
        }
    }

    dynamic
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
pub fn reconcile_fragments<G: GenericNode>(parent: &G, a: &mut [G], b: &[G]) {
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
    let mut map = None::<HashMap<G, usize>>;

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
                parent.insert_child_before(new_node, node.as_ref());
            }
            b_start = b_end;
        } else if b_end == b_start {
            // Remove.
            for node in &a[a_start..a_end] {
                if map.is_none() || !map.as_ref().unwrap().contains_key(node) {
                    parent.remove_child(node);
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
            parent.insert_child_before(&b[b_start], a[a_start].next_sibling().as_ref());
            parent.insert_child_before(&b[b_end - 1], node.as_ref());
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
                    .map(|(i, g)| (g.clone(), i))
                    .collect();
                map = Some(tmp);
            }
            let map = map.as_ref().unwrap();

            if let Some(&index) = map.get(&a[a_start]) {
                if b_start < index && index < b_end {
                    let mut i = a_start;
                    let mut sequence = 1;
                    let mut t;

                    while i + 1 < a_end && i + 1 < b_end {
                        i += 1;
                        t = map.get(&a[i]).copied();
                        if t != Some(index + sequence) {
                            break;
                        }
                        sequence += 1;
                    }

                    if sequence > index - b_start {
                        let node = &a[a_start];
                        while b_start < index {
                            parent.insert_child_before(&b[b_start], Some(node));
                            b_start += 1;
                        }
                    } else {
                        parent.replace_child(&a[a_start], &b[b_start]);
                        a_start += 1;
                        b_start += 1;
                    }
                } else {
                    a_start += 1;
                }
            } else {
                parent.remove_child(&a[a_start]);
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
