/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::Debug;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{mpsc, Mutex};

use crate::core_api::loop_::{Loop, LoopRef};
use crate::spa::loop_::EventSource;

#[derive(Debug)]
pub enum SendError<T> {
    SendError(mpsc::SendError<T>),
    CannotSignalEvent(crate::Error),
}

impl<T> From<SendError<T>> for crate::Error {
    fn from(value: SendError<T>) -> Self {
        match value {
            SendError::SendError(e) => {
                crate::Error::ErrorMessage("Receiver is disconnected, unable to send message")
            }
            SendError::CannotSignalEvent(e) => e,
        }
    }
}

pub struct LoopChannel<L: Loop> {
    event: Option<EventSource<'static, L>>,
}

impl<L: Loop> LoopChannel<L> {
    pub fn channel<T>() -> (Sender<T, L>, Receiver<T, L>) {
        Self::from_channel(mpsc::channel())
    }

    pub fn from_channel<T>(
        (sender, receiver): (mpsc::Sender<T>, mpsc::Receiver<T>),
    ) -> (Sender<T, L>, Receiver<T, L>) {
        let channel = Rc::new(Mutex::new(LoopChannel { event: None }));
        (
            Sender {
                channel: channel.clone(),
                sender,
            },
            Receiver { channel, receiver },
        )
    }
}

#[derive(Clone)]
pub struct Sender<T, L: Loop> {
    sender: mpsc::Sender<T>,
    channel: Rc<Mutex<LoopChannel<L>>>,
}

impl<T, L: Loop> Sender<T, L> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        let mut channel = self.channel.lock().unwrap();
        self.sender
            .send(value)
            .map_err(|e| SendError::SendError(e))?;
        if let LoopChannel { event: Some(event) } = channel.deref_mut() {
            event
                .signal()
                .map_err(|e| SendError::CannotSignalEvent(e))?;
        }
        Ok(())
    }

    pub fn into_sender(self) -> mpsc::Sender<T> {
        self.sender
    }

    pub fn detach(&self) {
        let mut channel = self.channel.lock().unwrap();
        channel.event = None;
    }
}

pub struct Receiver<T, L: Loop> {
    receiver: mpsc::Receiver<T>,
    channel: Rc<Mutex<LoopChannel<L>>>,
}

pub type ReceiverCallback<T> = Box<dyn FnMut(&mpsc::Receiver<T>)>;

impl<T: 'static, L: Loop> Receiver<T, L> {
    pub fn attach(self, loop_: &L, mut callback: ReceiverCallback<T>) -> crate::Result<()> {
        let channel = self.channel.clone();
        let event = loop_.add_event(Box::new({
            move |_count| {
                callback(&self.receiver);
            }
        }))?;
        let mut channel = channel.lock().unwrap();
        channel.event = Some(event);
        Ok(())
    }

    pub fn into_receiver(self) -> mpsc::Receiver<T> {
        self.receiver
    }
}
