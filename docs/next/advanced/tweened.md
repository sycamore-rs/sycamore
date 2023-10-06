# `Tweened`

Tweened states update their values over a period of time. For example, the following code snippet
interpolates a value from `0` to `100` over a period of `250ms`:

```rust
use std::time::Duration;

use sycamore::easing;
use sycamore::motion::create_tweened_signal;

let tweened = create_tweened_signal(0.0f32, Duration::from_millis(250), easing::quad_out);

tweened.set(100.0);
```

Different easing functions are provided in the `sycamore::easing` module.
