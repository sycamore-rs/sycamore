use sycamore_core2::view::ToView;
use sycamore_reactive::*;
use sycamore_web2::html::elements::*;
use sycamore_web2::html::*;
use sycamore_web2::*;

fn app(cx: Scope) -> View {
    [
        p(cx)
            .with(attr::class, "test")
            .child("Hello World!")
            .child("This is a test!")
            .view(),
        button(cx)
            .with(attr::class, "btn")
            .child("Click me!")
            .view(),
        svg(cx)
            .with(attr::class, "svg")
            .child(
                line(cx)
                    .with(attr::color, "red")
                    .with(attr::x1, "0")
                    .with(attr::y1, "0")
                    .with(attr::x2, "200")
                    .with(attr::y2, "200")
                    .with(attr::style, "stroke:rgb(255,0,0);stroke-width:2")
                    .view(),
            )
            .view(),
    ]
    .to_view(cx)
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let ssr = sycamore_web2::render::render_to_string(app);
    log::info!("{ssr}");
    sycamore_web2::render::render(app);
}
