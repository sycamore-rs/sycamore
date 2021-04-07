# Components

Components in `maple` are simply functions that return `TemplateResult<G>`.
They receive their props through function arguments.

For components to automatically react to prop changes, they should accept a prop with type `StateHandle<T>` and call the function in the `template!` to subscribe to the state.
A `StateHandle<T>` is just a readonly `Signal<T>`.

Getting a `StateHandle<T>` from a `Signal<T>` is easy. Just call the `.handle()` method.

Here is an example of a simple component that displays the value of its prop:

```rust
// This is temporary and will later be removed.
// Currently, the template! macro assumes that all
// components start with an uppercase character.


use maple_core::prelude::*;

fn MyComponent<G: GenericNode>(value: StateHandle<i32>) -> TemplateResult<G> {
    template! {
        div(class="my-component") {
            "Value: " (value.get())
        }
    }
}

// ...
let state = Signal::new(0);

template! {
    MyComponent(state.handle())
}

state.set(1); // automatically updates value in Component
```
