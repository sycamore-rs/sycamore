use super::*;

pub fn mapped<T, U>(list: Signal<Vec<T>>, map_fn: impl Fn(T) -> U) -> StateHandle<U> {
    todo!();
}

pub fn indexed<T, U>(list: Signal<Vec<T>>, map_fn: impl Fn(T) -> U) -> StateHandle<U> {
    todo!();
}
