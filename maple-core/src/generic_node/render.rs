use std::collections::HashMap;

use crate::generic_node::GenericNode;
use crate::prelude::create_effect;
use crate::template_result::{TemplateResult, TemplateResultInner};

pub fn insert<G: GenericNode>(
    parent: G,
    accessor: TemplateResult<G>,
    initial: Option<TemplateResult<G>>,
    marker: Option<G>,
) {
    let mut current = initial;

    match accessor.inner {
        TemplateResultInner::Lazy(f) => {
            create_effect(move || {
                let value = f.as_ref().unwrap().borrow_mut()();
                insert_expression(
                    parent.clone(),
                    value.clone(),
                    current.clone(),
                    marker.clone(),
                    false,
                );
                current = Some(value);
            });
        }
        _ => {
            insert_expression(parent, accessor, current, marker, false);
        }
    }
}

pub fn insert_expression<G: GenericNode>(
    parent: G,
    value: TemplateResult<G>,
    mut current: Option<TemplateResult<G>>,
    marker: Option<G>,
    unwrap_fragment: bool,
) {
    match value.inner {
        TemplateResultInner::Node(node) => {
            if let Some(current) = current {
                clear_children(parent, current.flatten(), None, Some(node));
            } else {
                parent.append_child(&node);
            }
        }
        TemplateResultInner::Lazy(f) => {
            create_effect(move || {
                let mut value = f.as_ref().unwrap().borrow_mut()();
                while let TemplateResultInner::Lazy(f) = value.inner {
                    value = f.as_ref().unwrap().borrow_mut()();
                }
                insert_expression(
                    parent.clone(),
                    value.clone(),
                    current.clone(),
                    marker.clone(),
                    false,
                );
                current = Some(value);
            });
        }
        TemplateResultInner::Fragment(fragment) => {
            let mut v = Vec::new();
            if normalize_incoming_fragment(&mut v, fragment, unwrap_fragment) {
                create_effect(move || {
                    let parent = parent.clone();
                    let current = current.clone();
                    let marker = marker.clone();
                    let v = v.clone();
                    insert_expression(
                        parent,
                        TemplateResult::new_fragment(
                            v.into_iter().map(TemplateResult::new_node).collect(),
                        ),
                        current,
                        marker,
                        true,
                    );
                });
            } else {
                reconcile_fragments(parent, current.map(|x| x.flatten()).unwrap_or_default(), v);
            }

            // current = Some(TemplateResult::new_fragment(
            //     new_fragment
            //         .into_iter()
            //         .map(TemplateResult::new_node)
            //         .collect(),
            // ));
        }
    }
}

pub fn clear_children<G: GenericNode>(
    parent: G,
    current: Vec<G>,
    marker: Option<G>,
    replacement: Option<G>,
) {
    let replacement = replacement.unwrap_or_else(|| G::text_node(""));

    if marker == None {
        parent.update_inner_text("");
        parent.append_child(&replacement);
        return;
    }

    for node in current {
        if node.parent_node().as_ref() == Some(&parent) {
            parent.replace_child(&node, &replacement);
        }
    }
}

/// Normalizes a `Vec<TemplateResult<G>>` into a `Vec<G>`.
///
/// Returns whether the normalized `Vec<G>` is dynamic (and should be rendered in an effect).
///
/// # Params
/// * `v` - The [`Vec`] to write the output to.
/// * `fragment` - The `Vec<TemplateResult<G>>` to normalize.
/// * `unwrap` - If `true`, unwraps the `fragment` without setting `dynamic` to true. In most cases,
///   this should be `false.
pub fn normalize_incoming_fragment<G: GenericNode>(
    v: &mut Vec<G>,
    fragment: Vec<TemplateResult<G>>,
    unwrap: bool,
) -> bool {
    let mut dynamic = false;

    web_sys::console::log_1(&format!("{:#?}", fragment).into());
    
    for template in fragment {
        match template.inner {
            TemplateResultInner::Node(node) => v.push(node),
            TemplateResultInner::Lazy(f) => {
                web_sys::console::log_1(&"lazy in fragment".into());
                if unwrap {
                    let mut value = f.as_ref().unwrap().borrow_mut()();
                    while let TemplateResultInner::Lazy(f) = value.inner {
                        value = f.as_ref().unwrap().borrow_mut()();
                    }
                    dynamic = normalize_incoming_fragment(
                        v,
                        match value.inner {
                            TemplateResultInner::Node(_) => vec![value],
                            TemplateResultInner::Fragment(fragment) => fragment,
                            _ => unreachable!(),
                        },
                        false,
                    ) || dynamic;
                } else {
                    dynamic = true;
                }
            }
            TemplateResultInner::Fragment(fragment) => {
                dynamic = normalize_incoming_fragment(v, fragment, false) || dynamic;
            }
        }
    }

    dynamic
}

pub fn reconcile_fragments<G: GenericNode>(parent: G, mut a: Vec<G>, b: Vec<G>) {
    let b_len = b.len();
    let mut a_end = a.len();
    let mut b_end = b_len;
    let mut a_start = 0;
    let mut b_start = 0;
    let mut map = None::<HashMap<G, usize>>;

    if a.is_empty() {
        if !b.is_empty() {
            for node in &b {
                parent.append_child(node);
            }
        }
        return;
    }

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
                if map.is_none() || map.as_ref().unwrap().contains_key(&a[a_start]) {
                    parent.remove_child(&a[a_start]);
                    a_start += 1;
                }
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
                map = Some(HashMap::new());
                let mut i = b_start;
                while i < b_end {
                    map.as_mut().unwrap().insert(b[i].clone(), i);
                    i += 1;
                }
            }

            let index = map.as_ref().unwrap().get(&a[a_start]);
            if let Some(index) = index {
                if b_start < *index && *index < b_end {
                    let mut i = a_start;
                    let mut sequence = 1;
                    let mut t;

                    while i + 1 < a_end && i < b_end {
                        i += 1;
                        t = map.as_ref().unwrap().get(&a[i]);
                        if t.is_none() || *t.unwrap() != *index + sequence {
                            sequence += 1;
                        }
                    }

                    if sequence > *index - b_start {
                        let node = &a[a_start];
                        while b_start < *index {
                            parent.insert_child_before(&b[b_start + 1], Some(node));
                            b_start += 1;
                        }
                    } else {
                        parent.replace_child(&b[b_start], &a[a_start]);
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
}
