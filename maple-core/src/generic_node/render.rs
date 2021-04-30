use crate::generic_node::GenericNode;
use crate::prelude::create_effect;
use crate::template_result::{TemplateResult, TemplateResultInner};

pub fn insert<G: GenericNode>(
    parent: G,
    accessor: TemplateResult<G>,
    initial: Option<TemplateResult<G>>,
    marker: Option<G>,
) {
    match accessor.inner {
        TemplateResultInner::Lazy(f) => {
            let mut current = initial;
            create_effect(move || {
                current = insert_expression(
                    parent.clone(),
                    f.as_ref().unwrap().borrow_mut()(),
                    current.clone(),
                    marker.clone(),
                );
            });
        }
        _ => {
            insert_expression(parent, accessor, initial, marker);
        }
    }
}

pub fn insert_expression<G: GenericNode>(
    parent: G,
    value: TemplateResult<G>,
    mut current: Option<TemplateResult<G>>,
    marker: Option<G>,
) -> Option<TemplateResult<G>> {
    match value.inner {
        TemplateResultInner::Node(node) => {
            parent.append_child(&node);

            Some(TemplateResult::new_node(node))
        }
        TemplateResultInner::Lazy(f) => {
            let mut v = f.unwrap().as_ref().borrow_mut()();
            while let TemplateResultInner::Lazy(f) = v.inner {
                v = f.unwrap().as_ref().borrow_mut()();
            }

            current = insert_expression(parent, v, current, marker);
            Some(TemplateResult::new_lazy(move || current.clone().unwrap()))
        }
        TemplateResultInner::Fragment(fragment) => {
            clear_children(parent.clone(), vec![], None, None);

            for template in fragment {
                insert_expression(parent.clone(), template, None, None);
            }

            Some(TemplateResult::new_lazy(move || current.clone().unwrap()))
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
