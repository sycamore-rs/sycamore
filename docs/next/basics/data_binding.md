# Data binding

You can bind your `Signal` to a DOM property with the `bind:` directive. When the DOM property is
updated, so is your `Signal`. Here is an example.

```rust
use sycamore::prelude::*;

let value = create_signal(cx, String::new());

view! {
    input(bind:value=value)
}
```

Now, when the user types into the input, the `value` signal will automatically be synced.

The way this works is by listening to specific events on the DOM node according to the property. For
instance, `value` uses the `on:input` event.

## Supported properties

Below is a table of supported properties and events that are listened to.

| Property        | Event name | Signal type |
| :-------------- | :--------- | :---------- |
| `value`         | `input`    | `String`    |
| `valueAsNumber` | `input`    | `f64`       |
| `checked`       | `change`   | `bool`      |

Be aware that the `valueAsNumber` property will only work on `input` elements with type "range" or "number".
