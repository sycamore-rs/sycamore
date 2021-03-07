# maple

A VDOM-less web library.

## Getting started

The recommended build tool is [Trunk](https://trunkrs.dev/).
Start by adding `maple-core` to your `Cargo.toml`:

```toml
maple-core = {path = "../../maple-core"}
```

Add the following to your `src/main.rs` file:

```rust
use maple_core::prelude::*;

fn main() {
    let root = template! {
        p {
            # "Hello World!"
        }
    };

    render(root);
}
```

That's it! There's your hello world program using `maple`. To run the app, simply run `trunk serve --open` and see the result in your web browser.

## The `template!` macro

`maple` uses the `template!` macro as an ergonomic way to create complex user interfaces.

```rust
// You can create nested elements.
template! {
    div {
        p {
            span {
                # "Hello "
            }
            strong {
                # "World!"
            }
        }
    }
};

// Attributes (including classes and ids) can also be specified.
template! {
    p(class="my-class", id="my-paragraph")
};

template! {
    button(disabled="true") {
        # "My button"
    }
}

// Events are attached using the `on:*` directive.
template! {
    button(on:click=|| { /* do something */ }) {
        # "Click me"
    }
}
```

## Reactivity

TODO: write docs

## Contributing

Issue reports and PRs are welcome!
