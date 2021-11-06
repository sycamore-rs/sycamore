#[test]
fn view_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/view/*-pass.rs");
    if std::env::var("RUN_UI_TESTS").is_ok() {
        t.compile_fail("tests/view/*-fail.rs");
    }
}

#[test]
fn component_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/component/*-pass.rs");
    if std::env::var("RUN_UI_TESTS").is_ok() {
        t.compile_fail("tests/component/*-fail.rs");
    }
}
