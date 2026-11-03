//! Noyau d'obliNiscor : les modèles, les politiques et les garde-fous.
//!
//! **Aucune entrée/sortie, aucun accès système.** Ce crate décide *quoi* faire et *quand* ;
//! la couche `obliniscor-sys` décide *comment* le faire, système par système (ADR-004). Cette
//! séparation est ce qui rend le noyau testable sur les trois plateformes, y compris celles
//! où une fonction n'existe pas (ADR-009).

/// Une zone de mémoire que l'on sait mesurer ou nettoyer.
///
/// L'énumération est volontairement **ouverte** : la liste des zones diffère selon le système
/// et selon la version du noyau, et un système peut en connaître une que Windows ignore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryZone {
    /// Pages d'un processus actuellement résidentes.
    WorkingSet,
    /// Cache de fichiers du système.
    SystemCache,
    /// Liste des pages modifiées, à écrire avant réutilisation.
    ModifiedList,
    /// Liste des pages en veille, par niveau de priorité.
    Standby,
    /// Cache du registre (Windows 8.1 et suivants).
    RegistryCache,
}

/// Un privilège que le processus peut activer le temps d'une opération.
///
/// Les noms sont ceux de Windows et ils sont **stables** : ils apparaissent dans les journaux
/// et les messages d'erreur, donc ils ne suivent pas le nom du variant Rust. Sous Linux et
/// macOS, l'équivalent est un droit du système de fichiers ou une capacité, pas un privilège
/// nommé : ces variants n'y ont pas de sens, et le code les déclare indisponibles (ADR-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Privilege {
    /// `SeProfileSingleProcessPrivilege` — interroger les listes de pages du noyau.
    ProfileSingleProcess,
    /// `SeIncreaseQuotaPrivilege` — réduire l'ensemble de travail d'un autre processus.
    IncreaseQuota,
    /// `SeIncreaseWorkingSetPrivilege` — agrandir son propre ensemble de travail.
    IncreaseWorkingSet,
    /// `SeDebugPrivilege` — agir sur les processus d'autres utilisateurs.
    Debug,
}

impl Privilege {
    /// Nom Windows du privilège, tel qu'il apparaît dans les journaux.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProfileSingleProcess => "SeProfileSingleProcessPrivilege",
            Self::IncreaseQuota => "SeIncreaseQuotaPrivilege",
            Self::IncreaseWorkingSet => "SeIncreaseWorkingSetPrivilege",
            Self::Debug => "SeDebugPrivilege",
        }
    }

    /// Vrai quand un utilisateur normal détient déjà ce privilège.
    ///
    /// Seul `SeIncreaseWorkingSetPrivilege` est accordé par défaut : c'est ce qui rend
    /// l'optimisation de son propre processus possible sans élévation, et ce qui sépare les
    /// opérations « utilisateur » des opérations « administrateur » (FR-MEM-5).
    pub const fn held_by_default(self) -> bool {
        matches!(self, Self::IncreaseWorkingSet)
    }

    /// Tous les privilèges connus.
    pub const ALL: [Self; 4] = [
        Self::ProfileSingleProcess,
        Self::IncreaseQuota,
        Self::IncreaseWorkingSet,
        Self::Debug,
    ];
}

/// Ce qu'un système sait faire d'une zone, **pour le nettoyage**.
///
/// La mesure est un autre axe : macOS mesure parfaitement la mémoire
/// (`host_statistics64`) mais ne sait pas purger de zone. Ses zones sont donc
/// `Unavailable` ici, ce qui répond à la seule question qui compte pour l'interface :
/// peut-on proposer l'action ? (ADR-009)
///
/// Une capacité absente n'est **pas** une erreur : c'est une information à afficher telle
/// quelle. L'interface grise la zone et dit pourquoi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// Nettoyable, et mesurable.
    Full,
    /// Nettoyable seulement avec des droits particuliers, ou déconseillé.
    Partial,
    /// Pas nettoyable sur ce système — ce qui ne dit rien de la mesure.
    Unavailable,
}

impl Availability {
    /// Vrai quand une action est possible sans mentir à l'utilisateur.
    pub const fn can_clean(self) -> bool {
        matches!(self, Self::Full | Self::Partial)
    }

    /// Identifiant Fluent de la mention affichée pour cette disponibilité.
    ///
    /// Les clés existent dans `locales/en/obliniscor.ftl` **et** `locales/fr/obliniscor.ftl`,
    /// et `cargo xtask ci` vérifie les deux sens : aucune langue incomplète, et aucune clé
    /// produite par ce code qui manquerait dans les traductions (NFR-I18N-1).
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Full => "availability-full",
            Self::Partial => "availability-partial",
            Self::Unavailable => "availability-unavailable",
        }
    }

    /// Toutes les variantes, dans l'ordre d'affichage. Sert au contrôle de complétude de
    /// `cargo xtask ci`.
    pub const ALL: [Self; 3] = [Self::Full, Self::Partial, Self::Unavailable];
}

impl MemoryZone {
    /// Nom stable de la zone, utilisé dans l'IPC et par la CLI.
    ///
    /// Ce nom est un **contrat** : le service et les clients peuvent être de versions
    /// différentes, et une zone renommée casserait la compatibilité. Il ne suit donc pas le
    /// nom du variant Rust, qui peut changer sans préavis.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkingSet => "working-set",
            Self::SystemCache => "system-cache",
            Self::ModifiedList => "modified-list",
            Self::Standby => "standby",
            Self::RegistryCache => "registry-cache",
        }
    }

    /// Toutes les zones connues, dans l'ordre d'affichage.
    pub const ALL: [Self; 5] = [
        Self::WorkingSet,
        Self::SystemCache,
        Self::ModifiedList,
        Self::Standby,
        Self::RegistryCache,
    ];

    /// Relit un nom stable ; `None` pour un nom inconnu, jamais pour un nom mal formé.
    ///
    /// L'IPC reçoit des chaînes : une zone inconnue doit être **refusée** par l'appelant
    /// (message d'erreur explicite) et non ignorée en silence.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|zone| zone.as_str() == name.trim().to_ascii_lowercase())
    }
}

/// Code d'erreur stable, exposé par l'IPC et la CLI (cahier des charges §8.3).
///
/// Un client de version différente doit pouvoir comprendre l'échec sans lire un message
/// traduit : c'est le code qui est le contrat, le texte n'est qu'un confort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// La fonction n'existe pas sur ce système (ADR-009).
    Unavailable,
    /// Droits insuffisants pour l'opération demandée.
    Denied,
    /// L'opération a échoué ; la cause est dans le message.
    Failed,
    /// Le message reçu n'est pas un message du protocole.
    Protocol,
    /// Erreur interne du service, qui ne devrait pas arriver.
    Internal,
}

impl ErrorCode {
    /// Forme sérialisée, stable et indépendante de la langue.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "E_UNAVAILABLE",
            Self::Denied => "E_DENIED",
            Self::Failed => "E_FAILED",
            Self::Protocol => "E_PROTOCOL",
            Self::Internal => "E_INTERNAL",
        }
    }

    /// Toutes les variantes ; sert aux tests de stabilité.
    pub const ALL: [Self; 5] = [
        Self::Unavailable,
        Self::Denied,
        Self::Failed,
        Self::Protocol,
        Self::Internal,
    ];
}

/// État de la mémoire à un instant donné, tel que le système le rapporte.
///
/// Les trois valeurs sont en octets et ne sont **pas** recalculées ici : `available` vient du
/// système, qui sait mieux que nous ce qui est réutilisable (sous Linux, `MemAvailable`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MemorySnapshot {
    pub total: u64,
    pub available: u64,
    pub free: u64,
}

/// Résultat d'une opération de nettoyage, zone par zone.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CleanReport {
    /// Octets libérés, tels que mesurés avant et après.
    pub freed: u64,
    /// Zones effectivement traitées, dans l'ordre d'exécution.
    pub zones: Vec<MemoryZone>,
}

/// Un processus observé, réduit à ce que la liste affiche.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub working_set: u64,
}

/// Un volume monté et son espace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    pub mount: String,
    pub total: u64,
    pub free: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_partial_zone_can_still_be_cleaned() {
        assert!(Availability::Partial.can_clean());
        assert!(Availability::Full.can_clean());
    }

    #[test]
    fn an_unavailable_zone_is_never_offered() {
        // C'est la règle de l'ADR-009 : ce qui n'existe pas ne s'annonce pas comme faisable.
        assert!(!Availability::Unavailable.can_clean());
    }

    #[test]
    fn every_availability_has_its_own_message_key() {
        let keys: Vec<&str> = Availability::ALL.iter().map(|a| a.label_key()).collect();
        let mut unique = keys.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            keys.len(),
            "deux disponibilités partagent une clé"
        );
        assert!(keys.iter().all(|key| key.starts_with("availability-")));
    }

    #[test]
    fn a_zone_name_survives_a_round_trip() {
        // Le nom est un contrat d'IPC : s'il change, un client d'une autre version casse.
        for zone in MemoryZone::ALL {
            assert_eq!(MemoryZone::parse(zone.as_str()), Some(zone));
        }
    }

    #[test]
    fn an_unknown_zone_name_is_refused() {
        assert_eq!(MemoryZone::parse("liste-standby"), None);
        assert_eq!(MemoryZone::parse(""), None);
        // Une casse ou des espaces ne changent pas le nom : c'est du confort de lecture, pas
        // une tolérance sur le contrat.
        assert_eq!(MemoryZone::parse("  Standby  "), Some(MemoryZone::Standby));
    }

    #[test]
    fn zone_names_are_unique_and_kebab_case() {
        let names: Vec<&str> = MemoryZone::ALL.iter().map(|z| z.as_str()).collect();
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len(), "deux zones partagent un nom");
        assert!(
            names
                .iter()
                .all(|name| name.chars().all(|c| c.is_ascii_lowercase() || c == '-')),
            "{names:?}"
        );
    }

    #[test]
    fn error_codes_are_stable_and_unique() {
        // Le code est ce qu'un client d'une autre version lit ; il ne doit jamais bouger.
        let expected = [
            "E_UNAVAILABLE",
            "E_DENIED",
            "E_FAILED",
            "E_PROTOCOL",
            "E_INTERNAL",
        ];
        let codes: Vec<&str> = ErrorCode::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(codes, expected);
        let mut unique = codes.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), codes.len(), "deux erreurs partagent un code");
    }

    #[test]
    fn only_one_privilege_is_held_by_default() {
        // C'est la frontière entre ce qu'un utilisateur peut faire sans élévation et le reste :
        // la déplacer sans le savoir changerait ce que l'interface promet.
        let default: Vec<Privilege> = Privilege::ALL
            .into_iter()
            .filter(|privilege| privilege.held_by_default())
            .collect();
        assert_eq!(default, [Privilege::IncreaseWorkingSet]);
    }

    #[test]
    fn privilege_names_are_windows_names_and_unique() {
        let names: Vec<&str> = Privilege::ALL.iter().map(|p| p.as_str()).collect();
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            names.len(),
            "deux privilèges partagent un nom"
        );
        assert!(names.iter().all(|name| name.starts_with("Se")));
        assert!(names.iter().all(|name| name.ends_with("Privilege")));
    }
}
