use pipewire_proc_macro::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
struct PodPointerBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_pointer_body,
}
