/*
 * SPDX-License-Identifier: MIT
 */

//! Listeners storage
//!
use std::mem;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::wrapper::{RawWrapper, Wrapper};

pub type ListenerId = usize;

/// Owned listeners storage.
/// Each listener associated with constant [ListenerId].
/// Can be cloned between threads.
#[derive(Debug)]
pub struct Listeners<T> {
    inner: Arc<Mutex<Vec<Option<T>>>>,
}

impl<T> Listeners<T> {
    /// Add new listener to the storage.
    ///
    /// # Arguments
    ///
    /// * `listener` - new listener
    ///
    /// Returns [ListenerId] associated with the given listener in this storage.
    pub fn add(&self, listener: T) -> ListenerId {
        let mut lock = self.inner.lock().unwrap();
        lock.push(Some(listener));
        lock.len() - 1
    }

    /// Remove listener by [ListenerId].
    ///
    /// # Arguments
    ///
    /// * `id` - [ListenerId]
    ///
    /// Returns the listener or None
    ///
    /// # Notes
    ///
    /// The listener will be unsubscribed after drop.
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

    /// Whether the storage contains listener with the given id.
    pub fn contains(&self, id: ListenerId) -> bool {
        self.inner
            .lock()
            .unwrap()
            .get(id)
            .map_or(false, |l| l.is_some())
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

/// [RawWrapper] that can register listener.
/// Ownership of the listener is not taken, so it's should be stored somewhere.
/// If this is possible, owned wrapper with the [OwnListeners] trait is preferred for usage.
pub trait AddListener<'a>: RawWrapper {
    /// Events listener struct
    type Events: 'a;

    /// Register listener and return it.
    ///
    /// # Arguments
    ///
    /// * `events` - Events listener struct
    ///
    /// Returns the registered listener instance. It should be alive to receive events.
    #[must_use]
    fn add_listener(&self, events: Pin<Box<Self::Events>>) -> Pin<Box<Self::Events>>;
}

/// [Wrapper] that can register and store events listeners.
///
/// # Example
///
/// ```no_run,ignore
/// use pipewire_wrapper::core_api::device::DeviceRef;
/// use pipewire_wrapper::core_api::registry::events::RegistryEventsBuilder;
/// use crate::pipewire_wrapper::core_api::proxy::Proxied;
/// let listener = RegistryEventsBuilder::default()
///         .global(Box::new(
///             move |id, _permissions, type_info, _version, _props| {
///                 if type_info == DeviceRef::type_info() {
///                     device_added_queue.lock().unwrap().push(id);
///                     main_loop.signal_event(&device_added_event).unwrap();
///                 }
///             },
///         ))
///         .build();
/// registry.add_listener(listener)
/// ```
pub trait OwnListeners<'a>
where
    Self: Wrapper,
    <Self as Wrapper>::RawWrapperType: AddListener<'a>,
{
    /// Listeners storege
    fn listeners(
        &self,
    ) -> &Listeners<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>>;

    /// Register new listener and add it to storage.
    ///
    /// # Arguments
    ///
    /// * `events` - new events listener
    ///
    /// Returns [ListenerId] that can be used to remove listener from storage.
    fn add_listener(
        &self,
        events: Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>,
    ) -> ListenerId {
        let raw_wrapper = unsafe { Self::RawWrapperType::from_raw_ptr(self.as_raw_ptr().cast()) };
        let mut listener = raw_wrapper.add_listener(events);
        self.listeners().add(listener)
    }

    /// Remove listener with the given [ListenerId] from storage.
    ///
    /// # Notes
    ///
    /// The listener will be unsubscribed after drop.
    fn remove_listener(
        &'a mut self,
        id: ListenerId,
    ) -> Option<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>> {
        self.listeners().remove(id)
    }

    /// Whether the storage contains listener with the given id.
    fn contains_listener(&self, id: ListenerId) -> bool {
        self.listeners().contains(id)
    }
}
