use crate::board_pos::BoardPos;


#[derive(Clone, Debug)]
pub struct Node {
    pos: BoardPos,
    edges: Vec<BoardPos>,
    next: Option<BoardPos>,
    prev: Option<BoardPos>,
}

impl Node {
    pub fn next(&self) -> Option<BoardPos> {
        self.next
    }

    pub fn prev(&self) -> Option<BoardPos> {
        self.prev
    }

    pub fn pos(&self) -> BoardPos {
        self.pos
    }

    pub fn next_mut(&mut self) -> &mut Option<BoardPos> {
        &mut self.next
    }

    pub fn prev_mut(&mut self) -> &mut Option<BoardPos> {
        &mut self.prev
    }

    pub fn edges(&self) -> &[BoardPos] {
        &self.edges
    }

    pub fn reverse(&self) -> Self {
        Self {
            pos: self.pos,
            edges: self.edges.clone(),
            next: self.prev,
            prev: self.next,
        }
    }

    pub fn reverse_in_place(&mut self) {
        std::mem::swap(&mut self.next, &mut self.prev);
    }

    pub fn new(pos: BoardPos, edges: Vec<BoardPos>) -> Self {
        Node {
            pos,
            edges,
            next: None,
            prev: None,
        }
    }

    pub fn clone_with_offset(&self, offset: BoardPos) -> Self {
        Node {
            pos: self.pos + offset,
            edges: self.edges.iter().map(|pos| *pos + offset).collect(),
            next: self.next.map(|pos| pos + offset),
            prev: self.prev.map(|pos| pos + offset),
        }
    }
}