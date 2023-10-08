# `NodeRef`

Sometimes we want to be able to reference a node in the DOM directly. We can do so by using
`NodeRef`.

A `NodeRef` can be created by using `create_node_ref`. This can be assigned, in turn, to a node
using the `ref` property in the `view!` macro.

```rust
let node_ref = create_node_ref();

view! {
    p(ref=node_ref) { "Hello World!" }
}
```

`node_ref` is now a reference to the `p` node.

We can access the raw node using the `.get()` method on `NodeRef`.

```rust
node_ref.get::<DomNode>()
```

Note that this method will `panic!` if the `NodeRef` has not been assigned to a node or if the
`NodeRef` has the wrong type. That means that calling `node_ref.get::<DomNode>()` will `panic!` in a
server side rendering context (which uses `SsrNode` instead of `DomNode`).
