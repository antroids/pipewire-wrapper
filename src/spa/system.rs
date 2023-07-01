use pipewire_proc_macro::{spa_interface, RawWrapper};

use crate::wrapper::*;

#[derive(RawWrapper, Debug)]
#[spa_interface(methods=spa_sys::spa_system_methods)]
#[repr(transparent)]
pub struct SystemRef {
    #[raw]
    raw: spa_sys::spa_system,
}
