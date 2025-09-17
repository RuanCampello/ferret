//! This module index recursively a directory files concurrently with [TF-IDF](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).

use dashmap::DashMap;
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator},
    slice::ParallelSlice,
};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, PartialEq)]
struct Indexer {
    min_token_length: usize,
    max_file_size: usize,
}

#[derive(Debug)]
pub struct Index<'i> {
    pub documents: Vec<Document<'i>>,
    pub scores: DashMap<(usize, &'i str), f64>,
    pub vocabulary: DashMap<&'i str, f64>,
}

#[derive(Debug)]
pub struct Document<'d> {
    pub id: usize,
    pub name: &'d str,
    pub path: &'d Path,
    pub tokens: DashMap<&'d str, usize>,
}

#[derive(Debug, Error)]
enum IndexError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// The max size in mb of a file that we can index.
const MAX_FILE_SIZE: usize = 10;
const MIN_TOKEN_LENGTH: usize = 2;

impl Indexer {
    const SEPARATORS: &[char] = &[
        '(', ')', '[', ']', '{', '}', '<', '>', ',', ';', ':', '.', '!', '?', '"', '\'', '`', ' ',
        '\t', '\n',
    ];
    const TRIMMERS: &[char] = &[
        '_', '-', '=', '+', '*', '/', '\\', '|', '&', '%', '$', '#', '@', '^', '~', ' ',
    ];

    fn index_directories<'i>(
        &self,
        paths: &'i [PathBuf],
        contents: &'i [String],
    ) -> Result<Index<'i>, IndexError> {
        let documents = self.document_files(&paths, contents);
        let (scores, vocabulary) = self.calculate_tf_idf(&documents);

        Ok(Index {
            documents,
            scores,
            vocabulary,
        })
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

    fn document_files<'i>(
        &self,
        paths: &'i [PathBuf],
        contents: &'i [String],
    ) -> Vec<Document<'i>> {
        // we need a trustful counter for this case
        let id = AtomicUsize::new(0);

        let documents = paths
            .par_iter()
            .zip(contents.par_iter())
            .map(|(path, content)| {
                let tokens = self.tokenize(content);
                let name = match path.file_name() {
                    Some(filename) => filename.to_str(),
                    None => Some("unknown"),
                }
                .unwrap_or("unknown");

                Document {
                    id: id.fetch_add(1, Ordering::Relaxed),
                    name,
                    path: path.as_path(),
                    tokens,
                }
            })
            .collect();

        documents
    }

    fn tokenize<'i>(&self, content: &'i str) -> DashMap<&'i str, usize> {
        let tokens = DashMap::new();
        let words: Vec<&str> = content.split_whitespace().collect();
        let chunk = (words.len() / rayon::current_num_threads()).max(100);

        words.par_chunks(chunk).for_each(|word_chunk| {
            word_chunk.into_iter().for_each(|word| {
                word.split(Self::SEPARATORS)
                    .map(|w| w.trim_matches(Self::TRIMMERS))
                    .filter(|w| {
                        w.len() >= self.min_token_length && !w.chars().all(|c| c.is_numeric())
                    })
                    .for_each(|token| *tokens.entry(token).or_insert(0) += 1);
            })
        });

        tokens
    }

    fn calculate_tf_idf<'i>(
        &self,
        documents: &[Document<'i>],
    ) -> (DashMap<(usize, &'i str), f64>, DashMap<&'i str, f64>) {
        let docs_len = documents.len() as f64;

        let doc_freq = DashMap::new();
        documents.par_iter().for_each(|doc| {
            doc.tokens
                .iter()
                .map(|token| *token.key())
                .for_each(|token| *doc_freq.entry(token).or_insert(0) += 1);
        });

        let scores = DashMap::new();
        documents.par_iter().for_each(|doc| {
            let doc_token_count: usize = doc.tokens.iter().map(|entry| *entry.value()).sum();

            doc.tokens.iter().for_each(|entry| {
                let token = *entry.key();
                let freq = *entry.value();

                if let Some(entry) = doc_freq.get(&token) {
                    let df = *entry.value() as f64;
                    let tf = freq as f64 / doc_token_count as f64;
                    let idf = (docs_len / df).ln();
                    let tf_idf = tf * idf;

                    scores.insert((doc.id, token), tf_idf);
                }
            });
        });

        let vocabulary = DashMap::new();
        doc_freq.iter().for_each(|entry| {
            let token = *entry.key();
            let idf = (docs_len / *entry.value() as f64).ln();
            vocabulary.insert(token, idf);
        });

        (scores, vocabulary)
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
