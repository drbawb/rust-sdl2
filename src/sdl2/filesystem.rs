use std::ffi::{c_str_to_bytes, CString};
use SdlResult;
use get_error;

pub use sys::filesystem as ll;

pub fn get_base_path() -> SdlResult<String> {
    let result = unsafe {
        let buf = c_str_to_bytes(ll::SDL_GetBasePath());
        String::from_utf8_lossy(buf);
    };

    if result.len() == 0 {
        Err(get_error())
    } else {
        Ok(result)
    }
}

pub fn get_pref_path(org: &str, app: &str) -> SdlResult<String> {
    let result = unsafe {
        let org_cstr = CString::from_slice(org);
        let app_cstr = CString::from_slice(app);
        let buf = c_str_to_bytes(ll::SDL_GetPrefPath(org_cstr, app_cstr));
        String::from_utf8_lossy(buf).into_string()
    };

    if result.len() == 0 {
        Err(get_error())
    } else {
        Ok(result)
    }
}

