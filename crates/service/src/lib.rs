//! Répartition des requêtes du service.
//!
//! Ce crate contient la **décision** : quelle capacité appeler pour quel message, quels
//! privilèges activer, comment traduire un échec en code stable. Il ne contient ni transport
//! (le canal nommé arrive avec OBL-018), ni écriture d'historique, ni boucle de vie : ces trois
//! choses dépendent de l'environnement, et les y mêler rendrait la décision intestable.
//!
//! La règle qui structure tout : **un privilège n'est activé que si une action réelle va avoir
//! lieu**. Une simulation ne demande rien, parce qu'elle ne touche à rien ; l'interface le sait
//! par [`Reply`] et peut le dire à l'utilisateur (FR-MEM-5).

use obliniscor_core::{CleanReport, ErrorCode, MemorySnapshot, MemoryZone, Privilege, ProcessInfo};
use obliniscor_platform::{
    EnabledPrivileges, MemoryCleaner, MemoryProbe, PlatformError, Privileges, ProcessControl,
};
use obliniscor_proto::Message;

/// Ce que le service répond à un message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// État courant de la mémoire.
    Status(MemorySnapshot),
    /// Processus visibles.
    Processes(Vec<ProcessInfo>),
    /// Résultat d'un nettoyage, réel ou simulé.
    Cleaned(CleanReport),
    /// L'opération a échoué, avec son code stable.
    Failed(ErrorCode),
}

/// Le service, réduit à ses dépendances utiles.
///
/// Les traits viennent de la frontière : le service ne connaît ni Windows, ni Linux, ni macOS,
/// et se teste donc entièrement avec les doublures.
pub struct Service<C, P, R> {
    cleaner: C,
    privileges: P,
    processes: R,
}

impl<C, P, R> Service<C, P, R>
where
    C: MemoryCleaner + MemoryProbe,
    P: Privileges,
    R: ProcessControl,
{
    /// Construit le service autour de ses trois capacités.
    pub fn new(cleaner: C, privileges: P, processes: R) -> Self {
        Self {
            cleaner,
            privileges,
            processes,
        }
    }

    /// Traite un message et rend la réponse correspondante.
    ///
    /// Ne renvoie jamais d'erreur : un échec est une **réponse** (`Reply::Failed`), pas une
    /// exception de transport. Un client doit pouvoir afficher la cause, pas seulement constater
    /// que la connexion s'est terminée.
    pub fn handle(&self, message: &Message) -> Reply {
        match message {
            Message::GetStatus => Reply::Status(self.cleaner.snapshot()),
            Message::ListProcesses => match self.processes.list() {
                Ok(processes) => Reply::Processes(processes),
                Err(error) => Reply::Failed(code_of(&error)),
            },
            Message::RunMemoryClean { zones, dry_run } => self.clean(zones, *dry_run),
            // La progression est un message de service vers client : la recevoir est une
            // erreur de protocole, pas une demande.
            Message::Progress { .. } => Reply::Failed(ErrorCode::Protocol),
        }
    }

    fn clean(&self, zones: &[String], dry_run: bool) -> Reply {
        // Un nom de zone inconnu est refusé avant toute action : nettoyer « ce qu'on a compris »
        // serait pire que refuser, parce que l'utilisateur croirait avoir nettoyé autre chose.
        let mut parsed = Vec::with_capacity(zones.len());
        for zone in zones {
            match MemoryZone::parse(zone) {
                Some(zone) => parsed.push(zone),
                None => return Reply::Failed(ErrorCode::Protocol),
            }
        }
        if parsed.is_empty() {
            return Reply::Failed(ErrorCode::Protocol);
        }

        // Une simulation ne demande aucun privilège : elle ne modifie rien, et exiger des
        // droits pour ne rien faire ferait échouer le mode le plus sûr.
        let _guard: Option<Box<dyn EnabledPrivileges>> = if dry_run {
            None
        } else {
            match self.privileges.acquire(&[Privilege::ProfileSingleProcess]) {
                Ok(guard) => Some(guard),
                Err(error) => return Reply::Failed(code_of(&error)),
            }
        };

        match self.cleaner.clean(&parsed, dry_run) {
            Ok(report) => Reply::Cleaned(report),
            Err(error) => Reply::Failed(code_of(&error)),
        }
    }
}

/// Traduit une erreur de la frontière en code stable.
///
/// Le code est le contrat (`FR-SVC-3`) : trois causes de la frontière, trois codes, et
/// `MissingPrivilege` rejoint `Denied` parce que, pour l'utilisateur, les deux veulent dire
/// « il faut des droits » — le message, lui, distingue les deux.
#[must_use]
pub fn code_of(error: &PlatformError) -> ErrorCode {
    match error {
        PlatformError::Unavailable(_) => ErrorCode::Unavailable,
        PlatformError::Denied(_) => ErrorCode::Denied,
        PlatformError::MissingPrivilege(_) => ErrorCode::Denied,
        PlatformError::Failed(_) => ErrorCode::Failed,
    }
}
