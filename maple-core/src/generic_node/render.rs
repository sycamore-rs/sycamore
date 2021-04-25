use crate::generic_node::GenericNode;
use crate::template_result::TemplateResult;

pub fn insert<G: GenericNode>(
    parent: &G,
    accessor: TemplateResult<G>,
    marker: Option<G>,
    initial: Option<G>,
) {
}

pub fn insert_expression<G: GenericNode>(parent: G, value: TemplateResult<G>) {}
