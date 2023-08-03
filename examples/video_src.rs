/*
 * SPDX-License-Identifier: MIT
 */
extern crate pipewire_wrapper;

#[cfg(feature = "spa-pod-object-builders")]
mod video_src {
    use std::ffi::CString;
    use std::mem::size_of;
    use std::ops::AddAssign;
    use std::rc::Rc;
    use std::sync::Mutex;
    use std::time::Duration;

    use pipewire_wrapper::core_api::core::Core;
    use pipewire_wrapper::core_api::main_loop::MainLoop;
    use pipewire_wrapper::listeners::OwnListeners;
    use pipewire_wrapper::spa::buffers::meta::{
        HeaderFlags, MetaBitmapRef, MetaCursorRef, MetaData,
    };
    use pipewire_wrapper::spa::loop_::TimerSource;
    use pipewire_wrapper::spa::param::ParamType;
    use pipewire_wrapper::spa::pod::choice::enum_::PodEnumValue;
    use pipewire_wrapper::spa::pod::choice::range::PodRangeValue;
    use pipewire_wrapper::spa::pod::choice::ChoiceStructType;
    use pipewire_wrapper::spa::pod::object::enum_format::ObjectEnumFormatBuilder;
    use pipewire_wrapper::spa::pod::object::format::{
        MediaSubType, MediaType, ObjectFormatType, VideoFormat,
    };
    use pipewire_wrapper::spa::pod::object::param_buffers::ParamBuffersBuilder;
    use pipewire_wrapper::spa::pod::object::param_meta::{MetaType, ParamMetaBuilder};
    use pipewire_wrapper::spa::pod::object::param_port_config::Direction;
    use pipewire_wrapper::spa::pod::object::{ObjectType, PodObjectRef};
    use pipewire_wrapper::spa::pod::pod_buf::AllocPod;
    use pipewire_wrapper::spa::pod::{BasicType, PodRef, PodValue, Upcast};
    use pipewire_wrapper::spa::type_::{FractionRef, RectangleRef};
    use pipewire_wrapper::stream::events::StreamEventsBuilder;
    use pipewire_wrapper::stream::{Stream, StreamFlags};
    use pipewire_wrapper::{properties_new, stream};

    struct State<'a> {
        loop_: Rc<MainLoop>,
        size: Option<RectangleRef>,
        timeout_timer: TimerSource<'a>,

        seq: u64,
        counter: u32,
        crop: f64,
        accumulator: f64,
    }

    impl State<'_> {
        pub fn stride(&self) -> Option<u32> {
            self.size.map(|size| ((size.width() * BPP) + 3) & !3)
        }
    }

    const BPP: u32 = 4;
    const CURSOR_WIDTH: u32 = 64;
    const CURSOR_HEIGHT: u32 = 64;
    const CURSOR_BPP: u32 = 4;

    const MAX_BUFFERS: i32 = 64;

    const PI_POW_2: f64 = std::f64::consts::PI * std::f64::consts::PI;
    const ACCUM_STEP: f64 = PI_POW_2 / 50f64;

    pub fn main() {
        let core = Rc::new(Core::default());
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

        let stream = Rc::new(
            Stream::new(
                &core,
                CString::new("Test video source").unwrap().as_ref(),
                properties_new!(pw_sys::PW_KEY_MEDIA_CLASS => "Video/Source\0"),
            )
            .unwrap(),
        );
        let timeout_timer = main_loop
            .get_loop()
            .add_timer(Box::new({
                let stream = stream.clone();
                move |_| {
                    println!("Triggering stream process by timer");
                    stream.trigger_process().unwrap();
                }
            }))
            .unwrap();
        let state = Rc::new(Mutex::new(State {
            loop_: main_loop.clone(),
            size: None,
            timeout_timer,
            seq: 0,
            counter: 0,
            crop: 0.0,
            accumulator: 0.0,
        }));

        let listener = StreamEventsBuilder::default()
            .process(Box::new({
                let state = state.clone();
                let stream = stream.clone();
                move || on_process(&state, &stream)
            }))
            .param_changed(Box::new({
                let state = state.clone();
                let stream = stream.clone();
                move |id, pod| {
                    if id == ParamType::FORMAT.into() {
                        on_format_changed(&state, &stream, pod).unwrap();
                    }
                }
            }))
            .state_changed(Box::new({
                let state = state.clone();
                move |_from, to, _err| on_state_changed(to, &state)
            }))
            .build();
        stream.add_listener(listener);
        stream
            .connect(
                Direction::OUTPUT,
                StreamFlags::DRIVER | StreamFlags::MAP_BUFFERS,
                &[format_param().unwrap().as_pod().upcast()],
            )
            .unwrap();

        println!("Running main loop");
        main_loop.run().unwrap();
    }

    fn draw_ellipse(bitmap: &mut [u32], width: u32, height: u32, color: u32) {
        let r1 = (width / 2) as i32;
        let r1_pow_2 = r1 * r1;
        let r2 = (height / 2) as i32;
        let r2_pow_2 = r2 * r2;
        let r12_pow_2 = r1_pow_2 * r2_pow_2;

        for i in -r2..r2 {
            for j in -r1..r1 {
                bitmap[((i + r2) * width as i32 + (j + r1)) as usize] =
                    if i * i * r1_pow_2 + j * j * r2_pow_2 <= r12_pow_2 {
                        color
                    } else {
                        0
                    };
            }
        }
    }

    fn on_process(state: &Rc<Mutex<State>>, stream: &Rc<Stream>) {
        if let Some(buf) = stream.dequeue_buffer() {
            let spa_buf = buf.buffer_mut();
            let data_ptr = spa_buf
                .datas_mut()
                .first()
                .map(|first| first.data())
                .and_then(|first_data| unsafe {
                    first_data.as_mut().map(|_| first_data as *mut u8)
                });
            if let Some(data_ptr) = data_ptr {
                let mut state = state.lock().unwrap();
                if let Some(size) = state.size {
                    for meta in spa_buf.metas_mut() {
                        match meta.data() {
                            MetaData::HEADER([header]) => {
                                header.set_pts(-1);
                                header.set_flags(HeaderFlags::empty());
                                state.seq += 1;
                                header.set_seq(state.seq);
                                header.set_dts_offset(0);
                            }
                            MetaData::VIDEO_DAMAGE([region, others @ ..]) => {
                                region
                                    .region_mut()
                                    .set_value(0, 0, size.width(), size.height());
                                if let Some(region) = others.first_mut() {
                                    region.region_mut().set_value(0, 0, 0, 0);
                                }
                            }
                            MetaData::VIDEO_CROP([region]) => {
                                let crop = (state.accumulator.sin() + 1f64) * 32f64;
                                state.crop = crop;
                                region.region_mut().set_value(
                                    crop as i32,
                                    crop as i32,
                                    size.width() - crop as u32 * 2,
                                    size.height() - crop as u32 * 2,
                                )
                            }
                            MetaData::CURSOR([cursor]) => unsafe {
                                cursor.set_id(1);
                                cursor.position_mut().set_value(
                                    ((state.accumulator.sin() + 1f64) * 160f64 + 80f64) as i32,
                                    ((state.accumulator.cos() + 1f64) * 160f64 + 50f64) as i32,
                                );
                                cursor.hotspot_mut().set_value(0, 0);
                                cursor.set_bitmap_offset(size_of::<MetaCursorRef>() as u32);

                                let bitmap = cursor.bitmap().unwrap();
                                bitmap.set_format(VideoFormat::RGB);
                                bitmap.size_mut().set_value(CURSOR_WIDTH, CURSOR_HEIGHT);
                                bitmap.set_stride((CURSOR_WIDTH * CURSOR_BPP) as i32);
                                bitmap.set_offset(size_of::<MetaBitmapRef>() as u32);

                                let color = ((state.accumulator.cos() + 1f64) as u32 * (1 << 23))
                                    | 0xff000000;
                                draw_ellipse(
                                    bitmap.bitmap().unwrap(),
                                    bitmap.size().width(),
                                    bitmap.size().height(),
                                    color,
                                );
                            },
                            _ => {}
                        }
                    }
                    unsafe {
                        let stride = state.stride().unwrap();
                        for i in 0..size.height() {
                            for j in 0..size.width() * BPP {
                                *data_ptr.offset((i * stride + j) as isize) =
                                    (state.counter + j * i) as u8;
                            }
                            state.counter.add_assign(13);
                        }
                        state.accumulator = (state.accumulator + ACCUM_STEP) % PI_POW_2;

                        if let [data] = spa_buf.datas_mut() {
                            let chunk = data.chunk_mut();
                            chunk.set_offset(0);
                            chunk.set_size(stride * size.height());
                            chunk.set_stride(stride as i32);
                        }
                    }
                }
            }
            stream.queue_buffer(buf).unwrap();
        }
    }

    fn on_state_changed(to: stream::State, state: &Rc<Mutex<State>>) {
        let state = state.lock().unwrap();
        println!("Video source status updated to {:?}", to);
        match to {
            stream::State::ERROR | stream::State::UNCONNECTED => state.loop_.quit().unwrap(),
            stream::State::PAUSED => state
                .loop_
                .get_loop()
                .disable_timer(&state.timeout_timer)
                .unwrap(),
            stream::State::STREAMING => state
                .loop_
                .get_loop()
                .update_timer(
                    &state.timeout_timer,
                    Duration::from_nanos(1),
                    Duration::from_millis(40),
                    false,
                )
                .unwrap(),
            _ => {}
        }
    }

    fn on_format_changed(
        state: &Rc<Mutex<State>>,
        stream: &Rc<Stream>,
        pod: &PodRef,
    ) -> pipewire_wrapper::Result<()> {
        if let BasicType::OBJECT(obj) = pod.downcast().unwrap() {
            if let ObjectType::OBJECT_FORMAT(format) = obj.param_value(ParamType::FORMAT).unwrap() {
                println!("Got format param {:?}", pod.downcast());
                for prop in format {
                    if let ObjectFormatType::VIDEO_SIZE(size) = prop.value()? {
                        let mut state = state.lock().unwrap();
                        let size = size.value()?;
                        state.size = Some(size);
                        let allocated_stream_params = stream_params(size, state.stride().unwrap())?;
                        let stream_objects_params: Vec<&PodObjectRef> = allocated_stream_params
                            .iter()
                            .map(|allocated| allocated.as_pod())
                            .collect();
                        let stream_params: Vec<&PodRef> = stream_objects_params
                            .iter()
                            .map(|obj| obj.upcast())
                            .collect();
                        stream.update_params(stream_params.as_slice()).unwrap();
                    }
                }
            }
        }
        Ok(())
    }

    fn format_param() -> pipewire_wrapper::Result<AllocPod<PodObjectRef>> {
        let format = ObjectEnumFormatBuilder::default()
            .body_id(ParamType::ENUM_FORMAT.into())
            .media_type(MediaType::VIDEO)
            .media_subtype(MediaSubType::RAW)
            .video_format(ChoiceStructType::ENUM(PodEnumValue::from_default(
                VideoFormat::RGB,
            )))
            .video_size(ChoiceStructType::RANGE(PodRangeValue::new(
                RectangleRef::new(320, 240),
                RectangleRef::new(1, 1),
                RectangleRef::new(4096, 4096),
            )))
            .video_framerate(ChoiceStructType::NONE(FractionRef::new(25, 1)))
            .build()?;
        Ok(format)
    }

    fn stream_params(
        size: RectangleRef,
        stride: u32,
    ) -> pipewire_wrapper::Result<Vec<AllocPod<PodObjectRef>>> {
        let buffers = ParamBuffersBuilder::default()
            .body_id(ParamType::BUFFERS.into())
            .buffers(ChoiceStructType::RANGE(PodRangeValue::new(
                8,
                2,
                MAX_BUFFERS,
            )))
            .blocks(ChoiceStructType::NONE(1))
            .size(ChoiceStructType::NONE((stride * size.height()) as i32))
            .stride(ChoiceStructType::NONE(stride as i32))
            .build()?;
        println!("Video output buffer: {:?}", buffers.as_pod());
        let meta_header = ParamMetaBuilder::default()
            .body_id(ParamType::META.into())
            .type_(MetaType::HEADER)
            .size(ChoiceStructType::NONE(
                size_of::<spa_sys::spa_meta_header>() as i32,
            ))
            .build()?;
        let meta_video_damage = ParamMetaBuilder::default()
            .body_id(ParamType::META.into())
            .type_(MetaType::VIDEO_DAMAGE)
            .size(ChoiceStructType::RANGE(PodRangeValue::new(
                size_of::<spa_sys::spa_meta_region>() as i32 * 16,
                size_of::<spa_sys::spa_meta_region>() as i32,
                size_of::<spa_sys::spa_meta_region>() as i32 * 16,
            )))
            .build()?;
        let meta_video_crop = ParamMetaBuilder::default()
            .body_id(ParamType::META.into())
            .type_(MetaType::VIDEO_CROP)
            .size(ChoiceStructType::NONE(
                size_of::<spa_sys::spa_meta_region>() as i32,
            ))
            .build()?;
        let meta_cursor = ParamMetaBuilder::default()
            .body_id(ParamType::META.into())
            .type_(MetaType::CURSOR)
            .size(ChoiceStructType::NONE(
                (size_of::<spa_sys::spa_meta_cursor>() as u32
                    + size_of::<spa_sys::spa_meta_bitmap>() as u32
                    + size.width() * size.height() * BPP) as i32,
            ))
            .build()?;
        Ok(vec![
            buffers,
            meta_header,
            meta_video_damage,
            meta_video_crop,
            meta_cursor,
        ])
    }
}

fn main() {
    #[cfg(feature = "spa-pod-object-builders")]
    video_src::main()
}
