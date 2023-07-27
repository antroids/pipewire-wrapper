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

/*
 * SPDX-License-Identifier: MIT
 */
#[test]
fn test_context_init() {
    let context = Context::default();

    let timer_callback = |_| {
        context.main_loop().quit().unwrap();
    };
    let timer = context
        .main_loop()
        .get_loop()
        .add_timer(Box::new(timer_callback))
        .unwrap();
    context
        .main_loop()
        .get_loop()
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    context.main_loop().run().unwrap();
}

#[test]
fn test_context_events() {
    let context = std::sync::Arc::new(Context::default());

    let events = ContextEventsBuilder::default()
        .global_added(Box::new(|global| {
            println!("Global added {:?}", global);
        }))
        .build();
    let _events = context.add_listener(events);

    let _core = Core::connect(&context, Properties::default()).unwrap();

    let timer_callback = |_| {
        context.main_loop().quit().unwrap();
    };
    let timer = context
        .main_loop()
        .get_loop()
        .add_timer(Box::new(timer_callback))
        .unwrap();
    context
        .main_loop()
        .get_loop()
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

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
