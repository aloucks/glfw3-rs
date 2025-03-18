#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(rustdoc::broken_intra_doc_links)]

mod ffi;
pub use ffi::*;

#[cfg(test)]
mod tests {
    use crate::{self as sys, GLFW_PLATFORM, GLFW_PLATFORM_NULL};

    #[test]
    fn glfw_init_terminate() {
        unsafe {
            sys::glfwInitHint(GLFW_PLATFORM, GLFW_PLATFORM_NULL);
            let status = sys::glfwInit();
            assert_eq!(sys::GLFW_TRUE, status);
            sys::glfwTerminate();
        }
    }
}
