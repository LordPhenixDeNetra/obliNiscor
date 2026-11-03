//! Réglages d'obliNiscor, schéma **v1**.
//!
//! Le fichier est du TOML, versionné par un champ `version`, et lu par
//! [`lpdnsoft_settings::SettingsStore`] : ce crate ne réécrit pas la mécanique de lecture, de
//! sauvegarde et de récupération, il ne déclare que **ses** champs et **ses** migrations.
//!
//! Règle de conception : un champ ajouté ici doit avoir une valeur par défaut qui préserve le
//! comportement précédent. Un réglage dont l'absence change ce que fait l'outil n'est pas un
//! réglage, c'est une décision d'architecture — elle appartient au noyau, pas au fichier.

use lpdnsoft_settings::{Settings, SettingsError};
use serde::{Deserialize, Serialize};

/// Nom du fichier, dans le dossier de configuration de l'application.
pub const FILE_NAME: &str = "settings.toml";

/// Réglages de l'application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObliNiscorSettings {
    /// Langue de l'interface (`en` ou `fr`). Le repli est `en`.
    pub language: String,
    /// Simuler les opérations destructrices au lieu de les exécuter.
    ///
    /// Vrai par défaut : un outil qui nettoie la mémoire ou le disque ne doit rien faire de
    /// définitif avant que l'utilisateur l'ait demandé explicitement.
    pub dry_run: bool,
    /// Processus jamais touchés, même quand l'utilisateur demande « tous ».
    pub excluded_processes: Vec<String>,
    /// Durée de conservation de l'historique, en jours ; zéro désactive l'historique.
    pub history_retention_days: u32,
    /// Afficher le résultat chiffré de chaque opération.
    pub show_report: bool,
}

impl Default for ObliNiscorSettings {
    fn default() -> Self {
        Self {
            language: "en".to_owned(),
            dry_run: true,
            excluded_processes: Vec::new(),
            history_retention_days: 90,
            show_report: true,
        }
    }
}

impl ObliNiscorSettings {
    /// Vérifie les valeurs après lecture, et répare celles qui n'ont pas de sens.
    ///
    /// Un fichier de réglages est édité à la main : une langue inconnue ou une durée absurde
    /// viennent d'une faute de frappe, pas d'une intention. On corrige et on le signale plutôt
    /// que de refuser le fichier entier et de perdre les autres réglages.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        if !matches!(self.language.as_str(), "en" | "fr") {
            self.language = Self::default().language;
        }
        // Cent ans de rétention n'est pas une faute de frappe, c'est une absence de limite.
        if self.history_retention_days > 3650 {
            self.history_retention_days = Self::default().history_retention_days;
        }
        self
    }
}

impl Settings for ObliNiscorSettings {
    const FILE_NAME: &'static str = FILE_NAME;
    const SCHEMA_VERSION: u32 = 1;

    /// Migration d'un schéma antérieur vers le suivant.
    ///
    /// Il n'existe pas de version 0 publiée : cet outil n'a jamais livré d'ancien format. La
    /// première migration réelle sera écrite **avec** la v2, quand il y aura quelque chose à
    /// migrer — pas avant, ce serait du code jamais exécuté.
    fn migrate_step(from: u32, table: toml::Table) -> Result<toml::Table, SettingsError> {
        let _ = table;
        Err(SettingsError::Unsupported { from })
    }
}
