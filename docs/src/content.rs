use pulldown_cmark::{html, Event, Options, Parser, Tag};
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

#[derive(Debug)]
struct Outline {
    name: String,
    children: Vec<Outline>,
}

#[component(Content<G>)]
pub fn content(pathname: String) -> Template<G> {
    let location = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .location()
        .unwrap();

    let docs_container_ref = NodeRef::<G>::new();

    let markdown = Signal::new(String::new());
    let content = create_memo(cloned!((markdown) => move || {
        let markdown = markdown.get();

        let mut outline = Vec::new();
        let mut tmp = None;

        let options = Options::all();
        let parser = Parser::new_ext(markdown.as_ref(), options)
            .map(|event| {
                match event {
                    Event::Start(Tag::Heading(_level)) => {
                        tmp = Some(Outline{
                            name:String::new(),
                            children:Vec::new(),
                        });
                    },
                    Event::End(Tag::Heading(level)) => {
                        if level == 1 {} // Do nothing for level 1 heading
                        else if level == 2 {
                            outline.push(tmp.take().unwrap());
                        } else {
                            let l = outline.last_mut().expect("cannot have non level 2 heading at root");
                            l.children.push(tmp.take().unwrap());
                        }
                    },
                    Event::Text(ref text) | Event::Code(ref text) => {
                        if tmp.is_some() {
                            tmp.as_mut().unwrap().name += text;
                        }
                    }
                    _ => {},
                };
                event
            });

        let mut html = String::new();
        html::push_html(&mut html, parser);

        (html, outline)
    }));

    create_effect(cloned!((content, docs_container_ref) => move || {
        if !content.get().0.is_empty() {
            docs_container_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlElement>()
                .set_inner_html(content.get().0.as_ref());
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
        div(class="flex w-full") {
            div(class="flex-none") {
                crate::sidebar::Sidebar()
            }
            div(ref=docs_container_ref, class="content flex-1 min-w-0 pr-4 mb-2") {
                "Loading..."
            }
            div(class="outline flex-none hidden lg:block lg:w-44") {
                (format!("{:#?}", content.get().1))
            }
        }
    }
}
