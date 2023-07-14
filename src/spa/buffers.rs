use std::slice;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::buffers::data::DataRef;
use crate::spa::buffers::meta::MetaRef;

pub mod chunk;
pub mod data;
pub mod meta;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct BufferRef {
    #[raw]
    raw: spa_sys::spa_buffer,
}

impl BufferRef {
    pub fn metas(&self) -> &[MetaRef] {
        unsafe { slice::from_raw_parts(self.raw.metas.cast(), self.raw.n_metas as usize) }
    }

    pub fn datas(&self) -> &[DataRef] {
        unsafe { slice::from_raw_parts(self.raw.datas.cast(), self.raw.n_datas as usize) }
    }
}
