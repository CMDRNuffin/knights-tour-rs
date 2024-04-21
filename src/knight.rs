use crate::{field_pos::FieldPos, matrix2d::Matrix2D};

#[derive(Clone, Copy)]
pub struct Knight {
    position: FieldPos,
}

impl Knight {
    pub fn new(position: FieldPos) -> Self {
        Knight { position }
    }

    pub fn update_position(&mut self, new_pos: FieldPos) {
        self.position = new_pos;
    }

    pub fn clone_to(&self, new_pos: FieldPos) -> Self {
        Knight { position: new_pos }
    }

    pub fn get_possible_moves(&self, board: &Matrix2D<u32>) -> Vec<FieldPos> {
        let mut moves = Vec::new();
        let mut add_move = |pos| {
            let pos = if let Some(pos) = pos { pos } else { return; };
            if board.is_in_range(pos) && *board.at(pos) == 0 {
                moves.push(pos);
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
}
