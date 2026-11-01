//! Linux : mémoire mesurable partout, nettoyage partiel et réservé à root (matrice 6.0).
//!
//! La mesure est complète (`/proc/meminfo`, `/proc/<pid>/` — FR-MEM-1, 2), mais le nettoyage ne
//! l'est pas : les caches système et le working set ne se libèrent qu'avec des droits root,
//! l'opération est désactivée par défaut, et le noyau récupère de toute façon seul la mémoire.
//! Les listes de Windows n'ont pas toutes un équivalent nommé, et il n'y a pas de registre :
//! [`Availability::Partial`] et [`Availability::Unavailable`] disent exactement cela, sans rien
//! simuler (ADR-009).
//!
//! **Table pure, aucun appel système.** Les appels réels (lecture de `/proc`, `sync`, réglages
//! `vm.*`) viendront avec les tâches des backends (ADR-004).

use obliniscor_core::{Availability, MemoryZone};

/// Disponibilité des zones mémoire sous Linux (matrice 6.0).
///
/// Quatre zones sont mesurables et nettoyables avec des droits particuliers ; le cache du
/// registre n'existe pas.
pub const fn availability(zone: MemoryZone) -> Availability {
    match zone {
        // Working set : mesurable, mais sa réduction (process_madvise, MADV_PAGEOUT) exige des
        // droits et le noyau peut annuler le gain (FR-PRC-3).
        MemoryZone::WorkingSet => Availability::Partial,
        // Caches système : purgeables, réservés à root et déconseillés en fonctionnement
        // courant.
        MemoryZone::SystemCache => Availability::Partial,
        // Pages modifiées : écrites par le noyau quand il le décide ; l'action ciblée demande
        // des droits et n'a pas d'équivalent exact de la liste Windows.
        MemoryZone::ModifiedList => Availability::Partial,
        // Mémoire inactive : pas de liste standby nommée ; le noyau la récupère seul, l'action
        // ciblée reste partielle.
        MemoryZone::Standby => Availability::Partial,
        // Aucun registre sous Linux : la zone n'existe pas et n'est jamais annoncée (ADR-009).
        MemoryZone::RegistryCache => Availability::Unavailable,
    }
}
