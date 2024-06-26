use std::{fmt::Debug, ops::Not};

use crate::{
    aliases::{BoardIndex as Idx, BoardIndexOverflow as IdxMath}, board::{matrix2d::Matrix2D, Board}, board_pos::BoardPos, board_size::BoardSize, dprintln
};

mod node;
mod node_ref;
mod move_graph_data;
mod nodes_iterator;
mod print_move;
pub use node::Node;
pub use node_ref::NodeRef;
use move_graph_data::MoveGraphData;
pub use nodes_iterator::NodesIterator;

use crate::print_move;

#[derive(Clone)]
pub struct MoveGraph<'a> {
    width: Idx,
    height: Idx,
    nodes: MoveGraphData<'a>,
}

impl<'a> Debug for MoveGraph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_len = BoardPos::new(self.width -1, self.height -1).to_string().len();
        let empty = " ".repeat(max_len);
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "| ")?;
                let pos = BoardPos::new(x, y);
                let node = self.node(pos);
                if let Some(prev) = node.prev() {
                    write!(f, "{: ^max_len$}", prev)?;
                }
                else {
                    write!(f, "{}", empty)?;
                }

                if let Some(next) = node.next() {
                    write!(f, " -> {: ^max_len$} ", next)?;
                }
                else {
                    write!(f, "    {} ", empty)?;
                }
            }
            writeln!(f, "|")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    pub fn from_bool(is_vertical: bool) -> Direction {
        if is_vertical {
            Self::Vertical
        }
        else {
            Self::Horizontal
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Vertical)
    }

    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Horizontal)
    }
}

impl Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.opposite()
    }
}

impl<'a> MoveGraph<'a> {
    pub fn new(width: Idx, height: Idx) -> Self {
        let mut res = Self::new_empty(width, height);

        for y in 0..height {
            for x in 0..width {
                let mut edges = Vec::with_capacity(8);
                for (dx, dy) in (-2..=2 as IdxMath).flat_map(|y|((-2..=2 as IdxMath).map(move |x|(x, y)))) {
                    if dx.abs() + dy.abs() == 3 && !matches!((dx, dy), (0,_)|(_,0)) {
                        let nx = x as IdxMath + dx;
                        let ny = y as IdxMath + dy;
                        if nx >= 0 && nx < width as IdxMath && ny >= 0 && ny < height as IdxMath {
                            edges.push(BoardPos::new(nx as Idx, ny as Idx));
                        }
                    }
                }

                *res.nodes.at_mut(BoardPos::new(x, y)) = Node::new(BoardPos::new(x, y), edges);
            }
        }

        res
    }

    pub fn ref_to(&'a self) -> Self {
        Self { width: self.width, height: self.height, nodes: MoveGraphData::Ref(self) }
    }

    pub fn width(&self) -> Idx {
        self.width
    }

    pub fn height(&self) -> Idx {
        self.height
    }

    pub fn nodes(&'a self) -> NodesIterator<'a> {
        self.nodes.into_iter()
    }

    pub fn node(&self, pos: BoardPos) -> NodeRef {
        self.nodes.at(pos)
    }

    pub fn node_mut(&mut self, pos: BoardPos) -> &mut Node {
        self.nodes.at_mut(pos)
    }

    pub fn to_board(self) -> Board {
        let dead_squares = self.nodes.into_iter().filter_map(|node| {
            let pos = node.pos();
            if node.next().is_none() && node.prev().is_none() {
                Some(pos)
            }
            else {
                None
            }
        }).collect();

        let mut board = Board::new(self.width, self.height, 0).with_dead_squares(dead_squares);
        let pos = BoardPos::new(0, 0);
        let mut node = self.node(pos);

        // find first node in the chain (or self.nodes[0].next in case of a cycle)
        while let Some(prev_pos) = node.prev() {
            if prev_pos == pos {
                dprintln!(2 => "Cycle detected at {pos}!");
                break;
            }

            dprintln!(3 => "Going back one {} -> {}!", pos, prev_pos);
            node = self.node(prev_pos);
        }

        let pos = if let Some(prev_node) = node.prev() {
            node = self.node(prev_node);
            pos
        }
        else {
            node.pos()
        };

        print_move!(1 => node.prev(), node.pos(), node.next());
        *board.at_mut(pos) = 1;

        let mut i = 2;
        while let Some(pos) = node.next() {
            if *board.at_mut(pos) != 0 {
                break;
            }

            *board.at_mut(pos) = i;
            node = self.node(pos);
            print_move!(i => node.prev(), node.pos(), node.next());
            i += 1;
        }

        dprintln!(3 => "i = {}", i);

        board
    }

    fn new_empty(width: Idx, height: Idx) -> Self {
        let mk_node = || Node::new(BoardPos::new(0, 0), Vec::new());
        Self { width, height, nodes: MoveGraphData::Direct(Matrix2D::new(width, height, mk_node)) }
    }

    fn ensure_dimension(&self, other: &Self, dim: impl Fn(&Self) -> Idx, name: &str) {
        if dim(self) != dim(other) {
            panic!("Cannot merge graphs with different {name}: self = {}, other = {}", dim(self), dim(other));
        }
    }

    pub fn combine(self, other: Self, direction: Direction) -> Self {
        let ((width, height), offset) = match direction {
            Direction::Horizontal => {
                self.ensure_dimension(&other, Self::height, "height");
                ((self.width + other.width, self.height), BoardPos::new(self.width, 0))
            },
            Direction::Vertical => {
                self.ensure_dimension(&other, Self::width, "width");
                ((self.width, self.height + other.height), BoardPos::new(0, self.height))
            },
        };

        let mut res = Self::new_empty(width, height);
        for node in &self.nodes {
            let pos = node.pos();
            let new_node = node.clone_with_offset(BoardPos::ZERO);

            *res.nodes.at_mut(pos) = new_node;
        }

        for node in &other.nodes {
            let new_node = node.clone_with_offset(offset);
            let pos = new_node.pos();

            *res.nodes.at_mut(pos) = new_node;
        }

        res
    }
    
    pub fn reverse(self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            nodes: match self.nodes {
                MoveGraphData::Direct(matrix) => MoveGraphData::Direct(matrix.map(|node| node.reverse())),
                MoveGraphData::Ref(data) => MoveGraphData::ReverseRef(data),
                MoveGraphData::ReverseRef(data) => MoveGraphData::Ref(data),
                MoveGraphData::Section(data, start, size) => MoveGraphData::ReverseSection(data, start, size),
                MoveGraphData::ReverseSection(data, start, size) => MoveGraphData::Section(data, start, size),
            }
        }
    }
    
    fn section_node(&self, start: BoardPos, size: BoardSize, pos: BoardPos) -> NodeRef {
        if !size.fits(pos) {
            panic!("Position out of bounds: {} > {}", pos, size);
        }
        self.nodes.at(pos + start)
    }
    
    pub fn insert_section(&mut self, graph: &MoveGraph, offset: BoardPos) {
        for node in &graph.nodes {
            let pos = node.pos() + offset;
            let target_node = self.nodes.at_mut(pos);
            *target_node.next_mut() = node.next().map(|pos| pos + offset);
            *target_node.prev_mut() = node.prev().map(|pos| pos + offset);
        }
    }
    
    pub fn reverse_section(&mut self, pos: BoardPos, size: BoardSize) {
        for col in pos.col()..(pos.col() + size.width()) {
            for row in pos.row()..(pos.row() + size.height()) {
                let pos = BoardPos::new(col, row);
                let target_node = self.nodes.at_mut(pos);
                target_node.reverse_in_place();
            }
        }
    }

    pub fn flip(&self) -> Self {
        let mut res = Self::new(self.height, self.width);
        for node in self.nodes() {
            let res_node = res.node_mut(node.pos().flip());
            *res_node.next_mut() = node.next().map(|p|p.flip());
            *res_node.prev_mut() = node.prev().map(|p|p.flip());
        }

        res
    }
}
