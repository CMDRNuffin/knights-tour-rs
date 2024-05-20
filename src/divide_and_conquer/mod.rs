use std::{mem::{replace, MaybeUninit}, time::{Duration, Instant}};

use crate::{
    aliases::BoardIndex as Idx,
    args::InputArgs,
    board_pos::BoardPos,
    board_size::BoardSize,
    move_graph::{Direction, MoveGraph},
    warnsdorff::{self, Mode, StructureMode}
};

mod merge;
mod partitions;

pub fn solve<'a>(args: InputArgs) -> Option<(Duration, MoveGraph<'a>)> {
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
        divide_and_conquer_impl
    } else {
        divide_and_conquer_open
    };

    let start = Instant::now();

    let graph = solve(size)?;

    let duration = start.elapsed();

    Some((duration, graph))
}

#[derive(Debug, Clone, Copy)]
enum SolveQuadrantMode {
    Closed,
    Stretched(Direction),
}

fn divide_and_conquer_open<'a>(size: BoardSize) -> Option<MoveGraph<'a>> {
    // split the graph into parts
    // solve each part (topmost leftmost as structured closed tour skipping (0,0))
    // merge the parts together
    let mut graph = divide_and_conquer_impl(size)?;
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

fn divide_and_conquer_impl<'a>(size: BoardSize) -> Option<MoveGraph<'a>> {
    let mut graph = MoveGraph::new(size.width(), size.height());

    // todo: parallelize
    let partitions = partitions::partition_size(size);
    for sector in partitions.iter() {
        let mode = match (sector.0.col(), sector.0.row()) {
            (0, 0) => SolveQuadrantMode::Closed,
            (x, y) => SolveQuadrantMode::Stretched(Direction::from_bool(x <= y)),
        };

        divide_and_conquer_impl_board(&mut graph, sector.0, sector.1, mode)?;
    }

    for sector in partitions.iter() {
        let direction = match (sector.0.col(), sector.0.row()) {
            (0, 0) => continue,
            (x, y) => Direction::from_bool(x <= y),
        };

        merge::merge(&mut graph, sector.0, sector.1, direction);
    }

    Some(graph)
}

fn divide_and_conquer_impl_board<'a, 'b>(move_graph: &'b mut MoveGraph<'a>, offset: BoardPos, size: BoardSize, mode: SolveQuadrantMode) -> Option<()> {
    assert!(size.width() <= 10 && size.height() <= 10, "size: {}, should be subdivided", size);

    let solver_mode = match mode {
        SolveQuadrantMode::Closed => {
            let [min_dimension, max_dimension] = minmax(size.width(), size.height());
            match (min_dimension, max_dimension) {
                (3, 4|7|8)|(4, _) => Mode::Freeform,
                (n, m) if (n >= 4) & (m > 4) => Mode::Structured(StructureMode::Closed((n % 2 != 0) & (m % 2 != 0))),
                _ => return None,
            }
        },
        SolveQuadrantMode::Stretched(direction) => Mode::Structured(StructureMode::Stretched(direction)),
    };

    let (graph, _) = warnsdorff::solve_internal(size.into(), solver_mode)?;
    
    move_graph.insert_section(&graph, offset);
    return Some(());
}

/// Order two values in ascending order
fn minmax(a: Idx, b: Idx) -> [Idx; 2] {
    // if a <= b { [a, b] } else { [b, a] }
    // except we do it without branching for performance
    // (doing a tiny bit of math is several orders of magnitude faster than a branch misprediction
    // and this method is called a lot of times in the code with essentially random arguments)
    // I've measured performance and on a 1000 by 1000 board this little trick saves about 10% of run time
    let mut res: [MaybeUninit<Idx>; 2] = [MaybeUninit::uninit(); 2];
    res[(a > b) as usize] = MaybeUninit::new(a);
    res[(a <= b) as usize] = MaybeUninit::new(b);
    unsafe { [res[0].assume_init(), res[1].assume_init()] }
}
