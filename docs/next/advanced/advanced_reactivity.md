# Advanced Reactivity

## Reactive scopes

### on_cleanup

### Nested effects

TODO

Help us out by writing the docs and sending us a PR!

## Batching Updates

Sycamore's fine grained reactivity system updates immediately when a signal changes. This works great
in most cases because only things that depend on that change will actually be rerun. But what if
you need to make changes to two or more related signals?

### The `batch` function

The batch function lets you execute a closure and delay any effects until it completes. This means
you can update multiple related signals and only have their dependent effects run once.

Not only can this improve performance, but can even improve safety in your code since updating
related signals synchronously can cause your effects to run with an unintended state. Batching
the calls means you only ever run your effects when you're done with your mutations. You can think
of batching a little bit like database transactions.

### Example

In this example, we are assigning both names on a button click and this triggers our rendered update
twice. Using batch lets us avoid that.

```rust
let update_names = || {
    batch(|| {
        first_name.set_fn(|first_name| first_name + "n");
        last_name.set_fn(|last_name| last_name + "!");
    });
}
create_effect(cx, || {
    // This would run twice when `update_names` is called without batching. With batching, it only
    // runs once.
    format!("{} {}", first_name.get(), last_name.get());
});
```
