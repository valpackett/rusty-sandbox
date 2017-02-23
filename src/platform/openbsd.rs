use fs;
use std::ptr;
use std::ffi::CString;
use libc::{c_int, c_char};

pub fn enter_sandbox<'a>(_: Box<Iterator<Item=&'a fs::Directory> + 'a>) -> bool {
    unsafe {
        // "The path whitelist feature is not available at this time", still (last 6.0 snapshot from 23 Feb 2017).
        // So I'm not quite sure how the whitelist will behave wrt subdirectories. So not adding the second arg yet.
        let promises = CString::new("stdio").unwrap();
        pledge(promises.as_ptr(), ptr::null()) == 0
    }
}

#[link(name = "c")]
extern {
    fn pledge(promises: *const c_char, paths: *const *const c_char) -> c_int;
}
