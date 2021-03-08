# Changelog

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
