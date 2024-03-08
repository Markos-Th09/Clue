use std::ffi::{c_char, CString};

use clue_core::Clue;

/// Compiles the given Clue code and returns the compiled code.
/// # Safety
/// The input `code` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn clue_compile(code: *const c_char) -> *const c_char {
	let code = unsafe { std::ffi::CStr::from_ptr(code) }
		.to_string_lossy()
		.into_owned();
	let out = Clue::new().compile_code(code).unwrap();
	CString::new(out).unwrap().into_raw()
}

/// Frees the given string.
/// # Safety
/// The input `s` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn clue_free_string(s: *mut c_char) {
	if s.is_null() {
		return;
	}
	let _ = unsafe { CString::from_raw(s) };
}
