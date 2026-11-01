//! `cargo xtask ci` : exactement les mêmes contrôles en local et en intégration continue.
//!
//! La commande est volontairement unique : une seule chose à lancer, donc une seule chose à
//! faire passer avant de livrer. Chaque étape est celle que la CI exécute, dans le même ordre,
//! pour qu'un échec local soit un échec distant.

use anyhow::{Context, Result, bail};
use std::process::Command;

/// Les étapes, dans l'ordre. `--locked` partout : sans lui, une dépendance ajoutée sans
/// mettre à jour `Cargo.lock` passerait la CI et casserait un build reproductible.
const STEPS: &[&[&str]] = &[
    &["fmt", "--all", "--check"],
    &[
        "clippy",
        "--workspace",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ],
    // Tous les binaires de test, même après un échec : aucun échec n'en cache un autre.
    &["test", "--workspace", "--no-fail-fast", "--locked"],
    &["deny", "check"],
];

fn main() -> Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("ci") => ci(),
        _ => bail!("usage : cargo xtask ci"),
    }
}

fn ci() -> Result<()> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());

    let deny = Command::new(&cargo)
        .args(["deny", "--version"])
        .output()
        .is_ok_and(|output| output.status.success());
    if !deny {
        bail!("cargo-deny manquant : cargo install cargo-deny --locked");
    }

    for args in STEPS {
        let shown = format!("cargo {}", args.join(" "));
        eprintln!("==> {shown}");
        let status = Command::new(&cargo)
            .args(*args)
            .status()
            .with_context(|| format!("lancement de `{shown}` impossible"))?;
        if !status.success() {
            bail!("`{shown}` a échoué ({status})");
        }
    }

    eprintln!("==> ci OK");
    Ok(())
}
