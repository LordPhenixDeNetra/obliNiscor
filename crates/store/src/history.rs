//! Historique des opérations, au format **JSONL** : un objet JSON par ligne.
//!
//! Le choix du JSONL tient à trois propriétés qu'un tableau JSON n'a pas : on peut ajouter une
//! ligne sans relire le fichier, une ligne corrompue ne perd que cette ligne, et un outil
//! standard (`jq`, un éditeur de texte) peut le lire sans schéma.
//!
//! L'historique est **descriptif, jamais exécutoire** : rien ne le relit pour rejouer une
//! action. C'est une trace pour l'utilisateur, pas un journal de reprise.

use obliniscor_core::ErrorCode;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::StoreError;

/// Nom du fichier d'historique, dans le dossier de l'application.
pub const FILE_NAME: &str = "history.jsonl";

/// Une opération consignée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case")]
pub enum Operation {
    /// Nettoyage de zones mémoire.
    MemoryClean {
        /// Zones demandées, par leur nom d'IPC (`working-set`, `standby`, …).
        zones: Vec<String>,
        /// Vrai quand rien n'a été exécuté : la ligne existe quand même, la simulation aussi
        /// est une information.
        dry_run: bool,
    },
    /// Réduction de l'ensemble de travail de processus précis.
    ProcessTrim {
        /// Identifiants des processus visés.
        pids: Vec<u32>,
    },
}

/// Résultat d'une opération.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum Outcome {
    /// L'opération a abouti, avec les octets effectivement libérés.
    Success {
        /// Octets libérés ; zéro pour une simulation.
        freed: u64,
    },
    /// L'opération a échoué, avec son code stable.
    ///
    /// Le code est stocké sous sa forme **chaîne** (`E_DENIED`, …) et non sous forme d'énuméré :
    /// c'est le contrat exposé aux clients, et le fichier doit rester lisible quand le code
    /// ajoute une variante (`FR-HIS-2`).
    Failed {
        /// Code stable, voir [`ErrorCode::as_str`].
        code: String,
    },
}

impl Outcome {
    /// Construit un échec à partir du code typé, sans laisser l'appelant formater à la main.
    #[must_use]
    pub fn failed(code: ErrorCode) -> Self {
        Self::Failed {
            code: code.as_str().to_owned(),
        }
    }
}

/// Une ligne d'historique.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Instant de l'opération, en secondes depuis l'époque UNIX.
    ///
    /// Un entier et non une chaîne ISO-8601 : l'horodatage sert à calculer une rétention et un
    /// ordre, pas à être lu par un humain, et un entier ne dépend pas du fuseau de la machine.
    pub at: u64,
    /// Ce qui a été demandé.
    #[serde(flatten)]
    pub operation: Operation,
    /// Ce qui en est résulté.
    #[serde(flatten)]
    pub outcome: Outcome,
}

/// Fichier d'historique d'un dossier d'application.
#[derive(Debug, Clone)]
pub struct History {
    dir: PathBuf,
}

/// Résultat de [`History::read_lenient`] : les lignes lisibles et le nombre d'ignorées.
#[derive(Debug, Clone, Default)]
pub struct ReadReport {
    /// Lignes valides, dans l'ordre du fichier.
    pub entries: Vec<HistoryEntry>,
    /// Lignes illisibles écartées.
    pub skipped: usize,
}

impl History {
    /// Ouvre (sans le créer) l'historique d'un dossier d'application.
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Chemin du fichier d'historique.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    /// Ajoute une ligne, en créant le fichier et son dossier au besoin.
    ///
    /// Une seule écriture par ligne : le fichier n'est pas relu, et une panne au milieu ne peut
    /// pas corrompre les lignes précédentes.
    pub fn append(&self, entry: &HistoryEntry) -> Result<(), StoreError> {
        std::fs::create_dir_all(&self.dir)?;
        let mut line = serde_json::to_string(entry)?;
        line.push('\n');
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path())?;
        file.write_all(line.as_bytes())?;
        Ok(())
    }

    /// Lit l'historique, en ignorant les lignes illisibles.
    ///
    /// Un historique corrompu ne doit pas empêcher l'application de démarrer : les lignes
    /// valides sont rendues, les autres comptées. L'appelant peut signaler le nombre.
    pub fn read_lenient(&self) -> Result<ReadReport, StoreError> {
        let bytes = match std::fs::read(self.path()) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ReadReport::default());
            }
            Err(error) => return Err(error.into()),
        };

        let mut report = ReadReport::default();
        for line in String::from_utf8_lossy(&bytes).lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<HistoryEntry>(line) {
                Ok(entry) => report.entries.push(entry),
                Err(_) => report.skipped += 1,
            }
        }
        Ok(report)
    }

    /// Écrit l'historique en ne gardant que les entrées postérieures à la rétention.
    ///
    /// Rend le nombre d'entrées supprimées. L'écriture passe par [`lpdnsoft_atomic`] : un
    /// élagage interrompu laisse l'ancien fichier intact, jamais un fichier à moitié réécrit.
    /// Une rétention de zéro jour vide l'historique, ce qui est la façon de le désactiver.
    pub fn prune(&self, retention_days: u32, now: u64) -> Result<usize, StoreError> {
        let report = self.read_lenient()?;
        if report.entries.is_empty() {
            return Ok(0);
        }
        let horizon = now.saturating_sub(u64::from(retention_days) * 86_400);
        let kept: Vec<&HistoryEntry> = report
            .entries
            .iter()
            .filter(|entry| retention_days > 0 && entry.at >= horizon)
            .collect();
        let removed = report.entries.len() - kept.len();
        if removed == 0 {
            return Ok(0);
        }

        let mut bytes = Vec::new();
        for entry in kept {
            bytes.extend_from_slice(serde_json::to_string(entry)?.as_bytes());
            bytes.push(b'\n');
        }
        lpdnsoft_atomic::write_atomic(&self.path(), &bytes)?;
        Ok(removed)
    }
}

/// Chemin de l'historique pour un dossier donné, sans construire d'objet.
#[must_use]
pub fn path_in(dir: &Path) -> PathBuf {
    dir.join(FILE_NAME)
}
