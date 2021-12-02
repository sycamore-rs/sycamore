use sycamore::context::use_context;
use sycamore::prelude::*;

use crate::DarkMode;

#[component(DarkModeToggle<G>)]
fn dark_mode_toggle() -> View<G> {
    static LIGHT_BULB_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" data-prefix="fas" data-icon="lightbulb" class="svg-inline--fa fa-lightbulb fa-w-11" role="img" viewBox="0 0 352 512"><path fill="currentColor" d="M96.06 454.35c.01 6.29 1.87 12.45 5.36 17.69l17.09 25.69a31.99 31.99 0 0 0 26.64 14.28h61.71a31.99 31.99 0 0 0 26.64-14.28l17.09-25.69a31.989 31.989 0 0 0 5.36-17.69l.04-38.35H96.01l.05 38.35zM0 176c0 44.37 16.45 84.85 43.56 115.78 16.52 18.85 42.36 58.23 52.21 91.45.04.26.07.52.11.78h160.24c.04-.26.07-.51.11-.78 9.85-33.22 35.69-72.6 52.21-91.45C335.55 260.85 352 220.37 352 176 352 78.61 272.91-.3 175.45 0 73.44.31 0 82.97 0 176zm176-80c-44.11 0-80 35.89-80 80 0 8.84-7.16 16-16 16s-16-7.16-16-16c0-61.76 50.24-112 112-112 8.84 0 16 7.16 16 16s-7.16 16-16 16z"/></svg>"#;
    static CLOUD_MOON_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" data-prefix="far" data-icon="moon" class="svg-inline--fa fa-moon fa-w-16" role="img" viewBox="0 0 512 512"><path fill="currentColor" d="M279.135 512c78.756 0 150.982-35.804 198.844-94.775 28.27-34.831-2.558-85.722-46.249-77.401-82.348 15.683-158.272-47.268-158.272-130.792 0-48.424 26.06-92.292 67.434-115.836 38.745-22.05 28.999-80.788-15.022-88.919A257.936 257.936 0 0 0 279.135 0c-141.36 0-256 114.575-256 256 0 141.36 114.576 256 256 256zm0-464c12.985 0 25.689 1.201 38.016 3.478-54.76 31.163-91.693 90.042-91.693 157.554 0 113.848 103.641 199.2 215.252 177.944C402.574 433.964 344.366 464 279.135 464c-114.875 0-208-93.125-208-208s93.125-208 208-208z"/></svg>"#;

    let dark_mode = use_context::<DarkMode>();
    let toggle = cloned!((dark_mode) => move |_| dark_mode.0.set(!*dark_mode.0.get()));

    view! {
        button(
            title="Toggle dark mode",
            class="w-3",
            on:click=toggle,
            // Use dangerously_set_inner_html because SVG is not supported yet in view! macro.
            dangerously_set_inner_html=if *dark_mode.0.get() { LIGHT_BULB_SVG } else { CLOUD_MOON_SVG },
        )
    }
}

#[component(Nav<G>)]
fn nav() -> View<G> {
    view! {
        nav(class="px-8 h-12 backdrop-filter backdrop-blur-sm backdrop-saturate-150 bg-opacity-80 \
        bg-gray-100 dark:bg-gray-800 border-b border-gray-400 dark:border-gray-600 transition-colors") {
            // Only show nav links in desktop view.
            div(class="hidden sm:flex flex-row justify-between items-center h-12") {
                // Brand section
                div(class="inline-block flex-initial") {
                    div(class="flex space-x-4") {
                        a(href="/#", class="py-2 px-3 text-sm text-white font-medium \
                        bg-gray-500 hover:bg-gray-600 transition-colors rounded") {
                            "Sycamore"
                        }
                    }
                }
                // Links section
                div(class="inline-flex flex-row ml-2 space-x-4 text-gray-600 dark:text-gray-300") {
                    NavLinks()
                    DarkModeToggle()
                }
            }
            // In mobile, collapse into hamburger menu.
            div(class="flex sm:hidden h-12") {
                HamburgerMenu()
            }
        }
    }
}

#[component(NavLinks<G>)]
pub fn nav_links() -> View<G> {
    static LINK_CLASS: &str =
        "py-2 px-3 text-sm hover:text-gray-800 dark:hover:text-gray-100 hover:underline";
    view! {
        a(class=LINK_CLASS, href="/docs/getting_started/installation") { "Book" }
        a(class=LINK_CLASS, href="https://docs.rs/sycamore") { "API" }
        a(class=LINK_CLASS, href="/news") { "News" }
        a(class=LINK_CLASS, href="https://github.com/sycamore-rs/sycamore") { "GitHub" }
        a(class=LINK_CLASS, href="https://discord.gg/vDwFUmm6mU") { "Discord" }
    }
}

#[component(HamburgerMenu<G>)]
pub fn hamburger_menu() -> View<G> {
    static HAMBURGER_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" data-prefix="fas" data-icon="bars" class="svg-inline--fa fa-bars fa-w-14" role="img" viewBox="0 0 448 512"><path fill="currentColor" d="M16 132h416c8.837 0 16-7.163 16-16V76c0-8.837-7.163-16-16-16H16C7.163 60 0 67.163 0 76v40c0 8.837 7.163 16 16 16zm0 160h416c8.837 0 16-7.163 16-16v-40c0-8.837-7.163-16-16-16H16c-8.837 0-16 7.163-16 16v40c0 8.837 7.163 16 16 16zm0 160h416c8.837 0 16-7.163 16-16v-40c0-8.837-7.163-16-16-16H16c-8.837 0-16 7.163-16 16v40c0 8.837 7.163 16 16 16z"/></svg>"#;
    static CLOSE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false" data-prefix="fas" data-icon="times" class="svg-inline--fa fa-times fa-w-11" role="img" viewBox="0 0 352 512"><path fill="currentColor" d="M242.72 256l100.07-100.07c12.28-12.28 12.28-32.19 0-44.48l-22.24-22.24c-12.28-12.28-32.19-12.28-44.48 0L176 189.28 75.93 89.21c-12.28-12.28-32.19-12.28-44.48 0L9.21 111.45c-12.28 12.28-12.28 32.19 0 44.48L109.28 256 9.21 356.07c-12.28 12.28-12.28 32.19 0 44.48l22.24 22.24c12.28 12.28 32.2 12.28 44.48 0L176 322.72l100.07 100.07c12.28 12.28 32.2 12.28 44.48 0l22.24-22.24c12.28-12.28 12.28-32.19 0-44.48L242.72 256z"/></svg>"#;

    let is_open = Signal::new(false);

    let toggle = cloned!(is_open => move |_| is_open.set(!*is_open.get()));

    view! {
        // Menu navbar, hamburger button.
        button(
            title="Menu",
            class="inline-block w-5",
            on:click=toggle,
            // Use dangerously_set_inner_html because SVG is not supported yet in view! macro.
            dangerously_set_inner_html=if *is_open.get() { CLOSE_SVG } else { HAMBURGER_SVG },
        )
    }
}

#[component(Header<G>)]
pub fn header() -> View<G> {
    view! {
        header(class="fixed top-0 z-50 w-full") {
            Nav()
        }
    }
}
