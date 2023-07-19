use pipewire_proc_macro::RawWrapper;

use crate::spa::buffers;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct BufferRef {
    #[raw]
    raw: pw_sys::pw_buffer,
}

impl BufferRef {
    pub fn buffer(&self) -> &buffers::BufferRef {
        unsafe { buffers::BufferRef::from_raw_ptr(self.raw.buffer) }
    }

    pub fn buffer_mut(&mut self) -> &mut buffers::BufferRef {
        unsafe { buffers::BufferRef::mut_from_raw_ptr(self.raw.buffer) }
    }

    pub unsafe fn get_user_data<T>(&self) -> Option<&mut T> {
        (self.raw.user_data as *mut T).as_mut()
    }

    pub fn size(&self) -> u64 {
        self.raw.size
    }

    // pub fn requested(&self) -> u64 {
    //     self.raw.requested
    // }
}
