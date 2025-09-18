use crate::{
    index::{Indexer, MAX_FILE_SIZE, MIN_TOKEN_LENGTH},
    writer::Writer,
};
use clap::Parser;
use rayon::prelude::*;
use std::{fs, path::PathBuf};

mod index;
mod writer;

#[derive(Parser)]
#[command(name = "ferret")]
struct Args {
    #[arg(required = true)]
    directories: Vec<PathBuf>,

    #[arg(short, long, default_value = "facts.pl")]
    output: PathBuf,

    #[arg(long, default_value_t = MIN_TOKEN_LENGTH)]
    min_token_length: usize,
    #[arg(long, default_value_t = MAX_FILE_SIZE)]
    max_file_size: usize, // in MB
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("Ferret indexer starting...");

    let indexer = Indexer {
        min_token_length: args.min_token_length,
        max_file_size: args.max_file_size,
    };

    let files = indexer.collect_files(&args.directories)?;
    println!("Found {} files to index.", files.len());

    let contents: Vec<String> = files
        .par_iter()
        .map(|path| fs::read_to_string(path).unwrap_or_default())
        .collect();

    let index = indexer.index_directories(&files, &contents)?;
    Writer::write_facts(&args.output, &index)?;

    println!("Facts written to {}", args.output.display());
    Ok(())
}
