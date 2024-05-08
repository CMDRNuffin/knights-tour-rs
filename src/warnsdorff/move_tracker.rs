use std::fmt::Display;

use crate::{board_pos::BoardPos, debug_output};

pub struct MoveTracker{
    data: Vec<BoardPos>
}

impl MoveTracker{
    pub fn new(capacity: usize) -> Self{
        let capacity = if debug_output::is_enabled() { capacity } else { 0 };
        MoveTracker{
            data: Vec::with_capacity(capacity)
        }
    }

    pub fn push(&mut self, pos: BoardPos){
        if debug_output::is_enabled() {
            self.data.push(pos);
        }
    }

    pub fn pop(&mut self) -> Option<BoardPos>{
        self.data.pop()
    }
}

impl Display for MoveTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for pos in &self.data {
            if !first {
                write!(f, " -> ")?;
            } else {
                first = false;
            }

            write!(f, "{}", pos)?;
        }
        Ok(())
    }
}
