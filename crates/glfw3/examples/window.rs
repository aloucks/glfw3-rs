use core::time::Duration;

use glfw3::{Glfw, WindowEvent};

mod gl;
use gl::{Gl, GL_COLOR_BUFFER_BIT};

fn main() {
    let glfw = Glfw::init(&[]).expect("GLFW failed to initialize");

    let window = glfw
        .create_window(&[], 800, 600, "GLFW Window", None, None)
        .expect("Failed to create window");

    window
        .make_context_current()
        .expect("glfwMakeContextCurrent");

    let gl = Gl::init().expect("Initialize GL");

    let timeout = Duration::from_secs(1);
    let mut running = true;
    while running {
        let result = glfw.wait_events_timeout(timeout, &mut |_window_id, (_time, event)| {
            println!("{:?}", event);
            match event {
                WindowEvent::Close => {
                    running = false;
                }
                WindowEvent::Refresh => {
                    gl.clear_color(0.2, 0.2, 0.2, 0.2);
                    gl.clear(GL_COLOR_BUFFER_BIT);
                    window.swap_buffers().expect("glfwSwapBuffers");
                }
                _ => {}
            }
            None
        });
        result.expect("glfwWaitEventsTimeout");
    }
}
