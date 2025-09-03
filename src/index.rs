//! This module index recursively a directory files concurrently with [TF-IDF](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).

use dashmap::DashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, PartialEq)]
struct Indexer {
    min_token_length: usize,
    max_file_size: usize,
}

#[derive(Debug)]
struct Index<'i> {
    documents: Vec<&'i Document<'i>>,
    scores: DashMap<(usize, &'i str), f64>,
    vocabulary: DashMap<&'i str, f64>,
}

#[derive(Debug)]
struct Document<'d> {
    id: usize,
    name: &'d str,
    path: &'d Path,
    tokens: DashMap<&'d str, usize>,
}

#[derive(Debug, Error)]
enum IndexError {}

/// The max size in mb of a file that we can index.
const MAX_FILE_SIZE: usize = 10;
const MIN_TOKEN_LENGTH: usize = 2;

impl Indexer {
    fn index_directories<'i>(&self, directories: &[PathBuf]) -> Result<Index<'i>, IndexError> {
        let paths = self.collect_files(directories)?;
        todo!()
    }

    fn collect_files(&self, directories: &[PathBuf]) -> Result<Vec<PathBuf>, IndexError> {
        let max_size = self.max_file_size * 1024 * 1024; // we need to convert the usize into mb

        let files = directories
            .par_iter()
            .flat_map(|dir| {
                WalkDir::new(dir)
                    .into_iter()
                    .par_bridge()
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.file_type().is_file())
                    .filter(|entry| match entry.metadata() {
                        Ok(metadata) => metadata.len() as usize <= max_size,
                        Err(_) => false,
                    })
                    .filter(|entry| Self::is_parsable(entry.path()))
                    .map(|entry| entry.path().to_path_buf())
            })
            .collect();

        Ok(files)
    }

    fn is_parsable(path: &Path) -> bool {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => matches!(
                ext.to_lowercase().as_str(),
                // languages
                "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" | "hpp"
                | "java" | "kt" | "swift" | "rb" | "php" | "cs" | "scala"
                | "clj" | "hs" | "ml" | "elm" | "ex" | "exs" | "erl"
                | "vim" | "lua" | "pl" 
                // normal text fiels
                | "txt" | "md" | "rst" | "org" | "tex" | "rtf"
                // configuration
                | "toml" | "yaml" | "yml" | "json" | "config"
                // markup & data
                | "xml" | "html" | "css" | "scss" | "sass" | "less"
                | "csv" |  "sql"
            ),
            _ => false,
        };

        match path.file_stem().and_then(|s| s.to_str()) {
            Some(stem) => matches!(
                stem.to_lowercase().as_str(),
                "readme"
                    | "license"
                    | "licence"
                    | "changelog"
                    | "changes"
                    | "history"
                    | "makefile"
                    | "dockerfile"
                    | "containerfile"
                    | "gitignore"
                    | "authors"
                    | "contributors"
                    | "manifest"
            ),
            _ => false,
        }
    }
}

impl Default for Indexer {
    fn default() -> Self {
        Self {
            min_token_length: MIN_TOKEN_LENGTH,
            max_file_size: MAX_FILE_SIZE,
        }
    }
}
