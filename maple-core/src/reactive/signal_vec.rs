use std::cell::RefCell;
use std::rc::Rc;

use super::*;

pub struct SignalVec<T: 'static> {
    signal: Signal<RefCell<Vec<T>>>,
    /// A list of past changes that is accessed by subscribers.
    /// Cleared when all subscribers are called.
    changes: Rc<RefCell<Vec<VecDiff<T>>>>,
}

impl<T: 'static> SignalVec<T> {
    pub fn new() -> Self {
        Self {
            signal: Signal::new(RefCell::new(Vec::new())),
            changes: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn with_values(values: Vec<T>) -> Self {
        Self {
            signal: Signal::new(RefCell::new(values)),
            changes: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn changes(&self) -> &Rc<RefCell<Vec<VecDiff<T>>>> {
        &self.changes
    }

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

    pub fn map<U: Clone>(&self, f: impl Fn(&T) -> U + 'static) -> impl Fn() -> SignalVec<U> {
        let data = self.inner_signal().get();
        let changes = Rc::clone(&self.changes());
        let f = Rc::new(f);

        move || {
            let data = Rc::clone(&data);
            let changes = Rc::clone(&changes);
            let f = Rc::clone(&f);

            create_effect_initial(move || {
                let derived =
                    SignalVec::with_values(data.borrow().iter().map(|value| f(value)).collect());

                let effect = {
                    let derived = derived.clone();
                    move || {
                        for change in changes.borrow().iter() {
                            match change {
                                VecDiff::Replace { values } => {
                                    derived.replace(values.iter().map(|value| f(value)).collect())
                                }
                                VecDiff::Insert { index, value } => {
                                    derived.insert(*index, f(value))
                                }
                                VecDiff::Update { index, value } => {
                                    derived.update(*index, f(value))
                                }
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
        assert_eq!(*my_vec.inner_signal().get().borrow(), vec![]);

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

        assert_eq!(*squared().inner_signal().get().borrow(), vec![1, 4, 9]);

        my_vec.push(4);
        assert_eq!(*squared().inner_signal().get().borrow(), vec![1, 4, 9, 16]);

        my_vec.pop();
        assert_eq!(*squared().inner_signal().get().borrow(), vec![1, 4, 9]);
    }
}
