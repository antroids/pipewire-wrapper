use std::ffi::CStr;

use pipewire_proc_macro::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ClientInfoRef {
    #[raw]
    raw: pw_sys::pw_client_info,
}

impl ClientInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    //todo change_mask
}
