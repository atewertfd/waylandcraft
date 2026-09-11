use crate::windows::error::WindowsError;
use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock,
};

const CF_UNICODETEXT: u32 = 13;

pub fn get_text() -> Result<Option<String>, WindowsError> {
    unsafe {
        if OpenClipboard(None).is_err() {
            return Ok(None);
        }
        let Ok(handle) = GetClipboardData(CF_UNICODETEXT) else {
            let _ = CloseClipboard();
            return Ok(None);
        };
        if handle.is_invalid() {
            let _ = CloseClipboard();
            return Ok(None);
        }
        let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u16;
        let text = if ptr.is_null() {
            None
        } else {
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            Some(String::from_utf16_lossy(slice))
        };
        let _ = GlobalUnlock(HGLOBAL(handle.0));
        let _ = CloseClipboard();
        Ok(text)
    }
}

pub fn set_text(text: &str) -> Result<(), WindowsError> {
    let encoded: Vec<u16> =
        text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        OpenClipboard(None)?;
        EmptyClipboard()?;
        let alloc = GlobalAlloc(
            GMEM_MOVEABLE,
            encoded.len() * std::mem::size_of::<u16>(),
        )?;
        let ptr = GlobalLock(alloc) as *mut u16;
        if !ptr.is_null() {
            std::ptr::copy_nonoverlapping(encoded.as_ptr(), ptr, encoded.len());
            let _ = GlobalUnlock(alloc);
            let _ = SetClipboardData(CF_UNICODETEXT, Some(HANDLE(alloc.0)));
        }
        let _ = CloseClipboard();
    }
    Ok(())
}
