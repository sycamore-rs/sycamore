use sycamore::prelude::*;
use sycamore::web::MaybeDynString;

#[derive(Props)]
pub struct CustomButtonProps {
    #[prop(setter(into))]
    id: MaybeDynString,
    #[prop(attributes(html, input))]
    attributes: Attributes,
    children: Children,
}

#[component]
fn CustomButton(mut props: CustomButtonProps) -> View {
    console_log!("Intercepted id attribute: {}", props.id.get_clone());

    let children = props.children.call();
    view! {
        // TODO: Remove the .get_clone() here.
        button(id=props.id.get_clone(), ..props.attributes) {
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
                    on:click=|_| console_log!("Button 1 clicked!"),
                ) { "Button 1" }
                CustomButton(
                    id="button2",
                    class="red-button",
                    style="background-color:red;",
                    on:click=|_| console_log!("Button 2 clicked!"),
                ) { "Button 2" }
            }
        }
    }
}

fn main() {
    sycamore::render(App);
}
