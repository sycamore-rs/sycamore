# Views

Sycamore uses the `view!` macro as an ergonomic way to create complex user
interfaces. You might have already seen it in the _"Hello, World!"_ example.

> Dislike macros? Check out the alternative [builder pattern](#builder-syntax).

## `view!` Syntax

Write your markup inside the `view!` macro and get a `View` expression.

### Elements

Creating HTML elements is easy as pie with the `view!` macro. Since you'll
likely want to create a lot of elements in your app, there is a special terse
syntax.

```rust
view! {
    // A simple div
    div {}
    // A div with a class
    div(class="foo")
    // An empty paragraph
    p {}
    // Custom elements!
    my-custom-element {}
}
```

### Text nodes

Of course, in your app, you probably want to display some text. To create a text
node, simply add a string literal.

```rust
view! {
    "Hello World!"
}
```

### Nesting

Creating all these top-level nodes is not very useful. You can create nested
nodes like so.

```rust
view! {
    div {
        p {
            span { "Hello " }
            strong { "World!" }
        }
    }
}
```

### Interpolation

Views can contain interpolated values. Anything that implements
`std::fmt::Display` will automatically be inserted as text into the DOM tree.
For example:

```rust
let my_number = 123;

view! {
    p {
        "This is my number: " (my_number)
    }
}
```

Other views created using the `view!` macro can also be interpolated using the
same syntax. For example:

```rust
let inner_view = view! {
    "Inside"
};

let outer_view = view! {
    "Outside"
    div {
        (inner_view)
    }
};
```

The cool thing about interpolation in Sycamore is that it is automatically kept
up to date with the value of the expression. Learn more about this in
[Reactivity](./reactivity).

### Attributes

Attributes (including classes and ids) can also be specified.

```rust
view! {
    p(class="my-class", id="my-paragraph", aria-label="My paragraph", "attr-42"="foo")
    button(disabled=true) {
       "My button"
    }
}
```

#### `dangerously_set_inner_html`

The special `dangerously_set_inner_html` attribute is used to set an HTML string
as the child of an element. This should generally be avoided because it is a
possible security risk. Never pass user input to this attribute as that will
create an XSS (Cross-Site Scripting) vulnerability.

```rust
view! {
    div(dangerously_set_inner_html="<span>Inner HTML!</span>")

    // DO NOT DO THIS!!!
    div(dangerously_set_inner_html=user_input)
    // DO NOT DO THIS!!!
}
```

Instead, when displaying user input, use interpolation syntax instead.

### Properties

Properties are set using the `prop:*` directive.

```rust
view! {
    input(type="checkbox", prop:indeterminate=true)
}
```

There are some properties that do not have an attribute, such as `indeterminate`
in HTML, which must be set using the `prop:*` directive.

There are a number of properties that have an associated attribute, such as
`value`, in these cases an attribute is deserialized to become the state of the
property. Consider using the `prop:*` for these cases when the value expected by
the element property is not a `string`.

### Events

Events are attached using the `on:*` directive.

```rust
view! {
    button(on:click=|_| { /* do something */ }) {
        "Click me"
    }
}
```

### Fragments

As seen in previous examples, views can also be fragments. You can create as
many nodes as you want at the top-level.

```rust
view! {
    p { "First child" }
    p { "Second child" }
}
```

Fragments can also be empty.

```rust
view! { }
```

## Builder syntax

For those who dislike macro DSLs, we also provide an ergonomic builder API for
constructing views. Add the builder prelude as well as the main sycamore prelude
to your source file.

```rust
use sycamore::prelude::*;
use sycamore::web::tags::*;
```

### Elements

Elements can easily be created by calling the corresponding function for the
HTML tag.

```rust
a()
button()
div()
// etc...
```

### Text nodes

Text nodes are just string literals. These can be added to a node using
`.children()`.

```rust
div().children(
    p().children("Hello World!")
)
```

`.children()` can take anything that implements `Into<View>`. This includes
other text nodes, elements, views, and more. Tuples can also be converted into
`View`. So to add more than a single child node, just construct a tuple:

```rust
div().children((
    span().children("Hello "),
    span().children("World!"),
))
```

### Interpolation

Functions also implement `Into<View>`. When a function is used, the node becomes
dynamic. This means that the reactive values accessed in the function will be
used as dependencies and will trigger a re-render whenever a value changes.

```rust
let state = create_signal(0);

p().children(("Value: ", move || state.get()))
```

### Attributes

Every attribute can also be set on an element via a method:

```rust
p().class("my-class").id("my-paragraph").attr("aria-label", "My paragraph")
```

Custom attributes (that are not part of the HTML spec or are `data-*` or
`aria-*` attributes) can be set using `.attr()`.

### Events

Events are attached using `.on(...)`.

```rust
button().on(ev::click, |_| console_dbg!("clicked!")).children("Click me!")
```
