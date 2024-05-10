use std::fmt::Display;

use crate::{
    aliases::{BoardIndex as Idx, BoardIndexOverflow as IdxMath},
    board_size::BoardSize,
    board_pos::BoardPos
};

#[derive(Debug, Clone)]
pub struct Matrix2D<T>
where T: Clone {
    data: Box<[Box<[T]>]>,
    w: Idx,
    h: Idx,
}

impl<T> Matrix2D<T>
where T: Clone
{
    pub fn new(w: Idx, h: Idx, f: impl Fn() -> T) -> Self {
        let data = make_slice(w, &|| make_slice(h, &f));
        Matrix2D { data, w, h, }
    }

    pub fn map<R>(self, mut f: impl FnMut(&T) -> R) -> Matrix2D<R>
    where R: Clone
    {
        let data = self.data.into_iter().map(|col| col.into_iter().map(|node| f(node)).collect()).collect();
        Matrix2D { data, w: self.w, h: self.h }
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

    pub fn size(&self) -> BoardSize {
        BoardSize::new(self.w, self.h)
    }

    pub fn iter(&self) -> Matrix2DIterator<T> {
        Matrix2DIterator { matrix: self, col: 0, row: 0 }
    }
}

pub struct Matrix2DIterator<'a, T>
where T: Clone {
    matrix: &'a Matrix2D<T>,
    col: Idx,
    row: Idx,
}

impl<'a, T> Iterator for Matrix2DIterator<'a, T>
where T: Clone {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row == self.matrix.h {
            return None;
        }

        let pos = BoardPos::new(self.col, self.row);
        let val = self.matrix.at(pos);
        self.col += 1;
        if self.col == self.matrix.w {
            self.col = 0;
            self.row += 1;
        }

        Some(val)
    }
}

impl<'a, T> IntoIterator for &'a Matrix2D<T>
where T: Clone {
    type Item = &'a T;
    type IntoIter = Matrix2DIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> Display for Matrix2D<T>
where T: Display + Copy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max = self.h as IdxMath * self.w as IdxMath;
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

fn make_slice<T, F>(len: Idx, f: &F) -> Box<[T]>
where F: Fn() -> T {
    let mut slice = Vec::<T>::with_capacity(len as usize);
    let mut idx = 0;
    slice.resize_with(len as usize, ||{
        let r = f();
        idx += 1;
        r
    });
    slice.into_boxed_slice()
}
