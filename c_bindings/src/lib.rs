use std::ffi::CStr;

#[no_mangle]
pub extern "C" fn wasi_chdir(path: *const libc::c_char) -> i32 {
    match wasi_ext_lib::chdir(
        match unsafe {
        CStr::from_ptr(path)
    }.to_str() {
        Ok(s) => s,
        Err(e) => {
            return -1;
        }
    }) {
        Ok(_) => 0,
        Err(e) => {
            -1
        }
    }
}
