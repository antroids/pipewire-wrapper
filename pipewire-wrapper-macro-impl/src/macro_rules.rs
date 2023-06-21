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
        #[derive(pipewire_proc_macro::RawWrapper, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct $name {
            #[raw]
            raw: $repr_type,
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

#[macro_export]
macro_rules! spa_interface_call {
    ($self:ident, $method:ident:$version:expr, $($arg:expr),*) => {{
        pipewire_macro_impl::spa_interface_call_impl!($self, $method:$version, $($arg),*)
    }};

    ($self:ident, $method:ident, $($arg:expr),*) => {{
        pipewire_macro_impl::spa_interface_call_impl!($self, $method:0u32, $($arg),*)
    }};

    ($self:ident, $method:ident) => {{
        spa_interface_call!($self, $method,)
    }};
}

#[macro_export]
macro_rules! spa_interface_call_impl {
    ($self:ident, $method:ident:$version:expr, $($arg:expr),*) => {{
        use pipewire_proc_macro::spa_interface;
        use crate::error::Error;
        use crate::wrapper::SpaInterface;

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
                Err(Error::MethodCallOnNull())
            }
        } else {
            Err(Error::VersionMismatch($version, $self.spa_interface().version()))
        }
    }};
}
