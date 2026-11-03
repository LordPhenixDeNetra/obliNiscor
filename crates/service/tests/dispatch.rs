//! Répartition des requêtes, testée avec les doublures de la frontière.
//!
//! Aucun test ne touche au système : tout passe par `FakeMemory`, `FakeProcesses` et
//! `FakePrivileges`, ce qui permet de provoquer les cas difficiles à obtenir autrement — zone
//! indisponible, droits refusés, privilège manquant.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use obliniscor_core::{CleanReport, ErrorCode, MemorySnapshot, MemoryZone, Privilege};
use obliniscor_platform::ProcessInfo;
use obliniscor_platform::fakes::{FakeMemory, FakePrivileges, FakeProcesses, PrivilegeEvent};
use obliniscor_proto::Message;
use obliniscor_service::{Reply, Service};

fn service() -> Service<FakeMemory, FakePrivileges, FakeProcesses> {
    Service::new(
        FakeMemory::new(MemorySnapshot::default()),
        FakePrivileges::default(),
        FakeProcesses::new(),
    )
}

fn clean(zones: &[&str], dry_run: bool) -> Message {
    Message::RunMemoryClean {
        zones: zones.iter().map(|zone| (*zone).to_owned()).collect(),
        dry_run,
    }
}

#[test]
fn a_status_request_returns_what_the_probe_reports() {
    let snapshot = MemorySnapshot {
        total: 32_000,
        available: 12_000,
        free: 8_000,
    };
    let service = Service::new(
        FakeMemory::new(snapshot),
        FakePrivileges::default(),
        FakeProcesses::new(),
    );
    assert_eq!(service.handle(&Message::GetStatus), Reply::Status(snapshot));
}

#[test]
fn a_process_list_request_returns_the_processes() {
    let processes = vec![ProcessInfo {
        pid: 42,
        name: "explorer.exe".to_owned(),
        working_set: 1024,
    }];
    let service = Service::new(
        FakeMemory::new(MemorySnapshot::default()),
        FakePrivileges::default(),
        FakeProcesses::new().with_processes(processes.clone()),
    );
    assert_eq!(
        service.handle(&Message::ListProcesses),
        Reply::Processes(processes)
    );
}

#[test]
fn a_dry_run_cleans_without_asking_for_any_privilege() {
    let privileges = FakePrivileges::default();
    let service = Service::new(
        FakeMemory::new(MemorySnapshot::default()),
        privileges.clone(),
        FakeProcesses::new(),
    );

    let reply = service.handle(&clean(&["standby"], true));
    match reply {
        Reply::Cleaned(report) => assert_eq!(report.freed, 0, "une simulation ne libère rien"),
        other => panic!("réponse inattendue : {other:?}"),
    }
    assert!(
        privileges.events().is_empty(),
        "une simulation ne doit demander aucun droit"
    );
}

#[test]
fn a_real_clean_acquires_its_privilege_and_releases_it() {
    let privileges = FakePrivileges::default().with_held([Privilege::ProfileSingleProcess]);
    let service = Service::new(
        FakeMemory::new(MemorySnapshot::default()),
        privileges.clone(),
        FakeProcesses::new(),
    );

    let reply = service.handle(&clean(&["standby"], false));
    assert!(matches!(reply, Reply::Cleaned(_)));
    // Le garde tombe à la fin de `clean` : l'acquisition et la restitution sont tracées.
    assert_eq!(
        privileges.events(),
        [PrivilegeEvent::Acquired, PrivilegeEvent::Released]
    );
}

#[test]
fn a_missing_privilege_is_reported_as_denied() {
    // Le faux jeton ne porte rien : l'acquisition échoue, et le nettoyage n'a pas lieu.
    let service = Service::new(
        FakeMemory::new(MemorySnapshot::default()),
        FakePrivileges::default(),
        FakeProcesses::new(),
    );
    assert_eq!(
        service.handle(&clean(&["standby"], false)),
        Reply::Failed(ErrorCode::Denied)
    );
}

#[test]
fn an_unknown_zone_name_is_refused_without_any_action() {
    let privileges = FakePrivileges::default().with_held([Privilege::ProfileSingleProcess]);
    let service = Service::new(
        FakeMemory::new(MemorySnapshot::default()),
        privileges.clone(),
        FakeProcesses::new(),
    );

    assert_eq!(
        service.handle(&clean(&["liste-standby"], false)),
        Reply::Failed(ErrorCode::Protocol)
    );
    assert!(
        privileges.events().is_empty(),
        "un nom invalide ne doit rien déclencher"
    );
}

#[test]
fn an_empty_zone_list_is_refused() {
    let service = service();
    assert_eq!(
        service.handle(&clean(&[], false)),
        Reply::Failed(ErrorCode::Protocol)
    );
}

#[test]
fn an_unavailable_zone_is_reported_as_unavailable() {
    let service = Service::new(
        FakeMemory::new(MemorySnapshot::default()).with_unavailable_zone(MemoryZone::RegistryCache),
        FakePrivileges::default().with_held([Privilege::ProfileSingleProcess]),
        FakeProcesses::new(),
    );
    assert_eq!(
        service.handle(&clean(&["registry-cache"], false)),
        Reply::Failed(ErrorCode::Unavailable)
    );
}

#[test]
fn a_denied_cleaner_becomes_a_denied_code() {
    let service = Service::new(
        FakeMemory::new(MemorySnapshot::default()).with_denied(),
        FakePrivileges::default().with_held([Privilege::ProfileSingleProcess]),
        FakeProcesses::new(),
    );
    assert_eq!(
        service.handle(&clean(&["standby"], false)),
        Reply::Failed(ErrorCode::Denied)
    );
}

#[test]
fn a_progress_message_is_a_protocol_error() {
    // La progression va du service vers le client : la recevoir n'a pas de sens.
    let service = service();
    assert_eq!(
        service.handle(&Message::Progress {
            done: 1,
            total: Some(2)
        }),
        Reply::Failed(ErrorCode::Protocol)
    );
}

#[test]
fn several_zones_are_all_passed_to_the_cleaner_in_order() {
    let service = Service::new(
        FakeMemory::new(MemorySnapshot::default()),
        FakePrivileges::default().with_held([Privilege::ProfileSingleProcess]),
        FakeProcesses::new(),
    );
    match service.handle(&clean(&["working-set", "standby"], false)) {
        Reply::Cleaned(CleanReport { zones, .. }) => {
            assert_eq!(zones, [MemoryZone::WorkingSet, MemoryZone::Standby]);
        }
        other => panic!("réponse inattendue : {other:?}"),
    }
}
