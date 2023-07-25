use std::io::{Seek, Write};

use crate::enum_wrapper;
use crate::spa::pod::array::PodArrayRef;
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::pod::string::PodStringRef;
use crate::spa::pod::struct_::PodStructRef;
use crate::spa::pod::{
    BasicTypePod, PodBoolRef, PodDoubleRef, PodError, PodFdRef, PodFloatRef, PodIntRef, PodLongRef,
    PodResult,
};
use crate::wrapper::RawWrapper;

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(u32)]
pub enum ObjectPropType<'a> {
    // Device
    DEVICE(&'a PodStringRef) = Prop::DEVICE.raw,
    DEVICE_NAME(&'a PodStringRef) = Prop::DEVICE_NAME.raw,
    DEVICE_FD(&'a PodFdRef) = Prop::DEVICE_FD.raw,
    CARD(&'a PodStringRef) = Prop::CARD.raw,
    CARD_NAME(&'a PodStringRef) = Prop::CARD_NAME.raw,
    MIN_LATENCY(&'a PodIntRef) = Prop::MIN_LATENCY.raw,
    MAX_LATENCY(&'a PodIntRef) = Prop::MAX_LATENCY.raw,
    PERIODS(&'a PodIntRef) = Prop::PERIODS.raw,
    PERIOD_SIZE(&'a PodIntRef) = Prop::PERIOD_SIZE.raw,
    PERIOD_EVENT(&'a PodBoolRef) = Prop::PERIOD_EVENT.raw,
    LIVE(&'a PodBoolRef) = Prop::LIVE.raw,
    RATE(&'a PodDoubleRef) = Prop::RATE.raw,
    QUALITY(&'a PodIntRef) = Prop::QUALITY.raw,
    BLUETOOTH_AUDIO_CODEC(&'a PodIdRef<BluetoothAudioCodec>) = Prop::BLUETOOTH_AUDIO_CODEC.raw,
    // Audio
    WAVE_TYPE(&'a PodIdRef<u32>) = Prop::WAVE_TYPE.raw,
    FREQUENCY(&'a PodIntRef) = Prop::FREQUENCY.raw,
    VOLUME(&'a PodFloatRef) = Prop::VOLUME.raw,
    MUTE(&'a PodBoolRef) = Prop::MUTE.raw,
    PATTERN_TYPE(&'a PodIdRef<u32>) = Prop::PATTERN_TYPE.raw,
    DITHER_TYPE(&'a PodIdRef<u32>) = Prop::DITHER_TYPE.raw,
    TRUNCATE(&'a PodBoolRef) = Prop::TRUNCATE.raw,
    CHANNEL_VOLUMES(&'a PodArrayRef<PodFloatRef>) = Prop::CHANNEL_VOLUMES.raw,
    VOLUME_BASE(&'a PodFloatRef) = Prop::VOLUME_BASE.raw,
    VOLUME_STEP(&'a PodFloatRef) = Prop::VOLUME_STEP.raw,
    CHANNEL_MAP(&'a PodArrayRef<PodIdRef<AudioChannel>>) = Prop::CHANNEL_MAP.raw,
    MONITOR_MUTE(&'a PodBoolRef) = Prop::MONITOR_MUTE.raw,
    MONITOR_VOLUMES(&'a PodArrayRef<PodFloatRef>) = Prop::MONITOR_VOLUMES.raw,
    LATENCY_OFFSET_NSEC(&'a PodLongRef) = Prop::LATENCY_OFFSET_NSEC.raw,
    SOFT_MUTE(&'a PodBoolRef) = Prop::SOFT_MUTE.raw,
    SOFT_VOLUMES(&'a PodArrayRef<PodFloatRef>) = Prop::SOFT_VOLUMES.raw,
    IEC958_CODECS(&'a PodArrayRef<PodIdRef<AudioIec958Codec>>) = Prop::IEC958_CODECS.raw,
    // Ramp missing
    // Video
    BRIGHTNESS(&'a PodIntRef) = Prop::BRIGHTNESS.raw,
    CONTRAST(&'a PodIntRef) = Prop::CONTRAST.raw,
    SATURATION(&'a PodIntRef) = Prop::SATURATION.raw,
    HUE(&'a PodIntRef) = Prop::HUE.raw,
    GAMMA(&'a PodIntRef) = Prop::GAMMA.raw,
    EXPOSURE(&'a PodIntRef) = Prop::EXPOSURE.raw,
    GAIN(&'a PodIntRef) = Prop::GAIN.raw,
    SHARPNESS(&'a PodIntRef) = Prop::SHARPNESS.raw,
    PARAMS(&'a PodStructRef) = Prop::PARAMS.raw,
}

impl<'a> PodPropKeyType<'a> for ObjectPropType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match self {
            ObjectPropType::DEVICE(pod) => Self::write_pod_prop(buffer, Prop::DEVICE.raw, 0, pod),
            ObjectPropType::DEVICE_NAME(pod) => {
                Self::write_pod_prop(buffer, Prop::DEVICE_NAME.raw, 0, pod)
            }
            ObjectPropType::DEVICE_FD(pod) => {
                Self::write_pod_prop(buffer, Prop::DEVICE_FD.raw, 0, pod)
            }
            ObjectPropType::CARD(pod) => Self::write_pod_prop(buffer, Prop::CARD.raw, 0, pod),
            ObjectPropType::CARD_NAME(pod) => {
                Self::write_pod_prop(buffer, Prop::CARD_NAME.raw, 0, pod)
            }
            ObjectPropType::MIN_LATENCY(pod) => {
                Self::write_pod_prop(buffer, Prop::MIN_LATENCY.raw, 0, pod)
            }
            ObjectPropType::MAX_LATENCY(pod) => {
                Self::write_pod_prop(buffer, Prop::MAX_LATENCY.raw, 0, pod)
            }
            ObjectPropType::PERIODS(pod) => Self::write_pod_prop(buffer, Prop::PERIODS.raw, 0, pod),
            ObjectPropType::PERIOD_SIZE(pod) => {
                Self::write_pod_prop(buffer, Prop::PERIOD_SIZE.raw, 0, pod)
            }
            ObjectPropType::PERIOD_EVENT(pod) => {
                Self::write_pod_prop(buffer, Prop::PERIOD_EVENT.raw, 0, pod)
            }
            ObjectPropType::LIVE(pod) => Self::write_pod_prop(buffer, Prop::LIVE.raw, 0, pod),
            ObjectPropType::RATE(pod) => Self::write_pod_prop(buffer, Prop::RATE.raw, 0, pod),
            ObjectPropType::QUALITY(pod) => Self::write_pod_prop(buffer, Prop::QUALITY.raw, 0, pod),
            ObjectPropType::BLUETOOTH_AUDIO_CODEC(pod) => {
                Self::write_pod_prop(buffer, Prop::BLUETOOTH_AUDIO_CODEC.raw, 0, pod)
            }
            ObjectPropType::WAVE_TYPE(pod) => {
                Self::write_pod_prop(buffer, Prop::WAVE_TYPE.raw, 0, pod)
            }
            ObjectPropType::FREQUENCY(pod) => {
                Self::write_pod_prop(buffer, Prop::FREQUENCY.raw, 0, pod)
            }
            ObjectPropType::VOLUME(pod) => Self::write_pod_prop(buffer, Prop::VOLUME.raw, 0, pod),
            ObjectPropType::MUTE(pod) => Self::write_pod_prop(buffer, Prop::MUTE.raw, 0, pod),
            ObjectPropType::PATTERN_TYPE(pod) => {
                Self::write_pod_prop(buffer, Prop::PATTERN_TYPE.raw, 0, pod)
            }
            ObjectPropType::DITHER_TYPE(pod) => {
                Self::write_pod_prop(buffer, Prop::DITHER_TYPE.raw, 0, pod)
            }
            ObjectPropType::TRUNCATE(pod) => {
                Self::write_pod_prop(buffer, Prop::TRUNCATE.raw, 0, pod)
            }
            ObjectPropType::CHANNEL_VOLUMES(pod) => {
                Self::write_pod_prop(buffer, Prop::CHANNEL_VOLUMES.raw, 0, pod)
            }
            ObjectPropType::VOLUME_BASE(pod) => {
                Self::write_pod_prop(buffer, Prop::VOLUME_BASE.raw, 0, pod)
            }
            ObjectPropType::VOLUME_STEP(pod) => {
                Self::write_pod_prop(buffer, Prop::VOLUME_STEP.raw, 0, pod)
            }
            ObjectPropType::CHANNEL_MAP(pod) => {
                Self::write_pod_prop(buffer, Prop::CHANNEL_MAP.raw, 0, pod)
            }
            ObjectPropType::MONITOR_MUTE(pod) => {
                Self::write_pod_prop(buffer, Prop::MONITOR_MUTE.raw, 0, pod)
            }
            ObjectPropType::MONITOR_VOLUMES(pod) => {
                Self::write_pod_prop(buffer, Prop::MONITOR_VOLUMES.raw, 0, pod)
            }
            ObjectPropType::LATENCY_OFFSET_NSEC(pod) => {
                Self::write_pod_prop(buffer, Prop::LATENCY_OFFSET_NSEC.raw, 0, pod)
            }
            ObjectPropType::SOFT_MUTE(pod) => {
                Self::write_pod_prop(buffer, Prop::SOFT_MUTE.raw, 0, pod)
            }
            ObjectPropType::SOFT_VOLUMES(pod) => {
                Self::write_pod_prop(buffer, Prop::SOFT_VOLUMES.raw, 0, pod)
            }
            ObjectPropType::IEC958_CODECS(pod) => {
                Self::write_pod_prop(buffer, Prop::IEC958_CODECS.raw, 0, pod)
            }
            ObjectPropType::BRIGHTNESS(pod) => {
                Self::write_pod_prop(buffer, Prop::BRIGHTNESS.raw, 0, pod)
            }
            ObjectPropType::CONTRAST(pod) => {
                Self::write_pod_prop(buffer, Prop::CONTRAST.raw, 0, pod)
            }
            ObjectPropType::SATURATION(pod) => {
                Self::write_pod_prop(buffer, Prop::SATURATION.raw, 0, pod)
            }
            ObjectPropType::HUE(pod) => Self::write_pod_prop(buffer, Prop::HUE.raw, 0, pod),
            ObjectPropType::GAMMA(pod) => Self::write_pod_prop(buffer, Prop::GAMMA.raw, 0, pod),
            ObjectPropType::EXPOSURE(pod) => {
                Self::write_pod_prop(buffer, Prop::EXPOSURE.raw, 0, pod)
            }
            ObjectPropType::GAIN(pod) => Self::write_pod_prop(buffer, Prop::GAIN.raw, 0, pod),
            ObjectPropType::SHARPNESS(pod) => {
                Self::write_pod_prop(buffer, Prop::SHARPNESS.raw, 0, pod)
            }
            ObjectPropType::PARAMS(pod) => Self::write_pod_prop(buffer, Prop::PARAMS.raw, 0, pod),
        }
    }
}

impl<'a> TryFrom<&'a PodPropRef<'a, ObjectPropType<'a>>> for ObjectPropType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ObjectPropType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match Prop::from_raw(value.raw.key) {
                Prop::DEVICE => Ok(ObjectPropType::DEVICE(value.pod().cast()?)),
                Prop::DEVICE_NAME => Ok(ObjectPropType::DEVICE_NAME(value.pod().cast()?)),
                Prop::DEVICE_FD => Ok(ObjectPropType::DEVICE_FD(value.pod().cast()?)),
                Prop::CARD => Ok(ObjectPropType::CARD(value.pod().cast()?)),
                Prop::CARD_NAME => Ok(ObjectPropType::CARD_NAME(value.pod().cast()?)),
                Prop::MIN_LATENCY => Ok(ObjectPropType::MIN_LATENCY(value.pod().cast()?)),
                Prop::MAX_LATENCY => Ok(ObjectPropType::MAX_LATENCY(value.pod().cast()?)),
                Prop::PERIODS => Ok(ObjectPropType::PERIODS(value.pod().cast()?)),
                Prop::PERIOD_SIZE => Ok(ObjectPropType::PERIOD_SIZE(value.pod().cast()?)),
                Prop::PERIOD_EVENT => Ok(ObjectPropType::PERIOD_EVENT(value.pod().cast()?)),
                Prop::LIVE => Ok(ObjectPropType::LIVE(value.pod().cast()?)),
                Prop::RATE => Ok(ObjectPropType::RATE(value.pod().cast()?)),
                Prop::QUALITY => Ok(ObjectPropType::QUALITY(value.pod().cast()?)),
                Prop::BLUETOOTH_AUDIO_CODEC => {
                    Ok(ObjectPropType::BLUETOOTH_AUDIO_CODEC(value.pod().cast()?))
                }
                Prop::WAVE_TYPE => Ok(ObjectPropType::WAVE_TYPE(value.pod().cast()?)),
                Prop::FREQUENCY => Ok(ObjectPropType::FREQUENCY(value.pod().cast()?)),
                Prop::VOLUME => Ok(ObjectPropType::VOLUME(value.pod().cast()?)),
                Prop::MUTE => Ok(ObjectPropType::MUTE(value.pod().cast()?)),
                Prop::PATTERN_TYPE => Ok(ObjectPropType::PATTERN_TYPE(value.pod().cast()?)),
                Prop::DITHER_TYPE => Ok(ObjectPropType::DITHER_TYPE(value.pod().cast()?)),
                Prop::TRUNCATE => Ok(ObjectPropType::TRUNCATE(value.pod().cast()?)),
                Prop::CHANNEL_VOLUMES => Ok(ObjectPropType::CHANNEL_VOLUMES(value.pod().cast()?)),
                Prop::VOLUME_BASE => Ok(ObjectPropType::VOLUME_BASE(value.pod().cast()?)),
                Prop::VOLUME_STEP => Ok(ObjectPropType::VOLUME_STEP(value.pod().cast()?)),
                Prop::CHANNEL_MAP => Ok(ObjectPropType::CHANNEL_MAP(value.pod().cast()?)),
                Prop::MONITOR_MUTE => Ok(ObjectPropType::MONITOR_MUTE(value.pod().cast()?)),
                Prop::MONITOR_VOLUMES => Ok(ObjectPropType::MONITOR_VOLUMES(value.pod().cast()?)),
                Prop::LATENCY_OFFSET_NSEC => {
                    Ok(ObjectPropType::LATENCY_OFFSET_NSEC(value.pod().cast()?))
                }
                Prop::SOFT_MUTE => Ok(ObjectPropType::SOFT_MUTE(value.pod().cast()?)),
                Prop::SOFT_VOLUMES => Ok(ObjectPropType::SOFT_VOLUMES(value.pod().cast()?)),
                Prop::IEC958_CODECS => Ok(ObjectPropType::IEC958_CODECS(value.pod().cast()?)),
                Prop::BRIGHTNESS => Ok(ObjectPropType::BRIGHTNESS(value.pod().cast()?)),
                Prop::CONTRAST => Ok(ObjectPropType::CONTRAST(value.pod().cast()?)),
                Prop::SATURATION => Ok(ObjectPropType::SATURATION(value.pod().cast()?)),
                Prop::HUE => Ok(ObjectPropType::HUE(value.pod().cast()?)),
                Prop::GAMMA => Ok(ObjectPropType::GAMMA(value.pod().cast()?)),
                Prop::EXPOSURE => Ok(ObjectPropType::EXPOSURE(value.pod().cast()?)),
                Prop::GAIN => Ok(ObjectPropType::GAIN(value.pod().cast()?)),
                Prop::SHARPNESS => Ok(ObjectPropType::SHARPNESS(value.pod().cast()?)),
                Prop::PARAMS => Ok(ObjectPropType::PARAMS(value.pod().cast()?)),
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl PodIdType for Prop {}

impl PodIdType for BluetoothAudioCodec {}

impl PodIdType for AudioChannel {}

impl PodIdType for AudioIec958Codec {}

enum_wrapper!(
    Prop,
    spa_sys::spa_prop,
    _START: spa_sys::SPA_PROP_START,
    UNKNOWN: spa_sys::SPA_PROP_unknown,
    _START_DEVICE: spa_sys::SPA_PROP_START_Device,
    DEVICE: spa_sys::SPA_PROP_device,
    DEVICE_NAME: spa_sys::SPA_PROP_deviceName,
    DEVICE_FD: spa_sys::SPA_PROP_deviceFd,
    CARD: spa_sys::SPA_PROP_card,
    CARD_NAME: spa_sys::SPA_PROP_cardName,
    MIN_LATENCY: spa_sys::SPA_PROP_minLatency,
    MAX_LATENCY: spa_sys::SPA_PROP_maxLatency,
    PERIODS: spa_sys::SPA_PROP_periods,
    PERIOD_SIZE: spa_sys::SPA_PROP_periodSize,
    PERIOD_EVENT: spa_sys::SPA_PROP_periodEvent,
    LIVE: spa_sys::SPA_PROP_live,
    RATE: spa_sys::SPA_PROP_rate,
    QUALITY: spa_sys::SPA_PROP_quality,
    BLUETOOTH_AUDIO_CODEC: spa_sys::SPA_PROP_bluetoothAudioCodec,
    //SPA_PROP_bluetoothOffloadActive
    _START_AUDIO: spa_sys::SPA_PROP_START_Audio,
    WAVE_TYPE: spa_sys::SPA_PROP_waveType,
    FREQUENCY: spa_sys::SPA_PROP_frequency,
    VOLUME: spa_sys::SPA_PROP_volume,
    MUTE: spa_sys::SPA_PROP_mute,
    PATTERN_TYPE: spa_sys::SPA_PROP_patternType,
    DITHER_TYPE: spa_sys::SPA_PROP_ditherType,
    TRUNCATE: spa_sys::SPA_PROP_truncate,
    CHANNEL_VOLUMES: spa_sys::SPA_PROP_channelVolumes,
    VOLUME_BASE: spa_sys::SPA_PROP_volumeBase,
    VOLUME_STEP: spa_sys::SPA_PROP_volumeStep,
    CHANNEL_MAP: spa_sys::SPA_PROP_channelMap,
    MONITOR_MUTE: spa_sys::SPA_PROP_monitorMute,
    MONITOR_VOLUMES: spa_sys::SPA_PROP_monitorVolumes,
    LATENCY_OFFSET_NSEC: spa_sys::SPA_PROP_latencyOffsetNsec,
    SOFT_MUTE: spa_sys::SPA_PROP_softMute,
    SOFT_VOLUMES: spa_sys::SPA_PROP_softVolumes,
    IEC958_CODECS: spa_sys::SPA_PROP_iec958Codecs,
    _START_VIDEO: spa_sys::SPA_PROP_START_Video,
    BRIGHTNESS: spa_sys::SPA_PROP_brightness,
    CONTRAST: spa_sys::SPA_PROP_contrast,
    SATURATION: spa_sys::SPA_PROP_saturation,
    HUE: spa_sys::SPA_PROP_hue,
    GAMMA: spa_sys::SPA_PROP_gamma,
    EXPOSURE: spa_sys::SPA_PROP_exposure,
    GAIN: spa_sys::SPA_PROP_gain,
    SHARPNESS: spa_sys::SPA_PROP_sharpness,
    _START_OTHER: spa_sys::SPA_PROP_START_Other,
    PARAMS: spa_sys::SPA_PROP_params,
    _START_CUSTOM: spa_sys::SPA_PROP_START_CUSTOM,
);

enum_wrapper!(
    BluetoothAudioCodec,
    spa_sys::spa_bluetooth_audio_codec,
    START: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_START,
    SBC: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_SBC,
    SBC_XQ: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_SBC_XQ,
    MPEG: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_MPEG,
    AAC: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_AAC,
    APTX: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_APTX,
    APTX_HD: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_APTX_HD,
    LDAC: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_LDAC,
    APTX_LL: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_APTX_LL,
    APTX_LL_DUPLEX: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_APTX_LL_DUPLEX,
    FASTSTREAM: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_FASTSTREAM,
    FASTSTREAM_DUPLEX: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_FASTSTREAM_DUPLEX,
    CVSD: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_CVSD,
    MSBC: spa_sys::SPA_BLUETOOTH_AUDIO_CODEC_MSBC,
);

enum_wrapper!(
    AudioChannel,
    spa_sys::spa_audio_channel,
    UNKNOWN: spa_sys::SPA_AUDIO_CHANNEL_UNKNOWN,
    NA: spa_sys::SPA_AUDIO_CHANNEL_NA,
    MONO: spa_sys::SPA_AUDIO_CHANNEL_MONO,
    FL: spa_sys::SPA_AUDIO_CHANNEL_FL,
    FR: spa_sys::SPA_AUDIO_CHANNEL_FR,
    FC: spa_sys::SPA_AUDIO_CHANNEL_FC,
    LFE: spa_sys::SPA_AUDIO_CHANNEL_LFE,
    SL: spa_sys::SPA_AUDIO_CHANNEL_SL,
    SR: spa_sys::SPA_AUDIO_CHANNEL_SR,
    FLC: spa_sys::SPA_AUDIO_CHANNEL_FLC,
    FRC: spa_sys::SPA_AUDIO_CHANNEL_FRC,
    RC: spa_sys::SPA_AUDIO_CHANNEL_RC,
    RL: spa_sys::SPA_AUDIO_CHANNEL_RL,
    RR: spa_sys::SPA_AUDIO_CHANNEL_RR,
    TC: spa_sys::SPA_AUDIO_CHANNEL_TC,
    TFL: spa_sys::SPA_AUDIO_CHANNEL_TFL,
    TFC: spa_sys::SPA_AUDIO_CHANNEL_TFC,
    TFR: spa_sys::SPA_AUDIO_CHANNEL_TFR,
    TRL: spa_sys::SPA_AUDIO_CHANNEL_TRL,
    TRC: spa_sys::SPA_AUDIO_CHANNEL_TRC,
    TRR: spa_sys::SPA_AUDIO_CHANNEL_TRR,
    RLC: spa_sys::SPA_AUDIO_CHANNEL_RLC,
    RRC: spa_sys::SPA_AUDIO_CHANNEL_RRC,
    FLW: spa_sys::SPA_AUDIO_CHANNEL_FLW,
    FRW: spa_sys::SPA_AUDIO_CHANNEL_FRW,
    LFE2: spa_sys::SPA_AUDIO_CHANNEL_LFE2,
    FLH: spa_sys::SPA_AUDIO_CHANNEL_FLH,
    FCH: spa_sys::SPA_AUDIO_CHANNEL_FCH,
    FRH: spa_sys::SPA_AUDIO_CHANNEL_FRH,
    TFLC: spa_sys::SPA_AUDIO_CHANNEL_TFLC,
    TFRC: spa_sys::SPA_AUDIO_CHANNEL_TFRC,
    TSL: spa_sys::SPA_AUDIO_CHANNEL_TSL,
    TSR: spa_sys::SPA_AUDIO_CHANNEL_TSR,
    LLFE: spa_sys::SPA_AUDIO_CHANNEL_LLFE,
    RLFE: spa_sys::SPA_AUDIO_CHANNEL_RLFE,
    BC: spa_sys::SPA_AUDIO_CHANNEL_BC,
    BLC: spa_sys::SPA_AUDIO_CHANNEL_BLC,
    BRC: spa_sys::SPA_AUDIO_CHANNEL_BRC,
    START_AUX: spa_sys::SPA_AUDIO_CHANNEL_START_Aux,
    AUX0: spa_sys::SPA_AUDIO_CHANNEL_AUX0,
    AUX1: spa_sys::SPA_AUDIO_CHANNEL_AUX1,
    AUX2: spa_sys::SPA_AUDIO_CHANNEL_AUX2,
    AUX3: spa_sys::SPA_AUDIO_CHANNEL_AUX3,
    AUX4: spa_sys::SPA_AUDIO_CHANNEL_AUX4,
    AUX5: spa_sys::SPA_AUDIO_CHANNEL_AUX5,
    AUX6: spa_sys::SPA_AUDIO_CHANNEL_AUX6,
    AUX7: spa_sys::SPA_AUDIO_CHANNEL_AUX7,
    AUX8: spa_sys::SPA_AUDIO_CHANNEL_AUX8,
    AUX9: spa_sys::SPA_AUDIO_CHANNEL_AUX9,
    AUX10: spa_sys::SPA_AUDIO_CHANNEL_AUX10,
    AUX11: spa_sys::SPA_AUDIO_CHANNEL_AUX11,
    AUX12: spa_sys::SPA_AUDIO_CHANNEL_AUX12,
    AUX13: spa_sys::SPA_AUDIO_CHANNEL_AUX13,
    AUX14: spa_sys::SPA_AUDIO_CHANNEL_AUX14,
    AUX15: spa_sys::SPA_AUDIO_CHANNEL_AUX15,
    AUX16: spa_sys::SPA_AUDIO_CHANNEL_AUX16,
    AUX17: spa_sys::SPA_AUDIO_CHANNEL_AUX17,
    AUX18: spa_sys::SPA_AUDIO_CHANNEL_AUX18,
    AUX19: spa_sys::SPA_AUDIO_CHANNEL_AUX19,
    AUX20: spa_sys::SPA_AUDIO_CHANNEL_AUX20,
    AUX21: spa_sys::SPA_AUDIO_CHANNEL_AUX21,
    AUX22: spa_sys::SPA_AUDIO_CHANNEL_AUX22,
    AUX23: spa_sys::SPA_AUDIO_CHANNEL_AUX23,
    AUX24: spa_sys::SPA_AUDIO_CHANNEL_AUX24,
    AUX25: spa_sys::SPA_AUDIO_CHANNEL_AUX25,
    AUX26: spa_sys::SPA_AUDIO_CHANNEL_AUX26,
    AUX27: spa_sys::SPA_AUDIO_CHANNEL_AUX27,
    AUX28: spa_sys::SPA_AUDIO_CHANNEL_AUX28,
    AUX29: spa_sys::SPA_AUDIO_CHANNEL_AUX29,
    AUX30: spa_sys::SPA_AUDIO_CHANNEL_AUX30,
    AUX31: spa_sys::SPA_AUDIO_CHANNEL_AUX31,
    AUX32: spa_sys::SPA_AUDIO_CHANNEL_AUX32,
    AUX33: spa_sys::SPA_AUDIO_CHANNEL_AUX33,
    AUX34: spa_sys::SPA_AUDIO_CHANNEL_AUX34,
    AUX35: spa_sys::SPA_AUDIO_CHANNEL_AUX35,
    AUX36: spa_sys::SPA_AUDIO_CHANNEL_AUX36,
    AUX37: spa_sys::SPA_AUDIO_CHANNEL_AUX37,
    AUX38: spa_sys::SPA_AUDIO_CHANNEL_AUX38,
    AUX39: spa_sys::SPA_AUDIO_CHANNEL_AUX39,
    AUX40: spa_sys::SPA_AUDIO_CHANNEL_AUX40,
    AUX41: spa_sys::SPA_AUDIO_CHANNEL_AUX41,
    AUX42: spa_sys::SPA_AUDIO_CHANNEL_AUX42,
    AUX43: spa_sys::SPA_AUDIO_CHANNEL_AUX43,
    AUX44: spa_sys::SPA_AUDIO_CHANNEL_AUX44,
    AUX45: spa_sys::SPA_AUDIO_CHANNEL_AUX45,
    AUX46: spa_sys::SPA_AUDIO_CHANNEL_AUX46,
    AUX47: spa_sys::SPA_AUDIO_CHANNEL_AUX47,
    AUX48: spa_sys::SPA_AUDIO_CHANNEL_AUX48,
    AUX49: spa_sys::SPA_AUDIO_CHANNEL_AUX49,
    AUX50: spa_sys::SPA_AUDIO_CHANNEL_AUX50,
    AUX51: spa_sys::SPA_AUDIO_CHANNEL_AUX51,
    AUX52: spa_sys::SPA_AUDIO_CHANNEL_AUX52,
    AUX53: spa_sys::SPA_AUDIO_CHANNEL_AUX53,
    AUX54: spa_sys::SPA_AUDIO_CHANNEL_AUX54,
    AUX55: spa_sys::SPA_AUDIO_CHANNEL_AUX55,
    AUX56: spa_sys::SPA_AUDIO_CHANNEL_AUX56,
    AUX57: spa_sys::SPA_AUDIO_CHANNEL_AUX57,
    AUX58: spa_sys::SPA_AUDIO_CHANNEL_AUX58,
    AUX59: spa_sys::SPA_AUDIO_CHANNEL_AUX59,
    AUX60: spa_sys::SPA_AUDIO_CHANNEL_AUX60,
    AUX61: spa_sys::SPA_AUDIO_CHANNEL_AUX61,
    AUX62: spa_sys::SPA_AUDIO_CHANNEL_AUX62,
    AUX63: spa_sys::SPA_AUDIO_CHANNEL_AUX63,
    LAST_AUX: spa_sys::SPA_AUDIO_CHANNEL_LAST_Aux,
    START_CUSTOM: spa_sys::SPA_AUDIO_CHANNEL_START_Custom,
);

enum_wrapper!(
    AudioIec958Codec,
    spa_sys::spa_audio_iec958_codec,
    UNKNOWN: spa_sys::SPA_AUDIO_IEC958_CODEC_UNKNOWN,
    PCM: spa_sys::SPA_AUDIO_IEC958_CODEC_PCM,
    DTS: spa_sys::SPA_AUDIO_IEC958_CODEC_DTS,
    AC3: spa_sys::SPA_AUDIO_IEC958_CODEC_AC3,
    MPEG: spa_sys::SPA_AUDIO_IEC958_CODEC_MPEG,
    MPEG2_AAC: spa_sys::SPA_AUDIO_IEC958_CODEC_MPEG2_AAC,
    EAC3: spa_sys::SPA_AUDIO_IEC958_CODEC_EAC3,
    TRUEHD: spa_sys::SPA_AUDIO_IEC958_CODEC_TRUEHD,
    DTSHD: spa_sys::SPA_AUDIO_IEC958_CODEC_DTSHD,
);
