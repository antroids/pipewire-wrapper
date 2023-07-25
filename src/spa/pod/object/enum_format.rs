use std::io::{Seek, Write};

use crate::enum_wrapper;
use crate::spa::pod::array::PodArrayRef;
use crate::spa::pod::choice::enum_::PodEnumRef;
use crate::spa::pod::choice::flags::PodFlagsRef;
use crate::spa::pod::choice::range::PodRangeRef;
use crate::spa::pod::choice::PodChoiceRef;
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::object::format::*;
use crate::spa::pod::object::prop::AudioIec958Codec;
use crate::spa::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::pod::restricted::PodRawValue;
use crate::spa::pod::{
    BasicTypePod, PodError, PodFractionRef, PodIntRef, PodLongRef, PodRectangleRef, PodResult,
    WriteValue,
};
use crate::wrapper::RawWrapper;

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(u32)]
pub enum ObjectEnumFormatType<'a> {
    // Media
    MEDIA_TYPE(&'a PodIdRef<MediaType>) = Format::MEDIA_TYPE.raw,
    MEDIA_SUBTYPE(&'a PodIdRef<MediaSubType>) = Format::MEDIA_SUBTYPE.raw,

    // Audio
    AUDIO_FORMAT(&'a PodChoiceRef<PodIdRef<AudioFormat>>) = Format::AUDIO_FORMAT.raw,
    AUDIO_FLAGS(&'a PodFlagsRef<PodIntRef>) = Format::AUDIO_FLAGS.raw,
    AUDIO_RATE(&'a PodChoiceRef<PodIntRef>) = Format::AUDIO_RATE.raw,
    AUDIO_CHANNELS(&'a PodChoiceRef<PodIntRef>) = Format::AUDIO_CHANNELS.raw,
    AUDIO_POSITION(&'a PodArrayRef<PodIdRef<u32>>) = Format::AUDIO_POSITION.raw, // Enum in comments, but getting array
    AUDIO_IEC958CODEC(&'a PodChoiceRef<PodIdRef<AudioIec958Codec>>) = Format::AUDIO_IEC958CODEC.raw,
    AUDIO_BITORDER(&'a PodChoiceRef<PodIdRef<ParamBitorder>>) = Format::AUDIO_BITORDER.raw,
    AUDIO_INTERLEAVE(&'a PodChoiceRef<PodIntRef>) = Format::AUDIO_INTERLEAVE.raw,
    // missing audio params

    // Video
    VIDEO_FORMAT(&'a PodChoiceRef<PodIdRef<VideoFormat>>) = Format::VIDEO_FORMAT.raw,
    VIDEO_MODIFIER(&'a PodLongRef) = Format::VIDEO_MODIFIER.raw,
    VIDEO_SIZE(&'a PodChoiceRef<PodRectangleRef>) = Format::VIDEO_SIZE.raw,
    VIDEO_FRAMERATE(&'a PodChoiceRef<PodFractionRef>) = Format::VIDEO_FRAMERATE.raw,
    VIDEO_MAX_FRAMERATE(&'a PodChoiceRef<PodFractionRef>) = Format::VIDEO_MAX_FRAMERATE.raw,
    VIDEO_VIEWS(&'a PodIntRef) = Format::VIDEO_VIEWS.raw,
    VIDEO_INTERLACE_MODE(&'a PodChoiceRef<PodIdRef<VideoInterlaceMode>>) =
        Format::VIDEO_INTERLACE_MODE.raw,
    VIDEO_PIXEL_ASPECT_RATIO(&'a PodChoiceRef<PodRectangleRef>) =
        Format::VIDEO_PIXEL_ASPECT_RATIO.raw,
    VIDEO_MULTIVIEW_MODE(&'a PodChoiceRef<PodIdRef<VideoMultiviewMode>>) =
        Format::VIDEO_MULTIVIEW_MODE.raw,
    VIDEO_MULTIVIEW_FLAGS(&'a PodChoiceRef<PodIdRef<VideoMultiviewFlags>>) =
        Format::VIDEO_MULTIVIEW_FLAGS.raw,
    VIDEO_CHROMA_SITE(&'a PodChoiceRef<PodIdRef<VideoChromaSite>>) = Format::VIDEO_CHROMA_SITE.raw,
    VIDEO_COLOR_RANGE(&'a PodChoiceRef<PodIdRef<VideoColorRange>>) = Format::VIDEO_COLOR_RANGE.raw,
    VIDEO_COLOR_MATRIX(&'a PodChoiceRef<PodIdRef<VideoColorMatrix>>) =
        Format::VIDEO_COLOR_MATRIX.raw,
    VIDEO_TRANSFER_FUNCTION(&'a PodChoiceRef<PodIdRef<VideoTransferFunction>>) =
        Format::VIDEO_TRANSFER_FUNCTION.raw,
    VIDEO_COLOR_PRIMARIES(&'a PodChoiceRef<PodIdRef<VideoColorPrimaries>>) =
        Format::VIDEO_COLOR_PRIMARIES.raw,
    VIDEO_PROFILE(&'a PodChoiceRef<PodIntRef>) = Format::VIDEO_PROFILE.raw,
    VIDEO_LEVEL(&'a PodChoiceRef<PodIntRef>) = Format::VIDEO_LEVEL.raw,
    VIDEO_H264_STREAM_FORMAT(&'a PodChoiceRef<PodIdRef<VideoH264StreamFormat>>) =
        Format::VIDEO_H264_STREAM_FORMAT.raw,
    VIDEO_H264_ALIGNMENT(&'a PodChoiceRef<PodIdRef<VideoH264Alignment>>) =
        Format::VIDEO_H264_ALIGNMENT.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ObjectEnumFormatType<'a>>> for ObjectEnumFormatType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ObjectEnumFormatType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match Format::from_raw(value.raw.key) {
                Format::MEDIA_TYPE => Ok(ObjectEnumFormatType::MEDIA_TYPE(value.pod().cast()?)),
                Format::MEDIA_SUBTYPE => {
                    Ok(ObjectEnumFormatType::MEDIA_SUBTYPE(value.pod().cast()?))
                }
                Format::AUDIO_FORMAT => Ok(ObjectEnumFormatType::AUDIO_FORMAT(value.pod().cast()?)),
                Format::AUDIO_FLAGS => Ok(ObjectEnumFormatType::AUDIO_FLAGS(value.pod().cast()?)),
                Format::AUDIO_RATE => Ok(ObjectEnumFormatType::AUDIO_RATE(value.pod().cast()?)),
                Format::AUDIO_CHANNELS => {
                    Ok(ObjectEnumFormatType::AUDIO_CHANNELS(value.pod().cast()?))
                }
                Format::AUDIO_POSITION => {
                    Ok(ObjectEnumFormatType::AUDIO_POSITION(value.pod().cast()?))
                }
                Format::AUDIO_IEC958CODEC => {
                    Ok(ObjectEnumFormatType::AUDIO_IEC958CODEC(value.pod().cast()?))
                }
                Format::AUDIO_BITORDER => {
                    Ok(ObjectEnumFormatType::AUDIO_BITORDER(value.pod().cast()?))
                }
                Format::AUDIO_INTERLEAVE => {
                    Ok(ObjectEnumFormatType::AUDIO_INTERLEAVE(value.pod().cast()?))
                }
                Format::VIDEO_FORMAT => Ok(ObjectEnumFormatType::VIDEO_FORMAT(value.pod().cast()?)),
                Format::VIDEO_MODIFIER => {
                    Ok(ObjectEnumFormatType::VIDEO_MODIFIER(value.pod().cast()?))
                }
                Format::VIDEO_SIZE => Ok(ObjectEnumFormatType::VIDEO_SIZE(value.pod().cast()?)),
                Format::VIDEO_FRAMERATE => {
                    Ok(ObjectEnumFormatType::VIDEO_FRAMERATE(value.pod().cast()?))
                }
                Format::VIDEO_MAX_FRAMERATE => Ok(ObjectEnumFormatType::VIDEO_MAX_FRAMERATE(
                    value.pod().cast()?,
                )),
                Format::VIDEO_VIEWS => Ok(ObjectEnumFormatType::VIDEO_VIEWS(value.pod().cast()?)),
                Format::VIDEO_INTERLACE_MODE => Ok(ObjectEnumFormatType::VIDEO_INTERLACE_MODE(
                    value.pod().cast()?,
                )),
                Format::VIDEO_PIXEL_ASPECT_RATIO => Ok(
                    ObjectEnumFormatType::VIDEO_PIXEL_ASPECT_RATIO(value.pod().cast()?),
                ),
                Format::VIDEO_MULTIVIEW_MODE => Ok(ObjectEnumFormatType::VIDEO_MULTIVIEW_MODE(
                    value.pod().cast()?,
                )),
                Format::VIDEO_MULTIVIEW_FLAGS => Ok(ObjectEnumFormatType::VIDEO_MULTIVIEW_FLAGS(
                    value.pod().cast()?,
                )),
                Format::VIDEO_CHROMA_SITE => {
                    Ok(ObjectEnumFormatType::VIDEO_CHROMA_SITE(value.pod().cast()?))
                }
                Format::VIDEO_COLOR_RANGE => {
                    Ok(ObjectEnumFormatType::VIDEO_COLOR_RANGE(value.pod().cast()?))
                }
                Format::VIDEO_COLOR_MATRIX => Ok(ObjectEnumFormatType::VIDEO_COLOR_MATRIX(
                    value.pod().cast()?,
                )),
                Format::VIDEO_TRANSFER_FUNCTION => Ok(
                    ObjectEnumFormatType::VIDEO_TRANSFER_FUNCTION(value.pod().cast()?),
                ),
                Format::VIDEO_COLOR_PRIMARIES => Ok(ObjectEnumFormatType::VIDEO_COLOR_PRIMARIES(
                    value.pod().cast()?,
                )),
                Format::VIDEO_PROFILE => {
                    Ok(ObjectEnumFormatType::VIDEO_PROFILE(value.pod().cast()?))
                }
                Format::VIDEO_LEVEL => Ok(ObjectEnumFormatType::VIDEO_LEVEL(value.pod().cast()?)),
                Format::VIDEO_H264_STREAM_FORMAT => Ok(
                    ObjectEnumFormatType::VIDEO_H264_STREAM_FORMAT(value.pod().cast()?),
                ),
                Format::VIDEO_H264_ALIGNMENT => Ok(ObjectEnumFormatType::VIDEO_H264_ALIGNMENT(
                    value.pod().cast()?,
                )),
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ObjectEnumFormatType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match self {
            ObjectEnumFormatType::MEDIA_TYPE(pod) => {
                Self::write_pod_prop(buffer, Format::MEDIA_TYPE.raw, 0, pod)
            }
            ObjectEnumFormatType::MEDIA_SUBTYPE(pod) => {
                Self::write_pod_prop(buffer, Format::MEDIA_SUBTYPE.raw, 0, pod)
            }
            ObjectEnumFormatType::AUDIO_FORMAT(pod) => {
                Self::write_pod_prop(buffer, Format::AUDIO_FORMAT.raw, 0, pod)
            }
            ObjectEnumFormatType::AUDIO_FLAGS(pod) => {
                Self::write_pod_prop(buffer, Format::AUDIO_FLAGS.raw, 0, pod)
            }
            ObjectEnumFormatType::AUDIO_RATE(pod) => {
                Self::write_pod_prop(buffer, Format::AUDIO_RATE.raw, 0, pod)
            }
            ObjectEnumFormatType::AUDIO_CHANNELS(pod) => {
                Self::write_pod_prop(buffer, Format::AUDIO_CHANNELS.raw, 0, pod)
            }
            ObjectEnumFormatType::AUDIO_POSITION(pod) => {
                Self::write_pod_prop(buffer, Format::AUDIO_POSITION.raw, 0, pod)
            }
            ObjectEnumFormatType::AUDIO_IEC958CODEC(pod) => {
                Self::write_pod_prop(buffer, Format::AUDIO_IEC958CODEC.raw, 0, pod)
            }
            ObjectEnumFormatType::AUDIO_BITORDER(pod) => {
                Self::write_pod_prop(buffer, Format::AUDIO_BITORDER.raw, 0, pod)
            }
            ObjectEnumFormatType::AUDIO_INTERLEAVE(pod) => {
                Self::write_pod_prop(buffer, Format::AUDIO_INTERLEAVE.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_FORMAT(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_FORMAT.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_MODIFIER(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_MODIFIER.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_SIZE(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_SIZE.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_FRAMERATE(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_FRAMERATE.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_MAX_FRAMERATE(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_MAX_FRAMERATE.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_VIEWS(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_VIEWS.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_INTERLACE_MODE(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_INTERLACE_MODE.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_PIXEL_ASPECT_RATIO(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_PIXEL_ASPECT_RATIO.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_MULTIVIEW_MODE(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_MULTIVIEW_MODE.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_MULTIVIEW_FLAGS(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_MULTIVIEW_FLAGS.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_CHROMA_SITE(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_CHROMA_SITE.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_COLOR_RANGE(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_COLOR_RANGE.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_COLOR_MATRIX(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_COLOR_MATRIX.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_TRANSFER_FUNCTION(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_TRANSFER_FUNCTION.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_COLOR_PRIMARIES(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_COLOR_PRIMARIES.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_PROFILE(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_PROFILE.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_LEVEL(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_LEVEL.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_H264_STREAM_FORMAT(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_H264_STREAM_FORMAT.raw, 0, pod)
            }
            ObjectEnumFormatType::VIDEO_H264_ALIGNMENT(pod) => {
                Self::write_pod_prop(buffer, Format::VIDEO_H264_ALIGNMENT.raw, 0, pod)
            }
        }
    }
}
