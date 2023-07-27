use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Duration,
};

use pipewire_wrapper::core_api::main_loop::MainLoop;

/*
 * SPDX-License-Identifier: MIT
 */
#[test]
fn test_init_main_loop() {
    let main_loop = MainLoop::default();

    let callback = |_expirations| {
        main_loop.quit().unwrap();
    };
    let timer = main_loop.get_loop().add_timer(Box::new(callback)).unwrap();
    main_loop
        .get_loop()
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    main_loop.run().unwrap();
}

#[test]
fn test_sources() {
    let main_loop = MainLoop::default();

    let callback = || {
        println!("Idle");
    };
    let idle = main_loop
        .get_loop()
        .add_idle(false, Box::new(callback))
        .unwrap();
    main_loop.get_loop().enable_idle(&idle, false).unwrap();

    let callback = |signal_number| {
        println!("Signal: {:?}", signal_number);
    };
    let _signal = main_loop
        .get_loop()
        .add_signal(123, Box::new(callback))
        .unwrap();

    let event_signal = AtomicBool::new(false);
    let callback = |count| {
        println!("Event: count {:?}", count);
        event_signal.store(true, Ordering::Relaxed);
    };
    let event = main_loop.get_loop().add_event(Box::new(callback)).unwrap();

    let callback = |_expirations| {
        main_loop.get_loop().signal_event(&event).unwrap();
    };
    let timer = main_loop.get_loop().add_timer(Box::new(callback)).unwrap();
    main_loop
        .get_loop()
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    let callback = |_expirations| {
        main_loop.quit().unwrap();
    };
    let timer = main_loop.get_loop().add_timer(Box::new(callback)).unwrap();
    main_loop
        .get_loop()
        .update_timer(&timer, Duration::from_secs(3), Duration::ZERO, false)
        .unwrap();

    main_loop.run().unwrap();

    assert!(event_signal.load(Ordering::Relaxed))
}

#[test]
fn test_iterate_main_loop() {
    let main_loop = MainLoop::default();
    let loop_iterations = Mutex::new(0);

    let callback = |_expirations| {
        for _elapsed in main_loop.iter(100) {
            let mut loop_iterations = loop_iterations.lock().unwrap();
            *loop_iterations += 1;
            if *loop_iterations == 10 {
                break;
            }
        }

        main_loop.quit().unwrap();

        assert_eq!(*loop_iterations.lock().unwrap(), 10)
    };
    let timer = main_loop.get_loop().add_timer(Box::new(callback)).unwrap();
    main_loop
        .get_loop()
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    main_loop.run().unwrap();
}
