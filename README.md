# rusty-sandbox [![unlicense](https://img.shields.io/badge/un-license-green.svg?style=flat)](http://unlicense.org)

rusty-sandbox is, obviously, a sandboxing library for Rust that's not [gaol].

It's based on a simple model where you can do the following in the sandbox:

- any normal computation (not I/O)
- I/O operations on *existing* file descriptors (i.e. files and sockets opened before entering the sandbox)
- accepting connections on an *existing* socket (which creates new file descriptors)
- opening files under pre-selected directories *though the Sandbox/SandboxContext API* (which creates new file descriptors)

All other ways of creating new file descriptors will fail in the sandbox!
As well as other potentially dangerous interactions with the outside world such as sysctls, process signals (kill), etc. (platform dependent).

## Underlying technology

rusty-sandbox strongly prefers simple sandboxing facilities that don't require any persistent and/or user-visible records (such as [chroot directories and bind mounts like gaol does on Linux](https://github.com/servo/gaol/blob/9d3753d6f6fb4b4d0f3cb5a29287db44659984fd/platform/linux/namespace.rs)).

- FreeBSD: [Capsicum], the best-supported sandbox that really inspired the design of this library.
- OpenBSD: [pledge], still without the path whitelist thing unfortunately (opening files under select directories DOES NOT WORK), [waiting for 6.4](https://www.openbsd.org/papers/BeckPledgeUnveilBSDCan2018.pdf) for that
- Apple OS X: [Seatbelt]/sandboxd, which Apple kinda wants to deprecate, in favor of App Store-only stuff I think?
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
    Sandbox::new().sandbox_this_process().expect("Couldn't enter sandbox");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    println!("Read file: {}", String::from_utf8_lossy(&buf));
    fs::File::open("README.md").expect("But can't open!");
    // on FreeBSD:
    // thread '<main>' panicked at 'But can't open!: Error { repr: Os { code: 94, message: "Not permitted in capability mode" } }', src/libcore/result.rs:760
}
```

And here's an example for the forked process & allowed directory support.
This silly sandboxed process reads files' first lines:

```rust
extern crate rusty_sandbox;
use std::io::{Write, BufRead, BufReader};
use rusty_sandbox::Sandbox;

fn main() {
    let mut process = Sandbox::new()
        .add_directory("repo", ".")
        .sandboxed_fork(|ctx, socket| {
            // This closure runs in a forked sandboxed process!
            let reader = BufReader::new(socket.try_clone().unwrap());
            for line in reader.lines() {
                let line = line.unwrap();
                if line == "" {
                    return;
                }
                // yes, this is an OpenOptions API!
                let file = ctx.directory("repo").unwrap()
                    .open_options().open(line).unwrap();
                socket.write_all(
                    BufReader::new(file).lines().next().unwrap().unwrap().as_bytes()
                ).unwrap();
                socket.write_all(b"\n").unwrap();
            }
        }).expect("Could not start the sandboxed process");
    process.socket.write_all(b"README.md\n").unwrap();
    let reader = BufReader::new(process.socket.try_clone().unwrap());
    println!("Line from the sandboxed process: {}", reader.lines().next().unwrap().unwrap());
    process.socket.write_all(b"\n").unwrap(); // The "stop" message
    process.wait().expect("Sandboxed process finished unsuccessfully");
}
```

(For a real service, use something like [urpc](https://github.com/kmcallister/urpc)!)

Of course, you can use the directories feature when sandboxing the current process too:

```rust
extern crate rusty_sandbox;
use std::io::Read;
use rusty_sandbox::Sandbox;

fn main() {
    let ctx = Sandbox::new()
        .add_directory("repo", ".")
        .sandbox_this_process()
        .unwrap();
    let mut file = ctx.directory("repo").unwrap()
        .open_options().open("README.md").unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    println!("Read file: {}", String::from_utf8_lossy(&buf));
}
```

Fun fact: [an early prototype of this library](https://gist.github.com/myfreeweb/9c13c245e9f4051236dd) used a shared memory arena for communicating between processes, like [the sandbox for the config parser in sandblast](https://github.com/myfreeweb/sandblast/blob/7dba442af2778ed7ee6a7b303ee709f015ea45fc/config.c#L181). Turns out it's not practical in any language that's higher level than C, because you can't just tell the language's standard library to allocate on an arena.

[gaol]: https://github.com/servo/gaol
[Capsicum]: https://www.cl.cam.ac.uk/research/security/capsicum/
[pledge]: http://www.openbsd.org/papers/hackfest2015-pledge/mgp00001.html
[Seatbelt]: https://www.chromium.org/developers/design-documents/sandbox/osx-sandboxing-design

## Contributing

Please feel free to submit pull requests!

By participating in this project you agree to follow the [Contributor Code of Conduct](http://contributor-covenant.org/version/1/4/).

[The list of contributors is available on GitHub](https://github.com/myfreeweb/rusty-sandbox/graphs/contributors).

## License

This is free and unencumbered software released into the public domain.  
For more information, please refer to the `UNLICENSE` file or [unlicense.org](http://unlicense.org).
