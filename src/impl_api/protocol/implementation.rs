/*
 * SPDX-License-Identifier: MIT
 */
use pipewire_wrapper_proc_macro::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ImplementationRef {
    #[raw]
    raw: pw_sys::pw_protocol_implementation,
}
