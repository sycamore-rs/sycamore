use std::cell::RefCell;
use std::rc::Rc;

use crate::{TemplateList, TemplateResult};

use super::*;

/// A reactive [`Vec`].
/// This is more effective than using a [`Signal<Vec>`](Signal) because it allows fine grained
/// reactivity within the `Vec`.
pub struct SignalVec<T: 'static> {
    signal: Signal<RefCell<Vec<T>>>,
    /// A list of past changes that is accessed by subscribers.
    /// Cleared when all subscribers are called.
    changes: Rc<RefCell<Vec<VecDiff<T>>>>,
}

impl<T: 'static> SignalVec<T> {
    /// Create a new empty `SignalVec`.
    pub fn new() -> Self {
        Self {
            signal: Signal::new(RefCell::new(Vec::new())),
            changes: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Create a new `SignalVec` with existing values from a [`Vec`].
    pub fn with_values(values: Vec<T>) -> Self {
        Self {
            signal: Signal::new(RefCell::new(values)),
            changes: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Get the current pending changes that will be applied to the `SignalVec`.
    pub fn changes(&self) -> &Rc<RefCell<Vec<VecDiff<T>>>> {
        &self.changes
    }

    /// Returns the inner backing [`Signal`] used to store the data. This method should used with
    /// care as unintentionally modifying the [`Vec`] will not trigger any updates and cause
    /// potential future problems.
    pub fn inner_signal(&self) -> &Signal<RefCell<Vec<T>>> {
        &self.signal
    }

    pub fn replace(&self, values: Vec<T>) {
        self.add_change(VecDiff::Replace { values });

        self.trigger_and_apply_changes();
    }

    pub fn insert(&self, index: usize, value: T) {
        self.add_change(VecDiff::Insert { index, value });

        self.trigger_and_apply_changes();
    }

    pub fn update(&self, index: usize, value: T) {
        self.add_change(VecDiff::Update { index, value })
    }

    pub fn remove(&self, index: usize) {
        self.add_change(VecDiff::Remove { index });

        self.trigger_and_apply_changes();
    }

    pub fn swap(&self, index1: usize, index2: usize) {
        self.add_change(VecDiff::Swap { index1, index2 });

        self.trigger_and_apply_changes();
    }

    pub fn push(&self, value: T) {
        self.add_change(VecDiff::Push { value });

        self.trigger_and_apply_changes();
    }

    pub fn pop(&self) {
        self.add_change(VecDiff::Pop);

        self.trigger_and_apply_changes();
    }

    pub fn clear(&self) {
        self.add_change(VecDiff::Clear);

        self.trigger_and_apply_changes();
    }

    fn add_change(&self, change: VecDiff<T>) {
        self.changes.borrow_mut().push(change);
    }

    fn trigger_and_apply_changes(&self) {
        self.signal.trigger_subscribers();

        for change in self.changes.take() {
            change.apply_to_vec(&mut self.signal.get().borrow_mut());
        }
    }

    /// Creates a derived `SignalVec`.
    ///
    /// # Example
    /// ```
    /// use maple_core::prelude::*;
    ///
    /// let my_vec = SignalVec::with_values(vec![1, 2, 3]);
    /// let squared = my_vec.map(|x| *x * *x);
    ///
    /// assert_eq!(*squared.inner_signal().get().borrow(), vec![1, 4, 9]);
    ///
    /// my_vec.push(4);
    /// assert_eq!(*squared.inner_signal().get().borrow(), vec![1, 4, 9, 16]);
    ///
    /// my_vec.swap(0, 1);
    /// assert_eq!(*squared.inner_signal().get().borrow(), vec![4, 1, 9, 16]);
    /// ```
    pub fn map<U: Clone>(&self, f: impl Fn(&T) -> U + 'static) -> SignalVec<U> {
        let signal = self.inner_signal().clone();
        let changes = Rc::clone(&self.changes());
        let f = Rc::new(f);

        create_effect_initial(move || {
            let derived = SignalVec::with_values(
                signal.get().borrow().iter().map(|value| f(value)).collect(),
            );

            let effect = {
                let derived = derived.clone();
                let signal = signal.clone();
                move || {
                    signal.get(); // subscribe to signal
                    for change in changes.borrow().iter() {
                        match change {
                            VecDiff::Replace { values } => {
                                derived.replace(values.iter().map(|value| f(value)).collect())
                            }
                            VecDiff::Insert { index, value } => derived.insert(*index, f(value)),
                            VecDiff::Update { index, value } => derived.update(*index, f(value)),
                            VecDiff::Remove { index } => derived.remove(*index),
                            VecDiff::Swap { index1, index2 } => derived.swap(*index1, *index2),
                            VecDiff::Push { value } => derived.push(f(value)),
                            VecDiff::Pop => derived.pop(),
                            VecDiff::Clear => derived.clear(),
                        }
                    }
                }
            };

            (Rc::new(effect), derived)
        })
    }
}

impl SignalVec<TemplateResult> {
    /// Create a [`TemplateList`] from the `SignalVec`.
    pub fn template_list(&self) -> TemplateList {
        TemplateList::from(self.clone())
    }
}

impl<T: 'static + Clone> SignalVec<T> {
    /// Create a [`Vec`] from a [`SignalVec`]. The returned [`Vec`] is cloned from the data which
    /// requires `T` to be `Clone`.
    /// 
    /// # Example
    /// ```
    /// use maple_core::prelude::*;
    /// 
    /// let signal = SignalVec::with_values(vec![1, 2, 3]);
    /// assert_eq!(signal.to_vec(), vec![1, 2, 3]);
    /// ```
    pub fn to_vec(&self) -> Vec<T> {
        self.signal.get().borrow().clone()
    }
}

impl<T: 'static> Default for SignalVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> Clone for SignalVec<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
            changes: Rc::clone(&self.changes),
        }
    }
}

/// An enum describing the changes applied on a [`SignalVec`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VecDiff<T> {
    Replace { values: Vec<T> },
    Insert { index: usize, value: T },
    Update { index: usize, value: T },
    Remove { index: usize },
    Swap { index1: usize, index2: usize },
    Push { value: T },
    Pop,
    Clear,
}

impl<T> VecDiff<T> {
    pub fn apply_to_vec(self, v: &mut Vec<T>) {
        match self {
            VecDiff::Replace { values } => *v = values,
            VecDiff::Insert { index, value } => v.insert(index, value),
            VecDiff::Update { index, value } => v[index] = value,
            VecDiff::Remove { index } => {
                v.remove(index);
            }
            VecDiff::Swap { index1, index2 } => v.swap(index1, index2),
            VecDiff::Push { value } => v.push(value),
            VecDiff::Pop => {
                v.pop();
            }
            VecDiff::Clear => v.clear(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_vec() {
        let my_vec = SignalVec::new();
        assert_eq!(*my_vec.inner_signal().get().borrow(), Vec::<i32>::new());

        my_vec.push(3);
        assert_eq!(*my_vec.inner_signal().get().borrow(), vec![3]);

        my_vec.push(4);
        assert_eq!(*my_vec.inner_signal().get().borrow(), vec![3, 4]);

        my_vec.pop();
        assert_eq!(*my_vec.inner_signal().get().borrow(), vec![3]);
    }

    #[test]
    fn map() {
        let my_vec = SignalVec::with_values(vec![1, 2, 3]);
        let squared = my_vec.map(|x| *x * *x);

        assert_eq!(*squared.inner_signal().get().borrow(), vec![1, 4, 9]);

        my_vec.push(4);
        assert_eq!(*squared.inner_signal().get().borrow(), vec![1, 4, 9, 16]);

        my_vec.pop();
        assert_eq!(*squared.inner_signal().get().borrow(), vec![1, 4, 9]);
    }

    #[test]
    fn map_chain() {
        let my_vec = SignalVec::with_values(vec![1, 2, 3]);
        let squared = my_vec.map(|x| *x * 2);
        let quadrupled = squared.map(|x| *x * 2);

        assert_eq!(*quadrupled.inner_signal().get().borrow(), vec![4, 8, 12]);

        my_vec.push(4);
        assert_eq!(
            *quadrupled.inner_signal().get().borrow(),
            vec![4, 8, 12, 16]
        );

        my_vec.pop();
        assert_eq!(*quadrupled.inner_signal().get().borrow(), vec![4, 8, 12]);
    }

    #[test]
    fn map_chain_temporary() {
        let my_vec = SignalVec::with_values(vec![1, 2, 3]);
        let quadrupled = my_vec.map(|x| *x * 2).map(|x| *x * 2);

        assert_eq!(*quadrupled.inner_signal().get().borrow(), vec![4, 8, 12]);

        my_vec.push(4);
        assert_eq!(
            *quadrupled.inner_signal().get().borrow(),
            vec![4, 8, 12, 16]
        );

        my_vec.pop();
        assert_eq!(*quadrupled.inner_signal().get().borrow(), vec![4, 8, 12]);
    }

    #[test]
    fn map_inner_scope() {
        let my_vec = SignalVec::with_values(vec![1, 2, 3]);
        let quadrupled;

        let doubled = my_vec.map(|x| *x * 2);
        assert_eq!(*doubled.inner_signal().get().borrow(), vec![2, 4, 6]);

        quadrupled = doubled.map(|x| *x * 2);
        assert_eq!(*quadrupled.inner_signal().get().borrow(), vec![4, 8, 12]);

        drop(doubled);
        assert_eq!(*quadrupled.inner_signal().get().borrow(), vec![4, 8, 12]);

        my_vec.push(4);
        assert_eq!(
            *quadrupled.inner_signal().get().borrow(),
            vec![4, 8, 12, 16]
        );
    }
}
