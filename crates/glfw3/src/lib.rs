use core::ffi::{c_int, CStr};
use glfw3_sys::{self as sys};
use std::{
    ffi::CString,
    fmt::Pointer,
    marker::PhantomData,
    mem,
    path::PathBuf,
    ptr,
    rc::Rc,
    sync::{LazyLock, Mutex, MutexGuard, TryLockError},
    time::Duration,
};

mod callbacks;
mod monitor;
mod window;

pub use monitor::*;
pub use window::*;

/// Unwrap errors that are expected to be impossible to happen unless
/// GLFW has not been initialized as described in the function documentation.
///
/// ```text
/// get_error().expect(GLFW_NOT_INITIALIZED);
/// ```
const GLFW_NOT_INITIALIZED: &str = "GLFW has not been initialized";

static INIT: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Debug)]
struct Terminate {
    _init_guard: InitGuard,
    _phantom: PhantomData<*mut ()>,
}

type InitGuard = MutexGuard<'static, ()>;

#[derive(Debug)]
pub struct Glfw {
    terminate: Rc<Terminate>,
}

impl Drop for Terminate {
    fn drop(&mut self) {
        unsafe {
            sys::glfwTerminate();
            if let Some(err) = Glfw::get_error().err() {
                log::warn!("glfwTerminate failed: {:?}", err);
            }
        }
    }
}

#[derive(Debug)]
pub enum InitError<'a> {
    Hint(&'a InitHint, Error),
    Init(Error),
    Poisoned,
}

#[derive(Debug)]
pub enum TryInitError<'a> {
    InitError(InitError<'a>),
    WouldBlock,
}

impl<'a> From<InitError<'a>> for TryInitError<'a> {
    fn from(value: InitError<'a>) -> Self {
        TryInitError::InitError(value)
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Platform {
    Any = sys::GLFW_ANY_PLATFORM,
    Win32 = sys::GLFW_PLATFORM_WIN32,
    Cocoa = sys::GLFW_PLATFORM_COCOA,
    Wayland = sys::GLFW_PLATFORM_WAYLAND,
    X11 = sys::GLFW_PLATFORM_X11,
    Null = sys::GLFW_PLATFORM_NULL,
}

impl TryFrom<i32> for Platform {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value >= sys::GLFW_ANY_PLATFORM && value <= sys::GLFW_PLATFORM_NULL {
            Ok(unsafe { mem::transmute(value) })
        } else {
            Err(value)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WaylandLibDecor {
    Prefer,
    Disable,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AnglePlatformType {
    None = sys::GLFW_ANGLE_PLATFORM_TYPE_NONE,
    OpenGl = sys::GLFW_ANGLE_PLATFORM_TYPE_OPENGL,
    OpenGlEs = sys::GLFW_ANGLE_PLATFORM_TYPE_OPENGLES,
    D3D9 = sys::GLFW_ANGLE_PLATFORM_TYPE_D3D9,
    D3D11 = sys::GLFW_ANGLE_PLATFORM_TYPE_D3D11,
    Vulkan = sys::GLFW_ANGLE_PLATFORM_TYPE_VULKAN,
    Metal = sys::GLFW_ANGLE_PLATFORM_TYPE_METAL,
}

#[derive(Debug)]
pub enum InitHint {
    Platform(Platform),
    JoystickHatButtons(bool),
    CocoaChdirResources(bool),
    CocoaMenubar(bool),
    WaylandLibDecor(WaylandLibDecor),
    X11XcbVulkanSurface(bool),
    AnglePlatformType(AnglePlatformType),
}

impl InitHint {
    pub fn none() -> &'static [InitHint] {
        &[]
    }

    /// https://www.glfw.org/docs/3.4/intro_guide.html#init_hints
    fn default_hints() -> &'static [InitHint] {
        &[
            InitHint::Platform(Platform::Any),
            InitHint::JoystickHatButtons(true),
            InitHint::AnglePlatformType(AnglePlatformType::None),
            InitHint::CocoaChdirResources(true),
            InitHint::CocoaMenubar(true),
            InitHint::WaylandLibDecor(WaylandLibDecor::Prefer),
            InitHint::X11XcbVulkanSurface(true),
        ]
    }
}

#[derive(Debug)]
pub struct Error {
    pub code: i32,
    pub desc: String,
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        &self.desc
    }

    fn cause(&self) -> Option<&dyn core::error::Error> {
        self.source()
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ({})", self.desc, self.code)
    }
}

fn unknown_error() -> Error {
    Error {
        code: -1,
        desc: String::from("Unknown error"),
    }
}

fn initialize<'a>(hints: &'a [InitHint], init_guard: InitGuard) -> Result<Glfw, InitError<'a>> {
    let default_hints = InitHint::default_hints();
    for hint in default_hints.iter().chain(hints.iter()) {
        match hint {
            &InitHint::Platform(platform) => unsafe {
                sys::glfwInitHint(sys::GLFW_PLATFORM, platform as i32);
                Glfw::get_error().map_err(|err| InitError::Hint(hint, err))?;
            },
            _ => {}
        }
    }
    unsafe {
        if sys::GLFW_TRUE == sys::glfwInit() {
            let glfw = Glfw {
                terminate: Rc::new(Terminate {
                    _init_guard: init_guard,
                    _phantom: PhantomData,
                }),
            };
            set_global_callbacks().map_err(|err| InitError::Init(err))?;
            Ok(glfw)
        } else {
            Err(InitError::Init(
                Glfw::get_error().err().unwrap_or_else(unknown_error),
            ))
        }
    }
}

impl Glfw {
    #[doc(alias = "glfwGetError")]
    pub fn get_error() -> Result<(), Error> {
        unsafe {
            let mut desc = ptr::null();
            let code = sys::glfwGetError(&mut desc);
            if sys::GLFW_NO_ERROR != code {
                Err(Error {
                    code,
                    desc: CStr::from_ptr(desc).to_string_lossy().into_owned(),
                })
            } else {
                Ok(())
            }
        }
    }

    #[doc(alias = "glfwInit")]
    #[doc(alias = "glfwInitHint")]
    pub fn init<'a>(hints: &'a [InitHint]) -> Result<Glfw, InitError<'a>> {
        let init_guard = INIT.lock().map_err(|_| InitError::Poisoned)?;
        Ok(initialize(hints, init_guard)?)
    }

    #[doc(alias = "glfwInit")]
    #[doc(alias = "glfwInitHint")]
    pub fn try_init<'a>(hints: &'a [InitHint]) -> Result<Glfw, TryInitError<'a>> {
        let init_guard = INIT.try_lock().map_err(|err| match err {
            TryLockError::Poisoned(_) => TryInitError::InitError(InitError::Poisoned),
            TryLockError::WouldBlock => TryInitError::WouldBlock,
        })?;
        Ok(initialize(hints, init_guard)?)
    }

    #[doc(alias = "glfwPlatformSupported")]
    pub fn platform_supported(platform: Platform) -> bool {
        unsafe { sys::GLFW_TRUE == sys::glfwPlatformSupported(platform as i32) }
    }

    #[doc(alias = "glfwGetVersion")]
    pub fn get_version() -> (i32, i32, i32) {
        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;
        unsafe { sys::glfwGetVersion(&mut major, &mut minor, &mut patch) }
        (major, minor, patch)
    }

    #[doc(alias = "glfwGetPlatform")]
    pub fn get_platform(&self) -> Platform {
        let platform = unsafe { sys::glfwGetPlatform() };
        match platform {
            sys::GLFW_PLATFORM_WIN32 => Platform::Win32,
            sys::GLFW_PLATFORM_COCOA => Platform::Cocoa,
            sys::GLFW_PLATFORM_WAYLAND => Platform::Wayland,
            sys::GLFW_PLATFORM_X11 => Platform::X11,
            sys::GLFW_PLATFORM_NULL => Platform::Null,
            _ => Platform::Any,
        }
    }

    #[doc(alias = "glfwCreateWindow")]
    #[doc(alias = "glfwWindowHint")]
    #[doc(alias = "glfwWindowHintString")]
    pub fn create_window<'a>(
        &self,
        hints: &'a [WindowHint],
        width: i32,
        height: i32,
        title: &str,
        monitor: Option<&Monitor>,
        share: Option<&Window>,
    ) -> Result<Window, CreateWindowError<'a>> {
        unsafe {
            sys::glfwDefaultWindowHints();
            Glfw::get_error().expect(GLFW_NOT_INITIALIZED);
            for hint in hints.iter() {
                match hint {
                    &WindowHint::ClientApi(client_api) => {
                        sys::glfwWindowHint(sys::GLFW_CLIENT_API, client_api as i32);
                        Glfw::get_error().map_err(|err| CreateWindowError::Hint(hint, err))?;
                    }
                    _ => {}
                }
            }
            let title = CString::new(title).expect("Failed to convert title to CString");
            let monitor_ptr = monitor.map(|m| m.monitor_ptr).unwrap_or(ptr::null_mut());
            let share_ptr = share.map(|w| w.window_ptr).unwrap_or(ptr::null_mut());
            let window_ptr =
                sys::glfwCreateWindow(width, height, title.as_ptr(), monitor_ptr, share_ptr);
            Glfw::get_error().map_err(|err| CreateWindowError::CreateWindow(err))?;
            callbacks::set_window_callbacks(window_ptr);
            let terminate = Some(Rc::clone(&self.terminate));
            Ok(Window::new(window_ptr, terminate))
        }
    }

    #[doc(alias = "glfwGetMonitors")]
    pub fn get_monitors(&self) -> Vec<Monitor> {
        unsafe {
            let mut count = 0;
            let monitor_ptrs = sys::glfwGetMonitors(&mut count);
            Glfw::get_error().expect(GLFW_NOT_INITIALIZED);
            let mut monitors = Vec::with_capacity(count as usize);
            for offset in 0..count {
                let monitor_ptr = *monitor_ptrs.offset(offset as isize);
                monitors.push(Monitor {
                    monitor_ptr,
                    _terminate: Rc::clone(&self.terminate),
                })
            }
            monitors
        }
    }

    #[doc(alias = "glfwGetPrimaryMonitor")]
    pub fn get_primary_monitor(&self) -> Option<Monitor> {
        unsafe {
            let monitor_ptr = sys::glfwGetPrimaryMonitor();
            if monitor_ptr.is_null() {
                None
            } else {
                Some(Monitor {
                    monitor_ptr,
                    _terminate: Rc::clone(&self.terminate),
                })
            }
        }
    }

    pub fn poll_events<F>(&self, event_handler: &mut F) -> Result<(), Error>
    where
        F: FnMut(WindowId, (f64, WindowEvent)) -> Option<(f64, WindowEvent)>,
    {
        let _unset_handler_guard = callbacks::set_handler(event_handler);
        unsafe {
            sys::glfwPollEvents();
            Glfw::get_error()?;
        }
        Ok(())
    }

    pub fn wait_events<F>(&self, event_handler: &mut F) -> Result<(), Error>
    where
        F: FnMut(WindowId, (f64, WindowEvent)) -> Option<(f64, WindowEvent)>,
    {
        let _unset_handler_guard = callbacks::set_handler(event_handler);
        unsafe {
            sys::glfwWaitEvents();
            Glfw::get_error()?;
        }
        Ok(())
    }

    pub fn wait_events_timeout<F>(
        &self,
        timeout: Duration,
        event_handler: &mut F,
    ) -> Result<(), Error>
    where
        F: FnMut(WindowId, (f64, WindowEvent)) -> Option<(f64, WindowEvent)>,
    {
        let _unset_handler_guard = callbacks::set_handler(event_handler);
        unsafe {
            sys::glfwWaitEventsTimeout(timeout.as_secs_f64());
            Glfw::get_error()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowHint {
    Resizable(bool),
    Visible(bool),
    Decorated(bool),
    Focused(bool),
    AutoIconify(bool),
    Floating(bool),
    Maximized(bool),
    CenterCursor(bool),
    TransparentFramebuffer(bool),
    FocusOnShow(bool),
    ScaleToMonitor(bool),
    ScaleFramebuffer(bool),
    MousePassthrough(bool),
    PositionX(i32),
    PositionY(i32),
    RedBits(i32),
    GreenBits(i32),
    BlueBits(i32),
    AlphaBits(i32),
    StencilBits(i32),
    AccumRedBits(i32),
    AccumGreenBits(i32),
    AccumBlueBits(i32),
    AccumAlphaBits(i32),
    AuxBuffers(i32),
    Samples(i32),
    RefreshRate(i32),
    Stereo(bool),
    SrgbCapable(bool),
    Doublebuffer(bool),
    ClientApi(ClientApi),
    ContextCreationApi(ContextCreationApi),
    ContextVersionMajor(i32),
    ContextVersionMinor(i32),
    ContextRobustness(ContextRobustness),
    ContextReleaseBehavior(ContextReleaseBehavior),
    // TODO: more
}

impl WindowHint {
    pub fn none() -> &'static [WindowHint] {
        &[]
    }
}

#[derive(Debug)]
pub enum CreateWindowError<'a> {
    Hint(&'a WindowHint, Error),
    CreateWindow(Error),
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientApi {
    OpenGl = sys::GLFW_OPENGL_API,
    OpenGlEs = sys::GLFW_OPENGL_ES_API,
    None = sys::GLFW_NO_API,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextCreationApi {
    Native,
    Egl,
    OsMesa,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRobustness {
    None,
    NoResetNotification,
    LoseContextOnReset,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextReleaseBehavior {
    Any,
    Flush,
    None,
}

#[cfg(test)]
mod tests {
    use crate::*;

    macro_rules! assert_not_impl {
        ($x:ty, $($t:path),+ $(,)*) => {
            const _: fn() -> () = || {
                struct Check<T: ?Sized>(T);
                trait AmbiguousIfImpl<A> { fn some_item() { } }

                impl<T: ?Sized> AmbiguousIfImpl<()> for Check<T> { }
                impl<T: ?Sized $(+ $t)*> AmbiguousIfImpl<u8> for Check<T> { }

                <Check::<$x> as AmbiguousIfImpl<_>>::some_item()
            };
        };
    }

    assert_not_impl!(Glfw, Send, Sync);
    assert_not_impl!(Terminate, Send, Sync);
    assert_not_impl!(Window, Send, Sync);
    assert_not_impl!(Monitor, Send, Sync);

    const INIT_HINTS: &[InitHint] = &[InitHint::Platform(Platform::Null)];

    #[test]
    fn platform_supported() {
        assert!(Glfw::platform_supported(Platform::Null));
    }

    #[test]
    fn init() {
        let glfw = Glfw::init(INIT_HINTS).expect("it failed");
        assert_eq!(Platform::Null, glfw.get_platform());
        println!("{:?}", glfw.get_platform());
    }

    #[test]
    fn get_version() {
        let (major, minor, patch) = Glfw::get_version();
        println!("{:?}", (major, minor, patch));
    }

    #[test]
    fn foo1() {
        let _glfw = Glfw::init(INIT_HINTS).unwrap();
    }

    #[test]
    fn foo2() {
        let _glfw = Glfw::init(INIT_HINTS).unwrap();
    }

    #[test]
    fn foo3() {
        let _glfw = Glfw::init(INIT_HINTS).unwrap();
    }

    #[test]
    fn get_monitors() {
        let glfw = Glfw::init(INIT_HINTS).unwrap();
        let monitors = glfw.get_monitors();
        for monitor in monitors.iter() {
            println!("name: {}", monitor.get_name());
        }
        drop(glfw);
        drop(monitors);
    }

    #[test]
    fn create_window() {
        let glfw = Glfw::init(&INIT_HINTS).unwrap();
        let _window = glfw
            .create_window(
                &[WindowHint::ClientApi(ClientApi::None)],
                800,
                600,
                "test",
                None,
                None,
            )
            .expect("create_window");
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Key {
    Space = sys::GLFW_KEY_SPACE,

    Apostrophe = sys::GLFW_KEY_APOSTROPHE,

    Comma = sys::GLFW_KEY_COMMA,
    Minus = sys::GLFW_KEY_MINUS,
    Period = sys::GLFW_KEY_PERIOD,
    Slash = sys::GLFW_KEY_SLASH,
    Num0 = sys::GLFW_KEY_0,
    Num1 = sys::GLFW_KEY_1,
    Num2 = sys::GLFW_KEY_2,
    Num3 = sys::GLFW_KEY_3,
    Num4 = sys::GLFW_KEY_4,
    Num5 = sys::GLFW_KEY_5,
    Num6 = sys::GLFW_KEY_6,
    Num7 = sys::GLFW_KEY_7,
    Num8 = sys::GLFW_KEY_8,
    Num9 = sys::GLFW_KEY_9,

    Semicolon = sys::GLFW_KEY_SEMICOLON,

    Equal = sys::GLFW_KEY_EQUAL,

    A = sys::GLFW_KEY_A,
    B = sys::GLFW_KEY_B,
    C = sys::GLFW_KEY_C,
    D = sys::GLFW_KEY_D,
    E = sys::GLFW_KEY_E,
    F = sys::GLFW_KEY_F,
    G = sys::GLFW_KEY_G,
    H = sys::GLFW_KEY_H,
    I = sys::GLFW_KEY_I,
    J = sys::GLFW_KEY_J,
    K = sys::GLFW_KEY_K,
    L = sys::GLFW_KEY_L,
    M = sys::GLFW_KEY_M,
    N = sys::GLFW_KEY_N,
    O = sys::GLFW_KEY_O,
    P = sys::GLFW_KEY_P,
    Q = sys::GLFW_KEY_Q,
    R = sys::GLFW_KEY_R,
    S = sys::GLFW_KEY_S,
    T = sys::GLFW_KEY_T,
    U = sys::GLFW_KEY_U,
    V = sys::GLFW_KEY_V,
    W = sys::GLFW_KEY_W,
    X = sys::GLFW_KEY_X,
    Y = sys::GLFW_KEY_Y,
    Z = sys::GLFW_KEY_Z,
    LeftBracket = sys::GLFW_KEY_LEFT_BRACKET,
    Backslash = sys::GLFW_KEY_BACKSLASH,
    RightBracket = sys::GLFW_KEY_RIGHT_BRACKET,

    GraveAccent = sys::GLFW_KEY_GRAVE_ACCENT,

    World1 = sys::GLFW_KEY_WORLD_1,
    World2 = sys::GLFW_KEY_WORLD_2,

    Escape = sys::GLFW_KEY_ESCAPE,
    Enter = sys::GLFW_KEY_ENTER,
    Tab = sys::GLFW_KEY_TAB,
    Backspace = sys::GLFW_KEY_BACKSPACE,
    Insert = sys::GLFW_KEY_INSERT,
    Delete = sys::GLFW_KEY_DELETE,
    Right = sys::GLFW_KEY_RIGHT,
    Left = sys::GLFW_KEY_LEFT,
    Down = sys::GLFW_KEY_DOWN,
    Up = sys::GLFW_KEY_UP,
    PageUp = sys::GLFW_KEY_PAGE_UP,
    PageDown = sys::GLFW_KEY_PAGE_DOWN,
    Home = sys::GLFW_KEY_HOME,
    End = sys::GLFW_KEY_END,

    CapsLock = sys::GLFW_KEY_CAPS_LOCK,
    ScrollLock = sys::GLFW_KEY_SCROLL_LOCK,
    NumLock = sys::GLFW_KEY_NUM_LOCK,
    PrintScreen = sys::GLFW_KEY_PRINT_SCREEN,
    Pause = sys::GLFW_KEY_PAUSE,

    F1 = sys::GLFW_KEY_F1,
    F2 = sys::GLFW_KEY_F2,
    F3 = sys::GLFW_KEY_F3,
    F4 = sys::GLFW_KEY_F4,
    F5 = sys::GLFW_KEY_F5,
    F6 = sys::GLFW_KEY_F6,
    F7 = sys::GLFW_KEY_F7,
    F8 = sys::GLFW_KEY_F8,
    F9 = sys::GLFW_KEY_F9,
    F10 = sys::GLFW_KEY_F10,
    F11 = sys::GLFW_KEY_F11,
    F12 = sys::GLFW_KEY_F12,
    F13 = sys::GLFW_KEY_F13,
    F14 = sys::GLFW_KEY_F14,
    F15 = sys::GLFW_KEY_F15,
    F16 = sys::GLFW_KEY_F16,
    F17 = sys::GLFW_KEY_F17,
    F18 = sys::GLFW_KEY_F18,
    F19 = sys::GLFW_KEY_F19,
    F20 = sys::GLFW_KEY_F20,
    F21 = sys::GLFW_KEY_F21,
    F22 = sys::GLFW_KEY_F22,
    F23 = sys::GLFW_KEY_F23,
    F24 = sys::GLFW_KEY_F24,
    F25 = sys::GLFW_KEY_F25,

    Kp0 = sys::GLFW_KEY_KP_0,
    Kp1 = sys::GLFW_KEY_KP_1,
    Kp2 = sys::GLFW_KEY_KP_2,
    Kp3 = sys::GLFW_KEY_KP_3,
    Kp4 = sys::GLFW_KEY_KP_4,
    Kp5 = sys::GLFW_KEY_KP_5,
    Kp6 = sys::GLFW_KEY_KP_6,
    Kp7 = sys::GLFW_KEY_KP_7,
    Kp8 = sys::GLFW_KEY_KP_8,
    Kp9 = sys::GLFW_KEY_KP_9,
    KpDecimal = sys::GLFW_KEY_KP_DECIMAL,
    KpDivide = sys::GLFW_KEY_KP_DIVIDE,
    KpMultiply = sys::GLFW_KEY_KP_MULTIPLY,
    KpSubtract = sys::GLFW_KEY_KP_SUBTRACT,
    KpAdd = sys::GLFW_KEY_KP_ADD,
    KpEnter = sys::GLFW_KEY_KP_ENTER,
    KpEqual = sys::GLFW_KEY_KP_EQUAL,

    LeftShift = sys::GLFW_KEY_LEFT_SHIFT,
    LeftControl = sys::GLFW_KEY_LEFT_CONTROL,
    LeftAlt = sys::GLFW_KEY_LEFT_ALT,
    LeftSuper = sys::GLFW_KEY_LEFT_SUPER,
    RightShift = sys::GLFW_KEY_RIGHT_SHIFT,
    RightControl = sys::GLFW_KEY_RIGHT_CONTROL,
    RightAlt = sys::GLFW_KEY_RIGHT_ALT,
    RightSuper = sys::GLFW_KEY_RIGHT_SUPER,
    Menu = sys::GLFW_KEY_MENU,

    Unknown = sys::GLFW_KEY_UNKNOWN,
}

impl TryFrom<i32> for Key {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let ranges = [
            32..=32,
            39..=39,
            44..=57,
            59..=59,
            61..=61,
            65..=93,
            96..=96,
            161..=162,
            256..=269,
            280..=284,
            290..=314,
            320..=336,
            340..=348,
        ];
        for range in ranges {
            if range.contains(&value) {
                return Ok(unsafe { mem::transmute(value) });
            }
        }
        Err(value)
    }
}

/// Alias to `MouseButton1`, supplied for improved clarity.
pub use self::MouseButton::Button1 as MouseButtonLeft;
/// Alias to `MouseButton2`, supplied for improved clarity.
pub use self::MouseButton::Button2 as MouseButtonRight;
/// Alias to `MouseButton3`, supplied for improved clarity.
pub use self::MouseButton::Button3 as MouseButtonMiddle;

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MouseButton {
    /// The left mouse button. A `MouseButtonLeft` alias is provided to improve clarity.
    Button1 = sys::GLFW_MOUSE_BUTTON_1,
    /// The right mouse button. A `MouseButtonRight` alias is provided to improve clarity.
    Button2 = sys::GLFW_MOUSE_BUTTON_2,
    /// The middle mouse button. A `MouseButtonMiddle` alias is provided to improve clarity.
    Button3 = sys::GLFW_MOUSE_BUTTON_3,
    Button4 = sys::GLFW_MOUSE_BUTTON_4,
    Button5 = sys::GLFW_MOUSE_BUTTON_5,
    Button6 = sys::GLFW_MOUSE_BUTTON_6,
    Button7 = sys::GLFW_MOUSE_BUTTON_7,
    Button8 = sys::GLFW_MOUSE_BUTTON_8,
}

impl TryFrom<i32> for MouseButton {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value >= sys::GLFW_MOUSE_BUTTON_1 && value <= sys::GLFW_MOUSE_BUTTON_LAST {
            return Ok(unsafe { mem::transmute(value) });
        }
        Err(value)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    Monitor,
    Window(WindowEvent),
}

#[derive(Clone, PartialEq, Debug)]
pub enum MonitorEvent {
    Connected(MonitorId),
    Disconnected(MonitorId),
}

#[derive(Clone, PartialEq, Debug)]
pub enum WindowEvent {
    Pos(i32, i32),
    Size(i32, i32),
    Close,
    Refresh,
    Focus(bool),
    Iconify(bool),
    FramebufferSize(i32, i32),
    MouseButton(MouseButton, Action, Modifiers),
    CursorPos(f64, f64),
    CursorEnter(bool),
    Scroll(f64, f64),
    Key(Key, Scancode, Action, Modifiers),
    Char(Codepoint),
    #[deprecated(note = "Scheduled for removal in version 4.0")]
    CharModifiers(Codepoint, Modifiers),
    FileDrop(Vec<PathBuf>),
    Maximize(bool),
    ContentScale(f32, f32),
}

pub type Scancode = core::ffi::c_int;

/// Native endian UTF-32
pub type Codepoint = core::ffi::c_uint;

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct Modifiers: i32 {
        const SHIFT     = sys::GLFW_MOD_SHIFT;
        const CONTROL   = sys::GLFW_MOD_CONTROL;
        const ALT       = sys::GLFW_MOD_ALT;
        const SUPER     = sys::GLFW_MOD_SUPER;
        const CAPS_LOCK  = sys::GLFW_MOD_CAPS_LOCK;
        const NUM_LOCK   = sys::GLFW_MOD_NUM_LOCK;
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Action {
    Release = sys::GLFW_RELEASE,
    Press = sys::GLFW_PRESS,
    Repeat = sys::GLFW_REPEAT,
}

impl TryFrom<i32> for Action {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value >= sys::GLFW_RELEASE && value <= sys::GLFW_REPEAT {
            return Ok(unsafe { mem::transmute(value) });
        }
        Err(value)
    }
}

unsafe extern "C" fn monitor_callback(monitor: *mut sys::GLFWmonitor, event: c_int) {
    unsafe {
        let name = CStr::from_ptr(sys::glfwGetMonitorName(monitor));
        println!("monitor: {:?} = {:?}", name, event);
    }
    // println!("monitor event: {}", event);
}

pub unsafe fn set_global_callbacks() -> Result<(), Error> {
    // sys::glfwSetErrorCallback(callback);
    sys::glfwSetMonitorCallback(Some(monitor_callback));
    Glfw::get_error()?;
    // sys::glfwSetJoystickCallback(Some(callback));
    Ok(())
}
