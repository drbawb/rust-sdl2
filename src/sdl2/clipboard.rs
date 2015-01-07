use std::ffi::{c_str_to_bytes, CString};

use SdlResult;
use get_error;

pub use sys::clipboard as ll;

pub fn set_clipboard_text(text: &String) -> SdlResult<()> {
    unsafe {
        let result = text.with_c_str(|buff| {
            ll::SDL_SetClipboardText(buff)
        });

        if result == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }
}

pub fn get_clipboard_text() -> SdlResult<String> {
    let result = unsafe {
        let buf = c_str_to_bytes(ll::SDL_GetClipboardText());
        String::from_utf8_lossy(buf).into_string()
    };

    if result.len() == 0 {
        Err(get_error())
    } else {
        Ok(result)
    }
}

pub fn has_clipboard_text() -> bool {
    unsafe { ll::SDL_HasClipboardText() == 1 }
}

