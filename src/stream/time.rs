/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};

use pipewire_wrapper_proc_macro::RawWrapper;

use crate::spa::type_::FractionRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct TimeRef {
    #[raw]
    raw: pw_sys::pw_time,
}

impl TimeRef {
    pub fn now(&self) -> i64 {
        self.raw.now
    }

    pub fn rate(&self) -> &FractionRef {
        unsafe { FractionRef::from_raw_ptr(&self.raw.rate) }
    }

    pub fn ticks(&self) -> u64 {
        self.raw.ticks
    }

    pub fn delay(&self) -> i64 {
        self.raw.delay
    }

    pub fn queued(&self) -> u64 {
        self.raw.queued
    }
}

impl Debug for TimeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimeRef")
            .field("now", &self.now())
            .field("rate", &self.rate())
            .field("ticks", &self.ticks())
            .field("delay", &self.delay())
            .field("queued", &self.queued())
            .finish()
    }
}

impl Default for TimeRef {
    fn default() -> Self {
        TimeRef::from_raw(pw_sys::pw_time {
            now: 0,
            rate: spa_sys::spa_fraction { num: 0, denom: 0 },
            ticks: 0,
            delay: 0,
            queued: 0,
        })
    }
}
