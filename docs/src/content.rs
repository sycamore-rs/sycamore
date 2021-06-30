use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outline {
    name: String,
    children: Vec<Outline>,
}

#[component(OutlineView<G>)]
pub fn outline_view(outline: StateHandle<Vec<Outline>>) -> Template<G> {
    template! {
        ul(class="mt-4 text-sm pl-2 border-l border-gray-400") {
            Indexed(IndexedProps {
                iterable: outline,
                template: |item| {
                    let Outline { name, children } = item;
                    let nested = children.iter().map(|x| {
                        let name = x.name.clone();
                        let href = format!("#{}", x.name.trim().to_lowercase().replace(" ", "-"));
                        template! {
                            li {
                                a(href=href) {
                                    (name)
                                }
                            }
                        }
                    }).collect();
                    let nested = Template::new_fragment(nested);

                    let href = format!("#{}", name.trim().to_lowercase().replace(" ", "-"));

                    template! {
                        li {
                            a(href=href) {
                                (name)
                            }
                            ul(class="ml-3") {
                                (nested)
                            }
                        }
                    }
                }
            })
        }
    }
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
    let outline = Signal::new(Vec::new());
    let html = create_memo(cloned!((markdown, outline) => move || {
        let markdown = markdown.get();

        let mut outline_tmp = Vec::new();
        let mut tmp = None;

        let options = Options::all();
        let parser = Parser::new_ext(markdown.as_ref(), options)
            .filter_map(|event| {
                match event {
                    Event::Start(Tag::Heading(level)) => {
                        if level == 1 {
                            Some(event)
                        } else {
                            tmp = Some(Outline{
                                name: String::new(),
                                children: Vec::new(),
                            });
                            None
                        }
                    },
                    Event::End(Tag::Heading(level)) => {
                        if level == 1 {
                            Some(event)
                        } else {
                            let tmp = tmp.take().unwrap();
                            let anchor = tmp.name.trim().to_lowercase().replace(" ", "-");
                            let name = tmp.name.clone();
                            if level == 2 {
                                outline_tmp.push(tmp);
                            } else {
                                let l = outline_tmp.last_mut().expect("cannot have non level 2 heading at root");
                                l.children.push(tmp);
                            }
                            Some(Event::Html(CowStr::from(format!("<h{} id=\"{}\">{}</h{}>", level, anchor, name, level))))
                        }
                    },
                    Event::Text(ref text) | Event::Code(ref text) => {
                        if tmp.is_some() {
                            tmp.as_mut().unwrap().name += text;
                            // Some(event)
                            None
                        } else {
                            Some(event)
                        }
                    }
                    _ => Some(event),
                }
            });

        let mut html = String::new();
        html::push_html(&mut html, parser);

        outline.set(outline_tmp);

        html
    }));

    create_effect(cloned!((html, docs_container_ref) => move || {
        if !html.get().is_empty() {
            docs_container_ref
                .get::<DomNode>()
                .unchecked_into::<HtmlElement>()
                .set_inner_html(html.get().as_ref());
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
            div(ref=docs_container_ref, class="content flex-1 min-w-0 pr-4 mb-2 lg:mr-44") {
                "Loading..."
            }
            div(class="outline flex-none hidden lg:block lg:w-44 fixed right-0") {
                OutlineView(outline.handle())
            }
        }
    }
}
