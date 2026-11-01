//! Backend Windows : `GlobalMemoryStatusEx`, `GetPerformanceInfo`, `RtlGetVersion`.

use std::thread::sleep;
use std::time::Duration;

/// `MEMORYSTATUSEX`
#[repr(C)]
#[derive(Default)]
struct MemoryStatusEx {
    length: u32,
    memory_load: u32,
    total_phys: u64,
    avail_phys: u64,
    total_page_file: u64,
    avail_page_file: u64,
    total_virtual: u64,
    avail_virtual: u64,
    avail_extended_virtual: u64,
}

/// `PERFORMANCE_INFORMATION` (psapi)
#[repr(C)]
#[derive(Default)]
struct PerformanceInformation {
    size: u32,
    commit_total: usize,
    commit_limit: usize,
    commit_peak: usize,
    physical_total: usize,
    physical_available: usize,
    system_cache: usize,
    kernel_total: usize,
    kernel_paged: usize,
    kernel_nonpaged: usize,
    page_size: usize,
    handle_count: u32,
    process_count: u32,
    thread_count: u32,
}

#[repr(C)]
struct OsVersionInfoW {
    size: u32,
    major: u32,
    minor: u32,
    build: u32,
    platform: u32,
    csd: [u16; 128],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
}

// `GetPerformanceInfo` vit dans psapi.dll, pas dans kernel32 : le lieur le signale, pas le
// compilateur — c'est exactement le genre de détail que le spike doit rencontrer avant que le
// code de production ne le suppose.
#[link(name = "psapi")]
unsafe extern "system" {
    fn GetPerformanceInfo(info: *mut PerformanceInformation, size: u32) -> i32;
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlGetVersion(version: *mut OsVersionInfoW) -> i32;
}

pub fn measure() -> String {
    let Some(status) = memory_status() else {
        return "{\"error\":\"GlobalMemoryStatusEx a échoué\"}".to_owned();
    };
    let Some(perf) = performance() else {
        return "{\"error\":\"GetPerformanceInfo a échoué\"}".to_owned();
    };

    // `PERFORMANCE_INFORMATION` compte en **pages**, pas en octets : sans cette
    // multiplication, le cache système paraît 4096 fois trop petit. Premier piège trouvé par
    // ce spike, et il est silencieux.
    let page = perf.page_size as u64;
    format!(
        "{{\"build\":\"{}\",\"load_pct\":{},\"total\":{},\"available\":{},\
         \"commit_available\":{},\"commit_total\":{},\"commit_limit\":{},\"commit_peak\":{},\
         \"system_cache\":{},\"kernel_total\":{},\"kernel_paged\":{},\"kernel_nonpaged\":{},\
         \"page_size\":{},\"processes\":{},\"threads\":{},\"handles\":{}}}",
        build(),
        status.memory_load,
        status.total_phys,
        status.avail_phys,
        status.avail_page_file,
        perf.commit_total as u64 * page,
        perf.commit_limit as u64 * page,
        perf.commit_peak as u64 * page,
        perf.system_cache as u64 * page,
        perf.kernel_total as u64 * page,
        perf.kernel_paged as u64 * page,
        perf.kernel_nonpaged as u64 * page,
        perf.page_size,
        perf.process_count,
        perf.thread_count,
        perf.handle_count
    )
}

/// `GlobalMemoryStatusEx` — RAM physique et charge de commit.
fn memory_status() -> Option<MemoryStatusEx> {
    let mut status = MemoryStatusEx {
        length: size_of::<MemoryStatusEx>() as u32,
        ..Default::default()
    };
    // SAFETY: `status` est une structure de la taille annoncée dans `length`, et l'appel ne
    // conserve pas le pointeur au-delà du retour.
    let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
    (ok != 0).then_some(status)
}

/// `GetPerformanceInfo` — charge de commit, cache système, compteurs d'objets.
fn performance() -> Option<PerformanceInformation> {
    let mut info = PerformanceInformation {
        size: size_of::<PerformanceInformation>() as u32,
        ..Default::default()
    };
    // SAFETY: structure de la taille annoncée dans `size`, pointeur valide le temps de l'appel.
    let ok = unsafe { GetPerformanceInfo(&mut info, info.size) };
    (ok != 0).then_some(info)
}

/// Version et build réels du noyau, sans passer par un manifeste qui pourrait mentir.
fn build() -> String {
    let mut version = OsVersionInfoW {
        size: size_of::<OsVersionInfoW>() as u32,
        major: 0,
        minor: 0,
        build: 0,
        platform: 0,
        csd: [0; 128],
    };
    // SAFETY: `version` est zéro-initialisé et sa taille est annoncée ; RtlGetVersion ne
    // conserve pas le pointeur.
    let status = unsafe { RtlGetVersion(&mut version) };
    if status < 0 {
        return "inconnu".to_owned();
    }
    format!("{}.{}.{}", version.major, version.minor, version.build)
}

/// Les appels ne servent pas dans ce backend ; ce `sleep` évite un avertissement de
/// compilation inutile si le module évolue.
#[allow(dead_code)]
fn pause(milliseconds: u64) {
    sleep(Duration::from_millis(milliseconds));
}
