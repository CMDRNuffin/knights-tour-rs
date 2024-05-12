use crate::{board::matrix2d::Matrix2D, board_pos::BoardPos, board_size::BoardSize};

use super::{MoveGraph, Node, NodeRef, NodesIterator};


#[derive(Clone, Debug)]
pub enum MoveGraphData<'a> {
    Direct(Matrix2D<Node>),
    Ref(&'a MoveGraph<'a>),
    ReverseRef(&'a MoveGraph<'a>),
    Section(&'a MoveGraph<'a>, BoardPos, BoardSize),
    ReverseSection(&'a MoveGraph<'a>, BoardPos, BoardSize),
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
            Self::Section(graph, start, size) => graph.section_node(*start, *size, pos),
            Self::ReverseSection(graph, start, size) => graph.section_node(*start, *size, pos).reverse(),
        }
    }

    pub fn iter_section(&'a self, start: BoardPos, size: BoardSize) -> NodesIterator<'a> {
        match self {
            Self::Direct(matrix) => matrix.iter_section(start, size).into(),
            Self::Ref(graph) => graph.nodes.iter_section(start, size),
            Self::ReverseRef(graph) => graph.nodes.iter_section(start, size).reverse(),
            Self::Section(graph, rel_to, section_size) => {
                if section_size > &size {
                    graph.nodes.iter_section(start + *rel_to, size)
                } else {
                    panic!("Section size is smaller than requested size");
                }

            },
            Self::ReverseSection(graph, rel_to, section_size) =>{
                if section_size > &size {
                    graph.nodes.iter_section(start + *rel_to, size).reverse()
                } else {
                    panic!("Section size is smaller than requested size");
                }
            },
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
            MoveGraphData::Section(graph, start, size) => graph.nodes.iter_section(*start, *size),
            MoveGraphData::ReverseSection(graph, start, size) => graph.nodes.iter_section(*start, *size).reverse(),
        }
    }
}
