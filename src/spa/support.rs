/*
 * SPDX-License-Identifier: MIT
 */
use pipewire_wrapper_proc_macro::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct SupportRef {
    #[raw]
    raw: spa_sys::spa_support,
}
