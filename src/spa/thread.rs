use pipewire_proc_macro::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ThreadRef {
    #[raw]
    raw: spa_sys::spa_thread,
}
