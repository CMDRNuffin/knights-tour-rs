use std::{collections::HashMap, sync::OnceLock};

use crate::{board_size::BoardSize, move_graph::{Direction, MoveGraph}};

static mut STRETCHED_CACHE: OnceLock<HashMap<(BoardSize, Direction), MoveGraph>> = OnceLock::new();

pub fn get_stretched_cached<'a>(size: BoardSize, direction: Direction) -> Option<&'a MoveGraph<'a>> {
    let cache = unsafe { STRETCHED_CACHE.get()? };
    cache.get(&(size, direction))
}

pub fn insert_stretched_cache(size: BoardSize, direction: Direction, graph: MoveGraph<'static>) {
    let cache = unsafe {
        STRETCHED_CACHE.get_or_init(HashMap::new);
        STRETCHED_CACHE.get_mut().unwrap()
    };
    cache.insert((size, direction), graph);
}