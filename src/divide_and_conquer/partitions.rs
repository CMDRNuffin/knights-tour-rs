use std::collections::VecDeque;

use crate::{aliases::BoardIndex as Idx, board_pos::BoardPos, board_size::BoardSize, move_graph::Direction};

use super::minmax;

fn segment_length(length: Idx, other: Idx) -> Vec<(Idx, Idx)> {
    if other <= 10 {
        if length <= 10 {
            return vec![(0, length)];
        }

        if other == 3 {
            let len = length as usize;
            let remainder = len - 10 % 4;
            let parts = (len - 10 / 4) + if remainder == 0 { 0 } else { 1 };
            let mut res = vec![(0, 0); parts + 1];
            res[0] = (0, length - parts as Idx * 4);
            for i in 1..=parts {
                let prev = res[i - 1];
                res[i] = (prev.0 + prev.1, 4);
            }

            return res;
        }
    }

    let capacity = (length as usize / 6) + 1;
    let mut segments = Vec::with_capacity(capacity);
    let mut queue = VecDeque::from(vec![(0, length)]);
    queue.reserve(capacity);

    while let Some((offset, length)) = queue.pop_front() {
        let (first, second) = split_length(length);
        let mut consume = |rel_offset, val| {
            if val > 10 {
                queue.push_back((offset + rel_offset, val));
            } else {
                segments.push((offset + rel_offset, val));
            }
        };

        consume(0, first);
        consume(first, second);
    }

    segments
}

pub fn split_length(length: Idx) -> (Idx, Idx) {
    // split the length into two parts, the first part is half of the length, rounded up, minus one if the remainder would be odd
    // (to make sure the second part is always even, because then we are guaranteed able to make a stretched tour for the second part)
    let half = (length / 4) * 2 + (length % 2);
    (half, length - half)
}

pub fn partition_size(size: BoardSize) -> Vec<(BoardPos, BoardSize, Direction)> {
    let (width, height) = (size.width(), size.height());
    let horizontal = segment_length(width, height);
    let vertical = if width == height {
         // optimization: if the board is square, the vertical partitions are the same as the horizontal partitions,
         // no need to calculate them again
        horizontal.clone()
    } else {
        segment_length(height, width)
    };

    sectors_from_partitions(horizontal, vertical)
}

fn sectors_from_partitions(horizontal: Vec<(Idx, Idx)>, vertical: Vec<(Idx, Idx)>) -> Vec<(BoardPos, BoardSize, Direction)> {
    let mut sectors = Vec::with_capacity(horizontal.len() * vertical.len() * 2);
    for (y, height) in vertical {
        for (x, width) in horizontal.iter().copied() {
            partition_sector_further(&mut sectors, BoardPos::new(x, y), BoardSize::new(width, height));
        }
    }

    sectors
}

fn partition_sector_further(sectors: &mut Vec<(BoardPos, BoardSize, Direction)>, pos: BoardPos, size: BoardSize) {
    let closed = pos == BoardPos::ZERO;

    if closed {
        partition_closed_sector(sectors, pos, size);
    } else {
        partition_open_sector(sectors, pos, size);
    }
}

fn partition_closed_sector(sectors: &mut Vec<(BoardPos, BoardSize, Direction)>, pos: BoardPos, size: BoardSize) {
    type Sectors<'a> = &'a mut Vec<(BoardPos, BoardSize, Direction)>;

    // redefine vec! macro to push elements to a mutable reference instead of allocating a new vector
    macro_rules! vec { ($sectors:expr => $($x:expr),* $(,)?) => { $($sectors.push($x));* }; }

    let [short_side, long_side] = minmax(size.width(), size.height());
    let new_size: fn(Idx, Idx) -> BoardSize;
    let new_pos: fn(Idx, Idx) -> BoardPos;
    let merge_direction;
    if short_side == size.width() {
        new_size = BoardSize::new;
        new_pos = BoardPos::new;
        merge_direction = Direction::Vertical;
    } else {
        new_size = |s, l| BoardSize::new(l, s);
        new_pos = |s, l| BoardPos::new(l, s);
        merge_direction = Direction::Horizontal;
    };

    let make_sector = |sectors: Sectors, short, long_segment1, long_segment2| {
        vec![sectors =>
            (pos, new_size(short, long_segment1), pos.merge_direction()),
            (pos + new_pos(0, long_segment1), new_size(short, long_segment2), merge_direction),
        ];
    };

    match (short_side, long_side) {
        (5, 10) => make_sector(sectors, 5, 6, 4),
        (5, 9) => make_sector(sectors, 5, 5, 4),
        (7, 9) => make_sector(sectors, 7, 5, 4),
        (_, _) => vec![sectors => (pos, size, pos.merge_direction())],
    };
}

fn partition_open_sector(sectors: &mut Vec<(BoardPos, BoardSize, Direction)>, pos: BoardPos, size: BoardSize) {
    type Sectors<'a> = &'a mut Vec<(BoardPos, BoardSize, Direction)>;

    // redefine vec! macro to push elements to a mutable reference instead of allocating a new vector
    macro_rules! vec { ($sectors:expr => $($x:expr),* $(,)?) => { $($sectors.push($x));* }; }

    let new_size: fn(Idx, Idx) -> BoardSize;
    let new_pos: fn(Idx, Idx) -> BoardPos;
    let merge_direction = pos.merge_direction();
    let merge_axis;
    let non_merge_axis;
    if merge_direction.is_horizontal() {
        new_size = |s, l| BoardSize::new(l, s);
        new_pos = |s, l| BoardPos::new(l, s);
        merge_axis = size.width();
        non_merge_axis = size.height();
    } else {
        new_size = BoardSize::new;
        new_pos = BoardPos::new;
        merge_axis = size.height();
        non_merge_axis = size.width();
    };

    let make_sector = |sectors: Sectors, non_merge_axis, merge_axis_1, merge_axis_2| {
        vec![sectors =>
            (pos, new_size(non_merge_axis, merge_axis_1), merge_direction),
            (pos + new_pos(0, merge_axis_1), new_size(non_merge_axis, merge_axis_2), merge_direction),
        ];
    };

    let make_sector_2 = |sectors: Sectors, non_merge_axis, merge_axis_1, merge_axis_2, merge_axis_3| {
        vec![sectors =>
            (pos, new_size(non_merge_axis, merge_axis_1), merge_direction),
            (pos + new_pos(0, merge_axis_1), new_size(non_merge_axis, merge_axis_2), merge_direction),
            (pos + new_pos(0, merge_axis_1 + merge_axis_2), new_size(non_merge_axis, merge_axis_3), merge_direction),
        ];
    };

    // warnsdorff's rule is still exceedingly slow for some particular combinations of board size and desired merge direction
    // (e.g. 10x8 with horizontal merge direction)
    // so we further partition the sectors to make them more manageable
    // partitions stolen from https://csie.ntnu.edu.tw/~linss/knighttours/bases.html
    match (non_merge_axis, merge_axis) {
        (5, 8) => make_sector(sectors, 5, 4, 4),
        (5, 10) => make_sector(sectors, 5, 6, 4),
        (6, 8) => make_sector(sectors, 6, 4, 4),
        (6, 10) => make_sector(sectors, 6, 6, 4),
        (7, 8) => make_sector(sectors, 7, 4, 4),
        (7, 10) => make_sector(sectors, 7, 6, 4),
        (8, 6) => make_sector(sectors, 8, 3, 3),
        (8, 8) => make_sector(sectors, 8, 4, 4),
        (8, 10) => make_sector_2(sectors, 8, 3, 3, 4),
        (9, 8) => make_sector(sectors, 9, 4, 4),
        (9, 10) => make_sector(sectors, 9, 6, 4),
        (10, 6) => make_sector(sectors, 10, 3, 3),
        (10, 8) => make_sector(sectors, 10, 4, 4),
        (10, 10) => make_sector_2(sectors, 10, 4, 3, 3),
        (_, _) => vec![sectors => (pos, size, pos.merge_direction())],
    };
}