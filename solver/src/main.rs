mod lib;
extern crate clap;
use anyhow::{Context, Result};
use clap::Parser;
use lib::{Board, InFile};
use std::path::{Path, PathBuf};
use std::{boxed::Box, fmt, fs::File, io::BufReader, ops::BitAnd};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Turn on verbose messages
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// json file to solve
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    // check args
    let json: InFile = serde_json::from_reader(BufReader::new(
        File::open(args.input.clone())
            .with_context(|| format!("unable to find file \"{}\"", args.input.display()))?,
    ))
    .with_context(|| format! {"unable to parse file \"{}\"", args.input.display()})?;
    let mut board = Board::new(json.clone())?;
    if args.verbose {
        println!(
            "parsed \"{}\" with shape ({},{})\ngrid:\n{}",
            args.input.display(),
            board.width(),
            board.height(),
            board.pretty_grid_str()
        );
    };
    // solve the board
    board.full_solve(args.verbose)?;
    // print the solution
    println!("{}", board.pretty_grid_str());
    Ok(())
}
