use std::mem;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub type ListenerId = usize;

#[derive(Debug)]
pub struct Listeners<T> {
    inner: Arc<Mutex<Vec<Option<T>>>>,
}

impl<T> Listeners<T> {
    pub fn add(&self, listener: T) -> ListenerId {
        let mut lock = self.inner.lock().unwrap();
        lock.push(Some(listener));
        lock.len() - 1
    }

    pub fn remove(&self, id: ListenerId) -> Option<T> {
        let mut lock = self.inner.lock().unwrap();
        if let Some(listener) = lock.get_mut(id) {
            let mut placeholder: Option<T> = None;
            mem::swap(listener, &mut placeholder);
            placeholder
        } else {
            None
        }
    }
}

impl<T> Clone for Listeners<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Default for Listeners<T> {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::default()),
        }
    }
}
