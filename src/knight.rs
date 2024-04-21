use crate::{board_pos::BoardPos, board::Board};

#[derive(Clone, Copy)]
pub struct Knight {
    position: BoardPos,
}

impl Knight {
    pub fn new(position: BoardPos) -> Self {
        Knight { position }
    }

    pub fn update_position(&mut self, new_pos: BoardPos) {
        self.position = new_pos;
    }

    pub fn clone_to(&self, new_pos: BoardPos) -> Self {
        Knight { position: new_pos }
    }

    pub fn get_possible_moves(&self, board: &Board, mut skip: u8) -> Vec<BoardPos> {
        // skipping all moves
        if skip >= 8 { return Vec::new(); }

        let mut moves = Vec::new();
        let mut add_move = |pos| {
            let pos = if let Some(pos) = pos { pos } else { return; };
            if board.is_in_range(pos) && *board.at(pos) == 0 {
                if skip > 0 { skip -= 1; }
                else { moves.push(pos); }
            }
        };

        add_move(self.position.try_translate(2, 1));
        add_move(self.position.try_translate(2, -1));
        add_move(self.position.try_translate(-2, 1));
        add_move(self.position.try_translate(-2, -1));
        add_move(self.position.try_translate(1, 2));
        add_move(self.position.try_translate(1, -2));
        add_move(self.position.try_translate(-1, 2));
        add_move(self.position.try_translate(-1, -2));

        moves
    }

    pub fn possible_moves_count(&self, board: &Board, moves_ahead: u8) -> usize {
        if moves_ahead == 0 { return 0; }

        let moves = self.get_possible_moves(board, 0);
        if moves_ahead == 1 { return moves.len(); }

        moves.iter()
            .map(|pos| self.clone_to(*pos).possible_moves_count(board, moves_ahead - 1))
            .sum()
    }
}
