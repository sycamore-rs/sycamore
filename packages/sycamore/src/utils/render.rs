//! Utilities for rendering nodes.

use std::rc::Rc;

use ahash::AHashMap;
use wasm_bindgen::UnwrapThrowExt;

use crate::generic_node::GenericNode;
use crate::reactive::create_effect;
use crate::template::{Template, TemplateType};

/// Insert a [`GenericNode`] under `parent` at the specified `marker`. If `initial` is `Some(_)`,
/// `initial` will be replaced with the new inserted node.
///
/// # Params
/// * `parent` - The parent node to insert `accessor` under.
/// * `accessor` - The [`Template`] to be inserted.
/// * `initial` - An optional initial node that is already inserted into the DOM.
/// * `marker` - An optional marker node. If `marker` is `Some(_)`, `accessor` will be inserted
///   directly before `marker`. If `marker` is `None`, `accessor` will be appended at the end of
///   `parent`.
/// * `multi` - A boolean flag indicating whether the node to be inserted is the only child of
///   `parent`. Setting this to `true` will enable certain optimizations when clearing the node.
///   Even if the node to be inserted is the only child of `parent`, `multi` can still be set to
///   `false` but forgoes the optimizations.
pub fn insert<G: GenericNode>(
    parent: &G,
    accessor: Template<G>,
    initial: Option<Template<G>>,
    marker: Option<&G>,
    multi: bool,
) {
    insert_expression(parent, &accessor, initial, marker, false, multi);
}

fn insert_expression<G: GenericNode>(
    parent: &G,
    value: &Template<G>,
    mut current: Option<Template<G>>,
    marker: Option<&G>,
    unwrap_fragment: bool,
    multi: bool,
) {
    while let Some(Template {
        inner: TemplateType::Dyn(f),
    }) = current
    {
        current = Some(f.get().as_ref().clone());
    }

    match &value.inner {
        TemplateType::Node(node) => {
            if let Some(current) = current {
                clean_children(parent, current.flatten(), marker, Some(node), multi);
            } else {
                parent.insert_child_before(node, marker);
            }
        }
        TemplateType::Dyn(f) => {
            let parent = parent.clone();
            let marker = marker.cloned();
            let f = f.clone();
            create_effect(move || {
                let mut value = f.get();
                while let TemplateType::Dyn(f) = &value.inner {
                    value = f.get();
                }
                insert_expression(
                    &parent,
                    &value,
                    current.clone(),
                    marker.as_ref(),
                    false,
                    multi,
                );
                current = Some(value.as_ref().clone());
            });
        }
        TemplateType::Fragment(fragment) => {
            let mut v = Vec::new();
            // normalize_incoming_fragment will subscribe to all dynamic nodes in the function so as
            // to trigger the create_effect when the template changes.
            let dynamic = normalize_incoming_fragment(&mut v, fragment.as_ref(), unwrap_fragment);
            if dynamic {
                let parent = parent.clone();
                let marker = marker.cloned();
                create_effect(move || {
                    let value = Template::new_fragment(v.clone());
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
                    current = Some(Template::new_fragment(
                        value
                            .flatten()
                            .into_iter()
                            .map(Template::new_node)
                            .collect(),
                    )); // TODO: do not perform unnecessary flattening of template
                });
            } else {
                let v = v
                    .into_iter()
                    .map(|x| match x.inner {
                        TemplateType::Node(node) => node,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>();

                if v.is_empty() && current.is_some() && !multi {
                    // Fast path when new array is empty.
                    clean_children(parent, Vec::new(), None, None, false);
                } else {
                    match current {
                        Some(current) => match current.inner {
                            TemplateType::Node(node) => {
                                reconcile_fragments(parent, &mut [node], &v);
                            }
                            TemplateType::Dyn(_) => unreachable!(),
                            TemplateType::Fragment(ref fragment) => {
                                if fragment.is_empty() {
                                    append_nodes(parent, v, marker);
                                } else {
                                    reconcile_fragments(parent, &mut current.flatten(), &v);
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
pub fn clean_children<G: GenericNode>(
    parent: &G,
    current: Vec<G>,
    _marker: Option<&G>,
    replacement: Option<&G>,
    multi: bool,
) {
    if !multi {
        parent.update_inner_text("");
        if let Some(replacement) = replacement {
            parent.append_child(replacement);
        }
        return;
    }

    for node in current {
        if node.parent_node().as_ref() == Some(parent) {
            if let Some(replacement) = replacement {
                parent.replace_child(&node, replacement);
            } else {
                parent.remove_child(&node);
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

/// Normalizes a `Vec<Template<G>>` into a `Vec<G>`.
///
/// Returns whether the normalized `Vec<G>` is dynamic (and should be rendered in an effect).
///
/// # Params
/// * `v` - The [`Vec`] to write the output to.
/// * `fragment` - The `Vec<Template<G>>` to normalize.
/// * `unwrap` - If `true`, unwraps the `fragment` without setting `dynamic` to true. In most cases,
///   this should be `false`.
pub fn normalize_incoming_fragment<G: GenericNode>(
    v: &mut Vec<Template<G>>,
    fragment: &[Template<G>],
    unwrap: bool,
) -> bool {
    let mut dynamic = false;

    for template in fragment {
        match &template.inner {
            TemplateType::Node(_) => v.push(template.clone()),
            TemplateType::Dyn(f) if unwrap => {
                let mut value = f.get().as_ref().clone();
                while let TemplateType::Dyn(f) = &value.inner {
                    value = f.get().as_ref().clone();
                }
                let fragment: Rc<Box<[Template<G>]>> = match &value.inner {
                    TemplateType::Node(_) => Rc::new(Box::new([value])),
                    TemplateType::Fragment(fragment) => Rc::clone(fragment),
                    _ => unreachable!(),
                };
                dynamic =
                    normalize_incoming_fragment(v, fragment.as_ref().as_ref(), false) || dynamic;
            }
            TemplateType::Dyn(_) => {
                // Not unwrap
                v.push(template.clone());
                dynamic = true;
            }
            TemplateType::Fragment(fragment) => {
                dynamic =
                    normalize_incoming_fragment(v, fragment.as_ref().as_ref(), false) || dynamic;
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
                panic!(
                    "node {} in existing nodes Vec is not a child of parent. node = {:#?}",
                    i, node
                );
            }
        }
    }

    let b_len = b.len();
    let mut a_end = a.len();
    let mut b_end = b_len;
    let mut a_start = 0;
    let mut b_start = 0;
    let mut map = None::<AHashMap<G, usize>>;

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

            while b_start < b_end {
                parent.insert_child_before(&b[b_start], node.as_ref());
                b_start += 1;
            }
        } else if b_end == b_start {
            // Remove.
            while a_start < a_end {
                if map.is_none() || !map.as_ref().unwrap_throw().contains_key(&a[a_start]) {
                    parent.remove_child(&a[a_start]);
                }
                a_start += 1;
            }
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
            a_end -= 1;
            b_end -= 1;
            let node = a[a_end].next_sibling();
            parent.insert_child_before(&b[b_start], a[a_start].next_sibling().as_ref());
            a_start += 1;
            b_start += 1;
            parent.insert_child_before(&b[b_end], node.as_ref());

            a[a_end] = b[b_end].clone();
        } else {
            // Fallback to map.
            if map.is_none() {
                map = Some(AHashMap::with_capacity(b_end - b_start));
                for (i, item) in b.iter().enumerate().take(b_end).skip(b_start) {
                    map.as_mut().unwrap_throw().insert(item.clone(), i);
                }
            }
            let map = map.as_ref().unwrap_throw();

            let index = map.get(&a[a_start]);
            if let Some(index) = index {
                if b_start < *index && *index < b_end {
                    let mut i = a_start;
                    let mut sequence = 1;
                    let mut t;

                    while i + 1 < a_end && i + 1 < b_end {
                        i += 1;
                        t = map.get(&a[i]);
                        if t.is_none() || *t.unwrap_throw() != *index + sequence {
                            break;
                        }
                        sequence += 1;
                    }

                    if sequence > *index - b_start {
                        let node = &a[a_start];
                        while b_start < *index {
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
                    "node {} in new nodes Vec is not a child of parent after reconciliation. node = {:#?}",
                    i, node
                );
            }
        }
    }
}
