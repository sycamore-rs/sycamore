# Development

Issues and Pull Requests are welcome!

To install and work on Maple locally:

```bash
git clone https://github.com/lukechu10/maple
cd maple
```

## Testing

To run unit tests, use:

```bash
cargo test
```

To run integration tests, you will need to have [wasm-pack](https://rustwasm.github.io/wasm-pack/) installed:

```bash
wasm-pack test maple-core --firefox # or --chrome
```

If you want to run the tests in a headless browser, pass the `--headless` flag as well.
