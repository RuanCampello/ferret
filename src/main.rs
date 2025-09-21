use crate::{
    index::{Indexer, MAX_FILE_SIZE, MIN_TOKEN_LENGTH},
    writer::Writer,
};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::{fs, path::PathBuf};

mod index;
mod writer;

#[derive(Parser)]
#[command(name = "ferret")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(required = true)]
    directories: Vec<PathBuf>,

    #[arg(short, long, default_value = "facts.pl")]
    output: PathBuf,

    #[arg(long, default_value_t = MIN_TOKEN_LENGTH)]
    min_token_length: usize,
    #[arg(long, default_value_t = MAX_FILE_SIZE)]
    max_file_size: usize, // in MB

    #[arg(long)]
    query: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    Query {
        query: String,
        #[arg(short, long, default_value = "facts.pl")]
        facts: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if let Some(Command::Query { query, facts }) = args.command {
        let contents = fs::read_to_string(&facts)?;
        let mut token_results = Vec::new();
        let mut vocab_results = Vec::new();

        for line in contents.lines() {
            if line.starts_with("token(") && line.contains(&format!("'{}'", query)) {
                token_results.push(line.trim());
            }
            if line.starts_with("vocab(") && line.contains(&format!("'{}'", query)) {
                vocab_results.push(line.trim());
            }
        }

        println!("\nToken Results");
        for result in token_results {
            println!("{result}");
        }
        println!("\nVocabulary Results");
        for result in vocab_results {
            println!("{result}");
        }
        return Ok(());
    }

    if args.directories.is_empty() {
        eprintln!("No directories specified.");
        return Ok(());
    }

    let indexer = Indexer {
        min_token_length: args.min_token_length,
        max_file_size: args.max_file_size,
    };

    let files = indexer.collect_files(&args.directories)?;
    println!("Found {} files to index.", files.len());

    let (contents, lengths): (Vec<String>, Vec<usize>) = files
        .par_iter()
        .map(|path| {
            let content = fs::read_to_string(path).unwrap_or_default();
            let len = content.len();
            (content, len)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .unzip();

    let total_length: usize = lengths.into_iter().sum();

    let index = indexer.index_directories(&files, &contents)?;
    Writer::write_facts(&args.output, &index)?;

    println!("Facts written to {}", args.output.display());
    println!("Read {total_length} total lines");
    Ok(())
}
