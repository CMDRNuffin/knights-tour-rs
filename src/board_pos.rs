use std::{fmt::Display, ops::{Add, Sub}};
use crate::{aliases::{BoardIndex as Idx, BoardIndexOverflow as IdxMath}, args::board_size::BoardSize};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct BoardPos(Idx, Idx);

impl From<(Idx, Idx)> for BoardPos {
    fn from(value: (Idx, Idx)) -> Self {
        Self(value.0, value.1)
    }
}

impl From<BoardPos> for (usize, usize) {
    fn from(value: BoardPos) -> Self {
        (value.col() as usize, value.row() as usize)
    }
}

impl TryFrom<(usize, usize)> for BoardPos {
    type Error = String;

    fn try_from(value: (usize, usize)) -> Result<Self, Self::Error> {
        if value.0 > u16::MAX as usize || value.1 > u16::MAX as usize {
            return Err("BoardPos: Invalid value.".into());
        }

        Ok(Self(value.0 as Idx, value.1 as Idx))
    }
}

impl From<BoardPos> for (Idx, Idx) {
    fn from(value: BoardPos) -> Self {
        (value.col(), value.row())
    }
}

impl BoardPos {
    pub fn new(col: Idx, row: Idx) -> Self {
        Self(col, row)
    }

    pub fn col(&self) -> Idx {
        self.0
    }

    pub fn row(&self) -> Idx {
        self.1
    }

    pub fn is_knight_move(&self, other: BoardPos) -> bool {
        let col_diff = (self.col() as IdxMath - other.col() as IdxMath).abs();
        let row_diff = (self.row() as IdxMath - other.row() as IdxMath).abs();

        (col_diff == 1 && row_diff == 2) || (col_diff == 2 && row_diff == 1)
    }

    pub fn try_translate(&self, col: IdxMath, row: IdxMath) -> Option<Self> {
        self.try_translate_on_board(col, row, BoardSize::new(Idx::MAX, Idx::MAX))
    }

    pub fn try_translate_on_board(&self, col: IdxMath, row: IdxMath, board_size: BoardSize) -> Option<Self> {
        let (w, h) = (board_size.width() as IdxMath, board_size.height() as IdxMath);
        let (self_col , self_row) = (self.col() as IdxMath, self.row() as IdxMath);
        let in_range = |min: IdxMath, val: IdxMath, max: IdxMath|{
            if val < 0 { return val.abs() <= min } else { return val <= max }
        };

        if !in_range(self_col, col, w) || !in_range(self_row, row, h) { return None; }

        Some(Self(
            if col >= 0 { self.col() + col as Idx } else { self.col() - col.abs() as Idx },
            if row >= 0 { self.row() + row as Idx } else { self.row() - row.abs() as Idx },
        ))
    }

    pub fn if_move(&self, pos: BoardPos) -> Option<BoardPos> {
        if self.is_knight_move(pos) { Some(*self) } else { None }
    }
}

impl Display for BoardPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = alphabetize(self.col() + 1);
        let h = self.row() + 1;

        write!(f, "{w}{h}")
    }
}

fn alphabetize(mut val: Idx) -> String {
    let mut buf = Vec::new();
    while val > 0 {
        val -= 1;
        let rem = val % 26;
        val = val / 26;
        buf.push(C('A') + rem);
    }

    if buf.len() == 0 { buf.push('A'); }

    buf.reverse();
    let mut str = String::with_capacity(buf.len());
    for c in buf {
        str.push(c);
    }

    str
}

#[test]
fn test_alphabetize() {
    assert_eq!("A", alphabetize(1));
    assert_eq!("Z", alphabetize(26));
    assert_eq!("AA", alphabetize(27));
    assert_eq!("AZ", alphabetize(52));
    assert_eq!("BA", alphabetize(53));
    assert_eq!("ZZZ", alphabetize(18278));

    assert_eq!(BoardPos::try_from("A-1").unwrap().col(), 1);
    assert_eq!(BoardPos::try_from("Z-1").unwrap().col(), 26);
    assert_eq!(BoardPos::try_from("AA-1").unwrap().col(), 27);
    assert_eq!(BoardPos::try_from("AZ-1").unwrap().col(), 52);
    assert_eq!(BoardPos::try_from("BA-1").unwrap().col(), 53);
    assert_eq!(BoardPos::try_from("ZZZ-1").unwrap().col(), 18278);
}

struct C(char);
impl Sub<char> for C {
    type Output = Idx;

    fn sub(self, rhs: char) -> Self::Output {
        self.0 as Idx - rhs as Idx
    }
}

impl Sub<Idx> for C {
    type Output = C;

    fn sub(self, rhs: Idx) -> Self::Output {
        C(char::from_u32(self.0 as u32 - rhs as u32).unwrap())
    }
}

impl Add<Idx> for C {
    type Output = char;
    
    fn add(self, rhs: Idx) -> Self::Output {
        char::from_u32(self.0 as u32 + rhs as u32).unwrap()
    }
}

impl TryFrom<&str> for BoardPos {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        const ERR: &str = "Expected a string of the format <COLUMN>[-]<ROW>, whith columns being letters and rows being numeric.";

        let mut col = None;
        let mut row = None;
        for c in value.chars() {
            match (col, row, c) {
                (_, None, c @ 'A'..='Z') => { col = Some(col.unwrap_or(0) * 26 + (C(c) - 'A') + 1); },
                (_, None, c @ 'a'..='z') => { col = Some(col.unwrap_or(0) * 26 + (C(c) - 'a') + 1); },
                (Some(_), None, '-') => { row = Some(0); },
                (Some(_), _, c @ '0'..='9') => { row = Some(row.unwrap_or(0) * 10 + (C(c) - '0')); },
                _ => {
                    return Err(ERR.into());
                }
            }
        }

        let col = col.ok_or(ERR.to_string())?;
        let row = row.ok_or(ERR.to_string())?;
        if row < 1 { return Err("Invalid row: 0".into()); }

        // 0-index, but 1-display
        Ok(BoardPos(col - 1, row - 1))
    }
}

pub fn parse_board_pos(arg: &str) -> Result<BoardPos, String> {
    arg.try_into()
}