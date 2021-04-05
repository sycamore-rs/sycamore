# Development

Issues and Pull Requests are welcome!

To install and work on Maple locally:

```bash
git clone https://github.com/lukechu10/maple
cd maple
```

## Testing

To run unit tests and non WASM integration tests, use:

```bash
cargo test
```

To run integration tests in the browser, you will need to have [wasm-pack](https://rustwasm.github.io/wasm-pack/) installed:

```bash
wasm-pack test maple-core --firefox # or --chrome
```

If you want to run the tests in a headless browser, pass the `--headless` flag as well.

#### PR Requirements

Before your PR can be merged, we ask that your code is properly formatted using `cargo fmt` and passes `cargo clippy`.

If your code introduces new functionality, we also ask you to write some unit tests and eventually some integration tests.

Thank you for taking the time to contribute to Maple!
