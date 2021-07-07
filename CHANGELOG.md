# Changelog

## ✨ **0.5.0** _(2021-07-06)_

Release post:
[https://sycamore-rs.netlify.app/news/announcing-v0.5.0](https://sycamore-rs.netlify.app/news/announcing-v0.5.0)

#### Changelog

- #### ⚡️ Features

  - Tweened signals [[@lukechu10], [#86](https://github.com/sycamore-rs/sycamore/pull/86)]
  - Add more easing functions [[@tmpr], [#90](https://github.com/sycamore-rs/sycamore/pull/90)]
  - Document fragments in the `template!` macro. `template!` can now accept the following input:
    ```rust
    template! {
        p { "First" }
        p { "Second" }
    }
    ```
    ```rust
    template! {} // empty template
    ```
    [[@lukechu10], [#89](https://github.com/sycamore-rs/sycamore/pull/89)]
  - 2-way data binding for certain supported props [[@lukechu10],
    [#93](https://github.com/sycamore-rs/sycamore/pull/93)]
  - Allow generic component's type parameters to be inferred from context [[@lukechu10],
    [#100](https://github.com/sycamore-rs/sycamore/pull/100)]
  - Refactored `Template` (renamed from `TemplateResult`) to allow make the template system more
    flexible. It is now possible to imperatively construct `Template`s from raw dom nodes, `Vec`s,
    and closures [[@lukechu10], [#104](https://github.com/sycamore-rs/sycamore/pull/104)]
  - Sycamore router [[@lukechu10], [#118](https://github.com/sycamore-rs/sycamore/pull/118)]
  - Temporary "fake" hydration [[@lukechu10],
    [#101](https://github.com/sycamore-rs/sycamore/pull/101)]
  - Router use anchor tags (`<a>`) instead of `Link` components [[@lukechu10],
    [#128](https://github.com/sycamore-rs/sycamore/pull/128)]
  - Reactive scope dependency count utility function [[@lukechu10],
    [#144](https://github.com/sycamore-rs/sycamore/pull/144)]

- #### 🛠 Fixes

  - Implement missing operations on `SsrNode` [[@lukechu10],
    [#82](https://github.com/sycamore-rs/sycamore/pull/82),
    [#138](https://github.com/sycamore-rs/sycamore/pull/138)]
  - Remove warning when effects are created outside of a reactive scope [[@JuanMarchetto],
    [#95](https://github.com/sycamore-rs/sycamore/pull/95)]
  - Do not assume `Signal` is valid for entire duration of the effect and make effect triggers
    deterministic (outer effects rerun first) [[@lukechu10],
    [#145](https://github.com/sycamore-rs/sycamore/pull/145)]
  - Eagerly evaluate dynamic `Template`s with `create_memo` [[@lukechu10],
    [#146](https://github.com/sycamore-rs/sycamore/pull/146)]

- #### 📃 Documentation

  - Host documentation on website [[@lukechu10],
    [#83](https://github.com/sycamore-rs/sycamore/pull/83)]
  - Write documentation [[@lukechu10], [#87](https://github.com/sycamore-rs/sycamore/pull/87),
    [#111](https://github.com/sycamore-rs/sycamore/pull/111),
    [#133](https://github.com/sycamore-rs/sycamore/pull/133)]
  - Fix `CHANGELOG.md` typo [[@Riey], [#102](https://github.com/sycamore-rs/sycamore/pull/102)]
  - Style documentation website with [TailwindCSS](https://tailwindcss.com) [[@lukechu10],
    [#117](https://github.com/sycamore-rs/sycamore/pull/117)]
  - Use router for documentation website header [[@lukechu10],
    [#132](https://github.com/sycamore-rs/sycamore/pull/132)]
  - Extract outline from markdown and new index page [[@lukechu10],
    [#134](https://github.com/sycamore-rs/sycamore/pull/134)]
  - Move documentation under `/docs/*` path [[@lukechu10],
    [#139](https://github.com/sycamore-rs/sycamore/pull/139)]

- #### 🛠 Internal Fixes and Improvements

  - Build website in GitHub Actions [[@lukechu10],
    [#84](https://github.com/sycamore-rs/sycamore/pull/84)]
  - Run unit tests in [Miri](https://github.com/rust-lang/miri) to catch memory errors
    [[@lukechu10], [#91](https://github.com/sycamore-rs/sycamore/pull/91)]
  - Update Trunk to v0.11.0 [[@lukechu10], [#96](https://github.com/sycamore-rs/sycamore/pull/96)]
  - Improve documentation website lighthouse score [[@lukechu10],
    [#97](https://github.com/sycamore-rs/sycamore/pull/97)]
  - Setup [gitpod.io](https://www.gitpod.io) configuration to make it easier to contribute
    [[@lukechu10], [#98](https://github.com/sycamore-rs/sycamore/pull/98)]
  - Update `wasm-bindgen` to v0.2.74 in `.gitpod.Dockerfile` [[@lukechu10],
    [#108](https://github.com/sycamore-rs/sycamore/pull/108)]
  - Update links to git repository to point to `sycamore-rs/sycamore` [[@lukechu10],
    [#110](https://github.com/sycamore-rs/sycamore/pull/110)]
  - Add micro-benchmarks for `map_indexed` and `map_keyed` [[@lukechu10],
    [#115](https://github.com/sycamore-rs/sycamore/pull/115)]
  - Run [js-framework-benchmark](https://github.com/krausest/js-framework-benchmark) in GitHub
    Actions. Pull requests with the `performance` tag will automatically trigger a benchmark run
    [[@lukechu10], [#114](https://github.com/sycamore-rs/sycamore/pull/114)]
  - Fix branch naming to work with continuous benchmark [[@lukechu10],
    [#116](https://github.com/sycamore-rs/sycamore/pull/116)]
  - Various performance improvements [[@lukechu10],
    [#126](https://github.com/sycamore-rs/sycamore/pull/126)]
  - Google search console verification file [[@lukechu10],
    [#135](https://github.com/sycamore-rs/sycamore/pull/135)]
  - Get `NodeId` for `DomNode` lazily to improve performance when `Hash` is not used [[@lukechu10],
    [#136](https://github.com/sycamore-rs/sycamore/pull/136)]
  - Do not insert unnecessary marker nodes [[@lukechu10],
    [#137](https://github.com/sycamore-rs/sycamore/pull/137)]
  - Remove unnecessary `Rc<RefCell<_>>`s [[@lukechu10],
    [#141](https://github.com/sycamore-rs/sycamore/pull/141)]
  - Cache `window.document` since it is frequently used to prevent going through JS interop
    [[@lukechu10], [#142](https://github.com/sycamore-rs/sycamore/pull/142)]

- #### ⚠ **BREAKING CHANGES**

  - Abstraction over rendering backend! This introduces the concept of `GenericNode` which is a
    trait to access the underlying rendering backend. Currently, Sycamore ships with `DomNode` and
    `SsrNode` out-of-the-box for rendering the the browser DOM and to a static string respectively.
    Components should now be generic over `G: GenericNode` to be able to render to multiple backends
    [[@lights0123], [#67](https://github.com/sycamore-rs/sycamore/pull/67)]
  - Require using the `#[component(_)]` attribute macro for defining components. This changes the
    component syntax to:
    ```rust
    #[component(MyComponent<G>)]
    fn my_component() -> Template<G> {
        todo!()
    }
    ```
    The `#[component(_)]` macro generates a `struct` under the hood that implements the `Component`
    trait for improved type safety. This also means that you no longer need
    `#[allow(non_snake_case)]` in your code! [[@lukechu10],
    [#70](https://github.com/sycamore-rs/sycamore/pull/70)]
    [#92](https://github.com/sycamore-rs/sycamore/pull/92)]
  - Rename `Owner` to `ReactiveScope` [[@lukechu10],
    [#99](https://github.com/sycamore-rs/sycamore/pull/99)]
  - Renamed crate from `maple-core` to `sycamore` and `maple-core-macro` to `sycamore-macro`. Also
    renamed all instances of "Maple" to "Sycamore" [[@lukechu10],
    [#109](https://github.com/sycamore-rs/sycamore/pull/109)]
  - Rename `TemplateResult` to `Template` [[@lukechu10],
    [#112](https://github.com/sycamore-rs/sycamore/pull/112)]
  - Rename `reactive` sub-module to `rx` [[@lukechu10],
    [#113](https://github.com/sycamore-rs/sycamore/pull/113)]
  - Remove render functions (`render`, `render_to`, `render_to_string`, etc...) from `prelude`.
    These functions are generally only called once in a Sycamore app so they do not belong in the
    prelude [[@lukechu10], [#140](https://github.com/sycamore-rs/sycamore/pull/140)]

## ✨ **0.4.3** _(2021-04-01)_

#### Changelog

- #### ⚡️ Features

  - Support `'-'` in attribute names. This makes the following syntax valid:
    ```rust
    template! {
        button(aria-hidden="true")
    }
    ```
    [[@lukechu10], [#79](https://github.com/sycamore-rs/sycamore/pull/79)]

- #### 🛠 Fixes

  - Delete removed nodes in `Keyed` first before adding new nodes and moving existing nodes
    [[@lukechu10], [#77](https://github.com/sycamore-rs/sycamore/pull/77)]

## ✨ **0.4.2** _(2021-03-31)_

#### Changelog

- #### 🛠 Fixes

  - Fix `Keyed` iteration (hopefully for the last time) when moving nodes already inserted
    [[@lukechu10], [#75](https://github.com/sycamore-rs/sycamore/pull/75)]

## ✨ **0.4.1** _(2021-03-31)_

#### Changelog

- #### 🛠 Fixes

  - Fix `Keyed` iteration (swapping and inserting not at the end) [[@lukechu10],
    [#73](https://github.com/sycamore-rs/sycamore/pull/73)]

- #### 📃 Documentation Fixes

  - Fix typo in `README.md` [[@iwburns], [#64](https://github.com/sycamore-rs/sycamore/pull/64)]]
  - Add discord server link to issue template [[@lukechu10],
    [#68](https://github.com/sycamore-rs/sycamore/pull/68)]

- #### 🎁 Example Fixes

  - Fix filter links in TodoMVC example [[@lukechu10],
    [#65](https://github.com/sycamore-rs/sycamore/pull/65)]

## ✨ **0.4.0** _(2021-03-25)_

#### Changelog

- #### ⚡️ Features

  - Iteration using `SignalVec`. This is more of an experiment and there are some bugs. This will
    most likely be removed in a future version [[@lukechu10],
    [#49](https://github.com/sycamore-rs/sycamore/pull/49)]
  - Keyed iteration using `Keyed` and non-keyed iteration using `Indexed` which can iterate over a
    `Signal<Vec>`. This is the recommended way to iterate over a list of values [[@lukechu10],
    [#51](https://github.com/sycamore-rs/sycamore/pull/51),
    [#53](https://github.com/sycamore-rs/sycamore/pull/53) and
    [#54](https://github.com/sycamore-rs/sycamore/pull/54)]
  - Node references. Use the `ref` attribute to bind an HTML element to a `NodeRef` [[@lukechu10],
    [#57](https://github.com/sycamore-rs/sycamore/pull/57)]

- #### 🛠 Fixes

  - Fix debug assertions in `Keyed` [[@lukechu10],
    [#53](https://github.com/sycamore-rs/sycamore/pull/53)]

- #### 🛠 Internal Fixes and Improvements

  - Setup integration tests [[@lukechu10], [#51](https://github.com/sycamore-rs/sycamore/pull/51)]

- #### 🎁 Examples

  - Complete spec conforming TodoMVC implementation [[@lukechu10],
    [#60](https://github.com/sycamore-rs/sycamore/pull/60)]

## ✨ **0.3.1** _(2021-03-16)_

#### Changelog

- #### ⚡️ Features

  - More types in `template!` macro. `template!` can now be nested [[@lukechu10],
    [#45](https://github.com/sycamore-rs/sycamore/pull/45)]
  - Component lifecycle using `on_cleanup` [[@lukechu10],
    [#24](https://github.com/sycamore-rs/sycamore/pull/24)]

- #### 🛠 Fixes

  - Add some badges to `README.md` [[@lukechu10],
    [#44](https://github.com/sycamore-rs/sycamore/pull/44) and
    [#48](https://github.com/sycamore-rs/sycamore/pull/48)]

## ✨ **0.3.0** _(2021-03-13)_

#### Changelog

- #### ⚡️ Features

  - Nested effects. Inner effects are destroyed and recreated when outer effects re-run
    [[@lukechu10], [#29](https://github.com/sycamore-rs/sycamore/pull/29)]
  - `cloned!` macro for making it easier to clone items into a new scope [[@lukechu10],
    [#34](https://github.com/sycamore-rs/sycamore/pull/34)]
  - Effects are created inside a reactivity root (using `create_root`). When the root `Owner` is
    dropped, all effects are also destroyed [[@lukechu10],
    [37](https://github.com/sycamore-rs/sycamore/pull/37)]
  - Nested templates. Using this, it is also possible to build simple `if`/`else` control flow
    although there will be a more polished version [[@lukechu10],
    [#41](https://github.com/sycamore-rs/sycamore/pull/41)]

- #### 🛠 Fixes

  - Parse html root as an `HtmlTree` [[@lukechu10],
    [#25](https://github.com/sycamore-rs/sycamore/pull/25)]
  - Recreate effect dependencies on each re-run [[@lukechu10],
    [#29](https://github.com/sycamore-rs/sycamore/pull/29)]

- #### 🛠 Internal Fixes and Improvements

  - Remove double boxing of `Computation` [[@Kestrer],
    [#31](https://github.com/sycamore-rs/sycamore/pull/31)]
  - Create `CODE_OF_CONDUCT.md` [[@lukechu10],
    [#33](https://github.com/sycamore-rs/sycamore/pull/33)]
  - Add some preliminary benchmarks for signals and effects [[@lukechu10],
    [#35](https://github.com/sycamore-rs/sycamore/pull/35)]
  - Add clippy to CI workflow [[@Kestrer], [#42](https://github.com/sycamore-rs/sycamore/pull/42)]

- #### ⚠ **BREAKING CHANGES**

  - Replaced `create_signal` with `Signal::new(...)` and return `Signal` instead of getter/setter
    functions for increased type safety [[@Kestrer],
    [#20](https://github.com/sycamore-rs/sycamore/pull/20)]

- #### 📢 Announcements

  - New documentation website: https://sycamore-rs.netlify.app/ [[@lukechu10],
    [#26](https://github.com/sycamore-rs/sycamore/pull/26) and
    [#40](https://github.com/sycamore-rs/sycamore/pull/40)]

## ✨ **0.2.0** _(2021-03-07)_

#### Changelog

- #### ⚡️ Features

  - Components! In `sycamore` they are simply plain old functions that take their props via their
    parameters [[#9](https://github.com/sycamore-rs/sycamore/pull/9)]
  - Event listeners now have access to the `Event` object
    [[#16](https://github.com/sycamore-rs/sycamore/pull/16)]

- #### 🛠 Changes

  - The `template!` macro now returns a `TemplateResult` instead of raw DOM nodes for increased type
    safety [[#10](https://github.com/sycamore-rs/sycamore/pull/10)]

## ✨ **0.1.1** _(2021-03-07)_

#### Changelog

- #### ⚡️ Features

  - New `untracked` utility for explicitly opting out of automatic dependency detection in reactive
    contexts [[#8](https://github.com/sycamore-rs/sycamore/pull/8)]

- #### 🛠 Fixes
  - Only subscribe to a dependency once in an effect, even if it is called multiple times
    [[#7](https://github.com/sycamore-rs/sycamore/pull/7)]

## ✨ **0.1.0** _(2021-03-06)_

#### Changelog

- #### ⚡️ Features

  - Initial release!
  - Added `template!` macro.
  - Added reactivity primitives.

[@iwburns]: https://github.com/iwburns
[@juanmarchetto]: https://github.com/JuanMarchetto
[@kestrer]: https://github.com/Kestrer
[@lukechu10]: https://github.com/lukechu10
[@lights0123]: https://github.com/lights0123
[@riey]: https://github.com/Riey
[@tmpr]: https://github.com/tmpr
