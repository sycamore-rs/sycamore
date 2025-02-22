---
title: Server Side Rendering
subsections:
  - streaming
---

# Server Side Rendering

Sycamore supports creating isomorphic web applications (apps that use the same
code to run on both the server and the client).

## Using `render_to_string`

So far, we've been using the `render` or `render_to` functions to mount our app
into the browser's DOM. When rendering on the server, however, we don't have a
DOM accessible to use so we use `render_to_string` instead.

`render_to_string` has the same API as `render` except it returns a string that
can be sent to the browser using your favorite web server.

```rust
let html = render_to_string(|| view! {
    div(class="my-class") {
        button { "Click me" }
    }
});

// Respond to the client with the rendered html.
```

## Hydration

Now that your app is rendered on the server and sent to the client as HTML, you
don't want the client to recreate all the DOM nodes when they are already there.
To _"hydrate"_ the app, use `hydrate` and `hydrate_to` instead of `render` and
`render_to` functions to mount your app. These functions do what they say on the
tin: render your app by reusing existing DOM nodes.

In your client-side app, enable the `"hydrate"` feature on `sycamore` in your
`Cargo.toml` file.
