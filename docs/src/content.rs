use pulldown_cmark::{html, Options, Parser};
use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

#[wasm_bindgen(inline_js = "\
export function highlight_all() { hljs.highlightAll(); }\
export async function fetch_md(url) { return await (await fetch(url)).text(); }")]
extern "C" {
    fn highlight_all();
    async fn fetch_md(url: &str) -> JsValue;
}

#[component(Content<G>)]
pub fn content() -> Template<G> {
    let location = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .location()
        .unwrap();
    let pathname = location.pathname().unwrap();

    let docs_container_ref = NodeRef::<G>::new();

    let markdown = Signal::new(String::new());
    let html = create_memo(cloned!((markdown) => move || {
        let markdown = markdown.get();

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(markdown.as_ref(), options);

        let mut output = String::new();
        html::push_html(&mut output, parser);

        output
    }));

    create_effect(cloned!((docs_container_ref) => move || {
        if !html.get().is_empty() {
            docs_container_ref.get::<DomNode>().unchecked_into::<HtmlElement>().set_inner_html(html.get().as_ref());
            highlight_all();
        }
    }));

    wasm_bindgen_futures::spawn_local(cloned!((markdown) => async move {
        log::info!("Getting documentation at {}", pathname);

        let url = format!("{}/markdown{}.md", location.origin().unwrap(), pathname);
        let text = fetch_md(&url).await.as_string().unwrap();
        markdown.set(text);
    }));

    template! {
        div(class="content flex w-full") {
            div(class="flex-none") {
                crate::sidebar::Sidebar()
            }
            div(ref=docs_container_ref, class="flex-1 min-w-0 pr-4") {
                "Loading..."
            }
        }
    }
}
