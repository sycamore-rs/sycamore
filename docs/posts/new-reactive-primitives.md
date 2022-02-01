_2022-02-01_

# A first look at Sycamore's new reactive primitives

_How the next version of Sycamore will be the most ergonomic yet_

Sycamore is a library for building isomorphic web applications in Rust and WebAssembly.

At its core, Sycamore is built on what are called "reactive primitives". These reactive primitives
are simply wrappers that can keep track of state changes and notify you about them. The most common
primitive is `Signal`.

Because everything in Sycamore is tied into its reactive system, this allows us to use a simpler and
easier model for describing and rendering components. Unlike React hooks where functional components
are re-run each time the state changes, components in Sycamore are only run once. The functions for
creating components are more like component builders in that it describes the structure of the UI
and reactivity does the rest.

What this means is that instead of the whole component function re-running, only the data that need
to be re-computed are re-run. We can achieve this with Rust's closures. And that's the main problem
that was limiting the ergonomics of Sycamore. `Signal` was `Clone`able but not `Copy`able. This
meant that each time a `Signal` was used within a closure it needed to be cloned first, then moved
into it.

```rust
let data = Signal::new(...);
let callback = {
    let data = data.clone();
    move || {
        // Do something useful with `data`
    }
}
```

As a workaround, we introduced the `cloned!` macro to make it a bit less boilerplate-y, but it still
wasn't the best.

```rust
let data = Signal::new(...);
let callback = cloned!(data => move || {
    // Do something useful with `data`
})
```

And it gets worse. In Sycamore, the `view!` macro is used to describe the UI, similarly to JSX in
React. The issue with this was that dynamic data in the `view!` macro needed to be wrapped inside a
closure that moved the signal in. That meant that the following code wouldn't work.

```rust
let data = Signal::new(...);
view! {
    p { (data.get()) }
    //   ^^^^ -> `data` is moved into a closure here
    p { (data.get()) }
    //   ^^^^ -> ERROR: `data` already moved
}
```

To workaround this issue, we had to resort to the ugly hack of creating another variable with a
different name outside the macro call.

```rust
let data = Signal::new(...);
let data_cloned = data.clone();
view! {
    p { (data.get()) }
    //   ^^^^ -> `data` is moved into a closure here
    p { (data_cloned.get()) }
    //   ^^^^ -> Ok. We are using `data_cloned`, not `data`.
}
```

That was the problem. It was like the thorns on the rose bush, or like a stone in the shoe.
Reactivity was great on paper, but when implemented in Rust, had a few unexpected gotchas.

In the next version of Sycamore (v0.8), this issue is now fixed! The jest of the issue was that the
closures needed to be `'static` because `Signal`s were internally reference counted and thus there
was no way to tell how long a `Signal` lived. Yet, most of the times, the lifetimes of signals were
stacked like a tower, with the longest living `Signal` at the bottom and the shortest at the top.
This was the perfect opportunity to take advantage of Rust's borrow checker!

With the new reactive primitives, `Signal`s are no longer reference-counted by default but are
instead tied to the lifetime of the reactive scope in which it is created.

```rust
// Before:
let data = Signal::new(...);
// After:
let data = ctx.create_signal(...);
```

The `ctx` is a reference to the current reactive scope. Whereas previously, reactive scopes were
internally tracked using a complicated orchestration of thread-locals, reactive scopes are now
explicitly represented by the `Scope` type. This change was necessary because otherwise, there would
be no way to associate the `Signal` to the `Scope`.

Now, how is this an improvement? It is in the return type of `Signal::new` and `ctx.create_signal`.
`Signal::new` returned, well, a `Signal` (which, if you remember, was `Clone`able but not
`Copy`able) but `ctx.create_signal` returns a `&Signal` (a reference to a `Signal`, which is always
`Copy`able). The way this works is that the `Scope` acts somewhat akin to an
[arena allocator](https://en.wikipedia.org/wiki/Region-based_memory_management). `Signal`s that are
created on a `Scope` are allocated in an internal allocator, thus making the `Signal` share the same
lifetime as the `Scope`.

This means that we can now use our `Signal` in as many closures as we want.

```rust
let data = ctx.create_signal(...);
let callback = || data.get();
//             ^^ -> Look ma, no clones!
let another_callback = || data.get();
ctx.create_effect(|| {
    log::info!("{data}");
});
```

Making reactive scopes explicit also allows another exciting possibility: first-class
`async`/`await` support directly inside components! The reason this wasn't possible before was
because using `async` broke the topological code execution upon which relied the global thread-local
solution. In other words, after a `.await` suspension point, we could no longer know what reactive
scope we were in. Now that we can access `ctx` directly, that makes writing the following code a
possibility:

```rust
#[component]
async fn AsyncFetch<G: Html>(ctx: ScopeRef) -> View<G> {
    let data = fetch_data().await;
    let derived = ctx.create_memo(|| data);
    //            ^^^ -> We can still access `ctx`, even after the `.await` suspension point.
    view! {
        (derived)
    }
}
```

It's probably as ergonomic as it can get when it comes to suspense and async in UI frameworks.

Although this is a dramatic improvement for readability and ergonomics when using Sycamore, this new
approach does have a few disadvantages.

The first is due to the nature of arena allocators. Arena allocators only free their memory all at
once when they are destroyed. There is no deallocation while the arena allocator is still valid.
This means that `Signal`s _must_ live as long as the `Scope`, no longer and no shorter. This means
that one must be more careful in preventing leaking memory, for example, by not using
`ctx.create_signal` in a loop or in an effect where it might be called multiple times.

In the pretty rare cases where something like this is necessary, it is still possible to use a
reference-counted `Signal`, an `RcSignal` which is pretty much identical to the old `Signal`.

The second is now that `Signal`s are tied to the `Scope`, it is impossible for the `Signal` to
"escape" the `Scope`. For example, the following code won't compile.

```rust
let mut outer = None;
// Crete a new reactive scope and allow access to it through `ctx`.
create_scope(|ctx| {
    let data = ctx.create_signal(0);
    outer = Some(data);
    //           ^^^^ -> ERROR: `data` cannot escape
});
```

However, this behavior is rarely desired. And when absolutely needed, one can always resort back to
an `RcSignal` just like before.

And this wraps up Sycamore's new reactive primitives. To try them out, install the `v0.8` beta
published on crates.io (as of writing, the latest beta is `v0.8.0-beta.1`). Note that there are
quite a few breaking changes, not just with reactivity, but also with some other aspects of Sycamore
(such as components) to make them compatible with the new reactivity system.
