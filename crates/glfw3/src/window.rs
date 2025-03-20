use core::ptr;
use std::{
    ffi::CStr,
    rc::Rc,
};

use glfw3_sys::{self as sys, GLFW_FALSE, GLFW_TRUE};

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
    pub(crate) _terminate: Option<Rc<Terminate>>,
}

impl Window {
    pub(crate) unsafe fn new(
        window_ptr: *mut sys::GLFWwindow,
        terminate: Option<Rc<Terminate>>,
    ) -> Window {
        Window {
            window_ptr,
            _terminate: terminate,
        }
    }

    fn leak(mut window: Window) {
        window.window_ptr = ptr::null_mut();
    }

    pub fn window_id(&self) -> WindowId {
        WindowId(self.window_ptr as usize)
    }

    pub fn should_close(&self) -> bool {
        unsafe { sys::glfwWindowShouldClose(self.window_ptr) == GLFW_TRUE }
    }

    pub fn set_should_close(&self, value: bool) {
        unsafe {
            let value = if value { GLFW_TRUE } else { GLFW_FALSE };
            sys::glfwSetWindowShouldClose(self.window_ptr, value);
        }
    }

    pub fn title(&self) -> String {
        unsafe {
            CStr::from_ptr(sys::glfwGetWindowTitle(self.window_ptr))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn set_title<S>(&self, title: S)
    where
        S: Into<String>,
    {
        let mut title = title.into();
        title.push(0 as char);
        unsafe {
            sys::glfwSetWindowTitle(self.window_ptr, title.as_ptr() as _);
        }
    }

    // TODO
    pub fn set_window_icon(&self) {}

    pub fn position(&self) -> (i32, i32) {
        let mut xpos = 0;
        let mut ypos = 0;
        unsafe {
            sys::glfwGetWindowPos(self.window_ptr, &mut xpos, &mut ypos);
        }
        (xpos, ypos)
    }

    #[doc(alias = "glfwSetWindowPos")]
    pub fn set_position(&self, xpos: i32, ypos: i32) {
        unsafe {
            sys::glfwSetWindowPos(self.window_ptr, xpos, ypos);
        }
    }

    pub fn size(&self) -> (i32, i32) {
        let mut width = 0;
        let mut height = 0;
        unsafe {
            sys::glfwGetWindowSize(self.window_ptr, &mut width, &mut height);
        }
        (width, height)
    }

    pub fn set_size(&self, width: i32, height: i32) {
        unsafe {
            sys::glfwSetWindowSize(self.window_ptr, width, height);
        }
    }

    #[doc(alias = "glfwSetWindowSizeLimits")]
    pub fn set_size_limits(&self, minwidth: i32, minheight: i32, maxwidth: i32, maxheight: i32) {
        unsafe {
            sys::glfwSetWindowSizeLimits(self.window_ptr, minwidth, minheight, maxwidth, maxheight);
        }
    }

    pub fn set_aspect_ratio(&self, numer: i32, denom: i32) {
        unsafe {
            sys::glfwSetWindowAspectRatio(self.window_ptr, numer, denom);
        }
    }

    pub fn framebuffer_size(&self) -> (i32, i32) {
        let mut width = 0;
        let mut height = 0;
        unsafe {
            sys::glfwGetFramebufferSize(self.window_ptr, &mut width, &mut height);
        }
        (width, height)
    }

    #[doc(alias = "glfwGetWindowFrameSize")]
    pub fn frame_size(&self) -> (i32, i32, i32, i32) {
        let mut left = 0;
        let mut top = 0;
        let mut right = 0;
        let mut bottom = 0;
        unsafe {
            sys::glfwGetWindowFrameSize(
                self.window_ptr,
                &mut left,
                &mut top,
                &mut right,
                &mut bottom,
            );
        }
        (left, top, right, bottom)
    }

    pub fn content_scale(&self) -> (f32, f32) {
        let mut xscale = 0.0;
        let mut yscale = 0.0;
        unsafe {
            sys::glfwGetWindowContentScale(self.window_ptr, &mut xscale, &mut yscale);
        }
        (xscale, yscale)
    }

    pub fn opacity(&self) -> f32 {
        unsafe { sys::glfwGetWindowOpacity(self.window_ptr) }
    }

    pub fn set_opacity(&self, opacity: f32) {
        unsafe { sys::glfwSetWindowOpacity(self.window_ptr, opacity) }
    }

    pub fn iconify(&self) {
        unsafe { sys::glfwIconifyWindow(self.window_ptr) }
    }

    pub fn restore(&self) {
        unsafe { sys::glfwRestoreWindow(self.window_ptr) }
    }

    pub fn maximize(&self) {
        unsafe { sys::glfwMaximizeWindow(self.window_ptr) }
    }

    pub fn show(&self) {
        unsafe { sys::glfwShowWindow(self.window_ptr) }
    }

    pub fn hide(&self) {
        unsafe { sys::glfwHideWindow(self.window_ptr) }
    }

    pub fn focus(&self) {
        unsafe { sys::glfwFocusWindow(self.window_ptr) }
    }

    #[doc(alias = "glfwRequestWindowAttention")]
    pub fn request_attention(&self) {
        unsafe { sys::glfwRequestWindowAttention(self.window_ptr) }
    }

    // glfwGetWindowMonitor

    // glfwSetWindowMonitor

    // glfwGetWindowAttrib

    // glfwSetWindowAttrib

    // glfwSetWindowUserPointer

    // glfwGetWindowUserPointer

    pub fn current_context() -> Option<WindowId> {
        unsafe {
            let window_ptr = sys::glfwGetCurrentContext();
            if !window_ptr.is_null() {
                Some(WindowId(window_ptr as usize))
            } else {
                None
            }
        }
    }

    pub fn is_context_current(&self) -> bool {
        Some(self.window_id()) == Window::current_context()
    }

    pub unsafe fn make_context_current(window_id: Option<WindowId>) -> Result<(), Error> {
        unsafe {
            let window_ptr = window_id
                .map(|id| id.0 as *mut _)
                .unwrap_or(ptr::null_mut());
            sys::glfwMakeContextCurrent(window_ptr);
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
