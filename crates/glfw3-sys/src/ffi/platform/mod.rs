#[cfg(any(target_os = "macos", doc))]
pub mod cocoa;

#[cfg(any(target_os = "windows", doc))]
pub mod win32;

#[cfg(any(
    all(
        any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"),
        feature = "wayland"
    ),
    doc
))]
pub mod wayland;

#[cfg(any(
    all(
        any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"),
        not(feature = "wayland")
    ),
    doc
))]
pub mod x11;
