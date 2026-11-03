//! Traduction entre le service et le protocole.
//!
//! Le service raisonne avec les types du noyau ; le fil transporte ceux de `obliniscor-proto`.
//! Cette conversion est le **seul** endroit où les deux se rencontrent : la déplacer ailleurs
//! ferait fuiter le contrat de fil dans le noyau, ou les types du noyau dans le protocole.
//!
//! Les noms de zones sortent par leur forme d'IPC (`working-set`, `standby`), jamais par le nom
//! d'un variant Rust : c'est le contrat stable (OBL-011).

use crate::{Reply, Service};
use obliniscor_platform::{MemoryCleaner, MemoryProbe, Privileges, ProcessControl};
use obliniscor_proto::{Message, ProcessSummary};

/// Exécute un message et rend la réponse prête à être encadrée et envoyée.
pub fn answer<C, P, R>(service: &Service<C, P, R>, message: &Message) -> obliniscor_proto::Reply
where
    C: MemoryCleaner + MemoryProbe,
    P: Privileges,
    R: ProcessControl,
{
    match service.handle(message) {
        Reply::Status(snapshot) => obliniscor_proto::Reply::Status {
            total: snapshot.total,
            available: snapshot.available,
            free: snapshot.free,
        },
        Reply::Processes(processes) => obliniscor_proto::Reply::Processes {
            processes: processes
                .into_iter()
                .map(|process| ProcessSummary {
                    pid: process.pid,
                    name: process.name,
                    working_set: process.working_set,
                })
                .collect(),
        },
        Reply::Cleaned(report) => obliniscor_proto::Reply::Cleaned {
            freed: report.freed,
            zones: report
                .zones
                .into_iter()
                .map(obliniscor_core::MemoryZone::as_str)
                .map(str::to_owned)
                .collect(),
        },
        Reply::Failed(code) => obliniscor_proto::Reply::Failed {
            code: code.as_str().to_owned(),
        },
    }
}
