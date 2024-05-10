use std::fmt::Debug;

use crate::{
    aliases::{BoardIndex as Idx, BoardIndexOverflow as IdxMath},
    board::{matrix2d::{Matrix2D, Matrix2DIterator}, Board},
    board_pos::BoardPos,
    dprintln
};

#[derive(Clone, Debug)]
enum MoveGraphData<'a> {
    Direct(Matrix2D<Node>),
    Ref(&'a MoveGraph<'a>),
    ReverseRef(&'a MoveGraph<'a>),
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

impl<'a> MoveGraphData<'a> {
    fn at_mut(&mut self, pos: BoardPos) -> &mut Node {
        match self {
            Self::Direct(matrix) => matrix.at_mut(pos),
            _ => panic!("Cannot mutate a reference to a MoveGraph"),
        }
    }

    fn at(&self, pos: BoardPos) -> NodeRef {
        match self {
            Self::Direct(matrix) => NodeRef::Direct(matrix.at(pos)),
            Self::Ref(graph) => graph.node(pos),
            Self::ReverseRef(graph) => graph.node(pos).reverse(),
        }
    }
}

#[derive(Clone)]
pub struct MoveGraph<'a> {
    width: Idx,
    height: Idx,
    nodes: MoveGraphData<'a>,
}

impl<'a> Debug for MoveGraph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "| ")?;
                let pos = BoardPos::new(x, y);
                let node = self.node(pos);
                if let Some(prev) = node.prev() {
                    write!(f, "{}", prev)?;
                }
                else {
                    write!(f, "  ")?;
                }

                if let Some(next) = node.next() {
                    write!(f, " -> {} ", next)?;
                }
                else {
                    write!(f, "       ")?;
                }
            }
            writeln!(f, "|")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pos: BoardPos,
    edges: Vec<BoardPos>,
    next: Option<BoardPos>,
    prev: Option<BoardPos>,
}

pub enum NodeRef<'a> {
    Direct(&'a Node),
    Reverse(&'a Node),
}

impl<'a> NodeRef<'a> {
    pub fn pos(&self) -> BoardPos {
        match self {
            Self::Direct(node) => node.pos,
            Self::Reverse(node) => node.pos,
        }
    }

    pub fn next(&self) -> Option<BoardPos> {
        match self {
            Self::Direct(node) => node.next,
            Self::Reverse(node) => node.prev,
        }
    }

    pub fn prev(&self) -> Option<BoardPos> {
        match self {
            Self::Direct(node) => node.prev,
            Self::Reverse(node) => node.next,
        }
    }

    pub fn reverse(self) -> Self {
        match self {
            Self::Direct(node) => Self::Reverse(node),
            Self::Reverse(node) => Self::Direct(node),
        }
    }
    
    fn edges(&self) -> &[BoardPos]{
        match *self {
            Self::Direct(node) => &node.edges,
            Self::Reverse(node) => &node.edges,
        }
    }
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

    pub fn reverse(&self) -> Self {
        Self {
            pos: self.pos,
            edges: self.edges.clone(),
            next: self.prev,
            prev: self.next,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    Horizontal,
    Vertical,
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

                *res.nodes.at_mut(BoardPos::new(x, y)) = Node { pos: BoardPos::new(x, y), edges, next: None, prev: None };
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

    pub fn node(&self, pos: BoardPos) -> NodeRef {
        self.nodes.at(pos)
    }

    pub fn node_mut(&mut self, pos: BoardPos) -> &mut Node {
        self.nodes.at_mut(pos)
    }

    pub fn to_board(self) -> Board {
        let mut board = Board::new(self.width, self.height, 0);
        let pos = BoardPos::new(0, 0);
        let mut node = self.node(pos);

        macro_rules! print_move {
            ($index:expr => $prev:expr, $pos:expr, $next:expr) => {
                let prev = $prev.map(|pos| pos.to_string()).unwrap_or_else(|| "".to_string());
                let next = $next.map(|pos| pos.to_string()).unwrap_or_else(|| "".to_string());
                dprintln!("{}: {} ({} -> {})", $index, $pos, prev, next);
            };
        }

        // find first node in the chain (or self.nodes[0].next in case of a cycle)
        while let Some(prev_pos) = node.prev() {
            if prev_pos == pos {
                dprintln!("Cycle detected at {pos}!");
                break;
            }

            dprintln!("Going back one {} -> {}!", pos, prev_pos);
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

        dprintln!("i = {}", i);

        board
    }

    fn new_empty(width: Idx, height: Idx) -> Self {
        let mk_node = || Node { pos: BoardPos::new(0, 0), edges: Vec::new(), next: None, prev: None };
        Self { width, height, nodes: MoveGraphData::Direct(Matrix2D::new(width, height, mk_node)) }
    }

    fn ensure_dimension(&self, other: &Self, dim: impl Fn(&Self) -> Idx, name: &str) {
        if dim(self) != dim(other) {
            panic!("Cannot merge graphs with different {name}: self = {}, other = {}", dim(self), dim(other));
        }
    }

    pub fn combine(self, other: Self, direction: Direction) -> Self {
        let ((width, height), (x_offset, y_offset)) = match direction {
            Direction::Horizontal => {
                self.ensure_dimension(&other, Self::height, "height");
                ((self.width + other.width, self.height), (self.width, 0))
            },
            Direction::Vertical => {
                self.ensure_dimension(&other, Self::width, "width");
                ((self.width, self.height + other.height), (0, self.height))
            },
        };

        let mut res = Self::new_empty(width, height);
        for node in &self.nodes {
            let map = |pos: &BoardPos| BoardPos::new(pos.col(), pos.row());
            let pos = node.pos();
            let new_node = Node {
                pos,
                edges: node.edges().iter().map(map).collect(),
                next: node.next().as_ref().map(map),
                prev: node.prev().as_ref().map(map),
            };

            *res.nodes.at_mut(pos) = new_node;
        }

        for node in &other.nodes {
            let map = |pos: &BoardPos| BoardPos::new(pos.col() + x_offset, pos.row() + y_offset);
            let pos = map(&node.pos());
            let new_node = Node {
                pos,
                edges: node.edges().iter().map(map).collect(),
                next: node.next().as_ref().map(map),
                prev: node.prev().as_ref().map(map),
            };

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
            }
        }
    }
}
