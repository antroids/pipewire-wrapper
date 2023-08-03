use std::ffi::{CStr, CString};
use std::mem::size_of;
use std::rc::Rc;
use std::slice;
use std::sync::Mutex;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::core_api::main_loop::MainLoop;
use pipewire_wrapper::listeners::OwnListeners;
use pipewire_wrapper::spa::buffers::meta::{
    MetaBitmapRef, MetaCursorRef, MetaData, MetaHeaderRef, MetaRegionRef,
};
use pipewire_wrapper::spa::io::{IOValue, IOVideoSizeRef};
use pipewire_wrapper::spa::param::ParamType;
use pipewire_wrapper::spa::pod::choice::enum_::PodEnumValue;
use pipewire_wrapper::spa::pod::choice::range::PodRangeValue;
use pipewire_wrapper::spa::pod::choice::ChoiceStructType;
use pipewire_wrapper::spa::pod::object::enum_format::ObjectEnumFormatBuilder;
use pipewire_wrapper::spa::pod::object::format::{
    MediaSubType, MediaType, ObjectFormatInfo, VideoFormat,
};
use pipewire_wrapper::spa::pod::object::param_buffers::ParamBuffersBuilder;
use pipewire_wrapper::spa::pod::object::param_meta::{MetaType, ParamMetaBuilder};
use pipewire_wrapper::spa::pod::object::param_port_config::Direction;
use pipewire_wrapper::spa::pod::object::PodObjectRef;
use pipewire_wrapper::spa::pod::pod_buf::AllocPod;
use pipewire_wrapper::spa::pod::PodValue;
use pipewire_wrapper::spa::pod::{BasicType, Upcast};
use pipewire_wrapper::spa::type_::{FractionRef, RectangleRef};
use pipewire_wrapper::stream::buffer::BufferRef;
use pipewire_wrapper::stream::events::StreamEventsBuilder;
use pipewire_wrapper::stream::{Stream, StreamFlags};
use pipewire_wrapper::{properties_new, stream};

const EINVAL: i32 = 22; // Invalid argument

struct State<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    event_pump: EventPump,
    main_loop: &'a MainLoop,
    stream: Rc<Stream<'a>>,

    video_format: VideoFormat,
    size: RectangleRef,
    media_subtype: MediaSubType,
    stride: usize,

    io_video_size: Option<IOVideoSizeRef>,

    texture: Option<Texture<'a>>,
    cursor: Option<Texture<'a>>,
    rect: Rect,
    cursor_rect: Rect,
}

impl State<'_> {
    fn pixel_format(&self) -> PixelFormatEnum {
        if self.media_subtype == MediaSubType::DSP {
            PixelFormatEnum::RGBA8888
        } else {
            video_to_pixel_format(self.video_format)
        }
    }

    fn is_yuv(&self) -> bool {
        matches!(
            self.video_format,
            VideoFormat::YV12
                | VideoFormat::YVYU
                | VideoFormat::UYVY
                | VideoFormat::YUY2
                | VideoFormat::I420
        )
    }
}

impl<'a> State<'a> {
    fn new(
        texture_creator: &'a TextureCreator<WindowContext>,
        event_pump: EventPump,
        main_loop: &'a MainLoop,
        stream: Rc<Stream<'a>>,
    ) -> Self {
        Self {
            texture_creator,
            event_pump,
            main_loop,
            stream,

            video_format: VideoFormat::UNKNOWN,
            size: RectangleRef::new(0, 0),
            media_subtype: MediaSubType::UNKNOWN,
            stride: 0,
            io_video_size: None,
            texture: None,
            cursor: None,
            rect: Rect::new(0, 0, 0, 0),
            cursor_rect: Rect::new(0, 0, 0, 0),
        }
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Window", 800, 600).build().unwrap();

    let core = Rc::new(Core::default());
    let main_loop = core.context().main_loop();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let event_pump = sdl_context.event_pump()?;

    let stream = Rc::new(
        Stream::new(
            &core,
            CString::new("SDL Video Player").unwrap().as_ref(),
            properties_new!(
                pw_sys::PW_KEY_MEDIA_TYPE => "Video\0",
                pw_sys::PW_KEY_MEDIA_CATEGORY => "Capture\0",
                pw_sys::PW_KEY_MEDIA_ROLE => "Camera\0"),
        )
        .unwrap(),
    );

    let state: Rc<Mutex<State>> = Rc::new(Mutex::new(State::new(
        &texture_creator,
        event_pump,
        main_loop,
        stream.clone(),
    )));

    let main_loop_timer = main_loop
        .get_loop()
        .add_timer(Box::new(|_| handle_sdl_events(&state).unwrap()))
        .unwrap();
    main_loop
        .get_loop()
        .update_timer(
            &main_loop_timer,
            Duration::from_millis(100),
            Duration::from_millis(100),
            false,
        )
        .unwrap();

    let stream_listener = StreamEventsBuilder::default()
        .state_changed(Box::new({
            let state = state.clone();
            move |_from, to, _err| {
                on_stream_state_changed(&state, to).expect("Failed to process state update");
            }
        }))
        .param_changed(Box::new({
            let state = state.clone();
            move |id, param| {
                if id == ParamType::FORMAT.into() {
                    println!("Got format param {:?}", param.downcast());
                    if let Ok(BasicType::OBJECT(obj)) = param.downcast() {
                        let format_info: ObjectFormatInfo =
                            obj.try_into().expect("Unable to parse object");
                        on_stream_param_changed(&state, format_info)
                            .expect("Failed to process param change");
                    }
                };
            }
        }))
        .process(Box::new({
            let state = state.clone();
            move || {
                on_stream_process(&state, &mut canvas).expect("Unable to process stream");
            }
        }))
        .io_changed(Box::new({
            let state = state.clone();
            move |io_value| {
                if let IOValue::POSITION(pos) = io_value {
                    state.lock().unwrap().io_video_size = Some(pos.video().clone());
                }
            }
        }))
        .build();
    stream.add_listener(stream_listener);

    stream
        .connect(
            Direction::INPUT,
            StreamFlags::INACTIVE | StreamFlags::MAP_BUFFERS,
            &[
                sdl_enum_format().unwrap().as_pod().upcast(),
                dsp_enum_format().unwrap().as_pod().upcast(),
            ],
        )
        .unwrap();

    main_loop.run().unwrap();
    Ok(())
}

fn handle_sdl_events(state: &Mutex<State>) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = state.lock().unwrap();
    let main_loop = state.main_loop;
    let stream = state.stream.clone();
    for event in state.event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                main_loop.quit().unwrap();
            }
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                let stream_state = stream.get_state();
                println!("Stream state: {:?}", stream_state);
                if stream_state == stream::State::PAUSED {
                    stream.set_active(true)?;
                    stream.trigger_process()?;
                } else {
                    stream.set_active(false)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn dsp_enum_format() -> pipewire_wrapper::Result<AllocPod<PodObjectRef>> {
    Ok(ObjectEnumFormatBuilder::default()
        .body_id(ParamType::ENUM_FORMAT.into())
        .media_type(MediaType::VIDEO)
        .media_subtype(MediaSubType::DSP)
        .video_format(ChoiceStructType::NONE(VideoFormat::DSP_F32))
        .build()?)
}

fn sdl_enum_format() -> pipewire_wrapper::Result<AllocPod<PodObjectRef>> {
    Ok(ObjectEnumFormatBuilder::default()
        .body_id(ParamType::ENUM_FORMAT.into())
        .media_type(MediaType::VIDEO)
        .media_subtype(MediaSubType::RAW)
        .video_format(ChoiceStructType::ENUM(PodEnumValue::new(
            VideoFormat::RGB,
            supported_video_formats().to_owned(),
        )))
        .video_size(ChoiceStructType::RANGE(PodRangeValue::new(
            RectangleRef::new(640, 480),
            RectangleRef::new(1, 1),
            RectangleRef::new(4096, 4096),
        )))
        .video_framerate(ChoiceStructType::RANGE(PodRangeValue::new(
            FractionRef::new(25, 1),
            FractionRef::new(1, 1),
            FractionRef::new(60, 1),
        )))
        .build()?)
}

fn on_stream_param_changed(
    state: &Mutex<State<'_>>,
    format_info: ObjectFormatInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    if let (Some(media_type), Some(media_subtype), Some(video_format)) = (
        format_info.media_type,
        format_info.media_subtype,
        format_info.video_format,
    ) {
        if media_type.value()? == MediaType::VIDEO {
            let mut state = state.lock().unwrap();
            state.media_subtype = media_subtype.value()?;
            state.video_format = video_format.value()?;
            state.size = match state.media_subtype {
                MediaSubType::RAW => format_info
                    .video_size
                    .map(|s| s.value())
                    .transpose()?
                    .unwrap_or(state.size),
                MediaSubType::DSP => state
                    .io_video_size
                    .as_ref()
                    .map(|v| *v.size())
                    .unwrap_or(state.size),
                _ => return Err("Unsupported media subtype".into()),
            };

            if state.pixel_format() == PixelFormatEnum::Unknown {
                state.stream.set_error(
                    -EINVAL,
                    CStr::from_bytes_with_nul(b"Unsupported pixel format\0").unwrap(),
                )?;
            }
            if state.size.width() == 0 || state.size.height() == 0 {
                state.stream.set_error(
                    -EINVAL,
                    CStr::from_bytes_with_nul(b"Unknown size\0").unwrap(),
                )?;
            }

            let mut texture = state
                .texture_creator
                .create_texture_streaming(
                    state.pixel_format(),
                    state.size.width(),
                    state.size.height(),
                )
                .unwrap();
            texture
                .with_lock(None, |_buf, stride| {
                    state.stride = stride;
                })
                .unwrap();
            state.texture = Some(texture);

            let buffer_stride = if state.media_subtype == MediaSubType::DSP {
                state.stride * 4
            } else {
                state.stride
            };
            let buffer_size = if state.is_yuv() {
                (buffer_stride as u32 * state.size.height()) * 3 / 2
            } else {
                buffer_stride as u32 * state.size.height()
            };
            state.rect = Rect::new(0, 0, state.size.width(), state.size.height());

            let buffers_param = ParamBuffersBuilder::default()
                .body_id(ParamType::BUFFERS.into())
                .buffers(ChoiceStructType::RANGE(PodRangeValue::new(8, 2, 64)))
                .blocks(ChoiceStructType::NONE(1))
                .size(ChoiceStructType::NONE(buffer_size as i32))
                .stride(ChoiceStructType::NONE(buffer_stride as i32))
                /* .datatype(ChoiceStructType::NONE(
                    (1u32 << Into::<u32>::into(DataType::MEM_PTR)) as i32,
                )) */
                .build()?;
            println!("Video input buffer: {:?}", buffers_param.as_pod());
            let meta_header_param = ParamMetaBuilder::default()
                .body_id(ParamType::META.into())
                .type_(MetaType::HEADER)
                .size(ChoiceStructType::NONE(size_of::<MetaHeaderRef>() as i32))
                .build()?;
            let meta_video_crop_param = ParamMetaBuilder::default()
                .body_id(ParamType::META.into())
                .type_(MetaType::VIDEO_CROP)
                .size(ChoiceStructType::NONE(size_of::<MetaRegionRef>() as i32))
                .build()?;
            let meta_cursor_bitmap_size =
                (size_of::<MetaCursorRef>() + size_of::<MetaBitmapRef>()) as i32;
            let meta_cursor_param = ParamMetaBuilder::default()
                .body_id(ParamType::META.into())
                .type_(MetaType::CURSOR)
                .size(ChoiceStructType::RANGE(PodRangeValue::new(
                    meta_cursor_bitmap_size + 64 * 64 * 4,
                    meta_cursor_bitmap_size + 4,
                    meta_cursor_bitmap_size + 256 * 256 * 4,
                )))
                .build()?;

            state.stream.update_params(&[
                buffers_param.as_pod().upcast(),
                meta_header_param.as_pod().upcast(),
                meta_video_crop_param.as_pod().upcast(),
                meta_cursor_param.as_pod().upcast(),
            ])?;
        }
    }
    Ok(())
}

fn on_stream_state_changed(
    state: &Mutex<State>,
    to: stream::State,
) -> pipewire_wrapper::Result<()> {
    let state = state.lock().unwrap();
    println!("Stream state updated to {:?}", to);
    match to {
        stream::State::UNCONNECTED => state.main_loop.quit()?,
        stream::State::PAUSED => state.stream.set_active(true)?,
        _ => {}
    }
    Ok(())
}

fn on_stream_process(
    state: &Mutex<State>,
    canvas: &mut Canvas<Window>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_buf: Option<&mut BufferRef> = None;
    let stream = state.lock().unwrap().stream.clone();
    while let Some(buf) = stream.dequeue_buffer() {
        last_buf = Some(buf);
    }
    handle_sdl_events(state)?;
    if let Some(buf) = last_buf {
        let spa_buf = buf.buffer_mut();
        let data_ptr = spa_buf
            .datas_mut()
            .first()
            .map(|first| first.data())
            .and_then(|first_data| unsafe { first_data.as_mut().map(|_| first_data as *mut u8) });
        if let Some(src_data_ptr) = data_ptr {
            let mut state = state.lock().unwrap();
            let mut render_cursor = false;
            for meta in spa_buf.metas() {
                match meta.data() {
                    MetaData::VIDEO_CROP([video_crop, ..]) => {
                        if video_crop.is_valid() {
                            let reg = video_crop.region();
                            state.rect = Rect::new(
                                reg.position().x(),
                                reg.position().y(),
                                reg.size().width(),
                                reg.size().height(),
                            );
                        }
                    }
                    MetaData::CURSOR([cursor, ..]) => {
                        if cursor.is_valid() {
                            if let Some(cursor_bitmap) = unsafe { cursor.bitmap() } {
                                state.cursor_rect = Rect::new(
                                    cursor.position().x(),
                                    cursor.position().y(),
                                    cursor_bitmap.size().width(),
                                    cursor_bitmap.size().height(),
                                );
                                if state.cursor.is_none() {
                                    let mut cursor =
                                        state.texture_creator.create_texture_streaming(
                                            state.pixel_format(),
                                            cursor_bitmap.size().width(),
                                            cursor_bitmap.size().height(),
                                        )?;
                                    cursor.set_blend_mode(sdl2::render::BlendMode::Blend);
                                    state.cursor = Some(cursor);
                                }

                                state.cursor.as_mut().unwrap().with_lock(
                                    None,
                                    |out_buf, out_stride| {
                                        let bitmap_stride = cursor_bitmap.stride() as usize;
                                        let copy_stride = out_stride.min(bitmap_stride);
                                        let bitmap =
                                            unsafe { cursor_bitmap.bitmap::<u8>().unwrap() };
                                        for y in 0..cursor_bitmap.size().height() {
                                            let out_buf_pos = out_stride * y as usize;
                                            let in_buf_pos = bitmap_stride * y as usize;
                                            let out_slice = &mut out_buf
                                                [out_buf_pos..(out_buf_pos + copy_stride)];
                                            let in_slice =
                                                &mut bitmap[in_buf_pos..(in_buf_pos + copy_stride)];
                                            out_slice.copy_from_slice(in_slice);
                                        }
                                        render_cursor = true;
                                    },
                                )?;
                            }
                        }
                    }
                    _ => {}
                };
            }

            // copy video image in texture
            let height = state.size.height() as usize;
            let width = state.size.width() as usize;
            let stride = state.stride;
            let media_subtype = state.media_subtype;
            let is_yuv = state.is_yuv();
            let rect = state.rect;
            if let Some(texture) = state.texture.as_mut() {
                if is_yuv {
                    let src = unsafe { slice::from_raw_parts(src_data_ptr, stride * height) };
                    texture.update_yuv(
                        None,
                        src,
                        stride,
                        &src[stride * height..],
                        stride / 2,
                        &src[5 * stride * height / 4..],
                        stride / 2,
                    )?;
                } else {
                    texture.with_lock(None, |dst_buf, dst_stride| {
                        let first_chunk = spa_buf.datas().first().unwrap().chunk();
                        let src_stride = first_chunk.stride();
                        let src_stride = if src_stride == 0 {
                            first_chunk.size() as usize / height
                        } else {
                            src_stride as usize
                        };
                        let src_slice =
                            unsafe { slice::from_raw_parts(src_data_ptr, src_stride * height) };
                        if media_subtype == MediaSubType::DSP {
                            for y in 0..height {
                                let src_pos = src_stride * y;
                                let src_line = &src_slice[src_pos..src_pos + src_stride];
                                let dst_pos = dst_stride * y;
                                let dst_line = &mut dst_buf[dst_pos..dst_pos + dst_stride];
                                for x in 0..width {
                                    let dst_pixel = &mut dst_line[4 * x..4 * (x + 1)];
                                    let src_pixel = &src_line[4 * x..4 * (x + 1)];
                                    dst_pixel.copy_from_slice(src_pixel);
                                }
                            }
                        } else {
                            let copy_stride = dst_stride.min(src_stride);
                            for y in 0..height {
                                let src_pos = src_stride * y;
                                let src_line = &src_slice[src_pos..src_pos + copy_stride];
                                let dst_pos = dst_stride * y;
                                let dst_line = &mut dst_buf[dst_pos..dst_pos + copy_stride];
                                dst_line.copy_from_slice(src_line);
                            }
                        }
                    })?;
                }
                canvas.clear();
                canvas.copy(texture, rect, None)?;
            }
            if render_cursor {
                let cursor_rect = state.cursor_rect;
                canvas.copy(state.cursor.as_ref().unwrap(), cursor_rect, None)?;
            }
            canvas.present();
        }

        stream.queue_buffer(buf)?;
    } else {
        return Err("Out of buffers".into());
    }

    Ok(())
}

fn video_to_pixel_format(v: VideoFormat) -> PixelFormatEnum {
    match v {
        VideoFormat::RGB => PixelFormatEnum::RGB888,
        VideoFormat::RGB15 => PixelFormatEnum::RGB555,
        VideoFormat::RGB16 => PixelFormatEnum::RGB565,

        VideoFormat::BGR => PixelFormatEnum::BGR888,
        VideoFormat::BGR15 => PixelFormatEnum::BGR555,
        VideoFormat::BGR16 => PixelFormatEnum::BGR565,

        VideoFormat::RGBA => PixelFormatEnum::RGBA8888,
        VideoFormat::BGRA => PixelFormatEnum::BGRA8888,
        VideoFormat::ARGB => PixelFormatEnum::ARGB8888,
        VideoFormat::ABGR => PixelFormatEnum::ABGR8888,

        VideoFormat::RGBX => PixelFormatEnum::RGBX8888,
        VideoFormat::BGRX => PixelFormatEnum::BGRX8888,

        VideoFormat::YV12 => PixelFormatEnum::YV12,
        VideoFormat::YVYU => PixelFormatEnum::YVYU,
        VideoFormat::UYVY => PixelFormatEnum::UYVY,
        VideoFormat::YUY2 => PixelFormatEnum::YUY2,
        VideoFormat::I420 => PixelFormatEnum::IYUV,

        _ => PixelFormatEnum::Unknown,
    }
}

fn supported_video_formats() -> &'static [VideoFormat] {
    &[
        VideoFormat::RGB,
        VideoFormat::RGB15,
        VideoFormat::RGB16,
        VideoFormat::BGR,
        VideoFormat::BGR15,
        VideoFormat::BGR16,
        VideoFormat::RGBA,
        VideoFormat::BGRA,
        VideoFormat::ARGB,
        VideoFormat::ABGR,
        VideoFormat::RGBX,
        VideoFormat::BGRX,
        VideoFormat::YV12,
        VideoFormat::YVYU,
        VideoFormat::UYVY,
        VideoFormat::YUY2,
        VideoFormat::I420,
    ]
}
