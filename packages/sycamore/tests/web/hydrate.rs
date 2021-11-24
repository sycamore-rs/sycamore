use expect_test::{expect, Expect};

use super::*;

fn check(actual: &str, expect: Expect) {
    expect.assert_eq(actual);
}

mod hello_world {
    use super::*;
    fn v<G: Html>() -> View<G> {
        view! { p { "Hello World!" } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(v),
            expect![[r#"<p data-hk="0.0">Hello World!</p>"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(v);
        let c = test_container();
        c.clone().set_inner_html(&html);

        sycamore::hydrate_to(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);
    }
}

mod hydrate_recursive {
    use super::*;
    fn v<G: Html>() -> View<G> {
        view! { div { p { "Nested" } } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(v),
            expect![[r#"<div data-hk="0.0"><p data-hk="0.1">Nested</p></div>"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(v);
        let c = test_container();
        c.clone().set_inner_html(&html);

        sycamore::hydrate_to(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);
    }
}

mod multiple_nodes_at_same_depth {
    use super::*;
    fn v<G: Html>() -> View<G> {
        view! { div { p { "First" } p { "Second" } } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(v),
            expect![[
                r#"<div data-hk="0.0"><p data-hk="0.1">First</p><p data-hk="0.2">Second</p></div>"#
            ]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(v);
        let c = test_container();
        c.clone().set_inner_html(&html);

        sycamore::hydrate_to(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);
    }
}

mod top_level_fragment {
    use super::*;
    fn v<G: Html>() -> View<G> {
        view! { p { "First" } p { "Second" } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(v),
            expect![[r#"<p data-hk="0.0">First</p><p data-hk="0.1">Second</p>"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(v);
        let c = test_container();
        c.clone().set_inner_html(&html);

        sycamore::hydrate_to(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);
    }
}

mod dynamic {
    use super::*;
    fn v<G: Html>(state: ReadSignal<i32>) -> View<G> {
        view! { p { (state.get()) } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(|| v(Signal::new(0).handle())),
            expect![[r#"<p data-hk="0.0">0</p>"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(|| v(Signal::new(0).handle()));
        let c = test_container();
        c.clone().set_inner_html(&html);

        let state = Signal::new(0);

        sycamore::hydrate_to(cloned!(state => move || v(state.handle())), &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);

        // Reactivity should work normally.
        state.set(1);
        assert_eq!(
            c.query_selector("p")
                .unwrap()
                .unwrap()
                .text_content()
                .unwrap(),
            "1"
        );

        // P tag should still be the SSR-ed node, not a new node.
        assert_eq!(
            c.query_selector("p")
                .unwrap()
                .unwrap()
                .get_attribute("data-hk")
                .as_deref(),
            Some("0.0")
        );
    }
}

mod dynamic_with_siblings {
    use super::*;
    fn v<G: Html>(state: ReadSignal<i32>) -> View<G> {
        view! { p { "Value: " (state.get()) "!" } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(|| v(Signal::new(0).handle())),
            expect![[r#"<p data-hk="0.0">Value: 0!</p>"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(|| v(Signal::new(0).handle()));
        let c = test_container();
        c.clone().set_inner_html(&html);

        let state = Signal::new(0);

        sycamore::hydrate_to(cloned!(state => move || v(state.handle())), &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);

        // Reactivity should work normally.
        state.set(1);
        assert_eq!(
            c.query_selector("p")
                .unwrap()
                .unwrap()
                .text_content()
                .unwrap(),
            "Value: 1!"
        );

        // P tag should still be the SSR-ed node, not a new node.
        assert_eq!(
            c.query_selector("p")
                .unwrap()
                .unwrap()
                .get_attribute("data-hk")
                .as_deref(),
            Some("0.0")
        );
    }
}
