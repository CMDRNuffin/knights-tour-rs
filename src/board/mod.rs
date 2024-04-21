mod matrix2d;
use std::fmt::Display;

use matrix2d::Matrix2D;

pub mod corner_radius;
mod corner;
use crate::board_pos::BoardPos;

use self::corner_radius::CornerRadius;

pub struct Board {
    data: CornerRadiusWrapper,
}

impl Board {
    pub fn new(w: u16, h: u16, value: usize, corner_radius: Option<CornerRadius>) -> Self {
        Self { data: CornerRadiusWrapper { data: Matrix2D::new(w, h, value), corner_radius: corner_radius.unwrap_or(CornerRadius::zero()) } }
    }

    pub fn at(&self, pos: BoardPos) -> &usize {
        self.data.data.at(pos)
    }

    pub fn at_mut(&mut self, pos: BoardPos) -> &mut usize {
        self.data.data.at_mut(pos)
    }

    pub fn is_in_range(&self, pos: BoardPos) -> bool {
        self.data.is_in_range(pos)
    }

    pub fn available_fields(&self) -> usize {
        self.data.available_fields()
    }

    pub fn print_errors(&self) {
        let size = self.data.data.size();
        for col in 0..size.width() {
            for row in 0..size.height() {
                let pos = BoardPos::new(col, row);
                if self.data.is_in_range(pos) && *self.at(pos) == 0{
                    println!("{}", pos);
                }
            }
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data.data)
    }
}

struct CornerRadiusWrapper {
    data: Matrix2D<usize>,
    corner_radius: CornerRadius,
}

impl CornerRadiusWrapper {
    fn available_fields(&self) -> usize {
        let size = self.data.size();
        let mut fields = 0;
        for col in 0..size.width() {
            for row in 0..size.height() {
                let pos = BoardPos::new(col, row);
                if self.corner_radius.is_in_range(pos, self.data.size()) {
                    fields += 1;
                }
            }
        }

        fields
    }

    fn is_in_range(&self, pos: BoardPos) -> bool {
        self.data.is_in_range(pos) && self.corner_radius.is_in_range(pos, self.data.size())
    }
}