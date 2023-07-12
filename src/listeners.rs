use std::mem;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::wrapper::{RawWrapper, Wrapper};

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

pub trait AddListener<'a>: RawWrapper {
    type Events: 'a;

    #[must_use]
    fn add_listener(&self, events: Pin<Box<Self::Events>>) -> Pin<Box<Self::Events>>;
}

pub trait OwnListeners<'a>
where
    Self: Wrapper,
    <Self as Wrapper>::RawWrapperType: AddListener<'a>,
{
    fn listeners(
        &self,
    ) -> &Listeners<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>>;

    fn add_listener(
        &self,
        events: Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>,
    ) -> ListenerId {
        let raw_wrapper = unsafe { Self::RawWrapperType::from_raw_ptr(self.as_raw_ptr().cast()) };
        let mut listener = raw_wrapper.add_listener(events);
        self.listeners().add(listener)
    }

    fn remove_listener(
        &'a mut self,
        id: ListenerId,
    ) -> Option<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>> {
        self.listeners().remove(id)
    }
}
