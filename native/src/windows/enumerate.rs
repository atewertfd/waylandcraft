use crate::windows::error::WindowsError;
use std::path::PathBuf;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GWL_EXSTYLE, GWL_STYLE, GetClassNameW, GetClientRect,
    GetWindowLongW, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindow, IsWindowVisible, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

const SKIP_CLASSES: &[&str] = &[
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
    "Progman",
    "WorkerW",
    "NotifyIconOverflowWindow",
    "DummyDWMListenerWindow",
    "CiceroUIWndFrame",
    "IME",
    "MSCTFIME UI",
    "Windows.UI.Core.CoreWindow",
    "ForegroundStaging",
];

#[derive(Clone, Debug)]
pub struct EnumeratedWindow {
    pub hwnd: isize,
    pub pid: u32,
    pub title: String,
    pub class_name: String,
    pub exe: PathBuf,
    pub width: i32,
    pub height: i32,
    pub minimized: bool,
    pub visible: bool,
    pub elevated: bool,
}

pub fn hwnd_from_raw(handle: isize) -> HWND {
    HWND(handle as *mut core::ffi::c_void)
}

pub fn hwnd_to_raw(hwnd: HWND) -> isize {
    hwnd.0 as isize
}

pub fn should_include_window(
    title: &str,
    class_name: &str,
    ex_style: u32,
    visible: bool,
    cloaked: bool,
    client_w: i32,
    client_h: i32,
    is_own_process: bool,
) -> bool {
    if is_own_process {
        return false;
    }
    if !visible || cloaked {
        return false;
    }
    if title.trim().is_empty() {
        return false;
    }
    if client_w <= 0 || client_h <= 0 {
        return false;
    }
    if (ex_style & WS_EX_TOOLWINDOW.0) != 0 {
        return false;
    }
    if (ex_style & WS_EX_NOACTIVATE.0) != 0 {
        return false;
    }
    if SKIP_CLASSES
        .iter()
        .any(|skip| class_name.eq_ignore_ascii_case(skip))
    {
        return false;
    }
    true
}

struct EnumContext {
    own_pid: u32,
    windows: Vec<EnumeratedWindow>,
}

pub fn enumerate_toplevels() -> Result<Vec<EnumeratedWindow>, WindowsError> {
    let mut ctx = EnumContext {
        own_pid: unsafe { GetCurrentProcessId() },
        windows: Vec::new(),
    };
    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            LPARAM((&mut ctx as *mut EnumContext) as isize),
        )?;
    }
    Ok(ctx.windows)
}

unsafe extern "system" fn enum_windows_proc(
    hwnd: HWND,
    lparam: LPARAM,
) -> BOOL {
    let ctx = unsafe { &mut *(lparam.0 as *mut EnumContext) };
    if let Ok(Some(window)) = inspect_window(hwnd, ctx.own_pid) {
        ctx.windows.push(window);
    }
    true.into()
}

fn inspect_window(
    hwnd: HWND,
    own_pid: u32,
) -> Result<Option<EnumeratedWindow>, WindowsError> {
    if !unsafe { IsWindow(hwnd).as_bool() } {
        return Ok(None);
    }

    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    let is_own = pid == own_pid;

    let title = window_text(hwnd);
    let class_name = class_name(hwnd);
    let visible = unsafe { IsWindowVisible(hwnd).as_bool() };
    let minimized = unsafe { IsIconic(hwnd).as_bool() };
    let ex_style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32;
    let _style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32;
    let cloaked = is_cloaked(hwnd);
    let (width, height) = client_size(hwnd);

    if !should_include_window(
        &title,
        &class_name,
        ex_style,
        visible,
        cloaked,
        width,
        height,
        is_own,
    ) {
        return Ok(None);
    }

    Ok(Some(EnumeratedWindow {
        hwnd: hwnd_to_raw(hwnd),
        pid,
        title,
        class_name,
        exe: crate::windows::apps::process_image_path(pid).unwrap_or_default(),
        width,
        height,
        minimized,
        visible,
        elevated: crate::windows::input::is_elevated_process(pid),
    }))
}

pub fn window_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
    if len <= 0 {
        String::new()
    } else {
        String::from_utf16_lossy(&buf[..len as usize])
    }
}

pub fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    if len <= 0 {
        String::new()
    } else {
        String::from_utf16_lossy(&buf[..len as usize])
    }
}

pub fn client_size(hwnd: HWND) -> (i32, i32) {
    let mut rect = RECT::default();
    if unsafe { GetClientRect(hwnd, &mut rect) }.is_err() {
        return (0, 0);
    }
    (rect.right - rect.left, rect.bottom - rect.top)
}

fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked = 0u32;
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut _,
            std::mem::size_of::<u32>() as u32,
        )
    };
    result.is_ok() && cloaked != 0
}

pub fn is_alive(handle: isize) -> bool {
    unsafe { IsWindow(hwnd_from_raw(handle)).as_bool() }
}

/// Inspect a window the user already chose, even if it would fail the picker filter
/// (for example while minimized).
pub fn inspect_adopted(handle: isize) -> Option<EnumeratedWindow> {
    let hwnd = hwnd_from_raw(handle);
    if !unsafe { IsWindow(hwnd).as_bool() } {
        return None;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    let (width, height) = client_size(hwnd);
    Some(EnumeratedWindow {
        hwnd: handle,
        pid,
        title: window_text(hwnd),
        class_name: class_name(hwnd),
        exe: crate::windows::apps::process_image_path(pid).unwrap_or_default(),
        width,
        height,
        minimized: unsafe { IsIconic(hwnd).as_bool() },
        visible: unsafe { IsWindowVisible(hwnd).as_bool() },
        elevated: crate::windows::input::is_elevated_process(pid),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_empty_title_and_tool_windows() {
        assert!(!should_include_window(
            "", "Notepad", 0, true, false, 800, 600, false
        ));
        assert!(!should_include_window(
            "Clock",
            "Clock",
            WS_EX_TOOLWINDOW.0,
            true,
            false,
            200,
            200,
            false
        ));
        assert!(!should_include_window(
            "Minecraft",
            "GLFW30",
            0,
            true,
            false,
            800,
            600,
            true
        ));
        assert!(!should_include_window(
            "Taskbar",
            "Shell_TrayWnd",
            0,
            true,
            false,
            1920,
            40,
            false
        ));
        assert!(should_include_window(
            "Untitled - Notepad",
            "Notepad",
            0,
            true,
            false,
            800,
            600,
            false
        ));
    }
}
