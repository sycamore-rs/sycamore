_2021-07-06_

# Announcing Sycamore v0.5.0!

_SSR + Routing_

Hello everybody! Sycamore is a library for building isomorphic web applications in Rust and
WebAssembly.

I'm happy to announce that we've just released v0.5.0. This is the biggest release yet, with loads of
new features and bug fixes.

(By the way, if you are looking for another library called "Maple", you found it. Maple is now
called Sycamore because Maple was already a trademarked name for another software product. The new
crate name is [`sycamore`](https://crates.io/crates/sycamore) on crates.io.).

## What's New?

### Server Side Rendering

Yep! That's right. Sycamore now supports server side rendering. A big shoutout to
[`@lights0123`](https://github.com/lights0123) for taking initiative to implement this feature in
[this Pull Request](https://github.com/sycamore-rs/sycamore/pull/67).

The hello world for SSR is just as simple for rendering to the DOM:

```rust
use sycamore::prelude::*;

fn main() {
    let string = sycamore::render_to_string(|| template! {
        p { "Hello, world!" }
    });
    println!("{}", string); // Prints <p>Hello, world!</p>
}
```

Just use `sycamore::render_to_string` instead of `sycamore::render` and you're good to go.

Check out the docs on [server side rendering](https://sycamore-rs.netlify.app/docs/advanced/ssr) for
more information.

### Routing

This release also introduces a full-featured routing system. Routing is provided by the
[`sycamore-router`](https://crates.io/crates/sycamore-router) crate. Creating a router is as easy as
pie!

```rust
use sycamore_router::Route;

#[derive(Route)]
enum MyRoutes {
    #[to("/")]
    Index,
    #[to("/about")]
    About,
    #[not_found]
    NotFound,
}
```

Just slap `#[derive(Route)]` on your enum and there you have it. Your very own router.

Check out the docs on [routing](https://sycamore-rs.netlify.app/docs/advanced/routing) to learn
more.

### New documentation website

The new documentation website is ready to roll. It is completely built with Sycamore. In fact, the
source code is available right here:
[github.com/sycamore-rs/sycamore/tree/master/docs](https://github.com/sycamore-rs/sycamore/tree/master/docs).

Check out the new documentation website at
[sycamore-rs.netlify.app](https://sycamore-rs.netlify.app).

### New Quick Start templates

We've also added three new templates for getting started with Sycamore.

- [`sycamore-trunk-gitpod-template`](https://github.com/sycamore-rs/sycamore-trunk-gitpod-template) -
  Template for creating a Sycamore Single Page Application (SPA).
- [`sycamore-rocket-minimal-template`](https://github.com/sycamore-rs/sycamore-rocket-minimal-template) -
  Bare bones template for building a Sycamore app with Rocket and server side rendering.
- [`sycamore-rocket-template`](https://github.com/sycamore-rs/sycamore-rocket-template) -
  SvelteKit-like started for creating a Sycamore isomorphic application.

## Conclusion

As always, a big thanks to all the
[contributors](https://github.com/sycamore-rs/sycamore/graphs/contributors) who made this release
possible! This would not have been possible without you.

For more detailed changes, check out the
[changelog](https://github.com/sycamore-rs/sycamore/blob/master/CHANGELOG.md#-050-2021-07-06).

If you are interested in contributing to Sycamore, check out the issues labeled with
[`good first issue`](https://github.com/sycamore-rs/sycamore/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)
on our GitHub repository. Don't hesitate to swing by our
[Discord server](https://discord.gg/vDwFUmm6mU) too! See you there.

Thanks!
