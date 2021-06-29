# Routing

Routers are the backbone of SPAs (Single Page Apps). They handle displaying different pages
depending on the URL. When an anchor tag (`<a>`) is clicked, the router will intercept it and
navigate to the correct page without performing a full refresh. This makes navigation feel faster
and smoother.

## Adding `sycamore-router`

To add routing to your Sycamore app, install the
[`sycamore-router`](https://crates.io/crates/sycamore-router) crate from crates.io.

```toml
sycamore-router = "0.5.0-beta.1"
```

### Compatibility with `sycamore`

Note that the major version
number for `sycamore-router` corresponds to the same major version number for `sycamore` (e.g.
`sycamore-router v0.5.x` is compatible with `sycamore v0.5.x`).

## How to use `sycamore-router`

Start off by adding `use sycamore_router::{BrowserRouter, Route}` to the top of your source code.
This imports the symbols needed to define our router.

### Creating routes

The heart of the router is an `enum`. Each variant of the `enum` represents a different route.
To make our `enum` usable with `BrowserRouter`, we will use the `Route` derive macro to implement
the required traits for us.

Here is an example:

```rust
#[derive(Route)]
enum AppRoutes {
    #[to("/")]
    Index,
    #[to("/about")]
    About,
    #[not_found]
    NotFound,
}
```

Note that each variant is marked with either the `#[to(_)]` or `#[not_found]` attribute.

The `#[to(_)]` attribute designates a route. For example, `#[to("/about")]` designates the route
for the about page.

The `#[not_found]` is a fallback route. It is the route that matches when all the other routes
don't. There must be one, and only one route marked with `#[not_found]`. Forgetting the not found
route will cause a compile error.

### Using `BrowserRouter`

To display content based on the route that matches, we can use a `BrowserRouter`.

```rust
template! {
    BrowserRouter(|route: AppRoutes| {
        match route {
            AppRoutes::Index => template! {
                "This is the index page"
            },
            AppRoutes::About => template! {
                "About this website"
            },
            AppRoutes::NotFound => template! {
                "404 Not Found"
            },
        }
    })
}
```

`BrowserRouter` is just a component like any other. The props accept a closure taking the matched
route as a parameter. Any clicks on anchor tags (`<a>`) created inside the `BrowserRouter` will be
intercepted and handled by the router.

### Using `StaticRouter`

Whereas `BrowserRouter` is used inside the context of a browser, `StaticRouter` is used for SSR.

The difference between a `BrowserRouter` and a `StaticRouter` is that the url is provided to
`StaticRouter` only during the initialization phase. The initial url is provided as an argument
to `StaticRouter`.

```rust
use sycamore_router::{Route, StaticRouter};

template! {
    StaticRouter(("/about", |route: AppRoutes| {
        match route {
            AppRoutes::Index => template! {
                "This is the index page"
            },
            AppRoutes::About => template! {
                "About this website"
            },
            AppRoutes::NotFound => template! {
                "404 Not Found"
            },
        }
    }))
}
```

### Using `navigate`

Calling `navigate` navigates to the specified `url`. The url should have the same origin as the app.

This is useful for imperatively navigating to an url when using an anchor tag (`<a>`) is not
possible/suitable (e.g. when submitting a form).
