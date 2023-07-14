use bitflags::bitflags;

use pipewire_proc_macro::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ChunkRef {
    #[raw]
    raw: spa_sys::spa_chunk,
}

impl ChunkRef {
    pub fn offset(&self) -> u32 {
        self.raw.offset
    }

    pub fn size(&self) -> u32 {
        self.raw.size
    }

    pub fn stride(&self) -> i32 {
        self.raw.stride
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits_retain(self.raw.flags)
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct Flags: i32 {
        const NONE = spa_sys::SPA_CHUNK_FLAG_NONE as i32;
        const CORRUPTED = spa_sys::SPA_CHUNK_FLAG_CORRUPTED as i32;
        //const EMPTY = spa_sys::SPA_CHUNK_FLAG_EMPTY;
    }
}
