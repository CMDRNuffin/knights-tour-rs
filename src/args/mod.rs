use std::path::PathBuf;

use clap::{*, builder::*};

use crate::{board::corner_radius::CornerRadius, board_pos::{BoardPos, parse_board_pos}};

pub mod board_size;
use board_size::{parse_board_size, BoardSize};

/// Calculates a knight's tour on a board of the given size with the provided dimensions and starting position.
#[derive(Parser, Clone)]
pub struct Args {
    /// If set, the program will use the Warnsdorff heuristic to calculate the knight's tour.
    /// Warning: This can take a long time for large boards.
    #[arg(long, short = 'w')]
    pub use_warnsdorff: bool,

    #[command(flatten)]
    pub warnsdorff: Option<Warnsdorff>,

    /// The size of the board in the form <WIDTH>[x<HEIGHT>]
    /// 
    /// e.g. "12x9" for a 12 wide, 9 high board or "23" for a 23x23 board
    #[arg(long, short = 's', conflicts_with("board_file"), value_parser = parse_board_size)]
    pub board_size: Option<BoardSize>,
    
    /// If set, the program will print additional debug information
    #[arg(long, short)]
    pub verbose: bool,

    /// If set, the program will not print the board and only output the number of moves
    #[arg(long, short)]
    pub quiet: bool,

}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ImageMode {
    BlackWhite,
    Alpha,
    Luminance,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BoardFileType {
    Text,
    Image,
}

#[derive(Parser, Clone)]
pub struct Warnsdorff {
    /// The path to the file containing the board layout. See documentation for --board-file-format for more information
    #[arg(long, short = 'f', requires_all(["use_warnsdorff", "board_file_format"]))]
    pub board_file: Option<PathBuf>,

    /// If set, reads a board layout of the specified type from the file specified by --board-file:
    /// - text: a text file where spaces represent inaccessible squares and printable characters
    ///   represent accessible squares. The file should have either windows or linux line endings.
    /// - image: a PNG image representing the board. Specify the mode via --image-mode:
    ///   - black-white: black pixels are accessible, white pixels are inaccessible, all other color
    ///     values are invalid
    ///   - alpha: the alpha channel is used to determine accessibility. any pixel with an alpha value
    ///     greater than or equal to <THRESHOLD> is considered accessible. If this mode is chosen, the
    ///     --threshold option is required
    ///   - luminance (DEFAULT): the luminance of each pixel is used to determine accessibility. If the
    ///     luminance is greater than or equal to <THRESHOLD>, the pixel is considered accessible. The
    ///     default threshold is 128
    #[arg(
        long,
        short,
        requires = "board_file",
        verbatim_doc_comment,
        help = "If set, reads a board layout of the specified type from the file specified by --board-file"
    )]
    pub board_file_format: Option<BoardFileType>,

    /// The mode to use when reading an image file. See --board-file-format for more information
    #[arg(
        long,
        short,
        requires = "board_file_format",
        default_value_if("board_file_format", ArgPredicate::Equals("image".into()), "luminance")
    )]
    pub image_mode: Option<ImageMode>,

    /// If set, the program will only consider squares with an alpha value greater than this threshold as accessible
    #[arg(
        short,
        long,
        requires = "image_mode",
        default_value_if("image_mode", ArgPredicate::Equals("luminance".into()), "128"),
        required_if_eq_any([("image_mode", "alpha"),])
    )]
    pub threshold: Option<u8>,

    /// The corner radius of the board. If set, the board will have rounded corners.
    /// This field supports both a single value for all corners as well as individual values for each corner.
    /// Individual values can be separated by either whitespace or commas. The order is top-left, top-right, bottom-right, bottom-left.
    /// Each corner value can either be a single number or a pair of numbers separated by a comma and enclosed in round brackets.
    /// In that case, the first number is the horizontal radius, the second the vertical radius.
    #[arg(
        long,
        short,
        value_parser = CornerRadius::parse,
        requires_all = ["use_warnsdorff", "board_size"],
        help = "The corner radius of the board. If set, the board will have rounded corners"
    )]
    pub corner_radius: Option<CornerRadius>,

    /// The starting position in the form <COLUMN>[-]<ROW> as on a normal chess board, starting in the upper left corner at A1 (or A-1).
    /// 
    /// The 27th column is addressed as AA, then follows AB, AC, ..., 52 is AZ, 53 is BA and so on
    #[arg(long, short = 'p', default_value = "A1", value_parser = parse_board_pos, requires = "use_warnsdorff")]
    pub starting_pos: Option<BoardPos>,
}
