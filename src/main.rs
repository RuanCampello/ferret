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

    #[arg(required = false)]
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
        let mut documents = std::collections::HashMap::new();
        let mut token_results = Vec::new();
        let mut vocab_results = Vec::new();

        for line in contents.lines() {
            if line.starts_with("document(") {
                let open = line.find('(').unwrap_or(0);
                let close = line.find(')').unwrap_or(line.len() - 1);
                let args: Vec<&str> = line[open + 1..close].split(',').collect();
                if args.len() >= 3 {
                    let id = args[0].trim().parse::<usize>().unwrap_or(0);
                    let path = args[1].trim().trim_matches('\'').to_string();
                    let name = args[2].trim().trim_matches('\'').to_string();
                    documents.insert(id, (path, name));
                }
            }
        }

        for line in contents.lines() {
            if line.starts_with("token(") && line.contains(&format!("'{}'", query)) {
                let open = line.find('(').unwrap_or(0);
                let close = line.find(')').unwrap_or(line.len() - 1);
                let args: Vec<&str> = line[open + 1..close].split(',').collect();
                if args.len() >= 3 {
                    let id = args[0].trim().parse::<usize>().unwrap_or(0);
                    let score = args[2].trim().parse::<f64>().unwrap_or(0.0);
                    let (path, name) = documents
                        .get(&id)
                        .cloned()
                        .unwrap_or(("???".into(), "???".into()));
                    token_results.push((score, query.clone(), path, name));
                }
            }
            if line.starts_with("vocab(") && line.contains(&format!("'{}'", query)) {
                vocab_results.push(line.trim().to_string());
            }
        }

        token_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        println!("\nResults");
        for (score, token, path, name) in token_results {
            println!(
                "score={:.6} | token='{}' | file='{}' | name='{}'",
                score, token, path, name
            );
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
