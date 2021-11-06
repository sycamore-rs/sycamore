# `view!`

Sycamore uses the `view!` macro as an ergonomic way to create complex user interfaces. You might
have already seen it in the _"Hello, World!"_ example.

## Syntax

### Elements

Creating HTML elements is easy as pie with the `view!` macro. Since you'll likely want to create
a lot of elements in your app, there is a special terse syntax.

```rust
view! {
    // A simple div
    div
    // A div with a class
    div(class="foo")
    // An empty paragraph
    p
    // Custom elements!
    my-custom-element
}
```

### Text nodes

Of course, in your app, you probably want to display some text. To create a text node, simply add a
string literal (using `"`).

```rust
view! {
    "Hello World!"
}
```

### Nesting

Creating all these top-level nodes is not very useful. You can create nested nodes like so.

```rust
view! {
    div(class="foo") {
        p {
            span { "Hello " }
            strong { "World!" }
        }
    }
}
```

### Attributes

Attributes (including classes and ids) can also be specified.

```rust
view! {
    p(class="my-class", id="my-paragraph", aria-label="My paragraph")
    button(disabled=true) {
       "My button"
    }
}
```

#### `dangerously_set_inner_html`

The special `dangerously_set_inner_html` attribute is used to set an HTML string as the child of an
element. This should generally be avoided because it is a possible security risk. Never pass user
input to this attribute as that will create an XSS (Cross-Site Scripting) vulnerability.

```rust
view! {
    div(dangerously_set_inner_html="<span>Inner HTML!</span>")

    // DO NOT DO THIS!!!
    div(dangerously_set_inner_html=user_input)
    // DO NOT DO THIS!!!
}
```

Instead, when displaying user input, use interpolation syntax instead.

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

As seen in previous examples, views can also be fragments. You can create as many nodes as you
want at the top-level.

```rust
view! {
    p { "First child" }
    p { "Second child" }
}
```

Fragments can also be empty.

```rust
view! {}
```

## Interpolation

Templates can contain interpolated values. Anything that implements `std::fmt::Display` will
automatically be inserted as text into the DOM tree. For example:

```rust
let my_number = 123;

view! {
    p {
        // Automatically display my_number as a string using std::fmt::Display
        "This is my number: " (my_number)
    }
}
```

Other views created using the `view!` macro can also be interpolated using the same syntax.
For example:

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

The cool thing about interpolation in Sycamore is that it is automatically kept up to date with the
value of the expression. Learn more about this in [Reactivity](./reactivity).
