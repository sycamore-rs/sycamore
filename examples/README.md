# Examples

## Viewing on `sycamore-rs.netlify.app`

All the examples are hosted under `sycamore-rs.netlify.app/examples/<example_name>` with
`<example_name>` being the name of the example you want to view. For instance, the `todomvc` example
is hosted on
[`sycamore-rs.netlify.app/examples/todomvc`](https://sycamore-rs.netlify.app/examples/todomvc).

## Building Locally

All the examples can also be built locally using [Trunk](https://trunkrs.dev). For instance, the
following command builds and serves the `todomvc` example:

```bash
cd examples/todomvc
trunk serve
```

Now open up `localhost:8080` in your browser to see "Hello World!".

## Example List

| Example                                            | Description                                                                                    |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| [components](components)                           | UI abstraction using components                                                                |
| [context](context)                                 | A simple counter which can be incremented and decremented                                      |
| [counter](counter)                                 | Demonstration for the Context API                                                              |
| [hello-builder](hello-builder)                     | Hello World! With the builder API!                                                             |
| [hello-world](hello-world)                         | Hello World!                                                                                   |
| [higher-order-components](higher-order-components) | Higher-order-components (functions that create components)                                     |
| [http-request](http-request)                       | Suspense + async components for sending HTTP requests                                          |
| [hydrate](hydrate)                                 | Making existing HTML reactive                                                                  |
| [iteration](iteration)                             | Demonstration of how to iterate over data in UI                                                |
| [js-framework-benchmark](js-framework-benchmark)   | Implementation of [js-framework-benchmark](https://github.com/krausest/js-framework-benchmark) |
| [motion](motion)                                   | Demonstration for using animation frames and tweened signals                                   |
| [ssr](ssr)                                         | Demonstration of server-side-rendering                                                         |
| [timer](timer)                                     | Demonstration of using futures to auto-increment a counter                                     |
| [todomvc](todomvc)                                 | Fully compliant implementation of [TodoMVC](https://todomvc.com/) spec                         |
| [transitions](transitions)                         | Suspense + async transitions                                                                   |

