use std::convert::TryFrom;

use crate::{aliases::{BoardIndex as Idx, BoardIndexOverflow as IdxMath}, board_size::BoardSize, board_pos::BoardPos};

pub use super::corner::Corner;

#[derive(Clone, Copy, Debug)]
pub struct CornerRadius {
    top_left: Corner,
    top_right: Corner,
    bottom_right: Corner,
    bottom_left: Corner,
}

impl CornerRadius {
    pub fn is_in_range (&self, pos: BoardPos, size: BoardSize) -> bool {
        if pos.col() > size.width() || pos.row() > size.height() {
            return false;
        }

        let (w, h) = (size.width() as IdxMath, size.height() as IdxMath);
        let square = |v| v * v;

        let is_in_corner = |ellipsis_size: Corner, point: BoardPos, sector: u8| {
            if ellipsis_size.horizontal() == 0 || ellipsis_size.vertical() == 0 {
                return false;
            }

            // center of the ellipsis is width -1 & height -1 away from the appropriate corner based on the sector
            let (e_w, e_h) = (ellipsis_size.horizontal() as IdxMath, ellipsis_size.vertical() as IdxMath);
            let center = match sector {
                0 => (e_w - 1, e_h - 1),
                1 => (w - e_w, e_h - 1),
                2 => (w - e_w, h - e_h),
                3 => (e_w - 1, h - e_h),
                _ => unreachable!(),
            };

            let point = (point.col() as IdxMath, point.row() as IdxMath);

            // ellipsis calculation (scale y axis to x axis for the ellipsis and the point, then compare the distance via pythagoras)
            // p = point, c = center, s = size
            // p' = p - c                    -> translate so center of the ellipsis is (0,0)
            // p'y = p'y * sy / sx           -> scale y axis to x axis (turning the ellipsis into a circle)
            // sqrt(p'x^2 + (p'y)^2) > sx    -> compare distance to the size of the now circle (square both sides to avoid sqrt)
            // sectors: 0 - top left, 1 - top right, 2 - bottom right, 3 - bottom left
            // if the point is farther from the corner than the ellipsis, we assume it's inside. otherwise, we have to apply the calculation above
            let is_in_corner = match sector {
                0 => point.0 < center.0 && point.1 < center.1,
                1 => point.0 > center.0 && point.1 < center.1,
                2 => point.0 > center.0 && point.1 > center.1,
                3 => point.0 < center.0 && point.1 > center.1,
                _ => unreachable!(),
            };
            
            is_in_corner && square(point.0 - center.0) + square((point.1 - center.1) * e_w / e_h) > square(e_w)
        };

        !is_in_corner(self.top_left.into(), pos, 0)
        && !is_in_corner(self.top_right.into(), pos, 1)
        && !is_in_corner(self.bottom_right.into(), pos, 2)
        && !is_in_corner(self.bottom_left.into(), pos, 3)
    }

    #[cfg(test)]
    pub fn top_left(&self) -> Corner {
        self.top_left
    }

    #[cfg(test)]
    pub fn bottom_right(&self) -> Corner {
        self.bottom_right
    }

    #[cfg(test)]
    pub fn top_right(&self) -> Corner {
        self.top_right
    }

    #[cfg(test)]
    pub fn bottom_left(&self) -> Corner {
        self.bottom_left
    }

    pub fn parse(input: &str) -> Result<Self, String> {
        Ok(Self::try_from(input)?)
    }
}

#[derive(Debug)]
enum ParseState {
    Start,
    After,
    Between,
    Number,
    GroupOpen,
    GroupBetween,
    GroupEnd,
    Group(u8),
}

impl TryFrom<&str> for CornerRadius {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // formats: (letters represent individual fields)
        // - "a,b,c,d" or "a b c d" -> top left, top right, bottom right, bottom left
        // - "a" -> all corners
        // individual field formats:
        // - "1" -> v & h
        // - "(1,2)" or (1 2) -> v, h
        let mut iter = value.chars().peekable();
        let mut corners = Vec::new();
        let mut state = ParseState::Start;
        let mut buf: (Idx, Idx) = (0, 0);

        while let Some(c) = iter.peek().copied() {
            macro_rules! consume {
                ($p:pat) => (if matches!(c, $p) { iter.next(); true } else { false });
                ($expr:expr) => (if $expr { iter.next(); true } else { false });
            }

            match state {
                ParseState::Start|ParseState::Between => {
                    if c.is_numeric() { state = ParseState::Number; }
                    else if consume!('(') { state = ParseState::GroupOpen; }
                    else if !consume!(c.is_whitespace()) { return Err(format!("Invalid character in corner radius: {c}")); }
                },
                ParseState::After => {
                    if consume!(',') || c == '(' || c.is_numeric() {
                        state = ParseState::Between;
                    }
                    else if !consume!(c.is_whitespace()) { return Err("Expected whitespace or ','".into()); }
                },
                ParseState::Number => {
                    if consume!(c.is_numeric()) {
                        buf.0 = buf.0 * 10 + c.to_digit(10).unwrap() as Idx;
                        buf.1 = buf.0;
                    } else {
                        corners.push(buf.into());
                        buf = (0, 0);
                        state = ParseState::After;
                    }
                },
                ParseState::GroupOpen => {
                    if c.is_numeric() { state = ParseState::Group(0); }
                    else if !consume!(c.is_whitespace()) { return Err("Expected digit or whitespace".into()); }
                },
                ParseState::GroupBetween => {
                    if consume!(',') || c.is_numeric() { state = ParseState::Group(1); }
                    else if c == ')' { buf.1 = buf.0; state = ParseState::GroupEnd; }
                    else if !consume!(c.is_whitespace()) { return Err("Expected digit, whitespace or ','".into()); }
                },
                ParseState::Group(0) => {
                    if consume!(c.is_numeric()) {
                        buf.0 = buf.0 * 10 + c.to_digit(10).unwrap() as Idx;
                    } else if consume!(',') || consume!(c.is_whitespace()) {
                        state = ParseState::GroupBetween;
                    }
                    else { return Err("Expected digit or ','".into()); }
                },
                ParseState::Group(1) => {
                    if consume!(c.is_numeric()) {
                        buf.1 = buf.1 * 10 + c.to_digit(10).unwrap() as Idx;
                    } else {
                        state = ParseState::GroupEnd;
                    }
                },
                ParseState::GroupEnd => {
                    if consume!(')') {
                        corners.push(buf.into());
                        buf = (0, 0);
                        state = ParseState::After;
                    } else if !consume!(c.is_whitespace()) {
                        return Err("Expected ')' or whitespace".into());
                    }
                },
                _ => unreachable!("Invalid state: {state:?}"),
            }
        }

        match state {
            ParseState::Number => corners.push(buf.into()),
            ParseState::Start|ParseState::After => {},
            _ => return Err(format!("Unexpected end of input").into()),
        }

        match corners.len() {
            1 => Ok(CornerRadius{ top_left: corners[0], top_right: corners[0], bottom_right: corners[0], bottom_left: corners[0] }),
            4 => Ok(CornerRadius{ top_left: corners[0], top_right: corners[1], bottom_right: corners[2], bottom_left: corners[3] }),
            _ => Err("Invalid number of corners - expected 1 or 4".into()),
        }
    }
}

#[test]
fn test_corner_radius_parsing() {
    let test_ok = |input, expected: ((Idx, Idx), (Idx, Idx), (Idx, Idx), (Idx, Idx))| {
        let result = CornerRadius::try_from(input).expect(&format!("Failed to parse {input}"));
        println!("{input} -> {result:?} == {expected:?}");
        assert_eq!(result.top_left().vertical(), expected.0.0, "Top left v");
        assert_eq!(result.top_left().horizontal(), expected.0.1, "Top left h");
        assert_eq!(result.top_right().vertical(), expected.1.0, "Top right v");
        assert_eq!(result.top_right().horizontal(), expected.1.1, "Top right h");
        assert_eq!(result.bottom_right().vertical(), expected.2.0, "Bottom right v");
        assert_eq!(result.bottom_right().horizontal(), expected.2.1, "Bottom right h");
        assert_eq!(result.bottom_left().vertical(), expected.3.0, "Bottom left v");
        assert_eq!(result.bottom_left().horizontal(), expected.3.1, "Bottom left h");
    };
    let test_err = |input, expected| {
        let result: Result<CornerRadius, String> = CornerRadius::try_from(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected);
    };

    test_ok("1", ((1, 1), (1, 1), (1, 1), (1, 1)));
    test_ok("1 2 3 4", ((1, 1), (2, 2), (3, 3), (4, 4)));
    test_ok("1,2,3,4", ((1, 1), (2, 2), (3, 3), (4, 4)));
    test_ok("1, 2, 3, 4", ((1, 1), (2, 2), (3, 3), (4, 4)));
    test_ok("(1 2) (3 4) (5 6) (7 8)", ((1, 2), (3, 4), (5, 6), (7, 8)));
    test_ok("(1,2)(3,4)(5,6)(7,8)", ((1, 2), (3, 4), (5, 6), (7, 8)));
    test_ok("(1, 2) (3, 4) (5, 6) (7, 8)", ((1, 2), (3, 4), (5, 6), (7, 8)));
    test_ok("(1,2) (3,4),(5,6),(7,8)", ((1, 2), (3, 4), (5, 6), (7, 8)));

    test_err("1 2 3", "Invalid number of corners - expected 1 or 4");
    test_err("1 2 3 4 5", "Invalid number of corners - expected 1 or 4");
    test_err("1 2 3 4 5,", "Unexpected end of input");
    test_err("(1 2 3 4 5 6", "Expected ')' or whitespace");
    test_err("()", "Expected digit or whitespace");
    test_err("(", "Unexpected end of input");
    test_err("1+", "Expected whitespace or ','");
}
