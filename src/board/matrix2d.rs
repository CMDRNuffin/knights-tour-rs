use std::fmt::Display;

use crate::{args::field_size::FieldSize, board_pos::BoardPos};

pub struct Matrix2D<T>
    where T: Copy {
    data: Box<[Box<[T]>]>,
    w: u16,
    h: u16,
}

impl<T> Matrix2D<T>
    where T: Copy {
    pub fn new(w: u16, h: u16, value: T) -> Self {
        let data = make_slice(w, |_| make_slice(h, |_| value));
        Matrix2D { data, w, h, }
    }

    pub fn at(&self, pos: BoardPos) -> &T {
        &self.data[pos.col() as usize][pos.row() as usize]
    }

    pub fn at_mut(&mut self, pos: BoardPos) -> &mut T {
        &mut self.data[pos.col() as usize][pos.row() as usize]
    }

    pub fn is_in_range(&self, pos: BoardPos) -> bool {
        pos.col() < self.w && pos.row() < self.h
    }

    pub fn size(&self) -> FieldSize {
        FieldSize::new(self.w, self.h)
    }
}

impl<T> Display for Matrix2D<T>
    where T: Display + Copy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max = self.h as u32 * self.w as u32;
        let max_len = max.to_string().len();

        let border = |f: &mut std::fmt::Formatter<'_>| -> std::fmt::Result {
            for _ in 0..self.w {
                write!(f, "+--")?;
                for _ in 0..max_len {
                    write!(f, "-")?;
                }
            }
            writeln!(f, "+")?;
            Ok(())
        };

        let pad = |f: &mut std::fmt::Formatter<'_>, val: T| -> std::fmt::Result {
            let padding = max_len - val.to_string().len();
            for _ in 0..padding { write!(f, " ")?; }
            write!(f, "{val}")?;

            Ok(())
        };

        border(f)?;
        for row in 0..self.h {
            for col in 0..self.w {
                write!(f, "| ")?;
                pad(f, *self.at(BoardPos::new(col, row)))?;
                write!(f, " ")?;
            }
            writeln!(f, "|")?;
            border(f)?;
        }
        Ok(())
    }
}

fn make_slice<T, F>(len: u16, f: F) -> Box<[T]>
    where F: Fn(u16) -> T {
    let mut slice = Vec::<T>::with_capacity(len as usize);
    let mut idx = 0;
    slice.resize_with(len as usize, ||{
        let r = f(idx);
        idx += 1;
        r
    });
    slice.into_boxed_slice()
}
