use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::os::raw::c_char;
use std::ptr::null_mut;
use crate::error::set_last_error;

pub const fn bool_to_u8(b: bool) -> u8 {
    if b { 1 } else { 0 }
}

pub const fn u8_to_bool(u8: u8) -> bool {
    u8 != 0
}

pub fn object_to_ptr<T>(object: T) -> *mut T {
    Box::into_raw(Box::new(object))
}

pub fn string_to_ptr<S: Into<Vec<u8>>>(context: &'static str, string: S) -> *mut c_char {
    handle_result(context, null_mut(), CString::new(string).map(CString::into_raw))
}

pub fn take_ownership<T>(raw_ptr: *mut T) -> Result<T, &'static str> {
    if raw_ptr.is_null() {
        Err("Invalid pointer")
    } else {
        Ok(*(unsafe { Box::from_raw(raw_ptr) }))
    }
}

pub fn take_string_ownership(string_ptr: *mut c_char) -> Result<CString, &'static str> {
    if string_ptr.is_null() {
        Err("Invalid string pointer")
    } else {
        Ok(unsafe { CString::from_raw(string_ptr) })
    }
}

pub fn handle_result<T, E: Display>(context: &'static str, error_return_value: T, result: Result<T, E>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            set_last_error(context, e);
            error_return_value
        }
    }
}

pub fn result_to_ptr<T, E: Display>(context: &'static str, result: Result<T, E>) -> *mut T {
    handle_result(context, null_mut(), result.map(object_to_ptr))
}

#[macro_export]
macro_rules! handle_result {
    ($context:expr, $error_return_value:expr, $result:expr) => {
        match $result {
            Ok(t) => t,
            Err(e) => {
                set_last_error($context, e);
                return $error_return_value;
            }
        }
    }
}

pub fn string_result_to_ptr<S: Into<Vec<u8>>, E: Display>(context: &'static str, result: Result<S, E>) -> *mut c_char {
    string_to_ptr(context, handle_result!(context, null_mut(), result))
}

pub fn flatten_result<T, E>(result: Result<Result<T, E>, E>) -> Result<T, E> {
    match result {
        Ok(r) => r,
        Err(e) => Err(e),
    }
}

pub fn flatten_mismatched_result<T, E1: Into<Box<dyn Error>>, E2: Into<Box<dyn Error>>>(result: Result<Result<T, E1>, E2>) -> Result<T, Box<dyn Error>> {
    match result {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(e1)) => Err(e1.into()),
        Err(e2) => Err(e2.into()),
    }
}

#[inline(always)]
pub fn with<T, R, F: FnOnce(&mut T) -> R>(context: &'static str, t_ptr: *mut T, error_return_value: R, f: F) -> R {
    if let Some(t) = unsafe { t_ptr.as_mut() } {
        f(t)
    } else {
        set_last_error(context, "Invalid pointer");
        error_return_value
    }
}

#[inline(always)]
pub fn with_str<R, F: FnOnce(&str) -> R>(context: &'static str, c_str: *const c_char, error_return_value: R, f: F) -> R {
    if c_str.is_null() {
        return f("");
    }
    let str = unsafe { CStr::from_ptr(c_str) };
    match str.to_str() {
        Ok(s) => f(s),
        Err(_) => {
            set_last_error(context, "Invalid string pointer");
            error_return_value
        }
    }
}

pub fn safe_index<T>(vector: &Vec<T>, index: usize) -> Result<&T, &'static str> {
    if index < vector.len() {
        Ok(&vector[index])
    } else {
        Err("Index out of bounds")
    }
}
