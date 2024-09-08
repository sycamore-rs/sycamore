use expect_test::{expect, Expect};
use sycamore::web::tags::*;

use super::*;

fn check(actual: impl FnOnce() -> View, expect: &Expect) {
    expect.assert_eq(&sycamore::render_to_string(actual));
}

mod hello_world {
    use super::*;
    fn v() -> View {
        p().children("Hello World!").into()
    }
    static EXPECT: Expect = expect!["<p data-hk=0>Hello World!</p>"];
    #[test]
    fn ssr() {
        check(v, &EXPECT);
    }
    #[wasm_bindgen_test]
    fn test() {
        let c = test_container();
        c.set_inner_html(EXPECT.data());

        sycamore::hydrate_to(v, &c);

        assert_text_content!(query("p"), "Hello World!");
    }
}

mod hydrate_recursive {
    use super::*;
    fn v() -> View {
        div().children(p().children("Nested")).into()
    }
    static EXPECT: Expect = expect!["<div data-hk=0><p data-hk=1>Nested</p></div>"];
    #[test]
    fn ssr() {
        check(v, &EXPECT);
    }
    #[wasm_bindgen_test]
    fn test() {
        let c = test_container();
        c.set_inner_html(EXPECT.data());

        sycamore::hydrate_to(v, &c);

        assert_text_content!(query("p"), "Nested");
    }
}

mod multiple_nodes_at_same_depth {
    use super::*;
    fn v() -> View {
        div()
            .children((
                p().id("first").children("First"),
                p().id("second").children("Second"),
            ))
            .into()
    }
    static EXPECT: Expect = expect![[
        r#"<div data-hk=0><p id="first" data-hk=1>First</p><p id="second" data-hk=2>Second</p></div>"#
    ]];
    #[test]
    fn ssr() {
        check(v, &EXPECT);
    }
    #[wasm_bindgen_test]
    fn test() {
        let c = test_container();
        c.set_inner_html(EXPECT.data());

        sycamore::hydrate_to(v, &c);

        assert_text_content!(query("div>p#first"), "First");
        assert_text_content!(query("div>p#second"), "Second");
    }
}

mod top_level_fragment {
    use super::*;
    fn v() -> View {
        (
            p().id("first").children("First"),
            p().id("second").children("Second"),
        )
            .into()
    }
    static EXPECT: Expect =
        expect![[r#"<p id="first" data-hk=0>First</p><p id="second" data-hk=1>Second</p>"#]];
    #[test]
    fn ssr() {
        check(v, &EXPECT);
    }
    #[wasm_bindgen_test]
    fn test() {
        let c = test_container();
        c.set_inner_html(EXPECT.data());

        sycamore::hydrate_to(v, &c);

        // Hydration should not change inner html.
        assert_text_content!(query("p#first"), "First");
        assert_text_content!(query("p#second"), "Second");
    }
}

mod dynamic {
    use super::*;
    fn v(state: ReadSignal<i32>) -> View {
        p().children(move || state.get()).into()
    }
    static EXPECT: Expect = expect!["<p data-hk=0><!--/-->0<!--/--></p>"];
    #[test]
    fn ssr() {
        check(|| v(*create_signal(0)), &EXPECT);
    }
    #[wasm_bindgen_test]
    fn test() {
        let c = test_container();
        c.set_inner_html(EXPECT.data());

        let _ = create_root(|| {
            let state = create_signal(0);

            sycamore::hydrate_in_scope(|| v(*state), &c);

            assert_text_content!(query("p"), "0");

            // Reactivity should work normally.
            state.set(1);
            assert_text_content!(query("p"), "1");

            // P tag should still be the SSR-ed node, not a new node.
            assert_eq!(query("p").get_attribute("data-hk").as_deref(), Some("0"));
        });
    }
}

mod dynamic_with_siblings {
    use super::*;
    fn v(state: ReadSignal<i32>) -> View {
        p().children(("Value: ", move || state.get(), "!")).into()
    }
    static EXPECT: Expect = expect!["<p data-hk=0>Value: <!--/-->0<!--/-->!</p>"];
    #[test]
    fn ssr() {
        check(|| v(*create_signal(0)), &EXPECT);
    }
    #[wasm_bindgen_test]
    fn test() {
        let c = test_container();
        c.set_inner_html(EXPECT.data());

        let _ = create_root(|| {
            let state = create_signal(0);

            sycamore::hydrate_in_scope(|| v(*state), &c);

            // Reactivity should work normally.
            state.set(1);
            assert_text_content!(query("p"), "Value: 1!");

            // P tag should still be the SSR-ed node, not a new node.
            assert_eq!(query("p").get_attribute("data-hk").as_deref(), Some("0"));
        });
    }
}

mod top_level_dynamic_with_siblings {
    use super::*;
    fn v(state: ReadSignal<i32>) -> View {
        ("Value: ", move || state.get(), "!").into()
    }
    static EXPECT: Expect = expect!["Value: <!--/-->0<!--/-->!"];
    #[test]
    fn ssr() {
        check(|| v(*create_signal(0)), &EXPECT);
    }
    #[wasm_bindgen_test]
    fn test() {
        let c = test_container();
        c.set_inner_html(EXPECT.data());

        let _ = create_root(|| {
            let state = create_signal(0);

            sycamore::hydrate_in_scope(|| v(*state), &c);

            // Reactivity should work normally.
            state.set(1);
            assert_text_content!(c, "Value: 1!");
        });
    }
}
