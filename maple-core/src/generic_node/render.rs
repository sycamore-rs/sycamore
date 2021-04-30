use crate::generic_node::GenericNode;
use crate::prelude::create_effect;
use crate::template_result::{TemplateResult, TemplateResultInner};

pub fn insert<G: GenericNode>(
    parent: G,
    accessor: TemplateResult<G>,
    _initial: Option<TemplateResult<G>>,
    marker: Option<G>,
) {
    match accessor.inner {
        TemplateResultInner::Lazy(f) => {
            create_effect(move || {
                insert_expression(
                    parent.clone(),
                    f.as_ref().unwrap().borrow_mut()(),
                    marker.clone(),
                );
            });
        }
        _ => {
            insert_expression(parent, accessor, marker);
        }
    }
}

pub fn insert_expression<G: GenericNode>(parent: G, value: TemplateResult<G>, marker: Option<G>) {
    match value.inner {
        TemplateResultInner::Node(node) => {
            parent.append_child(&node);
        }
        TemplateResultInner::Lazy(f) => {
            let mut v = f.unwrap().as_ref().borrow_mut()();
            while let TemplateResultInner::Lazy(f) = v.inner {
                v = f.unwrap().as_ref().borrow_mut()();
            }

            insert_expression(parent, v, marker);
        }
        TemplateResultInner::Fragment(fragment) => {
            clear_children(parent.clone(), vec![], None, None);

            for template in fragment {
                insert_expression(parent.clone(), template, None);
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
