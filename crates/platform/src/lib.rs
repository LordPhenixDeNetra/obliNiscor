//! Frontière testable entre le noyau d'obliNiscor et le système d'exploitation.
//!
//! Ce crate ne fait **aucun appel système** : il ne contient que des traits de capacité, les
//! types qu'ils échangent, et des doublures de test ([`fakes`]). La couche `obliniscor-sys`
//! décide *comment* lire la mémoire, nettoyer une zone ou énumérer les volumes, système par
//! système (ADR-004) ; le noyau décide *quoi* faire et *quand*. Cette séparation est ce qui
//! permet de tester le noyau sur une machine qui n'est pas Windows, et de déclarer une
//! fonction absente plutôt que de la simuler (ADR-009).
//!
//! Les traits sont volontairement **petits et orientés capacité** : un trait par verbe
//! (`snapshot`, `clean`, `list`, `volumes`). Un trait unique regroupant toutes les opérations
//! obligerait chaque doublure à implémenter ce qu'elle ne teste pas, et rendrait les tests du
//! noyau plus pénibles à écrire qu'à lire. Chaque trait reste utilisable derrière `&dyn` ou
//! `Box<dyn>`, ce qui est la condition de l'injection.
//!
//! Une capacité absente n'est pas une erreur d'exécution : le noyau l'apprend par
//! [`obliniscor_core::Availability`] et n'appelle pas le backend concerné. Si un backend
//! constate malgré tout l'absence, il renvoie [`PlatformError::Unavailable`] — jamais un
//! succès silencieux.

use obliniscor_core::MemoryZone;
use thiserror::Error;

pub mod fakes;

// Les modèles vivent dans le noyau : `obliniscor-platform` ne définit que les **traits** et
// leurs doublures. Les réexporter ici évite aux appelants de dépendre du noyau pour lire un
// simple instantané, et laisse une seule définition, celle que le noyau documente.
pub use obliniscor_core::{CleanReport, MemorySnapshot, ProcessInfo, Volume};
/// Erreur d'un backend de la frontière système.
///
/// Trois causes seulement, distinctes pour l'utilisateur comme pour le noyau :
/// une capacité qui n'existe pas sur ce système n'est pas un échec, et un refus de droits
/// n'est pas une panne.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlatformError {
    /// La fonction n'existe pas sur ce système (ADR-009).
    ///
    /// C'est l'écho d'`Availability::Unavailable` : on ne simule pas, on déclare. Une zone
    /// déclarée indisponible ne doit jamais être annoncée comme nettoyable.
    #[error("zone indisponible sur ce système : {0:?}")]
    Unavailable(MemoryZone),
    /// Le système a refusé l'opération faute de droits.
    #[error("accès refusé : {0}")]
    Denied(String),
    /// L'opération a échoué pour une autre raison, décrite par le backend.
    #[error("échec de l'opération système : {0}")]
    Failed(String),
}

/// Capacité de lecture de l'état de la mémoire.
///
/// La mesure est une lecture non privilégiée : le trait est infaillible. Un backend qui ne
/// sait pas la produire ne doit pas être appelé ; c'est au noyau de le savoir par
/// `Availability` avant l'appel.
pub trait MemoryProbe {
    /// Relève l'état courant de la mémoire.
    fn snapshot(&self) -> MemorySnapshot;
}

/// Capacité de nettoyage de zones mémoire.
///
/// `dry_run` à `true` demande une simulation : le backend valide la demande et décrit ce
/// qu'il ferait, mais ne libère rien. Une zone absente du système renvoie
/// [`PlatformError::Unavailable`] même en simulation : déclarer une capacité qu'on n'a pas
/// serait exactement le mensonge que l'ADR-009 interdit.
pub trait MemoryCleaner {
    /// Nettoie `zones`, ou simule le nettoyage quand `dry_run` est vrai.
    fn clean(&self, zones: &[MemoryZone], dry_run: bool) -> Result<CleanReport, PlatformError>;
}

/// Capacité d'énumération des processus.
pub trait ProcessControl {
    /// Liste les processus visibles par l'appelant.
    fn list(&self) -> Result<Vec<ProcessInfo>, PlatformError>;
}

/// Capacité d'inventaire des volumes montés.
pub trait DiskInventory {
    /// Liste les volumes montés sur le système.
    fn volumes(&self) -> Result<Vec<Volume>, PlatformError>;
}
