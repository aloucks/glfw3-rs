use core::{ffi, mem};
use glfw3::Error;
use glfw3_sys as sys;

use glfw3::Glfw;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const ffi::c_char
    };
}

pub const GL_COLOR_BUFFER_BIT: i32 = 0x4000;

pub struct Gl {
    clear: Option<fn(i32)>,
    clear_color: Option<fn(f32, f32, f32, f32)>,
}

impl Gl {
    pub fn init() -> Result<Gl, Error> {
        unsafe {
            let gl = Gl {
                clear: mem::transmute(sys::glfwGetProcAddress(c_str!("glClear"))),
                clear_color: mem::transmute(sys::glfwGetProcAddress(c_str!("glClearColor"))),
            };
            Glfw::get_error().map(|_| gl)
        }
    }

    pub fn clear(&self, mask: i32) {
        self.clear.expect("glClear")(mask);
    }

    pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.clear_color.expect("glClearColor")(red, green, blue, alpha);
    }
}
