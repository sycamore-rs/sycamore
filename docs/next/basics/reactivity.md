# Reactivity

Reactivity is at the heart of Sycamore.

Instead of relying on a Virtual DOM (VDOM), Sycamore uses fine-grained reactivity to keep the DOM
and state in sync. In fact, Sycamore's reactivity system can be used on its own without pulling in
all the DOM rendering part. It just turns out that fine-grained reactivity and UI rendering are a
great match which is the whole point of Sycamore.

## Reactive scopes

Whenever reactivity is used, there must be a reactive scope. Such a scope is provided by functions
such as `sycamore::render` and `sycamore::render_to` as an argument to the render closure.

```rust
sycamore::render(|cx| {
    // `cx` is the reactive scope.
});
```

From this point on, we assume all code, unless otherwise specified, is run inside such a scope so
that it can access `cx`.

## Signal

Reactivity is based on reactive primitives. A `Signal` is one such example of a reactive primitive.
At it's simplest, a `Signal` is simply a wrapper around a type that can be read and written to and
which can be listened on whenever its wrapped value is mutated.

To create a signal, we use `create_signal(cx, ...)`. 
Here is an example of creating a signal, accessing it via `.get()`, and modifying it via
`.set(...)`.

```rust
let state = create_signal(cx, 0); // Create a reactive atom with an initial value of `0`.
println!("The state is: {}", state.get()); // prints "The state is: 0"
state.set(1);
println!("The state is: {}", state.get()); // should now print "The state is: 1"
```

## Effects

We mentioned earlier that signals can be listened on to tell us whenever its value has changed.
Let's do that! For example, imagine we wanted to print out every state change. This can easily be
accomplished like so:

```rust
let state = create_signal(cx, 0);
create_effect(cx, move || println!("The state changed. New value: {}", state.get()));
// Prints "The state changed. New value: 0"
// (note that the effect is always executed at least 1 regardless of state changes)

state.set(1); // Prints "The state changed. New value: 1"
state.set(2); // Prints "The state changed. New value: 2"
state.set(3); // Prints "The state changed. New value: 3"
```

How does the `create_effect(...)` function know to execute the closure every time the state changes?
Calling `create_effect` creates a new _"listener scope"_ (not to be confused with reactive scope)
and calling `state.get()` inside this listener scope adds itself as a _dependency_. Now, when
`state.set(...)` is called, it automatically calls all its _dependents_. In this case, whenever
`state` is updated, the new value will be printed!

## Memos

Sure, effects are nice but Rust is a multi-paradigm language, not just an imperative language. Let's
take advantage of the more functional side of Rust!

In fact, we can easily create a derived state (also known as derive stores) using `create_memo(...)`.

```rust
let state = create_signal(cx, 0);
let double = create_memo(cx, || state.get() * 2);

assert_eq!(double.get(), 0);
state.set(1);
assert_eq!(double.get(), 2);
```

`create_memo(...)` automatically recomputes the derived value when any of its dependencies change.

Now that you understand the basics of Sycamore's reactivity system, we can take a look at how this
is used together with UI rendering.

## Using reactivity with DOM updates

Reactivity is automatically built-in into the `view!` macro. Say we have the following code:

```rust
let state = create_signal(cx, 0);
view! { cx,
    p {
        (state.get())
    }
}
```

This will expand to something approximately like:

```rust
let state = create_signal(cx, 0);
{
    let element = G::element(p);
    let text = G::text(String::new() /* placeholder */);
    create_effect(cx, move || {
        // Update text when `state` changes.
        text.update_text(Some(&state.get(),to_string()));
    });
    element.append(&text);
    element
}
```

If we call `state.set(...)` somewhere else in our code, the text content will automatically be
updated!

## Common pitfalls

Dependency tracking is _topological_, which means that reactive dependencies (like a `Signal`) must
be accessed (and thus recorded as reactive dependencies) _before_ the listener scope (like the one
in a `create_effect`) returns.

For example, code inside the `spawn_local` won't be tracked:

```rust
create_effect(cx, move || {
    wasm_bindgen_futures::spawn_local(async move {
        // This scope is not tracked because spawn_local runs on the
        // next microtask tick once the effect closure has returned already.
    };
    // Everything that is accessed until here is tracked.
    // Once this closure returns, nothing is tracked.
});
```

We'll find that any `Signal`s we track in the `create_effect` won't be tracked properly in the
`wasm_bindgen_futures::spawn_local`, which is often not what's intended. This problem can be gotten
around by accessing reactive dependencies as needed before going into a future, or with this simple
fix:

```rust
create_effect(cx, move || {
    signal.track(); // Same as calling `.get()` but without returning a value.
    wasm_bindgen_futures::spawn_local(async move {
        // This scope is not tracked because spawn_local runs on the next microtask tick (in other words, some time later).
    };
    // Everything that is accessed until here is tracked. Once this closure returns, nothing is tracked.
});
```

All we're doing there is accessing the dependency before we move into the future, which means
dependency tracking should work as intended.
