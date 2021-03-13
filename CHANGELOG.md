# Changelog

## ✨ **0.3.0** _(2021-03-13)_

- #### ⚡️ Features

  - Nested effects. Inner effects are destroyed and recreated when outer effects re-run [[@lukechu10], [#29]]
  - `cloned!` macro for making it easier to clone items into a new scope [[@lukechu10], [#34]]
  - Effects are created inside a reactivity root (using `create_root`). When the root `Owner` is dropped, all effects are also destroyed [[@lukechu10], [37]]
  - Nested templates. Using this, it is also possible to build simple `if`/`else` control flow although there will be a more polished version [[@lukechu10], [#41]]

- #### 🛠 Fixes

  - Parse html root as an `HtmlTree` [[@lukechu10], [#25]]
  - Recreate effect dependencies on each re-run [[@lukechu10], [#29]]

- #### Internal Fixes and Improvements

  - Remove double boxing of `Computation` [[@Kestrer], [#31]]
  - Create `CODE_OF_CONDUCT.md` [[@lukechu10], [#33]]
  - Add some preliminary benchmarks for signals and effects [[@lukechu10], [#35]]
  - Add clippy to CI workflow [[@Kestrer], [#42]]

- #### ⚠ **BREAKING CHANGES**

  - Replaced `create_signal` with `Signal::new(...)` and return `Signal` instead of getter/setter functions for increased type safety [[@Kestrer], [#20]]

- #### Announcements

  - New documentation website: https://maple-rs.netlify.app/ [[@lukechu10], [#26] and [#40]]

## ✨ **0.2.0** _(2021-03-07)_

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
