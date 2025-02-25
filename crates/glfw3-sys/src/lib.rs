#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(rustdoc::broken_intra_doc_links)]

mod ffi;
pub use ffi::*;

#[cfg(test)]
mod tests {
    use crate as sys;

    #[test]
    fn glfw_init_terminate() {
        unsafe {
            let status = sys::glfwInit();
            assert_eq!(sys::GLFW_TRUE, status);
            sys::glfwTerminate();
        }
    }
}
