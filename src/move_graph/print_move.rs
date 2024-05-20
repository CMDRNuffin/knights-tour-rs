#[macro_export]
macro_rules! print_move {
    ($index:expr => $prev:expr, $pos:expr, $next:expr) => {
        let prev = $prev.map(|pos| pos.to_string()).unwrap_or_else(|| "".to_string());
        let next = $next.map(|pos| pos.to_string()).unwrap_or_else(|| "".to_string());
        dprintln!(3 => "{}: {} ({} -> {})", $index, $pos, prev, next);
    };
}