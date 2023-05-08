use crate::signals::Signal;

pub struct Memo<T: 'static> {
    signal: Signal<T>,
}
