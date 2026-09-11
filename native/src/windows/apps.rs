use crate::windows::error::WindowsError;
use crate::windows::icons;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, CoCreateInstance, IPersistFile, STGM_READ,
};
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Shell::{
    FOLDERID_CommonPrograms, FOLDERID_Programs, IShellLinkW, KF_FLAG_DEFAULT,
    SHGetKnownFolderPath, ShellLink,
};
use windows::core::{Interface, PCWSTR};

#[derive(Clone, Debug)]
pub struct DesktopApp {
    pub app_id: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub exec: String,
    pub exec_terminal: bool,
    pub comment: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub visible: bool,
    pub icon_path: Option<String>,
}

pub fn process_image_path(pid: u32) -> Option<PathBuf> {
    unsafe {
        let process =
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; MAX_PATH as usize];
        let len = K32GetModuleFileNameExW(Some(process), None, &mut buf);
        let _ = windows::Win32::Foundation::CloseHandle(process);
        if len == 0 {
            return None;
        }
        Some(PathBuf::from(String::from_utf16_lossy(
            &buf[..len as usize],
        )))
    }
}

pub fn load_start_menu_apps(
    icon_dir: &Path,
) -> Result<Vec<DesktopApp>, WindowsError> {
    let mut apps = Vec::new();
    for folder in [
        known_folder(&FOLDERID_Programs),
        known_folder(&FOLDERID_CommonPrograms),
    ]
    .into_iter()
    .flatten()
    {
        collect_shortcuts(&folder, &folder, icon_dir, &mut apps)?;
    }
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.app_id == b.app_id);
    Ok(apps)
}

fn known_folder(id: &windows::core::GUID) -> Option<PathBuf> {
    unsafe {
        SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None)
            .ok()
            .map(|pw| PathBuf::from(pw.to_string().unwrap_or_default()))
    }
}

fn collect_shortcuts(
    root: &Path,
    dir: &Path,
    icon_dir: &Path,
    apps: &mut Vec<DesktopApp>,
) -> Result<(), WindowsError> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_shortcuts(root, &path, icon_dir, apps)?;
            continue;
        }
        if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
        {
            if let Some(app) = shortcut_to_app(root, &path, icon_dir) {
                apps.push(app);
            }
        }
    }
    Ok(())
}

fn shortcut_to_app(
    root: &Path,
    path: &Path,
    icon_dir: &Path,
) -> Option<DesktopApp> {
    let (target, args, comment) = resolve_shortcut(path).ok()?;
    if target.is_empty() {
        return None;
    }
    let name = path.file_stem()?.to_string_lossy().into_owned();
    let exec = if args.is_empty() {
        target.clone()
    } else {
        format!("\"{}\" {}", target, args)
    };
    let category = category_from_path(root, path);
    let icon_path = icons::extract_file_icon_png(Path::new(&target), icon_dir)
        .map(|p| p.to_string_lossy().into_owned());
    Some(DesktopApp {
        app_id: path.to_string_lossy().into_owned(),
        name,
        generic_name: None,
        exec,
        exec_terminal: false,
        comment,
        keywords: vec![target],
        categories: vec![category],
        visible: true,
        icon_path,
    })
}

fn resolve_shortcut(
    path: &Path,
) -> Result<(String, String, Option<String>), WindowsError> {
    unsafe {
        let link: IShellLinkW =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        let persist: IPersistFile = link.cast()?;
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        persist.Load(PCWSTR(wide.as_ptr()), STGM_READ)?;
        let mut target = [0u16; MAX_PATH as usize];
        let mut args = [0u16; MAX_PATH as usize];
        let mut desc = [0u16; MAX_PATH as usize];
        let _ = link.GetPath(&mut target, std::ptr::null_mut(), 0);
        let _ = link.GetArguments(&mut args);
        let _ = link.GetDescription(&mut desc);
        Ok((
            utf16_z(&target),
            utf16_z(&args),
            Some(utf16_z(&desc)).filter(|s| !s.is_empty()),
        ))
    }
}

fn utf16_z(buf: &[u16]) -> String {
    let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

pub fn category_from_path(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let folder = rel
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("");
    map_start_menu_category(folder)
}

pub fn map_start_menu_category(folder: &str) -> String {
    let lower = folder.to_ascii_lowercase();
    if lower.contains("game") {
        "Game".into()
    } else if lower.contains("develop")
        || lower.contains("programming")
        || lower.contains("visual studio")
    {
        "Development".into()
    } else if lower.contains("office")
        || lower.contains("word")
        || lower.contains("excel")
    {
        "Office".into()
    } else if lower.contains("graphic")
        || lower.contains("photo")
        || lower.contains("design")
    {
        "Graphics".into()
    } else if lower.contains("music") || lower.contains("audio") {
        "Audio".into()
    } else if lower.contains("video") {
        "Video".into()
    } else if lower.contains("network")
        || lower.contains("internet")
        || lower.contains("browser")
    {
        "Network".into()
    } else if lower.contains("accessories") || lower.contains("windows tools") {
        "Utility".into()
    } else if lower.contains("admin") || lower.contains("system") {
        "System".into()
    } else if lower.contains("setting") {
        "Settings".into()
    } else if lower.contains("educat") {
        "Education".into()
    } else {
        "Utility".into()
    }
}

pub fn launch_app(app: &DesktopApp) -> Result<u32, WindowsError> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        CREATE_NEW_CONSOLE, CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW,
    };
    use windows::core::PWSTR;
    let exec = app.exec.clone();
    let (command, args) = if let Some(stripped) = exec.strip_prefix('"') {
        if let Some((exe, rest)) = stripped.split_once('"') {
            (exe.to_string(), rest.trim().to_string())
        } else {
            (exec, String::new())
        }
    } else if let Some((exe, rest)) = exec.split_once(' ') {
        (exe.to_string(), rest.to_string())
    } else {
        (exec, String::new())
    };
    let cmdline = if args.is_empty() {
        format!("\"{command}\"")
    } else {
        format!("\"{command}\" {args}")
    };
    let mut cmdline_wide: Vec<u16> =
        cmdline.encode_utf16().chain(std::iter::once(0)).collect();
    let exe_wide: Vec<u16> =
        command.encode_utf16().chain(std::iter::once(0)).collect();
    let mut startup = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut info = PROCESS_INFORMATION::default();
    unsafe {
        CreateProcessW(
            PCWSTR(exe_wide.as_ptr()),
            Some(PWSTR(cmdline_wide.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_NEW_CONSOLE,
            None,
            None,
            &startup,
            &mut info,
        )?;
        let pid = info.dwProcessId;
        let _ = CloseHandle(info.hProcess);
        let _ = CloseHandle(info.hThread);
        Ok(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_start_menu_folders() {
        assert_eq!(map_start_menu_category("Games"), "Game");
        assert_eq!(map_start_menu_category("Windows Accessories"), "Utility");
        assert_eq!(map_start_menu_category("Administrative Tools"), "System");
        assert_eq!(
            map_start_menu_category("Visual Studio 2022"),
            "Development"
        );
    }
}
