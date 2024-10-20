---
title: View Builder API
---

# The View Builder API

For those that prefer not to use macros, Sycamore also provides an ergonomic
builder API for composing views.

Start by importing all of the builder HTML tags.

```rust
use sycamore::web::tags::*;
```

## Elements

Elements can easily be created by calling the corresponding function for the
HTML tag.

```rust
a()
button()
div()
// etc...
```

These functions return specific element types such as `HtmlButtonElement`. To
convert those into `View`s, simply use `.into()`. For example, if you are
composing a component, you could write something like:

```rust
#[component]
fn Button() -> View {
    button().into()
}
```

### Children

You can add children to a element by using the `.children(...)` method. Anything
that implements `Into<View>` can be appended as a child node. This includes
string literals, numbers, other elements, signals, functions, and views.

```rust
div().children(
    p().children("Hello World!")
)
```

Tuples also implement `Into<View>` which lets you add multiple children in one
shot.

```rust
div().children((
    span().children("Hello "),
    span().children("World!"),
))
```

### Dynamic views

To add dynamic content, you can pass a function to `.children(...)`. This will
automatically make the view dynamic and will trigger a re-render whenever a
dependency changes.

```rust
let state = create_signal(0);

p().children((
    "value = ",
    state,
    ", doubled = ",
    move || state.get() * 2,
))
```

## Attributes

Every attribute can be set on an element via a method of the same name. Custom
attributes (that are not part of the HTML spec or are `data-*` or `aria-*`
attributes) can be set using `.attr(...)`.

```rust
p().class("my-class").id("my-paragraph").attr("aria-label", "My paragraph")
```

### Setting inner html

The inner HTML can be set using the special `.dangerously_set_inner_html(...)`
method. This should generally be avoided if possible because it is a possible
security risk. Never pass user input to this attribute as that will create an
XSS (Cross-Site Scripting) vulnerability.

```rust
div().dangerously_set_inner_html(inner_html)
```

### Setting node refs

A node ref can be attached using the special `r#ref(...)` method.

```rust
let node = create_node_ref();
button().r#ref(node)
```

For more details, see [Node Ref](/book/guide/node-ref)

### Properties

Properties are set using the `.prop(...)` method.

```rust
input().r#type("checkbox").prop("indeterminate", true)
```

There are some properties that do not have a matching attribute, such as
`indeterminate` in HTML, and which must be set using the `prop:*` directive.
Other properties such as `value` have
[unintuitive behavior](https://stackoverflow.com/a/7986111/9443288) when using
the attribute version.

### Events

Events are attached using `.on(...)`.

```rust
button().on(ev::click, |_| console_log!("clicked!")).children("Click me!")
```

### Optional attributes

Stringy attributes can also be optional. To make an attribute optional, simply
pass in an `Option`! For example:

```rust
let attr = create_signal(None::<String>);
div().attr("data-attr", attr)
// Will render <div></div> instead of <div data-attr></div> if attr is None.
```

## Components

Unfortunately with the builder API, we have a little more friction when using
components. Since components are just functions, you can call them directly.
However, when passing props, you will usually need to reference the prop type
explicitly and call `.build()` at the end to construct the final prop value.

```rust
// The component macro automatically generates the `Button_Props` type
// if we are using inline props.
#[component(inline_props)]
fn Button(class: String) -> View { ... }

div()
    .children(Button(
        Button_Props::builder().class("my-button".to_string()).build()
    ))
```
