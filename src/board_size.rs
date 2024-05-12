use std::fmt::Display;

use crate::{aliases::{BoardIndex as Idx, BoardIndexOverflow as IdxMath}, board_pos::BoardPos};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BoardSize {
    width: Idx,
    height: Idx,
}

impl std::fmt::Debug for BoardSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = self.width;
        let h = self.height;

        write!(f, "{w}x{h}")
    }

}

impl BoardSize {
    pub fn width(&self) -> Idx {
        self.width
    }

    pub fn height(&self) -> Idx {
        self.height
    }
    
    pub fn new(w: Idx, h: Idx) -> BoardSize {
        BoardSize { width: w, height: h }
    }
    
    pub fn with_height(self, height: Idx) -> BoardSize {
        BoardSize { height, ..self }
    }

    pub fn with_width(self, width: Idx) -> BoardSize {
        BoardSize { width, ..self }
    }
    
    pub fn area(&self) -> IdxMath {
        self.width as IdxMath * self.height as IdxMath
    }
    
    pub fn fits(&self, pos: BoardPos) -> bool {
        pos.col() < self.width && pos.row() < self.height
    }
}

impl PartialOrd for BoardSize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let cmp_w = self.width.cmp(&other.width);
        let cmp_h = self.height.cmp(&other.height);
        if cmp_w == cmp_h {
            Some(cmp_w)
        }
        else if cmp_w == std::cmp::Ordering::Equal {
            Some(cmp_h)
        }
        else if cmp_h == std::cmp::Ordering::Equal {
            Some(cmp_w)
        } else {
            None
        }
    }
}

impl Display for BoardSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let w = self.width;
        let h = self.height;

        write!(f, "{w}x{h}")
    }
}

impl From<(Idx, Idx)> for BoardSize {
    fn from(value: (Idx, Idx)) -> Self {
        BoardSize { width: value.0, height: value.1 }
    }
}

impl From<BoardSize> for (Idx, Idx) {
    fn from(value: BoardSize) -> Self {
        (value.width, value.height)
    }
}

impl TryFrom<&str> for BoardSize {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut w = None;
        let mut h = None;
        for part in value.split('x') {
            if w.is_none() {
                w = match part.parse::<Idx>() {
                    Ok(val) => Some(val),
                    Err(e) => return Err(e.to_string()),
                };
            }
            else if h.is_none() {
                h = match part.parse::<Idx>() {
                    Ok(val) => Some(val),
                    Err(e) => return Err(e.to_string()),
                };
            }
            else {
                return Err("Expected string of the form <width>x<height> or <length>".into());
            }
        }

        let w = w.unwrap();
        if h.is_none() {
            h = Some(w);
        }
        let h = h.unwrap();

        Ok(BoardSize { width: w, height: h })
    }
}

pub fn parse_board_size(arg: &str) -> Result<BoardSize, String> {
    arg.try_into()
}
