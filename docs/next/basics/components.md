# Components

Components in `sycamore` are structs that implement `Component`. A component can automatically be
created with the `#[component(ComponentName<G>)]` attribute on a function.

Components receive their props through function arguments.

For components to automatically react to prop changes, they should accept a prop with type
`StateHandle<T>` and call the function in the `view!` to subscribe to the state. A
`StateHandle<T>` is just a readonly `Signal<T>`.

Getting a `StateHandle<T>` from a `Signal<T>` is easy. Just call the `.handle()` method.

Here is an example of a simple component that displays the value of its prop:

```rust
use sycamore::prelude::*;

#[component(MyComponent<G>)]
fn my_component(value: StateHandle<i32>) -> View<G> {
    view! {
        div(class="my-component") {
            "Value: " (value.get())
        }
    }
}

// ...
let state = Signal::new(0);

view! {
    MyComponent(state.handle())
}

state.set(1); // automatically updates value in Component
```

## Props

Components that don't accept props (same as accepting `()`) can omit the function argument.

```rust
use sycamore::prelude::*;

#[component(MyComponent<G>)]
fn my_component() -> View<G> {
    ...
}
```

## Lifecycle

Component lifecycle is strongly tied to the reactive system. Under the hood, components are simply
functions that are run inside an untracked scope. Component functions only run once (unlike React
where functional-components are called on every render). Think of it as a builder-pattern for
constructing UI.

This means that we can use the same helpers in the reactive system to attach callbacks to the
component lifecycle.

### `on_cleanup`

```rust
use sycamore::prelude::*;

#[component(MyComponent<G>)]
fn my_component() -> View<G> {
    on_cleanup(move || {
        // Perform cleanup.
    });
    ...
}
```
