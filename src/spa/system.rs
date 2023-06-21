use pipewire_proc_macro::{RawWrapper, spa_interface};

use crate::wrapper::*;

#[derive(RawWrapper, Debug)]
#[spa_interface(methods=spa_sys::spa_system_methods)]
#[repr(transparent)]
pub struct SystemRef {
    #[raw]
    raw: spa_sys::spa_system,
}
