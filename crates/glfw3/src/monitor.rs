use core::ffi::CStr;
use std::rc::Rc;

use glfw3_sys as sys;

use crate::{Glfw, Terminate, GLFW_NOT_INITIALIZED};

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonitorId(pub(crate) usize);

impl MonitorId {
    pub fn monitor_ptr(self) -> *const sys::GLFWmonitor {
        self.0 as *const _
    }

    pub fn monitor_mut_ptr(self) -> *mut sys::GLFWmonitor {
        self.0 as *mut _
    }
}

pub struct Monitor {
    pub(crate) monitor_ptr: *mut sys::GLFWmonitor,
    pub(crate) _terminate: Rc<Terminate>,
}

impl Monitor {
    pub fn monitor_id(&self) -> MonitorId {
        MonitorId(self.monitor_ptr as usize)
    }

    #[doc(alias = "glfwGetMonitorName")]
    pub fn get_name(&self) -> String {
        unsafe {
            let name_ptr = sys::glfwGetMonitorName(self.monitor_ptr);
            Glfw::get_error().expect(GLFW_NOT_INITIALIZED);
            CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
        }
    }
}
