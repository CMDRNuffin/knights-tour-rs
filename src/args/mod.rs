use clap::Parser;

pub mod field_size;
use field_size::*;
use crate::field_pos::*;

/// Calculates a knight's tour on a board of the given size with the provided dimensions and starting position.
#[derive(Parser)]
pub struct Args {
    /// The size of the board in the form <WIDTH>[x<HEIGHT>] e.g. "12x9" for a 12 wide, 9 high board or "23" for a 23x23 board
    #[arg(long = "field-size", short, default_value("8x8"), value_parser = parse_field_size)]
    pub field: FieldSize,
    
    #[arg(long, short, default_value("A-1"), value_parser = parse_field_pos)]
    /// The starting position in the form <COLUMN>[-]<ROW> as on a normal chess board, starting in the lower left corner at A-1.
    /// The 27th column is addressed as AA, then follows AB, AC, ..., 52 is AZ, 53 is BA and so on
    pub starting_pos: FieldPos,

    /// If set, the program will not print the board and only output the number of moves
    #[arg(long, short)]
    pub quiet: bool,
}
