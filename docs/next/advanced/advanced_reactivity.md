# Advanced Reactivity

## Contexts

Contexts provide an easy way to share data between components without drilling props through
multiple levels of the component hierarchy.

Creating a `ContextProvider` is required before any components can use the context. The value used
should implement `Clone`.

### Using `ContextProvider`

`ContextProvider` is a component like any other. It takes a `value` prop which is the context value
and a `children` prop which is the child components that have access to the context value.

### Using `use_context`

`use_context` returns a clone of the value for a context of a given type.

### Example

```rust
use sycamore::prelude::*;
use sycamore::context::{ContextProvider, ContextProviderProps, use_context};

#[derive(Clone)]
struct Counter(Signal<i32>);

#[component(CounterView<G>)]
fn counter_view() -> Template<G> {
    let counter = use_context::<Counter>();

    template! {
        (counter.0.get())
    }
}

template! {
    ContextProvider(ContextProviderProps {
        value: Counter(Signal::new(0)),
        children: || template! {
            CounterView()
        }
    })
}
```

Remember that unlike contexts in React and many other libraries, the `value` prop is not reactive by
itself. This is because components only run once. In order to make a context value reactive, you
need to use a `Signal` or other reactive data structure.

## Reactive scopes

### on_cleanup

### Nested effects

TODO

Help us out by writing the docs and sending us a PR!
