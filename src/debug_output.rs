static mut DEBUG_ENABLED: u8 = 0;
pub fn set(value: u8) {
    unsafe { DEBUG_ENABLED = value; }
}

pub fn disable() {
    unsafe { DEBUG_ENABLED = 0; }
}

pub fn is_enabled(value: u8) -> bool {
    unsafe { DEBUG_ENABLED >= value }
}

pub fn suspended<T>(f: impl FnOnce() -> T) -> T {
    let old = unsafe{ DEBUG_ENABLED };
    disable();
    let res = f();
    if old > 0 {
        unsafe { DEBUG_ENABLED = old; };
    }

    res
}

#[macro_export]
macro_rules! dprint {
    (1 => $($arg:tt)*) => {
        if $crate::debug_output::is_enabled(1) {
            eprint!($($arg)*);
        }
    };
    (2 => $($arg:tt)*) => {
        if $crate::debug_output::is_enabled(2) {
            eprint!($($arg)*);
        }
    };
    (3 => $($arg:tt)*) => {
        if $crate::debug_output::is_enabled(3) {
            eprint!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! dprintln {
    (1 => $($arg:tt)*) => {
        if $crate::debug_output::is_enabled(1) {
            eprintln!($($arg)*);
        }
    };
    (2 => $($arg:tt)*) => {
        if $crate::debug_output::is_enabled(2) {
            eprintln!($($arg)*);
        }
    };
    (3 => $($arg:tt)*) => {
        if $crate::debug_output::is_enabled(3) {
            eprintln!($($arg)*);
        }
    };
}
