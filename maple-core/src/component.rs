//! The definition for the [`Component`] trait.

use crate::generic_node::GenericNode;
use crate::template_result::TemplateResult;

pub trait Component<G: GenericNode> {
    type Props;

    fn create(props: Self::Props) -> TemplateResult<G>;
}
