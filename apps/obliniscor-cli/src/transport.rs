//! Le transport du client : une demande, une réponse, sur le canal du service.

use obliniscor_core::MemorySnapshot;
use obliniscor_platform::PlatformError;

#[cfg(windows)]
pub use windows_impl::ask_status;

/// Nom du canal du service, sans le préfixe de pipe.
///
/// Windows seulement : sous Linux et macOS, il n'y a pas encore de transport, et une constante
/// inutilisée serait du code mort que clippy refuse (-D warnings).
#[cfg(windows)]
pub const PIPE_NAME: &str = "obliniscord";

#[cfg(windows)]
mod windows_impl {
    use super::{MemorySnapshot, PIPE_NAME, PlatformError};
    use obliniscor_proto::{self as proto, MAX_FRAME, Message};

    /// Demande l'état de la mémoire au service.
    ///
    /// Échoue proprement quand le service n'est pas lancé : le client ne doit pas se lancer
    /// lui-même, ni attendre indéfiniment un processus qui n'existe pas.
    pub fn ask_status() -> Result<MemorySnapshot, PlatformError> {
        let client = obliniscor_sys::windows::pipe::PipeClient::connect(PIPE_NAME, 4)?;
        let request = proto::encode(&Message::GetStatus)
            .map_err(|error| PlatformError::Failed(error.to_string()))?;
        client.write_frame(&request)?;
        let frame = client.read_frame(MAX_FRAME)?;
        let reply = proto::decode::<proto::Reply>(&frame)
            .map_err(|error| PlatformError::Failed(error.to_string()))?;
        match reply {
            proto::Reply::Status {
                total,
                available,
                free,
            } => Ok(MemorySnapshot {
                total,
                available,
                free,
            }),
            proto::Reply::Failed { code } => Err(PlatformError::Failed(format!(
                "le service a refusé la demande : {code}"
            ))),
            other => Err(PlatformError::Failed(format!(
                "réponse inattendue du service : {other:?}"
            ))),
        }
    }
}

/// Sous Linux et macOS, le transport n'existe pas encore : l'IPC y arrivera avec P6b.
#[cfg(not(windows))]
pub fn ask_status() -> Result<MemorySnapshot, PlatformError> {
    Err(PlatformError::Unavailable(
        obliniscor_core::MemoryZone::Standby,
    ))
}
