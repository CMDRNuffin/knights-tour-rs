mod args;
mod field_pos;
mod matrix2d;
mod knight;
use knight::Knight;
use matrix2d::Matrix2D;
use args::Args;
use clap::Parser;

fn main() {
    let args = Args::parse();
    
    // from wikipedia: https://en.wikipedia.org/wiki/Knight%27s_tour#Existence
    // For any m × n board with m ≤ n, a knight's tour is always possible unless one or more of these three conditions are met:
    // - m = 1 or 2
    // - m = 3 and n = 3, 5, or 6
    // - m = 4 and n = 4
    let min_dimension = args.field.width().min(args.field.height());
    let max_dimension = args.field.width().max(args.field.height());
    if min_dimension <= 2
        || (min_dimension == 3 && matches!{max_dimension, 3 | 5 | 6})
        || (min_dimension == 4 && max_dimension == 4) {
        println!("No solution possible for this board size");
        return;
    }

    // Initialize the board and the knight
    // The board is a 2D matrix of booleans, where true means the knight has visited the field
    // The knight is a struct that holds the current position of the knight and various methods to move it around as well as calculate possible moves
    let mut board = Matrix2D::new(args.field.width(), args.field.height(), 0u32);
    let mut knight = Knight::new(args.starting_pos);

    if !board.is_in_range(args.starting_pos) {
        println!("Invalid starting position");
        return;
    }
    
    let mut moves = 1;
    *board.at_mut(args.starting_pos) = moves;

    // The knight's tour algorithm
    while moves < args.field.width() as u32 * args.field.height() as u32 {
        let possible_moves = knight.get_possible_moves(&board);
        let next_move = possible_moves.iter()
            .filter(|pos| *board.at(**pos) == 0)
            .min_by_key(|pos| match knight.clone_to(**pos).get_possible_moves(&board).len(){
                0 => usize::MAX,
                n => n
            })
            .copied();

        if let Some(next_move) = next_move {
            moves += 1;
            *board.at_mut(next_move) = moves;
            knight.update_position(next_move);
        } else {
            // todo: backtracking
            break;
        }
    }

    if !args.quiet {
        println!("{board}");
    }
    
    println!("{moves} moves");
}
