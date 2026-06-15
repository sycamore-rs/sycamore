# Examples

## Live Demos

All the examples are hosted under `examples.sycamore.dev/<example_name>` with
`<example_name>` replaced with the name of the example you want to view. For instance, the `todomvc` example
can be viewed at
[`examples.sycamore.dev/todomvc`](https://examples.sycamore.dev/todomvc).

For purely client-side examples—which make up the majority of the examples—viewing the live demos should be equivalent to viewing locally. However, the live demos are broken in various ways for the few examples that aren't purely client-side.

## Viewing Locally

All the examples can also be viewed locally. Most of them are client-side applications meant to be viewed through a [Trunk](https://trunk-rs.github.io/trunk/) development server, but a few of them use other viewing methods.

### Purely client-side examples

Most of the examples are client-side applications meant to be viewed through a [Trunk](https://trunk-rs.github.io/trunk/) development server. For instance, the `todomvc` example can be viewed as follows.

1. Go to the `examples/todomvc` folder.
2. Make the client-side code available by calling `trunk serve`.
3. View the example by visiting `http://localhost:8080` in a web browser.

### Purely server-side examples

The `ssr` example is a server-side application. It can be viewed as follows.

1. Go to the `examples/ssr` folder.
2. Write the server-side rendered page to standard output by calling `cargo run`.
3. View the page by examining its source code in the terminal or redirecting it to an HTML file and opening it in a web browser.

### Examples that serve their client-side and server-side parts together

The `ssr-streaming` example has both client-side and server-side parts. It comes with a unified development server that both delivers the client-side application and runs the server-side application. It can be viewed as follows.

1. Go to the `examples/ssr-streaming` folder.
2. Make the client-side code available while also running the server-side code by calling `cargo run`.
3. View the example by visiting `http://localhost:8080` in a web browser.

### Examples that serve their client-side and server-side parts separately

The `websockets` example has both client-side and server-side parts. The client-side application is delivered through a [Trunk](https://trunk-rs.github.io/trunk/) development server, and the server-side application runs on a separate development server.

1. Go to the `examples/websockets` folder.
2. Launch the server-side application by calling `cargo run`.
   - _The server listens for WebSocket messages at the root endpoint on port 1110._
3. Make the client-side application available through a development server by calling `trunk serve`.
   - _The client is available by HTTP request at the root endpoint on port 8080._
   - _WebSocket messages to the `/websocket` endpoint on port 8080 are now forwarded to the server._
4. View the client by visiting `http://localhost:8080` in a web browser.
   - _The client shows a [trigram](https://en.wikipedia.org/wiki/Bagua#Trigrams), with its name below it._
   - _The server announces a new connection on standard output._
5. Click a line on the trigram.
   - _In the client, the line you clicked switches from unbroken to broken._
   - _The server announces an update on standard output. The trigram in the announcement matches the trigram now shown on the client._

## Example List

| Example                                            | Description                                                                                    |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| [attributes-passthrough](attributes-passthrough)   | Passing through dynamic attributes to an inner element                                         |
| [components](components)                           | UI abstraction using components                                                                |
| [context](context)                                 | Demonstration for the Context API                                                              |
| [counter](counter)                                 | A simple counter which can be incremented and decremented                                      |
| [hello-builder](hello-builder)                     | Hello World! With the builder API!                                                             |
| [hello-world](hello-world)                         | Hello World!                                                                                   |
| [higher-order-components](higher-order-components) | Higher-order-components (functions that create components)                                     |
| [http-request](http-request)                       | Suspense + async components for sending HTTP requests                                          |
| [http-request-builder](http-request-builder)       | Suspense + async components for sending HTTP requests using the builder API!                   |
| [hydrate](hydrate)                                 | Making existing HTML reactive                                                                  |
| [iteration](iteration)                             | Demonstration of how to iterate over data in UI                                                |
| [js-framework-benchmark](js-framework-benchmark)   | Implementation of [js-framework-benchmark](https://github.com/krausest/js-framework-benchmark) |
| [js-snippets](js-snippets)                         | Demonstration of importing a function from JS                                                  |
| [motion](motion)                                   | Demonstration for using animation frames and tweened signals                                   |
| [number-binding](number-binding)                   | Demonstration of binding the value of a range or number input                                  |
| [router](router)                                   | Demonstration of sycamore-router                                                               |
| [ssr](ssr)                                         | Demonstration of server-side-rendering                                                         |
| [ssr-suspense](ssr-suspense)                       | Demonstration of server-side rendering with suspense support                                   |
| [ssr-streaming](ssr-streaming)                     | Demonstration of server-side-rendering with streaming                                          |
| [svg](svg)                                         | Creating SVGs with the `view!` macro                                                           |
| [timer](timer)                                     | Demonstration of using futures to auto-increment a counter                                     |
| [todomvc](todomvc)                                 | Fully compliant implementation of [TodoMVC](https://todomvc.com/) spec                         |
| [transitions](transitions)                         | Suspense + async transitions                                                                   |
| [websockets](websockets)                           | Using a WebSocket to communicate with a server                                                 |
