//! Spike P0 — état de la mémoire, **en lecture seule**.
//!
//! Binaire d'exploration jetable, hors du code de production (cahier des charges, OBL-005) :
//! il a le droit d'utiliser `unsafe` et de déclarer ses propres prototypes, ce que le
//! workspace interdit partout ailleurs.
//!
//! Il répond à une seule question, celle du spike : *quelles valeurs les API de chaque système
//! permettent-elles réellement d'obtenir, et sont-elles cohérentes ?* Il ne nettoie rien.
//!
//! Usage :
//!   cargo run --release                      # une mesure
//!   cargo run --release -- --repeat 5 --interval-ms 1000

use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "spike-memoire — lecture de l'état mémoire, sans aucune modification\n\
             \n  --repeat N        nombre de mesures (défaut 1)\
             \n  --interval-ms N   attente entre deux mesures (défaut 1000)\
             \n\nAucune purge n'est implémentée : ce spike observe, il ne modifie rien."
        );
        return ExitCode::SUCCESS;
    }

    let repeat = value_of(&args, "--repeat").unwrap_or(1).max(1);
    let interval = value_of(&args, "--interval-ms").unwrap_or(1000);

    println!(
        "{{\"tool\":\"spike-memoire\",\"os\":\"{}\",\"measurements\":[",
        std::env::consts::OS
    );
    for index in 0..repeat {
        let separator = if index + 1 == repeat { "" } else { "," };
        println!("{}{separator}", measure());
        if index + 1 < repeat {
            sleep(Duration::from_millis(interval));
        }
    }
    println!("]}}");
    ExitCode::SUCCESS
}

#[cfg(target_os = "windows")]
fn measure() -> String {
    windows::measure()
}

#[cfg(target_os = "linux")]
fn measure() -> String {
    linux::measure()
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn measure() -> String {
    // macOS n'est pas encore couvert : le spike le dit au lieu d'inventer une valeur.
    "{\"error\":\"système non couvert par ce spike\"}".to_owned()
}

fn value_of(args: &[String], name: &str) -> Option<u64> {
    let position = args.iter().position(|arg| arg == name)?;
    args.get(position + 1)?.parse().ok()
}
