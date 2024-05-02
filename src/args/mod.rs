use std::path::PathBuf;

use clap::Parser;

pub mod board_size;
use board_size::*;
use crate::{board::corner_radius::CornerRadius, board_pos::*};

/// Calculates a knight's tour on a board of the given size with the provided dimensions and starting position.
#[derive(Parser, Clone)]
pub struct Args {
    /// The size of the board in the form <WIDTH>[x<HEIGHT>] e.g. "12x9" for a 12 wide, 9 high board or "23" for a 23x23 board
    #[arg(long = "field-size", short, default_value("8x8"), value_parser = parse_board_size)]
    pub field: BoardSize,

    /// If set, the program will print additional debug information
    #[arg(long, short)]
    pub verbose: bool,
    
    #[arg(long, short, default_value("A-1"), value_parser = parse_board_pos, requires = "use_warnsdorff")]
    /// The starting position in the form <COLUMN>[-]<ROW> as on a normal chess board, starting in the lower left corner at A-1.
    /// The 27th column is addressed as AA, then follows AB, AC, ..., 52 is AZ, 53 is BA and so on
    pub starting_pos: BoardPos,

    /// The corner radius of the board. If not set, the board will have no rounded corners.
    /// This field supports both a single value for all corners as well as individual values for each corner.
    /// Individual values can be separated by either whitespace or commas. The order is top-left, top-right, bottom-right, bottom-left.
    /// Each corner value can either be a single number or a pair of numbers separated by a comma and enclosed in round brackets.
    /// In that case, the first number is the horizontal radius, the second the vertical radius.
    #[arg(long, short, value_parser = CornerRadius::parse, requires = "use_warnsdorff")]
    pub corner_radius: Option<CornerRadius>,

    /// If set, reads a board layout from the given file. The file should be one of the following:
    /// 
    /// - a text file with spaces representing inaccessible squares and printable characters representing accessible squares, and either windows or linux line endings
    /// - a PNG file with a black and white image, where black represents inaccessible squares and white represents accessible squares
    /// - a PNG file with a colored image which is then converted to black and white by taking the luminance of each pixel (this is done automatically if any pixel is neither black nor white)
    /// - a PNG file where the alpha channel is used to determine accessibility.to use this variant, specify --alpha-threshold
    #[arg(long, short, requires = "use_warnsdorff", conflicts_with = "corner_radius")]
    pub board_file: Option<PathBuf>,

    /// If set, the program will only consider squares with an alpha value greater than this threshold as accessible
    #[arg(long, requires_all(["board_file", "use_warnsdorff"]))]
    pub alpha_threshold: Option<u8>,

    /// If set, the program will not print the board and only output the number of moves
    #[arg(long, short)]
    pub quiet: bool,

    /// If set, the program will use the Warnsdorff heuristic to calculate the knight's tour. Warning: This can take a long time for large boards.
    #[arg(long, short = 'w')]
    pub use_warnsdorff: bool,
}