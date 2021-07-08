# Iteration

Sycamore uses components for rendering lists. This is to prevent recreating DOM nodes every time the
list changes. The components serve as wrappers to make rendering lists more ergonomic.

## `Keyed`

The `Keyed` component is used to render a list of items with a key. A key for each item is used to
identify the item to prevent re-rendering templates twice. Every time the list changes, a diffing
algorithm is used to determine which items need to be re-rendered according to the associated key.

```rust
let count = Signal::new(vec![1, 2]);

template! {
    ul {
        Keyed(KeyedProps {
            iterable: count.handle(),
            template: |x| template! {
                li { (x) }
            },
            key: |x| *x,
        })
    }
}
```

## `Indexed`

The `Indexed` component is used to render a list of items that is keyed by index. `Keyed` is
generally preferred over `Indexed` because it is more efficient in most scenarios.

```rust
let count = Signal::new(vec![1, 2]);

template! {
    ul {
        Indexed(IndexedProps {
            iterable: count.handle(),
            template: |x| template! {
                li { (x) }
            },
        })
    }
}
```

## `.iter().map()`

Lastly, to render a static list (a list that will never change), you can use the good-ol' `.map()`
function from the standard library. Be sure that the list is indeed static, because otherwise every
single node will be re-rendered every time the list changes.

```rust
let count = vec![1, 2];

let templates = Template::new_fragment(
    count.iter().map(|&x| template! { li (x) }).collect()
);

template! {
    ul {
        (templates)
    }
}
```
