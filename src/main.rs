mod args;
mod board_pos;
mod board_size;
mod board;
mod knight;
mod warnsdorff;
mod divide_and_conquer;
mod debug_output;
mod move_graph;
mod svg;

use args::Args;
use clap::Parser;

pub mod aliases {
    // aliases for the board index type
    // Note that the Overflow type must be signed, otherwise it WILL overflow. It should also be larger than the Index type as to prevent overflows with very large boards.
    pub type BoardIndex = u32;
    pub type BoardIndexOverflow = i64;

    // Ensure that we don't accidentally define invalid index types
    const _: () = assert!(std::mem::size_of::<BoardIndex>() <= std::mem::size_of::<BoardIndexOverflow>());
    const _: () = assert!(std::mem::size_of::<BoardIndex>() <= std::mem::size_of::<usize>());
    const _: () = assert!(BoardIndex::MIN == 0);
    const _: () = assert!(BoardIndexOverflow::MIN < 0);
}

fn main() {
    let args = Args::parse();
    
    if args.verbose {
        debug_output::enable();
    }

    let solve = if args.use_warnsdorff {
        // cannot solve with divide and conquer if the field is not rectangular
        warnsdorff::solve
    } else {
        divide_and_conquer::solve
    };

    let quiet = args.quiet;
    let (elapsed, board) = if let Some(res) = solve(args) {
        res
    } else {
        println!("No solution possible for this board configuration");
        return;
    };

    if !quiet {
        let writer = std::io::stdout();
        svg::render_svg(&mut writer.lock(), &board, elapsed).unwrap();
        println!("<!-- Elapsed time: {}.{:03} seconds -->", elapsed.as_secs(), elapsed.subsec_millis());
    }
    else {
        println!("Elapsed time: {}.{:03} seconds", elapsed.as_secs(), elapsed.subsec_millis());
    }
}
