use glfw3::{Glfw, Window, WindowEvent};

mod gl;
use gl::{Gl, GL_COLOR_BUFFER_BIT};

fn main() {
    let glfw = Glfw::init(&[]).expect("GLFW failed to initialize");

    let window = glfw
        .create_window(&[], 800, 600, "GLFW Window", None, None)
        .expect("Failed to create window");

    unsafe {
        Window::make_context_current(Some(window.window_id()))
            .expect("Failed to make context current");
    }

    let gl = Gl::init().expect("Failed to initialize GL");

    let mut running = true;
    while running {
        let result = glfw.wait_events(&mut |_window_id, (_time, event)| {
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
