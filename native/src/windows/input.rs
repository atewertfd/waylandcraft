use crate::windows::enumerate::{EnumeratedWindow, hwnd_from_raw};
use crate::windows::error::WindowsError;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::Security::{
    GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, MAPVK_VSC_TO_VK, MapVirtualKeyW, ToUnicode, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    ClientToScreen, GetForegroundWindow, MK_LBUTTON, MK_MBUTTON, MK_RBUTTON,
    PostMessageW, ScreenToClient, SendNotifyMessageW, WHEEL_DELTA, WM_CHAR,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN,
    WM_RBUTTONUP,
};
use windows::core::Owned;

pub const BTN_LEFT: i32 = 0x110;
pub const BTN_RIGHT: i32 = 0x111;
pub const BTN_MIDDLE: i32 = 0x112;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub fn linux_button_to_mouse(button: i32) -> Option<MouseButton> {
    match button {
        BTN_LEFT => Some(MouseButton::Left),
        BTN_RIGHT => Some(MouseButton::Right),
        BTN_MIDDLE => Some(MouseButton::Middle),
        _ => None,
    }
}

pub fn client_to_lparam(x: i32, y: i32) -> LPARAM {
    let packed = ((y as u16 as u32) << 16) | (x as u16 as u32);
    LPARAM(packed as isize)
}

pub fn key_lparam(scancode: u32, key_up: bool) -> LPARAM {
    let mut value = (scancode & 0xFF) << 16;
    if key_up {
        value |= 1 << 30;
        value |= 1 << 31;
    }
    LPARAM(value as isize)
}

pub fn scale_client_point(
    x: f64,
    y: f64,
    capture_width: i32,
    capture_height: i32,
    client_width: i32,
    client_height: i32,
) -> (i32, i32) {
    if capture_width <= 0 || capture_height <= 0 {
        return (x.round() as i32, y.round() as i32);
    }
    let sx = client_width as f64 / capture_width as f64;
    let sy = client_height as f64 / capture_height as f64;
    ((x * sx).round() as i32, (y * sy).round() as i32)
}

pub fn send_mouse_move(
    hwnd_raw: isize,
    x: f64,
    y: f64,
    capture_width: i32,
    capture_height: i32,
    client_width: i32,
    client_height: i32,
) -> Result<(), WindowsError> {
    let hwnd = hwnd_from_raw(hwnd_raw);
    let (cx, cy) = scale_client_point(
        x,
        y,
        capture_width,
        capture_height,
        client_width,
        client_height,
    );
    let lp = client_to_lparam(cx, cy);
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_MOUSEMOVE, WPARAM(0), lp);
    }
    Ok(())
}

pub fn send_mouse_button(
    hwnd_raw: isize,
    button: i32,
    pressed: bool,
    x: f64,
    y: f64,
    capture_width: i32,
    capture_height: i32,
    client_width: i32,
    client_height: i32,
    double_click: bool,
) -> Result<i32, WindowsError> {
    let Some(kind) = linux_button_to_mouse(button) else {
        return Err(WindowsError::UnknownPointerButton(button));
    };
    let hwnd = hwnd_from_raw(hwnd_raw);
    let (cx, cy) = scale_client_point(
        x,
        y,
        capture_width,
        capture_height,
        client_width,
        client_height,
    );
    let lp = client_to_lparam(cx, cy);
    let (msg, wp) = match (kind, pressed, double_click) {
        (MouseButton::Left, true, true) => {
            (WM_LBUTTONDBLCLK, WPARAM(MK_LBUTTON.0 as usize))
        }
        (MouseButton::Left, true, false) => {
            (WM_LBUTTONDOWN, WPARAM(MK_LBUTTON.0 as usize))
        }
        (MouseButton::Left, false, _) => (WM_LBUTTONUP, WPARAM(0)),
        (MouseButton::Right, true, _) => {
            (WM_RBUTTONDOWN, WPARAM(MK_RBUTTON.0 as usize))
        }
        (MouseButton::Right, false, _) => (WM_RBUTTONUP, WPARAM(0)),
        (MouseButton::Middle, true, _) => {
            (WM_MBUTTONDOWN, WPARAM(MK_MBUTTON.0 as usize))
        }
        (MouseButton::Middle, false, _) => (WM_MBUTTONUP, WPARAM(0)),
    };
    unsafe {
        if !PostMessageW(Some(hwnd), msg, wp, lp).as_bool() {
            let _ = SendNotifyMessageW(hwnd, msg, wp, lp);
        }
    }
    Ok(1)
}

pub fn send_scroll(
    hwnd_raw: isize,
    axis: i32,
    value: f64,
    x: i32,
    y: i32,
) -> Result<(), WindowsError> {
    if axis != 0 && axis != 1 {
        return Err(WindowsError::UnknownScrollDirection(axis));
    }
    if axis != 0 {
        return Ok(());
    }
    let hwnd = hwnd_from_raw(hwnd_raw);
    let delta = (value * WHEEL_DELTA as f64).round() as i16;
    let wp = WPARAM((delta as u16 as u32 as usize) << 16);
    let mut pt = POINT { x, y };
    unsafe {
        let _ = ClientToScreen(hwnd, &mut pt);
        let lp = LPARAM(
            (((pt.y as u16 as u32) << 16) | (pt.x as u16 as u32)) as isize,
        );
        let _ = PostMessageW(Some(hwnd), WM_MOUSEWHEEL, wp, lp);
    }
    Ok(())
}

pub fn send_key(
    hwnd_raw: isize,
    scancode: u32,
    pressed: bool,
) -> Result<(), WindowsError> {
    let hwnd = hwnd_from_raw(hwnd_raw);
    let vk = unsafe { MapVirtualKeyW(scancode, MAPVK_VSC_TO_VK) };
    if vk == 0 {
        return Ok(());
    }
    let msg = if pressed { WM_KEYDOWN } else { WM_KEYUP };
    let lp = key_lparam(scancode, !pressed);
    unsafe {
        let _ = PostMessageW(Some(hwnd), msg, WPARAM(vk as usize), lp);
        if pressed {
            if let Some(ch) = scancode_to_char(scancode, vk) {
                let _ =
                    PostMessageW(Some(hwnd), WM_CHAR, WPARAM(ch as usize), lp);
            }
        }
    }
    Ok(())
}

fn scancode_to_char(scancode: u32, vk: u32) -> Option<u16> {
    let mut state = [0u8; 256];
    for i in 0..256 {
        let down = unsafe { GetAsyncKeyState(i as i32) } as u16;
        if down & 0x8000 != 0 {
            state[i] = 0x80;
        }
    }
    let mut buf = [0u16; 4];
    let written = unsafe { ToUnicode(vk, scancode, Some(&state), &mut buf, 0) };
    if written > 0 { Some(buf[0]) } else { None }
}

pub fn is_elevated_process(pid: u32) -> bool {
    unsafe {
        let Ok(process) =
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
        else {
            return true;
        };
        let process = Owned::new(process);
        let mut token = Default::default();
        if OpenProcessToken(*process, TOKEN_QUERY, &mut token).is_err() {
            return true;
        }
        let token = Owned::new(token);
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0u32;
        if GetTokenInformation(
            *token,
            TokenElevation,
            Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        )
        .is_err()
        {
            return true;
        }
        let mut self_token = Default::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut self_token)
            .is_err()
        {
            return elevation.TokenIsElevated != 0;
        }
        let self_token = Owned::new(self_token);
        let mut self_elevation = TOKEN_ELEVATION::default();
        let mut self_size = 0u32;
        let _ = GetTokenInformation(
            *self_token,
            TokenElevation,
            Some((&mut self_elevation as *mut TOKEN_ELEVATION).cast()),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut self_size,
        );
        elevation.TokenIsElevated != 0 && self_elevation.TokenIsElevated == 0
    }
}

pub fn describe_input_restriction(window: &EnumeratedWindow) -> Option<String> {
    if window.elevated {
        Some(format!(
            "Windows blocked input to HWND {:#x} (PID {}). The target is elevated and UIPI forbids injecting input from Minecraft.",
            window.hwnd, window.pid
        ))
    } else {
        None
    }
}

pub fn screen_to_client(
    hwnd_raw: isize,
    screen_x: i32,
    screen_y: i32,
) -> (i32, i32) {
    let mut pt = POINT {
        x: screen_x,
        y: screen_y,
    };
    unsafe {
        let _ = ScreenToClient(hwnd_from_raw(hwnd_raw), &mut pt);
    }
    (pt.x, pt.y)
}

#[allow(dead_code)]
pub fn is_foreground(hwnd_raw: isize) -> bool {
    unsafe { GetForegroundWindow() == hwnd_from_raw(hwnd_raw) }
}

#[allow(dead_code)]
pub fn virtual_key(_key: VIRTUAL_KEY) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_linux_buttons() {
        assert_eq!(linux_button_to_mouse(BTN_LEFT), Some(MouseButton::Left));
        assert_eq!(linux_button_to_mouse(BTN_RIGHT), Some(MouseButton::Right));
        assert_eq!(linux_button_to_mouse(0x99), None);
    }

    #[test]
    fn packs_mouse_lparam() {
        let lp = client_to_lparam(10, 20);
        assert_eq!(lp.0 as u32 & 0xFFFF, 10);
        assert_eq!((lp.0 as u32 >> 16) & 0xFFFF, 20);
    }

    #[test]
    fn scales_capture_to_client() {
        assert_eq!(
            scale_client_point(50.0, 25.0, 100, 50, 200, 100),
            (100, 50)
        );
        assert_eq!(scale_client_point(0.0, 0.0, 0, 0, 200, 100), (0, 0));
    }

    #[test]
    fn key_up_sets_transition_bits() {
        let down = key_lparam(0x1E, false);
        let up = key_lparam(0x1E, true);
        assert_eq!((down.0 as u32 >> 16) & 0xFF, 0x1E);
        assert_ne!(up.0 & (1 << 31), 0);
    }
}
