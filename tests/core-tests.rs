/*
 * SPDX-License-Identifier: MIT
 */

use pipewire_wrapper::core_api::{core::Core, registry::events::RegistryEventsBuilder};
use pipewire_wrapper::listeners::OwnListeners;
use std::time::Duration;

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

    let _timer = main_loop.quit_after(Duration::from_secs(1)).unwrap();

    main_loop.run().unwrap();
}
