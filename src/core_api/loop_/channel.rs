use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use crate::core_api::loop_::LoopRef;
use crate::spa::loop_::EventSource;

pub enum ChannelError {
    ReceiverNotAttached,
    ChannelPoisoned,
}

impl Debug for ChannelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelError::ReceiverNotAttached => write!(f, "Receiver is not attached to the loop"),
            ChannelError::ChannelPoisoned => write!(f, "Channel was poisoned"),
        }
    }
}

impl From<ChannelError> for crate::error::Error {
    fn from(value: ChannelError) -> Self {
        crate::error::Error::ChannelError(value)
    }
}

pub type ChannelResult<T> = Result<T, ChannelError>;

pub struct Channel<'a, T> {
    loop_: Option<&'a LoopRef>,
    event: Option<EventSource<'a>>,
    buf: VecDeque<T>,
}

impl<'a, T> Channel<'a, T> {
    pub fn channel() -> (Sender<'a, T>, Receiver<'a, T>) {
        let channel = Arc::new(Mutex::new(Channel {
            loop_: None,
            event: None,
            buf: VecDeque::new(),
        }));
        (
            Sender {
                channel: channel.clone(),
            },
            Receiver { channel },
        )
    }
}

pub struct Sender<'a, T> {
    channel: Arc<Mutex<Channel<'a, T>>>,
}

impl<'a, T> Sender<'a, T> {
    pub fn send(&mut self, value: T) -> ChannelResult<()> {
        let mut channel = self.channel.lock().unwrap();
        if let Channel {
            loop_: Some(loop_),
            event: Some(event),
            buf,
        } = channel.deref_mut()
        {
            buf.push_back(value);
            loop_.utils().signal_event(&event).unwrap();
            Ok(())
        } else {
            Err(ChannelError::ReceiverNotAttached)
        }
    }
}

pub struct Receiver<'a, T> {
    channel: Arc<Mutex<Channel<'a, T>>>,
}

impl<'a, T: 'a> Receiver<'a, T> {
    pub fn attach(
        &mut self,
        loop_: &'a LoopRef,
        mut callback: Box<dyn FnMut(&mut VecDeque<T>) + 'a>,
    ) {
        let event = loop_
            .utils()
            .add_event(
                loop_,
                Box::new({
                    let channel = self.channel.clone();
                    move |_count| {
                        callback(&mut channel.lock().unwrap().buf);
                    }
                }),
            )
            .unwrap();
        let mut channel = self.channel.lock().unwrap();
        channel.event = Some(event);
        channel.loop_ = Some(loop_);
    }
}
