#![feature(iter_intersperse)]
#![feature(test)]

mod splitter;

use std::{
    fs::File,
    io::{self, Read},
};

use anyhow::Result;
use clap::Parser;
use console::style;
use splitter::Splitter;

/// An extremely simple CLI tool to split string contents.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The string on which the input is split
    #[arg(short, long)]
    split: String,

    /// The delimiter for the split elements
    #[arg(short, long, default_value = "\n")]
    delimiter: String,

    /// Only select a given index, indices or index ranges (separated by ',');
    /// Ranges are defined in the form of {start}-{end} (.i.e. 3-7)
    #[arg(short, long)]
    idx: Option<String>,

    /// A file to be read as input; If not provided, StdIn is used as input
    file: Option<String>,
}

fn main() {
    if let Err(err) = run() {
        println!("{} {}", style("error:").red().bold(), style(err).red());
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let input: Box<dyn Read> = match args.file {
        Some(f) => Box::new(File::open(f)?),
        None => Box::new(io::stdin()),
    };

    let splitter = Splitter::try_from_idx(args.idx, args.delimiter)?;
    splitter.split_stream(input, io::stdout(), args.split)?;

    Ok(())
}
