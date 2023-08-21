/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::Debug;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{mpsc, Mutex};

use crate::core_api::loop_::LoopRef;
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

pub struct LoopChannel<'a> {
    event: Option<EventSource<'a, LoopRef>>,
}

impl<'a> LoopChannel<'a> {
    pub fn channel<T>() -> (Sender<'a, T>, Receiver<'a, T>) {
        Self::from_channel(mpsc::channel())
    }

    pub fn from_channel<T>(
        (sender, receiver): (mpsc::Sender<T>, mpsc::Receiver<T>),
    ) -> (Sender<'a, T>, Receiver<'a, T>) {
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
pub struct Sender<'a, T> {
    sender: mpsc::Sender<T>,
    channel: Rc<Mutex<LoopChannel<'a>>>,
}

impl<'a, T> Sender<'a, T> {
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

pub struct Receiver<'a, T: 'a> {
    receiver: mpsc::Receiver<T>,
    channel: Rc<Mutex<LoopChannel<'a>>>,
}

pub type ReceiverCallback<'a, T> = Box<dyn FnMut(&mpsc::Receiver<T>) + 'a>;

impl<'a, T: 'a> Receiver<'a, T> {
    pub fn attach(
        self,
        loop_: &'a LoopRef,
        mut callback: ReceiverCallback<'a, T>,
    ) -> crate::Result<()> {
        let channel = self.channel.clone();
        let event = loop_.utils().add_event(
            loop_,
            Box::new({
                move |_count| {
                    callback(&self.receiver);
                }
            }),
        )?;
        let mut channel = channel.lock().unwrap();
        channel.event = Some(event);
        Ok(())
    }

    pub fn into_receiver(self) -> mpsc::Receiver<T> {
        self.receiver
    }
}
