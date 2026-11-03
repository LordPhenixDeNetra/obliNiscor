//! Windows 10/11/Server : la cible la plus complète de la matrice 6.0.
//!
//! Les cinq zones de [`MemoryZone`] sont mesurables et nettoyables ; c'est le seul système de
//! la v1 où le cache du registre existe (Windows 8.1 et suivants, FR-MEM-3). Une disponibilité
//! complète ne dit pas que l'opération est gratuite : elle demande des privilèges que le
//! processus n'a pas par défaut, activés puis retirés autour de chaque opération (FR-MEM-5).
//!
//! **Table pure, aucun appel système.** Le module est compilé sur les trois plateformes pour que
//! ses valeurs soient vérifiées partout ; les appels réels (requête noyau, compteurs, registre)
//! viendront avec les tâches des backends (ADR-004).
//!
//! **Sauf la FFI.** [`privilege`] et [`pipe`] lient `advapi32` et `kernel32` : ces
//! bibliothèques n'existent pas sous Linux ni macOS, et un `cargo test` les chercherait à
//! l'édition de liens de chaque binaire de test. Ces modules sont donc compilés **uniquement**
//! sous Windows — c'est la fin de la règle « tout est compilé partout », qui vaut pour les
//! tables, pas pour les appels.

#[cfg(windows)]
pub mod pipe;
#[cfg(windows)]
pub mod privilege;

use obliniscor_core::{Availability, MemoryZone};

/// Disponibilité des zones mémoire sous Windows (matrice 6.0).
///
/// Windows est le système où toutes les zones sont à la fois mesurables et nettoyables.
pub const fn availability(zone: MemoryZone) -> Availability {
    match zone {
        // Working set de tous les processus : réductible processus par processus (FR-PRC-3).
        MemoryZone::WorkingSet => Availability::Full,
        // Cache de fichiers du système : les pages propres peuvent être libérées (FR-MEM-3).
        MemoryZone::SystemCache => Availability::Full,
        // Liste des pages modifiées : exposée par le noyau, vidable (FR-MEM-3).
        MemoryZone::ModifiedList => Availability::Full,
        // Liste standby, complète ou basse priorité (FR-MEM-3).
        MemoryZone::Standby => Availability::Full,
        // Cache du registre : existe à partir de Windows 8.1 (FR-MEM-3), version à détecter
        // au moment de l'appel (FR-MEM-4).
        MemoryZone::RegistryCache => Availability::Full,
    }
}
