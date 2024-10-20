---
title: View Builder API
---

# The View Builder API

For those that prefer not to use macros, Sycamore also provides an ergonomic
builder API for composing views.

## Builder syntax

For those who dislike macro DSLs, we also provide an ergonomic builder API for
constructing views. Add the builder prelude as well as the main sycamore prelude
to your source file.

```rust
use sycamore::prelude::*;
use sycamore::web::tags::*;
```

### Elements

Elements can easily be created by calling the corresponding function for the
HTML tag.

```rust
a()
button()
div()
// etc...
```

### Text nodes

Text nodes are just string literals. These can be added to a node using
`.children()`.

```rust
div().children(
    p().children("Hello World!")
)
```

`.children()` can take anything that implements `Into<View>`. This includes
other text nodes, elements, views, and more. Tuples can also be converted into
`View`. So to add more than a single child node, just construct a tuple:

```rust
div().children((
    span().children("Hello "),
    span().children("World!"),
))
```

### Interpolation

Functions also implement `Into<View>`. When a function is used, the node becomes
dynamic. This means that the reactive values accessed in the function will be
used as dependencies and will trigger a re-render whenever a value changes.

```rust
let state = create_signal(0);

p().children(("Value: ", move || state.get()))
```

### Attributes

Every attribute can also be set on an element via a method:

```rust
p().class("my-class").id("my-paragraph").attr("aria-label", "My paragraph")
```

Custom attributes (that are not part of the HTML spec or are `data-*` or
`aria-*` attributes) can be set using `.attr()`.

### Events

Events are attached using `.on(...)`.

```rust
button().on(ev::click, |_| console_dbg!("clicked!")).children("Click me!")
```
