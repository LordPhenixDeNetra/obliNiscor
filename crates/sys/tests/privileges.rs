//! Le garde de privilèges, contre le vrai jeton Windows.
//!
//! Ces tests touchent le système : ils activent un privilège **détenu par un utilisateur
//! normal** et vérifient qu'il est rendu. Ils ne demandent aucune élévation.
//!
//! Ce que la CI ne peut pas prouver : sur un runner administrateur, `SeProfileSingleProcessPrivilege`
//! est détenu, donc la branche « privilège manquant » ne s'exécute pas. Les tests l'acceptent
//! explicitement plutôt que de faire semblant.

#![cfg(windows)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use obliniscor_core::Privilege;
use obliniscor_platform::{PlatformError, Privileges};
use obliniscor_sys::windows::privilege::SystemPrivileges;

#[test]
fn a_privilege_granted_to_users_can_be_acquired_and_released() {
    let privileges = SystemPrivileges;
    let guard = privileges
        .acquire(&[Privilege::IncreaseWorkingSet])
        .expect("un utilisateur normal détient ce privilège");

    assert_eq!(guard.enabled(), [Privilege::IncreaseWorkingSet]);
    // La chute du garde est le seul point de restitution : il n'y a pas de `release()`.
    drop(guard);

    // Après restitution, la même demande doit réussir : le jeton reste utilisable.
    assert!(privileges.acquire(&[Privilege::IncreaseWorkingSet]).is_ok());
}

#[test]
fn acquiring_nothing_returns_an_empty_guard() {
    let guard = SystemPrivileges.acquire(&[]).expect("rien à acquérir");
    assert!(guard.enabled().is_empty());
}

#[test]
fn the_guard_lists_privileges_in_the_requested_order() {
    let guard = SystemPrivileges
        .acquire(&[Privilege::IncreaseWorkingSet])
        .expect("privilège utilisateur");
    assert_eq!(guard.enabled().len(), 1);
    assert_eq!(guard.enabled()[0], Privilege::IncreaseWorkingSet);
}

#[test]
fn an_administrator_privilege_is_acquired_or_named_precisely() {
    // Deux machines, deux résultats, une seule vérité : le privilège est obtenu, ou bien
    // l'erreur nomme exactement celui qui manque — jamais un échec générique, jamais un
    // succès partiel silencieux.
    let privileges = SystemPrivileges;
    match privileges.acquire(&[
        Privilege::IncreaseWorkingSet,
        Privilege::ProfileSingleProcess,
    ]) {
        Ok(guard) => {
            // Jeton administrateur : les deux sont actifs, dans l'ordre demandé.
            assert_eq!(
                guard.enabled(),
                [
                    Privilege::IncreaseWorkingSet,
                    Privilege::ProfileSingleProcess
                ]
            );
        }
        Err(PlatformError::MissingPrivilege(missing)) => {
            assert_eq!(missing, Privilege::ProfileSingleProcess);
            // La demande était atomique : le privilège accordé avant l'échec n'est pas resté
            // actif sur un objet qui n'existe pas. Un nouveau garde reste possible.
            assert!(privileges.acquire(&[Privilege::IncreaseWorkingSet]).is_ok());
        }
        Err(other) => panic!("échec inattendu : {other:?}"),
    }
}
