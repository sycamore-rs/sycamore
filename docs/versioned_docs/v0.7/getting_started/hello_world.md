# Hello, World!

Sycamore tries to have as simple of an API as possible.

Here it is:

```rust
// main.rs

use sycamore::prelude::*;

fn main() {
    sycamore::render(|| view! {
        p { "Hello, World!" }
    });
}
```

Let's dissect what the code above was doing.

```rust
fn main() {
```

Nothing really special here. Trunk automatically uses `fn main` as your project's entrypoint. No
need for any `#[wasm_bindgen(start)]` here.

```rust
sycamore::render(...)
```

This function is provided by Sycamore and is used to render your app to the DOM (browser window).
`render` accepts a closure (aka. lambda function) which should return a view to be rendered.

```rust
view! {
    p { "Hello, World!" }
}
```

The `view!` macro allows creating complex user interfaces ergonomically in HTML. In this case,
we want to render the following HTML:

```html
<p>Hello World!</p>
```

The `p { ... }` creates a new `<p>` tag. The `"Hello, World!"` creates a new text node that is
nested within the `<p>` tag.

There it is! Trunk just needs one thing to turn this into a website; a html source file to inject
the view into. Copy the following code to a file called `index.html` in the root of your crate
(alongside `Cargo.toml`):

```html
<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>My first Sycamore app</title>
  </head>
  <body></body>
</html>
```

To try it out, copy the Hello World code snippet to your `main.rs` file and run `trunk serve` from
your command prompt. Open up your browser at `localhost:8080` and you should see _"Hello, World!"_
printed to the screen in all its glory.

If you modify your code, Trunk should automatically rebuild your app. Just refresh your browser tab
to see the latest changes.

## Quick Start Templates

- [`sycamore-trunk-gitpod-template`](https://github.com/sycamore-rs/sycamore-trunk-gitpod-template):
  A quick start view for building a Sycamore single-page app. Comes with support for using
  [gitpod.io](https://www.gitpod.io).
