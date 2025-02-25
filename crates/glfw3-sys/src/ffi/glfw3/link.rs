#[cfg(feature = "vendored")]
#[link(name = "glfw3", kind = "static")]
extern "C" {}

#[cfg(not(feature = "vendored"))]
// leaving off `kind = static` allows for the specification of a dynamic library if desired
#[cfg(target_family = "unix")]
#[link(name = "glfw")]
extern "C" {}

#[cfg(not(feature = "vendored"))]
#[cfg(target_family = "windows")]
#[link(name = "glfw3")]
extern "C" {}
