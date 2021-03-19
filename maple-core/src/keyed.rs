//! Keyed iteration in [`template`](crate::template).

use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::TemplateResult;

pub trait KeyedWith<Key, I, T, F, K>
where
    I: Iterator<Item = T>,
    F: Fn(T) -> TemplateResult,
    K: Fn(&T) -> Key,
{
    /// # Panics
    ///
    /// Panics when two keys have the same value as determined by their [`Eq`] implementation.
    fn keyed_with(self, f: F, key: K) -> Keyed<Key, T, F, K>;
}

impl<Key, I, T, F, K> KeyedWith<Key, I, T, F, K> for I
where
    Key: Hash + Eq,
    I: Iterator<Item = T>,
    F: Fn(T) -> TemplateResult,
    K: Fn(&T) -> Key,
{
    fn keyed_with(self, f: F, key: K) -> Keyed<Key, T, F, K> {
        let mut map = HashMap::new();

        for item in self {
            let key = key(&item);

            map.insert(key, f(item)).expect("duplicate key not allowed");
        }

        Keyed {
            f,
            key,
            template_map: map,
            _phantom: PhantomData,
        }
    }
}

pub struct Keyed<Key, T, F, K>
where
    F: Fn(T) -> TemplateResult,
    K: Fn(&T) -> Key,
{
    f: F,
    key: K,
    template_map: HashMap<Key, TemplateResult>,
    _phantom: PhantomData<T>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_lazy() {}
}
