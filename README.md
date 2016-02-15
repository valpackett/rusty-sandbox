# rusty-sandbox [![unlicense](https://img.shields.io/badge/un-license-green.svg?style=flat)](http://unlicense.org)


rusty-sandbox is, obviously, a sandboxing library for Rust (that's not [gaol]).

It's based on a simple model where you can do the following in the sandbox:

- any normal non-IO stuff
- I/O operations on *existing* file descriptors (i.e. files and sockets opened before entering the sandbox)
- accepting connections on an *existing* socket, creating new file descriptors
- opening files under pre-selected directories *though the Sandbox/SandboxContext API*, creating new file descriptors

All other ways of creating new file descriptors will fail in the sandbox!

## Underlying technology

rusty-sandbox strongly prefers simple sandboxing facilities that don't require any persistent and/or user-visible records (such as [chroot directories and bind mounts like gaol does on Linux](https://github.com/servo/gaol/blob/9d3753d6f6fb4b4d0f3cb5a29287db44659984fd/platform/linux/namespace.rs)).

- FreeBSD: [Capsicum], the first supported sandbox.
- OpenBSD: **TODO** [pledge]
- Apple OS X: **TODO** [Seatbelt]
- Linux: **TODO** oh fuck. This is going to involve seccomp-bpf. Unfortunately, the openat O_BENEATH behavior proposed on [capsicum-linux](http://capsicum-linux.org) hasn't been accepted into the Linux kernel!

## Usage

You can sandbox the current process:

```rust
extern crate rusty_sandbox;
use std::fs;
use std::io::Read;
use rusty_sandbox::Sandbox;

fn main() {
    let mut file = fs::File::open("README.md").unwrap();
    Sandbox::new().sandbox_this_process();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    println!("Read file: {}", String::from_utf8_lossy(&buf));
    fs::File::open("README.md").expect("But can't open!");
    // on FreeBSD:
    // thread '<main>' panicked at 'But can't open!: Error { repr: Os { code: 94, message: "Not permitted in capability mode" } }', src/libcore/result.rs:760
}
```

And here's a forked process reading files from allowed directories and communicating with the parent process:

```rust
extern crate rusty_sandbox;
use std::io::{self, Read, Write};
use rusty_sandbox::Sandbox;

fn main() {
    let mut pipe = Sandbox::new()
        .add_directory("repo", ".")
        .sandboxed_fork(|ctx, pipe| {
            // This closure runs in a forked sandboxed process!
            // Let's open a file under the "repo" directory
            // and write it to the pipe that communicates
            // with the parent process...
            let mut file = ctx.directory("repo").unwrap()
                .open_options().open("README.md").unwrap();
                // yes, this is an OpenOptions API!
            io::copy(&mut file, pipe);
            // In a real program, you can do any RPC you want
            // over this pipe. Just don't .wait() early.
        }).expect("Could not start the sandboxed process")
        .wait().expect("Sandboxed process finished unsuccessfully")
        .pipe;
    let mut buf = Vec::new();
    pipe.read_to_end(&mut buf).unwrap();
    println!("From the sandboxed process: {}", String::from_utf8_lossy(&buf));
}
```

[gaol]: https://github.com/servo/gaol
[Capsicum]: https://www.cl.cam.ac.uk/research/security/capsicum/
[pledge]: http://www.openbsd.org/papers/hackfest2015-pledge/mgp00001.html
[Seatbelt]: https://www.chromium.org/developers/design-documents/sandbox/osx-sandboxing-design
