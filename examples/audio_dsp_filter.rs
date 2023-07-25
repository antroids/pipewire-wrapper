use std::ffi::CString;
use std::sync::Arc;

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::filter::events::FilterEventsBuilder;
use pipewire_wrapper::filter::{Filter, FilterFlags, PortFlags};
use pipewire_wrapper::listeners::OwnListeners;
use pipewire_wrapper::properties_new;
use pipewire_wrapper::spa::param::ParamType;
use pipewire_wrapper::spa::pod::iterator::AllocatedPodIterator;
use pipewire_wrapper::spa::pod::object::param_port_config::Direction;
use pipewire_wrapper::spa::pod::object::param_process_latency::ParamProcessLatencyType;
use pipewire_wrapper::spa::pod::object::{ObjectType, PodObjectRef};
use pipewire_wrapper::spa::pod::{FromPrimitiveValue, PodLongRef, Upcast};

type AudioDataType = f32;

struct CustomPort {}

pub fn main() {
    let core = Arc::new(Core::default());
    let main_loop = core.context().main_loop();

    let quit_main_loop = Box::new(|_| {
        main_loop.quit().unwrap();
    });
    let _sigint_handler = main_loop
        .get_loop()
        .add_signal(signal_hook::consts::SIGINT, quit_main_loop.clone());
    let _sigterm_handler = main_loop
        .get_loop()
        .add_signal(signal_hook::consts::SIGTERM, quit_main_loop);

    let latency_param = PodObjectRef::from_id_and_value(
        ParamType::PROCESS_LATENCY,
        &ObjectType::OBJECT_PARAM_PROCESS_LATENCY(
            AllocatedPodIterator::from_values(&[ParamProcessLatencyType::NS(
                PodLongRef::from_primitive(10_000_000).unwrap().as_pod(),
            )])
            .unwrap()
            .iter(),
        ),
    )
    .unwrap();

    let filter_name = CString::new("Test filter").unwrap();
    let mut filter = Filter::<CustomPort>::new(
        &core,
        filter_name.as_ref(),
        properties_new!(
        pw_sys::PW_KEY_MEDIA_TYPE => "Audio\0",
        pw_sys::PW_KEY_MEDIA_CATEGORY => "Filter\0",
        pw_sys::PW_KEY_MEDIA_ROLE => "DSP\0"),
    )
    .unwrap();
    let input_port = filter
        .add_port(
            CustomPort {},
            Direction::INPUT,
            PortFlags::MAP_BUFFERS,
            properties_new!(
        pw_sys::PW_KEY_FORMAT_DSP => "32 bit float mono audio\0",
        pw_sys::PW_KEY_PORT_NAME => "input\0"),
            None,
        )
        .unwrap();
    let output_port = filter
        .add_port(
            CustomPort {},
            Direction::OUTPUT,
            PortFlags::MAP_BUFFERS,
            properties_new!(
        pw_sys::PW_KEY_FORMAT_DSP => "32 bit float mono audio\0",
        pw_sys::PW_KEY_PORT_NAME => "output\0"),
            None,
        )
        .unwrap();
    let filter = Arc::new(filter);
    let events = FilterEventsBuilder::<CustomPort>::default()
        .process(Box::new({
            let filter = filter.clone();
            move |pos| {
                let n_samples = pos.clock().duration() as u32;
                let input_buf = filter.get_dsp_buffer::<AudioDataType>(&input_port, n_samples);
                let output_buf = filter.get_dsp_buffer::<AudioDataType>(&output_port, n_samples);
                if let (Ok(input_buf), Ok(output_buf)) = (input_buf, output_buf) {
                    output_buf.copy_from_slice(input_buf);
                }
            }
        }))
        .build();
    filter.add_listener(events);

    filter
        .connect(FilterFlags::RT_PROCESS, &[latency_param.as_pod().upcast()])
        .unwrap();

    println!("Running main loop");
    main_loop.run().unwrap();
}
