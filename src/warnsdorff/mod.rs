use std::{collections::{HashMap, HashSet}, hash::Hash, path::PathBuf, sync::OnceLock, time::{Duration, Instant}};

use crate::{
    aliases::BoardIndex as Idx, args::{board_size::BoardSize, Args}, board::Board, board_pos::BoardPos, divide_and_conquer::move_graph::{Direction, MoveGraph}, dprint, dprintln, knight::Knight
};

mod move_tracker;
use move_tracker::MoveTracker;

pub fn solve(args: Args) -> Option<(Duration, Board)> {
    let result = solve_internal_impl(args.board_size?.into(), Mode::Basic(args))?;
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

static mut STRETCHED_CACHE: OnceLock<HashMap<(BoardSize, Direction), MoveGraph>> = OnceLock::new();

fn get_stretched_cached<'a>(size: BoardSize, direction: Direction) -> Option<&'a MoveGraph<'a>> {
    let cache = unsafe { STRETCHED_CACHE.get()? };
    cache.get(&(size, direction))
}

pub fn solve_internal<'a>(size: BoardSize, mode: Mode) -> Option<(MoveGraph<'a>, Duration)> {
    solve_internal_impl(size, mode).map(|(graph, duration, _)|(graph, duration))
}

struct SolveParams {
    dead_squares: HashSet<BoardPos>,
    end_point: Option<BoardPos>,
    pos: BoardPos,
    cache: bool,
    direction: Direction,
}

fn parse_mode(mode: &Mode) -> Option<SolveParams> {
    let end_point;
    let mut dead_squares = HashSet::new();
    let pos;
    let cache;
    let mut direction = Direction::Horizontal;
    match mode {
        Mode::Basic(ref args) => {
            end_point = None;
            if !populate_dead_squares(&mut dead_squares, &args) {
                return None;
            }

            pos = args.warnsdorff.as_ref().unwrap().starting_pos.unwrap().into();
            cache = false;
        },
        Mode::Structured(StructureMode::Closed(skip_corner)) => {
            cache = false;
            if *skip_corner {
                dead_squares.insert(BoardPos::new(0, 0));
                pos = BoardPos::new(1, 0);
            } else {
                pos = BoardPos::new(0, 0);
            }

            end_point = Some(pos);
        },
        Mode::Structured(StructureMode::Stretched(dir)) => {
            direction = *dir;
            end_point = if matches!(direction, Direction::Horizontal)  { Some(BoardPos::new(0, 1)) } else { Some(BoardPos::new(1, 0)) };
            cache = true;
            pos = BoardPos::new(0, 0);
        },
        Mode::Freeform /* very small board, no structured/closed tour possible */ => {
            cache = true;
            end_point = None;
            pos = BoardPos::new(0, 0);
        },
    }

    Some(SolveParams {
        dead_squares,
        end_point,
        pos,
        cache,
        direction
    })
}

pub fn solve_internal_impl<'a>(size: BoardSize, mode: Mode) -> Option<(MoveGraph<'a>, Duration, HashSet<BoardPos>)> {
    let SolveParams {
        dead_squares,
        end_point,
        pos: start_pos,
        cache,
        direction
    } = parse_mode(&mode)?;

    if cache {
        if let Some(cached) = get_stretched_cached(size, direction) {
            return Some((MoveGraph::ref_to(cached), Duration::ZERO, HashSet::new()));
        }
    }

    let mut graph = MoveGraph::new(size.width(), size.height());
    *graph.node_mut(start_pos).prev_mut() = Some(start_pos); // mark start as visited and start
    let mut knight = Knight::new(start_pos);

    let predetermined_moves = preconnect_corners(&graph, mode);

    let expected_move_count = (graph.width() * graph.height() - dead_squares.len() as Idx) as usize
        - if end_point.is_some() && end_point == Some(start_pos) { 0 } else { 1 };
    dprintln!("Expected move count: {expected_move_count}.");

    let mut moves = vec![ 0 ];

    let now = Instant::now();
    let mut count: usize = 0;
    let mut move_tracker = MoveTracker::new(expected_move_count);
    move_tracker.push(start_pos);

    while moves.len() <= expected_move_count {
        count += 1;
        let skip = moves.last().copied().unwrap();

        let target = if moves.len() == expected_move_count { end_point } else { None };

        let checker = ReachabilityChecker {
            target,
            end_point,
            dead_squares: &dead_squares,
            graph: &graph,
            start: start_pos,
            predetermined_moves: &predetermined_moves,
            move_to_end_allowed: expected_move_count - moves.len() < 3,
        };
        let reachable = |from, to| checker.reachable(from, to);

        let possible_moves = knight.get_possible_moves(&reachable);

        let next_move = possible_moves.iter()
            .skip(skip as usize)
            .next()
            .copied();

        if let Some(next_move) = next_move {
            moves.push(0);

            let current_node = graph.node_mut(knight.position());
            *current_node.next_mut() = Some(next_move);

            let next_node = graph.node_mut(next_move);
            *next_node.prev_mut() = Some(knight.position());

            knight.update_position(next_move);
            move_tracker.push(next_move);
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

            let current_node = graph.node_mut(knight.position());
            if let Some(prev_pos) = current_node.prev_mut().take(){
                let prev_node = graph.node_mut(prev_pos);
                *prev_node.next_mut() = None;
                knight.update_position(prev_pos);
            }
            else {
                dprintln!("Move #{count}: return from {}", knight.position());
                dprintln!("{graph}");
                dprintln!();

                panic!("No previous move found for {}!", knight.position());
            }

            dprintln!("Move #{count}: return to {}", knight.position());
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
        
            cache.insert((size, direction), graph.clone());
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
        if (i == 0) & !top_left.0 { continue; }

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
        if (i == 0) & !top_left.1 { continue; }
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
    graph: &'a MoveGraph<'a>,
    predetermined_moves: &'a HashMap<BoardPos, Vec<BoardPos>>,
    start: BoardPos,
    move_to_end_allowed: bool
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

        if (from == pos) | (self.start == pos) {
            dprintln!("target square is {} -> false", if from == pos { "current square" } else { "starting square" });
            return false;
        }

        let size = BoardSize::new(self.graph.width(), self.graph.height());
        if self.dead_squares.contains(&pos)
            | (size.width() <= pos.col())
            | (size.height() <= pos.row()) {
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
                .filter(|pos|!is_occupied(*pos) | (*pos == from) | (Some(*pos) == self.end_point))
                .collect();
            let len = next.len();
            if next.contains(&from) {
                dprintln!("predetermined move -> true");
                return true;
            } else if len > 1 {
                dprintln!("unrelated square to the middle of two chained predetermined move {next:?} -> false");
                return false;
            } else if let Some(end_point) = self.end_point {
                if !self.move_to_end_allowed & next.contains(&end_point) {
                    dprintln!("predetermined move to end point -> false");
                    return false;
                }
            }
        }

        if let Some(prev) = self.predetermined_moves.get(&from) {
            let res = prev.iter().all(|pos|is_occupied(*pos));
            const BOOLS: [&str; 2] = ["false", "true"];
            dprintln!("from a predetermined move {prev:?} -> {}", BOOLS[res as usize]);
            res
        }
        else {
            dprintln!("not part of a predetermined move whatsoever -> true");
            true
        }
    }
}

fn populate_dead_squares(dead_squares: &mut HashSet<BoardPos>, args: &Args) -> bool {
    if let Some(ref path) = args.warnsdorff.as_ref().unwrap().board_file {
        populate_dead_squares_from_file(dead_squares, path, args)
    }
    else {
        populate_dead_squares_from_corner_radius(dead_squares, args);
        true
    }
}

fn populate_dead_squares_from_corner_radius(dead_squares: &mut HashSet<BoardPos>, args: &Args) {
    let radius = if let Some(radius) = args.warnsdorff.as_ref().unwrap().corner_radius { radius } else { return };
    let size = args.board_size.unwrap();
    let w = size.width();
    let h = size.height();

    for (i, j) in (0..w).flat_map(|i| (0..h).map(move |j| (i, j))) {
        if radius.is_in_range(BoardPos::new(i, j), size) {
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
