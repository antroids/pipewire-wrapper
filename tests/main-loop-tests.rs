/*
 * SPDX-License-Identifier: MIT
 */

mod tests {
    use pipewire_wrapper::core_api::loop_::Loop;
    use std::rc::Rc;
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Mutex,
        },
        time::Duration,
    };

    use pipewire_wrapper::core_api::main_loop::MainLoop;

    #[test]
    fn test_init_main_loop() {
        let main_loop = MainLoop::default();

        let _timer = main_loop.quit_after(Duration::from_secs(1)).unwrap();

        main_loop.run().unwrap();
    }

    #[test]
    fn test_sources() {
        let main_loop = MainLoop::default();

        let idle = main_loop
            .add_idle(false, || {
                println!("Idle");
            })
            .unwrap();
        idle.enable(false).unwrap();

        let _signal = main_loop
            .add_signal(123, |signal_number| {
                println!("Signal: {:?}", signal_number);
            })
            .unwrap();

        let event_signal = Rc::new(AtomicBool::new(false));
        let event = main_loop
            .add_event({
                let event_signal = event_signal.clone();
                move |count| {
                    println!("Event: count {:?}", count);
                    event_signal.store(true, Ordering::Relaxed);
                }
            })
            .unwrap();

        let timer = main_loop
            .add_timer({
                move |_expirations| {
                    let _ = &event.signal().unwrap();
                }
            })
            .unwrap();
        timer
            .update(Duration::from_secs(1), Duration::ZERO, false)
            .unwrap();

        let _timer = main_loop.quit_after(Duration::from_secs(3)).unwrap();

        main_loop.run().unwrap();

        assert!(event_signal.load(Ordering::Relaxed))
    }

    #[test]
    fn test_iterate_main_loop() {
        let main_loop = MainLoop::default();
        let loop_iterations = Mutex::new(0);

        let callback = {
            let main_loop = main_loop.clone();
            move |_expirations| {
                for _elapsed in main_loop.iter(100) {
                    let mut loop_iterations = loop_iterations.lock().unwrap();
                    *loop_iterations += 1;
                    if *loop_iterations == 10 {
                        break;
                    }
                }

                main_loop.quit().unwrap();

                assert_eq!(*loop_iterations.lock().unwrap(), 10)
            }
        };
        let timer = main_loop.add_timer(callback).unwrap();
        timer
            .update(Duration::from_secs(1), Duration::ZERO, false)
            .unwrap();

        main_loop.run().unwrap();
    }
}
