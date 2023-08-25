/*
 * SPDX-License-Identifier: MIT
 */

use std::time::Duration;

use pipewire_wrapper::{
    core_api::{
        context::{events::ContextEventsBuilder, Context},
        core::Core,
        properties::Properties,
    },
    impl_api::global::GlobalRef,
    listeners::AddListener,
    wrapper::RawWrapper,
};

#[test]
fn test_context_init() {
    let context = Context::default();
    let main_loop = context.main_loop().clone();

    let _timer = main_loop.quit_after(Duration::from_secs(1)).unwrap();

    context.main_loop().run().unwrap();
}

#[test]
fn test_context_events() {
    let context = Context::default();
    let main_loop = context.main_loop().clone();

    let events = ContextEventsBuilder::default()
        .global_added(Box::new(|global| {
            println!("Global added {:?}", global);
        }))
        .build();
    let _events = context.add_listener(events);

    let _core = Core::connect(context.clone(), Properties::default()).unwrap();

    let _timer = main_loop.quit_after(Duration::from_secs(1)).unwrap();

    context.main_loop().run().unwrap();
}

#[test]
fn test_for_each_global() {
    let context = Context::default();

    context
        .for_each_global(|global: &GlobalRef| {
            println!("Global {:?}", global.as_raw_ptr());
            0i32
        })
        .unwrap();
}
