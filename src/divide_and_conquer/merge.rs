use std::fmt::{Debug, Display};

use crate::{board_pos::BoardPos, board_size::BoardSize, move_graph::{Direction, MoveGraph, Node}};

pub fn merge<'a, 'b>(board: &'b mut MoveGraph<'a>, pos: BoardPos, latter_size: BoardSize, direction: Direction) {
    // for the start and end of the second graph, find the possible moves ending on the first graph
    // among those moves, find any one where both target nodes are directly connected by a single move (this can be hardcoded for each direction)
    // connect the target nodes to the corresponding nodes in the second graph

    let second_start = pos;
    let second_end = pos + match direction {
        Direction::Horizontal => BoardPos::new(0, 1),
        Direction::Vertical => BoardPos::new(1, 0),
    };

    let (first_end, first_start) = match direction {
        Direction::Horizontal => (pos.translate(-2, 0), pos.translate(-1, 2)),
        Direction::Vertical => (pos.translate(0, -2), pos.translate(2, -1)),
    };

    if board.node(first_end).next() == Some(first_start) {
        board.reverse_section(pos, latter_size);
    }
    
    let update_node = |node: &mut Node, old_target, new_target| -> Result<(), ErrInfo>{
        if (node.prev() == old_target) | (old_target.is_none() & (node.prev() == Some(node.pos()))) {
            *node.prev_mut() = Some(new_target);
        }
        else if (node.next() == old_target) | (old_target.is_none() & (node.next() == Some(node.pos()))) {
            *node.next_mut() = Some(new_target);
        }
        else {
            let node_pos = node.pos();
            let node_prev = node.prev();
            let node_next = node.next();
            return Err(ErrInfo {
                node_pos,
                node_prev,
                node_next,
                old_target,
                new_target,
            });
        }

        Ok(())
    };

    macro_rules! chain {
        ($res:ident = $expr:expr) => {
            if $res.is_ok() {
                $res = $expr;
            }
        };
    }

    let mut res = Ok(());
    chain!(res = update_node(board.node_mut(first_start), Some(first_end), second_start));
    chain!(res = update_node(board.node_mut(first_end), Some(first_start), second_end));
    chain!(res = update_node(board.node_mut(second_start), None, first_start));
    chain!(res = update_node(board.node_mut(second_end), None, first_end));

    if let Err(info) = res {
        eprintln!("pos: {pos:?} latter_size: {latter_size:?}");
        eprintln!("first_start: {first_start} ({first_start:?}), first_end: {first_end} ({first_end:?})");
        eprintln!("second_start: {second_start} ({second_start:?}), second_end: {second_end} ({second_end:?})");
        eprintln!("{board:?}");
        let ErrInfo { node_pos, node_prev, node_next, old_target, new_target } = info;
        panic!(
            concat!(
                "Invalid node: {0} ({0:?}) ",
                "[ {1} ({1:?}) -> {2} ({2:?}) ] ",
                "- {3:?} - {4:?} [{5:?}]"
            ),
            node_pos,
            BPO(node_prev),
            BPO(node_next),
            old_target,
            new_target,
            direction
        );
    }
}

struct ErrInfo {
    node_pos: BoardPos,
    node_prev: Option<BoardPos>,
    node_next: Option<BoardPos>,
    old_target: Option<BoardPos>,
    new_target: BoardPos,
}

/// Display adapter for [Option]&lt;[BoardPos]&gt;
struct BPO(Option<BoardPos>);
impl Display for BPO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(pos) => Display::fmt(&pos, f),
            None => write!(f, "None"),
        }
    }
}

impl Debug for BPO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}
