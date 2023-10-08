# Development

## Using [gitpod.io](https://www.gitpod.io)

The Sycamore repository is configured for [gitpod.io](https://www.gitpod.io). This is the easiest
way to contribute. Just click on the button below and a full-fledged development workspace will be
spun up with all dependencies pre-installed and VSCode ready to go.

[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/sycamore-rs/sycamore)

## Using your local dev machine

To install and work on Sycamore locally:

```bash
git clone https://github.com/sycamore-rs/sycamore
cd sycamore
```

## Testing

To run unit tests and non Wasm integration tests, use:

```bash
cargo test --all-features
```

To run integration tests in the browser, you will need to have
[wasm-pack](https://rustwasm.github.io/wasm-pack/) installed:

```bash
wasm-pack test sycamore --firefox # or --chrome
```

If you want to run the tests in a headless browser, pass the `--headless` flag as well.

## Adding an example

TOOD

## Writing proc-macro code

TODO: explain how to run UI tests with trybuild

## Creating a new docs page

TOOD: explain `docs` utility for generating HTML from markdown.

## Benchmarking

TOOD: explain benchmark tool.

### Using the js-framework-benchmark CI

To have the CI automatically run the `js-framework-benchmark`, simply add the `performance` label to your PR. This will automatically queue up a benchmark for every commit and the result of the run will be posted as a comment.

## PR Requirements

Before your PR can be merged, we ask that your code is properly formatted using `cargo fmt` and
passes `cargo clippy`.

If your code introduces new functionality, we also ask you to write some unit tests and eventually
some integration tests.

Thank you for taking the time to contribute to Sycamore!
