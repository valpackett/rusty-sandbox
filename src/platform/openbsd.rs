use fs;
use std::ptr;
use std::ffi::CString;
use libc::{c_int, c_char};

pub fn enter_sandbox<'a>(_: Box<Iterator<Item=&'a fs::Directory> + 'a>) -> bool {
    unsafe {
        // Path whitelist is going to be in 6.4: https://www.openbsd.org/papers/BeckPledgeUnveilBSDCan2018.pdf
        // 'error' makes sandbox violations return ENOSYS instead of exploding the program.
        let promises = CString::new("stdio error").unwrap();
        // Second arg is execpromises, i.e. the pledges for exec()'d processes.
        // We don't want exec to quit the sandbox, of course.
        pledge(promises.as_ptr(), promises.as_ptr()) == 0
    }
}

#[link(name = "c")]
extern {
    fn pledge(promises: *const c_char, paths: *const *const c_char) -> c_int;
}
