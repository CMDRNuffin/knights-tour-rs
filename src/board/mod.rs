pub mod matrix2d;
pub mod corner_radius;
mod corner;

use std::{collections::HashSet, fmt::Display, vec};
use matrix2d::Matrix2D;

use crate::{aliases::BoardIndex as Idx, board_pos::BoardPos};

pub struct Board {
    data: Matrix2D<usize>,
    dead_squares: HashSet<BoardPos>,
}

impl Board {
    pub fn new(w: Idx, h: Idx, value: usize) -> Self {
        Self { data: Matrix2D::new(w, h, ||value), dead_squares: HashSet::new() }
    }

    pub fn at(&self, pos: BoardPos) -> &usize {
        self.data.at(pos)
    }

    pub fn at_mut(&mut self, pos: BoardPos) -> &mut usize {
        self.data.at_mut(pos)
    }

    pub fn print_errors(&self) {
        let size = self.data.size();
        for col in 0..size.width() {
            for row in 0..size.height() {
                let pos = BoardPos::new(col, row);
                if self.data.is_in_range(pos) && *self.at(pos) == 0{
                    println!("{}", pos);
                }
            }
        }
    }
    
    pub fn with_dead_squares(self, dead_squares: HashSet<BoardPos>) -> Board {
        Board { dead_squares, ..self }
    }
}

#[derive(Debug, Clone, Copy)]
enum Neighbor{
    Left,
    _Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.data.size();
        let max = size.area() as usize - self.dead_squares.len();
        let max_len = max.to_string().len();

        let border = |f: &mut std::fmt::Formatter<'_>, row: Idx, is_after: bool| -> std::fmt::Result {
            let corner = |pos, h: Neighbor, v: Neighbor, with_self: bool| {
                let mut neighbors = vec![v, h];
                match (h, v) {
                    (Neighbor::Left, Neighbor::Top) => neighbors.push(Neighbor::TopLeft),
                    (Neighbor::_Right, Neighbor::Top) => neighbors.push(Neighbor::TopRight),
                    (Neighbor::Left, Neighbor::Bottom) => neighbors.push(Neighbor::BottomLeft),
                    (Neighbor::_Right, Neighbor::Bottom) => neighbors.push(Neighbor::BottomRight),
                    _ => (),
                }

                if (!with_self || self.is_alive(pos)) || self.has_alive_neighbor(pos, neighbors) { "+" } else { " " }
            };

            for x in 0..size.width() {
                let pos = BoardPos::new(x, row);
                let vertical = if is_after { Neighbor::Bottom } else { Neighbor::Top };
                let left_corner = corner(pos, Neighbor::Left, vertical, true);
                let border = if self.is_alive(pos) || self.has_alive_neighbor(pos, vec![vertical]) { "-" } else { " " };

                write!(f, "{}{}", left_corner, border.repeat(max_len + 2))?;
            }

            if self.is_alive(BoardPos::new(size.width() - 1, row)) {
                write!(f, "+")?;
            }

            writeln!(f)?;
            Ok(())
        };

        border(f, 0, false)?;
        for row in 0..size.height() {
            for col in 0..size.width() {
                let pos = BoardPos::new(col, row);
                if self.is_alive(pos) || self.has_alive_neighbor(pos, vec![Neighbor::Left]){
                    write!(f, "| ")?;
                }

                if self.is_alive(pos) {
                    // format specifier syntax used:
                    // {
                    // 1  -> the second argument (because 0-based)
                    // :  -> a format specifier follows
                    //' ' -> the padding character (a space)
                    // >  -> right-align the text (alternatives are < for left-align and ^ for center)
                    // 0$ -> the maximum width of the text, passed as an argument
                    // }
                    write!(f, "{1: >0$} ", max_len, *self.at(pos))?;
                }
                else {
                    write!(f, "{1: >0$} ", max_len, ' ')?;
                }
            }
            if self.is_alive(BoardPos::new(size.width() - 1, row)) {
                write!(f, "|")?;
            }
            writeln!(f)?;
            border(f, row, true)?;
        }
        Ok(())
    }
}

impl Board {
    fn has_alive_neighbor(&self, pos: BoardPos, neighbors: Vec<Neighbor>) -> bool {
        let size = self.data.size();
        for neighbor in neighbors {
            let translate = match neighbor {
                Neighbor::Left => (-1, 0),
                Neighbor::_Right => (1, 0),
                Neighbor::Top => (0, -1),
                Neighbor::Bottom => (0, 1),
                Neighbor::TopLeft => (-1, -1),
                Neighbor::TopRight => (1, -1),
                Neighbor::BottomLeft => (-1, 1),
                Neighbor::BottomRight => (1, 1),
            };

            let neighbor = pos.try_translate_on_board(translate.0, translate.1, size);

            if let Some(neighbor) = neighbor {
                if self.is_alive(neighbor) {
                    return true;
                }
            }
        }

        false
    }

    fn is_alive(&self, pos: BoardPos) -> bool {
        !self.dead_squares.contains(&pos)
    }
}