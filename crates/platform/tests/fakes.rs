//! Tests de la frontière système : les doublures renvoient exactement ce qu'on leur a
//! configuré, signalent une capacité absente et un refus de droits, et s'injectent derrière
//! `&dyn` — la preuve que le noyau peut être testé sans Windows ni privilèges.

use obliniscor_core::MemoryZone;
use obliniscor_platform::fakes::{DENIED_MESSAGE, FakeMemory, FakeProcesses, FakeVolumes};
use obliniscor_platform::{
    CleanReport, DiskInventory, MemoryCleaner, MemoryProbe, MemorySnapshot, PlatformError,
    ProcessControl, ProcessInfo, Volume,
};

const KIB: u64 = 1024;
const MIB: u64 = 1024 * KIB;
const GIB: u64 = 1024 * MIB;

fn configured_snapshot() -> MemorySnapshot {
    MemorySnapshot {
        total: 32 * GIB,
        available: 12 * GIB,
        free: Some(4 * GIB),
    }
}

fn configured_processes() -> Vec<ProcessInfo> {
    vec![
        ProcessInfo {
            pid: 4321,
            name: "obliniscord".to_owned(),
            working_set: 8 * MIB,
        },
        ProcessInfo {
            pid: 8765,
            name: "explorer".to_owned(),
            working_set: 64 * MIB,
        },
    ]
}

fn configured_volumes() -> Vec<Volume> {
    vec![
        Volume {
            mount: "C:\\".to_owned(),
            total: 512 * GIB,
            free: 128 * GIB,
        },
        Volume {
            mount: "/".to_owned(),
            total: 256 * GIB,
            free: 32 * GIB,
        },
    ]
}

#[test]
fn a_fake_returns_exactly_what_was_configured() {
    let snapshot = configured_snapshot();
    let processes = configured_processes();
    let volumes = configured_volumes();
    let zones = [MemoryZone::WorkingSet, MemoryZone::SystemCache];

    let memory = FakeMemory::new(snapshot).with_freed(3 * MIB);
    assert_eq!(memory.snapshot(), snapshot);

    // Un nettoyage réel renvoie le gain configuré et les zones demandées, dans l'ordre.
    assert_eq!(
        memory.clean(&zones, false),
        Ok(CleanReport {
            freed: 3 * MIB,
            zones: zones.to_vec(),
        }),
        "le rapport doit reprendre le gain et les zones configurés"
    );
    // Une simulation ne libère rien mais décrit quand même les zones demandées.
    assert_eq!(
        memory.clean(&zones, true),
        Ok(CleanReport {
            freed: 0,
            zones: zones.to_vec(),
        })
    );

    let process_control = FakeProcesses::new().with_processes(processes.clone());
    assert_eq!(process_control.list(), Ok(processes));

    let disks = FakeVolumes::new().with_volumes(volumes.clone());
    assert_eq!(disks.volumes(), Ok(volumes));
}

#[test]
fn an_unavailable_zone_is_an_error_not_a_silent_success() {
    let memory =
        FakeMemory::new(configured_snapshot()).with_unavailable_zone(MemoryZone::RegistryCache);

    // La zone absente est demandée : on refuse, on ne fait pas semblant.
    let requested = [MemoryZone::SystemCache, MemoryZone::RegistryCache];
    assert_eq!(
        memory.clean(&requested, false),
        Err(PlatformError::Unavailable(MemoryZone::RegistryCache)),
        "une zone indisponible ne doit jamais devenir un succès silencieux"
    );
    // La simulation ne doit pas mentir davantage sur la capacité.
    assert_eq!(
        memory.clean(&requested, true),
        Err(PlatformError::Unavailable(MemoryZone::RegistryCache)),
        "une simulation ne rend pas disponible une zone qui ne l'est pas"
    );

    // Les zones disponibles restent utilisables : c'est la capacité qui manque à une zone.
    let supported = [MemoryZone::SystemCache];
    assert_eq!(
        memory.clean(&supported, false),
        Ok(CleanReport {
            freed: 0,
            zones: supported.to_vec(),
        })
    );
}

#[test]
fn a_denied_access_is_reported_as_denied() {
    let denied = PlatformError::Denied(DENIED_MESSAGE.to_owned());

    let memory = FakeMemory::new(configured_snapshot()).with_denied();
    assert_eq!(
        memory.clean(&[MemoryZone::Standby], false),
        Err(denied.clone())
    );
    // Un refus en simulation reste un refus : les droits sont vérifiés avant d'agir.
    assert_eq!(
        memory.clean(&[MemoryZone::Standby], true),
        Err(denied.clone())
    );

    let process_control = FakeProcesses::new().with_denied();
    assert_eq!(process_control.list(), Err(denied.clone()));

    let disks = FakeVolumes::new().with_denied();
    assert_eq!(disks.volumes(), Err(denied));
}

/// Assemblage typique du noyau : quatre capacités injectées séparément, vues comme des
/// objets de trait. Si un trait cessait d'être utilisable en `&dyn`, ce test ne compilerait
/// plus — c'est précisément la propriété à garantir.
#[derive(Debug, PartialEq, Eq)]
struct Status {
    available: u64,
    freed: u64,
    process_count: usize,
    volume_count: usize,
}

struct Host<'a> {
    probe: &'a dyn MemoryProbe,
    cleaner: &'a dyn MemoryCleaner,
    processes: &'a dyn ProcessControl,
    disks: &'a dyn DiskInventory,
}

impl Host<'_> {
    fn status(&self) -> Status {
        Status {
            available: self.probe.snapshot().available,
            freed: self
                .cleaner
                .clean(&[MemoryZone::SystemCache], false)
                .map_or(0, |report| report.freed),
            process_count: self.processes.list().map_or(0, |processes| processes.len()),
            volume_count: self.disks.volumes().map_or(0, |volumes| volumes.len()),
        }
    }
}

#[test]
fn fakes_are_injectable_behind_trait_objects() {
    let memory = FakeMemory::new(configured_snapshot()).with_freed(2 * MIB);
    let processes = FakeProcesses::new().with_processes(configured_processes());
    let disks = FakeVolumes::new().with_volumes(configured_volumes());

    let host = Host {
        probe: &memory,
        cleaner: &memory,
        processes: &processes,
        disks: &disks,
    };

    assert_eq!(
        host.status(),
        Status {
            available: 12 * GIB,
            freed: 2 * MIB,
            process_count: 2,
            volume_count: 2,
        }
    );
}

#[test]
fn privileges_are_released_when_the_guard_falls() {
    use obliniscor_core::Privilege;
    use obliniscor_platform::Privileges;
    use obliniscor_platform::fakes::{FakePrivileges, PrivilegeEvent};

    // Le cas dangereux du cahier des charges : un privilège activé qui n'est pas rendu.
    let fake = FakePrivileges::default().with_held([Privilege::IncreaseWorkingSet]);
    {
        let guard = fake
            .acquire(&[Privilege::IncreaseWorkingSet])
            .expect("le faux jeton le porte");
        assert_eq!(guard.enabled(), [Privilege::IncreaseWorkingSet]);
        assert_eq!(fake.events(), [PrivilegeEvent::Acquired]);
    }
    // La chute du garde est la seule restitution : elle est tracée.
    assert_eq!(
        fake.events(),
        [PrivilegeEvent::Acquired, PrivilegeEvent::Released]
    );
}

#[test]
fn a_missing_privilege_activates_nothing() {
    use obliniscor_core::Privilege;
    use obliniscor_platform::PlatformError;
    use obliniscor_platform::Privileges;
    use obliniscor_platform::fakes::{FakePrivileges, PrivilegeEvent};

    // Demande de deux privilèges, un seul détenu : rien ne doit rester actif, et l'erreur doit
    // nommer celui qui manque — pas un échec générique.
    let fake = FakePrivileges::default().with_held([Privilege::IncreaseWorkingSet]);
    let result = fake.acquire(&[
        Privilege::IncreaseWorkingSet,
        Privilege::ProfileSingleProcess,
    ]);
    assert_eq!(
        result.err(),
        Some(PlatformError::MissingPrivilege(
            Privilege::ProfileSingleProcess
        ))
    );
    assert_eq!(fake.events(), [PrivilegeEvent::Acquired; 0]);
}
