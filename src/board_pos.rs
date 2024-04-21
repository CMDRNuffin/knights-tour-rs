use std::{fmt::Display, ops::{Add, Sub}};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BoardPos {
    col: u16,
    row: u16,
}

impl BoardPos {
    pub fn new(col: u16, row: u16) -> Self {
        Self {
            row,
            col,
        }
    }

    pub fn col(&self) -> u16 {
        self.col
    }

    pub fn row(&self) -> u16 {
        self.row
    }

    pub fn try_translate(&self, col: i16, row: i16) -> Option<Self> {
        if col < 0 && self.col < col.abs() as u16 { return None; }
        if row < 0 && self.row < row.abs() as u16 { return None; }
        if col > 0 && self.col > u16::MAX - col as u16 { return None; }
        if row > 0 && self.row > u16::MAX - row as u16 { return None; }

        Some(Self {
            col: if col >= 0 { self.col + col as u16 } else { self.col - col.abs() as u16 },
            row: if row >= 0 { self.row + row as u16 } else { self.row - row.abs() as u16 },
        })
    }
}

impl Display for BoardPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = alphabetize(self.col + 1);
        let h = self.row + 1;

        write!(f, "{w}-{h}")
    }
}

fn alphabetize(mut val: u16) -> String {
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

    assert_eq!(BoardPos::try_from("A-1").unwrap().col, 1);
    assert_eq!(BoardPos::try_from("Z-1").unwrap().col, 26);
    assert_eq!(BoardPos::try_from("AA-1").unwrap().col, 27);
    assert_eq!(BoardPos::try_from("AZ-1").unwrap().col, 52);
    assert_eq!(BoardPos::try_from("BA-1").unwrap().col, 53);
    assert_eq!(BoardPos::try_from("ZZZ-1").unwrap().col, 18278);
}

struct C(char);
impl Sub<char> for C {
    type Output = u16;

    fn sub(self, rhs: char) -> Self::Output {
        self.0 as u16 - rhs as u16
    }
}

impl Sub<u16> for C {
    type Output = C;

    fn sub(self, rhs: u16) -> Self::Output {
        C(char::from_u32(self.0 as u32 - rhs as u32).unwrap())
    }
}

impl Add<u16> for C {
    type Output = char;
    
    fn add(self, rhs: u16) -> Self::Output {
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
        Ok(BoardPos{ col: col - 1, row: row - 1 })
    }
}

pub fn parse_field_pos(arg: &str) -> Result<BoardPos, String> {
    arg.try_into()
}