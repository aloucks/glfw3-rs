use crate::{Action, Key, Modifiers, MouseButton, WindowEvent, WindowId};
use core::ffi::{c_char, c_double, c_float, c_int, c_uint, CStr};
use glfw3_sys as sys;
use std::{cell::RefCell, marker::PhantomData, path::PathBuf};

type CallbackPtr = *mut core::ffi::c_void;

type HandlerFn = fn(
    window_id: WindowId,
    event: (f64, WindowEvent),
    callback_ptr: CallbackPtr,
) -> Option<(f64, WindowEvent)>;

thread_local! {
    static HANDLER: RefCell<Option<(HandlerFn, CallbackPtr)>> = RefCell::new(None);
}

pub struct UnsetHandlerGuard<'a, F> {
    _private: PhantomData<&'a mut F>,
}

impl<'a, F> Drop for UnsetHandlerGuard<'a, F> {
    fn drop(&mut self) {
        HANDLER.with(|ref_cell| {
            *ref_cell.borrow_mut() = None;
        })
    }
}

fn call_handler(window_id: WindowId, event: (f64, WindowEvent)) -> Option<(f64, WindowEvent)> {
    HANDLER.with(|ref_cell| {
        if let Some((handler, callback_ptr)) = *ref_cell.borrow() {
            handler(window_id, event, callback_ptr)
        } else {
            Some(event)
        }
    })
}

pub fn set_handler<'a, F>(callback: &'a mut F) -> UnsetHandlerGuard<'a, F>
where
    F: FnMut(WindowId, (f64, WindowEvent)) -> Option<(f64, WindowEvent)>,
{
    fn handler<F>(
        window_id: WindowId,
        event: (f64, WindowEvent),
        callback_ptr: CallbackPtr,
    ) -> Option<(f64, WindowEvent)>
    where
        F: FnMut(WindowId, (f64, WindowEvent)) -> Option<(f64, WindowEvent)>,
    {
        unsafe {
            let callback: &mut F = &mut *(callback_ptr as *mut F);
            callback(window_id, event)
        }
    }

    HANDLER.with(|ref_cell| {
        let callback_ptr = callback as *mut F as CallbackPtr;
        *ref_cell.borrow_mut() = Some((handler::<F>, callback_ptr));
    });

    UnsetHandlerGuard {
        _private: PhantomData,
    }
}

unsafe extern "C" fn window_refresh_callback(window: *mut sys::GLFWwindow) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::Refresh);
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn key_callback(
    window: *mut sys::GLFWwindow,
    key: c_int,
    scancode: c_int,
    action: c_int,
    mods: c_int,
) {
    let time = sys::glfwGetTime();
    let key = Key::try_from(key);
    let action = Action::try_from(action);
    let mods = Modifiers::from_bits_truncate(mods);
    match (key, action) {
        (Ok(key), Ok(action)) => {
            let event = (time, WindowEvent::Key(key, scancode, action, mods));
            call_handler(WindowId(window as usize), event);
        }
        (Err(key), Ok(_)) => {
            log::warn!("ignoring unidentified key: {}", key);
        }
        (Ok(key), Err(action)) => {
            log::warn!(
                "ignoring unidentified action for key ({:?}): {}",
                key,
                action
            );
        }
        (Err(key), Err(action)) => {
            log::warn!(
                "ignoring unidentified key and action: key = {}, action = {}",
                key,
                action
            );
        }
    }
}

unsafe extern "C" fn char_callback(window: *mut sys::GLFWwindow, codepoint: c_uint) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::Char(codepoint));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn char_mods_callback(
    window: *mut sys::GLFWwindow,
    codepoint: c_uint,
    mods: c_int,
) {
    let time = sys::glfwGetTime();
    let mods = Modifiers::from_bits_truncate(mods);
    #[allow(deprecated)]
    let event = (time, WindowEvent::CharModifiers(codepoint, mods));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn drop_callback(
    window: *mut sys::GLFWwindow,
    count: c_int,
    paths: *mut *const c_char,
) {
    let time = sys::glfwGetTime();
    let mut filepaths = Vec::with_capacity(count as usize);

    for i in 0..count as isize {
        if let Ok(path) = CStr::from_ptr(*paths.offset(i)).to_str() {
            filepaths.push(PathBuf::from(path));
        } else {
            log::warn!("file drop callback received invalid path");
        }
    }

    let event = (time, WindowEvent::FileDrop(filepaths));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn scroll_callback(
    window: *mut sys::GLFWwindow,
    xoffset: c_double,
    yoffset: c_double,
) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::Scroll(xoffset, yoffset));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn cursor_position_callback(
    window: *mut sys::GLFWwindow,
    xpos: c_double,
    ypos: c_double,
) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::CursorPos(xpos, ypos));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn window_position_callback(
    window: *mut sys::GLFWwindow,
    xpos: c_int,
    ypos: c_int,
) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::Pos(xpos, ypos));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn window_size_callback(
    window: *mut sys::GLFWwindow,
    width: c_int,
    height: c_int,
) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::Size(width, height));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn cursor_entered_callback(window: *mut sys::GLFWwindow, entered: c_int) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::CursorEnter(entered != 0));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn mouse_button_callback(
    window: *mut sys::GLFWwindow,
    button: c_int,
    action: c_int,
    mods: c_int,
) {
    let time = sys::glfwGetTime();
    let button = MouseButton::try_from(button);
    let action = Action::try_from(action);
    let mods = Modifiers::from_bits_truncate(mods);
    match (button, action) {
        (Ok(button), Ok(action)) => {
            let event = (time, WindowEvent::MouseButton(button, action, mods));
            call_handler(WindowId(window as usize), event);
        }
        (Err(key), Ok(_)) => {
            log::warn!("ignoring unidentified mouse button: {}", key);
        }
        (Ok(key), Err(action)) => {
            log::warn!(
                "ignoring unidentified action for mouse button ({:?}): {}",
                key,
                action
            );
        }
        (Err(key), Err(action)) => {
            log::warn!(
                "ignoring unknown mouse button and action: key = {}, action = {}",
                key,
                action
            );
        }
    }
}

unsafe extern "C" fn window_close_callback(window: *mut sys::GLFWwindow) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::Close);
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn window_focus_callback(window: *mut sys::GLFWwindow, focused: c_int) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::Focus(focused != 0));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn window_iconify_callback(window: *mut sys::GLFWwindow, iconify: c_int) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::Iconify(iconify != 0));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn framebuffer_size_callback(
    window: *mut sys::GLFWwindow,
    width: c_int,
    height: c_int,
) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::FramebufferSize(width, height));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn content_scale_callback(
    window: *mut sys::GLFWwindow,
    xscale: c_float,
    yscale: c_float,
) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::ContentScale(xscale, yscale));
    call_handler(WindowId(window as usize), event);
}

unsafe extern "C" fn window_maximize_callback(window: *mut sys::GLFWwindow, maximized: c_int) {
    let time = sys::glfwGetTime();
    let event = (time, WindowEvent::Maximize(maximized != 0));
    call_handler(WindowId(window as usize), event);
}

pub unsafe fn set_window_callbacks(window: *mut sys::GLFWwindow) {
    sys::glfwSetKeyCallback(window, Some(key_callback));
    sys::glfwSetCharCallback(window, Some(char_callback));
    sys::glfwSetDropCallback(window, Some(drop_callback));
    sys::glfwSetScrollCallback(window, Some(scroll_callback));
    sys::glfwSetCharModsCallback(window, Some(char_mods_callback));
    sys::glfwSetCursorPosCallback(window, Some(cursor_position_callback));
    sys::glfwSetWindowPosCallback(window, Some(window_position_callback));
    sys::glfwSetWindowSizeCallback(window, Some(window_size_callback));
    sys::glfwSetCursorEnterCallback(window, Some(cursor_entered_callback));
    sys::glfwSetMouseButtonCallback(window, Some(mouse_button_callback));
    sys::glfwSetWindowCloseCallback(window, Some(window_close_callback));
    sys::glfwSetWindowFocusCallback(window, Some(window_focus_callback));
    sys::glfwSetWindowIconifyCallback(window, Some(window_iconify_callback));
    sys::glfwSetWindowRefreshCallback(window, Some(window_refresh_callback));
    sys::glfwSetFramebufferSizeCallback(window, Some(framebuffer_size_callback));
    sys::glfwSetWindowContentScaleCallback(window, Some(content_scale_callback));
    sys::glfwSetWindowMaximizeCallback(window, Some(window_maximize_callback));
}
