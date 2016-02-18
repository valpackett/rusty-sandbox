extern crate libc;

mod platform;
pub mod fs;

use std::{io, process};
use std::collections::BTreeMap;
use std::path::Path;
use std::fs::File;
use std::os::unix::io::FromRawFd;

pub struct RunningSandbox {
    pid: libc::c_int,
    pub pipe: File,
}

impl RunningSandbox {
    pub fn wait(self) -> io::Result<RunningSandbox> {
        let mut status = 0;
        unsafe { libc::waitpid(self.pid, &mut status, 0) };
        if status == 0 {
            Ok(self)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Sandboxed child process exited with non-zero status"))
        }
    }
}

pub struct SandboxContext {
    dirs: BTreeMap<String, fs::Directory>,
}

impl SandboxContext {
    pub fn directory(&self, key: &str) -> Option<&fs::Directory> {
        self.dirs.get(key)
    }
}

pub struct Sandbox {
    dirs: BTreeMap<String, fs::Directory>,
}

impl Sandbox {
    pub fn new() -> Sandbox {
        Sandbox {
            dirs: BTreeMap::new()
        }
    }

    pub fn add_directory<P: AsRef<Path>>(&mut self, key: &str, path: P) -> &mut Sandbox {
        if let Some(dir) = fs::Directory::new(path) {
            self.dirs.insert(key.to_owned(), dir);
        }
        self
    }

    pub fn sandbox_this_process(&self) -> SandboxContext {
        platform::enter_sandbox(Box::new(self.dirs.values()));
        self.context()
    }

    pub fn sandboxed_fork<F>(&self, fun: F) -> io::Result<RunningSandbox>
    where F: Fn(&mut SandboxContext, &mut File) -> () {
        let mut fds: [libc::c_int; 2] = [0, 0];
        if unsafe { libc::pipe(&mut fds[0]) } != 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "pipe() failed"))
        }

        let pid = unsafe { libc::fork() };
        if pid < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "fork() failed"))
        } else if pid > 0 { // parent
            unsafe { libc::close(fds[1]) };
            Ok(RunningSandbox {
                pid: pid,
                pipe: unsafe { File::from_raw_fd(fds[0]) },
            })
        } else { // child
            platform::enter_sandbox(Box::new(self.dirs.values()));
            unsafe { libc::close(fds[0]) };
            let mut pipe = unsafe { File::from_raw_fd(fds[1]) };
            let mut ctx = self.context();
            fun(&mut ctx, &mut pipe);
            process::exit(0);
        }
    }

    fn context(&self) -> SandboxContext {
        SandboxContext {
            dirs: self.dirs.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Write};

    #[test]
    fn test_output() {
        let mut buf = Vec::new();
        let mut f = Sandbox::new()
            .sandboxed_fork(|_, pipe| { pipe.write_all(b"Hello World").unwrap() })
            .unwrap().wait().unwrap().pipe;
        f.read_to_end(&mut buf).unwrap();
        assert_eq!(&buf[..], b"Hello World");
    }

    #[test]
    fn test_directory() {
        Sandbox::new()
            .add_directory("temp", "/tmp")
            .sandboxed_fork(|ctx, _| {
                let mut file = ctx.directory("temp").unwrap()
                    .open_options().write(true).create(true)
                    .open("hello_rusty_sandbox").unwrap();
                file.write_all(b"Hello World").unwrap();
            }).unwrap().wait().unwrap();
        let mut buf = Vec::new();
        let mut f = File::open("/tmp/hello_rusty_sandbox").unwrap();
        f.read_to_end(&mut buf).unwrap();
        assert_eq!(&buf[..], b"Hello World");
    }

}
