use crate::{
    board_pos::BoardPos, board_size::BoardSize, move_graph::{
        Direction,
        MoveGraph,
    }, warnsdorff::{
        get_stretched_cached,
        insert_stretched_cache,
    }
};

pub fn get<'a>(direction: Direction, size: BoardSize) -> Option<&'a MoveGraph<'a>> {
    match (direction, size.width(), size.height()) {
        (Direction::Horizontal, 4, 10) | (Direction::Vertical, 10, 4) => Some(get_4_by_10(direction)),
        _ => None
    }
}

fn get_4_by_10<'a>(direction: Direction) -> &'a MoveGraph<'a> {
    let size = if direction == Direction::Horizontal {
        BoardSize::new(4, 10)
    }else {
        BoardSize::new(10, 4)
    };

    if let Some(result) = get_stretched_cached(size, direction) {
        return result
    }

    let mut result = MoveGraph::new(4, 10);
    let mut prev = BoardPos::new(0, 0);
    let sequence = vec![
        (2, 1), (3, 3), (1, 2), (3, 1), (1, 0), (0, 2), (2, 3), (0, 4), (2, 5), (3, 7),
        (2, 9), (0, 8), (1, 6), (3, 5), (1, 4), (0, 6), (1, 8), (3, 9), (2, 7), (1, 5),
        (3, 6), (2, 8), (0, 9), (1, 7), (3, 8), (1, 9), (0, 7), (2, 6), (3, 4), (2, 2),
        (3, 0), (1, 1), (0, 3), (2, 4), (0, 5), (1, 3), (3, 2), (2, 0), (0, 1),
    ];

    for next in sequence {
        let next = BoardPos::new(next.0, next.1);
        *result.node_mut(prev).next_mut() = Some(next);
        *result.node_mut(next).prev_mut() = Some(prev);
        prev = next;
    }

    let flipped_result = result.flip();
    insert_stretched_cache(BoardSize::new(4, 10), Direction::Horizontal, result);
    insert_stretched_cache(BoardSize::new(10, 4), Direction::Vertical, flipped_result);

    get_stretched_cached(size, direction).unwrap()
}