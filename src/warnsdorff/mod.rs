use std::{collections::{HashMap, HashSet}, hash::Hash, path::PathBuf, sync::OnceLock, time::{Duration, Instant}};

use crate::{
    aliases::BoardIndex as Idx, args::{board_size::BoardSize, Args}, board::Board, board_pos::BoardPos, divide_and_conquer::move_graph::{Direction, MoveGraph}, dprint, dprintln, knight::Knight
};

mod move_tracker;
use move_tracker::MoveTracker;

pub fn solve(args: Args) -> Option<(Duration, Board)> {
    let result = solve_internal_impl(args.field.into(), Mode::Basic(args))?;
    Some((result.1, result.0.to_board().with_dead_squares(result.2)))
}

pub enum Mode {
    Basic(Args),
    Structured(StructureMode),
    Freeform,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StructureMode {
    Closed(bool),
    Stretched(Direction),
}

static mut STRETCHED_CACHE: OnceLock<HashMap<(Idx, Idx, Direction), MoveGraph>> = OnceLock::new();

fn get_stretched_cached(size: (Idx, Idx), direction: Direction) -> Option<MoveGraph> {
    let cache = unsafe { STRETCHED_CACHE.get()? };
    cache.get(&(size.0, size.1, direction)).cloned()
}

pub fn solve_internal(size: (Idx, Idx), mode: Mode) -> Option<(MoveGraph, Duration)> {
    solve_internal_impl(size, mode).map(|(graph, duration, _)|(graph, duration))
}

pub fn solve_internal_impl(size: (Idx, Idx), mode: Mode) -> Option<(MoveGraph, Duration, HashSet<BoardPos>)> {
    let mut dead_squares: HashSet<BoardPos> = HashSet::new();
    let mut end_point = Some(BoardPos::new(0, 0));
    let mut pos = BoardPos::new(0, 0);
    let cache;
    let mut direction = Direction::Horizontal;
    match mode {
        Mode::Basic(ref args) => {
            end_point = None;
            if !populate_dead_squares(&mut dead_squares, &args) {
                return None;
            }

            pos = args.starting_pos.into();
            cache = false;
        },
        Mode::Structured(StructureMode::Closed(skip_corner)) => {
            cache = false;
            if skip_corner {
                dead_squares.insert(pos);
                pos = BoardPos::new(1, 0);
                end_point = Some(pos);
            }
        },
        Mode::Structured(StructureMode::Stretched(dir)) => {
            if let Some(cached) = get_stretched_cached(size, dir) {
                return Some((cached, Duration::ZERO, HashSet::new()));
            }

            direction = dir;
            end_point = if matches!(direction, Direction::Horizontal)  { Some(BoardPos::new(0, 1)) } else { Some(BoardPos::new(1, 0)) };
            cache = true;
        },
        Mode::Freeform /* very small board, no structured/closed tour possible */ => {
            cache = true;
            end_point = None;
        },
    }

    let mut graph = MoveGraph::new(size.0, size.1);
    *graph.node_mut(pos).prev_mut() = Some(pos); // mark start as visited and start
    let mut knight = Knight::new(pos);

    let predetermined_moves = preconnect_corners(&graph, mode);

    let expected_move_count = (graph.width() * graph.height() - dead_squares.len() as Idx) as usize
        + if end_point.is_some() { 1 } else { 0 };
    dprintln!("Expected move count: {expected_move_count}.");

    let mut moves = vec![ 0 ];

    let now = Instant::now();
    let mut count: usize = 0;
    let start_pos = pos;
    let mut move_tracker = MoveTracker::new(expected_move_count);
    move_tracker.push(pos);

    while moves.len() < expected_move_count {
        count += 1;
        let skip = moves.last().copied().unwrap();

        let target = if moves.len() == expected_move_count - 1 { end_point } else { None };

        let checker = ReachabilityChecker {
            target,
            end_point,
            dead_squares: &dead_squares,
            graph: &graph,
            start: start_pos,
            predetermined_moves: &predetermined_moves,
        };
        let reachable = |from, to| checker.reachable(from, to);

        let possible_moves = knight.get_possible_moves(&reachable);

        let next_move = possible_moves.iter()
            .skip(skip as usize)
            .next()
            .copied();

        if let Some(next_move) = next_move {
            moves.push(0);

            let current_node = graph.node_mut(pos);
            *current_node.next_mut() = Some(next_move);

            let next_node = graph.node_mut(next_move);
            *next_node.prev_mut() = Some(pos);

            knight.update_position(next_move);
            pos = next_move;
            move_tracker.push(pos);
            dprintln!("Move #{count}:");
            dprintln!("{move_tracker}");
            dprintln!("{graph}");
            dprintln!();
        } else if moves.len() > 1 {
            // undo the last move
            moves.pop();
            move_tracker.pop();
            let prev_move = moves.last_mut().unwrap();
            // skip the last move
            *prev_move += 1;

            let current_node = graph.node_mut(pos);
            if let Some(prev_pos) = current_node.prev_mut().take(){
                let prev_node = graph.node_mut(prev_pos);
                *prev_node.next_mut() = None;
                pos = prev_pos;
                knight.update_position(prev_pos);
            }
            else {
                dprintln!("Move #{count}: return from {pos}");
                dprintln!("{graph}");
                dprintln!();

                panic!("No previous move found for {pos}!");
            }

            dprintln!("Move #{count}: return to {pos}");
            dprintln!("{move_tracker}");
            dprintln!("{graph}");
            dprintln!();
        }
        else {
            println!("No knight's tour possible for this board configuration.");
            break;
        }
    }

    if cache {
        unsafe {
            STRETCHED_CACHE.get_or_init(||HashMap::new());
            let cache = STRETCHED_CACHE.get_mut().unwrap();
        
            cache.insert((size.0, size.1, direction), graph.clone());
        }
    }

    dprintln!("{graph}");

    let duration = now.elapsed();
    Some((graph, duration, dead_squares))
}

fn preconnect_corners(graph: &MoveGraph, mode: Mode) -> HashMap<BoardPos, Vec<BoardPos>> {
    let top_left = match mode {
        Mode::Basic(_) => return HashMap::new(),
        Mode::Structured(StructureMode::Closed(skip_corner)) => {
            (true, !skip_corner)
        },
        _ => {
            (false, false)
        },
    };

    // multipliers for the offsets
    let w = [1,-1, 1];
    let h = [1, 1, -1];

    let mut res: HashMap<BoardPos, Vec<BoardPos>> = HashMap::new();
    let mut add = |from: BoardPos, to: BoardPos| {
        if let Some(ref mut vec) = res.get_mut(&from) {
            vec.push(to);
        }
        else {
            res.insert(from, vec![to]);
        }
    };

    for i in 0..3 /* skip bottom right because don't connect anything to it ever */ {
        // create structured moves
        if i == 0 && !top_left.0 { continue; }

        let pos = BoardPos::new(
            if w[i] == 1 { 0 } else { graph.width() - 1 },
            if h[i] == 1 { 0 } else { graph.height() - 1 }
        );

        let offsets = [ ((2 * w[i], 0), (0, h[i])), ((w[i], 0), (0, 2 * h[i])) ];
        for offset in offsets {
            let next = pos.try_translate(offset.0.0, offset.0.1).unwrap();
            let current = pos.try_translate(offset.1.0, offset.1.1).unwrap();
            add(current, next);
            add(next, current);
        }

        // skip top left because either we're starting there or it's dead
        if i == 0 && !top_left.1 { continue; }
        let prev = pos.try_translate(2 * w[i], h[i]).unwrap();
        add(prev, pos);
        add(pos, prev);
        let prev = pos.try_translate(w[i], 2 * h[i]).unwrap();
        add(prev, pos);
        add(pos, prev);
    }

    dprintln!("Preconnected moves: {res:?}");

    res
}

struct ReachabilityChecker<'a>{
    target: Option<BoardPos>,
    end_point: Option<BoardPos>,
    dead_squares: &'a HashSet<BoardPos>,
    graph: &'a MoveGraph,
    predetermined_moves: &'a HashMap<BoardPos, Vec<BoardPos>>,
    start: BoardPos,
}

impl<'a> ReachabilityChecker<'a> {
    fn reachable(&self, from: BoardPos, pos: BoardPos) -> bool {
        dprint!("Move from {from} to {pos}: ");
        if let Some(target) = self.target {
            dprintln!("trying to reach {target} -> {}", if pos == target { "true" } else { "false" });
            return pos == target;
        }

        if let Some(end_point) = self.end_point {
            if pos == end_point {
                dprintln!("target square is end point -> false");
                return false;
            }
        }

        if from == pos || self.start == pos {
            dprintln!("target square is {} -> false", if from == pos { "current square" } else { "starting square" });
            return false;
        }

        let size = BoardSize::new(self.graph.width(), self.graph.height());
        if self.dead_squares.contains(&pos)
            || size.width() <= pos.col()
            || size.height() <= pos.row() {
            dprintln!("target square is out of bounds -> false");
            return false;
        }
        
        let is_occupied = |pos| {
            self.graph.node(pos).prev().is_some()
        };

        if is_occupied(pos) {
            dprintln!("target square is already visited -> false");
            return false;
        }

        if let Some(next) = self.predetermined_moves.get(&pos) {
            let next: HashSet<BoardPos> = next
                .iter()
                .copied()
                .filter(|pos|!is_occupied(*pos) || *pos == from)
                .collect();
            let len = next.len();
            if next.contains(&from) {
                dprintln!("predetermined move -> true");
                return true;
            } else if len > 1 {
                dprintln!("unrelated square to the middle of two chained predetermined move {next:?} -> false");
                return false;
            }
        }

        if let Some(prev) = self.predetermined_moves.get(&from) {
            let res = prev.iter().all(|pos|is_occupied(*pos));
            dprintln!("from a predetermined move {prev:?} -> {}", if res { "true" } else { "false" });
            res
        }
        else {
            dprintln!("not part of a predetermined move whatsoever -> true");
            true
        }
    }
}

fn populate_dead_squares(dead_squares: &mut HashSet<BoardPos>, args: &Args) -> bool {
    if let Some(ref path) = args.board_file {
        populate_dead_squares_from_file(dead_squares, path, args)
    }
    else {
        populate_dead_squares_from_corner_radius(dead_squares, args);
        true
    }
}

fn populate_dead_squares_from_corner_radius(dead_squares: &mut HashSet<BoardPos>, args: &Args) {
    let radius = if let Some(radius) = args.corner_radius { radius } else { return };
    let w = args.field.width();
    let h = args.field.height();

    for (i, j) in (0..w).flat_map(|i| (0..h).map(move |j| (i, j))) {
        if radius.is_in_range(BoardPos::new(i, j), args.field) {
            dead_squares.insert(BoardPos::new(i, j));
        }
    }
}

fn populate_dead_squares_from_file(
    _dead_squares: &mut HashSet<BoardPos>,
    _path: &PathBuf,
    _args: &Args
) -> bool {
    todo!();
    //let file = File::open(path).expect("Failed to open file");
}
