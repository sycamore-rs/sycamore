#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/router/*-fail.rs");

    t.pass("tests/router/*-pass.rs");
}
