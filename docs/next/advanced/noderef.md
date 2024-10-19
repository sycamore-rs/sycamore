# `NodeRef`

Sometimes we want to be able to reference a node in the DOM directly. We can do
so by using `NodeRef`.

A `NodeRef` can be created by using `create_node_ref`. This can be assigned, in
turn, to a node using the `r#ref` property.

```rust
let node_ref = create_node_ref();

view! {
    p(r#ref=node_ref) { "Hello World!" }
}
```

`node_ref` is now a reference to the `p` node.

We can access the raw node using the `.get()` method on `NodeRef`.

```rust
let node = node_ref.get();
```

Note that this method will `panic!` if the `NodeRef` has not been assigned to a
node or is being accessed on the server. For this reason, `NodeRef`s are usually
accessed within `on_mount` or event handlers that are never run on the server.
