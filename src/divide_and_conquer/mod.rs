use std::{mem::replace, time::{Duration, Instant}};

use crate::{
    aliases::BoardIndex as Idx, args::Args, board::Board, board_pos::{BoardPos, BPO}, board_size::BoardSize, dprintln, move_graph::{Direction, MoveGraph, Node}, warnsdorff::{self, Mode, StructureMode}
};

pub fn solve(args: Args) -> Option<(Duration, Board)> {
    // algorithm shamelessly stolen from https://www.sciencedirect.com/science/article/pii/S0166218X04003488
    // if live squares % 2 == 1, then we can't have a closed tour

    // trivial chunk sizes:
    // - square with even side length (fill with concentric braided tourneys)
    // - 4 x 2n+4 (fill with braided tourneys)
    // remaining chunks sizes:
    // - 6 x 2n+6 (todo: find a way to generate a closed tour for this)

    // todo: implement divide and conquer algorithm
    // step 1: break up board into manageable rectangular chunks. if we can't create a closed tour make sure we have enough space on the edges to fill the rest with warnsdorff
    // step 2: generate a closed knight's tour for each chunk if possible, and noting start and finish otherwise
    // step 2.5 (optional, if I have time): generate each chunk in parallel
    // step 3: stitch the tours together
    // step 4 (optional, if I have time): apply the obfuscation algorithm
    let size = args.board_size?;
    let solve = if size.width() % 2 == 0 || size.height() % 2 == 0 /* can be a closed tour */ {
        |size|divide_and_conquer_impl(size, SolveQuadrantMode::Closed)
    } else {
        divide_and_conquer_open
    };

    let start = Instant::now();

    let graph = solve(size)?;

    let duration = start.elapsed();

    let board = if args.quiet {
        Board::new(1, 1, 0)
    } else {
        graph.to_board()
    };

    Some((duration, board))
}

enum SolveQuadrantMode {
    Closed,
    Stretched(Direction),
}

fn divide_and_conquer_open<'a>(size: BoardSize) -> Option<MoveGraph<'a>> {
    // split the graph into parts
    // solve each part (topmost leftmost as structured closed tour skipping (0,0))
    // merge the parts together
    let mut graph = divide_and_conquer_impl(size, SolveQuadrantMode::Closed)?;
    // insert move from (0,0) into the tour
    let node = graph.node_mut(BoardPos::new(0, 0));
    *node.next_mut() = Some(BoardPos::new(2, 1));

    let next = graph.node_mut(BoardPos::new(2, 1));
    let prev = replace(next.prev_mut(), Some(BoardPos::new(0, 0)));
    if let Some(prev) = prev {
        let prev = graph.node_mut(prev);
        *prev.next_mut() = None;
    }

    Some(graph)
}

fn divide_and_conquer_impl<'a>(size: BoardSize, mode: SolveQuadrantMode) -> Option<MoveGraph<'a>> {
    let mut graph = MoveGraph::new(size.width(), size.height());
    divide_and_conquer_impl_board(&mut graph, BoardPos::ZERO, size, mode)?;
    Some(graph)
}

fn divide_and_conquer_impl_board<'a, 'b>(move_graph: &'b mut MoveGraph<'a>, offset: BoardPos, size: BoardSize, mode: SolveQuadrantMode) -> Option<()> {
    let min_dimension = size.width().min(size.height());
    let max_dimension = size.width().max(size.height());

    // Case 1: n <= 10 && m <= 10
    if size.width() <= 10 && size.height() <= 10 {
        let solver_mode = match mode {
            SolveQuadrantMode::Closed => {
                match (min_dimension, max_dimension) {
                    (3, 4|7|8)|(4, _) => Mode::Freeform,
                    (n, m) if (n >= 4) & (m > 4) => Mode::Structured(StructureMode::Closed(n % 2 != 0 && m % 2 != 0)),
                    _ => return None,
                }
            },
            SolveQuadrantMode::Stretched(direction) => Mode::Structured(StructureMode::Stretched(direction)),
        };

        let (graph, _) = warnsdorff::solve_internal(size.into(), solver_mode)?;
        
        move_graph.insert_section(&graph, offset);
        return Some(());
    }

    let direction;
    let set_max_dimension: fn(BoardSize, Idx) -> BoardSize;
    let new_size: fn(min: Idx, max: Idx) -> BoardSize;
    let new_pos: fn(Idx, Idx) -> BoardPos;
    if size.width() > size.height() {
        direction = Direction::Horizontal;
        set_max_dimension = BoardSize::with_width;
        new_size = |min, max| BoardSize::new(max, min);
        new_pos = |y, x|BoardPos::new(x, y);
    } else {
        direction = Direction::Vertical;
        set_max_dimension = BoardSize::with_height;
        new_size = |min, max| BoardSize::new(min, max);
        new_pos = BoardPos::new;
    };

    let sectors;

    // todo: turn iterative
    // Case 2: n == 3 && m > 10 (guaranteed, because n <= m and we already checked the case where n <= m <= 10)
    if min_dimension == 3 {
        let second_pos = offset + new_pos(0, max_dimension - 4);
        let new_size = set_max_dimension(size, max_dimension - 4);
        let remainder = set_max_dimension(size, 4);
        sectors = vec![
            (offset, new_size, Direction::Horizontal),
            (second_pos, remainder, direction),
        ];
    } else {
        let m = split_length(max_dimension); // will be used in both remaining cases

        // Case 3: 4 <= n <= 10 && m > 10
        // Split only in one direction, because the other direction is already small enough
        if (4..=10).contains(&min_dimension) {
            let second_pos = offset + new_pos(0, m.0);
            let new_size = set_max_dimension(size, m.0);
            let remainder = set_max_dimension(size, m.1);
            sectors = vec![
                (offset, new_size, Direction::Horizontal),
                (second_pos, remainder, direction),
            ];
        } else {
            // Case 4: n > 10 && m > 10
            // Split in both directions
            let n = split_length(min_dimension);
            let sizes = (
                new_size(n.0, m.0),
                new_size(n.1, m.0),
                new_size(n.0, m.1),
                new_size(n.1, m.1),
            );

            let other_direction = match direction {
                Direction::Horizontal => Direction::Vertical,
                Direction::Vertical => Direction::Horizontal,
            };

            dprintln!("quad: {sizes:?} {direction:?}");

            sectors = vec![
                (offset, sizes.0, Direction::Horizontal),
                (offset + new_pos(n.0, 0), sizes.1, other_direction) /* horizontal* merger of the top* half */,
                (offset + new_pos(0, m.0), sizes.2, direction) /* vertical* merger of both halves together */,
                (offset + new_pos(n.0, m.0), sizes.3, other_direction) /* horizontal* merger of the bottom* half */,
            ];
            // *if the graph is higher than long, the board is split vertically and each half horizontally
        }
    }

    let mut first = true;
    for (offset, size, direction) in sectors {
        let (direction, should_merge) = if first {
            first = false;
            if let SolveQuadrantMode::Stretched(direction) = mode {
                (direction, false)
            } else {
                divide_and_conquer_impl_board(move_graph, offset, size, SolveQuadrantMode::Closed)?;
                continue;
            }
        } else {
            (direction, true)
        };
        
        divide_and_conquer_impl_board(move_graph, offset, size, SolveQuadrantMode::Stretched(direction))?;
        if should_merge {
            merge(move_graph, offset, size, direction);
        }
    }

    Some(())
}

fn split_length(length: Idx) -> (Idx, Idx) {
    // split the length into two parts, the first part is half of the length, rounded up, minus one if the remainder would be odd
    // (to make sure the second part is always even, because then we are guaranteed able to make a stretched tour for the second part)
    let half = (length / 4) * 2 + (length % 2);
    (half, length - half)
}

fn merge<'a, 'b>(board: &'b mut MoveGraph<'a>, pos: BoardPos, latter_size: BoardSize, direction: Direction) {
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