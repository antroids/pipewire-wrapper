use pipewire_proc_macro::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct WorkQueueRef {
    #[raw]
    raw: pw_sys::pw_work_queue,
}
