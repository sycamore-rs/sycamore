# Routing

Routers are the backbone of SPAs (Single Page Apps). They handle displaying different pages
depending on the URL. When an anchor tag (`<a>`) is clicked, the router will intercept it and
navigate to the correct page without performing a full refresh. This makes navigation feel faster
and smoother.

## Adding `sycamore-router`

To add routing to your Sycamore app, install the
[`sycamore-router`](https://crates.io/crates/sycamore-router) crate from crates.io.

```toml
sycamore-router = "0.6.3"
```

### Compatibility with `sycamore`

Note that the major version number for `sycamore-router` corresponds to the same major version
number for `sycamore` (e.g. `sycamore-router v0.5.x` is compatible with `sycamore v0.5.x`).

## Creating routes

Start off by adding `use sycamore_router::{Route, Router, RouterProps}` to the top of your source
code. This imports the symbols needed to define our router.

The heart of the router is an `enum`. Each variant of the `enum` represents a different route. To
make our `enum` usable with `Router`, we will use the `Route` derive macro to implement the required
traits for us.

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

The `#[to(_)]` attribute designates a route. For example, `#[to("/about")]` designates the route for
the about page.

The `#[not_found]` is a fallback route. It is the route that matches when all the other routes
don't. There must be one, and only one route marked with `#[not_found]`. Forgetting the not found
route will cause a compile error.

## Routes syntax

### Static routes

The simplest routes are static routes. We already have the `"/"` and `"/about"` routes in our above
example which are both static.

Static routes can also be nested, e.g. `"/my/nested/path"`.

#### Dynamic parameters

Path parameters can be dynamic by using angle brackets around a variable name in the route's path.
This will allow any segment to match the route in that position.

For example, to match any route with `"hello"` followed by a name, we could use:

```rust
#[to("/hello/<name>")]
Hello {
    name: String,
}
```

The `<name>` parameter is _captured_ by the `name` field in the `Hello` variant. For example, if we
were to visit `/hello/sycamore`, we would find

```rust
AppRoutes::Hello { name: "sycamore".to_string() }
```

Multiple dynamic parameters are allowed. For example, the following route...

```rust
#[to("/repo/<org>/<name>")]
Repo {
    org: String,
    name: String,
}
```

...would match `/repo/sycamore-rs/sycamore` with a value of

```rust
AppRoutes::Repo {
    org: "sycamore-rs".to_string(),
    name: "sycamore".to_string(),
}
```

### Dynamic segments

Dynamic segments can also be captured using the `<param..>` syntax.

For example, the following route will match `"page"` followed by an arbitrary number of segments
(including 0 segments).

```rust
#[to("/page/<path..>")]
Page {
    path: Vec<String>,
}
```

Dynamic segments match lazily, meaning that once the next segment can be matched, the capture will
be completed. For example, the following route will **not** capture the final `end` segment.

```rust
#[to("/start/<path..>/<end>")]
Path {
    path: Vec<String>,
}
```

#### Unit variants

Enum unit variants are also supported. The following route has the same behavior as the hello
example from before.

```rust
#[to("/hello/<name>")]
Hello(String)
```

#### Capture types

Capture variables are not limited to `String`. In fact, any type that implements the
[`FromParam`](https://docs.rs/sycamore-router/latest/sycamore_router/trait.FromParam.html) trait can
be used as a capture.

This trait is automatically implemented for types that already implement `FromStr`, which includes
many standard library types.

Because `FromParam` is fallible, the route will only match if the parameter can be parsed into the
corresponding type.

For example, `/account/123` will match the following route but `/account/abc` will not.

```rust
#[to("/account/<id>")]
Account { id: u32 }
```

Likewise, the
[`FromSegments`](https://docs.rs/sycamore-router/latest/sycamore_router/trait.FromSegments.html)
trait is the equivalent for dynamic segments.

## Using `Router`

To display content based on the route that matches, we can use a `Router`.

```rust
template! {
    Router(RouterProps::new(HistoryIntegration::new(), |route: StateHandle<AppRoutes>| {
        let t = create_memo(move || match route.get().as_ref() {
            AppRoutes::Index => template! {
                "This is the index page"
            },
            AppRoutes::About => template! {
                "About this website"
            },
            AppRoutes::NotFound => template! {
                "404 Not Found"
            },
        });
        template! {
            div(class="app) {
                (t.get().as_ref().clone())
            }
        }
    })
}
```

`Router` is just a component like any other. The props accept a closure taking a `StateHandle` of
the matched route as a parameter and an "integration". The integration is for adapting the router to
different environments (e.g. server-side rendering). The `HistoryIntegration` is a built-in
integration that uses the
[HTML5 History API](https://developer.mozilla.org/en-US/docs/Web/API/History_API).

Any clicks on anchor tags (`<a>`) created inside the `Router` will be intercepted and handled by the
router.

## Server-side rendering and `StaticRouter`

Whereas `Router` is used inside the context of a browser, `StaticRouter` can be used for SSR.

The difference between a `Router` and a `StaticRouter` is that the route is provided to
`StaticRouter` during the initialization phase. The initial route is provided as an argument to
`StaticRouterProps::new`.

This is so that `StaticRouter` can return a `Template` immediately without blocking to wait for the
route preload. The route is expected to be resolved separately using the `Route::match_path`
function.

```rust
let route = AppRoutes::match_path(path);

template! {
    StaticRouter(StaticRouterProps::new(route, |route: AppRoutes| {
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

## Integrations

TODO: docs for creating custom router integrations.

## Using `navigate`

Calling `navigate` navigates to the specified `url`. The url should have the same origin as the app.

This is useful for imperatively navigating to an url when using an anchor tag (`<a>`) is not
possible/suitable (e.g. when submitting a form).

## Data fetching and preloading

When data fetching (e.g. from a REST API) is required to load a page, it is recommended to preload
the data. This will cause the router to wait until the data is loaded before rendering the page,
removing the need for some "Loading..." indicator.

`spawn_local_in_scope` is a simple wrapper around `wasm_bindgen_futures::spawn_local` that extends
the current scope into inside the `async` block. Make sure you enable the `"futures"` feature on
`sycamore`.

```rust
use sycamore::futures::spawn_local_in_scope;

template! {
    Router(RouterProps::new(HistoryIntegration::new(), |route: StateHandle<AppRoutes>| {
        let template = Signal::new(Template::empty());
        create_effect(cloned!((template) => move || {
            let route = route.get();
            spawn_local_in_scope(cloned!((template) => async move {
                let t = match route.as_ref() {
                    AppRoutes::Index => template! {
                        "This is the index page"
                    },
                    AppRoutes::About => template! {
                        "About this website"
                    },
                    AppRoutes::NotFound => template! {
                        "404 Not Found"
                    },
                };
                template.set(t);
            }));
        }));

        template! {
            div(class="app") {
                (template.get().as_ref().clone())
            }
        }
    }
}
```

## `rel="external"`

By default, the router will intercept all `<a>` elements that have the same origin as the current
page. Sometimes, we just want the browser to handle navigation without being intercepted by the
router. To bypass the router, we can add the `rel="external"` attribute to the anchor tag.

```rust
template! {
    a(href="path", rel="external") { "Path" }
}
```
