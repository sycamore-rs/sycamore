# Changelog

## ✨ **0.6.2** _(2021-10-09)_

- #### ⚡️ Features

  - [Add `GenericNode::remove_property` method.](https://github.com/sycamore-rs/sycamore/pull/267)
  - [Add `node!` macro for creating a single node.](https://github.com/sycamore-rs/sycamore/pull/265)

- #### 🛠 Fixes

  - [Do not strip leading `/` from pathname.](https://github.com/sycamore-rs/sycamore/pull/271) This
    fixes an issue with the router on Firefox when navigating to the index page.

- #### Internal

  - [Remove `ToTokens` impl for `TagName`.](https://github.com/sycamore-rs/sycamore/pull/273)

## ✨ **0.6.1** _(2021-09-25)_

- #### 🛠 Fixes

  - [Disable `full` feature on `syn` to reduce compile times.](https://github.com/sycamore-rs/sycamore/pull/245)
  - [Use a global `HashSet` for void elements lookup in SSR.](https://github.com/sycamore-rs/sycamore/pull/246)
  - [Remove part of formatting machinery from `sycamore` and `sycamore-reactive` to reduce binary size.](https://github.com/sycamore-rs/sycamore/pull/247)
  - [Fix panic on updating dynamic node inside a template fragment.](https://github.com/sycamore-rs/sycamore/pull/251)
  - [Implement `Default` for `Signal`.](https://github.com/sycamore-rs/sycamore/pull/257)
  - [Implement `Default` for `StateHandle`.](https://github.com/sycamore-rs/sycamore/pull/260)

- #### 📃 Documentation

  - [Update docs to use boolean for disabled attribute on button.](https://github.com/sycamore-rs/sycamore/pull/248)
  - [Add documentation on topological dependency tracking.](https://github.com/sycamore-rs/sycamore/pull/253)

## ✨ **0.6.0** _(2021-09-12)_

Release post: https://sycamore-rs.netlify.app/news/announcing-v0.6.0

- #### ⚡️ Features

  - [Add integrations for `sycamore-router`.](https://github.com/sycamore-rs/sycamore/pull/183)
  - [Added `dangerously_set_inner_html` special attribute to html elements.](https://github.com/sycamore-rs/sycamore/pull/190)
    This allows directly setting an element's inner html without going through a `NodeRef` and
    manually calling `.set_inner_html()`.
  - [Implement `Portal`s.](https://github.com/sycamore-rs/sycamore/pull/209) Portals allow adding
    nodes to elements that are in another tree.
  - [Allow instantiating components with the `Component` trait.](https://github.com/sycamore-rs/sycamore/pull/213)
    You can now also create components that are generic over another component. This pattern can be
    seen in the
    [`higher-order-components`](https://github.com/sycamore-rs/sycamore/tree/master/examples/higher-order-components)
    example.
  - [Respect base html tag in router.](https://github.com/sycamore-rs/sycamore/pull/220)
  - [Dark mode on website!](https://github.com/sycamore-rs/sycamore/pull/225)
  - [`create_reducer` hook.](https://github.com/sycamore-rs/sycamore/pull/233) The `create_reducer`
    hook is an alternative to `Signal::new`. It allows you to use a reducer function to get the next
    state from the previous state.
    ```rust
    enum Msg {
        Increment,
        Decrement,
    }
    let (state, dispatch) = create_reducer(0, |state, msg: Msg| match msg {
        Msg::Increment => *state + 1,
        Msg::Decrement => *state - 1,
    });
    ```
  - [Opt out of router by using `rel="external"` on an anchor tag.](https://github.com/sycamore-rs/sycamore/pull/238)
    Adding `rel="external"` to an anchor tag will use the browser's default navigation behavior.
    ```rust
    template! {
        a(href="path", rel="external") { "Link" }
    }
    ```

- #### 🛠 Fixes

  - [Fix logic error in `reconcile_fragments`.](https://github.com/sycamore-rs/sycamore/pull/180)
  - [Fix grammar on website index page.](https://github.com/sycamore-rs/sycamore/pull/181)
  - [Scroll to top when navigating to a page.](https://github.com/sycamore-rs/sycamore/pull/186)
  - [Use `ahash` instead of default SipHash for better performance.](https://github.com/sycamore-rs/sycamore/pull/193)
  - [Explicitly define MSRV to 1.53 and run CI in a matrix.](https://github.com/sycamore-rs/sycamore/pull/195)
  - [Remove inline JS snippet.](https://github.com/sycamore-rs/sycamore/pull/194) This removes the
    need to load an extra JS file before Sycamore can start.
  - [Add some UI tests for `#[component]` attribute macro.](https://github.com/sycamore-rs/sycamore/pull/198)
  - [Generate a `sitemap.xml` for the website.](https://github.com/sycamore-rs/sycamore/pull/201)
  - [Fix broken link to the reactivity page on the website index page.](https://github.com/sycamore-rs/sycamore/pull/203)
  - [Explain that Trunk needs a `index.html` file in Hello World docs.](https://github.com/sycamore-rs/sycamore/pull/206)
  - [Remove internal `Rc` from `DomNode`.](https://github.com/sycamore-rs/sycamore/pull/210) This
    significantly improves performance and memory usage. See the PR for benchmarks.
  - [Optimize the website with `wasm-opt` to reduce binary size.](https://github.com/sycamore-rs/sycamore/pull/211)
  - [Optimize `create_effect`.](https://github.com/sycamore-rs/sycamore/pull/216)
  - [Fix `SsrNode`'s implementation of `remove_child` removing two children instead of just one.](https://github.com/sycamore-rs/sycamore/pull/218)
  - [Hold a backlink for each `ReactiveScope` to its parent scope.](https://github.com/sycamore-rs/sycamore/pull/223)
    This fixes a bug where `use_context` could only access the context on the first render and would
    panic on subsequent accesses.
  - [Remove dependency on `chrono`.](https://github.com/sycamore-rs/sycamore/pull/224) This was
    replaced with direct access to browser APIs to reduce the number of dependencies and thus to
    improve build times.
  - [Replace internal usage of `.unwrap()` with `.unwrap_throw()`.](https://github.com/sycamore-rs/sycamore/pull/226)
    Slightly improves binary sizes.
  - [Derive `Clone` for `sycamore_router` path types.](https://github.com/sycamore-rs/sycamore/pull/232)
  - [Update `todomvc` example with latest features.](https://github.com/sycamore-rs/sycamore/pull/229)
  - [Fix router not actually parsing identifiers.](https://github.com/sycamore-rs/sycamore/pull/234)
    Fixes a bug where a dynamic parameter followed by a dynamic segment would parse as a single
    segment.
  - [Build rustdocs in CI.](https://github.com/sycamore-rs/sycamore/pull/235) The API documentation
    for the `master` branch is available at
    [sycamore-rs.netlify.app/api](https://sycamore-rs.netlify.app/api).
  - [Reorganize documentation a bit.](https://github.com/sycamore-rs/sycamore/pull/236)

- #### 🚨 **BREAKING CHANGES**

  - [Extract reactive primitives into separate crate `sycamore-reactive`.](https://github.com/sycamore-rs/sycamore/pull/204)
    Reactive primitives are now re-exported in the `sycamore` crate to avoid adding new dependencies
    to your project. It is also now possible to use reactive primitives without using `sycamore` by
    directly depending on `sycamore-reactive`.
  - [Rename sub-module `sycamore::rx` to `sycamore::reactive`.](https://github.com/sycamore-rs/sycamore/pull/205)
    `rx` might be ambiguous with [Rx family of libraries](http://reactivex.io/). Renaming to
    `reactive` makes it clear that it works differently from Rx libraries.
  - [Refactored router with new API.](https://github.com/sycamore-rs/sycamore/pull/222) See the
    [new documentation](https://sycamore-rs.netlify.app/docs/v0.6/advanced/routing) for more
    details.
  - [Support boolean attributes.](https://github.com/sycamore-rs/sycamore/pull/239) Some attributes
    now expect a `bool` instead of `impl ToString`. This also fixes an issue where previously,
    attributes couldn't be removed directly from the `template!` macro.
    ```rust
    // Before
    template! {
        input(type="checkbox", checked="") { "Checkbox" }
    }
    // After
    template! {
        input(type="checkbox", checked=true) { "Checkbox" }
    }
    ```

## ✨ **0.5.2** _(2021-07-17)_

#### Changelog

- #### ⚡️ Features

  - Context API: introducing `ContextProvider` and `use_context` [[@lukechu10],
    [#169](https://github.com/sycamore-rs/sycamore/pull/169)]

- #### 🛠 Fixes

  - Router should not prevent default if meta keys are held down [[@baile320],
    [#165](https://github.com/sycamore-rs/sycamore/pull/165)]
  - Remove some `optional` tags on dependencies [[@lukechu10],
    [#167](https://github.com/sycamore-rs/sycamore/pull/167)]
  - Explicitly enable `std` feature in `indexmap` to prevent compile error [[@Gearme],
    [#170](https://github.com/sycamore-rs/sycamore/pull/170)]
  - Do not panic when `map_keyed` is updated with same data in debug mode [[@lukechu10],
    [#173](https://github.com/sycamore-rs/sycamore/pull/173)]

- #### 🛠 Internal Fixes and Improvements

  - Add some integration tests for `StaticRouter` [[@lukechu10],
    [#168](https://github.com/sycamore-rs/sycamore/pull/168)]
  - Fix intra-doc link [[@tshepang], [#162](https://github.com/sycamore-rs/sycamore/pull/162)]
  - Refactor `sycamore-macro` static text and splices [[@lukechu10],
    [#175](https://github.com/sycamore-rs/sycamore/pull/175)]

- #### 📃 Documentation

  - Simplify cargo command in documentation [[@tshepang],
    [#163](https://github.com/sycamore-rs/sycamore/pull/163)]
  - Fix link to book in website version selector [[@lukechu10],
    [#166](https://github.com/sycamore-rs/sycamore/pull/166)]

- #### 🚅 Performance Improvements

  - Performance tweaks [[@lukechu10], [#171](https://github.com/sycamore-rs/sycamore/pull/171)]

## ✨ **0.5.1** _(2021-07-09)_

#### Changelog

- #### 🛠 Fixes

  - Remove `Hash` trait bound from `T` in `Keyed` [[@lukechu10],
    [#148](https://github.com/sycamore-rs/sycamore/pull/148)]

- #### 🛠 Internal Fixes and Improvements

  - Add news section to website with v0.5.0 release post [[@lukechu10],
    [#149](https://github.com/sycamore-rs/sycamore/pull/149),
    [#149](https://github.com/sycamore-rs/sycamore/pull/149)]
  - Fix typo in v0.5.0 release post [[@tshepang],
    [#156](https://github.com/sycamore-rs/sycamore/pull/156)]
  - Add versioned docs to website [[@lukechu10],
    [#160](https://github.com/sycamore-rs/sycamore/pull/160)]

- #### 🚅 Performance Improvements

  - Reduce allocations when creating `Template`s [[@lukechu10],
    [#143](https://github.com/sycamore-rs/sycamore/pull/143)]
  - Do not create effects when splice is static (using simple heuristic) [[@lukechu10],
    [#155](https://github.com/sycamore-rs/sycamore/pull/155)]
  - Set `className` directly instead of calling `setAttribute` [[@lukechu10],
    [#157](https://github.com/sycamore-rs/sycamore/pull/157)]
  - Optimize `create_effect` [[@lukechu10],
    [#159](https://github.com/sycamore-rs/sycamore/pull/159)]

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

- #### 🚨 **BREAKING CHANGES**

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

[@baile320]: https://github.com/baile320
[@dicej]: https://github.com/dicej
[@gearme]: https://github.com/Gearme
[@iwburns]: https://github.com/iwburns
[@juanmarchetto]: https://github.com/JuanMarchetto
[@kestrer]: https://github.com/Kestrer
[@lukechu10]: https://github.com/lukechu10
[@lights0123]: https://github.com/lights0123
[@riey]: https://github.com/Riey
[@tmpr]: https://github.com/tmpr
[@tshepang]: https://github.com/tshepang
