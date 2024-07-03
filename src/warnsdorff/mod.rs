use std::{collections::{HashMap, HashSet}, error::Error, io::BufRead, path::PathBuf, time::{Duration, Instant}};

use crate::{
    aliases::BoardIndex as Idx,
    args::{BoardFileType, ImageMode, InputArgs},
    board_pos::BoardPos,
    board_size::BoardSize,
    dprint,
    dprintln,
    knight::Knight,
    move_graph::{Direction, MoveGraph}
};

mod mode;
mod move_tracker;
mod cache;
use move_tracker::MoveTracker;
pub use mode::*;
pub use cache::{get_stretched_cached, insert_stretched_cache};
use image::{Rgba, GenericImageView};

pub fn solve<'a>(args: InputArgs) -> Option<(Duration, MoveGraph<'a>)> {
    let result = solve_internal_impl(args.board_size, Mode::Basic(args))?;
    Some((result.1, result.0))
}

pub fn solve_internal<'a>(size: BoardSize, mode: Mode) -> Option<(MoveGraph<'a>, Duration)> {
    solve_internal_impl(Some(size), mode).map(|(graph, duration, _)|(graph, duration))
}

struct SolveParams {
    dead_squares: HashSet<BoardPos>,
    end_point: Option<BoardPos>,
    pos: BoardPos,
    cache: bool,
    direction: Direction,
    size: BoardSize
}

fn parse_mode(mode: &Mode, mut size: Option<BoardSize>) -> Option<SolveParams> {
    let end_point;
    let mut dead_squares = HashSet::new();
    let pos;
    let cache;
    let mut direction = Direction::Horizontal;
    match mode {
        Mode::Basic(ref args) => {
            end_point = None;
            size = Some(populate_dead_squares(&mut dead_squares, &args)?);

            pos = args.warnsdorff.as_ref().map(|w|w.starting_pos).flatten().unwrap_or(BoardPos::ZERO);
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
        direction,
        size: size?,
    })
}

pub fn solve_internal_impl<'a>(size: Option<BoardSize>, mode: Mode) -> Option<(MoveGraph<'a>, Duration, HashSet<BoardPos>)> {
    let SolveParams {
        dead_squares,
        end_point,
        pos: start_pos,
        cache,
        direction,
        size
    } = parse_mode(&mode, size)?;

    if cache {
        if let Some(cached) = get_stretched_cached(size, direction) {
            return Some((MoveGraph::ref_to(cached), Duration::ZERO, HashSet::new()));
        }

        if let Some(cached) = get_stretched_cached(size.flip(), direction.opposite()) {
            let now = Instant::now();
            let result = cached.flip();
            let duration = now.elapsed();
            insert_stretched_cache(size, direction, result);
            return Some((MoveGraph::ref_to(get_stretched_cached(size, direction).unwrap()), duration, HashSet::new()));
        }
    }

    let mut graph = MoveGraph::new(size.width(), size.height());
    *graph.node_mut(start_pos).prev_mut() = Some(start_pos); // mark start as visited and start
    let mut knight = Knight::new(start_pos);

    let predetermined_moves = preconnect_corners(&graph, &mode, size);

    let expected_move_count = (graph.width() * graph.height() - dead_squares.len() as Idx) as usize
        - if end_point.is_some() && end_point == Some(start_pos) { 0 } else { 1 };
    dprintln!(2 => "Expected move count: {expected_move_count}.");

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
            dprintln!(3 => "Move #{count}:");
            dprintln!(3 => "{move_tracker}");
            dprintln!(3 => "{graph:?}");
            dprintln!(3 => );
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
                dprintln!(3 => "Move #{count}: return from {}", knight.position());
                dprintln!(3 => "{graph:?}");
                dprintln!(3 => );

                panic!("No previous move found for {}!", knight.position());
            }

            dprintln!(3 => "Move #{count}: return to {}", knight.position());
            dprintln!(3 => "{move_tracker}");
            dprintln!(3 => "{graph:?}");
            dprintln!(3 => );
        }
        else {
            println!("No knight's tour possible for this board configuration ({size} {mode}).");
            break;
        }
    }

    if cache {
        insert_stretched_cache(size, direction, graph.clone());
    }

    dprintln!(3 => "{graph:?}");

    let duration = now.elapsed();
    Some((graph, duration, dead_squares))
}

fn preconnect_corners(graph: &MoveGraph, mode: &Mode, size: BoardSize) -> HashMap<BoardPos, HashSet<BoardPos>> {
    let top_left = match mode {
        Mode::Basic(_) => return HashMap::new(),
        Mode::Structured(StructureMode::Closed(skip_corner)) => {
            (true, !skip_corner, None)
        },
        Mode::Structured(StructureMode::Stretched(direction)) => {
            (false, false, Some(direction))
        },
        _ => (false, false, None)
    };

    // multipliers for the offsets
    let w = [1,-1, 1];
    let h = [1, 1, -1];

    let mut res: HashMap<BoardPos, HashSet<BoardPos>> = HashMap::new();
    let mut add = |from: BoardPos, to: BoardPos| {
        if let Some(ref mut vec) = res.get_mut(&from) {
            vec.insert(to);
        }
        else {
            res.insert(from, vec![to].iter().copied().collect());
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

    if let Some(direction) = top_left.2 {
        preconnect_end_point(&mut res, *direction, size);
    }

    dprintln!(3 => "Preconnected moves: {res:?}");

    res
}

fn preconnect_end_point(preconnected_corners: &mut HashMap<BoardPos, HashSet<BoardPos>>, direction: Direction, size: BoardSize) {
    let half_size = size.width().max(size.height()) / 2;
    let half_size = half_size.min(size.width()).min(size.height());

    if half_size < 5 {
        // small board, no need to preconnect the end point
        return;
    }

    let (start, offset) = match direction {
        Direction::Horizontal => (BoardPos::new(0, 1), (2, 1)),
        Direction::Vertical => (BoardPos::new(1, 0), (1, 2)),
    };

    let mut prev = start;
    loop {
        if let Some(next) = prev.try_translate(offset.0, offset.1) {
            preconnected_corners.entry(prev).or_insert_with(HashSet::new).insert(next);
            preconnected_corners.entry(next).or_insert_with(HashSet::new).insert(prev);
            prev = next;
            if prev.col() >= half_size && prev.row() >= half_size {
                break;
            }
        }
        else {
            break;
        }
    }
}

struct ReachabilityChecker<'a>{
    target: Option<BoardPos>,
    end_point: Option<BoardPos>,
    dead_squares: &'a HashSet<BoardPos>,
    graph: &'a MoveGraph<'a>,
    predetermined_moves: &'a HashMap<BoardPos, HashSet<BoardPos>>,
    start: BoardPos,
    move_to_end_allowed: bool
}

impl<'a> ReachabilityChecker<'a> {
    fn reachable(&self, from: BoardPos, pos: BoardPos) -> bool {
        dprint!(3 => "Move from {from} to {pos}: ");
        if let Some(target) = self.target {
            dprintln!(3 => "trying to reach {target} -> {}", if pos == target { "true" } else { "false" });
            return pos == target;
        }

        if let Some(end_point) = self.end_point {
            if pos == end_point {
                dprintln!(3 => "target square is end point -> false");
                return false;
            }
        }

        if (from == pos) | (self.start == pos) {
            dprintln!(3 => "target square is {} -> false", if from == pos { "current square" } else { "starting square" });
            return false;
        }

        let size = BoardSize::new(self.graph.width(), self.graph.height());
        if self.dead_squares.contains(&pos)
            | (size.width() <= pos.col())
            | (size.height() <= pos.row()) {
            dprintln!(3 => "target square is out of bounds -> false");
            return false;
        }
        
        let is_occupied = |pos| {
            self.graph.node(pos).prev().is_some()
        };

        if is_occupied(pos) {
            dprintln!(3 => "target square is already visited -> false");
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
                dprintln!(3 => "predetermined move -> true");
                return true;
            } else if len > 1 {
                dprintln!(3 => "unrelated square to the middle of two chained predetermined move {next:?} -> false");
                return false;
            } else if let Some(end_point) = self.end_point {
                if !self.move_to_end_allowed & next.contains(&end_point) {
                    dprintln!(3 => "predetermined move to end point -> false");
                    return false;
                }
            }
        }

        if let Some(prev) = self.predetermined_moves.get(&from) {
            let res = prev.iter().all(|pos|is_occupied(*pos));
            const BOOLS: [&str; 2] = ["false", "true"];
            dprintln!(3 => "from a predetermined move {prev:?} -> {}", BOOLS[res as usize]);
            res
        }
        else {
            dprintln!(3 => "not part of a predetermined move whatsoever -> true");
            true
        }
    }
}

fn populate_dead_squares(dead_squares: &mut HashSet<BoardPos>, args: &InputArgs) -> Option<BoardSize> {
    if let Some(ref path) = args.warnsdorff.as_ref().map(|w|w.board_file.as_ref()).flatten() {
        populate_dead_squares_from_file(dead_squares, path, args)
    }
    else {
        populate_dead_squares_from_corner_radius(dead_squares, args);
        args.board_size
    }
}

fn populate_dead_squares_from_corner_radius(dead_squares: &mut HashSet<BoardPos>, args: &InputArgs) {
    let radius = if let Some(radius) = args.warnsdorff.as_ref().map(|w|w.corner_radius.as_ref()).flatten() { radius } else { return };
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
    dead_squares: &mut HashSet<BoardPos>,
    path: &PathBuf,
    args: &InputArgs
) -> Option<BoardSize> {
    let warnsdorff = args.warnsdorff.as_ref()?;
    let board_file_format = if let Some(ff) = warnsdorff.board_file_format {
        ff
    } else {
        match path.extension() {
            Some(osstr) if osstr.eq_ignore_ascii_case("txt") => { BoardFileType::Text },
            Some(osstr) if image::ImageFormat::from_extension(osstr).is_some() => { BoardFileType::Image },
            _ => {
                eprintln!("Unknown file type. Please provide the board file type explicitly.");
                return None;
            }
        }
    };

    match board_file_format {
        BoardFileType::Text => populate_dead_squares_from_text_file(dead_squares, path),
        BoardFileType::Image => populate_dead_squares_from_image_file(
            dead_squares,
            path,
            warnsdorff.image_mode.unwrap(),
            warnsdorff.threshold.unwrap_or(128)
        ).map(|s|Some(s)).unwrap_or(None),
    }
}

fn populate_dead_squares_from_text_file(dead_squares: &mut HashSet<BoardPos>, path: &PathBuf) -> Option<BoardSize> {
    let file = std::fs::File::open(path).map(|f|Some(f)).unwrap_or(None)?;
    let mut lines = Vec::new();
    let mut max_len = 0;
    for line in std::io::BufReader::new(file).lines() {
        let str = line.map(|f|Some(f)).unwrap_or(None)?;
        max_len =max_len.max(str.len());
        lines.push(str);
    }

    let size = BoardSize::new(max_len as Idx, lines.len() as Idx);
    let mut row = 0;
    for line in lines {
        let mut col = 0;
        for ch in line.chars() {
            if ch.is_whitespace() || ch.is_control() {
                dead_squares.insert(BoardPos::new(col, row));
            }

            col += 1;
        }

        while (col as usize) < max_len {
            dead_squares.insert(BoardPos::new(col, row));
            col += 1;
        }

        row += 1;
    }

    Some(size)
}

fn populate_dead_squares_from_image_file(dead_squares: &mut HashSet<BoardPos>, path: &PathBuf, image_mode: ImageMode, threshold: u8) -> Result<BoardSize, Box<dyn Error + 'static>> {
    let image = image::open(path)?;

    for (x, y, pixel) in image.pixels() {
        let visible = match image_mode {
            ImageMode::Alpha => pixel.0[3] >= threshold,
            ImageMode::BlackWhite =>  {
                if pixel == Rgba([255, 255, 255, 255]) {
                    false
                } else if pixel == Rgba([0, 0, 0, 255]) {
                    true
                } else {
                    return Err("Only black and white pixels are supported. Try the mode \"luminance\" or \"alpha\" instead.".into());
                }
            },
            ImageMode::Luminance => {
                let [r, g, b, _] = pixel.0;
                let (r, g, b) = (r as u16, g as u16, b as u16);
                let r = r * 30;
                let g = g * 59;
                let b = b * 11;
                let sum = r + g + b;
                let lum = (sum / 100) as u8;
                lum >= threshold
            },
        };

        if visible {
            dead_squares.insert(BoardPos::new(x as Idx, y as Idx));
        }
    }

    Ok(BoardSize::new(image.width() as Idx, image.height() as Idx))
}