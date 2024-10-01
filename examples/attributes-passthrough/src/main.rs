use std::borrow::Cow;

use sycamore::prelude::*;

#[derive(Props)]
pub struct CustomButtonProps {
    // TODO: remove this monstrosity.
    #[prop(setter(into))]
    id: MaybeDyn<Cow<'static, str>>,
    #[prop(attributes(html, button))]
    attributes: Attributes,
    children: Children,
}

#[component]
fn CustomButton(props: CustomButtonProps) -> View {
    console_log!("Intercepted `id` attribute: {}", props.id.get_clone());

    let children = props.children.call();
    view! {
        // TODO: Remove the .clone() here.
        button(id=props.id.clone(), ..props.attributes) {
            (children)
        }
    }
}

#[component]
fn App() -> View {
    view! {
        div {
            p { "Passthrough attributes demo" }

            div {
                CustomButton(
                    id="button1",
                    r#type="button",
                    on:click=|_| console_log!("Button 1 clicked!"),
                ) { "Button 1" }
                CustomButton(
                    id="button2",
                    r#type="button",
                    class="red-button",
                    style="background-color:red;",
                    prop:disabled=true,
                    on:click=|_| console_log!("Button 2 clicked!"),
                ) { "Button 2" }
            }
        }
    }
}

fn main() {
    sycamore::render(App);
}
