static mut DEBUG_ENABLED: bool = false;
pub fn enable() {
    unsafe { DEBUG_ENABLED = true; }
}

pub fn disable() {
    unsafe { DEBUG_ENABLED = false; }
}

pub fn is_enabled() -> bool {
    unsafe { DEBUG_ENABLED }
}

pub fn suspended<T>(f: impl FnOnce() -> T) -> T {
    let old = is_enabled();
    disable();
    let res = f();
    if old {
        enable();
    }

    res
}

#[macro_export]
macro_rules! dprint {
    ($($arg:tt)*) => {
        if $crate::debug_output::is_enabled() {
            eprint!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! dprintln {
    ($($arg:tt)*) => {
        if $crate::debug_output::is_enabled() {
            eprintln!($($arg)*);
        }
    };
}