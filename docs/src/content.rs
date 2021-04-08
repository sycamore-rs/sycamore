use maple_core::prelude::*;
use pulldown_cmark::{html, Options, Parser};
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

#[wasm_bindgen(inline_js = "export function highlight_all() { hljs.highlightAll(); }")]
extern "C" {
    fn highlight_all();
}

#[component(Content<G>)]
pub fn content() -> TemplateResult<G> {
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
        match reqwest::get(url).await {
            Ok(res) => {
                markdown.set(res.text().await.unwrap());
            }
            Err(err) => {
                log::error!("Unknown error: {}", err);
            }
        }
    }));

    template! {
        div(class="d-flex") {
            crate::sidebar::Sidebar()
            div(ref=docs_container_ref, class="container") { "Loading..." }
        }
    }
}
