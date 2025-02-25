
# Win32
bindgen \
 --raw-line 'use crate::*;' \
 --raw-line 'mod link;' \
 --allowlist-function "glfwGetWin32.*" \
 --allowlist-function "glfwGetWGL.*" \
 --blocklist-type GLFWwindow \
 --blocklist-type GLFWmonitor \
 --ctypes-prefix "core::ffi" \
 --generate functions,types \
 -o crates/glfw3-sys/src/ffi/platform/win32/mod.rs \
 crates/glfw3-sys/bindgen/platform/win32.h

# Cocoa
bindgen \
 --raw-line 'use crate::*;' \
 --raw-line 'mod link;' \
 --allowlist-function "glfwGetCocoa.*" \
 --allowlist-function "glfwGetNS.*" \
 --blocklist-type GLFWwindow \
 --blocklist-type GLFWmonitor \
 --ctypes-prefix "core::ffi" \
 -o crates/glfw3-sys/src/ffi/platform/cocoa/mod.rs \
 --generate functions,types \
 crates/glfw3-sys/bindgen/platform/cocoa.h

# X11
bindgen \
 --raw-line 'use crate::*;' \
 --raw-line 'mod link;' \
 --allowlist-function "glfwGetX11.*" \
 --allowlist-function "glfwSetX11.*" \
 --allowlist-function "glfwGetGLX.*" \
 --blocklist-type GLFWwindow \
 --blocklist-type GLFWmonitor \
 --ctypes-prefix "core::ffi" \
 -o crates/glfw3-sys/src/ffi/platform/x11/mod.rs \
 --generate functions,types \
 crates/glfw3-sys/bindgen/platform/x11.h

# Wayland
bindgen \
 --raw-line 'use crate::*;' \
 --raw-line 'mod link;' \
 --raw-line 'pub type wl_output = core::ffi::c_void;' \
 --raw-line 'pub type wl_display = core::ffi::c_void;' \
 --raw-line 'pub type wl_surface = core::ffi::c_void;' \
 --allowlist-function "glfwGetWayland.*" \
 --blocklist-type "wl_output" \
 --blocklist-type "wl_display" \
 --blocklist-type "wl_surface" \
 --blocklist-type GLFWwindow \
 --blocklist-type GLFWmonitor \
 --ctypes-prefix "core::ffi" \
 -o crates/glfw3-sys/src/ffi/platform/wayland/mod.rs \
 --generate functions,types \
 crates/glfw3-sys/bindgen/platform/wayland.h

 # Functions and Types
bindgen \
 --raw-line 'mod link;' \
 --allowlist-type "GLFW.*" \
 --allowlist-function "glfw.*" \
 --generate functions,types \
 --ctypes-prefix "core::ffi" \
 -o crates/glfw3-sys/src/ffi/glfw3/mod.rs \
 crates/glfw3-sys/bindgen/glfw3.h

# Constants
bindgen \
 --raw-line '#![allow(overflowing_literals)]' \
 --raw-line 'use core::ffi::c_int;' \
 --ctypes-prefix "core::ffi" \
 --allowlist-var "GLFW.*" \
 --generate vars \
 -o crates/glfw3-sys/src/ffi/constants.rs \
 crates/glfw3-sys/bindgen/glfw3.h

# Force all constants to be signed int
sed -i s/u32/c_int/ crates/glfw3-sys/src/ffi/constants.rs
sed -i s/i32/c_int/ crates/glfw3-sys/src/ffi/constants.rs