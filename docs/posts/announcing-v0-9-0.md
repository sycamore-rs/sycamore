---
title: Announcing Sycamore v0.9.0
description:
  Reactivity v3, View v2, resources API and suspense, SSR streaming, attributes
  passthrough, new website, and more!
date: 2024-10-24
---

# Announcing Sycamore v0.9.0

I'm happy to announce the release of Sycamore v0.9.0!

Sycamore is a reactive Rust UI framework for building web apps using
WebAssembly. This release is by far the biggest release we've ever had, with
tons of new features and improvements. If you have not used Sycamore before,
here's a quick sample:

```rust
#[component]
fn Counter(initial: i32) -> View {
    let mut value = create_signal(initial);

    view! {
        button(on:click=move |_| value += 1) {
            "Count: " (value)
        }
    }

}
```

Sycamore's community has also grown a lot since the v0.8.0 release. We've gone
from just over **1.0k** stars to **2.8k** stars on GitHub. What used to be just
over **350** discord members has now grown to **626**! We've also reached
**151k** downloads on [crates.io](https://crates.io/crates/sycamore).

For migrating over from v0.8, check out the
[migration guide](/book/migration/0-8-to-0-9)

## A shinny new website

We now have a shinny new website along with a shinny new domain:
[sycamore.dev](https://sycamore.dev)! This was long overdue. We were previously
using a Netlify subdomain so this change hopefully makes Sycamore look more
legitimate. The old website had a bunch of issues such as buggy navigation, no
server side rendering support, and an awkward layout. This new website redesign
fixes many of those things. The old docs are still available at the old website
but the index page will now automatically redirect to the new website.

A lot of the content has also been rewritten and updated for this new version of
Sycamore. This includes a brand new "Introduction" section which helps guide you
through creating your first Sycamore app, a simple Todo manager. This introduces
various topics such as the view macro, the basics of reactivity, and how
rendering lists work. This will hopefully help new users interested in Sycamore
to get started with the main concepts.

Here are a few comparaisons between the old and new website.

<style>
figure img,video {
    border: 1px black;
    border-style: solid;
    border-radius: 5px;
}
figure video {
    margin-bottom: 0 !important;
}
figure figcaption {
    text-align: center;
    margin-top: 0.4em !important;
}
</style>

<figure>
    <img src="https://github.com/user-attachments/assets/b4c2d894-2ea0-41d7-b170-ce6eb7865ef9" alt="old homepage" />
    <figcaption>The old homepage</figcaption>
</figure>

<figure>
    <img src="https://github.com/user-attachments/assets/453d9510-d9ab-4796-91d7-09f8ae9cf9ef" alt="new homepage" />
    <figcaption>The new homepage</figcaption>
</figure>

<figure>
    <img src="https://github.com/user-attachments/assets/daa6f658-f250-4228-8bc3-8b8eefd6b3aa" alt="old docs" />
    <figcaption>The old docs</figcaption>
</figure>

<figure>
    <img src="https://github.com/user-attachments/assets/6a50cfd2-3e2f-4a8e-9507-ec99725c964b" alt="new docs" />
    <figcaption>The new docs</figcaption>
</figure>

There are still currently a few sections of the docs that needs writting or
simply needs a few more details. You can help us out by contributing to the
docs! Simply go to the relevant page and click on "Edit this page on GitHub" at
the bottom and send us a Pull Request.

## Reactivity v3

What is probably the biggest new feature of this release is our new reactivity
system, dubbed **Reactivity v3**! In Reactivity v2 (introduced in the
[v0.8](/post/announcing-v0-8-0) release), we eliminated the need for cloning
signals and other reactive primitives into closures. This, however, came at the
expense of introducing lifetimes for tracking whether a signal was alive and
could be accessed.

Lifetimes are well known to add complexity to a Rust codebase. So although we no
longer needed to deal with cloning, we now needed to deal with lifetimes.
Reactivity v3 fixes all this. We made all signals and other reactive datatypes
both `'static` and `Copy`-able. This way, you get both the benefit of passing
signals wherever you want without littering your codebase with `.clone()`
everywhere, all without having to worry about lifetimes. Along the way, we also
eliminated the need for the `cx` parameter as well!

Whereas previously, you might have written:

```rust
let signal = create_signal(cx, 123);
create_effect_scoped(cx, |cx| {
    let nested = create_signal(cx, 456);
    println!("{signal}, {nested}");
});
```

Now, you can simply write:

```rust
let signal = create_signal(123);
create_effect(move || {
    let nested = create_signal(456);
    println!("{signal}, {nested}");
});
```

Although a very contrived example, hopefully this demonstrates that the new
reactivity system is much more simple and intuitive. We no longer need to thread
the `cx` parameter everywhere, we no longer have to worry about scoped versus
non-scoped effects, and we can pass signals wherever we want without infecting
everything with lifetimes.

Under the hood, this involved a
[huge rewrite](https://github.com/sycamore-rs/sycamore/pull/612) of essentially
the entire `sycamore-reactive` crate from scratch. The new implementation uses a
singleton `Root` datatype for managing the reactive graph instead of a bunch of
smart pointers everywhere in a tangled mess. This should hopefully make the
implementation more robust and reliable.

## View v2

Another major change coming to Sycamore v0.9 is **View v2**. Reactivity v3
removed a lot of friction and boilerplate when interacting with reactive state.
View v2 continues this theme and removes _a bunch_ of boilerplate from
components and views.

The biggest change is the complete removal of the `GenericNode` and `Html`
traits which have been infesting Sycamore codebases ever since we introduced SSR
(server side rendering) support all the way back in v0.5.

Witness the difference yourself. Here is Sycamore v0.8 code:

```rust
#[component(inline_props)]
fn Component<'a, G: Html>(cx: Scope<'a>, value: &'a ReadSignal<i32>) -> View<G> {
    ...
}
```

There is a bunch of noise here that is distracting from what this component
does, such as the `'a` lifetime and the `G: Html` generic parameter. Reactivity
v3 and View v2 together turns this into:

```rust
#[component(inline_props)]
fn Component(value: ReadSignal<i32>) -> View {
    ...
}
```

Doesn't this just look so much better?

### New builder API

This refactor also introduces a new builder API. Apologies to all the churn the
builder API has received over the past few releases, but I really think this new
API is much better than before. For a long time, the builder API was always a
second-class citizen compared to the macro. This is no more. In fact, the
`view!` macro has been refactored to simply codegen the builder API behind the
hood, making the builder API a true first-class citizen in Sycamore. Here is
what it looks like:

```rust
div().class("hello-world").children((
    span().style("color: red").children("Hello "),
    em().children("World!"),
))
```

For more information, check out the [builder API](/book/guide/view-builder) docs
in the book.

### Type-checked HTML attributes

Since we are now using the builder API as the codegen target for the view macro,
we also get type-checked and auto-completed HTML attributes!

<figure>
    <img src="https://github.com/user-attachments/assets/a1fc6f21-c046-46a2-9d45-7e74d06fe19d" alt="lsp hover for attributes" />
    <figcaption>Documentation for attributes, provided by Rust-Analyzer in VSCode</figcaption>
</figure>

This also means no more silly typos causing hard to spot bugs, and finally,
proper support for boolean and optional attributes.

## Attribute passthrough

Suppose you're writing a component library and are creating a `Button`
component. Which props should you component accept? Ideally, you want your
component to be as flexible as possible so you should try to provide as many
HTML attributes as you can. This quickly becomes tedious: you'll need to provide
`class`, `id`, `disabled`, `r#type`, `value`, etc. Furthermore, HTML allows
arbitrary custom attributes of the form `data-*` as well as a bunch of
accessibility attributes like `aria-*`, making this task essentially impossible.

Enter **attribute passthrough**. This allows your component to behave as if it
were an HTML element, accepting HTML attributes, and letting you forward all of
these attributes onto the element itself. Here's an example:

```rust
#[component(inline_props)]
fn Button(
    #[prop(attributes(html, button))]
    attributes: Attributes,
    children: Children,
    accent: StringAttribute,
) -> View {
    view! {
        // Spread the attributes onto the wrapped element.
        button(..attributes) {
            (children)
        }
    }
}

// Now use your component just as if it were a normal HTML element.
view! {
    Button(
        class="btn btn-red",
        id="login-button",
        on:click=move |_| login(),
        // `accent` is passed as a prop, not as an attribute.
        accent="primary",
    ) {
        "Login"
    }
}
```

To learn more, read the section on
[Attribute passthrough](/book/guide/attribute-passthrough) in the book.

## Resources

Sycamore v0.9 introduces the Resources API. Resources let you load asynchronous
data into your app, in a way that is tightly coupled with the reactivity system
and suspense.

Resources are essentially asynchronous nodes in the reactive graph. This means
that resources can depend on reactive values. For instance, this will refetch
the resource whenever the `id` signal is updated.

```rust
let id = create_signal(...);
let resource = create_resource(on(id, move || async move {
    fetch_user(id.get()).await
}));
```

You can then use the resource value like so:

```rust
view! {
    (if let Some(data) = resource.get_clone() {
        view! {
            ...
        }
    } else {
        view! {}
    })
}
```

Accessing the value will automatically trigger suspense, letting you easily
define loading screens etc. To learn more, read the section on
[Resources](/book/guide/resources-and-suspense) in the book.

## SSR streaming

We've had support for server side rendering (SSR) for quite a while now. This
release, however, introduces SSR streaming. What is SSR streaming?

Let's first look at how normal server side rendering works. If we don't fetch
any asynchronous data, everything is simple: just render the app in one shot on
the server and send it over to the client. If, however, we do have asynchronous
data, we have a few choices. We might choose not to do any data-fetching on the
server and instead just send the loading fallback. We can then do all the
data-fetching client side. This approach has a major disadvantage. When we make
the request to the server, we already, _in principle_, know all the asynchronous
data that needs to be fetched. The client, however, can not know this until the
WASM binary has been sent over, loaded, and the app hydrated. So we are wasting
a lot of time where we could have been fetching these asynchronous resources in
parallel.

Another approach would be to load all the data on the server-side and wait for
all loading to complete before sending the HTMl over to the client. Such an
approach, however, causes an annoying delay on the client where nothing is
displayed while the data is loading.

SSR streaming strikes a balance between these two approaches. First, an initial
HTML shell is sent over to the client displaying the fallback view, such as
loading text or spinners. Then as the data is fetched on the server, the new
view is rendered and subsequently _streamed_ over to the client over the same
HTTP request. This new view is then dynamically inserted into the right position
in the DOM.

<figure>
    <video controls>
        <source src="https://github.com/user-attachments/assets/27081b18-637a-49f7-9ee7-0c9644a523c8" type="video/mp4" />
    </video>
    <figcaption>SSR Streaming Demo</figcaption>
</figure>

SSR streaming offers the best of both worlds. The client displays something
right away, and data is fetched as soon as possible on the server and the result
streamed over to the client.

This feature is seamlessly integrated with Suspense. The natural streaming
boundaries are the suspense boundaries, so that the suspense fallback is sent
first, and then when the suspense resolves, the suspense content is streamed
over.

Learn more by reading the [SSR Streaming](/book/server-side-rendering/streaming)
section of the book.

## The future of Sycamore
