# Components

Any serious UI framework needs a way to compose and abstract UI elements. In Sycamore, this can be
accomplished using components.

Components in `sycamore` are simply functions slapped with the `#[component]` attribute that take a
argument of type `Scope` (a reactive scope). Component functions only run once (unlike React
where functional-components are called on every render). Think of it as a builder-pattern for
constructing UI.

In order for the `view!` macro to distinguish between regular elements and components, it is
convention to name components using `PascalCase`.

```rust
#[allow(non_snake_case)]
#[component]
fn MyComponent<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        // ...
    }
}
```

To use the component from elsewhere, the `view!` macro has some special syntax.

```rust
view! { cx,
    MyComponent {
        // props...
    }
}
```

## Properties

Components would be much more useful if they can accept data from the parent and render the given
data. Luckily, we can do this with properties. To allow your component to accept properties, take a
second argument with a type that implements the `Prop` trait. For convenience, you can automatically
derive the `Prop` trait with a derive-macro.

```rust
#[derive(Prop)]
struct MyProps {
    name: String,
    email: String,
}
```

The component can then be constructed by passing the properties to it from the `view!` macro.

```rust
view! { cx,
    MyComponent {
        name: "John Doe",
        email: "...",
    }
}
```

## Reactive data

Accepting data from the parent sure is nice but it would be even better if updating the data in the
parent also updates the view in the child component! For components to automatically react to prop
changes, they should accept a signal. Most of the times, you'll want a `&ReadSignal` unless you want
mutable access to the data in which case you should use a `&Signal`. This way, updating the signal
will automatically update whatever is listening to it, even if it is inside the child component.

Here is an example of a simple component that displays the value of its prop and that automatically
updates the displayed value when the prop changes.

```rust
#[derive(Prop)]
struct MyProps<'a> {
    value: &'a ReadSignal<i32>,
}

#[allow(non_snake_case)]
#[component]
fn MyComponent<'a, G: Html>(cx: Scope<'a>, props: MyProps<'a>) -> View<G> {
    view! {
        div(class="my-component") {
            "Value: " (props.value.get())
        }
    }
}

let state = create_signal(cx, 0);
view! {
    MyComponent {
        value: state
    }
}
state.set(1); // automatically updates value in MyComponent
```

Note how the `'a` lifetime is used to ensure that the data lives as long as the `Scope`.

## Lifecycle

Component lifecycle is strongly tied to the reactive system, since, under the hood, components are
simply functions that are run inside an untracked scope.

This means that we can use the same helpers in the reactive system to attach callbacks to the
component lifecycle.

### `on_cleanup`

The `on_cleanup` method schedules a callback to be called when the reactive scope is destroyed. This
can also be used to schedule a callback when the component is destroyed.

```rust
#[allow(non_snake_case)]
#[component]
fn MyComponent(cx: Scope) -> View<G> {
    on_cleanup(cx, || {
        // Perform cleanup.
    });
    // ...
}
```

### Nested components

You can also build components that can have sub components. We need to create a Props to support this.

```rust
#[derive(Prop)]
pub struct MyComponentProps<'a, G: Html> {
    children: Children<'a, G>,
    class: String
}

#[allow(non_snake_case)]
#[component]
pub fn MyComponent<'a, G: Html>(cx: Scope<'a>, props: MyComponentProps<'a, G>) -> View<G> {
    let children = props.children.call(cx);
    view! { cx,
      div(class=props.class) {
        (children)
      }
    }
}

view! {
    MyComponent {
        class: "my-class"
        {
            p { "My sub item 1" }
            p { "My sub item 2" }
        }
    }
}
```

### Default props

When your making a custom component but you don't need all of them all the time. You can add a default value to it.

```rust
#[derive(Prop)]
struct MyProps {
    name: String,
    #[builder(default)]
    email: String,
}

view! { cx,
    MyComponent {
        name: "John Doe",
        // email can be left out and will have the default value. In case of a string "".
    }
}
```