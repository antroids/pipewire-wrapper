/*
 * SPDX-License-Identifier: MIT
 */

//! Macro rules
//!

/// Wrap the given `key: value` pairs to struct with constants.
///
/// The following traits will be implemented automatically:
///
/// * [crate::RawWrapper]
/// * [std::fmt::Debug]
///
/// # Examples
/// ```no_run,ignore
/// enum_wrapper!(
///    PositionState,
///    spa_sys::spa_io_position_state,
///    STOPPED: spa_sys::SPA_IO_POSITION_STATE_STOPPED,
///    STARTING: spa_sys::SPA_IO_POSITION_STATE_STARTING,
///    RUNNING: spa_sys::SPA_IO_POSITION_STATE_RUNNING,
/// );
/// ```
///
/// # Arguments
///
/// * `name` - struct name
/// * `repr_type` - enum discriminator type
/// * `enum_variant:enum_value` - enum variant name and discriminator value
/// * `...`
#[macro_export]
macro_rules! enum_wrapper {
    (@add_enum_variant $name: ident, $enum_variant: ident : $enum_value: expr, $($tts:tt)*) => {
        pub const $enum_variant: $name = $name { raw: $enum_value };
        enum_wrapper!(@add_enum_variant $name, $($tts)* ,);
    };
    (@add_enum_variant $name: ident, $enum_variant: ident : $enum_value: path, $($tts:tt)*) => {
        pub const $enum_variant: $name = $name { raw: $enum_value };
        enum_wrapper!(@add_enum_variant $name, $($tts)* ,);
    };
    (@add_enum_variant $name: ident, $(,)*) => {};

    (@add_debug_variant $self:ident, $f:ident, $enum_variant: ident : $enum_value: path, $($tts:tt)*) => {
        if $self.raw == Self::$enum_variant.raw {
            write!($f, stringify!($enum_variant));
            return Ok(())
        };
        enum_wrapper!(@add_debug_variant $self, $f, $($tts)* ,)
    };
    (@add_debug_variant $self:ident, $f:ident, $enum_variant: ident : $enum_value: expr, $($tts:tt)*) => {
        if $self.raw == Self::$enum_variant.raw {
            write!($f, stringify!($enum_variant));
            return Ok(())
        };
        enum_wrapper!(@add_debug_variant $self, $f, $($tts)* ,)
    };
    (@add_debug_variant $self:ident, $f:ident, $(,)*) => {};

    ($name: ident, $repr_type: ty, $($tts:tt)+) => {
        #[derive(pipewire_wrapper_proc_macro::RawWrapper, PartialEq, Eq, Clone, Copy, Hash)]
        #[repr(transparent)]
        pub struct $name {
            #[raw]
            pub(crate) raw: $repr_type,
        }

        impl $name {
            enum_wrapper!(@add_enum_variant $name, $($tts)*);
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                enum_wrapper!(@add_debug_variant self, f, $($tts)*);
                write!(f, "UNKNOWN({:?})", self.raw);
                Ok(())
            }
        }
    };
}

/// Call the method on [SPA Interface](https://docs.pipewire.org/group__spa__interfaces.html).
///
/// [SpaInterface](crate::wrapper::SpaInterface) must be implemented for `Self`.
///
/// # Examples
///
/// ```no_run,ignore
/// pub fn hello(&self, version: u32) -> crate::Result<()> {
///     let result = spa_interface_call!(self, hello, version)?;
///     i32_as_void_result(result)
/// }
/// ```
///
/// ```no_run,ignore
/// pub fn sync(&self, id: u32, seq: i32) -> crate::Result<()> {
///      let result = spa_interface_call!(self, sync, id, seq)?;
///     i32_as_void_result(result)
/// }
/// ```
///
/// # Arguments
///
/// * `self` - self reference to instance with SPA Interface
/// * `method:version` or `method' - method name with optional version
/// * `...` - comma-separated arguments
#[macro_export]
macro_rules! spa_interface_call {
    ($self:ident, $method:ident:$version:expr, $($arg:expr),*) => {{
        spa_interface_call!(@implementation $self, $method:$version, $($arg),*)
    }};

    ($self:ident, $method:ident, $($arg:expr),*) => {{
        spa_interface_call!(@implementation $self, $method:0u32, $($arg),*)
    }};

    ($self:ident, $method:ident) => {{
        spa_interface_call!($self, $method,)
    }};

    (@implementation $self:ident, $method:ident:$version:expr, $($arg:expr),*) => {{
        use pipewire_wrapper_proc_macro::spa_interface;
        use $crate::error::Error;
        use $crate::wrapper::SpaInterface;

        let funcs: *const <Self as SpaInterface>::Methods = $self.spa_interface().cb().funcs();

        if $self.spa_interface().version_min($version) {
            if let Some(funcs) = unsafe { funcs.as_ref() } {
                if let Some(func) = funcs.$method {
                    let result = unsafe { func($self.spa_interface().cb().data(), $($arg),*) };
                    Ok(result)
                } else {
                    Err(Error::MethodNotFound(String::from(stringify!($method))))
                }
            } else {
                Err(Error::MethodCallOnNull)
            }
        } else {
            Err(Error::VersionMismatch($version, $self.spa_interface().version()))
        }
    }};
}

/// Implement `build` method for Events struct builder.
#[macro_export]
macro_rules! events_builder_build {
    ($events_struct:ident<$generic_type:ident>, $events_raw:ident, $($callback_field:ident => $callback:ident,)*) => {
         pub fn build(self) -> Pin<Box<$events_struct<$generic_type>>> {
            events_builder_build!(@function_body_generic self, $events_struct<$generic_type>, $events_raw, $($callback_field => $callback,)*)
         }
    };
    ($events_struct:ident<$lifetime:lifetime, $generic_type:ident>, $events_raw:ident, $($callback_field:ident => $callback:ident,)*) => {
         pub fn build(self) -> Pin<Box<$events_struct<$lifetime, $generic_type>>> {
            events_builder_build!(@function_body_generic self, $events_struct<$generic_type>, $events_raw, $($callback_field => $callback,)*)
         }
    };
    ($events_struct:ident<$lifetime:lifetime>, $events_raw:ident, $($callback_field:ident => $callback:ident,)*) => {
         pub fn build(self) -> Pin<Box<$events_struct<$lifetime>>> {
            events_builder_build!(@function_body self, $events_struct, $events_raw, $($callback_field => $callback,)*)
         }
    };
    ($events_struct:ident, $events_raw:ident, $($callback_field:ident => $callback:ident,)*) => {
         pub fn build(self) -> Pin<Box<$events_struct>> {
            events_builder_build!(@function_body self, $events_struct, $events_raw, $($callback_field => $callback,)*)
         }
    };
    (@function_body $self:ident, $events_struct:ident, $events_raw:ident, $($callback_field:ident => $callback:ident,)*) => {{
            let hook = $crate::spa::interface::Hook::new();
            let raw = $events_raw {
                version: 0,
                $($callback_field: None,
                )*
            };
            let raw = <$events_struct as $crate::wrapper::Wrapper>::RawWrapperType::from_raw(raw);
            let mut pinned_raw = Box::into_pin(Box::new(raw));

            let mut events = Box::into_pin(Box::new($events_struct {
                ref_: NonNull::new(pinned_raw.as_ptr()).unwrap(),
                raw: pinned_raw,
                hook,
                $($callback_field: $self.$callback_field.flatten(),
                )*
            }));

            $(if events.$callback_field.is_some() {
                events.raw.raw.$callback_field = Some(<$events_struct>::$callback);
            })*

            events
    }};
    (@function_body_generic $self:ident, $events_struct:ident<$generic_type:ident>, $events_raw:ident, $($callback_field:ident => $callback:ident,)*) => {{
            let hook = $crate::spa::interface::Hook::new();
            let raw = $events_raw {
                version: 0,
                $($callback_field: None,
                )*
            };
            let raw = <$events_struct<$generic_type> as $crate::wrapper::Wrapper>::RawWrapperType::from_raw(raw);
            let mut pinned_raw = Box::into_pin(Box::new(raw));

            let mut events = Box::into_pin(Box::new($events_struct {
                ref_: NonNull::new(pinned_raw.as_ptr()).unwrap(),
                raw: pinned_raw,
                hook,
                $($callback_field: $self.$callback_field.flatten(),
                )*
            }));

            $(if events.$callback_field.is_some() {
                events.raw.raw.$callback_field = Some(<$events_struct::<$generic_type>>::$callback);
            })*

            events
    }};
}

/// Implement `build` method for Events struct channel builder.
#[macro_export]
macro_rules! events_channel_builder {
    ($struct_name:ident, $($callback_field:ident => $callback:ident,)*) => {
        paste::paste! {
            events_channel_builder!(
                [<$struct_name EventsChannelBuilder>],
                [<$struct_name EventsBuilder>],
                [<$struct_name Events>],
                [<$struct_name EventType>],
                $($callback_field => $callback,)*);
        }
    };
    ($channel_builder:ident, $events_builder:ident, $events_struct:ident, $events_type:ident, $($callback_field:ident => $callback:ident,)*) => {
        pub struct $channel_builder<L: Loop> {
            events_builder: $events_builder,
            sender: Sender<$events_type, L>,
            receiver: Receiver<$events_type, L>,
        }

        impl<L: Loop> $channel_builder<L> {
            $(pub fn $callback_field(mut self) -> Self {
                self.events_builder = self
                    .events_builder
                    .$callback_field(Self::$callback(self.sender.clone()));
                self
            })*

            pub fn build_loop_channel(self) -> (Pin<Box<$events_struct>>, Receiver<$events_type, L>) {
                let events = self.events_builder.build();
                (events, self.receiver)
            }

            pub fn build_channel(self) -> (Pin<Box<$events_struct>>, mpsc::Receiver<$events_type>) {
                let (sender, receiver) = self.build_loop_channel();
                (sender, receiver.into_receiver())
            }
        }

        impl<L: Loop> Default for $channel_builder<L> {
            fn default() -> Self {
                let (sender, receiver) = loop_::channel::LoopChannel::channel();
                Self {
                    events_builder: Default::default(),
                    sender,
                    receiver,
                }
            }
        }
    };
}
