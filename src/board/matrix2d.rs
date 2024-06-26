use std::{fmt::Display, vec};

use crate::{
    aliases::{BoardIndex as Idx, BoardIndexOverflow as IdxMath},
    board_size::BoardSize,
    board_pos::BoardPos
};

#[derive(Debug)]
pub struct Matrix2D<T>
where T: 'static + Clone {
    data: Box<[&'static mut [T]]>,
    w: Idx,
    h: Idx,
}

impl<T> Matrix2D<T>
where T: Clone
{
    pub fn new(w: Idx, h: Idx, f: impl Fn() -> T) -> Self {
        let base_vec = vec![f(); w as usize * h as usize];
        let data = Self::split_buffer(w, h, base_vec);
        Matrix2D { data, w, h, }
    }

    pub fn map<R>(self, mut f: impl FnMut(&T) -> R) -> Matrix2D<R>
    where R: Clone
    {
        let mut base_vec = Vec::with_capacity(self.w as usize * self.h as usize);
        for row in self.data.iter() {
            for node in row.iter() {
                base_vec.push(f(node));
            }
        }

        let data = Self::split_buffer(self.w, self.h, base_vec);
        Matrix2D { data, w: self.w, h: self.h }
    }

    fn split_buffer<X>(_width: Idx, height: Idx, base_vec: Vec<X>) -> Box<[&'static mut[X]]>
    where X: Clone + 'static
    {
        let parts: Vec<_> = base_vec.leak().chunks_mut(height as usize).collect();
        parts.into_boxed_slice()
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
        Matrix2DIterator { matrix: self, col: 0, row: 0, start: BoardPos::new(0, 0), size: self.size() }
    }

    pub fn iter_section<'a>(&'a self, start: BoardPos, size: BoardSize) -> Matrix2DIterator<'a, T> {
        Matrix2DIterator { matrix: self, col: start.col(), row: start.row(), start, size }
    }
}

impl<T> Clone for Matrix2D<T>
where T: 'static + Clone {
    fn clone(&self) -> Self {
        let mut base_vec = Vec::with_capacity(self.w as usize * self.h as usize);
        for row in self.data.iter() {
            for node in row.iter() {
                base_vec.push(node.clone());
            }
        }
        
        let data = Self::split_buffer(self.w, self.h, base_vec);
        Matrix2D { data, w: self.w, h: self.h }
    }
}

pub struct Matrix2DIterator<'a, T>
where T: 'static + Clone {
    matrix: &'a Matrix2D<T>,
    col: Idx,
    row: Idx,
    start: BoardPos,
    size: BoardSize,
}

impl<'a, T> Matrix2DIterator<'a, T>
where T: Clone {
    pub fn start(&self) -> BoardPos {
        self.start
    }

    pub fn size(&self) -> BoardSize {
        self.size
    }

}

impl<'a, T> Iterator for Matrix2DIterator<'a, T>
where T: Clone {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row == self.size.height() + self.start.row() {
            return None;
        }

        let pos = BoardPos::new(self.col, self.row);
        let val = self.matrix.at(pos);
        self.col += 1;

        // this is technically branchless, because turning a bool into 1 or 0 is trivial with opimizations turned on
        // and we do this because multiplication and addition is much faster than a branch prediction miss
        self.col *= if self.col == self.size.width() + self.start.col() { 0 } else { 1 };
        self.row += if self.col == 0 { 1 } else { 0 };

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
