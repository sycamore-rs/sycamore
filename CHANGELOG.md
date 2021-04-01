# Changelog

## ✨ **0.4.2** _(2021-03-31)_

#### Changelog

- #### 🛠 Fixes

  - Fix `Keyed` iteration (hopefully for the last time) when moving nodes already inserted [[@lukechu10], [#75](https://github.com/lukechu10/maple/pull/75)]

## ✨ **0.4.1** _(2021-03-31)_

#### Changelog

- #### 🛠 Fixes

  - Fix `Keyed` iteration (swapping and inserting not at the end) [[@lukechu10], [#73](https://github.com/lukechu10/maple/pull/73)]

- #### 📃 Documentation Fixes

  - Fix typo in `README.md` [[@iwburns], [#64](https://github.com/lukechu10/maple/pull/64)]]
  - Add discord server link to issue template [[@lukechu10], [#68](https://github.com/lukechu10/maple/pull/68)]

- #### 🎁 Example Fixes

  - Fix filter links in TodoMVC example [[@lukechu10], [#65](https://github.com/lukechu10/maple/pull/65)]

## ✨ **0.4.0** _(2021-03-25)_

#### Changelog

- #### ⚡️ Features

  - Iteration using `SignalVec`. This is more of an experiment and there are some bugs. This will most likely be removed in a future version [[@lukechu10], [#49](https://github.com/lukechu10/maple/pull/49)]
  - Keyed iteration using `Keyed` and non-keyed iteration using `Indexed` which can iterate over a `Signal<Vec>`. This is the recommended way to iterate over a list of values [[@lukechu10], [#51](https://github.com/lukechu10/maple/pull/51), [#53](https://github.com/lukechu10/maple/pull/53) and [#54](https://github.com/lukechu10/maple/pull/54)]
  - Node references. Use the `ref` attribute to bind an HTML element to a `NodeRef` [[@lukechu10], [#57](https://github.com/lukechu10/maple/pull/57)]

- #### 🛠 Fixes

  - Fix debug assertions in `Keyed` [[@lukechu10], [#53](https://github.com/lukechu10/maple/pull/53)]

- #### 🛠 Internal Fixes and Improvements

  - Setup integration tests [[@lukechu10], [#51](https://github.com/lukechu10/maple/pull/51)]

- #### 🎁 Examples

  - Complete spec conforming TodoMVC implementation [[@lukechu10], [#60](https://github.com/lukechu10/maple/pull/60)]

## ✨ **0.3.1** _(2021-03-16)_

#### Changelog

- #### ⚡️ Features

  - More types in `template!` macro. `template!` can now be nested [[@lukechu10], [#45](https://github.com/lukechu10/maple/pull/45)]
  - Component lifecycle using `on_cleanup` [[@lukechu10], [#24](https://github.com/lukechu10/maple/pull/24)]

- #### 🛠 Fixes

  - Add some badges to `README.md` [[@lukechu10], [#44](https://github.com/lukechu10/maple/pull/44) and [#48](https://github.com/lukechu10/maple/pull/48)]

## ✨ **0.3.0** _(2021-03-13)_

#### Changelog

- #### ⚡️ Features

  - Nested effects. Inner effects are destroyed and recreated when outer effects re-run [[@lukechu10], [#29](https://github.com/lukechu10/maple/pull/29)]
  - `cloned!` macro for making it easier to clone items into a new scope [[@lukechu10], [#34](https://github.com/lukechu10/maple/pull/34)]
  - Effects are created inside a reactivity root (using `create_root`). When the root `Owner` is dropped, all effects are also destroyed [[@lukechu10], [37](https://github.com/lukechu10/maple/pull/37)]
  - Nested templates. Using this, it is also possible to build simple `if`/`else` control flow although there will be a more polished version [[@lukechu10], [#41](https://github.com/lukechu10/maple/pull/41)]

- #### 🛠 Fixes

  - Parse html root as an `HtmlTree` [[@lukechu10], [#25](https://github.com/lukechu10/maple/pull/25)]
  - Recreate effect dependencies on each re-run [[@lukechu10], [#29](https://github.com/lukechu10/maple/pull/29)]

- #### 🛠 Internal Fixes and Improvements

  - Remove double boxing of `Computation` [[@Kestrer], [#31](https://github.com/lukechu10/maple/pull/31)]
  - Create `CODE_OF_CONDUCT.md` [[@lukechu10], [#33](https://github.com/lukechu10/maple/pull/33)]
  - Add some preliminary benchmarks for signals and effects [[@lukechu10], [#35](https://github.com/lukechu10/maple/pull/35)]
  - Add clippy to CI workflow [[@Kestrer], [#42](https://github.com/lukechu10/maple/pull/42)]

- #### ⚠ **BREAKING CHANGES**

  - Replaced `create_signal` with `Signal::new(...)` and return `Signal` instead of getter/setter functions for increased type safety [[@Kestrer], [#20](https://github.com/lukechu10/maple/pull/20)]

- #### 📢 Announcements

  - New documentation website: https://maple-rs.netlify.app/ [[@lukechu10], [#26](https://github.com/lukechu10/maple/pull/26) and [#40](https://github.com/lukechu10/maple/pull/40)]

## ✨ **0.2.0** _(2021-03-07)_

#### Changelog

- #### ⚡️ Features

  - Components! In `maple` they are simply plain old functions that take their props via their parameters [[#9](https://github.com/lukechu10/maple/pull/9)]
  - Event listeners now have access to the `Event` object [[#16](https://github.com/lukechu10/maple/pull/16)]

- #### 🛠 Changes

  - The `template!` macro now returns a `TemplateResult` instead of raw DOM nodes for increased type safety [[#10](https://github.com/lukechu10/maple/pull/10)]

## ✨ **0.1.1** _(2021-03-07)_

#### Changelog

- #### ⚡️ Features

  - New `untracked` utility for explicitly opting out of automatic dependency detection in reactive contexts [[#8](https://github.com/lukechu10/maple/pull/8)]

- #### 🛠 Fixes
  - Only subscribe to a dependency once in an effect, even if it is called multiple times [[#7](https://github.com/lukechu10/maple/pull/7)]

## ✨ **0.1.0** _(2021-03-06)_

#### Changelog

- #### ⚡️ Features

  - Initial release!
  - Added `template!` macro.
  - Added reactivity primitives.

[@iwburns]: https://github.com/iwburns
[@Kestrer]: https://github.com/Kestrer
[@lukechu10]: https://github.com/lukechu10
