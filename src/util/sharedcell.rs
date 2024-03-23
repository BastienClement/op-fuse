use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

/// A `Rc<RefCell<T>>` used to share a mutable value between multiple owners.
pub struct SharedCell<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> SharedCell<T> {
    pub fn new(inner: T) -> SharedCell<T> {
        SharedCell {
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    /// Immutably borrows the value.
    pub fn borrow(&self) -> Ref<T> {
        (*self.inner).borrow()
    }

    /// Mutably borrows the value.
    pub fn borrow_mut(&self) -> RefMut<T> {
        (*self.inner).borrow_mut()
    }
}

impl<T> Clone for SharedCell<T> {
    fn clone(&self) -> SharedCell<T> {
        SharedCell {
            inner: self.inner.clone(),
        }
    }
}
