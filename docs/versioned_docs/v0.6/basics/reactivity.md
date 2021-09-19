# Reactivity

Instead of relying on a Virtual DOM (VDOM), Sycamore uses fine-grained reactivity to keep the DOM
and state in sync. In fact, the reactivity part of Sycamore can be used on its own without the DOM
rendering part.

## Signal

Reactivity is based on reactive primitives. A `Signal` is data that is reactive. Here is an example:

```rust
use sycamore::prelude::*;
let state = Signal::new(0); // create an atom with an initial value of 0
```

Now, to access the state, we call the `.get()` method on `state` like this:

```rust
println!("The state is: {}", state.get()); // prints "The state is: 0"
```

To update the state, we call the `.set(...)` method on `state`:

```rust
state.set(1);
println!("The state is: {}", state.get()); // should now print "The state is: 1"
```

## Effects

Why would this be useful? It's useful because it provides a way to easily be notified of any state
changes. For example, say we wanted to print out every state change. This can easily be accomplished
like so:

```rust
let state = Signal::new(0);

create_effect(cloned!((state) => move || {
    println!("The state changed. New value: {}", state.get());
}));
// prints "The state changed. New value: 0"
// (note that the effect is always executed at least 1 regardless of state changes)

state.set(1); // prints "The state changed. New value: 1"
state.set(2); // prints "The state changed. New value: 2"
state.set(3); // prints "The state changed. New value: 3"
```

How does the `create_effect(...)` function know to execute the closure every time the state changes?
Calling `create_effect` creates a new _"reactivity scope"_ and calling `state.get()` inside this
scope adds itself as a _dependency_. Now, when `state.set(...)` is called, it automatically calls
all its _dependents_, in this case, `state` as it was called inside the closure.

> ### What's that `cloned!` macro doing?
>
> The `cloned!` macro is an utility macro for cloning the variables into the following expression.
> The previous `create_effect` function call could very well have been written as:
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
> This is ultimately just a workaround until something happens in
> [Rust RFC #2407](https://github.com/rust-lang/rfcs/issues/2407).

## Memos

We can also easily create a derived state using `create_memo(...)` which is really just an ergonomic
wrapper around `create_effect`:

```rust
let state = Signal::new(0);
let double = create_memo(cloned!((state) => move || *state.get() * 2));

assert_eq!(*double.get(), 0);

state.set(1);
assert_eq!(*double.get(), 2);
```

`create_memo(...)` automatically recomputes the derived value when any of its dependencies change.

Now that you understand `sycamore`'s reactivity system, we can look at how to use this to update the
DOM.

## Using reactivity with DOM updates

Reactivity is automatically built-in into the `template!` macro. Say we have the following code:

```rust
use sycamore::prelude::*;

let state = Signal::new(0);

let root = template! {
    p {
        (state.get())
    }
};
```

This will expand to something approximately like:

```rust
use sycamore::prelude::*;

let state = Signal::new(0);

let root = {
    let element = GenericNode::element(p);
    let text = GenericNode::text(String::new() /* placeholder */);
    create_effect(move || {
        // update text when state changes
        text.update_text(Some(&state.get()));
    });

    element.append(&text);

    element
};
```

If we call `state.set(...)` somewhere else in our code, the text content will automatically be
updated!

## Common pitfalls

Dependency tracking is *topological*, which means that reactive dependencies (like a `Signal`) must be accessed (and thus recorded as reactive dependencies) *before* the tracking scope (like a `create_effect`) returns.

For example, this code won't work as intended:

```rust
create_effect(move || {
    wasm_bindgen_futures::spawn_local(async move {
        // This scope is not tracked because spawn_local runs on the next microtask tick (in other words, some time later).
    };
    // Everything that is accessed until here is tracked. Once this closure returns, nothing is tracked.
});
```

We'll find that any `Signal`s we track in the `create_effect` won't be tracked properly in the `wasm_bindgen_futures::spawn_local`, which is often not what's intended. This problem can be gotten around by accessing reactive dependencies as needed before going into a future, or with this simple fix:

```rust
create_effect(move || {
	let _ = signal.get(); // Where `signal` is a reactive dependency
    wasm_bindgen_futures::spawn_local(async move {
        // This scope is not tracked because spawn_local runs on the next microtask tick (in other words, some time later).
    };
    // Everything that is accessed until here is tracked. Once this closure returns, nothing is tracked.
});
```

All we're doing there is accessing the dependency before we move into the future, which means dependency tracking should work as intended.
