---
title: Announcing Sycamore v0.8.0
description: Reactivity v2, better component props and children, async/await support, and more...
date: 2022-08-28
---

# Announcing Sycamore v0.8.0!

_Reactivity v2, better component props and children, async/await support, and more..._

Sycamore is a library for building reactive web apps in Rust and WebAssembly.

After **83** PRs and **8 months** in the making, the v0.8.0 release of Sycamore is no doubt the
largest so far! We also got the biggest community involvement in this release with **19** community
contributors to the codebase (compared to 10 for v0.7 and 8 for 0.6)!

For migrating from v0.7 to v0.8, check out the [migration guide](/docs/v0.8/migration/0.7-to-0.8)

## Improved reactivity

_Detailed blog post:
[A first look at Sycamore’s new reactive primitives](/news/new-reactive-primitives)_

The biggest change by far in v0.8 is the new reactivity system. This greatly improves developer
ergonomics by reducing cloning of variables into closures. However, this is a **major breaking
change** since reactivity is at the foundation of Sycamore and therefore nearly all code is
affected.

Below is presented a brief outline of the changes. For more details, read the blog post linked at
the top of this section and check out the section on [Reactivity](/docs/v0.8/basics/reactivity) in
the Sycamore book that has been updated to v0.8.

### No more clones!

The motivation for this change is mostly about ergonomics. Sycamore used to suffer from a problem of
cloning signals and other variables into event handlers and closures. This was because these
closures needed to be `'static` since we didn't know how long they needed to live.

In Sycamore v0.8, however, closures only need to live as long as the component/reactive scope. We
can therefore access local variables directly without any need for cloning!

### Explicit `cx` parameter

The reactive scope is no longer implicitly tracked through global variables but is now explicitly
tracked as a variable. In most cases, this means passing the `cx` variable to hooks and other
functions that need it.

```rust
// Old v0.7 syntax.
let signal = Signal::new(123);
create_effect({
    let signal = signal.clone();
    move || {
        let _ = signal.get();
    }
});
view! {
    div {}
}

// New v0.8 syntax.
let signal = create_signal(cx, 123);
create_effect(cx, || {
    let _ = signal.get();
});
view! { cx,
    div {}
}
```

### Signals are owned by the reactive scope.

The new `create_signal` function returns a `&Signal` instead of a `Signal`. This is because the
underlying signal value is owned by the reactive scope (`cx`). This means that signals can no longer
escape the reactive scope in which it was created.

To create a signal that is "detached" from the reactive scope, use a `RcSignal` instead.

### Obtaining the reactive scope

The `cx` variable is provided by the `sycamore::render` function as the first parameter to the
closure.

```rust
sycamore::render(|cx| ...);
```

It can also be accessed from a component.

```rust
#[component]
fn MyComponent<G: Html>(cx: Scope, props: MyProps) -> View<G> { ... }
```

## Component improvements

### `#[derive(Prop)]` and default and optional props

Component props should now have `#[derive(Prop)]` on them. Currently, this can only be done for
`struct`s. If the prop does not derive `Prop`, the features described below won't be available.

Props fields can now be set to their default value by adding the `#[builder(default)]` attribute.

```rust
#[derive(Prop)]
struct MyProps {
    #[builder(default)]
    count: i64,
}
```

Since an `Option<T>` value defaults to `None`, we can use this mechanism for optional props as well!
However, we don't want to pass all our props wrapped in `Some`. Therefore, we can add the
`#[builder(default, setter(strip_option))]` attribute instead.

```rust
#[derive(Prop)]
struct MyProps {
    #[builder(default, setter(strip_option))]
    email: Option<String>,
}
```

Under the hood, the `Prop` macro uses the excellent
[`typed-builder`](https://crates.io/crates/typed-builder) crate.

### Component children

Components can now accept children. To do so, add the `children` field of type `Children<'a, G>` to
the prop struct.

```rust
#[derive(Prop)]
struct MyProps<'a, G: Html> {
    children: Children<'a, G>
}

#[component]
fn MyComponent<'a, G: Html>(cx: Scope<'a>, props: MyProps<'a, G>) -> View<G> { ... }

view! { cx,
    MyComponent {
        p { "Children" }
    }
}
```

To access the children as a `View` from inside the component, use `props.children.call(cx)`.

## `async`/`await`

Sycamore v0.8 adds first-class support for `async`/`await`.

### Async components

The first part of this story is _async components_. Since components are essentially just functions,
adding the `async` keyword will automatically make your component an async component. From within
the async component, you can call other async functions and `.await` them just like you expect!

```rust
async fn fetch_data() -> Data { ... }

#[component]
async fn DataDisplayer<G: Html>(cx: Scope) -> View<G> {
    let data = fetch_data().await;
    view! { cx, ... }
}
```

When the async component is used, a blank view is returned. When the component's future resolves,
the view is immediately replaced with the returned view.

This makes is substantially easier to perform data-fetching and other async tasks with Sycamore.

### Suspense

The second part of this story is `Suspense`. `Suspense` is a component that allows you to show a
loading indicator while async components do their work. As long as the async component is somewhere
in the `Suspense` in the node hierarchy, the loading indicator will be shown, no matter how deep the
async component is away from the root.

```rust
view! { cx,
    Suspense(fallback=view! { cx, "Loading..." }) {
        DataDisplayer {}
    }
}
```

## Type checked element tags and completion support for `view!`

HTML element tags are now type checked so that it is impossible to create an invalid HTML element.

```rust
view! { cx,
    div {} // OK.
    notvalid {} // Compile time error.
}
```

As an added benefit, we also get completions (intellisense) support for HTML tags in your code
editor.

![intellisense](https://user-images.githubusercontent.com/37006668/187043672-b6e66740-e6d4-4a02-95c6-4fb9cb6f8654.png)

## New builder API syntax

The old builder API needed a `.build()` for every node. The new builder API does this automatically
behind the hood. The syntax has also been changed to be more concise and ergonomic. Check out the
[book](/docs/v0.8/basics/view#builder-syntax) for more information.

## More web utilities

- `on_mount`: Queue up a callback to be executed once the component has been mounted (i.e. attached
  to the DOM).
- `NoHydrate` and `NoSsr`: Prevent a part of the view from being hydrated or from being rendered on
  the server.

## Hydration fixes

Although Sycamore has supported Server Side Rendering (SSR) since v0.5, client-side hydration has
always been buggy and unreliable. Sycamore v0.8 fixes many of these hydration bugs. Hopefully,
hydration will now work out of the box for your use-case.

## Community and Ecosystem

Sycamore's community has grown a lot since the last release. We now have over 350 members on our
[Discord server](https://discord.gg/vDwFUmm6mU). We also recently achieved the 1k stars milestone on
GitHub.

### awesome-sycamore

<https://github.com/sycamore-rs/awesome-sycamore>

We now have an
<img alt="awesome" src="https://camo.githubusercontent.com/64f8905651212a80869afbecbf0a9c52a5d1e70beab750dea40a994fa9a9f3c6/68747470733a2f2f617765736f6d652e72652f62616467652e737667" style="display: inline-block" />
list for all things related to Sycamore! If you have a project you wish to share, please feel free
to send a PR our way.

### Sycamore Playground

<https://sycamore-rs.github.io/playground>

It is now possible to try out Sycamore directly in your web browser! Use the Sycamore Playground to
test out snippets, reproduce bugs, and share code with others!

## Conclusion

A big thank you to the
[Sycamore contributors](https://github.com/sycamore-rs/sycamore/graphs/contributors) for making this
release possible!

For more detailed changes, check out the
[changelog](https://github.com/sycamore-rs/sycamore/blob/master/CHANGELOG.md#-080-2022-08-28).
