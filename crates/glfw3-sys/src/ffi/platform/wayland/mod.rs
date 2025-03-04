/* automatically generated by rust-bindgen 0.71.1 */

use crate::*;
mod link;
pub type wl_output = core::ffi::c_void;
pub type wl_display = core::ffi::c_void;
pub type wl_surface = core::ffi::c_void;

unsafe extern "C" {
    #[doc = " @brief Returns the `struct wl_display*` used by GLFW.\n\n  @return The `struct wl_display*` used by GLFW, or `NULL` if an\n  [error](@ref error_handling) occurred.\n\n  @errors Possible errors include @ref GLFW_NOT_INITIALIZED and @ref\n  GLFW_PLATFORM_UNAVAILABLE.\n\n  @thread_safety This function may be called from any thread.  Access is not\n  synchronized.\n\n  @since Added in version 3.2.\n\n  @ingroup native"]
    pub fn glfwGetWaylandDisplay() -> *mut wl_display;
}
unsafe extern "C" {
    #[doc = " @brief Returns the `struct wl_output*` of the specified monitor.\n\n  @return The `struct wl_output*` of the specified monitor, or `NULL` if an\n  [error](@ref error_handling) occurred.\n\n  @errors Possible errors include @ref GLFW_NOT_INITIALIZED and @ref\n  GLFW_PLATFORM_UNAVAILABLE.\n\n  @thread_safety This function may be called from any thread.  Access is not\n  synchronized.\n\n  @since Added in version 3.2.\n\n  @ingroup native"]
    pub fn glfwGetWaylandMonitor(monitor: *mut GLFWmonitor) -> *mut wl_output;
}
unsafe extern "C" {
    #[doc = " @brief Returns the main `struct wl_surface*` of the specified window.\n\n  @return The main `struct wl_surface*` of the specified window, or `NULL` if\n  an [error](@ref error_handling) occurred.\n\n  @errors Possible errors include @ref GLFW_NOT_INITIALIZED and @ref\n  GLFW_PLATFORM_UNAVAILABLE.\n\n  @thread_safety This function may be called from any thread.  Access is not\n  synchronized.\n\n  @since Added in version 3.2.\n\n  @ingroup native"]
    pub fn glfwGetWaylandWindow(window: *mut GLFWwindow) -> *mut wl_surface;
}
