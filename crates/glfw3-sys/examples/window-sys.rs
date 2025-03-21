use core::{ffi, mem, ptr};
use glfw3_sys as sys;

pub const GL_COLOR_BUFFER_BIT: i32 = 0x4000;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const ffi::c_char
    };
}

pub struct Gl {
    clear: Option<fn(i32)>,
    clear_color: Option<fn(f32, f32, f32, f32)>,
}

impl Gl {
    pub unsafe fn load() -> Gl {
        Gl {
            clear_color: mem::transmute(sys::glfwGetProcAddress(c_str!("glClearColor"))),
            clear: mem::transmute(sys::glfwGetProcAddress(c_str!("glClear"))),
        }
    }

    pub fn clear(&self, mask: i32) {
        self.clear.expect("glClear")(mask);
    }

    pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.clear_color.expect("glClearColor")(red, green, blue, alpha);
    }
}

unsafe extern "C" fn error_callback(code: ffi::c_int, desc: *const ffi::c_char) {
    let desc = ffi::CStr::from_ptr(desc);
    println!("GLFW Error ({}): {:?}", code, desc);
}

unsafe extern "C" fn refresh_callback(window: *mut sys::GLFWwindow) {
    let gl = &*(sys::glfwGetWindowUserPointer(window) as *mut Gl);
    draw(window, gl);
}

unsafe fn draw(window: *mut sys::GLFWwindow, gl: &Gl) {
    let time: f32 = 2.0 * sys::glfwGetTime() as f32;
    let red = time.sin();
    let green = time.cos();
    let blue = 1.0 - red;
    let alpha = 1.0;
    gl.clear_color(red, green, blue, alpha);
    gl.clear(GL_COLOR_BUFFER_BIT);
    sys::glfwSwapBuffers(window);
}

fn main() {
    unsafe {
        sys::glfwSetErrorCallback(Some(error_callback));

        let result = sys::glfwInit();
        assert_eq!(sys::GLFW_TRUE, result, "Failed to initialize GLFW");

        let window = sys::glfwCreateWindow(
            800,
            600,
            c_str!("GLFW Window"),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_ne!(ptr::null_mut(), window, "Failed to create window");

        sys::glfwMakeContextCurrent(window);

        let gl = Gl::load();

        sys::glfwSetWindowUserPointer(window, &gl as *const Gl as *mut ffi::c_void);
        sys::glfwSetWindowRefreshCallback(window, Some(refresh_callback));

        while sys::GLFW_FALSE == sys::glfwWindowShouldClose(window) {
            sys::glfwPollEvents();
            draw(window, &gl);
        }

        sys::glfwMakeContextCurrent(ptr::null_mut());
        sys::glfwTerminate();
    }
}
