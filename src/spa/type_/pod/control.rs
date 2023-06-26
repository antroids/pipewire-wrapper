use pipewire_proc_macro::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct PodControlRef {
    #[raw]
    raw: spa_sys::spa_pod_control,
}
