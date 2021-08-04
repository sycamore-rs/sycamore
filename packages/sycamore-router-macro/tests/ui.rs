#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/router/*-pass.rs");
    if std::env::var("RUN_UI_TESTS").is_ok() {
        t.compile_fail("tests/router/*-fail.rs");
    }
}
