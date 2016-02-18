use fs;
use std::os::raw::c_int;

pub fn enter_sandbox<'a>(_: Box<Iterator<Item=&'a fs::Directory> + 'a>) -> bool {
    unsafe { cap_enter() == 0 }
}

#[link(name = "c")]
extern {
    fn cap_enter() -> c_int;
}
