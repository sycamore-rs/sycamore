use crate::template::HtmlRoot;

static VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr", "command", "keygen", "menuitem",
];

/// Generates a string to be used for creating a `<template>` element.
pub fn gen_template_string(component: &HtmlRoot) -> String {
    let mut buf = String::new();

    buf
}
