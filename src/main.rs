use clap::Parser;
use std::path::PathBuf;

use crate::index::{Index, Indexer};

mod index;
mod writer;

#[derive(Parser)]
#[command(name = "ferret")]
struct Args {
    #[arg(required = true)]
    directories: Vec<PathBuf>,

    #[arg(short, long, default_value = "facts.pl")]
    output: PathBuf,

    #[arg(long)]
    min_token_length: usize,
    #[arg(long)]
    max_file_size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("Ferret indexer starting...");

    let indexer = Indexer {
        min_token_length: args.min_token_length,
        max_file_size: args.max_file_size,
    };

    Ok(())
}
