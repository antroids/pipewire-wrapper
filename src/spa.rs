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
