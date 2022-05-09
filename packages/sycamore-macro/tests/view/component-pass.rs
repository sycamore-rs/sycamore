use sycamore::prelude::*;

#[derive(Prop)]
pub struct Prop {
    prop: &'static str,
}

#[component]
pub fn PropComponent<G: Html>(cx: Scope, Prop { prop: _ }: Prop) -> View<G> {
    view! { cx,
        div
    }
}

#[component]
async fn AsyncPropComponent<G: Html>(cx: Scope<'_>, Prop { prop: _ }: Prop) -> View<G> {
    view! { cx,
        div
    }
}

#[derive(Prop)]
pub struct PropWithChildren<'a, G: GenericNode> {
    children: Children<'a, G>,
}

#[component]
pub fn ComponentWithChildren<'a, G: Html>(cx: Scope<'a>, prop: PropWithChildren<'a, G>) -> View<G> {
    prop.children.call(cx)
}

#[component]
pub fn Component<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div
    }
}

fn compile_pass<G: Html>() {
    create_scope_immediate(|cx| {
        let _: View<G> = view! { cx, Component() };
        let _: View<G> = view! { cx, Component {} };

        let prop = "prop";
        let _: View<G> = view! { cx, PropComponent { prop: prop } };
        let _: View<G> = view! { cx, PropComponent { prop } };

        let _: View<G> = view! { cx,
            ComponentWithChildren {
                Component()
            }
        };

        let _: View<G> = view! { cx,
            ComponentWithChildren {
                Component {}
            }
        };
    });
}

fn main() {}
