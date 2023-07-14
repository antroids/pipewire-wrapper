use std::os::fd::RawFd;

use bitflags::bitflags;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::buffers::chunk::ChunkRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct DataRef {
    #[raw]
    raw: spa_sys::spa_data,
}

impl DataRef {
    pub fn type_(&self) -> DataType {
        DataType::from_raw(self.raw.type_)
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits_retain(self.raw.flags)
    }

    pub fn fd(&self) -> RawFd {
        self.raw.fd as RawFd
    }

    pub fn map_offset(&self) -> u32 {
        self.raw.mapoffset
    }

    pub fn max_size(&self) -> u32 {
        self.raw.maxsize
    }

    pub fn data(&self) -> *mut ::std::os::raw::c_void {
        self.raw.data
    }

    pub fn chunk(&self) -> &ChunkRef {
        unsafe { ChunkRef::from_raw_ptr(self.raw.chunk) }
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct Flags: u32 {
        const NONE = spa_sys::SPA_DATA_FLAG_NONE;
        const READABLE = spa_sys::SPA_DATA_FLAG_READABLE;
        const WRITABLE = spa_sys::SPA_DATA_FLAG_WRITABLE;
        const DYNAMIC = spa_sys::SPA_DATA_FLAG_DYNAMIC;
        const READWRITE = spa_sys::SPA_DATA_FLAG_READWRITE;
    }
}

enum_wrapper!(
    DataType,
    spa_sys::spa_data_type,
    INVALID: spa_sys::SPA_DATA_Invalid,
    MEM_PTR: spa_sys::SPA_DATA_MemPtr,
    MEM_FD: spa_sys::SPA_DATA_MemFd,
    DMA_BUF: spa_sys::SPA_DATA_DmaBuf,
    MEM_ID: spa_sys::SPA_DATA_MemId,
    _LAST: spa_sys::_SPA_DATA_LAST,
);
