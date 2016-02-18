use fs;
use std::os::raw::{c_char, c_int};
use std::io::Write;
use std::ffi::CString;
use std::ptr;
// use std::str;

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
    unsafe { sandbox_init(profile.as_ptr(), 0, &mut err) == 0 }
}

#[link(name = "c")]
extern {
    fn sandbox_init(profile: *const c_char, flags: u64, errorbuf: *mut *mut c_char) -> c_int;
}
