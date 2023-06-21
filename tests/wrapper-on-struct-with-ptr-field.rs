use std::ops::Deref;

use pipewire_macro::*;

type TestCType = i32;

#[derive(RefWrapper)]
struct TestRef {
    #[pointer]
    ptr_: TestCType,
}

impl Default for TestRef {
    fn default() -> Self {
        Self { ptr_: 123i32 }
    }
}

#[derive(Wrapper)]
struct Test {
    #[reference]
    ref_: TestRef,
}

impl Drop for Test {
    fn drop(&mut self) {
        ()
    }
}

impl Default for Test {
    fn default() -> Self {
        Self {
            ref_: TestRef::default(),
        }
    }
}

fn main() {
    let ref_wrapper = TestRef::default();
    assert_eq!(*ref_wrapper.as_raw(), 123i32);

    let wrapper = Test::default();
    assert_eq!(*wrapper.as_ref().as_raw(), 123i32);
    assert_eq!(*wrapper.deref().as_raw(), 123i32)
}
