# Components

Components in `sycamore` are structs that implement `Component`. A component can automatically be
created with the `#[component(ComponentName<G>)]` attribute on a function.

Components receive their props through function arguments.

For components to automatically react to prop changes, they should accept a prop with type
`StateHandle<T>` and call the function in the `template!` to subscribe to the state. A
`StateHandle<T>` is just a readonly `Signal<T>`.

Getting a `StateHandle<T>` from a `Signal<T>` is easy. Just call the `.handle()` method.

Here is an example of a simple component that displays the value of its prop:

```rust
use sycamore::prelude::*;

#[component(MyComponent<G>)]
fn my_component(value: StateHandle<i32>) -> Template<G> {
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
