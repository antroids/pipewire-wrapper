use pipewire_macro_impl::enum_wrapper;

use crate::spa::type_::pod::array::PodArrayRef;
use crate::spa::type_::pod::choice::enum_::PodEnumRef;
use crate::spa::type_::pod::choice::PodChoiceRef;
use crate::spa::type_::pod::id::{PodIdRef, PodIdType};
use crate::spa::type_::pod::object::prop::AudioIec958Codec;
use crate::spa::type_::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::type_::pod::{
    BasicTypePod, PodError, PodFractionRef, PodIntRef, PodLongRef, PodRectangleRef,
};
use crate::wrapper::RawWrapper;

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(u32)]
pub enum ObjectFormatType<'a> {
    // Media
    MEDIA_TYPE(&'a PodIdRef<MediaType>) = Format::MEDIA_TYPE.raw,
    MEDIA_SUBTYPE(&'a PodIdRef<MediaSubType>) = Format::MEDIA_SUBTYPE.raw,

    // Audio
    AUDIO_FORMAT(&'a PodIdRef<AudioFormat>) = Format::AUDIO_FORMAT.raw,
    AUDIO_FLAGS(&'a PodIntRef) = Format::AUDIO_FLAGS.raw,
    AUDIO_RATE(&'a PodIntRef) = Format::AUDIO_RATE.raw, // Getting choice
    AUDIO_CHANNELS(&'a PodIntRef) = Format::AUDIO_CHANNELS.raw,
    AUDIO_POSITION(&'a PodArrayRef<PodIdRef<u32>>) = Format::AUDIO_POSITION.raw, // Enum in comments, but getting array
    AUDIO_IEC958CODEC(&'a PodEnumRef<PodIdRef<AudioIec958Codec>>) = Format::AUDIO_IEC958CODEC.raw,
    AUDIO_BITORDER(&'a PodEnumRef<PodIdRef<ParamBitorder>>) = Format::AUDIO_BITORDER.raw,
    AUDIO_INTERLEAVE(&'a PodIntRef) = Format::AUDIO_INTERLEAVE.raw,
    // missing audio params

    // Video
    VIDEO_FORMAT(&'a PodIdRef<VideoFormat>) = Format::VIDEO_FORMAT.raw,
    VIDEO_MODIFIER(&'a PodLongRef) = Format::VIDEO_MODIFIER.raw,
    VIDEO_SIZE(&'a PodRectangleRef) = Format::VIDEO_SIZE.raw,
    VIDEO_FRAMERATE(&'a PodChoiceRef) = Format::VIDEO_FRAMERATE.raw, // There is a choice for some reason, but should be fraction
    VIDEO_MAX_FRAMERATE(&'a PodFractionRef) = Format::VIDEO_MAX_FRAMERATE.raw,
    VIDEO_VIEWS(&'a PodIntRef) = Format::VIDEO_VIEWS.raw,
    VIDEO_INTERLACE_MODE(&'a PodEnumRef<PodIdRef<VideoInterlaceMode>>) =
        Format::VIDEO_INTERLACE_MODE.raw,
    VIDEO_PIXEL_ASPECT_RATIO(&'a PodRectangleRef) = Format::VIDEO_PIXEL_ASPECT_RATIO.raw,
    VIDEO_MULTIVIEW_MODE(&'a PodEnumRef<PodIdRef<VideoMultiviewMode>>) =
        Format::VIDEO_MULTIVIEW_MODE.raw,
    VIDEO_MULTIVIEW_FLAGS(&'a PodEnumRef<PodIdRef<VideoMultiviewFlags>>) =
        Format::VIDEO_MULTIVIEW_FLAGS.raw,
    VIDEO_CHROMA_SITE(&'a PodEnumRef<PodIdRef<VideoChromaSite>>) = Format::VIDEO_CHROMA_SITE.raw,
    VIDEO_COLOR_RANGE(&'a PodEnumRef<PodIdRef<VideoColorRange>>) = Format::VIDEO_COLOR_RANGE.raw,
    VIDEO_COLOR_MATRIX(&'a PodEnumRef<PodIdRef<VideoColorMatrix>>) = Format::VIDEO_COLOR_MATRIX.raw,
    VIDEO_TRANSFER_FUNCTION(&'a PodEnumRef<PodIdRef<VideoTransferFunction>>) =
        Format::VIDEO_TRANSFER_FUNCTION.raw,
    VIDEO_COLOR_PRIMARIES(&'a PodEnumRef<PodIdRef<VideoColorPrimaries>>) =
        Format::VIDEO_COLOR_PRIMARIES.raw,
    VIDEO_PROFILE(&'a PodIntRef) = Format::VIDEO_PROFILE.raw,
    VIDEO_LEVEL(&'a PodIntRef) = Format::VIDEO_LEVEL.raw,
    VIDEO_H264_STREAM_FORMAT(&'a PodEnumRef<PodIdRef<VideoH264StreamFormat>>) =
        Format::VIDEO_H264_STREAM_FORMAT.raw,
    VIDEO_H264_ALIGNMENT(&'a PodEnumRef<PodIdRef<VideoH264Alignment>>) =
        Format::VIDEO_H264_ALIGNMENT.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ObjectFormatType<'a>>> for ObjectFormatType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ObjectFormatType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match Format::from_raw(value.raw.key) {
                Format::MEDIA_TYPE => Ok(ObjectFormatType::MEDIA_TYPE(value.pod().cast()?)),
                Format::MEDIA_SUBTYPE => Ok(ObjectFormatType::MEDIA_SUBTYPE(value.pod().cast()?)),
                Format::AUDIO_FORMAT => Ok(ObjectFormatType::AUDIO_FORMAT(value.pod().cast()?)),
                Format::AUDIO_FLAGS => Ok(ObjectFormatType::AUDIO_FLAGS(value.pod().cast()?)),
                Format::AUDIO_RATE => Ok(ObjectFormatType::AUDIO_RATE(value.pod().cast()?)),
                Format::AUDIO_CHANNELS => Ok(ObjectFormatType::AUDIO_CHANNELS(value.pod().cast()?)),
                Format::AUDIO_POSITION => Ok(ObjectFormatType::AUDIO_POSITION(value.pod().cast()?)),
                Format::AUDIO_IEC958CODEC => {
                    Ok(ObjectFormatType::AUDIO_IEC958CODEC(value.pod().cast()?))
                }
                Format::AUDIO_BITORDER => Ok(ObjectFormatType::AUDIO_BITORDER(value.pod().cast()?)),
                Format::AUDIO_INTERLEAVE => {
                    Ok(ObjectFormatType::AUDIO_INTERLEAVE(value.pod().cast()?))
                }
                Format::VIDEO_FORMAT => Ok(ObjectFormatType::VIDEO_FORMAT(value.pod().cast()?)),
                Format::VIDEO_MODIFIER => Ok(ObjectFormatType::VIDEO_MODIFIER(value.pod().cast()?)),
                Format::VIDEO_SIZE => Ok(ObjectFormatType::VIDEO_SIZE(value.pod().cast()?)),
                Format::VIDEO_FRAMERATE => {
                    Ok(ObjectFormatType::VIDEO_FRAMERATE(value.pod().cast()?))
                }
                Format::VIDEO_MAX_FRAMERATE => {
                    Ok(ObjectFormatType::VIDEO_MAX_FRAMERATE(value.pod().cast()?))
                }
                Format::VIDEO_VIEWS => Ok(ObjectFormatType::VIDEO_VIEWS(value.pod().cast()?)),
                Format::VIDEO_INTERLACE_MODE => {
                    Ok(ObjectFormatType::VIDEO_INTERLACE_MODE(value.pod().cast()?))
                }
                Format::VIDEO_PIXEL_ASPECT_RATIO => Ok(ObjectFormatType::VIDEO_PIXEL_ASPECT_RATIO(
                    value.pod().cast()?,
                )),
                Format::VIDEO_MULTIVIEW_MODE => {
                    Ok(ObjectFormatType::VIDEO_MULTIVIEW_MODE(value.pod().cast()?))
                }
                Format::VIDEO_MULTIVIEW_FLAGS => {
                    Ok(ObjectFormatType::VIDEO_MULTIVIEW_FLAGS(value.pod().cast()?))
                }
                Format::VIDEO_CHROMA_SITE => {
                    Ok(ObjectFormatType::VIDEO_CHROMA_SITE(value.pod().cast()?))
                }
                Format::VIDEO_COLOR_RANGE => {
                    Ok(ObjectFormatType::VIDEO_COLOR_RANGE(value.pod().cast()?))
                }
                Format::VIDEO_COLOR_MATRIX => {
                    Ok(ObjectFormatType::VIDEO_COLOR_MATRIX(value.pod().cast()?))
                }
                Format::VIDEO_TRANSFER_FUNCTION => Ok(ObjectFormatType::VIDEO_TRANSFER_FUNCTION(
                    value.pod().cast()?,
                )),
                Format::VIDEO_COLOR_PRIMARIES => {
                    Ok(ObjectFormatType::VIDEO_COLOR_PRIMARIES(value.pod().cast()?))
                }
                Format::VIDEO_PROFILE => Ok(ObjectFormatType::VIDEO_PROFILE(value.pod().cast()?)),
                Format::VIDEO_LEVEL => Ok(ObjectFormatType::VIDEO_LEVEL(value.pod().cast()?)),
                Format::VIDEO_H264_STREAM_FORMAT => Ok(ObjectFormatType::VIDEO_H264_STREAM_FORMAT(
                    value.pod().cast()?,
                )),
                Format::VIDEO_H264_ALIGNMENT => {
                    Ok(ObjectFormatType::VIDEO_H264_ALIGNMENT(value.pod().cast()?))
                }
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ObjectFormatType<'a> {}

enum_wrapper!(
    Format,
    spa_sys::spa_format,
    _START: spa_sys::SPA_FORMAT_START,
    MEDIA_TYPE: spa_sys::SPA_FORMAT_mediaType,
    MEDIA_SUBTYPE: spa_sys::SPA_FORMAT_mediaSubtype,
    _START_AUDIO: spa_sys::SPA_FORMAT_START_Audio,
    AUDIO_FORMAT: spa_sys::SPA_FORMAT_AUDIO_format,
    AUDIO_FLAGS: spa_sys::SPA_FORMAT_AUDIO_flags,
    AUDIO_RATE: spa_sys::SPA_FORMAT_AUDIO_rate,
    AUDIO_CHANNELS: spa_sys::SPA_FORMAT_AUDIO_channels,
    AUDIO_POSITION: spa_sys::SPA_FORMAT_AUDIO_position,
    AUDIO_IEC958CODEC: spa_sys::SPA_FORMAT_AUDIO_iec958Codec,
    AUDIO_BITORDER: spa_sys::SPA_FORMAT_AUDIO_bitorder,
    AUDIO_INTERLEAVE: spa_sys::SPA_FORMAT_AUDIO_interleave,
    _START_VIDEO: spa_sys::SPA_FORMAT_START_Video,
    VIDEO_FORMAT: spa_sys::SPA_FORMAT_VIDEO_format,
    VIDEO_MODIFIER: spa_sys::SPA_FORMAT_VIDEO_modifier,
    VIDEO_SIZE: spa_sys::SPA_FORMAT_VIDEO_size,
    VIDEO_FRAMERATE: spa_sys::SPA_FORMAT_VIDEO_framerate,
    VIDEO_MAX_FRAMERATE: spa_sys::SPA_FORMAT_VIDEO_maxFramerate,
    VIDEO_VIEWS: spa_sys::SPA_FORMAT_VIDEO_views,
    VIDEO_INTERLACE_MODE: spa_sys::SPA_FORMAT_VIDEO_interlaceMode,
    VIDEO_PIXEL_ASPECT_RATIO: spa_sys::SPA_FORMAT_VIDEO_pixelAspectRatio,
    VIDEO_MULTIVIEW_MODE: spa_sys::SPA_FORMAT_VIDEO_multiviewMode,
    VIDEO_MULTIVIEW_FLAGS: spa_sys::SPA_FORMAT_VIDEO_multiviewFlags,
    VIDEO_CHROMA_SITE: spa_sys::SPA_FORMAT_VIDEO_chromaSite,
    VIDEO_COLOR_RANGE: spa_sys::SPA_FORMAT_VIDEO_colorRange,
    VIDEO_COLOR_MATRIX: spa_sys::SPA_FORMAT_VIDEO_colorMatrix,
    VIDEO_TRANSFER_FUNCTION: spa_sys::SPA_FORMAT_VIDEO_transferFunction,
    VIDEO_COLOR_PRIMARIES: spa_sys::SPA_FORMAT_VIDEO_colorPrimaries,
    VIDEO_PROFILE: spa_sys::SPA_FORMAT_VIDEO_profile,
    VIDEO_LEVEL: spa_sys::SPA_FORMAT_VIDEO_level,
    VIDEO_H264_STREAM_FORMAT: spa_sys::SPA_FORMAT_VIDEO_H264_streamFormat,
    VIDEO_H264_ALIGNMENT: spa_sys::SPA_FORMAT_VIDEO_H264_alignment,
    _START_IMAGE: spa_sys::SPA_FORMAT_START_Image,
    _START_BINARY: spa_sys::SPA_FORMAT_START_Binary,
    _START_STREAM: spa_sys::SPA_FORMAT_START_Stream,
    _START_APPLICATION: spa_sys::SPA_FORMAT_START_Application,
);
impl PodIdType for Format {}

enum_wrapper!(
    MediaType,
    spa_sys::spa_media_type,
    UNKNOWN: spa_sys::SPA_MEDIA_TYPE_unknown,
    AUDIO: spa_sys::SPA_MEDIA_TYPE_audio,
    VIDEO: spa_sys::SPA_MEDIA_TYPE_video,
    IMAGE: spa_sys::SPA_MEDIA_TYPE_image,
    BINARY: spa_sys::SPA_MEDIA_TYPE_binary,
    STREAM: spa_sys::SPA_MEDIA_TYPE_stream,
    APPLICATION: spa_sys::SPA_MEDIA_TYPE_application,
);
impl PodIdType for MediaType {}

enum_wrapper!(
    MediaSubType,
    spa_sys::spa_media_subtype,
    UNKNOWN: spa_sys::SPA_MEDIA_SUBTYPE_unknown,
    RAW: spa_sys::SPA_MEDIA_SUBTYPE_raw,
    DSP: spa_sys::SPA_MEDIA_SUBTYPE_dsp,
    IEC958: spa_sys::SPA_MEDIA_SUBTYPE_iec958,
    DSD: spa_sys::SPA_MEDIA_SUBTYPE_dsd,
    _START_AUDIO: spa_sys::SPA_MEDIA_SUBTYPE_START_Audio,
    MP3: spa_sys::SPA_MEDIA_SUBTYPE_mp3,
    AAC: spa_sys::SPA_MEDIA_SUBTYPE_aac,
    VORBIS: spa_sys::SPA_MEDIA_SUBTYPE_vorbis,
    WMA: spa_sys::SPA_MEDIA_SUBTYPE_wma,
    RA: spa_sys::SPA_MEDIA_SUBTYPE_ra,
    SBC: spa_sys::SPA_MEDIA_SUBTYPE_sbc,
    ADPCM: spa_sys::SPA_MEDIA_SUBTYPE_adpcm,
    G723: spa_sys::SPA_MEDIA_SUBTYPE_g723,
    G726: spa_sys::SPA_MEDIA_SUBTYPE_g726,
    G729: spa_sys::SPA_MEDIA_SUBTYPE_g729,
    AMR: spa_sys::SPA_MEDIA_SUBTYPE_amr,
    GSM: spa_sys::SPA_MEDIA_SUBTYPE_gsm,
    _START_VIDEO: spa_sys::SPA_MEDIA_SUBTYPE_START_Video,
    H264: spa_sys::SPA_MEDIA_SUBTYPE_h264,
    MJPG: spa_sys::SPA_MEDIA_SUBTYPE_mjpg,
    DV: spa_sys::SPA_MEDIA_SUBTYPE_dv,
    MPEGTS: spa_sys::SPA_MEDIA_SUBTYPE_mpegts,
    H263: spa_sys::SPA_MEDIA_SUBTYPE_h263,
    MPEG1: spa_sys::SPA_MEDIA_SUBTYPE_mpeg1,
    MPEG2: spa_sys::SPA_MEDIA_SUBTYPE_mpeg2,
    MPEG4: spa_sys::SPA_MEDIA_SUBTYPE_mpeg4,
    XVID: spa_sys::SPA_MEDIA_SUBTYPE_xvid,
    VC1: spa_sys::SPA_MEDIA_SUBTYPE_vc1,
    VP8: spa_sys::SPA_MEDIA_SUBTYPE_vp8,
    VP9: spa_sys::SPA_MEDIA_SUBTYPE_vp9,
    BAYER: spa_sys::SPA_MEDIA_SUBTYPE_bayer,
    _START_IMAGE: spa_sys::SPA_MEDIA_SUBTYPE_START_Image,
    JPEG: spa_sys::SPA_MEDIA_SUBTYPE_jpeg,
    _START_BINARY: spa_sys::SPA_MEDIA_SUBTYPE_START_Binary,
    _START_STREAM: spa_sys::SPA_MEDIA_SUBTYPE_START_Stream,
    MIDI: spa_sys::SPA_MEDIA_SUBTYPE_midi,
    _START_APPLICATION: spa_sys::SPA_MEDIA_SUBTYPE_START_Application,
    CONTROL: spa_sys::SPA_MEDIA_SUBTYPE_control,
);
impl PodIdType for MediaSubType {}

enum_wrapper!(
    AudioFormat,
    spa_sys::spa_audio_format,
    UNKNOWN: spa_sys::SPA_AUDIO_FORMAT_UNKNOWN,
    ENCODED: spa_sys::SPA_AUDIO_FORMAT_ENCODED,
    _START_INTERLEAVED: spa_sys::SPA_AUDIO_FORMAT_START_Interleaved,
    S8: spa_sys::SPA_AUDIO_FORMAT_S8,
    U8: spa_sys::SPA_AUDIO_FORMAT_U8,
    S16_LE: spa_sys::SPA_AUDIO_FORMAT_S16_LE,
    S16_BE: spa_sys::SPA_AUDIO_FORMAT_S16_BE,
    U16_LE: spa_sys::SPA_AUDIO_FORMAT_U16_LE,
    U16_BE: spa_sys::SPA_AUDIO_FORMAT_U16_BE,
    S24_32_LE: spa_sys::SPA_AUDIO_FORMAT_S24_32_LE,
    S24_32_BE: spa_sys::SPA_AUDIO_FORMAT_S24_32_BE,
    U24_32_LE: spa_sys::SPA_AUDIO_FORMAT_U24_32_LE,
    U24_32_BE: spa_sys::SPA_AUDIO_FORMAT_U24_32_BE,
    S32_LE: spa_sys::SPA_AUDIO_FORMAT_S32_LE,
    S32_BE: spa_sys::SPA_AUDIO_FORMAT_S32_BE,
    U32_LE: spa_sys::SPA_AUDIO_FORMAT_U32_LE,
    U32_BE: spa_sys::SPA_AUDIO_FORMAT_U32_BE,
    S24_LE: spa_sys::SPA_AUDIO_FORMAT_S24_LE,
    S24_BE: spa_sys::SPA_AUDIO_FORMAT_S24_BE,
    U24_LE: spa_sys::SPA_AUDIO_FORMAT_U24_LE,
    U24_BE: spa_sys::SPA_AUDIO_FORMAT_U24_BE,
    S20_LE: spa_sys::SPA_AUDIO_FORMAT_S20_LE,
    S20_BE: spa_sys::SPA_AUDIO_FORMAT_S20_BE,
    U20_LE: spa_sys::SPA_AUDIO_FORMAT_U20_LE,
    U20_BE: spa_sys::SPA_AUDIO_FORMAT_U20_BE,
    S18_LE: spa_sys::SPA_AUDIO_FORMAT_S18_LE,
    S18_BE: spa_sys::SPA_AUDIO_FORMAT_S18_BE,
    U18_LE: spa_sys::SPA_AUDIO_FORMAT_U18_LE,
    U18_BE: spa_sys::SPA_AUDIO_FORMAT_U18_BE,
    F32_LE: spa_sys::SPA_AUDIO_FORMAT_F32_LE,
    F32_BE: spa_sys::SPA_AUDIO_FORMAT_F32_BE,
    F64_LE: spa_sys::SPA_AUDIO_FORMAT_F64_LE,
    F64_BE: spa_sys::SPA_AUDIO_FORMAT_F64_BE,
    ULAW: spa_sys::SPA_AUDIO_FORMAT_ULAW,
    ALAW: spa_sys::SPA_AUDIO_FORMAT_ALAW,
    _START_PLANAR: spa_sys::SPA_AUDIO_FORMAT_START_Planar,
    U8P: spa_sys::SPA_AUDIO_FORMAT_U8P,
    S16P: spa_sys::SPA_AUDIO_FORMAT_S16P,
    S24_32P: spa_sys::SPA_AUDIO_FORMAT_S24_32P,
    S32P: spa_sys::SPA_AUDIO_FORMAT_S32P,
    S24P: spa_sys::SPA_AUDIO_FORMAT_S24P,
    F32P: spa_sys::SPA_AUDIO_FORMAT_F32P,
    F64P: spa_sys::SPA_AUDIO_FORMAT_F64P,
    S8P: spa_sys::SPA_AUDIO_FORMAT_S8P,
    _START_OTHER: spa_sys::SPA_AUDIO_FORMAT_START_Other,
    DSP_S32: spa_sys::SPA_AUDIO_FORMAT_DSP_S32,
    DSP_F32: spa_sys::SPA_AUDIO_FORMAT_DSP_F32,
    DSP_F64: spa_sys::SPA_AUDIO_FORMAT_DSP_F64,
    S16: spa_sys::SPA_AUDIO_FORMAT_S16,
    U16: spa_sys::SPA_AUDIO_FORMAT_U16,
    S24_32: spa_sys::SPA_AUDIO_FORMAT_S24_32,
    U24_32: spa_sys::SPA_AUDIO_FORMAT_U24_32,
    S32: spa_sys::SPA_AUDIO_FORMAT_S32,
    U32: spa_sys::SPA_AUDIO_FORMAT_U32,
    S24: spa_sys::SPA_AUDIO_FORMAT_S24,
    U24: spa_sys::SPA_AUDIO_FORMAT_U24,
    S20: spa_sys::SPA_AUDIO_FORMAT_S20,
    U20: spa_sys::SPA_AUDIO_FORMAT_U20,
    S18: spa_sys::SPA_AUDIO_FORMAT_S18,
    U18: spa_sys::SPA_AUDIO_FORMAT_U18,
    F32: spa_sys::SPA_AUDIO_FORMAT_F32,
    F64: spa_sys::SPA_AUDIO_FORMAT_F64,
    S16_OE: spa_sys::SPA_AUDIO_FORMAT_S16_OE,
    U16_OE: spa_sys::SPA_AUDIO_FORMAT_U16_OE,
    S24_32_OE: spa_sys::SPA_AUDIO_FORMAT_S24_32_OE,
    U24_32_OE: spa_sys::SPA_AUDIO_FORMAT_U24_32_OE,
    S32_OE: spa_sys::SPA_AUDIO_FORMAT_S32_OE,
    U32_OE: spa_sys::SPA_AUDIO_FORMAT_U32_OE,
    S24_OE: spa_sys::SPA_AUDIO_FORMAT_S24_OE,
    U24_OE: spa_sys::SPA_AUDIO_FORMAT_U24_OE,
    S20_OE: spa_sys::SPA_AUDIO_FORMAT_S20_OE,
    U20_OE: spa_sys::SPA_AUDIO_FORMAT_U20_OE,
    S18_OE: spa_sys::SPA_AUDIO_FORMAT_S18_OE,
    U18_OE: spa_sys::SPA_AUDIO_FORMAT_U18_OE,
    F32_OE: spa_sys::SPA_AUDIO_FORMAT_F32_OE,
    F64_OE: spa_sys::SPA_AUDIO_FORMAT_F64_OE,
);
impl PodIdType for AudioFormat {}

enum_wrapper!(
    VideoFormat,
    spa_sys::spa_video_format,
    UNKNOWN: spa_sys::SPA_VIDEO_FORMAT_UNKNOWN,
    ENCODED: spa_sys::SPA_VIDEO_FORMAT_ENCODED,
    I420: spa_sys::SPA_VIDEO_FORMAT_I420,
    YV12: spa_sys::SPA_VIDEO_FORMAT_YV12,
    YUY2: spa_sys::SPA_VIDEO_FORMAT_YUY2,
    UYVY: spa_sys::SPA_VIDEO_FORMAT_UYVY,
    AYUV: spa_sys::SPA_VIDEO_FORMAT_AYUV,
    RGBX: spa_sys::SPA_VIDEO_FORMAT_RGBx,
    BGRX: spa_sys::SPA_VIDEO_FORMAT_BGRx,
    XRGB: spa_sys::SPA_VIDEO_FORMAT_xRGB,
    XBGR: spa_sys::SPA_VIDEO_FORMAT_xBGR,
    RGBA: spa_sys::SPA_VIDEO_FORMAT_RGBA,
    BGRA: spa_sys::SPA_VIDEO_FORMAT_BGRA,
    ARGB: spa_sys::SPA_VIDEO_FORMAT_ARGB,
    ABGR: spa_sys::SPA_VIDEO_FORMAT_ABGR,
    RGB: spa_sys::SPA_VIDEO_FORMAT_RGB,
    BGR: spa_sys::SPA_VIDEO_FORMAT_BGR,
    Y41B: spa_sys::SPA_VIDEO_FORMAT_Y41B,
    Y42B: spa_sys::SPA_VIDEO_FORMAT_Y42B,
    YVYU: spa_sys::SPA_VIDEO_FORMAT_YVYU,
    Y444: spa_sys::SPA_VIDEO_FORMAT_Y444,
    V210: spa_sys::SPA_VIDEO_FORMAT_v210,
    V216: spa_sys::SPA_VIDEO_FORMAT_v216,
    NV12: spa_sys::SPA_VIDEO_FORMAT_NV12,
    NV21: spa_sys::SPA_VIDEO_FORMAT_NV21,
    GRAY8: spa_sys::SPA_VIDEO_FORMAT_GRAY8,
    GRAY16_BE: spa_sys::SPA_VIDEO_FORMAT_GRAY16_BE,
    GRAY16_LE: spa_sys::SPA_VIDEO_FORMAT_GRAY16_LE,
    V308: spa_sys::SPA_VIDEO_FORMAT_v308,
    RGB16: spa_sys::SPA_VIDEO_FORMAT_RGB16,
    BGR16: spa_sys::SPA_VIDEO_FORMAT_BGR16,
    RGB15: spa_sys::SPA_VIDEO_FORMAT_RGB15,
    BGR15: spa_sys::SPA_VIDEO_FORMAT_BGR15,
    UYVP: spa_sys::SPA_VIDEO_FORMAT_UYVP,
    A420: spa_sys::SPA_VIDEO_FORMAT_A420,
    RGB8P: spa_sys::SPA_VIDEO_FORMAT_RGB8P,
    YUV9: spa_sys::SPA_VIDEO_FORMAT_YUV9,
    YVU9: spa_sys::SPA_VIDEO_FORMAT_YVU9,
    IYU1: spa_sys::SPA_VIDEO_FORMAT_IYU1,
    ARGB64: spa_sys::SPA_VIDEO_FORMAT_ARGB64,
    AYUV64: spa_sys::SPA_VIDEO_FORMAT_AYUV64,
    R210: spa_sys::SPA_VIDEO_FORMAT_r210,
    I420_10BE: spa_sys::SPA_VIDEO_FORMAT_I420_10BE,
    I420_10LE: spa_sys::SPA_VIDEO_FORMAT_I420_10LE,
    I422_10BE: spa_sys::SPA_VIDEO_FORMAT_I422_10BE,
    I422_10LE: spa_sys::SPA_VIDEO_FORMAT_I422_10LE,
    Y444_10BE: spa_sys::SPA_VIDEO_FORMAT_Y444_10BE,
    Y444_10LE: spa_sys::SPA_VIDEO_FORMAT_Y444_10LE,
    GBR: spa_sys::SPA_VIDEO_FORMAT_GBR,
    GBR_10BE: spa_sys::SPA_VIDEO_FORMAT_GBR_10BE,
    GBR_10LE: spa_sys::SPA_VIDEO_FORMAT_GBR_10LE,
    NV16: spa_sys::SPA_VIDEO_FORMAT_NV16,
    NV24: spa_sys::SPA_VIDEO_FORMAT_NV24,
    NV12_64Z32: spa_sys::SPA_VIDEO_FORMAT_NV12_64Z32,
    A420_10BE: spa_sys::SPA_VIDEO_FORMAT_A420_10BE,
    A420_10LE: spa_sys::SPA_VIDEO_FORMAT_A420_10LE,
    A422_10BE: spa_sys::SPA_VIDEO_FORMAT_A422_10BE,
    A422_10LE: spa_sys::SPA_VIDEO_FORMAT_A422_10LE,
    A444_10BE: spa_sys::SPA_VIDEO_FORMAT_A444_10BE,
    A444_10LE: spa_sys::SPA_VIDEO_FORMAT_A444_10LE,
    NV61: spa_sys::SPA_VIDEO_FORMAT_NV61,
    P010_10BE: spa_sys::SPA_VIDEO_FORMAT_P010_10BE,
    P010_10LE: spa_sys::SPA_VIDEO_FORMAT_P010_10LE,
    IYU2: spa_sys::SPA_VIDEO_FORMAT_IYU2,
    VYUY: spa_sys::SPA_VIDEO_FORMAT_VYUY,
    GBRA: spa_sys::SPA_VIDEO_FORMAT_GBRA,
    GBRA_10BE: spa_sys::SPA_VIDEO_FORMAT_GBRA_10BE,
    GBRA_10LE: spa_sys::SPA_VIDEO_FORMAT_GBRA_10LE,
    GBR_12BE: spa_sys::SPA_VIDEO_FORMAT_GBR_12BE,
    GBR_12LE: spa_sys::SPA_VIDEO_FORMAT_GBR_12LE,
    GBRA_12BE: spa_sys::SPA_VIDEO_FORMAT_GBRA_12BE,
    GBRA_12LE: spa_sys::SPA_VIDEO_FORMAT_GBRA_12LE,
    I420_12BE: spa_sys::SPA_VIDEO_FORMAT_I420_12BE,
    I420_12LE: spa_sys::SPA_VIDEO_FORMAT_I420_12LE,
    I422_12BE: spa_sys::SPA_VIDEO_FORMAT_I422_12BE,
    I422_12LE: spa_sys::SPA_VIDEO_FORMAT_I422_12LE,
    Y444_12BE: spa_sys::SPA_VIDEO_FORMAT_Y444_12BE,
    Y444_12LE: spa_sys::SPA_VIDEO_FORMAT_Y444_12LE,
    RGBA_F16: spa_sys::SPA_VIDEO_FORMAT_RGBA_F16,
    RGBA_F32: spa_sys::SPA_VIDEO_FORMAT_RGBA_F32,
    XRGB_210LE: spa_sys::SPA_VIDEO_FORMAT_xRGB_210LE,
    XBGR_210LE: spa_sys::SPA_VIDEO_FORMAT_xBGR_210LE,
    RGBX_102LE: spa_sys::SPA_VIDEO_FORMAT_RGBx_102LE,
    BGRX_102LE: spa_sys::SPA_VIDEO_FORMAT_BGRx_102LE,
    ARGB_210LE: spa_sys::SPA_VIDEO_FORMAT_ARGB_210LE,
    ABGR_210LE: spa_sys::SPA_VIDEO_FORMAT_ABGR_210LE,
    RGBA_102LE: spa_sys::SPA_VIDEO_FORMAT_RGBA_102LE,
    BGRA_102LE: spa_sys::SPA_VIDEO_FORMAT_BGRA_102LE,
    DSP_F32: spa_sys::SPA_VIDEO_FORMAT_DSP_F32,
);
impl PodIdType for VideoFormat {}

enum_wrapper!(
    ParamBitorder,
    spa_sys::spa_param_bitorder,
    UNKNOWN: spa_sys::SPA_PARAM_BITORDER_unknown,
    MSB: spa_sys::SPA_PARAM_BITORDER_msb,
    LSB: spa_sys::SPA_PARAM_BITORDER_lsb,
);
impl PodIdType for ParamBitorder {}

enum_wrapper!(
    VideoInterlaceMode,
    spa_sys::spa_video_interlace_mode,
    PROGRESSIVE: spa_sys::SPA_VIDEO_INTERLACE_MODE_PROGRESSIVE,
    INTERLEAVED: spa_sys::SPA_VIDEO_INTERLACE_MODE_INTERLEAVED,
    MIXED: spa_sys::SPA_VIDEO_INTERLACE_MODE_MIXED,
    FIELDS: spa_sys::SPA_VIDEO_INTERLACE_MODE_FIELDS,
);
impl PodIdType for VideoInterlaceMode {}

enum_wrapper!(
    VideoMultiviewMode,
    spa_sys::spa_video_multiview_mode,
    _NONE: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_NONE, // Impossible value, means no info. ID Pod cannot store values below zero
    MONO: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_MONO,
    LEFT: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_LEFT,
    RIGHT: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_RIGHT,
    SIDE_BY_SIDE: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_SIDE_BY_SIDE,
    SIDE_BY_SIDE_QUINCUNX: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_SIDE_BY_SIDE_QUINCUNX,
    COLUMN_INTERLEAVED: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_COLUMN_INTERLEAVED,
    ROW_INTERLEAVED: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_ROW_INTERLEAVED,
    TOP_BOTTOM: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_TOP_BOTTOM,
    CHECKERBOARD: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_CHECKERBOARD,
    FRAME_BY_FRAME: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_FRAME_BY_FRAME,
    MULTIVIEW_FRAME_BY_FRAME: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_MULTIVIEW_FRAME_BY_FRAME,
    SEPARATED: spa_sys::SPA_VIDEO_MULTIVIEW_MODE_SEPARATED,
);
impl From<u32> for VideoMultiviewMode {
    fn from(value: u32) -> Self {
        VideoMultiviewMode::from_raw(value as spa_sys::spa_video_multiview_mode)
    }
}
impl From<VideoMultiviewMode> for u32 {
    fn from(value: VideoMultiviewMode) -> Self {
        value.raw as Self
    }
}
impl PodIdType for VideoMultiviewMode {}

enum_wrapper!(
    VideoMultiviewFlags,
    spa_sys::spa_video_multiview_flags,
    NONE: spa_sys::SPA_VIDEO_MULTIVIEW_FLAGS_NONE,
    RIGHT_VIEW_FIRST: spa_sys::SPA_VIDEO_MULTIVIEW_FLAGS_RIGHT_VIEW_FIRST,
    LEFT_FLIPPED: spa_sys::SPA_VIDEO_MULTIVIEW_FLAGS_LEFT_FLIPPED,
    LEFT_FLOPPED: spa_sys::SPA_VIDEO_MULTIVIEW_FLAGS_LEFT_FLOPPED,
    RIGHT_FLIPPED: spa_sys::SPA_VIDEO_MULTIVIEW_FLAGS_RIGHT_FLIPPED,
    RIGHT_FLOPPED: spa_sys::SPA_VIDEO_MULTIVIEW_FLAGS_RIGHT_FLOPPED,
    HALF_ASPECT: spa_sys::SPA_VIDEO_MULTIVIEW_FLAGS_HALF_ASPECT,
    MIXED_MONO: spa_sys::SPA_VIDEO_MULTIVIEW_FLAGS_MIXED_MONO,
);
impl PodIdType for VideoMultiviewFlags {}

enum_wrapper!(
    VideoChromaSite,
    spa_sys::spa_video_chroma_site,
    UNKNOWN: spa_sys::SPA_VIDEO_CHROMA_SITE_UNKNOWN,
    NONE: spa_sys::SPA_VIDEO_CHROMA_SITE_NONE,
    H_COSITED: spa_sys::SPA_VIDEO_CHROMA_SITE_H_COSITED,
    V_COSITED: spa_sys::SPA_VIDEO_CHROMA_SITE_V_COSITED,
    ALT_LINE: spa_sys::SPA_VIDEO_CHROMA_SITE_ALT_LINE,
    COSITED: spa_sys::SPA_VIDEO_CHROMA_SITE_COSITED,
    JPEG: spa_sys::SPA_VIDEO_CHROMA_SITE_JPEG,
    MPEG2: spa_sys::SPA_VIDEO_CHROMA_SITE_MPEG2,
    DV: spa_sys::SPA_VIDEO_CHROMA_SITE_DV,
);
impl PodIdType for VideoChromaSite {}

enum_wrapper!(
    VideoColorRange,
    spa_sys::spa_video_color_range,
    UNKNOWN: spa_sys::SPA_VIDEO_COLOR_RANGE_UNKNOWN,
    RANGE_0_255: spa_sys::SPA_VIDEO_COLOR_RANGE_0_255,
    RANGE_16_235: spa_sys::SPA_VIDEO_COLOR_RANGE_16_235,
);
impl PodIdType for VideoColorRange {}

enum_wrapper!(
    VideoColorMatrix,
    spa_sys::spa_video_color_matrix,
    UNKNOWN: spa_sys::SPA_VIDEO_COLOR_MATRIX_UNKNOWN,
    RGB: spa_sys::SPA_VIDEO_COLOR_MATRIX_RGB,
    FCC: spa_sys::SPA_VIDEO_COLOR_MATRIX_FCC,
    BT709: spa_sys::SPA_VIDEO_COLOR_MATRIX_BT709,
    BT601: spa_sys::SPA_VIDEO_COLOR_MATRIX_BT601,
    SMPTE240M: spa_sys::SPA_VIDEO_COLOR_MATRIX_SMPTE240M,
    BT2020: spa_sys::SPA_VIDEO_COLOR_MATRIX_BT2020,
);
impl PodIdType for VideoColorMatrix {}

enum_wrapper!(
    VideoTransferFunction,
    spa_sys::spa_video_transfer_function,
    UNKNOWN: spa_sys::SPA_VIDEO_TRANSFER_UNKNOWN,
    GAMMA10: spa_sys::SPA_VIDEO_TRANSFER_GAMMA10,
    GAMMA18: spa_sys::SPA_VIDEO_TRANSFER_GAMMA18,
    GAMMA20: spa_sys::SPA_VIDEO_TRANSFER_GAMMA20,
    GAMMA22: spa_sys::SPA_VIDEO_TRANSFER_GAMMA22,
    BT709: spa_sys::SPA_VIDEO_TRANSFER_BT709,
    SMPTE240M: spa_sys::SPA_VIDEO_TRANSFER_SMPTE240M,
    SRGB: spa_sys::SPA_VIDEO_TRANSFER_SRGB,
    GAMMA28: spa_sys::SPA_VIDEO_TRANSFER_GAMMA28,
    LOG100: spa_sys::SPA_VIDEO_TRANSFER_LOG100,
    LOG316: spa_sys::SPA_VIDEO_TRANSFER_LOG316,
    BT2020_12: spa_sys::SPA_VIDEO_TRANSFER_BT2020_12,
    ADOBERGB: spa_sys::SPA_VIDEO_TRANSFER_ADOBERGB,
);
impl PodIdType for VideoTransferFunction {}

enum_wrapper!(
    VideoColorPrimaries,
    spa_sys::spa_video_color_primaries,
    UNKNOWN: spa_sys::SPA_VIDEO_COLOR_PRIMARIES_UNKNOWN,
    BT709: spa_sys::SPA_VIDEO_COLOR_PRIMARIES_BT709,
    BT470M: spa_sys::SPA_VIDEO_COLOR_PRIMARIES_BT470M,
    BT470BG: spa_sys::SPA_VIDEO_COLOR_PRIMARIES_BT470BG,
    SMPTE170M: spa_sys::SPA_VIDEO_COLOR_PRIMARIES_SMPTE170M,
    SMPTE240M: spa_sys::SPA_VIDEO_COLOR_PRIMARIES_SMPTE240M,
    FILM: spa_sys::SPA_VIDEO_COLOR_PRIMARIES_FILM,
    BT2020: spa_sys::SPA_VIDEO_COLOR_PRIMARIES_BT2020,
    ADOBERGB: spa_sys::SPA_VIDEO_COLOR_PRIMARIES_ADOBERGB,
);
impl PodIdType for VideoColorPrimaries {}

enum_wrapper!(
    VideoH264StreamFormat,
    spa_sys::spa_h264_stream_format,
    UNKNOWN: spa_sys::SPA_H264_STREAM_FORMAT_UNKNOWN,
    AVC: spa_sys::SPA_H264_STREAM_FORMAT_AVC,
    AVC3: spa_sys::SPA_H264_STREAM_FORMAT_AVC3,
    BYTESTREAM: spa_sys::SPA_H264_STREAM_FORMAT_BYTESTREAM,
);
impl PodIdType for VideoH264StreamFormat {}

enum_wrapper!(
    VideoH264Alignment,
    spa_sys::spa_h264_alignment,
    UNKNOWN: spa_sys::SPA_H264_ALIGNMENT_UNKNOWN,
    AU: spa_sys::SPA_H264_ALIGNMENT_AU,
    NAL: spa_sys::SPA_H264_ALIGNMENT_NAL,
);
impl PodIdType for VideoH264Alignment {}
