//! État persistant d'obliNiscor : réglages, historique et règles.
//!
//! Ce crate est le seul qui **écrit** sur le disque de l'utilisateur, et il le fait par des
//! bibliothèques de la suite (`lpdnsoft-settings` pour les réglages, `lpdnsoft-atomic` pour
//! les écritures tout-ou-rien) plutôt qu'avec ses propres fichiers temporaires : un fichier à
//! moitié écrit est un bug qu'on ne voit qu'au prochain démarrage.
//!
//! Rien ici ne parle au système d'exploitation : pas de processus, pas de privilège, pas de
//! volume. Le stockage est une affaire de fichiers, et c'est tout (ADR-004).

pub mod history;
pub mod settings;

pub use history::{History, HistoryEntry, Operation, Outcome};
pub use settings::ObliNiscorSettings;

/// Erreur de persistance.
///
/// Une par crate de bibliothèque (`AGENTS.md`) : les réglages ont la leur, dans
/// `lpdnsoft-settings`, et l'historique a celle-ci. Les deux causes réelles sont le disque et
/// le format ; tout le reste en découle.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Le disque a refusé la lecture ou l'écriture.
    #[error("erreur d'entrée/sortie : {0}")]
    Io(#[from] std::io::Error),
    /// Une ligne d'historique n'a pas pu être écrite ou relue.
    #[error("erreur de sérialisation JSON : {0}")]
    Json(#[from] serde_json::Error),
    /// L'écriture atomique a échoué.
    #[error("écriture atomique impossible : {0}")]
    Atomic(#[from] lpdnsoft_atomic::AtomicError),
}
