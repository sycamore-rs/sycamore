# Data binding

You can bind your `Signal` to a DOM property with the `bind:` directive. When the DOM property is updated, so is your `Signal`. Here is an example.

```rust
use sycamore::prelude::*;

let value = Signal::new(String::new());

template! {
    input(bind:value=value)
}
```

Now, when the user types into the input, the `value` signal will automatically be synced.

The way this works is by listening to specific events on the DOM node according to the property. For instance, `value` uses the `on:input` event. 

## Supported properties

Below is a table of supported properties and events that are listened to.

| Property      | Event name    | Signal type  |
| :------------ | :------------ | :----------- |
| `value`       | `input`       | `String`     |
| `checked`     | `change`      | `bool`       |