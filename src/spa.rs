use std::time::Duration;

use pipewire_proc_macro::RawWrapper;

pub mod dict;
pub mod handle;
pub mod interface;
pub mod list;
pub mod loop_;
pub mod param;
pub mod support;
pub mod system;
pub mod thread;
pub mod type_;

pub const SPA_ID_INVALID: u32 = 0xffffffff; // Missing in the bindings for some reason

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct TimespecRef {
    #[raw]
    raw: spa_sys::timespec,
}

impl TryFrom<Duration> for TimespecRef {
    type Error = crate::error::Error;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        Ok(Self {
            raw: spa_sys::timespec {
                tv_sec: value
                    .as_secs()
                    .try_into()
                    .map_err(|_| crate::error::Error::WrongTimeFormat)?,
                tv_nsec: value
                    .subsec_nanos()
                    .try_into()
                    .map_err(|_| crate::error::Error::WrongTimeFormat)?,
            },
        })
    }
}
