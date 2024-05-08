use crate::{board_pos::BoardPos, debug_output, dprintln};

#[derive(Clone, Copy)]
pub struct Knight {
    position: BoardPos,
}

impl Knight {
    pub fn new(position: BoardPos) -> Self {
        Knight { position }
    }

    pub fn position(&self) -> BoardPos {
        self.position
    }

    pub fn update_position(&mut self, new_pos: BoardPos) {
        self.position = new_pos;
    }

    pub fn clone_to(&self, new_pos: BoardPos) -> Self {
        Knight { position: new_pos }
    }

    pub fn get_possible_moves(&self, reachable: &impl Fn(BoardPos, BoardPos) -> bool) -> Vec<BoardPos> {
        let mut possible_moves = self.get_possible_moves_impl(reachable);

        const MOVES_AHEAD: u8 = 1;
        possible_moves.sort_by_cached_key(|pos| match self.clone_to(*pos).possible_moves_count(&reachable, MOVES_AHEAD){
            n if n < MOVES_AHEAD as usize => usize::MAX,
            n => n
        });

        possible_moves
    }

    fn get_possible_moves_impl(&self, reachable: &impl Fn(BoardPos, BoardPos) -> bool) -> Vec<BoardPos> {
        let mut moves = Vec::new();
        let mut add_move = |pos| {
            let pos = if let Some(pos) = pos { pos } else { return; };
            if reachable(self.position, pos) {
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

    pub fn possible_moves_count(&self, reachable: &impl Fn(BoardPos, BoardPos) -> bool, moves_ahead: u8) -> usize {
        if moves_ahead == 0 { return 0; }

        let moves = debug_output::suspended(||
            self.get_possible_moves_impl(reachable)
        );
        if moves_ahead == 1 {
            dprintln!("{} -> {} moves", self.position, moves.len());
            return moves.len();
        }

        let move_count = debug_output::suspended(||
            moves.iter()
                .map(|pos| self.clone_to(*pos).possible_moves_count(reachable, moves_ahead - 1))
                .sum()
        );
        dprintln!("{} -> {} moves", self.position, move_count);
        move_count
    }
}
