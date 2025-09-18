<div align="center">
    <h1>Ferret</h1>
    <h3>(/ˈfɛr.ɪt/)</h3>
    <img alt="Ferret" src=".github/ferret-icon.png" width="180" height="180" />
    <h5>Sniffs out code, leaves no file unturned :mag: </h5>

![Rust](https://img.shields.io/badge/rust-yes-d62828?style=for-the-badge&logo=rust)
![Prolog](https://img.shields.io/badge/prolog-yes-313244?style=for-the-badge&logo=prolog)
</div>

---

## 🕵️‍♂️ What is Ferret?

Ferret is your multi-lingual search sidekick. It tunnels through your project directories, sniffs out words (no matter how suspicious), and builds a fuzzy search index with TF-IDF superpowers.

**Goal:**  
To help you find code, config, or cryptic comments using natural, fuzzy queries.  
No more `grep` gymnastics. No more lost semicolons. Just ask, and Ferret finds.

---

## 🧩 Project Structure

- **Rust Indexer**  
  Ferret's nose. Scans your files, calculates TF-IDF, and writes Prolog facts.  
  Heavy lifting: parallel file crawling, tokenizing, statistical magic.

- **Prolog Search Client**  
  Ferret's brain. Loads the facts, answers your fuzzy search queries interactively.  
  All the logic lives here.  
  Example:  
  ```prolog
  ?- fuzzy_search("red button", Results).
  ```

---

## ⚡ Usage

**1. Index your codebase (Rust):**

```bash
cargo run --release -- ~/your/project --output fuzzy_index.pl
```

**2. Search interactively (Prolog):**

```prolog
swipl
?- consult('fuzzy_finder.pl').
?- fuzzy_search("suspicious config", Results).
```

---

## 🦦 Why "Ferret"?

Because a ferret will always find what you lost – even if it was a function called `do_stuff_123` or a config file named `secrets.txt` you swore you deleted.

---

## 📚 Philosophy

> "Grepping is good. Fuzzy finding is ferret."  
> If you want precision, use a robot. If you want intuition, use a ferret.

---

## 🚧 Status

- [x] Rust indexer (sniffs, scores, writes facts)
- [ ] Prolog client (work in progress: fuzzy logic, actual answers)
- [ ] World domination

---

**Ferret: Because you deserve to find your code before your deadline.**
