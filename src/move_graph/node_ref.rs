use crate::board_pos::BoardPos;

use super::Node;


pub enum NodeRef<'a> {
    Direct(&'a Node),
    Reverse(&'a Node),
}

impl<'a> NodeRef<'a> {
    pub fn pos(&self) -> BoardPos {
        match self {
            Self::Direct(node) => node.pos(),
            Self::Reverse(node) => node.pos(),
        }
    }

    pub fn next(&self) -> Option<BoardPos> {
        match self {
            Self::Direct(node) => node.next(),
            Self::Reverse(node) => node.prev(),
        }
    }

    pub fn prev(&self) -> Option<BoardPos> {
        match self {
            Self::Direct(node) => node.prev(),
            Self::Reverse(node) => node.next(),
        }
    }

    pub fn reverse(self) -> Self {
        match self {
            Self::Direct(node) => Self::Reverse(node),
            Self::Reverse(node) => Self::Direct(node),
        }
    }

    pub fn clone_with_offset(&self, offset: BoardPos) -> Node {
        match self {
            Self::Direct(node) => node.clone_with_offset(offset),
            Self::Reverse(node) => node.clone_with_offset(offset).reverse(),
        }
    }
}