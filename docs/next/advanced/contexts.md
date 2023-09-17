# Contexts

Contexts provide an easy way to share data between components without drilling props through
multiple levels of the component hierarchy.

## Using contexts

It is a good habit to use the
[new type idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) when describing
the type of the data to be passed. Imagine the simple use-case of creating a global dark mode state
for our website. We can define the following `DarkMode` struct.

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
struct DarkMode(bool);

impl DarkMode {
    fn is_enabled(self) -> bool {
        self.0
    }
}
```

### Providing contexts

To make a context value accessible, we need to use the `provide_context_ref` method. Since we want
the context value to be reactive, we actually want a `Signal<DarkMode>` to be provided.

```rust
let dark_mode = create_signal(cx, DarkMode(false));
provide_context_ref(cx, dark_mode);
```

You might notice that there are two different methods for providing context: `provide_context` and
`provide_context_ref`. The first one is for providing a value, whereas the latter is for providing a
reference. The first one is simply a wrapper around `create_ref` and `provide_context_ref`. For
example, the two following code snippets are equivalent.

```rust
let value = 123;

let value_ref = create_ref(cx, value);
provide_context_ref(cx, value_ref);
// or equivalently...
provide_context(cx, value);
```

### Using contexts.

Once the context has been provided, it can be used in any nested scope including from the same scope
where the context value was provided.

To access the context, use the `use_context` method.

```rust
#[component]
fn ChildComponent<G: Html>(cx: Scope) -> View<G> {
    let dark_mode = use_context::<Signal<DarkMode>>(cx);
    // ...
}

let dark_mode = create_signal(cx, DarkMode(false));
provide_context_ref(cx, dark_mode);
view! { cx,
    ChildComponent {}
}
```

Remember that unlike contexts in React, the context is not reactive by itself. This is because
components only run once. In order to make a context value reactive, you need to use a `Signal` or
other reactive data structure.
