# `template!`

Maple uses the `template!` macro as an ergonomic way to create complex user interfaces. You might have already seen it in the _"Hello, World!"_ example.

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
    button(disabled="true") {
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
