use std::{mem::replace, time::{Duration, Instant}};

use crate::{
    aliases::BoardIndex as Idx,
    board_size::BoardSize,
    args::Args,
    board_pos::BoardPos,
    move_graph::{MoveGraph, Node, Direction},
    dprintln,
    warnsdorff::{self, Mode, StructureMode}
};

pub fn solve<'a>(args: Args) -> Option<(Duration, MoveGraph<'a>)> {
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

    Some((duration, graph))
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
    let min_dimension = size.width().min(size.height());
    let max_dimension = size.width().max(size.height());

    // Case 1: n <= 10 && m <= 10
    if size.width() <= 10 && size.height() <= 10 {
        let structure_mode = match mode {
            SolveQuadrantMode::Closed => {
                if min_dimension == 4 && max_dimension == 5 {
                    return Some(warnsdorff::solve_internal(size.into(), Mode::Freeform)?.0);
                } else {
                    let skip_top_left = (min_dimension % 2 != 0) & (max_dimension % 2 != 0);
                    StructureMode::Closed(skip_top_left)
                }
            },
            SolveQuadrantMode::Stretched(direction) => StructureMode::Stretched(direction),
        };

        return Some(warnsdorff::solve_internal(size.into(), Mode::Structured(structure_mode))?.0);
    }

    let make_closed = |size: BoardSize| -> Option<MoveGraph> { divide_and_conquer_impl(size, SolveQuadrantMode::Closed) };
    let make_stretched = |size: BoardSize, direction: Direction| -> Option<MoveGraph> { divide_and_conquer_impl(size, SolveQuadrantMode::Stretched(direction)) };

    let direction;
    let set_max_dimension: fn(BoardSize, Idx) -> BoardSize;
    let new_size: fn(min: Idx, max: Idx) -> BoardSize;
    if size.width() > size.height() {
        direction = Direction::Horizontal;
        set_max_dimension = BoardSize::with_width;
        new_size = |min, max| BoardSize::new(max, min);
    } else {
        direction = Direction::Vertical;
        set_max_dimension = BoardSize::with_height;
        new_size = |min, max| BoardSize::new(min, max);
    };

    // todo: turn iterative
    // Case 2: n == 3 && m > 10 (guaranteed, because n <= m and we already checked the case where n <= m <= 10)
    if min_dimension == 3 {
        let new_size = set_max_dimension(size, max_dimension - 4);
        let remainder = set_max_dimension(size, 4);
        return Some(
            merge(
                make_closed(new_size)?,
                make_stretched(remainder, direction)?,
                direction));
    }

    let m = split_length(max_dimension); // will be used in both remaining cases

    // Case 3: 4 <= n <= 10 && m > 10
    // Split only in one direction, because the other direction is already small enough
    if (4..=10).contains(&min_dimension) {
        let new_size = set_max_dimension(size, m.0);
        let remainder = set_max_dimension(size, m.1);
        dprintln!("double {direction:?}");
        return Some(
            merge(
                make_closed(new_size)?,
                make_stretched(remainder, direction)?,
                direction));
    }

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

    let make_first = |size: BoardSize| {
        match mode {
            SolveQuadrantMode::Closed => make_closed(size),
            SolveQuadrantMode::Stretched(direction) => make_stretched(size, direction),
        }
    };

    dprintln!("quad: {sizes:?} {direction:?}");
    let res = merge(
        merge(
            make_first(sizes.0)?,
            make_stretched(sizes.1, other_direction)?,
            other_direction
        ), 
        merge(
            make_stretched(sizes.2, direction)?,
            make_stretched(sizes.3, other_direction)?,
            other_direction
        ),
        direction);

    Some(res)
}

fn split_length(length: Idx) -> (Idx, Idx) {
    // split the length into two parts, the first part is half of the length, rounded up, minus one if the remainder would be odd
    // (to make sure the second part is always even, because then we are guaranteed able to make a stretched tour for the second part)
    let half = (length / 4) * 2 + (length % 2);
    (half, length - half)
}

fn merge<'a>(first: MoveGraph<'a>, second: MoveGraph<'a>, direction: Direction) -> MoveGraph<'a> {
    // for the start and end of the second graph, find the possible moves ending on the first graph
    // among those moves, find any one where both target nodes are directly connected by a single move (this can be hardcoded for each direction)
    // connect the target nodes to the corresponding nodes in the second graph

    let (second_start, second_end) = match direction {
        Direction::Horizontal => (BoardPos::new(first.width(), 0), BoardPos::new(first.width(), 1)),
        Direction::Vertical => (BoardPos::new(0, first.height()), BoardPos::new(1, first.height())),
    };

    let (first_end, first_start) = match direction {
        Direction::Horizontal => (BoardPos::new(first.width() - 2, 0), BoardPos::new(first.width() - 1, 2)),
        Direction::Vertical => (BoardPos::new(0, first.height() - 2), BoardPos::new(2, first.height() - 1)),
    };

    let second = if first.node(first_end).next() == Some(first_start) {
        second.reverse()
    }
    else {
        second
    };
    
    let mut merged = first.combine(second, direction);

    let update_node = |node: &mut Node, old_target, new_target|{
        if (node.prev() == old_target) | (old_target.is_none() & (node.prev() == Some(node.pos()))) {
            *node.prev_mut() = Some(new_target);
        }
        else if (node.next() == old_target) | (old_target.is_none() & (node.next() == Some(node.pos()))) {
            *node.next_mut() = Some(new_target);
        }
        else {
            panic!("Invalid node: {:?} [ {:?} -> {:?} ] - {:?} - {:?} [{:?}]", node.pos(), node.prev(), node.next(), old_target, new_target, direction);
        }
    };

    update_node(merged.node_mut(first_start), Some(first_end), second_start);
    update_node(merged.node_mut(first_end), Some(first_start), second_end);
    update_node(merged.node_mut(second_start), None, first_start);
    update_node(merged.node_mut(second_end), None, first_end);

    dprintln!("Merged:");
    dprintln!("{merged:?}");

    merged
}
