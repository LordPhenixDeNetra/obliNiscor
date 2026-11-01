//! macOS : mesure seule, aucun nettoyage (matrice 6.0, ADR-009).
//!
//! macOS mesure la mémoire (`host_statistics64`) et les processus (`libproc`), mais n'offre
//! **aucune API publique** pour libérer les caches mémoire ou réduire un working set : le
//! nettoyage mémoire y est déclaré indisponible, pas simulé. L'interface grise donc ces zones et
//! dit pourquoi ; aucun droit Apple ni extension système n'est demandé (SEC-14).
//!
//! **Table pure, aucun appel système.** Le module est compilé sur les trois plateformes pour que
//! ses valeurs soient vérifiées partout ; la mesure réelle viendra avec les tâches des backends
//! (ADR-004).

use obliniscor_core::{Availability, MemoryZone};

/// Disponibilité des zones mémoire sous macOS (matrice 6.0).
///
/// Mesure seule : aucune zone n'est nettoyable, donc toutes sont indisponibles.
pub const fn availability(zone: MemoryZone) -> Availability {
    match zone {
        // Pas de réduction de working set : aucun équivalent public de process_madvise (FR-PRC-3).
        MemoryZone::WorkingSet => Availability::Unavailable,
        // Pas d'API publique de purge des caches système.
        MemoryZone::SystemCache => Availability::Unavailable,
        // Pas de liste des pages modifiées exposée au nettoyage.
        MemoryZone::ModifiedList => Availability::Unavailable,
        // Pas de liste standby exposée au nettoyage.
        MemoryZone::Standby => Availability::Unavailable,
        // Pas de registre sous macOS : la zone n'existe pas (ADR-009).
        MemoryZone::RegistryCache => Availability::Unavailable,
    }
}
