//! The definition for the [`Component`] trait.

use crate::generic_node::GenericNode;

pub trait Component<G: GenericNode> {
    /// The name of the component (for use in debug mode).
    const NAME: &'static str = "UnnamedComponent";

    // fn create(props: Self::Props) -> TemplateResult<G>;
}
