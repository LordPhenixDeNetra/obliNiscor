//! Backend Linux : `/proc/meminfo` et `/proc/vmstat`.
//!
//! Aucun `unsafe`, aucune bibliothèque : lire deux fichiers texte suffit, et c'est ce qui
//! rend le backend portable et testable.

/// Lit la valeur d'une clé de `/proc/meminfo`, convertie de kibioctets en octets.
///
/// La clé est suivie de deux espaces et de l'unité (`kB`), et non d'un `=` : c'est un format
/// à part, pas du JSON ni du TOML. Un `strip_prefix` naïf ne suffit pas.
fn meminfo_kib(text: &str, key: &str) -> Option<u64> {
    let line = text.lines().find(|line| line.starts_with(key))?;
    let value = line[key.len()..].split_whitespace().next()?;
    value.parse::<u64>().ok()
}

/// Lit un compteur de `/proc/vmstat` (valeur brute, sans unité).
fn vmstat(text: &str, key: &str) -> Option<u64> {
    let line = text.lines().find(|line| line.starts_with(key))?;
    let value = line.split_whitespace().nth(1)?;
    value.parse::<u64>().ok()
}

pub fn measure() -> String {
    let meminfo = match std::fs::read_to_string("/proc/meminfo") {
        Ok(text) => text,
        Err(error) => return format!("{{\"error\":\"{error}\"}}"),
    };
    let vmstat_text = std::fs::read_to_string("/proc/vmstat").unwrap_or_default();

    // `MemAvailable` est l'estimation du noyau : c'est la seule valeur à présenter comme
    // « disponible », et elle n'existe que depuis Linux 3.14.
    let kib = |key| meminfo_kib(&meminfo, key).map(|value| value * 1024);
    let total = kib("MemTotal:").unwrap_or(0);
    let available = kib("MemAvailable:").unwrap_or(0);
    let free = kib("MemFree:").unwrap_or(0);
    let buffers = kib("Buffers:").unwrap_or(0);
    let cached = kib("Cached:").unwrap_or(0);
    let reclaimable = kib("SReclaimable:").unwrap_or(0);
    let swap_total = kib("SwapTotal:").unwrap_or(0);
    let swap_free = kib("SwapFree:").unwrap_or(0);

    // Défauts de page majeurs : l'indicateur de coût de FR-MEM-6.
    let major_faults = vmstat(&vmstat_text, "pgmajfault").unwrap_or(0);
    let swap_in = vmstat(&vmstat_text, "pswpin").unwrap_or(0);
    let swap_out = vmstat(&vmstat_text, "pswpout").unwrap_or(0);

    format!(
        "{{\"kernel\":\"{}\",\"total\":{total},\"available\":{available},\"free\":{free},\
         \"buffers\":{buffers},\"cached\":{cached},\"sreclaimable\":{reclaimable},\
         \"swap_total\":{swap_total},\"swap_free\":{swap_free},\
         \"major_faults\":{major_faults},\"swap_in\":{swap_in},\"swap_out\":{swap_out}}}",
        kernel_release()
    )
}

fn kernel_release() -> String {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|text| text.trim().to_owned())
        .unwrap_or_else(|_| "inconnu".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_meminfo_line() {
        // Forme réelle de `/proc/meminfo` : la clé, deux espaces, la valeur, l'unité.
        let text = "MemTotal:       16318732 kB\nMemAvailable:    9123456 kB\n";
        assert_eq!(meminfo_kib(text, "MemTotal:"), Some(16318732));
        assert_eq!(meminfo_kib(text, "MemAvailable:"), Some(9123456));
    }

    #[test]
    fn refuses_a_key_that_is_not_there() {
        let text = "MemTotal:       16318732 kB\n";
        assert_eq!(meminfo_kib(text, "SReclaimable:"), None);
    }

    #[test]
    fn reads_a_vmstat_counter() {
        let text = "pgpgin 1234\npgmajfault 56\npswpin 7\n";
        assert_eq!(vmstat(text, "pgmajfault"), Some(56));
        assert_eq!(vmstat(text, "pswpin"), Some(7));
        assert_eq!(vmstat(text, "absent"), None);
    }
}
