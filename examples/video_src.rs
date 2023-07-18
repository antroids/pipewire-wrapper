extern crate pipewire_wrapper;

use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::core_api::main_loop::MainLoop;
use pipewire_wrapper::core_api::properties::Properties;
use pipewire_wrapper::properties_new;
use pipewire_wrapper::spa::pod::choice::none::PodNoneRef;
use pipewire_wrapper::spa::pod::choice::range::{PodRangeRef, PodRangeValue};
use pipewire_wrapper::spa::pod::id::PodIdRef;
use pipewire_wrapper::spa::pod::iterator::AllocatedPodIterator;
use pipewire_wrapper::spa::pod::object::format::{
    MediaSubType, MediaType, ObjectFormatType, VideoFormat,
};
use pipewire_wrapper::spa::pod::object::param_buffers::ParamBuffersType;
use pipewire_wrapper::spa::pod::object::{ObjectPropsIterator, ObjectType, PodObjectRef};
use pipewire_wrapper::spa::pod::pod_buf::{AllocatedData, PodBuf};
use pipewire_wrapper::spa::pod::{FromValue, PodFractionRef, PodRectangleRef, PodRef};
use pipewire_wrapper::spa::type_::{FractionRef, RectangleRef};
use pipewire_wrapper::stream::events::{StreamEvents, StreamEventsBuilder};
use pipewire_wrapper::stream::Stream;

struct State {
    core: Arc<Core>,
    loop_: Arc<MainLoop>,
}

const BPP: u32 = 3;
const CURSOR_WIDTH: u32 = 64;
const CURSOR_HEIGHT: u32 = 64;
const CURSOR_BPP: u32 = 4;

const MAX_BUFFERS: u32 = 64;

pub fn main() {
    let core = Arc::new(Core::default());
    let main_loop = core.context().main_loop();
    let state = Arc::new(Mutex::new(State {
        core: core.clone(),
        loop_: main_loop.clone(),
    }));

    let quit_main_loop = Box::new(|_| {
        main_loop.quit().unwrap();
    });
    let _sigint_handler = main_loop.add_signal(signal_hook::consts::SIGINT, quit_main_loop.clone());
    let _sigterm_handler = main_loop.add_signal(signal_hook::consts::SIGTERM, quit_main_loop);

    let stream = Stream::new(
        &core,
        CString::new("Test video source").unwrap().as_ref(),
        properties_new!(pw_sys::PW_KEY_MEDIA_CLASS => "Video/Source\0").as_ref(),
    )
    .unwrap();

    let listener = StreamEventsBuilder::default()
        .process(Box::new({
            let state = state.clone();
            let stream = stream.clone();
            || {}
        }))
        .param_changed(Box::new({
            let state = state.clone();
            let stream = stream.clone();
            |id, pod| {}
        }))
        .build();

    println!("Running main loop");
    main_loop.run().unwrap();
}

fn format_param() -> pipewire_wrapper::Result<AllocatedData<PodObjectRef>> {
    Ok(PodObjectRef::from_value(&ObjectType::OBJECT_FORMAT(
        AllocatedPodIterator::from_values([
            &ObjectFormatType::MEDIA_TYPE(&PodIdRef::from_value(&MediaType::VIDEO)?.as_pod()),
            &ObjectFormatType::MEDIA_SUBTYPE(&PodIdRef::from_value(&MediaSubType::RAW)?.as_pod()),
            &ObjectFormatType::VIDEO_FORMAT(&PodIdRef::from_value(&VideoFormat::RGB)?.as_pod()),
            &ObjectFormatType::VIDEO_SIZE(
                &PodRangeRef::<PodRectangleRef>::from_value(&PodRangeValue::new(
                    RectangleRef::new(320, 240),
                    RectangleRef::new(1, 1),
                    RectangleRef::new(4096, 4096),
                ))?
                .as_pod()
                .choice(),
            ),
            &ObjectFormatType::VIDEO_FRAMERATE(
                &PodNoneRef::<PodFractionRef>::from_value(&Some(FractionRef::new(25, 1)))?
                    .as_pod()
                    .choice(),
            ),
        ])?
        .iter(),
    ))?)
}

// fn stream_params(
//     size: RectangleRef,
//     stride: u32,
// ) -> pipewire_wrapper::Result<Vec<AllocatedData<PodObjectRef>>> {
//     Ok(vec![PodObjectRef::from_value(
//         &ObjectType::OBJECT_PARAM_BUFFERS(
//             AllocatedPodIterator::from_values([&ParamBuffersType::BUFFERS()])?.iter(),
//         ),
//     )?])
// }
