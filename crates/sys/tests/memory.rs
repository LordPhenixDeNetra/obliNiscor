//! La sonde mémoire contre le vrai système.
//!
//! Ces tests vérifient des **cohérences**, pas des valeurs : la mémoire d'une machine change
//! entre deux exécutions, donc affirmer « 15 234 Mo disponibles » serait un test qui échoue au
//! hasard des machines. Ce qui doit toujours être vrai, en revanche, c'est que la mesure est
//! plausible et que l'absence de mesure est déclarée.

#![cfg(windows)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use obliniscor_platform::{MemoryProbe, MemorySnapshot};
use obliniscor_sys::windows::memory::{WindowsMemory, details};

#[test]
fn the_snapshot_is_plausible() {
    let snapshot = WindowsMemory.snapshot();
    assert!(snapshot.total > 0, "un système réel a de la mémoire");
    assert!(
        snapshot.available <= snapshot.total,
        "le disponible ({}) dépasse le total ({})",
        snapshot.available,
        snapshot.total
    );
    // Une machine qui démarre a au moins quelques mégaoctets disponibles.
    assert!(
        snapshot.available > 64 * 1024 * 1024,
        "disponible : {}",
        snapshot.available
    );
}

#[test]
fn the_free_memory_is_declared_absent_rather_than_guessed() {
    // Le spike a montré que `GlobalMemoryStatusEx` n'expose pas la mémoire libre : la remplir
    // avec `available` serait un mensonge, et `Some(0)` aussi. `None` est la seule réponse vraie.
    let snapshot: MemorySnapshot = WindowsMemory.snapshot();
    assert_eq!(
        snapshot.free, None,
        "Windows ne donne pas la mémoire libre sans la requête noyau"
    );
}

#[test]
fn the_details_are_converted_from_pages_to_bytes() {
    let details = details().expect("GetPerformanceInfo répond");
    assert!(
        details.page_size >= 4096,
        "taille de page : {}",
        details.page_size
    );
    // Le cache système dépasse toujours quelques pages ; en octets il doit dépasser 1 Mio.
    assert!(
        details.system_cache > 1024 * 1024,
        "cache système invraisemblable : {} octets",
        details.system_cache
    );
    assert!(details.kernel_total > 0);
}

#[test]
fn two_measurements_are_both_plausible() {
    // Deux appels successifs ne doivent pas se contredire grossièrement : c'est le contrôle de
    // répétabilité du protocole, réduit à ce qu'un test peut affirmer sans connaître la machine.
    let first = WindowsMemory.snapshot();
    let second = WindowsMemory.snapshot();
    assert_eq!(first.total, second.total, "la RAM totale ne change pas");
    let variation = first.available.abs_diff(second.available);
    assert!(
        variation < first.total / 10,
        "le disponible a varié de {variation} octets entre deux appels"
    );
}
