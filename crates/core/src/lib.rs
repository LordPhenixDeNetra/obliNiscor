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

/// Ce qu'un système sait faire d'une zone.
///
/// Une capacité absente n'est **pas** une erreur : c'est une information à afficher telle
/// quelle (ADR-009). L'interface grise la zone et dit pourquoi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// Mesurable et nettoyable.
    Full,
    /// Mesurable, nettoyable seulement avec des droits particuliers, ou déconseillé.
    Partial,
    /// Ni mesurable ni nettoyable sur ce système.
    Unavailable,
}

impl Availability {
    /// Vrai quand une action est possible sans mentir à l'utilisateur.
    pub const fn can_clean(self) -> bool {
        matches!(self, Self::Full | Self::Partial)
    }
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
}
