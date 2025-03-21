use core::{ffi, mem, ptr};
use glfw3_sys::{self as sys, GLFWmonitor};
use parking_lot::ReentrantMutex;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

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
    let app_state = &*(sys::glfwGetWindowUserPointer(window) as *const AppState);
    app_state.wait_for_refresh.store(false, Ordering::SeqCst);
}

unsafe extern "C" fn framebuffer_size_callback(
    window: *mut sys::GLFWwindow,
    _width: i32,
    _height: i32,
) {
    let app_state = &*(sys::glfwGetWindowUserPointer(window) as *const AppState);
    let _frame_guard = app_state.frame_lock.lock();
    app_state.wait_for_refresh.store(true, Ordering::SeqCst);
}

unsafe extern "C" fn key_callback(
    window: *mut sys::GLFWwindow,
    key: i32,
    _: i32,
    action: i32,
    _: i32,
) {
    let app_state = &*(sys::glfwGetWindowUserPointer(window) as *const AppState);
    match (key, action) {
        (sys::GLFW_KEY_F, sys::GLFW_RELEASE) => {
            let _frame_guard = app_state.frame_lock.lock();
            app_state.wait_for_refresh.store(true, Ordering::SeqCst);
            let fullscreen = app_state.fullscreen.fetch_not(Ordering::SeqCst);
            if !fullscreen {
                let monitor = sys::glfwGetPrimaryMonitor();
                let mode = select_video_mode(monitor);
                sys::glfwSetWindowMonitor(
                    window,
                    monitor,
                    0,
                    0,
                    mode.width,
                    mode.height,
                    mode.refreshRate,
                );
            } else {
                sys::glfwSetWindowMonitor(
                    window,
                    ptr::null_mut(),
                    200,
                    200,
                    800,
                    600,
                    sys::GLFW_DONT_CARE,
                );
            }
        }
        _ => {}
    }
}

/// Pick a video mode that is most likely to ignore the camera housing if it exists.
unsafe fn select_video_mode(monitor: *mut GLFWmonitor) -> sys::GLFWvidmode {
    let mut count = 0;
    let raw_video_modes = sys::glfwGetVideoModes(monitor, &mut count);
    let mut video_modes = Vec::with_capacity(count as usize);
    for i in 0..count as isize {
        let video_mode = *raw_video_modes.offset(i);
        video_modes.push(video_mode);
    }
    let max_refresh_rate = video_modes
        .iter()
        .fold(0, |refresh_rate, mode| refresh_rate.max(mode.refreshRate));
    let min_aspect_ratio = video_modes.iter().fold(f32::MAX, |ratio, mode| {
        ratio.min(mode.width as f32 / mode.height as f32)
    });
    video_modes
        .iter()
        .filter(|mode| {
            let aspect_ratio = mode.width as f32 / mode.height as f32;
            max_refresh_rate == mode.refreshRate && min_aspect_ratio == aspect_ratio
        })
        .next()
        .copied()
        .expect("Failed to select video mode")
}

#[derive(Default)]
struct AppState {
    frame_lock: ReentrantMutex<()>,
    wait_for_refresh: AtomicBool,
    fullscreen: AtomicBool,
}

unsafe fn draw(window: *mut sys::GLFWwindow, gl: &Gl) {
    let app_state = &*(sys::glfwGetWindowUserPointer(window) as *const AppState);
    let _frame_guard = app_state.frame_lock.lock();
    if !app_state.wait_for_refresh.load(Ordering::SeqCst) {
        let time: f32 = 2.0 * sys::glfwGetTime() as f32;
        let red = time.sin();
        let green = time.cos();
        let blue = 1.0 - red;
        let alpha = 1.0;
        gl.clear_color(red, green, blue, alpha);
        gl.clear(GL_COLOR_BUFFER_BIT);
        sys::glfwSwapBuffers(window);
    } else {
        thread::sleep(Duration::from_millis(16));
    }
}

struct WindowPtr(*mut sys::GLFWwindow);
unsafe impl Send for WindowPtr {}
unsafe impl Sync for WindowPtr {}

impl WindowPtr {
    fn get(&self) -> *mut sys::GLFWwindow {
        self.0
    }
}

fn main() {
    unsafe {
        sys::glfwSetErrorCallback(Some(error_callback));

        let result = sys::glfwInit();
        assert_eq!(sys::GLFW_TRUE, result, "Failed to initialize GLFW");

        let app_state = Box::new(AppState::default());

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
        sys::glfwMakeContextCurrent(ptr::null_mut());

        sys::glfwSetWindowUserPointer(window, &*app_state as *const _ as *mut ffi::c_void);
        sys::glfwSetWindowRefreshCallback(window, Some(refresh_callback));
        sys::glfwSetFramebufferSizeCallback(window, Some(framebuffer_size_callback));
        sys::glfwSetKeyCallback(window, Some(key_callback));

        let window_ptr = WindowPtr(window);

        let join_handle = std::thread::spawn(move || {
            let window = window_ptr.get();
            sys::glfwMakeContextCurrent(window);

            while sys::GLFW_FALSE == sys::glfwWindowShouldClose(window) {
                draw(window, &gl);
            }

            sys::glfwMakeContextCurrent(ptr::null_mut());
        });

        while sys::GLFW_FALSE == sys::glfwWindowShouldClose(window) {
            sys::glfwWaitEvents();
        }

        join_handle.join().expect("Failed to join render thread");

        sys::glfwTerminate();
    }
}
