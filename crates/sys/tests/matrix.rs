//! Matrice de disponibilité 6.0 rendue exécutable : les trois tables sont vérifiées zone par
//! zone, sur toutes les plateformes.
//!
//! Ces tests sont volontairement exhaustifs : chaque zone et chaque système ont leur propre
//! assertion, donc modifier une seule entrée d'une table casse la suite (critère d'acceptation
//! de la tâche). Ils ne testent que l'API publique du crate : `windows`, `linux`, `macos` et la
//! sélection du système courant.

use obliniscor_sys::{Availability, MemoryZone, availability, current, linux, macos, windows};

/// Les cinq zones connues du noyau, dans l'ordre de `MemoryZone`.
const ZONES: [MemoryZone; 5] = [
    MemoryZone::WorkingSet,
    MemoryZone::SystemCache,
    MemoryZone::ModifiedList,
    MemoryZone::Standby,
    MemoryZone::RegistryCache,
];

/// Matrice 6.0 du cahier des charges : `(zone, Windows, Linux, macOS)`.
///
/// C'est la table de référence : le crate ne fait que la publier, ces tests la comparent
/// entrée par entrée.
const MATRIX: [(MemoryZone, Availability, Availability, Availability); 5] = [
    (
        MemoryZone::WorkingSet,
        Availability::Full,
        Availability::Partial,
        Availability::Unavailable,
    ),
    (
        MemoryZone::SystemCache,
        Availability::Full,
        Availability::Partial,
        Availability::Unavailable,
    ),
    (
        MemoryZone::ModifiedList,
        Availability::Full,
        Availability::Partial,
        Availability::Unavailable,
    ),
    (
        MemoryZone::Standby,
        Availability::Full,
        Availability::Partial,
        Availability::Unavailable,
    ),
    (
        MemoryZone::RegistryCache,
        Availability::Full,
        Availability::Unavailable,
        Availability::Unavailable,
    ),
];

/// La table du module nommé par [`current()`], ou `None` si le nom n'est pas couvert.
fn table_named(name: &str) -> Option<fn(MemoryZone) -> Availability> {
    match name {
        "windows" => Some(windows::availability),
        "linux" => Some(linux::availability),
        "macos" => Some(macos::availability),
        _ => None,
    }
}

#[test]
fn the_three_tables_follow_the_specification_zone_by_zone() {
    for (zone, expected_windows, expected_linux, expected_macos) in MATRIX {
        assert_eq!(
            windows::availability(zone),
            expected_windows,
            "Windows, zone {zone:?}"
        );
        assert_eq!(
            linux::availability(zone),
            expected_linux,
            "Linux, zone {zone:?}"
        );
        assert_eq!(
            macos::availability(zone),
            expected_macos,
            "macOS, zone {zone:?}"
        );
    }
}

#[test]
fn every_zone_has_exactly_one_row_in_the_matrix() {
    assert_eq!(
        MATRIX.len(),
        ZONES.len(),
        "la matrice de référence doit couvrir toutes les zones"
    );
    for zone in ZONES {
        let rows = MATRIX
            .iter()
            .filter(|(row_zone, ..)| *row_zone == zone)
            .count();
        assert_eq!(rows, 1, "zone {zone:?} : lignes trouvées dans la matrice");
    }
}

#[test]
fn macos_never_offers_a_cleaning_action() {
    // ADR-009 : une fonction absente n'est pas simulée. Le nettoyage mémoire n'existe pas sous
    // macOS, donc aucune zone ne doit s'annoncer comme nettoyable.
    for zone in ZONES {
        assert!(
            !macos::availability(zone).can_clean(),
            "macOS ne peut pas nettoyer la zone {zone:?}"
        );
    }
}

#[test]
fn current_names_the_machine_running_the_tests() {
    let name = current();
    assert!(
        table_named(name).is_some(),
        "current() renvoie un système inconnu : {name}"
    );

    let expected = if cfg!(target_os = "windows") {
        Some("windows")
    } else if cfg!(target_os = "linux") {
        Some("linux")
    } else if cfg!(target_os = "macos") {
        Some("macos")
    } else {
        None
    };
    if let Some(expected) = expected {
        assert_eq!(name, expected, "current() doit nommer le système de test");
    }
}

#[test]
fn the_public_function_uses_the_table_named_by_current() {
    let table = table_named(current()).expect("current() doit nommer un système couvert");
    for zone in ZONES {
        assert_eq!(
            availability(zone),
            table(zone),
            "availability() doit déléguer à current(), zone {zone:?}"
        );
    }
}

#[test]
fn the_public_function_only_ever_returns_one_of_the_three_tables() {
    // Aucun quatrième module ne peut exister : la fonction publique renvoie toujours la valeur
    // de Windows, de Linux ou de macOS, jamais une disponibilité inventée.
    for zone in ZONES {
        let row = MATRIX
            .iter()
            .find(|(row_zone, ..)| *row_zone == zone)
            .expect("chaque zone doit avoir sa ligne dans la matrice");
        let known = [row.1, row.2, row.3];
        assert!(
            known.contains(&availability(zone)),
            "zone {zone:?} : disponibilité hors des trois tables connues"
        );
    }
}
