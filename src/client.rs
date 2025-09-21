use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
};

struct Client {
    swi_path: String,
    facts: String,
}

impl Client {
    pub fn new<P: AsRef<Path>, F: AsRef<Path>>(swi: P, facts: F) -> Self {
        Self {
            swi_path: swi.as_ref().to_string_lossy().to_string(),
            facts: facts.as_ref().to_string_lossy().to_string(),
        }
    }

    pub fn query(&self, query: &str) -> std::io::Result<Vec<String>> {
        let prolog_query = format!(
            "consult('{}'), forall({}, writeln(_)), halt.",
            self.facts.replace("//", "\\\\"),
            query
        );

        let mut process = Command::new(&self.swi_path)
            .arg("-q")
            .arg("-t")
            .arg(prolog_query)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = process.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        let results = reader.lines().filter_map(|line| line.ok()).collect();

        Ok(results)
    }
}
