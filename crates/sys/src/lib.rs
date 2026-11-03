//! Couche système d'obliNiscor : la seule couche qui parlera au système.
//!
//! Le crate est découpé par système d'exploitation — `windows`, `linux`, `macos` — comme le
//! demande l'ADR-004 : ajouter un système est un module, pas une refonte. Les trois modules sont
//! **toujours compilés**, quel que soit l'hôte, et la sélection se fait à l'exécution avec
//! [`current()`]. C'est ce qui permet de vérifier les trois tables depuis n'importe quelle
//! machine de la CI.
//!
//! Cette étape est un **squelette** : aucune fonction du crate n'appelle le système. Elle rend
//! seulement exécutable la matrice de disponibilité 6.0 du cahier des charges, celle que
//! l'interface, le noyau et les tests citeront (ADR-009). La table répond à la question « cette
//! zone peut-elle être **nettoyée** ? » : macOS mesure la mémoire globale, mais ne nettoie
//! aucune zone, donc toutes ses zones sont [`Availability::Unavailable`].
//!
//! La gestion des privilèges ([`windows::privilege`]) est la première incursion d'`unsafe` :
//! elle est dérogatoire, limitée à ce crate, et chaque bloc porte sa justification `SAFETY:`
//! (`undocumented_unsafe_blocks` est en erreur, SEC-9).

// `unsafe` est permis dans ce crate et **seulement** dans ce crate (AGENTS.md, SEC-9).
#![deny(unsafe_op_in_unsafe_fn)]

pub mod linux;
pub mod macos;
pub mod windows;

// Les types de la matrice viennent du noyau ; les réexporter évite à un appelant de dépendre de
// `obliniscor-core` uniquement pour lire une table de disponibilité.
pub use obliniscor_core::{Availability, MemoryZone};

/// Nom du module système retenu pour la plateforme courante.
///
/// Renvoie `"windows"`, `"linux"` ou `"macos"`, c'est-à-dire le module dont la table est
/// appliquée par [`availability`]. La sélection se fait avec `cfg!`, pas avec un `#[cfg]` de
/// compilation : les trois modules restent compilés et testables partout.
///
/// Un système que la v1 ne couvre pas retombe sur `"macos"`, dont la table est la plus
/// restrictive (tout indisponible) : par défaut, on ne promet jamais un nettoyage qui n'existe
/// pas.
pub const fn current() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "macos"
    }
}

/// Disponibilité de `zone` sur le système courant.
///
/// Délègue à la table du module désigné par [`current()`] ; c'est la seule porte d'entrée dont
/// les appelants ont besoin. Une zone absente du système n'est pas une erreur : c'est
/// [`Availability::Unavailable`] (ADR-009).
///
/// # Exemple
///
/// ```
/// use obliniscor_sys::{MemoryZone, availability, current};
///
/// // Le cache du registre n'existe que sous Windows.
/// let cleanable = availability(MemoryZone::RegistryCache).can_clean();
/// assert_eq!(cleanable, current() == "windows");
/// ```
pub const fn availability(zone: MemoryZone) -> Availability {
    if cfg!(target_os = "windows") {
        windows::availability(zone)
    } else if cfg!(target_os = "linux") {
        linux::availability(zone)
    } else {
        macos::availability(zone)
    }
}
