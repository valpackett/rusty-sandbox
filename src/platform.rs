use ffi;
use super::fs;
#[cfg(target_os = "macos")] use std::io::Write;
#[cfg(target_os = "macos")] use std::ffi::CString;
#[cfg(target_os = "macos")] use std::ptr;
// #[cfg(target_os = "macos")] use std::str;

#[cfg(target_os = "freebsd")]
pub fn enter_sandbox<'a>(_: Box<Iterator<Item=&'a fs::Directory> + 'a>) -> bool {
    unsafe { ffi::cap_enter() == 0 }
}

#[cfg(target_os = "macos")]
pub fn enter_sandbox<'a>(allowed_dirs: Box<Iterator<Item=&'a fs::Directory> + 'a>) -> bool {
    let mut profile = Vec::new();
    profile.write_all(b"(version 1)\n").unwrap();
    profile.write_all(b"(deny default)\n").unwrap();
    for dir in allowed_dirs {
        let path = dir.path.to_string_lossy().replace("\"", "\\\"");
        profile.write_all(format!("(allow file-write* file-read* (subpath \"{}\"))\n", path).as_bytes()).unwrap();
    }
    profile.write_all(b"(import \"bsd.sb\")\n").unwrap();
    // println!("{}", str::from_utf8(&*profile).unwrap());
    let profile = CString::new(profile).unwrap();
    let mut err = ptr::null_mut();
    unsafe { ffi::sandbox_init(profile.as_ptr(), 0, &mut err) == 0 }
}

#[cfg(not(any(target_os = "freebsd", target_os = "macos")))]
pub fn enter_sandbox<'a>(_: Box<Iterator<Item=&'a fs::Directory> + 'a>) -> bool {
    false
}
