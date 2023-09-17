# Passing Attributes to a wrapped element

When you're working on code for your own website, props are a great way to pass customizable values
to a component. If you need to customize an additional attribute on your HTML element, just add a
prop! On the other hand, when you're working on components used by other people (i.e. component libraries),
specifying every possible customization can quickly become unfeasible or even impossible.

This is where the `Attributes` type is a useful escape hatch. Simply add a field named `attributes`
of that type to your props,

```rust
#[derive(Props)]
struct Props {
    attributes: Attributes<'cx, G>
}
```

and then "spread" the attributes onto an HTML element.

```rust
view! { cx,
    p(..props.attributes) { "Hello World!" }
}
```

Any attributes set by the user are now passed through onto your `p` element.

## Setting attributes

The user can set attributes by prefixing them with `attr:`. Event handlers or bindings are also
automatically passed through.

```rust
view! { cx,
    AccessibleLabel(attr:class = "bg-neutral-800 rounded", on:click = label_clicked) { "Label 1" }
}
```

Attributes are still fully type checked when the user sets them and will throw compiler errors
if the value doesn't fit the attribute.

## Accessing and modifying attributes

`Attributes` exposes a number of ways of accessing attributes. In addition to `get` and `remove`,
which return a general attribute value, typed versions of each method are available for strings,
booleans and refs.

```rust
let id = props.attributes.remove_str("id").unwrap_or_else(|| generate_id());
```

It's common that the user should be able to set any attribute except a few set by the component.
For this, there's a convenience method called `exclude_keys`.

```rust
props.attributes.exclude_keys(&["id", "aria-labelled-by", "aria-role"]);
```
