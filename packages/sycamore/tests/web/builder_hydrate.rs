use expect_test::{expect, Expect};
use sycamore::builder::prelude::*;

use super::*;

fn check(actual: &str, expect: Expect) {
    expect.assert_eq(actual);
}

mod hello_world {
    use super::*;
    fn v<G: Html>(ctx: Scope) -> View<G> {
        h(p).t("Hello World!").view(ctx)
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
        let html_str = sycamore::render_to_string(v);
        let c = test_container();
        c.set_inner_html(&html_str);

        sycamore::hydrate_to(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html_str);
    }
}

mod hydrate_recursive {
    use super::*;
    fn v<G: Html>(ctx: Scope) -> View<G> {
        h(div).c(h(p).t("Nested")).view(ctx)
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
        let html_str = sycamore::render_to_string(v);
        let c = test_container();
        c.set_inner_html(&html_str);

        sycamore::hydrate_to(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html_str);
    }
}

mod multiple_nodes_at_same_depth {
    use super::*;
    fn v<G: Html>(ctx: Scope) -> View<G> {
        h(div).c(h(p).t("First")).c(h(p).t("Second")).view(ctx)
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
        let html_str = sycamore::render_to_string(v);
        let c = test_container();
        c.set_inner_html(&html_str);

        sycamore::hydrate_to(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html_str);
    }
}

mod top_level_fragment {
    use super::*;
    fn v<G: Html>(ctx: Scope) -> View<G> {
        fragment([h(p).t("First").view(ctx), h(p).t("Second").view(ctx)])
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
        let html_str = sycamore::render_to_string(v);
        let c = test_container();
        c.set_inner_html(&html_str);

        sycamore::hydrate_to(v, &c);

        // Hydration should not change inner html.
        assert_eq!(c.inner_html(), html_str);
    }
}

mod dynamic {
    use super::*;
    fn v<'a, G: Html>(ctx: Scope<'a>, state: &'a ReadSignal<i32>) -> View<G> {
        h(p).dyn_t(|| state.get().to_string()).view(ctx)
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(|ctx| v(ctx, ctx.create_signal(0))),
            expect![[r##"<p data-hk="0.0"><!--#-->0<!--/--></p>"##]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html_str = sycamore::render_to_string(|ctx| v(ctx, ctx.create_signal(0)));
        let c = test_container();
        c.set_inner_html(&html_str);

        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);

            sycamore::hydrate_to(|_| v(ctx, state), &c);

            assert_eq!(
                c.query_selector("p")
                    .unwrap()
                    .unwrap()
                    .text_content()
                    .unwrap(),
                "0"
            );

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
        });
    }
}

mod dynamic_with_siblings {
    use super::*;
    fn v<'a, G: Html>(ctx: Scope<'a>, state: &'a ReadSignal<i32>) -> View<G> {
        h(p).t("Value: ")
            .dyn_t(|| state.get().to_string())
            .t("!")
            .view(ctx)
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(|ctx| v(ctx, ctx.create_signal(0))),
            expect![[r##"<p data-hk="0.0">Value: <!--#-->0<!--/-->!</p>"##]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html_str = sycamore::render_to_string(|ctx| v(ctx, ctx.create_signal(0)));
        let c = test_container();
        c.set_inner_html(&html_str);

        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);

            sycamore::hydrate_to(|_| v(ctx, state), &c);

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
        });
    }
}

mod dynamic_template {
    use super::*;
    fn v<'a, G: Html>(ctx: Scope<'a>, state: &'a ReadSignal<View<G>>) -> View<G> {
        h(p).t("before")
            .dyn_c(|| state.get().as_ref().clone())
            .t("after")
            .view(ctx)
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(|ctx| v(ctx, ctx.create_signal(view! { ctx, "text" }))),
            expect![[r##"<p data-hk="0.0">before<!--#-->text<!--/-->after</p>"##]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html_str =
            sycamore::render_to_string(|ctx| v(ctx, ctx.create_signal(view! { ctx, "text" })));
        let c = test_container();
        c.set_inner_html(&html_str);

        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(view! { ctx, "text" });

            sycamore::hydrate_to(|_| v(ctx, state), &c);

            // Reactivity should work normally.
            state.set(view! { ctx, span { "nested node" } });
            assert_eq!(
                c.query_selector("p")
                    .unwrap()
                    .unwrap()
                    .text_content()
                    .unwrap(),
                "beforenested nodeafter"
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
        });
    }
}

mod top_level_dynamic_with_siblings {
    use super::*;
    fn v<'a, G: Html>(ctx: Scope<'a>, state: &'a ReadSignal<i32>) -> View<G> {
        fragment([t("Value: "), dyn_t(ctx, || state.get().to_string()), t("!")])
    }
    #[test]
    fn ssr() {
        check(
            &sycamore::render_to_string(|ctx| v(ctx, ctx.create_signal(0))),
            expect![[r#"Value: 0!"#]],
        );
    }
    #[wasm_bindgen_test]
    fn test() {
        let html_str = sycamore::render_to_string(|ctx| v(ctx, ctx.create_signal(0)));
        let c = test_container();
        c.set_inner_html(&html_str);

        create_scope_immediate(|ctx| {
            let state = ctx.create_signal(0);

            sycamore::hydrate_to(|_| v(ctx, state), &c);

            // Reactivity should work normally.
            state.set(1);
            assert_eq!(c.text_content().unwrap(), "Value: 1!");
        });
    }
}
