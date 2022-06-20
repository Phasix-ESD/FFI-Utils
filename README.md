# FFI Utils

This is a utility library for helping to create C APIs when you need to manipulate rust objects.

## Installation

Cargo.toml:
```toml
ffi-utils = { git = "https://github.com/Phasix-ESD/FFI-Utils", rev = "1.0.0" }
```

## Paradigm

### Errors

This library assumes that you'll handle errors using a [`GetLastError`](https://docs.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror)-style API for error handling, where the previous error is statically stored per thread. Each Rust function exposed via FFI that can fail should have a way to indicate this to the calling code (eg. returning a null pointer) and the calling code should call your "Get Last Error" function to find out exactly what went wrong. (Note: **do not** call your function `GetLastError` as, on Windows, this will interfere with the system `GetLastError` function that the program has probably loaded, and calling code will call the wrong function.)

Get Last Error example:

```rust
#[no_mangle]
pub extern fn GetMyLibraryError() -> *mut c_char {
    ffi_utils::string_to_ptr("Getting Library Error", get_last_error())
}
```

However, if you stick to using `Result` types and the utility functions in this library to handle them, you'll never have to set the error directly.

### Strings

This library includes utilities to give strings to C code as a C-string. You will also need to include a function to free those strings as C code doesn't know how to use Rust's private allocator.

String free example (that handles null string pointers correctly):

```rust
#[no_mangle]
pub extern fn FreeMyLibraryString(string_ptr: *mut c_char) {
    let _ = ffi_utils::take_string_ownership(string_ptr); // Value is dropped here
}
```

### Booleans

This library also assumes that you'll use a `u8` to represent a `bool` in FFI (0 = false, 1/non-zero = true). There are two utility functions, `bool_to_u8` and `u8_to_bool` to convert between these types.

## Examples:

### Giving a Rust object to C:

The `object_to_ptr` function safely boxes your object and leaks it so that C code can have a pointer to it. You need to have a corresponding free function for C code to call:

```rust
fn create_object() -> MyObject { /* Object creation code */ }

#[no_mangle]
pub extern fn CreateMyObject() -> *mut MyObject {
    ffi_utils::object_to_ptr(create_object())
}

#[no_mangle]
pub extern fn FreeMyObject(my_object_ptr: *mut MyObject) {
    let _ = ffi_utils::take_ownership(my_object_ptr); // Value is dropped here
}
```

If your create function returns a result, you can use `result_to_ptr` instead. This takes a "context" string, which describes what is happening for the purposes of error handling, and will set the last error if the result is an `Err`. It will then return the "error return value", which should be `null_mut()` for something returning a pointer.

```rust
fn create_object() -> Result<MyObject, Box<dyn Error>> { /* Object creation code */ }

#[no_mangle]
pub extern fn CreateMyObject() -> *mut MyObject {
    ffi_utils::result_to_ptr("Creating My Object", null_mut(), create_object())
}

// You'll still need the free function from above!
```

### Using a Rust object that C has "ownership" of

Once you have given a Rust object to C using `object_to_ptr` or `result_to_ptr`, you'll want to write functions that do stuff with it, as C doesn't know how to directly call functions on Rust objects. Here, you'll want to use the `with` function to "borrow" it from C:

```rust
use ffi_utils::*;

#[no_mangle]
pub extern fn MyObject_DoSomething(my_object_ptr: *mut MyObject) -> u8 {
    const CONTEXT: &str = "Doing something";
    bool_to_u8(with(CONTEXT, my_object_ptr, false, |my_object| {
        my_object.do_something();
        true
    }))
}
```

`with` will propagate the return value of the closure given to it. Again, it has an "error return value", which it will return if the pointer is null.

### Exposing a rust (object) function that returns a result

This is similar to the last example, but assume that `do_something()` returns a `Result<i32, Box<dyn Error>>` and `-1` will be used as the "error signalling" return value:

```rust
use ffi_utils::*;

#[no_mangle]
pub extern fn MyObject_DoSomething(my_object_ptr: *mut MyObject) -> i32 {
    const CONTEXT: &str = "Doing something";
    with(CONTEXT, my_object_ptr, -1, |my_object| {
        handle_result(CONTEXT, -1, my_object.do_something())
    })
}
```

Here, `handle_result` returns the result of the operation if it was `Ok`, and returns the "error return value" if it was `Err`. Calling C code should check for the error return value and call your Get Last Error function to find out more details about the error.

There is also a `handle_result!()` macro for when the result type is different to the function return type, and you want to manipulate that result further to get to the return value:

```rust
use ffi_utils::*;

#[no_mangle]
pub extern fn MyObject_DataLength(my_object_ptr: *mut MyObject) -> i32 {
    const CONTEXT: &str = "Getting my object's data length";
    with(CONTEXT, my_object_ptr, -1, |my_object| {
        let slice = handle_result!(CONTEXT, -1, my_object.get_data());
        slice.len() as isize
    })
}
```

### Using a C-String

```rust
use std::os::raw::c_char;
use ffi_utils::*;

#[no_mangle]
pub extern fn FunctionThatTakesString(string_ptr: *mut c_char) {
    const CONTEXT: &str = "Doing something with a string";
    with_string(CONTEXT, string_ptr, -1, |string| {
        // string is a &str, use it here...
    })
}
```

### Returning a C-String

```rust
use std::os::raw::c_char;
use ffi_utils::*;

#[no_mangle]
pub extern fn FunctionThatReturnsString() -> *mut c_char {
    string_to_ptr("Returning a string", "Hello World!")
}
```

There is also `string_result_to_ptr` which will return null if the result provided to it fails. This will indicate to the C code that the last error should be checked.

Remember, calling C code must call your string free function when it is finished with the string! If it wants to hold the string for a long time, it is probably best for it to copy it into some memory that it manages.


