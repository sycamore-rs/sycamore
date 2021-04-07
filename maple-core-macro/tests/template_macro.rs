#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/template/*-fail.rs");

    t.pass("tests/template/*-pass.rs");
}
