//! Réglages v1 : lecture, écriture et récupération, contre le vrai système de fichiers.
//!
//! Les fichiers sont créés dans le dossier temporaire du système, dans un sous-dossier unique
//! par test, puis supprimés. Aucun test ne touche aux réglages réels de la machine.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_settings::{Recovery, SettingsStore};
use obliniscor_store::ObliNiscorSettings;
use std::path::PathBuf;

/// Sous-dossier temporaire unique, supprimé au premier échec comme au succès.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!("obliniscor-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("dossier temporaire");
        Self(path)
    }

    fn path(&self) -> PathBuf {
        self.0.clone()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn defaults_never_act_destroying_without_being_asked() {
    let settings = ObliNiscorSettings::default();
    assert!(settings.dry_run, "la simulation est le défaut");
    assert_eq!(settings.language, "en");
    assert_eq!(settings.history_retention_days, 90);
    assert!(settings.excluded_processes.is_empty());
}

#[test]
fn a_missing_file_yields_defaults_and_creates_nothing() {
    let dir = TempDir::new("absent");
    let store = SettingsStore::<ObliNiscorSettings>::new(dir.path());
    let loaded = store
        .load()
        .expect("absence de fichier n'est pas une erreur");

    assert!(loaded.recovery.is_none());
    assert!(!loaded.read_only);
    assert_eq!(loaded.settings, ObliNiscorSettings::default());
    // Lire ne doit pas écrire : le fichier apparaît à la première sauvegarde, pas avant.
    assert!(!store.path().exists());
}

#[test]
fn settings_survive_a_save_and_a_load() {
    let dir = TempDir::new("aller-retour");
    let store = SettingsStore::<ObliNiscorSettings>::new(dir.path());
    let settings = ObliNiscorSettings {
        language: "fr".to_owned(),
        dry_run: false,
        excluded_processes: vec!["explorer.exe".to_owned()],
        history_retention_days: 30,
        show_report: false,
    };
    let mut loaded = store.load().expect("chargement");
    loaded.settings = settings.clone();
    store.save(&loaded).expect("sauvegarde");

    let reloaded = store.load().expect("relecture");
    assert_eq!(reloaded.settings, settings);
    assert!(reloaded.recovery.is_none());
}

#[test]
fn a_corrupt_file_is_backed_up_and_defaults_are_used() {
    let dir = TempDir::new("corrompu");
    let store = SettingsStore::<ObliNiscorSettings>::new(dir.path());
    std::fs::write(store.path(), b"ceci n'est pas du TOML {{{").expect("écriture du fichier cassé");

    let loaded = store
        .load()
        .expect("un fichier cassé ne doit pas faire échouer le démarrage");
    assert_eq!(loaded.settings, ObliNiscorSettings::default());
    match loaded.recovery {
        Some(Recovery::Corrupt { backup }) => {
            assert!(backup.exists(), "la sauvegarde doit exister : {backup:?}");
        }
        other => panic!("récupération inattendue : {other:?}"),
    }
}

#[test]
fn a_file_from_a_newer_version_is_opened_read_only() {
    let dir = TempDir::new("futur");
    let store = SettingsStore::<ObliNiscorSettings>::new(dir.path());
    // La clé est `schema_version` : la bibliothèque la retire de la table avant de désérialiser,
    // donc écrire `version` produirait un fichier de version 1 avec un champ inconnu — et le
    // test passerait pour la mauvaise raison.
    std::fs::write(store.path(), b"schema_version = 2\nlanguage = \"fr\"\n").expect("écriture");

    let loaded = store.load().expect("un fichier futur reste lisible");
    assert!(
        loaded.read_only,
        "une version plus récente ne doit pas être écrasée"
    );
}

#[test]
fn an_unknown_language_falls_back_to_english() {
    let settings = ObliNiscorSettings {
        language: "de".to_owned(),
        ..ObliNiscorSettings::default()
    }
    .sanitized();
    assert_eq!(settings.language, "en");
}

#[test]
fn an_absurd_retention_is_corrected() {
    let settings = ObliNiscorSettings {
        history_retention_days: 100_000,
        ..ObliNiscorSettings::default()
    }
    .sanitized();
    assert_eq!(settings.history_retention_days, 90);
}

#[test]
fn sanitizing_keeps_the_fields_it_did_not_correct() {
    let settings = ObliNiscorSettings {
        language: "fr".to_owned(),
        dry_run: false,
        excluded_processes: vec!["svchost.exe".to_owned()],
        history_retention_days: 7,
        show_report: false,
    };
    assert_eq!(settings.clone().sanitized(), settings);
}
