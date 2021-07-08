# Server Side Rendering

Sycamore supports creating isomorphic web applications (apps that use the same code to run on both
the server and the client).

## `render_to_string`

So far, we've been using the `render` or `render_to` functions to mount our app into the browser's
DOM. When rendering on the server, however, we don't have a DOM accessible to use so we use
`render_to_string` instead.

`render_to_string` has the same API as `render` except it returns a string that can be sent to the
browser using your favorite web server.

```rust
let node = template! {
    div(class="my-class") {
        button { "Click me" }
    }
}
let html = render_to_string(|| node);

// Prints: <div class="my-class"><button>Click me</button></div>
println!("{}", html);
```

Note that you will need to enable the `"ssr"` feature on `sycamore` in your `Cargo.toml` file.

## Hydration

Sycamore currently implements a very "naive" method of hydration. The current `hydrate` and
`hydrate_to` methods merely recreate the entire DOM tree and replaces the old one sent from the
server.

This still retains many benefits of SSR. The initial load time will still be faster and crawlers
will be able to see markup without executing anything.

Once proper hydration is implemented, time to interactive will be improved.

## Quick Start Templates

- [`sycamore-rocket-template`](https://github.com/sycamore-rs/sycamore-rocket-template): A quick
  start template for using Sycamore with Rocket. Batteries included with `sycamore-router`.
- [`sycamore-rocket-minimal-template`](https://github.com/sycamore-rs/sycamore-rocket-minimal-template):
  A minimal template for using Sycamore with Rocket.
