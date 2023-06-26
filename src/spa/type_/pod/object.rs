use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::choice::PodChoiceRef;
use crate::spa::type_::pod::id::PodIdRef;
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::struct_::PodStructRef;
use crate::spa::type_::pod::{
    BasicType, BasicTypePod, PodBoolRef, PodError, PodRef, PodResult, ReadablePod, SizedPod,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
struct PodObjectBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_object_body,
}

impl PodObjectBodyRef {
    pub fn type_(&self) -> Type {
        Type::from_raw(self.raw.type_)
    }

    pub fn id(&self) -> u32 {
        self.raw.id
    }
}

pub struct ObjectPropIterator<'a, T: SizedPod> {
    object: &'a PodObjectRef,
    first_prop_ptr: *const T,
    current_prop_ptr: *const T,
}

impl<'a, T: SizedPod> ObjectPropIterator<'a, T> {
    const ALIGN: usize = 8;

    fn new(object: &'a PodObjectRef) -> Self {
        unsafe {
            let first_prop_ptr: *const T = object.content_ptr();
            Self {
                object,
                first_prop_ptr,
                current_prop_ptr: first_prop_ptr,
            }
        }
    }

    unsafe fn next_ptr(&self, ptr: *const T) -> *const T {
        let size = (&*ptr).size_bytes();
        let next_ptr = (ptr as *const u8).offset(size as isize);
        let aligned = next_ptr
            .offset(next_ptr.align_offset(ObjectPropIterator::<T>::ALIGN) as isize)
            .cast();
        aligned
    }

    unsafe fn inside(&self, ptr: *const T) -> bool {
        let max_offset = self.object.upcast().content_size() as isize;
        let offset = (ptr as *const u8).offset_from(self.first_prop_ptr as *const u8);
        offset < max_offset && (offset + (*ptr).size_bytes() as isize) <= max_offset
    }
}

impl<'a, T: SizedPod + 'a> Iterator for ObjectPropIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let current_prop_ptr = self.current_prop_ptr;
            if self.inside(current_prop_ptr) {
                self.current_prop_ptr = self.next_ptr(current_prop_ptr);
                Some(&*current_prop_ptr)
            } else {
                None
            }
        }
    }
}

impl Debug for PodObjectBodyRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodObjectBodyRef")
            .field("type", &self.type_())
            .field("id", &self.id())
            .finish()
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodObjectRef {
    #[raw]
    raw: spa_sys::spa_pod_object,
}

impl SizedPod for PodObjectRef {
    fn size_bytes(&self) -> usize {
        self.upcast().size_bytes()
    }
}

impl BasicTypePod for PodObjectRef {}

impl Debug for PodObjectRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodObjectRef")
                .field("pod", &self.upcast())
                .field("body_type", &self.body_type())
                .field("body_id", &self.body_id())
                .finish()
        }
    }
}

impl PodObjectRef {
    fn body(&self) -> &PodObjectBodyRef {
        unsafe { PodObjectBodyRef::from_raw_ptr(&self.raw.body) }
    }

    pub fn body_type(&self) -> Type {
        self.body().type_()
    }

    pub fn body_id(&self) -> u32 {
        self.body().id()
    }
}

impl<'a> ReadablePod for &'a PodObjectRef {
    type Value = ObjectType<'a>;

    fn value(&self) -> PodResult<Self::Value> {
        Ok(match self.body_type() {
            Type::OBJECT_PROP_INFO => ObjectType::OBJECT_PROP_INFO(ObjectPropIterator::new(self)),
            //Type::OBJECT_PROPS => ObjectType::OBJECT_PROPS(ObjectPropIterator::new(self)),
            _ => {
                todo!()
            }
        })
    }
}

pub trait PodPropKeyType<'a>
where
    Self: 'a,
    Self: TryFrom<&'a PodPropRef<'a, Self>, Error = PodError>,
    Self: Debug,
{
}

#[repr(transparent)]
pub struct PodPropRef<'a, T: PodPropKeyType<'a>> {
    raw: spa_sys::spa_pod_prop,
    phantom_type: PhantomData<&'a T>,
}

impl<'a, T: PodPropKeyType<'a>> RawWrapper for PodPropRef<'a, T> {
    type CType = spa_sys::spa_pod_prop;

    fn as_raw_ptr(&self) -> *mut Self::CType {
        &self.raw as *const _ as *mut _
    }

    fn as_raw(&self) -> &Self::CType {
        &self.raw
    }

    fn from_raw(raw: Self::CType) -> Self {
        Self {
            raw,
            phantom_type: PhantomData::default(),
        }
    }

    unsafe fn mut_from_raw_ptr<'b>(raw: *mut Self::CType) -> &'b mut Self {
        &mut *(raw as *mut PodPropRef<T>)
    }
}

impl<'a, T: PodPropKeyType<'a>> SizedPod for PodPropRef<'a, T> {
    fn size_bytes(&self) -> usize {
        size_of::<PodPropRef<T>>() + self.pod().content_size() as usize
    }
}

impl<'a, T: PodPropKeyType<'a>> PodPropRef<'a, T> {
    pub fn key(&self) -> u32 {
        self.raw.key
    }

    pub fn flags(&self) -> u32 {
        self.raw.flags
    }

    pub fn pod(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.value) as *const spa_sys::spa_pod) }
    }
}

impl<'a, T: PodPropKeyType<'a>> ReadablePod for &'a PodPropRef<'a, T> {
    type Value = T;

    fn value(&self) -> PodResult<Self::Value> {
        (*self).try_into()
    }
}

impl<'a, T: PodPropKeyType<'a>> Debug for &'a PodPropRef<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodPropRef")
                .field("key", &self.key())
                .field("flags", &self.flags())
                .field("pod", &self.pod())
                .field("value", &self.value())
                .finish()
        }
    }
}

#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum ObjectType<'a> {
    OBJECT_PROP_INFO(ObjectPropIterator<'a, PodPropRef<'a, ObjectPropInfoType<'a>>>) =
        Type::OBJECT_PROP_INFO.raw,
    //OBJECT_PROPS(ObjectPropIterator<'a, PodPropRef>) = Type::OBJECT_PROPS.raw,
    //OBJECT_FORMAT() = Type::OBJECT_FORMAT.raw,
}

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ObjectPropInfoType<'a> {
    ID(&'a PodIdRef<Prop>) = PropInfo::ID.raw,
    NAME(&'a PodStringRef) = PropInfo::NAME.raw,
    TYPE(&'a PodChoiceRef) = PropInfo::TYPE.raw,
    LABELS(&'a PodStructRef) = PropInfo::LABELS.raw,
    CONTAINER(&'a PodIdRef<u32>) = PropInfo::CONTAINER.raw,
    PARAMS(&'a PodBoolRef) = PropInfo::PARAMS.raw,
    DESCRIPTION(&'a PodStringRef) = PropInfo::DESCRIPTION.raw,
}

impl<'a> PodPropKeyType<'a> for ObjectPropInfoType<'a> {}

impl<'a> TryFrom<&'a PodPropRef<'a, ObjectPropInfoType<'a>>> for ObjectPropInfoType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ObjectPropInfoType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match PropInfo::from_raw(value.raw.key) {
                PropInfo::ID => Ok(ObjectPropInfoType::ID(value.pod().cast()?)),
                PropInfo::NAME => Ok(ObjectPropInfoType::NAME(value.pod().cast()?)),
                PropInfo::TYPE => Ok(ObjectPropInfoType::TYPE(value.pod().cast()?)),
                PropInfo::LABELS => Ok(ObjectPropInfoType::LABELS(value.pod().cast()?)),
                PropInfo::CONTAINER => Ok(ObjectPropInfoType::CONTAINER(value.pod().cast()?)),
                PropInfo::PARAMS => Ok(ObjectPropInfoType::PARAMS(value.pod().cast()?)),
                PropInfo::DESCRIPTION => Ok(ObjectPropInfoType::DESCRIPTION(value.pod().cast()?)),
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

enum_wrapper!(
    PropInfo,
    spa_sys::spa_prop_info,
    START: spa_sys::SPA_PROP_INFO_START,
    ID: spa_sys::SPA_PROP_INFO_id,
    NAME: spa_sys::SPA_PROP_INFO_name,
    TYPE: spa_sys::SPA_PROP_INFO_type,
    LABELS: spa_sys::SPA_PROP_INFO_labels,
    CONTAINER: spa_sys::SPA_PROP_INFO_container,
    PARAMS: spa_sys::SPA_PROP_INFO_params,
    DESCRIPTION: spa_sys::SPA_PROP_INFO_description,
);

enum_wrapper!(
    Prop,
    spa_sys::spa_prop,
    _START: spa_sys::SPA_PROP_START,
    UNKNOWN: spa_sys::SPA_PROP_unknown,
    START_DEVICE: spa_sys::SPA_PROP_START_Device,
    DEVICE: spa_sys::SPA_PROP_device,
    DEVICENAME: spa_sys::SPA_PROP_deviceName,
    DEVICEFD: spa_sys::SPA_PROP_deviceFd,
    CARD: spa_sys::SPA_PROP_card,
    CARDNAME: spa_sys::SPA_PROP_cardName,
    MINLATENCY: spa_sys::SPA_PROP_minLatency,
    MAXLATENCY: spa_sys::SPA_PROP_maxLatency,
    PERIODS: spa_sys::SPA_PROP_periods,
    PERIODSIZE: spa_sys::SPA_PROP_periodSize,
    PERIODEVENT: spa_sys::SPA_PROP_periodEvent,
    LIVE: spa_sys::SPA_PROP_live,
    RATE: spa_sys::SPA_PROP_rate,
    QUALITY: spa_sys::SPA_PROP_quality,
    BLUETOOTHAUDIOCODEC: spa_sys::SPA_PROP_bluetoothAudioCodec,
    AUDIO: spa_sys::SPA_PROP_START_Audio,
    WAVETYPE: spa_sys::SPA_PROP_waveType,
    FREQUENCY: spa_sys::SPA_PROP_frequency,
    VOLUME: spa_sys::SPA_PROP_volume,
    MUTE: spa_sys::SPA_PROP_mute,
    PATTERNTYPE: spa_sys::SPA_PROP_patternType,
    DITHERTYPE: spa_sys::SPA_PROP_ditherType,
    TRUNCATE: spa_sys::SPA_PROP_truncate,
    CHANNELVOLUMES: spa_sys::SPA_PROP_channelVolumes,
    VOLUMEBASE: spa_sys::SPA_PROP_volumeBase,
    VOLUMESTEP: spa_sys::SPA_PROP_volumeStep,
    CHANNELMAP: spa_sys::SPA_PROP_channelMap,
    MONITORMUTE: spa_sys::SPA_PROP_monitorMute,
    MONITORVOLUMES: spa_sys::SPA_PROP_monitorVolumes,
    LATENCYOFFSETNSEC: spa_sys::SPA_PROP_latencyOffsetNsec,
    SOFTMUTE: spa_sys::SPA_PROP_softMute,
    SOFTVOLUMES: spa_sys::SPA_PROP_softVolumes,
    IEC958CODECS: spa_sys::SPA_PROP_iec958Codecs,
    VIDEO: spa_sys::SPA_PROP_START_Video,
    BRIGHTNESS: spa_sys::SPA_PROP_brightness,
    CONTRAST: spa_sys::SPA_PROP_contrast,
    SATURATION: spa_sys::SPA_PROP_saturation,
    HUE: spa_sys::SPA_PROP_hue,
    GAMMA: spa_sys::SPA_PROP_gamma,
    EXPOSURE: spa_sys::SPA_PROP_exposure,
    GAIN: spa_sys::SPA_PROP_gain,
    SHARPNESS: spa_sys::SPA_PROP_sharpness,
    START_OTHER: spa_sys::SPA_PROP_START_Other,
    PARAMS: spa_sys::SPA_PROP_params,
    START_CUSTOM: spa_sys::SPA_PROP_START_CUSTOM,
);
