use dashmap::DashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::index::{Document, Index};

pub struct Writer;

impl Writer {
    pub fn write_facts<'a>(
        output_path: &Path,
        index: &Index<'a>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        Self::write_document_facts(&mut writer, &index.documents)?;
        Self::write_token_facts(&mut writer, &index.scores)?;
        Self::write_vocab_facts(&mut writer, &index.vocabulary)?;

        writer.flush()?;
        Ok(())
    }

    fn write_document_facts<'a>(
        writer: &mut BufWriter<File>,
        documents: &[Document<'a>],
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(writer, "% Document facts: document(ID, Path, Name)")?;
        for doc in documents {
            writeln!(
                writer,
                "document({}, '{}', '{}').",
                doc.id,
                doc.path.display().to_string().replace('\'', "\\'"),
                doc.name.replace('\'', "\\'")
            )?;
        }
        writeln!(writer)?;
        Ok(())
    }

    fn write_token_facts<'a>(
        writer: &mut BufWriter<File>,
        scores: &DashMap<(usize, &'a str), f64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(writer, "% Token facts: token(DocID, Token, TF_IDF_Score)")?;

        let mut sorted_tokens: Vec<_> = scores
            .iter()
            .map(|entry| (*entry.key(), *entry.value()))
            .collect();
        sorted_tokens.sort_by(|a, b| a.0.cmp(&b.0));

        for ((doc_id, token), score) in sorted_tokens {
            writeln!(
                writer,
                "token({}, '{}', {:.6}).",
                doc_id,
                token.replace('\'', "\\'"),
                score
            )?;
        }
        writeln!(writer)?;
        Ok(())
    }

    fn write_vocab_facts<'a>(
        writer: &mut BufWriter<File>,
        vocab_scores: &DashMap<&'a str, f64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(writer, "% Vocabulary facts: vocab(Token, IDF_Score)")?;

        let mut sorted_vocab: Vec<_> = vocab_scores
            .iter()
            .map(|entry| (*entry.key(), *entry.value()))
            .collect();
        sorted_vocab.sort_by(|a, b| a.0.cmp(&b.0));

        for (token, score) in sorted_vocab {
            writeln!(
                writer,
                "vocab('{}', {:.6}).",
                token.replace('\'', "\\'"),
                score
            )?;
        }
        Ok(())
    }
}
