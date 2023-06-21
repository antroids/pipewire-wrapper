use std::fmt::{Debug, Display, Formatter, Write};

use bitflags::{bitflags, Flags};

const PERMISSION_R_CHAR: char = 'r';
const PERMISSION_W_CHAR: char = 'w';
const PERMISSION_X_CHAR: char = 'x';
const PERMISSION_M_CHAR: char = 'm';
const PERMISSION_NO_CHAR: char = '-';

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(C)]
    pub struct Permissions: u32 {
        const R = pw_sys::PW_PERM_R;
        const W = pw_sys::PW_PERM_W;
        const X = pw_sys::PW_PERM_X;
        const M = pw_sys::PW_PERM_M;
        const RWX = pw_sys::PW_PERM_RWX;
        const RWXM = pw_sys::PW_PERM_RWXM;
        const ALL = pw_sys::PW_PERM_ALL;
    }
}

impl Display for Permissions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            if self.contains(Permissions::R) {
                PERMISSION_R_CHAR
            } else {
                PERMISSION_NO_CHAR
            },
            if self.contains(Permissions::W) {
                PERMISSION_W_CHAR
            } else {
                PERMISSION_NO_CHAR
            },
            if self.contains(Permissions::X) {
                PERMISSION_X_CHAR
            } else {
                PERMISSION_NO_CHAR
            },
            if self.contains(Permissions::M) {
                PERMISSION_M_CHAR
            } else {
                PERMISSION_NO_CHAR
            },
        )
    }
}

#[repr(C)]
pub struct ObjectPermissions {
    pub id: u32,
    pub permissions: Permissions,
}
