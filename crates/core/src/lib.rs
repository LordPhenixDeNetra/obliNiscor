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
}
