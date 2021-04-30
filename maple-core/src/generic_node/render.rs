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
                );
                current = Some(value);
            });
        }
        _ => {
            insert_expression(parent, accessor, current, marker);
        }
    }
}

pub fn insert_expression<G: GenericNode>(
    parent: G,
    value: TemplateResult<G>,
    mut current: Option<TemplateResult<G>>,
    marker: Option<G>,
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
                let value = f.as_ref().unwrap().borrow_mut()();
                insert_expression(
                    parent.clone(),
                    value.clone(),
                    current.clone(),
                    marker.clone(),
                );
                current = Some(value);
            });
        }
        TemplateResultInner::Fragment(fragment) => {
            clear_children(parent.clone(), vec![], None, None);

            for template in fragment {
                insert_expression(
                    parent.clone(),
                    template,
                    current.clone(), /* TODO */
                    None,
                );
            }
        }
    }
}

pub fn clear_children<G: GenericNode>(
    parent: G,
    current: Vec<G>,
    marker: Option<G>,
    replacement: Option<G>,
) {
    if marker == None {
        parent.update_inner_text("");
        return;
    }

    let replacement = replacement.unwrap_or_else(|| G::text_node(""));

    for node in current {
        if node.parent_node().as_ref() == Some(&parent) {
            parent.replace_child(&node, &replacement);
        }
    }
}
