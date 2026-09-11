use crate::windows::error::WindowsError;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC,
    GetDIBits, ReleaseDC,
};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetIconInfo, ICON_BIG, ICONINFO, SMTO_ABORTIFHUNG,
    SendMessageTimeoutW, WM_GETICON,
};

pub fn extract_window_icon_png(hwnd: HWND, dest_dir: &Path) -> Option<PathBuf> {
    let mut result = 0usize;
    let sent = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_GETICON,
            WPARAM(ICON_BIG as usize),
            LPARAM(0),
            SMTO_ABORTIFHUNG,
            50,
            Some(&mut result),
        )
    };
    if sent.0 == 0 || result == 0 {
        return None;
    }
    // WM_GETICON returns a handle owned by the window; do not destroy it.
    let hicon = HICON(result as *mut core::ffi::c_void);
    let png = icon_to_png_bytes(hicon)?;
    write_png_file(dest_dir, &format!("hwnd-{:x}", hwnd.0 as usize), &png).ok()
}

fn icon_to_png_bytes(icon: HICON) -> Option<Vec<u8>> {
    let mut info = ICONINFO::default();
    unsafe { GetIconInfo(icon, &mut info).ok()? };
    let hdc = unsafe { GetDC(None) };
    let mut header = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: 32,
        biHeight: -32,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };
    let mut bmi = BITMAPINFO {
        bmiHeader: header,
        ..Default::default()
    };
    let mut pixels = vec![0u8; 32 * 32 * 4];
    let ok = unsafe {
        GetDIBits(
            hdc,
            info.hbmColor,
            0,
            32,
            Some(pixels.as_mut_ptr().cast()),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };
    unsafe {
        let _ = DeleteObject(info.hbmColor.into());
        let _ = DeleteObject(info.hbmMask.into());
        ReleaseDC(None, hdc);
    }
    if ok == 0 {
        return None;
    }
    bgra_to_rgba_inplace(&mut pixels);
    Some(encode_png_rgba(32, 32, &pixels))
}

pub fn extract_file_icon_png(path: &Path, dest_dir: &Path) -> Option<PathBuf> {
    use windows::Win32::UI::Shell::{
        SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW,
    };
    use windows::core::PCWSTR;
    let wide = wide_path(path);
    let mut info = SHFILEINFOW::default();
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if result == 0 || info.hIcon.is_invalid() {
        return None;
    }
    let png = icon_to_png_bytes(info.hIcon);
    unsafe {
        let _ = DestroyIcon(info.hIcon);
    }
    let png = png?;
    write_png_file(dest_dir, &path.display().to_string(), &png).ok()
}

fn write_png_file(
    dest_dir: &Path,
    key: &str,
    png: &[u8],
) -> Result<PathBuf, WindowsError> {
    fs::create_dir_all(dest_dir)
        .map_err(|e| WindowsError::message(e.to_string()))?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    let name = format!("icon-{:016x}.png", hasher.finish());
    let path = dest_dir.join(name);
    if !path.exists() {
        fs::write(&path, png)
            .map_err(|e| WindowsError::message(e.to_string()))?;
    }
    Ok(path)
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide_extra()
}

trait EncodeWide {
    fn encode_wide_extra(&self) -> Vec<u16>;
}

impl EncodeWide for std::ffi::OsStr {
    fn encode_wide_extra(&self) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        self.encode_wide().chain(std::iter::once(0)).collect()
    }
}

fn bgra_to_rgba_inplace(pixels: &mut [u8]) {
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
}

pub fn encode_png_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut raw =
        Vec::with_capacity((width as usize + 1) * height as usize * 4);
    for row in 0..height as usize {
        raw.push(0);
        let start = row * width as usize * 4;
        let end = start + width as usize * 4;
        raw.extend_from_slice(&rgba[start..end]);
    }
    let zlib = zlib_store(&raw);
    let mut png = Vec::new();
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    write_chunk(&mut png, b"IHDR", &{
        let mut data = Vec::new();
        data.extend_from_slice(&width.to_be_bytes());
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&[8, 6, 0, 0, 0]);
        data
    });
    write_chunk(&mut png, b"IDAT", &zlib);
    write_chunk(&mut png, b"IEND", &[]);
    png
}

fn write_chunk(out: &mut Vec<u8>, ty: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(ty);
    out.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(ty);
    crc_data.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

fn zlib_store(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut remaining = data;
    while !remaining.is_empty() {
        let chunk = remaining.len().min(65535);
        let last = chunk == remaining.len();
        out.push(if last { 1 } else { 0 });
        let n = chunk as u16;
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&(!n).to_le_bytes());
        out.extend_from_slice(&remaining[..chunk]);
        remaining = &remaining[chunk..];
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for byte in data {
        a = (a + *byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = if crc & 1 != 0 { 0xEDB88320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_signature_and_ihdr() {
        let pixels = vec![255u8, 0, 0, 255];
        let png = encode_png_rgba(1, 1, &pixels);
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        assert!(png.len() > 33);
    }
}
