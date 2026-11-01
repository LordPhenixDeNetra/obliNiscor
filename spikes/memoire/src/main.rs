//! Spike P0 — état de la mémoire sous Windows, **en lecture seule**.
//!
//! Binaire d'exploration jetable, hors du code de production (cahier des charges, OBL-005) :
//! il a le droit d'utiliser `unsafe` et de déclarer ses propres prototypes, ce que le
//! workspace interdit partout ailleurs.
//!
//! Il répond à une seule question, celle du spike : *quelles valeurs la documentation permet
//! d'obtenir, et sont-elles cohérentes entre elles ?* Il ne nettoie rien. Les commandes de
//! purge existent (`--help`) mais ne sont jamais lancées par défaut.
//!
//! Usage :
//!   cargo run --release            # une mesure
//!   cargo run --release -- --repeat 5 --interval-ms 1000

use std::process::ExitCode;
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

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "spike-memoire — lecture de l'état mémoire de Windows\n\
             \n  --repeat N        nombre de mesures (défaut 1)\
             \n  --interval-ms N   attente entre deux mesures (défaut 1000)\
             \n\nAucune purge n'est implémentée ici : ce spike observe, il ne modifie rien."
        );
        return ExitCode::SUCCESS;
    }

    let repeat = value_of(&args, "--repeat").unwrap_or(1).max(1);
    let interval = value_of(&args, "--interval-ms").unwrap_or(1000);

    println!(
        "{{\"tool\":\"spike-memoire\",\"os\":\"{}\",\"build\":\"{}\",\"measurements\":[",
        std::env::consts::OS,
        build()
    );
    for index in 0..repeat {
        let status = memory_status();
        let perf = performance();
        let separator = if index + 1 == repeat { "" } else { "," };
        match (status, perf) {
            (Some(status), Some(perf)) => {
                // `PERFORMANCE_INFORMATION` compte en **pages**, pas en octets : sans cette
                // multiplication, le cache système paraît 4096 fois trop petit. C'est le
                // premier piège que le spike a trouvé, et il est silencieux.
                let page = perf.page_size as u64;
                println!(
                    "{{\"load_pct\":{},\"total\":{},\"available\":{},\"commit_available\":{},\
                     \"commit_total\":{},\"commit_limit\":{},\"commit_peak\":{},\"system_cache\":{},\
                     \"kernel_total\":{},\"kernel_paged\":{},\"kernel_nonpaged\":{},\
                     \"page_size\":{},\"processes\":{},\"threads\":{},\"handles\":{}}}{separator}",
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
            _ => {
                eprintln!("échec d'un appel système à la mesure {index}");
                println!("{{}}{separator}");
            }
        }
        if index + 1 < repeat {
            sleep(Duration::from_millis(interval));
        }
    }
    println!("]}}");
    ExitCode::SUCCESS
}

/// `GlobalMemoryStatusEx` — RAM physique et fichier d'échange.
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

fn value_of(args: &[String], name: &str) -> Option<u64> {
    let position = args.iter().position(|arg| arg == name)?;
    args.get(position + 1)?.parse().ok()
}
