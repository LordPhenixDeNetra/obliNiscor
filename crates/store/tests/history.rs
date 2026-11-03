//! Historique JSONL : ajout, relecture tolérante et élagage, contre le vrai système de fichiers.
//!
//! Chaque test travaille dans un dossier temporaire unique, supprimé à la fin.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use obliniscor_core::ErrorCode;
use obliniscor_store::{History, HistoryEntry, Operation, Outcome};
use std::path::PathBuf;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!("obliniscor-history-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("dossier temporaire");
        Self(path)
    }

    fn path(&self) -> PathBuf {
        self.0.clone()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn clean(at: u64) -> HistoryEntry {
    HistoryEntry {
        at,
        operation: Operation::MemoryClean {
            zones: vec!["standby".to_owned()],
            dry_run: true,
        },
        outcome: Outcome::Success { freed: 0 },
    }
}

#[test]
fn a_missing_history_reads_as_empty_and_creates_nothing() {
    let dir = TempDir::new("absent");
    let history = History::new(dir.path());
    let report = history
        .read_lenient()
        .expect("absence n'est pas une erreur");
    assert!(report.entries.is_empty());
    assert_eq!(report.skipped, 0);
    assert!(!history.path().exists(), "lire ne doit rien créer");
}

#[test]
fn entries_come_back_in_the_order_they_were_added() {
    let dir = TempDir::new("ordre");
    let history = History::new(dir.path());
    for at in [10, 20, 30] {
        history.append(&clean(at)).expect("ajout");
    }
    let report = history.read_lenient().expect("relecture");
    let instants: Vec<u64> = report.entries.iter().map(|entry| entry.at).collect();
    assert_eq!(instants, [10, 20, 30]);
    assert_eq!(report.skipped, 0);
}

#[test]
fn the_file_is_one_json_object_per_line() {
    let dir = TempDir::new("jsonl");
    let history = History::new(dir.path());
    history.append(&clean(10)).expect("ajout");
    history
        .append(&HistoryEntry {
            at: 20,
            operation: Operation::ProcessTrim { pids: vec![42, 43] },
            outcome: Outcome::failed(ErrorCode::Denied),
        })
        .expect("ajout");

    let text = std::fs::read_to_string(history.path()).expect("lecture brute");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "une ligne par entrée");
    for line in lines {
        // Chaque ligne se lit seule : c'est la propriété qui rend le JSONL récupérable.
        serde_json::from_str::<HistoryEntry>(line).expect("ligne autonome");
    }
    assert!(text.contains("\"E_DENIED\""), "le code est stocké tel quel");
}

#[test]
fn a_corrupt_line_is_skipped_and_counted() {
    let dir = TempDir::new("corrompu");
    let history = History::new(dir.path());
    history.append(&clean(10)).expect("ajout");
    // Ligne ajoutée à la main, comme le ferait une panne d'écriture.
    let mut text = std::fs::read_to_string(history.path()).expect("lecture");
    text.push_str("{ceci n'est pas du JSON}\n");
    std::fs::write(history.path(), text).expect("écriture");
    history.append(&clean(30)).expect("ajout");

    let report = history.read_lenient().expect("relecture");
    assert_eq!(
        report.entries.len(),
        2,
        "les lignes valides sont conservées"
    );
    assert_eq!(report.skipped, 1, "la ligne cassée est comptée, pas fatale");
}

#[test]
fn pruning_keeps_recent_entries_and_removes_old_ones() {
    let dir = TempDir::new("elagage");
    let history = History::new(dir.path());
    let now = 1_000_000_000;
    let day = 86_400;
    history.append(&clean(now - 10 * day)).expect("ancien");
    history.append(&clean(now - 2 * day)).expect("récent");
    history.append(&clean(now)).expect("maintenant");

    let removed = history.prune(7, now).expect("élagage");
    assert_eq!(removed, 1);
    let report = history.read_lenient().expect("relecture");
    assert_eq!(report.entries.len(), 2);
    assert!(report.entries.iter().all(|entry| entry.at > now - 7 * day));
}

#[test]
fn pruning_with_zero_retention_clears_the_history() {
    let dir = TempDir::new("desactive");
    let history = History::new(dir.path());
    history.append(&clean(1)).expect("ajout");
    history.append(&clean(2)).expect("ajout");

    let removed = history.prune(0, 100).expect("élagage");
    assert_eq!(removed, 2);
    assert!(
        history
            .read_lenient()
            .expect("relecture")
            .entries
            .is_empty()
    );
}

#[test]
fn pruning_nothing_leaves_the_file_untouched() {
    let dir = TempDir::new("intact");
    let history = History::new(dir.path());
    history.append(&clean(100)).expect("ajout");
    let before = std::fs::read(history.path()).expect("lecture");

    let removed = history.prune(30, 100 + 86_400).expect("élagage");
    assert_eq!(removed, 0);
    let after = std::fs::read(history.path()).expect("lecture");
    assert_eq!(before, after, "aucune réécriture inutile");
}
