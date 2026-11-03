//! Boucle de service sur canal nommé (OBL-018).
//!
//! Le canal est un transport **fidèle et ordonné** : une trame en entrée, une trame en sortie,
//! puis on raccroche. Le service reste la décision ([`crate::Service::handle`]) ; ce module ne
//! fait que lire, décoder, appeler et écrire.
//!
//! Une trame invalide fait répondre `E_PROTOCOL` plutôt que de fermer brutalement : un client
//! qui parle mal doit apprendre *pourquoi*, et le service doit rester debout pour les suivants.

use crate::Service;
use obliniscor_platform::{MemoryCleaner, MemoryProbe, PlatformError, Privileges, ProcessControl};
use obliniscor_proto as proto;
use obliniscor_proto::{MAX_FRAME, Message};

/// Sert un client déjà connecté : une demande, une réponse, puis déconnexion.
///
/// Rend `Ok(true)` quand un message a été traité, `Ok(false)` quand la connexion s'est fermée
/// proprement ; une erreur signifie que le canal lui-même a échoué.
pub fn serve_one<C, P, R>(
    server: &obliniscor_sys::windows::pipe::PipeServer,
    service: &Service<C, P, R>,
) -> Result<bool, PlatformError>
where
    C: MemoryCleaner + MemoryProbe,
    P: Privileges,
    R: ProcessControl,
{
    let frame = server.read_frame(MAX_FRAME)?;
    let reply = match proto::decode::<Message>(&frame) {
        Ok(message) => crate::answer::answer(service, &message),
        Err(_) => obliniscor_proto::Reply::Failed {
            code: obliniscor_core::ErrorCode::Protocol.as_str().to_owned(),
        },
    };
    // Le canal transporte des trames opaques : leur cadrage appartient à `obliniscor-proto`,
    // et le canal ne fait que les déplacer.
    let reply_frame =
        proto::encode(&reply).map_err(|error| PlatformError::Failed(error.to_string()))?;
    server.write_frame(&reply_frame)?;
    Ok(true)
}
