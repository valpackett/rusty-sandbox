use libc;
use libc::c_int;
use std::io;
use std::path::Path;
use std::fs::File;
use std::ffi::CString;
use std::os::unix::raw;
use std::os::unix::io::{RawFd, FromRawFd};
use ffi;

fn path_to_c<P: AsRef<Path>>(path: P) -> CString {
    CString::new(path.as_ref().as_os_str().to_str().unwrap()).unwrap()
}

pub struct OpenOptions {
    dir_fd: RawFd,
    flags: c_int,
    read: bool,
    write: bool,
    mode: raw::mode_t,
}

impl OpenOptions {
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        let flags = self.flags | match (self.read, self.write) {
            (true, true) => libc::O_RDWR,
            (false, true) => libc::O_WRONLY,
            (true, false) | (false, false) => libc::O_RDONLY,
        };
        let fd = unsafe { ffi::openat(self.dir_fd, path_to_c(path).as_ptr(), flags, self.mode as c_int) };
        if fd > 0 {
            Ok(unsafe { File::from_raw_fd(fd) })
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "openat() failed"))
        }
    }

    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.read = read; self
    }

    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.write = write; self
    }

    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.flag(libc::O_APPEND, append); self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        self.flag(libc::O_TRUNC, truncate); self
    }

    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.flag(libc::O_CREAT, create); self
    }

    pub fn mode(&mut self, mode: raw::mode_t) -> &mut OpenOptions {
        self.mode = mode; self
    }

    fn flag(&mut self, bit: c_int, on: bool) {
        if on {
            self.flags |= bit;
        } else {
            self.flags &= !bit;
        }
    }
}

#[derive(Clone)]
pub struct Directory {
    fd: RawFd,
}

impl Directory {
    pub fn new<P: AsRef<Path>>(path: P) -> Option<Directory> {
        let path = path.as_ref();
        if path.is_dir() {
            Some(Directory {
                fd: unsafe { libc::open(path_to_c(path).as_ptr(), libc::O_CLOEXEC) }
            })
        } else {
            None
        }
    }

    pub fn open_options(&self) -> OpenOptions {
        OpenOptions {
            dir_fd: self.fd,
            flags: libc::O_CLOEXEC,
            read: false,
            write: false,
            mode: 0o666,
        }
    }
}
