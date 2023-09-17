# Components

Any serious UI framework needs a way to compose and abstract UI elements. In Sycamore, this can be
accomplished using components.

Components in `sycamore` are simply functions slapped with the `#[component]` attribute that take an
argument of type `Scope` (a reactive scope). Component functions only run once (unlike React where
functional-components are called on every render). Think of it as a builder-pattern for constructing
UI.

In order for the `view!` macro to distinguish between regular elements and components, it is
convention to name components using `PascalCase`.

```rust
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
    MyComponent {}
}
```

## Properties

Components would be much more useful if they can accept data from the parent and render the given
data. Luckily, we can do this with properties. To allow your component to accept properties, take a
second argument with a type that implements the `Props` trait. For convenience, you can automatically
derive the `Props` trait with a derive-macro.

```rust
#[derive(Props)]
struct MyProps {
    name: String,
    email: String,
}
```

The component can then be constructed by passing the properties to it from the `view!` macro.

```rust
view! { cx,
    MyComponent(name="John Doe", email="...")
}
```

## Reactive props

Accepting data from the parent sure is nice but it would be even better if updating the data in the
parent also updates the view in the child component! For components to automatically react to prop
changes, they should accept a signal. Most of the times, you'll want a `&ReadSignal` unless you want
mutable access to the data in which case you should use a `&Signal`. This way, updating the signal
will automatically update whatever is listening to it, even if it is inside the child component.

Here is an example of a simple component that displays the value of its prop and that automatically
updates the displayed value when the prop changes.

```rust
#[derive(Props)]
struct MyProps<'a> {
    value: &'a ReadSignal<i32>,
}

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
    MyComponent(value=state)
}
state.set(1); // automatically updates value in MyComponent
```

Note how the `'a` lifetime is used to ensure that the data lives as long as the `Scope`.

## Component children

Components can also be wrappers around other child views by adding the `children` field to our
properties struct.

```rust
#[derive(Props)]
pub struct MyComponentProps<'a, G: Html> {
    children: Children<'a, G>,
    class: String
}

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
    MyComponent(class="my-class") {
        p { "My sub item 1" }
        p { "My sub item 2" }
    }
}
```

## Default props

Some property fields might have a default value. Use the `#[prop(default)]` attribute to allow
omitting the property when constructing the component.

```rust
#[derive(Props)]
struct MyProps {
    name: String,
    #[prop(default)]
    email: String,
}

view! { cx,
    // Since the `email` prop is left out, it will be set to the default value of "".
    MyComponent(name="John Doe")
}
```

## `inline_props`

Creating a struct just for accepting a few props might be a bit boilerplate-y depending on your
tastes. Feeling lazy? Just slap `inline_props` onto your component and the function arguments will
automatically be converted into props! For example, the two code snippets below are essentially
identical except that in one case, the struct is manually specified whereas in the other case, it is
automatically generated.

```rust
// Manual method.
#[derive(Props)]
struct MyProps<'a> {
    value: &'a ReadSignal<i32>,
}
#[component]
fn MyComponent<'a, G: Html>(cx: Scope<'a>, props: MyProps<'a>) -> View<G> {
    view! {
        div(class="my-component") {
            "Value: " (props.value.get())
        }
    }
}

// inline_props for the win!
#[component(inline_props)]
fn MyComponent<'a, G: Html>(cx: Scope<'a>, value: &'a ReadSignal<i32>) -> View<G> {
    view! {
        div(class="my-component") {
            "Value: " (value.get())
        }
    }
}
```

One thing to keep in mind is that adding `inline_props` makes your function's argument names part of
the public API. Changing the name of the argument also changes the name of the prop and therefore
the users of your component will also need to update their code.

## Lifecycle

Component lifecycle is strongly tied to the reactive system, since, under the hood, components are
simply functions that are run inside an untracked scope.

This means that we can use the same helpers in the reactive system to attach callbacks to the
component lifecycle.

### `on_cleanup`

The `on_cleanup` method schedules a callback to be called when the reactive scope is destroyed. This
can also be used to schedule a callback when the component is destroyed.

```rust
#[component]
fn MyComponent(cx: Scope) -> View<G> {
    on_cleanup(cx, || {
        // Perform cleanup.
    });
    // ...
}
```
