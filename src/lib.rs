extern crate libc;
extern crate unix_socket;

mod platform;
pub mod fs;

use std::{io, process, mem};
use std::collections::BTreeMap;
use std::path::Path;
use unix_socket::UnixStream;

pub struct RunningSandbox {
    pid: libc::c_int,
    pub socket: UnixStream,
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

    pub fn sandbox_this_process(&self) -> Result<SandboxContext, ()> {
        if platform::enter_sandbox(Box::new(self.dirs.values())) {
            Ok(self.context())
        } else {
            Err(())
        }
    }

    pub fn sandboxed_fork<F>(&self, fun: F) -> io::Result<RunningSandbox>
    where F: Fn(&mut SandboxContext, &mut UnixStream) -> () {
        let (parent_socket, child_socket) = try!(UnixStream::pair());

        let pid = unsafe { libc::fork() };
        if pid < 0 {
            Err(io::Error::new(io::ErrorKind::Other, "fork() failed"))
        } else if pid > 0 { // parent
            // child_socket is dropped by going out of scope
            Ok(RunningSandbox {
                pid: pid,
                socket: parent_socket,
            })
        } else { // child
            mem::drop(parent_socket);
            platform::enter_sandbox(Box::new(self.dirs.values()));
            let mut socket = child_socket;
            fun(&mut self.context(), &mut socket);
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
    use std::net;
    use std::fs::File;
    use std::io::{Read, Write, BufRead, BufReader};

    #[test]
    fn test_socket() {
        let mut process = Sandbox::new()
            .sandboxed_fork(|_, socket| {
                let msg = BufReader::new(socket.try_clone().unwrap()).lines().next().unwrap().unwrap() + " > from sandbox\n";
                socket.write_all(msg.as_bytes()).unwrap();
                socket.flush().unwrap();
            })
            .unwrap();
        process.socket.write_all(b"from parent\n").unwrap();
        process.socket.flush().unwrap();
        let msg = BufReader::new(process.socket.try_clone().unwrap()).lines().next().unwrap().unwrap();
        assert_eq!(msg, "from parent > from sandbox");
        process.wait().unwrap();
    }

    #[test]
    fn test_preopened_file() {
        let mut f = File::open("UNLICENSE").unwrap();
        let mut process = Sandbox::new()
            .sandboxed_fork(|_, socket| {
                let mut buf = String::new();
                BufReader::new(&f).read_line(&mut buf).unwrap();
                let msg = buf.replace("\n", "") + " from sandbox\n";
                socket.write_all(msg.as_bytes()).unwrap();
                socket.flush().unwrap();
            })
            .unwrap();
        let msg = BufReader::new(process.socket.try_clone().unwrap()).lines().next().unwrap().unwrap();
        assert_eq!(msg, "This is free and unencumbered software released into the public domain. from sandbox");
        process.wait().unwrap();
    }

    #[cfg(not(target_os = "openbsd"))]
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

    #[cfg(not(target_os = "openbsd"))]
    #[test]
    fn test_forbidden_file() {
        let process = Sandbox::new()
            .sandboxed_fork(|_, socket| {
                socket.write_all(match File::open("README.md") {
                    Ok(_) => b"ok",
                    Err(_) => b"err",
                }).unwrap();
                socket.flush().unwrap();
            })
            .unwrap();
        let msg = BufReader::new(process.socket.try_clone().unwrap()).lines().next().unwrap().unwrap();
        assert_eq!(msg, "err");
        process.wait().unwrap();
    }

    #[cfg(not(target_os = "openbsd"))]
    #[test]
    fn test_forbidden_socket() {
        let process = Sandbox::new()
            .sandboxed_fork(|_, socket| {
                socket.write_all(match net::TcpStream::connect("8.8.8.8:53") { // yes it's available on tcp too
                    Ok(_) => b"ok",
                    Err(_) => b"err",
                }).unwrap();
                socket.flush().unwrap();
            })
            .unwrap();
        let msg = BufReader::new(process.socket.try_clone().unwrap()).lines().next().unwrap().unwrap();
        assert_eq!(msg, "err");
        process.wait().unwrap();
    }

}
