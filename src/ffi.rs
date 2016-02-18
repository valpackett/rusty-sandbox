use std::os::raw::{c_char, c_int};

#[link(name = "c")]
extern {
    pub fn openat(dirfd: c_int, path: *const c_char, oflag: c_int, ...) -> c_int;

    #[cfg(target_os = "freebsd")]
    pub fn cap_enter() -> c_int;

    #[cfg(target_os = "macos")]
    pub fn sandbox_init(profile: *const c_char, flags: u64, errorbuf: *mut *mut c_char) -> c_int;
}
