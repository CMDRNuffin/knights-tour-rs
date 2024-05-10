use crate::board::matrix2d::Matrix2DIterator;

use super::{Node, NodeRef};


pub struct NodesIterator<'a> {
    iter: Matrix2DIterator<'a, Node>,
    is_reversed: bool,
}

impl<'a> NodesIterator<'a> {
    pub fn reverse(self) -> Self {
        Self { iter: self.iter, is_reversed: !self.is_reversed }
    }
}

impl<'a> From<Matrix2DIterator<'a, Node>> for NodesIterator<'a> {
    fn from(iter: Matrix2DIterator<'a, Node>) -> Self {
        Self { iter, is_reversed: false }
    }
}

impl<'a> Iterator for NodesIterator<'a> {
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.iter.next()?;
        if self.is_reversed {
            Some(NodeRef::Reverse(result))
        }
        else {
            Some(NodeRef::Direct(result))
        }
    }
}