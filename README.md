# maple

A VDOM-less web library with fine-grained reactivity.

## Getting started

The recommended build tool is [Trunk](https://trunkrs.dev/).
Start by adding `maple-core` to your `Cargo.toml`:

```toml
maple-core = "0.2.0"
```

Add the following to your `src/main.rs` file:

```rust
use maple_core::prelude::*;

fn main() {
    let root = template! {
        p {
            "Hello World!"
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
            span { "Hello " }
            strong { "World!" }
        }
    }
};

// Attributes (including classes and ids) can also be specified.
template! {
    p(class="my-class", id="my-paragraph")
};

template! {
    button(disabled="true") {
        "My button"
    }
}

// Events are attached using the `on:*` directive.
template! {
    button(on:click=|_| { /* do something */ }) {
        "Click me"
    }
}
```

## Reactivity

Instead of relying on a Virtual DOM (VDOM), `maple` uses fine-grained reactivity to keep the DOM and state in sync.
In fact, the reactivity part of `maple` can be used on its own without the DOM rendering part.

Reactivity is based on reactive primitives. Here is an example:

```rust
use maple_core::prelude::*;
let state = Signal::new(0); // create an atom with an initial value of 0
```

If you are familiar with [React](https://reactjs.org/) hooks, this will immediately seem familiar to you.

Now, to access the state, we call the `.get()` method on `state` like this:

```rust
println!("The state is: {}", state.get()); // prints "The state is: 0"
```

To update the state, we call the `.set(...)` method on `state`:

```rust
state.set(1);
println!("The state is: {}", state.get()); // should now print "The state is: 0"
```

Why would this be useful? It's useful because it provides a way to easily be notified of any state changes.
For example, say we wanted to print out every state change. This can easily be accomplished like so:

```rust
let state = Signal::new(0);

create_effect(cloned!((state) => move || {
    println!("The state changed. New value: {}", state.get());
}));  // prints "The state changed. New value: 0" (note that the effect is always executed at least 1 regardless of state changes)

state.set(1); // prints "The state changed. New value: 1"
state.set(2); // prints "The state changed. New value: 2"
state.set(3); // prints "The state changed. New value: 3"
```

How does the `create_effect(...)` function know to execute the closure every time the state changes?
Calling `create_effect` creates a new _"reactivity scope"_ and calling `state.get()` inside this scope adds itself as a _dependency_.
Now, when `state.set(...)` is called, it automatically calls all its _dependents_, in this case, `state` as it was called inside the closure.

> #### What's that `cloned!` macro doing?
>
> The `cloned!` macro is an utility macro for cloning the variables into the following expression. The previous `create_effect` function call could very well have been written as:
>
> ```rust
> create_effect({
>     let state = state.clone();
>     move || {
>         println!("The state changed. New value: {}", state.get());
>     }
> }));
> ```
>
> This is ultimately just a workaround until something happens in [Rust RFC #2407](https://github.com/rust-lang/rfcs/issues/2407).

We can also easily create a derived state using `create_memo(...)` which is really just an ergonomic wrapper around `create_effect`:

```rust
let state = Signal::new(0);
let double = create_memo(cloned!((state) => move || *state.get() * 2));

assert_eq!(*double.get(), 0);

state.set(1);
assert_eq!(*double.get(), 2);
```

`create_memo(...)` automatically recomputes the derived value when any of its dependencies change.

Now that you understand `maple`'s reactivity system, we can look at how to use this to update the DOM.

### Using reactivity with DOM updates

Reactivity is automatically built-in into the `template!` macro. Say we have the following code:

```rust
use maple_core::prelude::*;

let state = Signal::new(0);

let root = template! {
    p {
        (state.get())
    }
}
```

This will expand to something approximately like:

```rust
use maple_core::prelude::*;
use maple_core::internal;

let state = Signal::new(0);

let root = {
    let element = internal::element(p);
    let text = internal::text(String::new() /* placeholder */);
    create_effect(move || {
        // update text when state changes
        text.set_text_content(Some(&state.get()));
    });

    internal::append(&element, &text);

    element
}
```

If we call `state.set(...)` somewhere else in our code, the text content will automatically be updated!

## Components

Components in `maple` are simply functions that return `HtmlElement`.
They receive their props through function arguments.

For components to automatically react to prop changes, they should accept a prop with type `StateHandle<T>` and call the function in the `template!` to subscribe to the state.

Getting a `StateHandle<T>` for a `Signal<T>` is easy. Just call the `.handle()` method.

Here is an example of a simple component:

```rust
// This is temporary and will later be removed.
// Currently, the template! macro assumes that all components start with an uppercase character.
#![allow(non_snake_case)]

use maple_core::prelude::*;

fn Component(value: StateHandle<i32>) -> TemplateResult {
    template! {
        div(class="my-component") {
            "Value: " (value.get())
        }
    }
}

// ...
let state = Signal::new(0);

template! {
    Component(state.handle())
}

state.set(1); // automatically updates value in Component
```

## Contributing

Issue reports and PRs are welcome!
Get familiar with the project structure with [ARCHITECTURE.md](https://github.com/lukechu10/maple/blob/master/ARCHITECTURE.md).
