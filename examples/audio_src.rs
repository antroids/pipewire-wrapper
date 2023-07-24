use std::ffi::CString;
use std::mem::size_of;
use std::ops::Add;
use std::slice;
use std::sync::{Arc, Mutex};

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::listeners::OwnListeners;
use pipewire_wrapper::properties_new;
use pipewire_wrapper::spa::param::ParamType;
use pipewire_wrapper::spa::pod::choice::enum_::PodEnumValue;
use pipewire_wrapper::spa::pod::id::PodIdType;
use pipewire_wrapper::spa::pod::iterator::AllocatedPodIterator;
use pipewire_wrapper::spa::pod::object::enum_format::ObjectEnumFormatType;
use pipewire_wrapper::spa::pod::object::format::{AudioFormat, MediaSubType, MediaType};
use pipewire_wrapper::spa::pod::object::param_port_config::Direction;
use pipewire_wrapper::spa::pod::object::{ObjectType, PodObjectRef};
use pipewire_wrapper::spa::pod::pod_buf::AllocatedData;
use pipewire_wrapper::spa::pod::{FromPrimitiveValue, PodIntRef, Upcast};
use pipewire_wrapper::stream::events::StreamEventsBuilder;
use pipewire_wrapper::stream::{Stream, StreamFlags};

const RATE: f64 = 44100f64;
const CHANNELS: u32 = 2;
const VOLUME: f64 = 0.7f64;

const PI_POW_2: f64 = std::f64::consts::PI * std::f64::consts::PI;

type AudioDataType = f32;

pub fn main() {
    let core = Arc::new(Core::default());
    let main_loop = core.context().main_loop();

    let quit_main_loop = Box::new(|_| {
        main_loop.quit().unwrap();
    });
    let _sigint_handler = main_loop.add_signal(signal_hook::consts::SIGINT, quit_main_loop.clone());
    let _sigterm_handler = main_loop.add_signal(signal_hook::consts::SIGTERM, quit_main_loop);

    let stream = Arc::new(
        Stream::new(
            &core,
            CString::new("Test audio source").unwrap().as_ref(),
            properties_new!(
                pw_sys::PW_KEY_MEDIA_TYPE => "Audio\0", 
                pw_sys::PW_KEY_MEDIA_CATEGORY => "Playback\0", 
                pw_sys::PW_KEY_MEDIA_ROLE => "Music\0"),
        )
        .unwrap(),
    );

    let accumulator = Arc::new(Mutex::new(0f64));
    let listener = StreamEventsBuilder::default()
        .process(Box::new({
            let stream = stream.clone();
            move || on_process(&stream, &accumulator)
        }))
        .build();
    stream.add_listener(listener);
    let param = format_param().unwrap();
    stream
        .connect(
            Direction::OUTPUT,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &[param.as_pod().upcast()],
        )
        .unwrap();

    println!("Running main loop");
    main_loop.run().unwrap();
}

fn format_param() -> pipewire_wrapper::Result<AllocatedData<PodObjectRef>> {
    Ok(PodObjectRef::from_id_and_value(
        ParamType::ENUM_FORMAT,
        &ObjectType::OBJECT_ENUM_FORMAT(
            AllocatedPodIterator::from_values(&[
                ObjectEnumFormatType::MEDIA_TYPE(MediaType::AUDIO.as_alloc_pod().as_pod()),
                ObjectEnumFormatType::MEDIA_SUBTYPE(MediaSubType::RAW.as_alloc_pod().as_pod()),
                ObjectEnumFormatType::AUDIO_FORMAT(
                    PodEnumValue::from_default(AudioFormat::F32)
                        .to_alloc_pod()?
                        .as_pod()
                        .choice(),
                ),
                ObjectEnumFormatType::AUDIO_CHANNELS(
                    PodIntRef::from_primitive(CHANNELS as i32)?.as_pod().into(),
                ),
                ObjectEnumFormatType::AUDIO_RATE(
                    PodIntRef::from_primitive(RATE as i32)?.as_pod().into(),
                ),
            ])?
            .iter(),
        ),
    )?)
}

fn on_process(stream: &Arc<Stream>, accumulator: &Arc<Mutex<f64>>) {
    if let Some(buf) = stream.dequeue_buffer() {
        let spa_buf = buf.buffer_mut();
        if let Some(data) = spa_buf.datas_mut().first_mut() {
            if !data.data().is_null() {
                let stride = CHANNELS * size_of::<AudioDataType>() as u32;
                let n_frames = data.max_size() / stride; // Since 0.3.49 buf.requested can be used here
                let data_slice = unsafe {
                    slice::from_raw_parts_mut(
                        data.data() as *mut AudioDataType,
                        (n_frames * CHANNELS) as usize,
                    )
                };

                let mut accumulator = accumulator.lock().unwrap();
                for i in 0..n_frames {
                    *accumulator = accumulator.add(PI_POW_2 * 440f64 / RATE) % PI_POW_2;
                    let val = (accumulator.sin() * VOLUME * 16767f64) as AudioDataType;
                    for c in 0..CHANNELS {
                        data_slice[(i * CHANNELS + c) as usize] = val;
                    }
                }

                let chunk = data.chunk_mut();
                chunk.set_offset(0);
                chunk.set_stride(stride as i32);
                chunk.set_size(n_frames * stride);

                stream.queue_buffer(buf).unwrap();
            }
        }
    }
}
