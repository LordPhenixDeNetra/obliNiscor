//! Doublures de test des traits de la frontière système.
//!
//! Chaque doublure est **configurable** et ne touche à rien : elle renvoie exactement ce
//! qu'on lui a donné, jusqu'au scénario d'échec demandé. Trois scénarios sont couverts :
//! une zone déclarée indisponible, un accès refusé, et une réponse normale. Ils suffisent à
//! tester le noyau sans Windows et sans privilèges.
//!
//! Les doublures vivent dans le crate de production parce que le noyau et la couche `sys`
//! les partagent ; elles ne sont pas destinées à un binaire livré.

use obliniscor_core::MemoryZone;

use crate::{
    CleanReport, DiskInventory, MemoryCleaner, MemoryProbe, MemorySnapshot, PlatformError,
    ProcessControl, ProcessInfo, Volume,
};

/// Message porté par [`PlatformError::Denied`] quand une doublure simule un refus.
///
/// Exposé pour que les tests puissent affirmer l'erreur *exacte*, pas seulement sa variante.
pub const DENIED_MESSAGE: &str = "accès refusé (doublure de test)";

/// Doublure de [`MemoryProbe`] et de [`MemoryCleaner`].
///
/// L'instantané est fixe ; le nettoyage est réglable par [`FakeMemory::with_freed`], par la
/// déclaration d'une zone indisponible ([`FakeMemory::with_unavailable_zone`], ADR-009) et par
/// le mode « accès refusé » ([`FakeMemory::with_denied`]).
#[derive(Debug, Clone)]
pub struct FakeMemory {
    snapshot: MemorySnapshot,
    unavailable: Vec<MemoryZone>,
    denied: bool,
    freed: u64,
}

impl FakeMemory {
    /// Une doublure qui renvoie `snapshot` et ne libère aucun octet.
    pub fn new(snapshot: MemorySnapshot) -> Self {
        Self {
            snapshot,
            unavailable: Vec::new(),
            denied: false,
            freed: 0,
        }
    }

    /// Déclare `zone` absente du système simulé.
    ///
    /// [`MemoryCleaner::clean`] renvoie alors [`PlatformError::Unavailable`] dès que la zone
    /// figure dans la demande, en simulation comme en nettoyage réel. Les autres zones
    /// restent utilisables : c'est la capacité qui manque, pas le système entier.
    pub fn with_unavailable_zone(mut self, zone: MemoryZone) -> Self {
        self.unavailable.push(zone);
        self
    }

    /// Simule un refus de droits sur toute demande de nettoyage.
    pub fn with_denied(mut self) -> Self {
        self.denied = true;
        self
    }

    /// Fixe le nombre d'octets qu'un nettoyage réel (hors simulation) prétend libérer.
    pub fn with_freed(mut self, freed: u64) -> Self {
        self.freed = freed;
        self
    }
}

impl MemoryProbe for FakeMemory {
    fn snapshot(&self) -> MemorySnapshot {
        self.snapshot
    }
}

impl MemoryCleaner for FakeMemory {
    fn clean(&self, zones: &[MemoryZone], dry_run: bool) -> Result<CleanReport, PlatformError> {
        // Le refus de droits passe avant tout : c'est ce que ferait le système.
        if self.denied {
            return Err(PlatformError::Denied(DENIED_MESSAGE.to_owned()));
        }
        // Une capacité absente se déclare, même en simulation (ADR-009).
        if let Some(zone) = zones
            .iter()
            .find(|zone| self.unavailable.contains(zone))
            .copied()
        {
            return Err(PlatformError::Unavailable(zone));
        }
        Ok(CleanReport {
            freed: if dry_run { 0 } else { self.freed },
            zones: zones.to_vec(),
        })
    }
}

/// Doublure de [`ProcessControl`].
#[derive(Debug, Clone, Default)]
pub struct FakeProcesses {
    processes: Vec<ProcessInfo>,
    denied: bool,
}

impl FakeProcesses {
    /// Une doublure qui ne voit aucun processus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fixe la liste renvoyée, dans l'ordre donné.
    pub fn with_processes(mut self, processes: impl IntoIterator<Item = ProcessInfo>) -> Self {
        self.processes = processes.into_iter().collect();
        self
    }

    /// Simule un refus de droits sur l'énumération.
    pub fn with_denied(mut self) -> Self {
        self.denied = true;
        self
    }
}

impl ProcessControl for FakeProcesses {
    fn list(&self) -> Result<Vec<ProcessInfo>, PlatformError> {
        if self.denied {
            return Err(PlatformError::Denied(DENIED_MESSAGE.to_owned()));
        }
        Ok(self.processes.clone())
    }
}

/// Doublure de [`DiskInventory`].
#[derive(Debug, Clone, Default)]
pub struct FakeVolumes {
    volumes: Vec<Volume>,
    denied: bool,
}

impl FakeVolumes {
    /// Une doublure qui ne voit aucun volume.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fixe la liste renvoyée, dans l'ordre donné.
    pub fn with_volumes(mut self, volumes: impl IntoIterator<Item = Volume>) -> Self {
        self.volumes = volumes.into_iter().collect();
        self
    }

    /// Simule un refus de droits sur l'inventaire.
    pub fn with_denied(mut self) -> Self {
        self.denied = true;
        self
    }
}

impl DiskInventory for FakeVolumes {
    fn volumes(&self) -> Result<Vec<Volume>, PlatformError> {
        if self.denied {
            return Err(PlatformError::Denied(DENIED_MESSAGE.to_owned()));
        }
        Ok(self.volumes.clone())
    }
}
