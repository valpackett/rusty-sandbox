use ffi;

#[cfg(target_os = "freebsd")]
pub fn enter_sandbox() {
    unsafe { ffi::cap_enter() };
}

#[cfg(not(target_os = "freebsd"))]
pub fn enter_sandbox() {
}
