mod args;
mod board_pos;
mod board;
mod knight;
use std::time::Instant;

use knight::Knight;
use board::Board;
use args::Args;
use clap::Parser;

use crate::board_pos::BoardPos;

fn main() {
    let args = Args::parse();
    
    // from wikipedia: https://en.wikipedia.org/wiki/Knight%27s_tour#Existence
    // For any m × n board with m ≤ n, a knight's tour is always possible unless one or more of these three conditions are met:
    // - m = 1 or 2
    // - m = 3 and n = 3, 5, or 6
    // - m = 4 and n = 4
    let min_dimension = args.field.width().min(args.field.height());
    let max_dimension = args.field.width().max(args.field.height());
    if min_dimension == 1 && max_dimension == 1 {
        println!("+---+");
        println!("| 1 |");
        println!("+---+");
        println!("Very funny.");
        println!();
        println!("1 move");
        return;
    }
    else if min_dimension <= 2
        || (min_dimension == 3 && matches!{max_dimension, 3 | 5 | 6})
        || (min_dimension == 4 && max_dimension == 4) {
        println!("No solution possible for this board size");
        return;
    }

    // Initialize the board and the knight
    // The board is a 2D matrix of booleans, where true means the knight has visited the field
    // The knight is a struct that holds the current position of the knight and various methods to move it around as well as calculate possible moves
    let mut board = Board::new(args.field.width(), args.field.height(), 0, args.corner_radius);
    let mut knight = Knight::new(args.starting_pos);

    if !board.is_in_range(args.starting_pos) {
        println!("Invalid starting position");
        return;
    }
    

    let algo = if args.use_warnsdorff { warnsdorff } else { divide_and_conquer };

    let now = Instant::now();
    algo(&mut board, &mut knight, args.starting_pos);

    let elapsed = now.elapsed();

    if !args.quiet {
        println!("{board}");
    }
    else {
        board.print_errors();
    }
    
    println!("Elapsed time: {}.{:03} seconds", elapsed.as_secs(), elapsed.subsec_millis());
}

fn warnsdorff(board: &mut Board, knight: &mut Knight, starting_pos: BoardPos) {
    // The knight's tour algorithm
    let mut moves: Vec<(BoardPos, u8)> = Vec::new();
    moves.push((starting_pos, 0));
    *board.at_mut(starting_pos) = moves.len();

    let expected_move_count = board.available_fields();
    while moves.len() < expected_move_count {
        let skip = moves.last().copied().unwrap().1;
        let possible_moves = knight.get_possible_moves(&board, skip);
        let next_move = possible_moves.iter()
            .min_by_key(|pos| match knight.clone_to(**pos).possible_moves_count(&board, 3){
                0 => usize::MAX,
                n => n
            })
            .copied();

        if let Some(next_move) = next_move {
            moves.push((next_move, 0));
            *board.at_mut(next_move) = moves.len();
            knight.update_position(next_move);
        } else if moves.len() > 1 {
            // undo the last move
            let last_move = moves.pop().unwrap();
            *board.at_mut(last_move.0) = 0;

            let prev_move = moves.last_mut().unwrap();
            // skip the last move
            prev_move.1 += 1;
            knight.update_position(prev_move.0);
        }
        else {
            println!("No knight's tour possible for this board configuration.");
            break;
        }
    }
}

fn divide_and_conquer(board: &mut Board, knight: &mut Knight, starting_pos: BoardPos) {
    // todo: implement divide and conquer algorithm
    // step 1: break up board into manageable chunks
    // step 2: generate a closed knight's tour for each chunk if possible, and noting start and finish otherwise
    // step 3: stitch the tours together
    // step 4 (optional, if I have time): apply the obfuscation algorithm
    warnsdorff(board, knight, starting_pos);
}