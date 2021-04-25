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

            return Some(TemplateResult::new_node(node));
        }
        TemplateResultInner::Lazy(f) => {
            let mut v = f.unwrap().as_ref().borrow_mut()();
            while let TemplateResultInner::Lazy(f) = v.inner {
                v = f.unwrap().as_ref().borrow_mut()();
            }

            current = insert_expression(parent, v, current, marker);
            return Some(TemplateResult::new_lazy(move || current.clone().unwrap()));
        }
        TemplateResultInner::Fragment(_) => {}
    }
    todo!();
}
