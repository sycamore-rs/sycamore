# Control Flow

Control flow in Sycamore can be achieved using the interpolation syntax. For
example:

```rust
let visible = create_signal(true);

view! {
    div {
        (if visible.get() {
            view! { "Now you see me" }
        } else {
            view! { } // Now you don't
        })
    }
}
```

Since the interpolated value subscribes to `visible`, the content inside the if
else will be toggled when `visible` is changed.

The conditions for displaying content can also be more complex. For example, the
following snippet will display the value of `name` when it is non-empty,
otherwise displaying `"World"`.

Note the usage of `create_selector` here. The reason for this is to memoize the
value of `name.get().is_empty()`. We don't want the inner `view!` (inside the
`if` block) to be recreated every time `name` is changed. Rather, we only want
it to be created when `name` becomes non-empty.

```rust
let name = create_signal(String::new());
let is_empty = create_selector(|| !name.get().is_empty());

view! {
    h1 {
        (if is_empty.get() {
            view! { span { (name.get()) } }
        } else {
            view! { span { "World" } }
        })
    }
}
```
