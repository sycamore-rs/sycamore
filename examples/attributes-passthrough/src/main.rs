use sycamore::prelude::*;
use sycamore::web::StringAttribute;

#[derive(Props)]
pub struct CustomButtonProps {
    #[prop(setter(into))]
    id: StringAttribute,
    #[prop(attributes(html, button))]
    attributes: Attributes,
    children: Children,
}

#[component]
fn CustomButton(props: CustomButtonProps) -> View {
    console_log!("Intercepted `id` attribute: {:?}", props.id.get_clone());

    view! {
        button(id=props.id, ..props.attributes) {
            (props.children)
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
