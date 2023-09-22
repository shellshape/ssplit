#![feature(iter_intersperse)]

use std::{
    fs::File,
    io::{self, Read, Write},
};

use anyhow::Result;
use clap::Parser;
use console::style;

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

    let mode = SplitMode::try_from_idx(args.idx, args.delimiter)?;

    split_stream(input, io::stdout(), args.split, mode)?;

    Ok(())
}

enum SplitMode {
    Replace(String),
    Indices {
        indices: Vec<u32>,
        delimiter: String,
    },
}

impl SplitMode {
    fn try_from_idx(idx: Option<String>, delimiter: String) -> Result<Self> {
        let Some(idx) = idx else {
            return Ok(Self::Replace(delimiter));
        };

        let elems = idx.split(',').map(str::trim);

        let mut indices = vec![];
        for elem in elems {
            if let Some((from, to)) = elem.split_once('-') {
                let from = from.parse()?;
                let to = to.parse()?;
                if from >= to {
                    anyhow::bail!("range begin value must be smaller than end value: {elem}");
                }
                for i in from..=to {
                    indices.push(i);
                }
            } else {
                indices.push(elem.parse()?);
            }
        }

        Ok(Self::Indices { indices, delimiter })
    }
}

fn split_stream<S: Into<String>>(
    mut input: impl Read,
    mut output: impl Write,
    split: S,
    mode: SplitMode,
) -> Result<()> {
    let split: String = split.into();

    let mut buf = [0u8; 16 * 1024];
    let mut i = 0;
    let mut first = true;

    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }

        let str = String::from_utf8(buf[..n].to_vec())?;
        let split = str.split(&split);

        match &mode {
            SplitMode::Replace(with) => {
                for elem in split.intersperse(with) {
                    output.write_all(elem.as_bytes())?;
                }
            }
            SplitMode::Indices { indices, delimiter } => {
                for elem in split {
                    if indices.contains(&i) {
                        if !first {
                            output.write_all(delimiter.as_bytes())?;
                        } else {
                            first = false;
                        }
                        output.write_all(elem.as_bytes())?;
                    }
                    i += 1;
                }
            }
        }
    }

    Ok(())
}
