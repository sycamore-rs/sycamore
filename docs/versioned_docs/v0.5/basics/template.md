# `template!`

Sycamore uses the `template!` macro as an ergonomic way to create complex user interfaces. You might have already seen it in the _"Hello, World!"_ example.

The `template!` macro is quite powerful. You can create nested nodes like so.

```rust
template! {
    div {
        p {
            span { "Hello " }
            strong { "World!" }
        }
    }
}
```

Attributes (including classes and ids) can also be specified.

```rust
template! {
    p(class="my-class", id="my-paragraph", aria-label="My paragraph")
}

template! {
    button(disabled=true) {
       "My button"
    }
}
```

Events are attached using the `on:*` directive.

```rust
template! {
    button(on:click=|_| { /* do something */ }) {
        "Click me"
    }
}
```

Templates can also be fragments.

```rust
template! {
    p { "First child" }
    p { "Second child" }
}
```

Or be empty.

```rust
template! {}
```

## Interpolation

Templates can contain interpolated values. Anything that implements `std::fmt::Display` will automatically be inserted as text into the DOM tree. For example:

```rust
let my_number = 123;

template! {
    p {
        // Automatically display my_number as a string using std::fmt::Display
        "This is my number: " (my_number)
    }
}
```

Other templates created using the `template!` macro can also be interpolated. For example:

```rust
let inner_template = template! {
    "Inside"
};

let outer_template = template! {
    "Outside"
    div {
        (inner_template)
    }
};
```

The cool thing about interpolation in Sycamore is that it is automatically kept up to date with the value of the expression. Learn more about this in [Reactivity](./reactivity).
