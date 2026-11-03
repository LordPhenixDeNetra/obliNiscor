//! Sonde mémoire Windows : `GlobalMemoryStatusEx` pour la RAM, `GetPerformanceInfo` pour le reste.
//!
//! Les deux appels ont été mesurés par le spike OBL-005, qui a trouvé deux pièges que ce module
//! évite explicitement :
//!
//! 1. `PERFORMANCE_INFORMATION` compte en **pages**, pas en octets — d'où la multiplication par
//!    `PageSize`, faite une seule fois et nommée.
//! 2. `GlobalMemoryStatusEx` **n'a pas de champ « mémoire libre »**. Ce module ne l'invente pas :
//!    il rend `free: None`, et la requête noyau qui la donnera arrive avec OBL-022.
//!
//! La sonde est infaillible par contrat ([`MemoryProbe`]) : si un appel échoue, elle rend un
//! instantané vide plutôt qu'une valeur fausse. Un zéro est visible ; un chiffre plausible mais
//! faux ne l'est pas.

use obliniscor_core::MemorySnapshot;
use obliniscor_platform::MemoryProbe;

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

/// `PERFORMANCE_INFORMATION` (psapi) — les tailles sont en **pages**.
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

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
}

// `GetPerformanceInfo` vit dans psapi.dll : le compilateur ne le voit pas, seul le lieur le
// signale (LNK2019) — constaté par le spike.
#[link(name = "psapi")]
unsafe extern "system" {
    fn GetPerformanceInfo(info: *mut PerformanceInformation, size: u32) -> i32;
}

/// La sonde mémoire de Windows, sans état.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsMemory;

impl MemoryProbe for WindowsMemory {
    fn snapshot(&self) -> MemorySnapshot {
        let mut status = MemoryStatusEx {
            length: size_of::<MemoryStatusEx>() as u32,
            ..Default::default()
        };
        // SAFETY: `status` est une structure de la taille annoncée dans `length`, et l'appel ne
        // conserve pas le pointeur au-delà du retour.
        let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
        if ok == 0 {
            // Un instantané vide plutôt qu'un chiffre inventé : l'appelant verra `total == 0`.
            return MemorySnapshot::default();
        }

        MemorySnapshot {
            total: status.total_phys,
            available: status.avail_phys,
            // `None` et non `available` : les deux valeurs ne veulent pas dire la même chose, et
            // le spike mémoire a montré ce que coûte la confusion.
            free: None,
        }
    }
}

/// Détails que seul `GetPerformanceInfo` fournit, pour les diagnostics à venir.
///
/// Rend `None` si l'appel échoue. Les valeurs sont converties en octets ici, une fois.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryDetails {
    /// Cache de fichiers du système, en octets.
    pub system_cache: u64,
    /// Mémoire noyau totale, en octets.
    pub kernel_total: u64,
    /// Taille de page réellement rapportée par le système.
    pub page_size: u64,
}

/// Lit les détails mémoire ; `None` si le système refuse de répondre.
#[must_use]
pub fn details() -> Option<MemoryDetails> {
    let mut info = PerformanceInformation {
        size: size_of::<PerformanceInformation>() as u32,
        ..Default::default()
    };
    // SAFETY: structure de la taille annoncée dans `size`, pointeur valide le temps de l'appel.
    let ok = unsafe { GetPerformanceInfo(&mut info, info.size) };
    if ok == 0 {
        return None;
    }
    let page = info.page_size as u64;
    Some(MemoryDetails {
        system_cache: info.system_cache as u64 * page,
        kernel_total: info.kernel_total as u64 * page,
        page_size: page,
    })
}
