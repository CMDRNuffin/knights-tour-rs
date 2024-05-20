use std::collections::VecDeque;

use crate::{aliases::BoardIndex as Idx, board_pos::BoardPos, board_size::BoardSize};

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

    let mut segments = Vec::new();
    let mut queue = VecDeque::from(vec![(0, length)]);

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

pub fn partition_size(size: BoardSize) -> Vec<(BoardPos, BoardSize)> {
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

fn sectors_from_partitions(horizontal: Vec<(Idx, Idx)>, vertical: Vec<(Idx, Idx)>) -> Vec<(BoardPos, BoardSize)> {
    let mut sectors = Vec::new();
    for (y, height) in vertical {
        for (x, width) in horizontal.iter().copied() {
            sectors.push((BoardPos::new(x, y), BoardSize::new(width, height)));
        }
    }

    sectors
}