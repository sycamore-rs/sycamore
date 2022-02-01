# Control Flow

TODO: update docs

Control flow in Sycamore can be achieved using the interpolation syntax. For example:

```rust
let visible = Signal::new(false);

view! {
    div {
        (if *visible.get() {
            view! { "Now you see me" }
        } else {
            view! {} // Now you don't
        })
    }
}
```

Since the interpolated value subscribes to `visible`, the content inside the if else will be toggled
when `visible` is changed.

The conditions for displaying content can also be more complex. For example, the following snippet
will display the value of `name` when it is non-empty, other wise displaying `"World"`.

Note the usage of `create_selector` here. The reason for this is to memoize the value of
`name.get().is_empty()`. We don't want the inner `view!` (inside the `if` block) to be recreated
every time `name` is changed. Rather, we only want it to be created when `name` becomes non-empty.

```rust
let name = Signal::new(String::new());

view! {
    h1 {
        (if *create_selector(cloned!((name) => move || !name.get().is_empty())).get() {
            cloned!((name) => view! {
                span { (name.get()) }
            })
        } else {
            view! { span { "World" } }
        })
    }
}
```
