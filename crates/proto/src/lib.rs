//! Protocole IPC d'obliNiscor : messages typés, cadrage par longueur et version.
//!
//! La GUI, la CLI et le service sont trois processus distincts : ils ne partagent qu'un canal
//! (canal nommé sous Windows, socket Unix ailleurs). Tout ce qui arrive par ce canal vient
//! d'un autre processus et doit donc être traité comme hostile (SEC-3). Ce crate est leur
//! contrat commun : les messages, leur forme sur le fil et les bornes qu'un pair impose à
//! l'autre (cahier des charges §8.4).
//!
//! Chaque trame commence par sa longueur en `u32` little-endian, puis par la charge utile en
//! JSON. Le préfixe permet au lecteur de savoir combien d'octets attendre avant d'analyser, et
//! surtout de refuser une annonce démesurée **avant** d'allouer : sans cette borne, un client
//! malveillant ferait réserver un gigaoctet au service pour rien.
//!
//! Rien de ce qui vient de l'extérieur ne provoque de panic : un service qui panique se coupe,
//! et c'est exactement ce qu'un attaquant cherche.

use serde::{Deserialize, Serialize};

/// Numéro du protocole, indépendant de la version de l'application.
///
/// Il ne change que si la forme d'une trame change. Une version mineure d'obliNiscor peut
/// ajouter des messages ou des champs sans l'incrémenter : c'est la compatibilité promise par
/// le cahier des charges (§8.4).
pub const PROTOCOL_VERSION: u16 = 1;

/// Taille maximale de la charge utile d'une trame, préfixe exclu : 1 Mio.
///
/// Tous les messages prévus tiennent très en dessous (une liste de zones, une progression).
/// La marge sert à un futur message un peu plus gros, pas à justifier un message énorme : la
/// limite existe pour borner la mémoire qu'un pair peut réclamer au service (FR-SVC-2).
pub const MAX_FRAME: usize = 1_048_576;

/// Taille du préfixe de longueur, en octets.
pub const HEADER_LEN: usize = 4;

// Les champs de variante sont lus sans `deny_unknown_fields` : un pair plus récent peut
// ajouter un champ, et le refuser casserait la compatibilité mineure promise. `#[serde(default)]`
// couvre le cas inverse — un champ retiré par un pair plus ancien reprend une valeur neutre au
// lieu de faire échouer la lecture.
/// Un message échangé entre un client (GUI ou CLI) et le service.
///
/// Le nom des variantes et des champs est le format du fil : le renommer rompt la compatibilité
/// avec un pair d'une autre version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
    /// Demande l'état courant du service (zones disponibles, tâche en cours).
    GetStatus,

    /// Demande la liste des processus observés par le service.
    ListProcesses,

    /// Demande un nettoyage mémoire, ou seulement sa simulation quand `dry_run` est vrai.
    RunMemoryClean {
        /// Zones visées, par identifiant stable du catalogue.
        #[serde(default)]
        zones: Vec<String>,
        /// Vrai pour un aperçu : le service calcule le résultat sans rien modifier.
        #[serde(default)]
        dry_run: bool,
    },

    /// Progression d'une opération longue, émise par le service pendant qu'elle s'exécute.
    Progress {
        /// Unités de travail déjà effectuées.
        #[serde(default)]
        done: u64,
        /// Total prévu, ou `None` quand il n'est pas connu d'avance.
        #[serde(default)]
        total: Option<u64>,
    },
}

/// Pourquoi une trame a été refusée.
///
/// Une variante par cause plutôt qu'un message texte : le client choisit sa réaction
/// (réessayer, signaler une incompatibilité de version, journaliser) sans analyser une chaîne.
#[derive(Debug, thiserror::Error)]
pub enum ProtoError {
    /// La trame s'arrête avant même de pouvoir annoncer sa taille.
    #[error("en-tête tronqué : {received} octet(s) reçus, au moins {HEADER_LEN} attendus")]
    TruncatedHeader { received: usize },

    /// La charge, écrite ou annoncée, dépasse `MAX_FRAME` : refus avant toute lecture.
    #[error("charge de {declared} octet(s), au-delà de la limite de {max} octet(s)")]
    FrameTooLarge { declared: usize, max: usize },

    /// La trame s'arrête avant la fin des octets annoncés.
    #[error("trame tronquée : {declared} octet(s) annoncés, {received} reçu(s)")]
    TruncatedPayload { declared: usize, received: usize },

    /// La trame porte plus d'octets que la charge annoncée : le flux est désaligné.
    #[error("octets excédentaires : {extra} octet(s) au-delà de la charge annoncée")]
    TrailingBytes { extra: usize },

    /// Le JSON ne représente pas un message : écriture impossible ou lecture refusée.
    #[error("message JSON invalide : {source}")]
    InvalidJson {
        /// Cause exacte fournie par serde_json, à l'écriture comme à la lecture.
        #[source]
        source: serde_json::Error,
    },
}

/// Encode un message en une trame complète : longueur en `u32` little-endian, puis JSON.
///
/// La longueur est celle de la charge seule, préfixe exclu ; [`decode`] s'en sert pour refuser
/// un flux qui annoncerait plus que ce qu'il porte.
///
/// L'échec est rendu à l'appelant, jamais masqué : une fonction qui renverrait une trame vide ou
/// tronquée en cas de problème ferait envoyer au service une trame invalide, constatée seulement
/// plus tard chez le lecteur, loin de la cause. L'appelant peut donc refuser tout de suite une
/// charge au-delà de [`MAX_FRAME`] ([`ProtoError::FrameTooLarge`]) ou une sérialisation
/// impossible ([`ProtoError::InvalidJson`]).
///
/// # Erreurs
///
/// [`ProtoError::FrameTooLarge`] si la charge sérialisée dépasse [`MAX_FRAME`] ; la trame n'est
/// alors jamais construite. [`ProtoError::InvalidJson`] si l'écriture JSON échoue, ce que
/// [`Message`] ne permet pas aujourd'hui, mais qu'il faut représenter sans panic.
pub fn encode(message: &Message) -> Result<Vec<u8>, ProtoError> {
    let payload =
        serde_json::to_vec(message).map_err(|source| ProtoError::InvalidJson { source })?;
    if payload.len() > MAX_FRAME {
        return Err(ProtoError::FrameTooLarge {
            declared: payload.len(),
            max: MAX_FRAME,
        });
    }

    let mut frame = Vec::with_capacity(HEADER_LEN + payload.len());
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Décode une trame complète, préfixe de longueur compris.
///
/// La tranche doit contenir exactement une trame. Un lecteur qui puise dans un tampon lit
/// d'abord les [`HEADER_LEN`] premiers octets pour connaître la longueur, puis découpe
/// exactement cette quantité : les octets en trop sont refusés, parce qu'ils signalent un flux
/// désaligné dont la trame suivante serait lue de travers.
///
/// # Erreurs
///
/// [`ProtoError::FrameTooLarge`] est renvoyé dès la lecture de l'en-tête, avant toute analyse
/// ou allocation proportionnelle à l'annonce ; [`ProtoError::TruncatedHeader`],
/// [`ProtoError::TruncatedPayload`] et [`ProtoError::TrailingBytes`] refusent une trame dont
/// les octets ne correspondent pas à son annonce ; [`ProtoError::InvalidJson`] refuse une
/// charge qui n'est pas un message du protocole.
pub fn decode(frame: &[u8]) -> Result<Message, ProtoError> {
    let (header, payload) =
        frame
            .split_at_checked(HEADER_LEN)
            .ok_or(ProtoError::TruncatedHeader {
                received: frame.len(),
            })?;
    let length = header
        .first_chunk::<HEADER_LEN>()
        .ok_or(ProtoError::TruncatedHeader {
            received: frame.len(),
        })?;
    let declared = u32::from_le_bytes(*length) as usize;

    // Le contrôle de taille précède toute lecture de la charge : une annonce démesurée ne se
    // traduit jamais par une allocation proportionnelle (SEC-3, FR-SVC-2).
    if declared > MAX_FRAME {
        return Err(ProtoError::FrameTooLarge {
            declared,
            max: MAX_FRAME,
        });
    }

    if payload.len() < declared {
        return Err(ProtoError::TruncatedPayload {
            declared,
            received: payload.len(),
        });
    }
    if payload.len() > declared {
        return Err(ProtoError::TrailingBytes {
            extra: payload.len() - declared,
        });
    }

    serde_json::from_slice(payload).map_err(|source| ProtoError::InvalidJson { source })
}
