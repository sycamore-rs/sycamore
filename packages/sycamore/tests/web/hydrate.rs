use expect_test::{expect, Expect};

use super::*;

fn check(actual: &str, expect: Expect) {
    expect.assert_eq(actual);
}

mod hello_world {
    use super::*;
    fn v() -> View {
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
        c.set_inner_html(&html);

        sycamore::hydrate_in_scope(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);
    }
}

mod hydrate_recursive {
    use super::*;
    fn v() -> View {
        view! { div { p { "Nested" } } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(v),
            expect![[r#"<div data-hk="0.0"><p>Nested</p></div>"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(v);
        let c = test_container();
        c.set_inner_html(&html);

        sycamore::hydrate_in_scope(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);
    }
}

mod multiple_nodes_at_same_depth {
    use super::*;
    fn v() -> View {
        view! { div { p { "First" } p { "Second" } } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(v),
            expect![[r#"<div data-hk="0.0"><p>First</p><p>Second</p></div>"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(v);
        let c = test_container();
        c.set_inner_html(&html);

        sycamore::hydrate_in_scope(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);
    }
}

mod top_level_fragment {
    use super::*;
    fn v() -> View {
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
        c.set_inner_html(&html);

        sycamore::hydrate_in_scope(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html);
    }
}

mod dynamic {
    use super::*;
    fn v(state: ReadSignal<i32>) -> View {
        view! { p { (state.get()) } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(|| v(*create_signal(0))),
            expect![[r#"<p data-hk="0.0">0</p>"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(|| v(*create_signal(0)));
        let c = test_container();
        c.set_inner_html(&html);

        let _ = create_root(|| {
            let state = create_signal(0);

            sycamore::hydrate_in_scope(|| v(*state), &c);

            assert_text_content!(query("p"), "0");

            // Reactivity should work normally.
            state.set(1);
            assert_text_content!(query("p"), "1");

            // P tag should still be the SSR-ed node, not a new node.
            assert_eq!(query("p").get_attribute("data-hk").as_deref(), Some("0.0"));
        });
    }
}

mod dynamic_with_siblings {
    use super::*;
    fn v(state: ReadSignal<i32>) -> View {
        view! { p { "Value: " (state.get()) "!" } }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(|| v(*create_signal(0))),
            expect![[r##"<p data-hk="0.0">Value: <!--#-->0<!--/-->!</p>"##]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(|| v(*create_signal(0)));
        let c = test_container();
        c.set_inner_html(&html);

        let _ = create_root(|| {
            let state = create_signal(0);

            sycamore::hydrate_in_scope(|| v(*state), &c);

            // Reactivity should work normally.
            state.set(1);
            assert_text_content!(query("p"), "Value: 1!");

            // P tag should still be the SSR-ed node, not a new node.
            assert_eq!(query("p").get_attribute("data-hk").as_deref(), Some("0.0"));
        });
    }
}

mod top_level_dynamic_with_siblings {
    use super::*;
    fn v(state: ReadSignal<i32>) -> View {
        view! { "Value: " (state.get()) "!" }
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(|| v(*create_signal(0))),
            expect![[r#"Value: 0!"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html = sycamore::render_to_string(|| v(*create_signal(0)));
        let c = test_container();
        c.set_inner_html(&html);

        let _ = create_root(|| {
            let state = create_signal(0);

            sycamore::hydrate_in_scope(|| v(*state), &c);

            // Reactivity should work normally.
            state.set(1);
            assert_text_content!(c, "Value: 1!");
        });
    }
}
