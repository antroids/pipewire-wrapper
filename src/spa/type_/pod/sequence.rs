use pipewire_proc_macro::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
struct PodSequenceBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_sequence_body,
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct PodSequenceRef {
    #[raw]
    raw: spa_sys::spa_pod_sequence,
}
