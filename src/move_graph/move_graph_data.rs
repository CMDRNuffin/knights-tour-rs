use crate::{board::matrix2d::Matrix2D, board_pos::BoardPos};

use super::{MoveGraph, Node, NodeRef, NodesIterator};


#[derive(Clone, Debug)]
pub enum MoveGraphData<'a> {
    Direct(Matrix2D<Node>),
    Ref(&'a MoveGraph<'a>),
    ReverseRef(&'a MoveGraph<'a>),
}

impl<'a> MoveGraphData<'a> {
    pub fn at_mut(&mut self, pos: BoardPos) -> &mut Node {
        match self {
            Self::Direct(matrix) => matrix.at_mut(pos),
            _ => panic!("Cannot mutate a reference to a MoveGraph"),
        }
    }

    pub fn at(&self, pos: BoardPos) -> NodeRef {
        match self {
            Self::Direct(matrix) => NodeRef::Direct(matrix.at(pos)),
            Self::Ref(graph) => graph.node(pos),
            Self::ReverseRef(graph) => graph.node(pos).reverse(),
        }
    }
}

impl<'a> IntoIterator for &'a MoveGraphData<'a> {
    type Item = NodeRef<'a>;
    type IntoIter = NodesIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            MoveGraphData::Direct(matrix) => matrix.into_iter().into(),
            MoveGraphData::Ref(graph) => graph.nodes.into_iter(),
            MoveGraphData::ReverseRef(graph) => graph.nodes.into_iter().reverse(),
        }
    }
}
