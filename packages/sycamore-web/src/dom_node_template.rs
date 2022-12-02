use std::cell::RefCell;
use std::rc::Rc;

use hashbrown::HashMap;
use sycamore_core::generic_node::{
    DynMarkerResult, GenericNode, Template, TemplateId, TemplateShape,
};
use sycamore_core::view::View;
use wasm_bindgen::{intern, JsCast, UnwrapThrowExt};
use web_sys::{HtmlTemplateElement, Node};

use crate::{document, Html, VOID_ELEMENTS};

/// A list of steps to perform when instantiating a template.
#[derive(Debug)]
pub struct Walk(pub Vec<WalkSteps>);

/// Instructions for the walker to perform.
#[derive(Debug, Clone, Copy)]
pub enum WalkSteps {
    /// Point to the next sibling.
    NextSibling,
    /// Point to the first child of current node.
    FirstChild,
    /// Point to the parent of the current node.
    /// This will pop the current node off the stack.
    Parent,
    /// Flag the current node.
    Flag,
    /// Mark the current node as the end tag of a dynamic fragment.
    DynMarker { last: bool, multi: bool },
}

#[derive(Clone)]
pub struct CachedTemplateResult {
    pub template_elem: HtmlTemplateElement,
    pub walk: Rc<Walk>,
}

impl CachedTemplateResult {
    pub fn clone_template_content(&self) -> Node {
        self.template_elem
            .content()
            .clone_node_with_deep(true)
            .unwrap_throw()
            .first_child()
            .unwrap_throw()
    }
}

thread_local! {
    static TEMPLATE_CACHE: RefCell<HashMap<TemplateId, CachedTemplateResult>> = RefCell::new(HashMap::new());
}

pub fn try_get_cached_template(id: TemplateId) -> Option<CachedTemplateResult> {
    TEMPLATE_CACHE.with(|cache| cache.borrow().get(&id).cloned())
}

fn insert_cached_template(id: TemplateId, result: CachedTemplateResult) {
    TEMPLATE_CACHE.with(|cache| cache.borrow_mut().insert(id, result));
}

pub struct WalkResult<G: GenericNode> {
    pub flagged_nodes: Vec<G>,
    pub dyn_markers: Vec<DynMarkerResult<G>>,
}

/// Execute the walk sequence.
pub fn execute_walk<G: Html>(walk: &Walk, root: &Node, hydrate_mode: bool) -> WalkResult<G> {
    let mut flagged_nodes = Vec::new();
    let mut dyn_markers = Vec::new();
    let mut stack = Vec::new();
    let mut cur = Some(root.clone());

    for step in &walk.0 {
        match *step {
            WalkSteps::NextSibling => {
                cur = cur.and_then(|node| node.next_sibling());
            }
            WalkSteps::FirstChild => {
                stack.push(cur.clone().unwrap());
                cur = cur.as_ref().unwrap().first_child();
            }
            WalkSteps::Parent => {
                cur = stack.pop();
            }
            WalkSteps::Flag => {
                flagged_nodes.push(G::from_web_sys(cur.clone().unwrap()));
            }
            WalkSteps::DynMarker { last, multi } => {
                if hydrate_mode {
                    if multi {
                        fn is_end_node(node: &Node) -> bool {
                            node.node_type() == Node::COMMENT_NODE
                                && node.node_value().as_deref() == Some("/")
                        }
                        let _start = cur.as_ref().expect("hydration start marker not found");
                        let mut initial = Vec::new();
                        // Find end node.
                        cur = cur.and_then(|node| node.next_sibling());
                        while cur.is_some() && !is_end_node(cur.as_ref().unwrap()) {
                            initial.push(View::new_node(G::from_web_sys(cur.clone().unwrap())));
                            cur = cur.and_then(|node| node.next_sibling());
                        }
                        // We should have reached the end node now.
                        debug_assert!(cur.is_some(), "hydration end marker not found");
                        dyn_markers.push(DynMarkerResult {
                            parent: G::from_web_sys(stack.last().unwrap().clone()),
                            before: cur.clone().map(G::from_web_sys),
                            initial: Some(View::new_fragment(initial)),
                            multi,
                        });
                    } else {
                        let mut initial = Vec::new();
                        while let Some(next) = cur {
                            initial.push(View::new_node(G::from_web_sys(next.clone())));
                            cur = next.next_sibling();
                        }
                        dyn_markers.push(DynMarkerResult {
                            parent: G::from_web_sys(stack.last().unwrap().clone()),
                            before: cur.clone().map(G::from_web_sys),
                            initial: Some(View::new_fragment(initial)),
                            multi,
                        });
                    }
                } else {
                    dyn_markers.push(DynMarkerResult {
                        parent: G::from_web_sys(stack.last().unwrap().clone()),
                        before: if last {
                            None
                        } else {
                            cur.clone().map(G::from_web_sys)
                        },
                        initial: None,
                        multi,
                    });
                }
            }
        }
    }

    WalkResult {
        flagged_nodes,
        dyn_markers,
    }
}

pub fn render_template_to_string(
    template: &TemplateShape,
    buf: &mut String,
    walk: &mut Vec<WalkSteps>,
    multi: bool,
    last: bool,
) {
    match template {
        TemplateShape::Element {
            tag: ident,
            ns: _,
            children,
            attributes,
            flag,
        } => {
            if *flag {
                walk.push(WalkSteps::Flag);
            }

            buf.push('<');
            buf.push_str(ident);
            for (name, value) in *attributes {
                buf.push(' ');
                buf.push_str(name);
                buf.push_str("=\"");
                html_escape::encode_double_quoted_attribute_to_string(value, buf);
                buf.push('"');
            }

            // Check if self-closing tag (void-element).
            if children.is_empty() && VOID_ELEMENTS.contains(ident) {
                buf.push_str("/>");
            } else {
                walk.push(WalkSteps::FirstChild);

                buf.push('>');
                let multi = children.len() != 1;
                for i in 0..children.len() {
                    render_template_to_string(
                        &children[i],
                        buf,
                        walk,
                        multi,
                        i == children.len() - 1,
                    );
                    if i != children.len() - 1 {
                        walk.push(WalkSteps::NextSibling);
                    }
                }
                buf.push_str("</");
                buf.push_str(ident);
                buf.push('>');

                walk.push(WalkSteps::Parent);
            }
        }
        TemplateShape::Text(text) => {
            html_escape::encode_text_minimal_to_string(text, buf);
        }
        TemplateShape::DynMarker => {
            if last {
                walk.push(WalkSteps::DynMarker { last, multi })
            } else {
                walk.push(WalkSteps::DynMarker { last, multi });
                buf.push_str("<!->")
            }
        }
    }
}

pub fn add_new_cached_template(template: &Template) -> CachedTemplateResult {
    // No cached template found. Create a new cached template and use it.
    let mut buf = String::new();
    let mut walk = Vec::new();

    render_template_to_string(&template.shape, &mut buf, &mut walk, false, false);
    intern(&buf);

    let template_elem = document().create_element("template").unwrap_throw();
    template_elem.set_inner_html(&buf);

    let result = CachedTemplateResult {
        template_elem: template_elem.unchecked_into(),
        walk: Rc::new(Walk(walk)),
    };
    insert_cached_template(template.id, result.clone());
    result
}
