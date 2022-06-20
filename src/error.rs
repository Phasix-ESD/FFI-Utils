use std::fmt::Display;
use std::cell::RefCell;

thread_local! {
    static LAST_ERROR: RefCell<String> = RefCell::new(String::new());
}

pub fn set_last_error<E: Display>(context: &'static str, error: E) {
    LAST_ERROR.with(|last_error| *last_error.borrow_mut() = format!("Error {}: {}", context, error));
}

pub fn get_last_error() -> String {
    LAST_ERROR.with(|it| it.borrow().clone())
}
