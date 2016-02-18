#[cfg(target_os = "freebsd")]
pub mod freebsd;
#[cfg(target_os = "freebsd")]
pub use self::freebsd::*;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::*;

#[cfg(not(any(target_os = "freebsd", target_os = "macos")))]
pub mod unsupported;
#[cfg(not(any(target_os = "freebsd", target_os = "macos")))]
pub use self::unsupported::*;
