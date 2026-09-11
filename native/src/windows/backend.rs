use crate::windows::apps::{self, DesktopApp};
use crate::windows::capture::{self, CpuFrame, SharedDevice, WindowCapture};
use crate::windows::clipboard;
use crate::windows::enumerate::{self, EnumeratedWindow};
use crate::windows::error::WindowsError;
use crate::windows::icons;
use crate::windows::input;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
use windows::Win32::UI::WindowsAndMessaging::{
    HWND_TOP, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SW_SHOW, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOZORDER, SetWindowPos, ShowWindow,
};

const FORMAT_ARGB8888: i32 = 0;

pub struct TrackedWindow {
    pub info: EnumeratedWindow,
    pub capture: Option<WindowCapture>,
    pub frame: Option<CpuFrame>,
    pub dirty: bool,
    pub last_error: Option<String>,
}

pub struct PendingLaunch {
    pub pid: u32,
    pub started: Instant,
}

pub struct WindowsBackend {
    pub device: SharedDevice,
    pub windows: HashMap<isize, TrackedWindow>,
    pub focus: Option<isize>,
    pub keyboard_active: bool,
    pub pointer_x: f64,
    pub pointer_y: f64,
    pub pointer_target: Option<isize>,
    pub output_size: (i32, i32),
    pub output_bounds: (i32, i32),
    pub apps: Vec<DesktopApp>,
    pub icon_dir: PathBuf,
    pub pending_launches: Vec<PendingLaunch>,
    pub adopted: HashSet<isize>,
    pub last_click: Instant,
    pub last_click_pos: (f64, f64),
    pub preferred_terminal: String,
}

impl WindowsBackend {
    pub fn new() -> Result<Self, WindowsError> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let _ = RoInitialize(RO_INIT_MULTITHREADED);
        }
        let icon_dir = std::env::temp_dir().join("waylandcraft-windows-icons");
        let _ = std::fs::create_dir_all(&icon_dir);
        let device = capture::create_shared_device()?;
        let apps = apps::load_start_menu_apps(&icon_dir).unwrap_or_default();
        eprintln!(
            "waylandcraft-windows: native=2.1.0-windows.26.2-alpha.2 backend=windows-graphics-capture pixel=BGRA32 apps={}",
            apps.len()
        );
        Ok(Self {
            device,
            windows: HashMap::new(),
            focus: None,
            keyboard_active: false,
            pointer_x: 0.0,
            pointer_y: 0.0,
            pointer_target: None,
            output_size: (1920, 1080),
            output_bounds: (1920, 1080),
            apps,
            icon_dir,
            pending_launches: Vec::new(),
            adopted: HashSet::new(),
            last_click: Instant::now() - Duration::from_secs(1),
            last_click_pos: (0.0, 0.0),
            preferred_terminal: String::new(),
        })
    }

    pub fn adopt(&mut self, hwnd: isize) -> bool {
        if hwnd == 0 || !enumerate::is_alive(hwnd) {
            return false;
        }
        self.adopted.insert(hwnd);
        if let Some(info) = enumerate::inspect_adopted(hwnd) {
            self.upsert_window(info);
        }
        eprintln!("waylandcraft-windows: adopted HWND {hwnd:#x}");
        true
    }

    fn upsert_window(&mut self, info: EnumeratedWindow) {
        let hwnd = info.hwnd;
        if let Some(existing) = self.windows.get_mut(&hwnd) {
            let resized = existing.info.width != info.width
                || existing.info.height != info.height;
            existing.info = info;
            if resized {
                existing.capture = None;
                existing.dirty = true;
            }
        } else {
            self.windows.insert(
                hwnd,
                TrackedWindow {
                    info,
                    capture: None,
                    frame: None,
                    dirty: true,
                    last_error: None,
                },
            );
        }
    }

    pub fn refresh_windows(&mut self) {
        let enumerated = enumerate::enumerate_toplevels().unwrap_or_default();
        let pending_pids: Vec<u32> =
            self.pending_launches.iter().map(|l| l.pid).collect();
        for pid in pending_pids {
            for info in &enumerated {
                if info.pid == pid {
                    self.adopted.insert(info.hwnd);
                    eprintln!(
                        "waylandcraft-windows: associated launched PID {pid} with HWND {:#x}",
                        info.hwnd
                    );
                }
            }
        }
        let adopted = self.adopted.clone();
        for hwnd in adopted {
            if let Some(info) =
                enumerated.iter().find(|w| w.hwnd == hwnd).cloned()
            {
                self.upsert_window(info);
            } else if let Some(info) = enumerate::inspect_adopted(hwnd) {
                self.upsert_window(info);
            }
        }
        self.windows.retain(|hwnd, _| {
            self.adopted.contains(hwnd) && enumerate::is_alive(*hwnd)
        });
        self.adopted.retain(|hwnd| enumerate::is_alive(*hwnd));
        self.pending_launches.retain(|launch| {
            launch.started.elapsed() < Duration::from_secs(20)
        });
    }

    pub fn adopted_handles(&self) -> Vec<isize> {
        self.adopted
            .iter()
            .copied()
            .filter(|h| enumerate::is_alive(*h))
            .collect()
    }

    pub fn ensure_capture(&mut self, hwnd: isize) {
        let Some(window) = self.windows.get_mut(&hwnd) else {
            return;
        };
        if window.info.minimized {
            return;
        }
        if window.info.elevated {
            if window.last_error.is_none() {
                let msg = format!(
                    "Capture/input of HWND {:#x} may fail because the process is elevated",
                    hwnd
                );
                eprintln!("waylandcraft-windows: {msg}");
                window.last_error = Some(msg);
            }
        }
        if window.capture.is_some() {
            return;
        }
        match WindowCapture::start(hwnd, self.device.clone()) {
            Ok(capture) => {
                window.capture = Some(capture);
                window.dirty = true;
            }
            Err(err) => {
                let msg = format!("WGC failed for HWND {hwnd:#x}: {err}");
                eprintln!("waylandcraft-windows: {msg}");
                window.last_error = Some(msg);
            }
        }
    }

    pub fn poll_frames(&mut self) {
        for window in self.windows.values_mut() {
            if let Some(capture) = &window.capture {
                if let Some(frame) = capture.take_frame() {
                    window.info.width = frame.width as i32;
                    window.info.height = frame.height as i32;
                    window.frame = Some(frame);
                    window.dirty = true;
                }
            }
        }
    }

    pub fn attach_ready(
        &mut self,
        hwnd: isize,
    ) -> Option<(*const u8, i32, i32, i32, i32)> {
        if !self.adopted.contains(&hwnd) {
            return None;
        }
        self.ensure_capture(hwnd);
        let window = self.windows.get_mut(&hwnd)?;
        if !window.dirty {
            return None;
        }
        let frame = window.frame.as_ref()?;
        window.dirty = false;
        Some((
            frame.bgra.as_ptr(),
            frame.width as i32,
            frame.height as i32,
            FORMAT_ARGB8888,
            frame.stride as i32,
        ))
    }

    pub fn title(&self, hwnd: isize) -> String {
        self.windows
            .get(&hwnd)
            .map(|w| {
                if w.info.elevated {
                    format!("{} [elevated]", w.info.title)
                } else {
                    w.info.title.clone()
                }
            })
            .unwrap_or_default()
    }

    pub fn app_id(&self, hwnd: isize) -> String {
        self.windows
            .get(&hwnd)
            .map(|w| w.info.exe.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn size(&self, hwnd: isize) -> (i32, i32) {
        self.windows
            .get(&hwnd)
            .map(|w| {
                if let Some(frame) = &w.frame {
                    (frame.width as i32, frame.height as i32)
                } else {
                    (w.info.width, w.info.height)
                }
            })
            .unwrap_or((0, 0))
    }

    pub fn focus(&mut self, hwnd: Option<isize>) {
        self.focus = hwnd;
        if let Some(hwnd) = hwnd {
            self.ensure_capture(hwnd);
        }
    }

    pub fn send_motion(&mut self, hwnd: Option<isize>, x: f64, y: f64) {
        self.pointer_x = x;
        self.pointer_y = y;
        self.pointer_target = hwnd;
        if let Some(hwnd) = hwnd {
            let (cw, ch) = self.size(hwnd);
            let (vw, vh) = self
                .windows
                .get(&hwnd)
                .map(|w| (w.info.width, w.info.height))
                .unwrap_or((cw, ch));
            let _ = input::send_mouse_move(hwnd, x, y, cw, ch, vw, vh);
        }
    }

    pub fn send_button(
        &mut self,
        button: i32,
        pressed: bool,
    ) -> Result<i32, WindowsError> {
        let Some(hwnd) = self.pointer_target.or(self.focus) else {
            return Ok(0);
        };
        if self.windows.get(&hwnd).is_some_and(|w| w.info.elevated) {
            if let Some(msg) = self
                .windows
                .get(&hwnd)
                .and_then(|w| input::describe_input_restriction(&w.info))
            {
                eprintln!("waylandcraft-windows: {msg}");
            }
            return Ok(0);
        }
        let now = Instant::now();
        let double = pressed
            && now.duration_since(self.last_click) < Duration::from_millis(400)
            && (self.pointer_x - self.last_click_pos.0).abs() < 4.0
            && (self.pointer_y - self.last_click_pos.1).abs() < 4.0;
        if pressed {
            self.last_click = now;
            self.last_click_pos = (self.pointer_x, self.pointer_y);
        }
        let (cw, ch) = self.size(hwnd);
        let (vw, vh) = self
            .windows
            .get(&hwnd)
            .map(|w| (w.info.width, w.info.height))
            .unwrap_or((cw, ch));
        input::send_mouse_button(
            hwnd,
            button,
            pressed,
            self.pointer_x,
            self.pointer_y,
            cw,
            ch,
            vw,
            vh,
            double,
        )
    }

    pub fn send_scroll(
        &mut self,
        axis: i32,
        value: f64,
    ) -> Result<(), WindowsError> {
        let Some(hwnd) = self.pointer_target.or(self.focus) else {
            return Ok(());
        };
        input::send_scroll(
            hwnd,
            axis,
            value,
            self.pointer_x as i32,
            self.pointer_y as i32,
        )
    }

    pub fn send_key(
        &mut self,
        scancode: i32,
        pressed: bool,
    ) -> Result<(), WindowsError> {
        if !self.keyboard_active {
            return Ok(());
        }
        let Some(hwnd) = self.focus else {
            return Ok(());
        };
        if self.windows.get(&hwnd).is_some_and(|w| w.info.elevated) {
            return Ok(());
        }
        input::send_key(hwnd, scancode as u32, pressed)
    }

    pub fn resize(
        &mut self,
        hwnd: isize,
        width: i32,
        height: i32,
    ) -> Result<(), WindowsError> {
        if width < 1 {
            return Err(WindowsError::NonPositiveWidth);
        }
        if height < 1 {
            return Err(WindowsError::NonPositiveHeight);
        }
        let hwnd_win = enumerate::hwnd_from_raw(hwnd);
        unsafe {
            SetWindowPos(
                hwnd_win,
                Some(HWND_TOP),
                0,
                0,
                width,
                height,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
        }
        if let Some(window) = self.windows.get_mut(&hwnd) {
            window.capture = None;
            window.dirty = true;
        }
        Ok(())
    }

    pub fn maximize(&mut self, hwnd: isize) {
        unsafe {
            let _ = ShowWindow(enumerate::hwnd_from_raw(hwnd), SW_MAXIMIZE);
        }
    }

    pub fn fullscreen(&mut self, hwnd: isize) {
        self.maximize(hwnd);
        let (w, h) = self.output_size;
        let _ = self.resize(hwnd, w, h);
    }

    pub fn minimize(&mut self, hwnd: isize) {
        unsafe {
            let _ = ShowWindow(enumerate::hwnd_from_raw(hwnd), SW_MINIMIZE);
        }
    }

    pub fn restore(&mut self, hwnd: isize) {
        unsafe {
            let _ = ShowWindow(enumerate::hwnd_from_raw(hwnd), SW_RESTORE);
            let _ = ShowWindow(enumerate::hwnd_from_raw(hwnd), SW_SHOW);
        }
    }

    pub fn unadopt(&mut self, hwnd: isize) {
        self.adopted.remove(&hwnd);
        self.windows.remove(&hwnd);
        if self.focus == Some(hwnd) {
            self.focus = None;
        }
        if self.pointer_target == Some(hwnd) {
            self.pointer_target = None;
        }
    }

    pub fn running_window_apps(&self) -> Vec<DesktopApp> {
        enumerate::enumerate_toplevels()
            .unwrap_or_default()
            .into_iter()
            .map(|info| {
                DesktopApp {
                    app_id: format!("hwnd:{}", info.hwnd),
                    name: format!("{} (open)", info.title),
                    generic_name: Some("Running Windows application".into()),
                    exec: info.exe.to_string_lossy().into_owned(),
                    exec_terminal: false,
                    comment: Some(format!(
                        "Capture the already-open window HWND {:#x}",
                        info.hwnd
                    )),
                    keywords: vec![info.title, info.class_name],
                    categories: vec!["Running".into()],
                    visible: true,
                    icon_path: None,
                }
            })
            .collect()
    }

    pub fn exec_app(&mut self, app_id: &str) -> bool {
        if let Some(rest) = app_id.strip_prefix("hwnd:") {
            if let Ok(hwnd) = rest.parse::<isize>() {
                return self.adopt(hwnd);
            }
        }
        let Some(app) = self.apps.iter().find(|a| a.app_id == app_id).cloned()
        else {
            eprintln!("waylandcraft-windows: unknown app id {app_id}");
            return false;
        };
        match apps::launch_app(&app) {
            Ok(pid) => {
                eprintln!(
                    "waylandcraft-windows: launched {} pid={}",
                    app.name, pid
                );
                if pid != 0 {
                    self.pending_launches.push(PendingLaunch {
                        pid,
                        started: Instant::now(),
                    });
                }
                true
            }
            Err(err) => {
                eprintln!(
                    "waylandcraft-windows: launch failed for {}: {err}",
                    app.name
                );
                false
            }
        }
    }

    pub fn icon_for_window(&self, hwnd: isize) -> Option<String> {
        icons::extract_window_icon_png(
            enumerate::hwnd_from_raw(hwnd),
            &self.icon_dir,
        )
        .map(|p| p.to_string_lossy().into_owned())
    }

    pub fn clipboard_text(&self) -> Option<String> {
        clipboard::get_text().ok().flatten()
    }
}
