use std::time::Duration;

use pipewire_wrapper::{
    core_api::{core::Core, registry::events::RegistryEventsBuilder},
    listeners::AddListener,
};

/*
 * SPDX-License-Identifier: MIT
 */
#[test]
fn test_create_core() {
    let core = Core::default();
    let context = core.context();
    let main_loop = context.main_loop();
    let registry = core.get_registry(0).unwrap();

    let _registry_events = registry.add_listener(
        RegistryEventsBuilder::default()
            .global(Box::new(
                |_id, permissions, type_info, version, properties| {
                    println!(
                        "Global {:?} {:?} {:?} {:?}",
                        permissions, type_info, version, properties
                    );
                },
            ))
            .build(),
    );

    let timer_callback = |_| {
        core.context().main_loop().quit().unwrap();
    };
    let timer = main_loop
        .get_loop()
        .add_timer(Box::new(timer_callback))
        .unwrap();
    main_loop
        .get_loop()
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    main_loop.run().unwrap();
}
