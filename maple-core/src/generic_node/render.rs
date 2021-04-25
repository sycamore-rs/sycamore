use crate::generic_node::GenericNode;
use crate::prelude::create_effect;
use crate::template_result::{TemplateResult, TemplateResultInner};

pub fn insert<G: GenericNode>(
    parent: G,
    accessor: TemplateResult<G>,
    marker: Option<G>,
    initial: Option<G>,
) {
    match accessor.inner {
        TemplateResultInner::Lazy(f) => {
            let mut current = initial;
            create_effect(move || {
                current = Some(insert_expression(
                    parent.clone(),
                    f.as_ref().unwrap().borrow_mut()(),
                    current.clone(),
                    marker.clone(),
                ));
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
    current: Option<G>,
    marker: Option<G>,
) -> G {
    if matches!(value.inner, TemplateResultInner::Node(node) if Some(&node) == current.as_ref()) {
        return current.unwrap();
    }

    todo!();
}
