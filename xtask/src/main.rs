//! `cargo xtask ci` : exactement les mêmes contrôles en local et en intégration continue.
//!
//! La commande est volontairement unique : une seule chose à lancer, donc une seule chose à
//! faire passer avant de livrer. Chaque étape est celle que la CI exécute, dans le même ordre,
//! pour qu'un échec local soit un échec distant.

mod tracking;

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

/// Dossier des traductions Fluent, à la racine du dépôt.
const LOCALES_DIR: &str = "locales";

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

    check_i18n()?;
    check_tracking()?;

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

/// Vérifie que les traductions sont complètes, et que les clés produites par le code existent.
///
/// Deux contrôles distincts, parce qu'ils attrapent deux erreurs différentes :
/// `check_complete` voit une clé ajoutée dans une langue et oubliée dans l'autre ; la boucle
/// ci-dessous voit une clé renommée dans le code et pas dans les `.ftl` — ce que la macro `t!`
/// attrapera plus tard à la compilation, mais qui serait invisible tant qu'aucune interface
/// n'existe.
fn check_i18n() -> Result<()> {
    eprintln!("==> i18n : {LOCALES_DIR}/ complet");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let catalog = lpdnsoft_i18n::load_catalog(&root.join(LOCALES_DIR))
        .context("lecture des fichiers de traduction")?;
    lpdnsoft_i18n::check_complete(&catalog).map_err(|e| anyhow::anyhow!("{e}"))?;

    for availability in obliniscor_core::Availability::ALL {
        let key = availability.label_key();
        for (locale, ids) in &catalog.locales {
            if !ids.contains(key) {
                bail!("clé `{key}` absente de la langue `{locale}`");
            }
        }
    }
    Ok(())
}

/// Vérifie la cohérence du document de suivi.
///
/// Le suivi est un fichier texte ; un fichier texte se désynchronise. Ces contrôles attrapent
/// les erreurs qu'un relecteur chercherait à l'œil : un total qui ne tombe pas juste, une tâche
/// bloquée sans ligne au registre, un journal daté à l'envers.
fn check_tracking() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let path = find_tracking_document(&root)?;
    eprintln!("==> suivi : {}", path.display());

    let content =
        std::fs::read_to_string(&path).with_context(|| format!("lecture de {}", path.display()))?;
    let tracking =
        tracking::parse(&content).with_context(|| format!("analyse de {}", path.display()))?;
    tracking::verify(&tracking).context("le document de suivi est incohérent")?;
    Ok(())
}

/// Trouve le document de suivi à la racine du dépôt.
///
/// Le nom contient une ponctuation que les systèmes de fichiers traitent différemment : on
/// cherche donc un `.md` dont le nom porte « Suivi des tâches », plutôt que de figer une
/// orthographe.
fn find_tracking_document(root: &Path) -> Result<std::path::PathBuf> {
    let entries =
        std::fs::read_dir(root).with_context(|| format!("lecture de {}", root.display()))?;
    for entry in entries {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if name.ends_with(".md") && name.contains("Suivi des t") {
            return Ok(path);
        }
    }
    bail!("document de suivi introuvable à la racine du dépôt");
}
