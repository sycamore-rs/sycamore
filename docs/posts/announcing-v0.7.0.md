_2021-12-08_

# Announcing Sycamore v0.7.0!

_Client-side hydration + Builder API_

Hello everybody! Sycamore is a library for building isomorphic web applications in Rust and
WebAssembly.

I'm happy to announce that we've just released v0.7.0. The big features that are landing with this
release are client-side hydration and builder API!

## What's New?

### Client-side hydration

Sycamore has supported server side rendering since v0.5 but now it also supports client-side
hydration! It used to be that calling `hydrate` or `hydrate_to` would simply delete whatever HTML
nodes were rendered on the server and create new nodes to replace them, not great for performance.
Now, existing HTML is "hydrated", meaning that reactivity is attached to the static HTML sent by the
server, eliminating useless node creation.

With this feature, Sycamore supports building fully isomorphic apps (app code that can be run on
both client and server).

### Builder API

A big shoutout to [`@jquesada2016`](https://github.com/jquesada2016) who worked hard on creating an
alternative builder API for Sycamore! If you are not a fan of the `view!` macro-DSL, rejoice! You
can now write your Sycamore apps just with plain Rust functions. Here is a sample:

```rust
let name = Signal::new(String::new());

div()
    .child(
        h1().text("Hello ")
            .dyn_child(cloned!((name) => move || {
                if *create_selector(cloned!(name => move || !name.get().is_empty())).get() {
                    span()
                        .dyn_text(cloned!(name => move || name.get().to_string()))
                        .build()
                } else {
                    span().text("World").build()
                }
            }))
            .text("!")
            .build(),
    )
    .child(input().bind_value(name).build())
    .build()
```

Note that the builder API is still experimental and does not play very well with hydration yet. If
you have ideas on how to make the builder API more ergonomic, don't hesitate to create an issue on
our [issue tracker](https://github.com/sycamore-rs/sycamore/issues).

## `Template` -> `View`

We renamed `Template` to `View` and `template!` to `view!` to make the name slightly shorter to type
and also to prevent conflict with Perseus' notion of
[templates](https://arctic-hen7.github.io/perseus/en-US/docs/next/templates/intro).

Migrating should be a pretty simple matter for most. Just perform a find-and-replace in your IDE or
using `grep` to replace all instances of `Template` to `View` and all instances of `template!` to
`view!`.

## `IS_BROWSER`

Checking whether some code was executing in the browser or on the server required an ugly hack:

```rust
if TypeId::of::<G>() == TypeId::of::<DomNode>() { ... }
```

What this did was basically check whether the generic parameter `G` (the rendering backend) was
`DomNode`. It wasn't very clear what this meant at a first glance.

Now the intention is much clearer (and also looks much nicer):

```rust
if G::IS_BROWSER { ... }
```

## Conclusion

A big thank you to all the
[contributors](https://github.com/sycamore-rs/sycamore/graphs/contributors) who made this release
possible!

For more detailed changes, check out the
[changelog](https://github.com/sycamore-rs/sycamore/blob/master/CHANGELOG.md#-070-2021-12-08).

If you are interested in contributing to Sycamore, check out the issues labeled with
[`good first issue`](https://github.com/sycamore-rs/sycamore/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)
on our GitHub repository. Don't hesitate to join our [Discord server](https://discord.gg/vDwFUmm6mU)
too! See you there.

Thanks!
