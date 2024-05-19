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
use std::io::Write;

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

    let solve = if args.input.use_warnsdorff {
        // cannot solve with divide and conquer if the field is not rectangular
        warnsdorff::solve
    } else {
        divide_and_conquer::solve
    };

    let quiet = args.quiet;
    let output_options = (args.output_file, args.output_format);
    let (elapsed, board) = if let Some(res) = solve(args.input) {
        res
    } else {
        println!("No solution possible for this board configuration");
        return;
    };

    let out_format = match output_options.0 {
        None => args::OutputFormat::Text,
        Some(_) => match output_options.1 {
            args::OutputFormat::Auto => {
                let ext = output_options.0.as_ref()
                    .map(|s|s.extension().map(|s|s.to_str()))
                    .flatten()
                    .flatten()
                    .unwrap_or("")
                    .to_lowercase();

                match &ext as &str {
                    "svg" => args::OutputFormat::Svg,
                    _ => args::OutputFormat::Text,
                }
            },
            other => other,
        },
    };

    let elapsed_text = format!("Elapsed time: {}.{:03} seconds", elapsed.as_secs(), elapsed.subsec_millis());
    if !quiet {
        let mut writer: Box<dyn Write> = if let Some(file) = output_options.0 {
            Box::new(std::fs::File::create(file).unwrap())
        } else {
            Box::new(std::io::stdout())
        };

        match out_format {
            args::OutputFormat::Text => {
                writeln!(writer, "{}", board.to_board()).unwrap();
                writeln!(writer, "{}", elapsed_text).unwrap();
            },
            args::OutputFormat::Svg => {
                svg::render_svg(&mut writer, &board, elapsed).unwrap();
                writeln!(writer, "<!-- {} -->", elapsed_text).unwrap();
            },
            args::OutputFormat::Auto => unreachable!(),
        }
    } else {
        println!("{}", elapsed_text);
    }
}
