//! Reactive signals.

use slotmap::new_key_type;

new_key_type! { pub(crate) struct SignalKey; }

pub struct Signal {}
