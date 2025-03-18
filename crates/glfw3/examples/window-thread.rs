use core::{ptr, time::Duration};

use glfw3::{Glfw, WindowEvent};
use glfw3_sys as sys;

mod gl;
use gl::{Gl, GL_COLOR_BUFFER_BIT};

fn main() {
    let glfw = Glfw::init(&[]).expect("GLFW failed to initialize");

    let window = glfw
        .create_window(&[], 800, 600, "GLFW Window", None, None)
        .expect("Failed to create window");

    let window_id = window.window_id();

    let join_handle = std::thread::spawn(move || {
        let window_ptr = window_id.window_mut_ptr();
        unsafe {
            sys::glfwMakeContextCurrent(window_ptr);
        }
        let gl = Gl::init().expect("Initialize GL");
        loop {
            let should_close = unsafe { sys::glfwWindowShouldClose(window_ptr) == sys::GLFW_TRUE };
            if should_close {
                break;
            } else {
                let time = unsafe { sys::glfwGetTime() as f32 } * 2.0;
                gl.clear_color(time.sin(), time.cos(), 1.0 - time.sin(), 1.0);
                gl.clear(GL_COLOR_BUFFER_BIT);
                unsafe {
                    sys::glfwSwapBuffers(window_ptr);
                }
            }
        }
        unsafe {
            sys::glfwMakeContextCurrent(ptr::null_mut());
        }
    });

    let timeout = Duration::from_secs(1);
    let mut running = true;
    while running {
        let result = glfw.wait_events_timeout(timeout, &mut |_window_id, (_time, event)| {
            println!("{:?}", event);
            match event {
                WindowEvent::Close => {
                    running = false;
                }
                _ => {}
            }
            None
        });
        result.expect("glfwWaitEventsTimeout");
    }

    join_handle.join().expect("failed to join render thread");
}
