use bitflags::{bitflags, Flags};

use pipewire_proc_macro::RawWrapper;

use crate::spa::dict::DictRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ClientInfoRef {
    #[raw]
    raw: pw_sys::pw_client_info,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ChangeMask: u64 {
        const PROPS = pw_sys::PW_CLIENT_CHANGE_MASK_PROPS as u64;
        const ALL = pw_sys::PW_CLIENT_CHANGE_MASK_ALL as u64;
    }
}

impl ClientInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn change_mask(&self) -> ChangeMask {
        ChangeMask::from_bits_retain(self.raw.change_mask)
    }

    pub fn props(&self) -> &DictRef {
        unsafe { DictRef::from_raw_ptr(self.raw.props) }
    }
}
