use crate::{aliases::BoardIndexOverflow as IdxMath, board_pos::BoardPos, debug_output, dprintln};

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
        let mut possible_moves: Vec<BoardPos> = self.get_possible_moves_impl(reachable).collect();

        const MOVES_AHEAD: u8 = 1;
        possible_moves.sort_by_cached_key(|pos| match self.clone_to(*pos).possible_moves_count(&reachable, MOVES_AHEAD){
            n if n < MOVES_AHEAD as usize => usize::MAX,
            n => n
        });

        possible_moves
    }

    fn get_possible_moves_impl<'a, F>(&'a self, reachable: &'a F) -> PossibleMovesIterator<'a, F>
    where F : Fn(BoardPos, BoardPos) -> bool
    {
        PossibleMovesIterator { knight: *self, reachable, offset: 0 }
    }

    pub fn possible_moves_count(&self, reachable: &impl Fn(BoardPos, BoardPos) -> bool, moves_ahead: u8) -> usize {
        if moves_ahead == 0 { return 0; }

        let moves = debug_output::suspended(||
            self.get_possible_moves_impl(reachable)
        );

        let move_count = if moves_ahead == 1 {
            moves.count()
        } else {
            debug_output::suspended(||
                moves.map(|pos| self.clone_to(pos).possible_moves_count(reachable, moves_ahead - 1))
                    .sum()
            )
        };

        dprintln!(3 => "{} -> {} moves", self.position, move_count);
        move_count
    }
}

struct PossibleMovesIterator<'a, F>
where F: Fn(BoardPos, BoardPos) -> bool
{
    knight: Knight,
    reachable: &'a F,
    offset: i8,
}

impl <'a, F> Iterator for PossibleMovesIterator<'a, F>
where F: Fn(BoardPos, BoardPos) -> bool
{
    type Item = BoardPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= 8 {
            return None;
        }

        // generate sequence of
        //  2,  1
        //  2, -1
        // -2,  1
        // -2, -1
        //  1,  2
        //  1, -2
        // -1,  2
        // -1, -2
        let offset = self.offset as IdxMath;
        let h_neg = 1 - (2 * ((offset / 2) % 2));
        let h_offset = (2 - offset / 4) * h_neg;
        let v_neg = 1 - (2 * (offset % 2));
        let v_offset = (1 + offset / 4) * v_neg;

        self.offset += 1;
        if let Some(pos) = self.knight.position.try_translate(h_offset, v_offset) {
            if (self.reachable)(self.knight.position, pos) {
                return Some(pos);
            }
        }

        self.next()
    }
}