use core::ptr;
use std::rc::Rc;

use glfw3_sys as sys;

use crate::{Error, Glfw, Terminate};

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub(crate) usize);

impl WindowId {
    pub fn window_ptr(self) -> *const sys::GLFWwindow {
        self.0 as *const _
    }

    pub fn window_mut_ptr(self) -> *mut sys::GLFWwindow {
        self.0 as *mut _
    }
}

pub struct Window {
    pub(crate) window_ptr: *mut sys::GLFWwindow,
    pub(crate) _terminate: Rc<Terminate>,
}

impl Window {
    pub fn window_id(&self) -> WindowId {
        WindowId(self.window_ptr as usize)
    }

    pub fn make_context_current(&self) -> Result<(), Error> {
        unsafe {
            sys::glfwMakeContextCurrent(self.window_ptr);
            Glfw::get_error()
        }
    }

    pub fn swap_buffers(&self) -> Result<(), Error> {
        unsafe {
            sys::glfwSwapBuffers(self.window_ptr);
            Glfw::get_error()
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if self.window_ptr != ptr::null_mut() {
                sys::glfwDestroyWindow(self.window_ptr);
                if let Some(err) = Glfw::get_error().err() {
                    log::warn!("glfwDestroyWindow failed: {:?}", err);
                }
            }
        }
    }
}
