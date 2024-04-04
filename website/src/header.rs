use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::sidebar::{SidebarCurrent, SidebarData};
use crate::DarkMode;

#[component]
fn DarkModeToggle() -> View {
    static LIGHT_BULB_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" data-prefix="fas" data-icon="lightbulb" class="svg-inline--fa fa-lightbulb fa-w-11" role="img" viewBox="0 0 352 512"><path fill="currentColor" d="M96.06 454.35c.01 6.29 1.87 12.45 5.36 17.69l17.09 25.69a31.99 31.99 0 0 0 26.64 14.28h61.71a31.99 31.99 0 0 0 26.64-14.28l17.09-25.69a31.989 31.989 0 0 0 5.36-17.69l.04-38.35H96.01l.05 38.35zM0 176c0 44.37 16.45 84.85 43.56 115.78 16.52 18.85 42.36 58.23 52.21 91.45.04.26.07.52.11.78h160.24c.04-.26.07-.51.11-.78 9.85-33.22 35.69-72.6 52.21-91.45C335.55 260.85 352 220.37 352 176 352 78.61 272.91-.3 175.45 0 73.44.31 0 82.97 0 176zm176-80c-44.11 0-80 35.89-80 80 0 8.84-7.16 16-16 16s-16-7.16-16-16c0-61.76 50.24-112 112-112 8.84 0 16 7.16 16 16s-7.16 16-16 16z"/></svg>"#;
    static CLOUD_MOON_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" data-prefix="far" data-icon="moon" class="svg-inline--fa fa-moon fa-w-16" role="img" viewBox="0 0 512 512"><path fill="currentColor" d="M279.135 512c78.756 0 150.982-35.804 198.844-94.775 28.27-34.831-2.558-85.722-46.249-77.401-82.348 15.683-158.272-47.268-158.272-130.792 0-48.424 26.06-92.292 67.434-115.836 38.745-22.05 28.999-80.788-15.022-88.919A257.936 257.936 0 0 0 279.135 0c-141.36 0-256 114.575-256 256 0 141.36 114.576 256 256 256zm0-464c12.985 0 25.689 1.201 38.016 3.478-54.76 31.163-91.693 90.042-91.693 157.554 0 113.848 103.641 199.2 215.252 177.944C402.574 433.964 344.366 464 279.135 464c-114.875 0-208-93.125-208-208s93.125-208 208-208z"/></svg>"#;

    let DarkMode(dark_mode) = use_context::<DarkMode>();
    let toggle = move |_| dark_mode.set(!dark_mode.get());

    // Update color-scheme when `dark_mode` changes.
    create_effect(move || {
        let document_element = document()
            .document_element()
            .unwrap()
            .unchecked_into::<HtmlElement>();
        document_element
            .style()
            .set_property("overflow", "hidden")
            .unwrap();
        document().body().unwrap().client_width(); // Trigger reflow.
        document_element
            .set_attribute(
                "data-color-scheme",
                if dark_mode.get() { "dark" } else { "light" },
            )
            .unwrap();
        document_element
            .style()
            .set_property("overflow", "")
            .unwrap();
    });

    view! {
        (if dark_mode.get() {
            view! {
                button(
                    title="Toggle dark mode",
                    class="w-3",
                    on:click=toggle,
                    dangerously_set_inner_html=LIGHT_BULB_SVG,
                )
            }
        } else {
            view! {
                button(
                    title="Toggle dark mode",
                    class="w-3",
                    on:click=toggle,
                    dangerously_set_inner_html=CLOUD_MOON_SVG,
                )
            }
        })
    }
}

#[component]
fn Nav() -> View {
    view! {
        // css hack: use pseudo elements to make nested backdrop filters work
        nav(class="after:absolute after:z-neg after:top-0 after:left-0 after:right-0 after:bottom-0 \
                after:backdrop-filter after:backdrop-blur-lg after:backdrop-saturate-150 \
                bg-opacity-80 dark:bg-opacity-80 bg-gray-100 dark:bg-gray-800 border-b border-gray-400 dark:border-gray-600 transition-colors \
                px-4"
        ) {
            div(class="flex flex-row justify-between items-center h-12") {
                div(class="inline-flex flex-initial items-center") {
                    // In mobile, show a hamburger menu.
                    div(class="flex sm:hidden mr-2 flex-row items-center h-12") {
                        HamburgerMenu {}
                    }
                    // Brand section
                    div(class="ml-0 sm:ml-3 inline-block flex-initial") {
                        div(class="flex space-x-4") {
                            a(href="/#") {
                                img(src="/logo.svg", class="h-10 w-10")
                            }
                        }
                    }
                }
                div(class="flex flex-row mr-4 space-x-4 items-center text-gray-600 dark:text-gray-300") {
                    // Links section, only show in desktop view.
                    div(class="hidden sm:inline-flex items-center") {
                        NavLinks()
                    }
                    DarkModeToggle()
                }
            }
        }
    }
}

#[component]
pub fn NavLinks() -> View {
    static LINK_CLASS: &str =
        "py-2 px-4 text-sm hover:text-gray-800 dark:hover:text-gray-100 hover:underline";
    view! {
        a(class=LINK_CLASS, href="/docs/getting_started/installation") { "Book" }
        a(class=LINK_CLASS, href="https://docs.rs/sycamore") { "API" }
        a(class=LINK_CLASS, href="/news") { "News" }
        a(class=LINK_CLASS, href="https://github.com/sycamore-rs/sycamore") { "GitHub" }
        a(class=LINK_CLASS, href="https://discord.gg/vDwFUmm6mU") { "Discord" }
    }
}

#[component]
pub fn HamburgerMenu() -> View {
    static HAMBURGER_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" data-prefix="fas" data-icon="bars" class="svg-inline--fa fa-bars fa-w-14" role="img" viewBox="0 0 448 512"><path fill="currentColor" d="M16 132h416c8.837 0 16-7.163 16-16V76c0-8.837-7.163-16-16-16H16C7.163 60 0 67.163 0 76v40c0 8.837 7.163 16 16 16zm0 160h416c8.837 0 16-7.163 16-16v-40c0-8.837-7.163-16-16-16H16c-8.837 0-16 7.163-16 16v40c0 8.837 7.163 16 16 16zm0 160h416c8.837 0 16-7.163 16-16v-40c0-8.837-7.163-16-16-16H16c-8.837 0-16 7.163-16 16v40c0 8.837 7.163 16 16 16z"/></svg>"#;
    static CLOSE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" data-prefix="fas" data-icon="times" class="svg-inline--fa fa-times fa-w-11" role="img" viewBox="0 0 352 512"><path fill="currentColor" d="M242.72 256l100.07-100.07c12.28-12.28 12.28-32.19 0-44.48l-22.24-22.24c-12.28-12.28-32.19-12.28-44.48 0L176 189.28 75.93 89.21c-12.28-12.28-32.19-12.28-44.48 0L9.21 111.45c-12.28 12.28-12.28 32.19 0 44.48L109.28 256 9.21 356.07c-12.28 12.28-12.28 32.19 0 44.48l22.24 22.24c12.28 12.28 32.2 12.28 44.48 0L176 322.72l100.07 100.07c12.28 12.28 32.2 12.28 44.48 0l22.24-22.24c12.28-12.28 12.28-32.19 0-44.48L242.72 256z"/></svg>"#;

    let is_open = create_signal(false);
    let toggle = move |_| is_open.set(!is_open.get());
    let sidebar = use_context::<Signal<Option<(Option<String>, SidebarData)>>>();

    view! {
        // Menu navbar, hamburger button.
        (if is_open.get() {
            view! {
                button(
                    title="Menu",
                    class="inline-block w-8 p-2",
                    on:click=toggle,
                    dangerously_set_inner_html=CLOSE_SVG,
                )
            }
        } else {
            view! {
                button(
                    title="Menu",
                    class="inline-block w-8 p-2",
                    on:click=toggle,
                    dangerously_set_inner_html=HAMBURGER_SVG,
                )
            }
        })
        div(class=format!("backdrop-filter backdrop-blur-lg backdrop-saturate-150 \
                bg-opacity-80 dark:bg-opacity-80 bg-gray-100 dark:bg-gray-800 \
                border-r border-t border-gray-400 dark:border-gray-600 transition-all \
                p-2 fixed top-0 left-0 h-screen w-8/12 mt-12 pb-16 flex flex-col overflow-y-auto {}",
                if is_open.get() {
                    "duration-200 ease-out"
                } else {
                    "invisible transform -translate-x-full opacity-0 duration-150 ease-in"
                }
            ),
            on:click=toggle
        ) {
            NavLinks {}
            (if let Some((version, data)) = sidebar.get_clone() {
                let sidebar_current = SidebarCurrent {
                    version: version.unwrap_or_else(|| "next".to_string()),
                    path: String::new(),
                    data,
                };
                view! {
                    div(class="opacity-25 mx-2 p-px my-2 bg-current")
                    div(class="w-full"){
                        crate::sidebar::Sidebar(sidebar=sidebar_current)
                    }
                }
            } else {
                view! {  }
            })
        }
    }
}

#[component]
pub fn Header() -> View {
    view! {
        header(class="fixed top-0 z-50 w-full") {
            Nav {}
        }
    }
}
