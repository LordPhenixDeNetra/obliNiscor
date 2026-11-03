//! Les trois commandes du squelette, et les codes de sortie qui vont avec.
//!
//! Un code de sortie est un **contrat de script** : un utilisateur qui écrit
//! `obliniscor status --json || echo échec` doit pouvoir distinguer « le service n'est pas
//! lancé » de « je n'ai pas le droit ». Les codes sont donc nommés, documentés dans `--help`,
//! et traduits depuis les codes stables du noyau.

use crate::cli::{Command, Format};
use obliniscor_core::ErrorCode;
use std::path::PathBuf;

/// Code de sortie d'un succès.
pub const EXIT_OK: i32 = 0;
/// Échec interne, non classable.
pub const EXIT_FAILURE: i32 = 1;
/// Usage invalide : l'utilisateur s'est trompé dans les arguments.
pub const EXIT_USAGE: i32 = 2;
/// Le service ne répond pas : problème d'installation ou de démarrage.
pub const EXIT_UNREACHABLE: i32 = 3;
/// Le système a refusé faute de droits.
pub const EXIT_DENIED: i32 = 4;
/// La fonction n'existe pas sur ce système (ADR-009).
pub const EXIT_UNAVAILABLE: i32 = 5;

/// Traduit un code d'erreur du noyau en code de sortie.
///
/// Deux codes du noyau partagent un code de sortie : `Denied` et `MissingPrivilege` veulent
/// tous deux dire « il faut des droits » à l'échelle d'un script, même si le message les
/// distingue.
#[must_use]
pub const fn exit_code_of(code: ErrorCode) -> i32 {
    match code {
        ErrorCode::Unavailable => EXIT_UNAVAILABLE,
        ErrorCode::Denied => EXIT_DENIED,
        ErrorCode::Protocol | ErrorCode::Failed | ErrorCode::Internal => EXIT_FAILURE,
    }
}

/// Ce qu'une commande produit : un texte à afficher et un code de sortie.
pub struct Outcome {
    pub text: String,
    pub exit: i32,
}

impl Outcome {
    pub fn success(text: String) -> Self {
        Self {
            text,
            exit: EXIT_OK,
        }
    }

    pub fn failure(text: String, exit: i32) -> Self {
        Self { text, exit }
    }
}

/// Exécute une commande.
pub fn run(command: &Command, format: Format) -> Outcome {
    match command {
        Command::Help => Outcome::success(crate::cli::HELP.to_owned()),
        Command::Version => Outcome::success(version_text(format)),
        Command::Config => config(format),
        Command::Status => status(format),
    }
}

fn version_text(format: Format) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let protocol = obliniscor_proto::PROTOCOL_VERSION;
    match format {
        Format::Json => format!("{{\"version\":\"{version}\",\"protocol\":{protocol}}}"),
        Format::Human => format!("obliniscor {version} (protocole {protocol})"),
    }
}

/// Montre les réglages, sans les modifier.
///
/// Cette commande réussit même quand le service est arrêté : elle ne lit qu'un fichier, et son
/// utilité est justement de diagnostiquer une installation qui ne répond pas.
fn config(format: Format) -> Outcome {
    let Some(dir) = config_dir() else {
        let code = ErrorCode::Failed;
        return Outcome::failure(
            render_error(format, code, "dossier de configuration introuvable"),
            exit_code_of(code),
        );
    };
    let store =
        lpdnsoft_settings::SettingsStore::<obliniscor_store::ObliNiscorSettings>::new(dir.clone());
    let loaded = match store.load() {
        Ok(loaded) => loaded,
        Err(error) => {
            return Outcome::failure(
                render_error(
                    format,
                    ErrorCode::Failed,
                    &format!("lecture des réglages impossible : {error}"),
                ),
                EXIT_FAILURE,
            );
        }
    };
    let settings = loaded.settings;
    let path = store.path();
    match format {
        Format::Json => format!(
            "{{\"path\":{path:?},\"language\":{language:?},\"dry_run\":{dry_run},\
             \"history_retention_days\":{retention}}}",
            path = path.display().to_string(),
            language = settings.language,
            dry_run = settings.dry_run,
            retention = settings.history_retention_days,
        ),
        Format::Human => format!(
            "réglages : {}\nlangue : {}\nsimulation : {}\nhistorique : {} jours",
            path.display(),
            settings.language,
            if settings.dry_run { "oui" } else { "non" },
            settings.history_retention_days
        ),
    }
    .into()
}

/// Demande son état au service, par le canal nommé.
fn status(format: Format) -> Outcome {
    match crate::transport::ask_status() {
        Ok(snapshot) => Outcome::success(render_status(format, &snapshot)),
        Err(error) => {
            // Le code vient de la nature de l'échec, pas du texte du message : une fonction
            // absente du système n'est pas une panne, et n'a donc pas le même code de sortie.
            let code = match &error {
                obliniscor_platform::PlatformError::Unavailable(_) => ErrorCode::Unavailable,
                obliniscor_platform::PlatformError::Denied(_)
                | obliniscor_platform::PlatformError::MissingPrivilege(_) => ErrorCode::Denied,
                obliniscor_platform::PlatformError::Failed(_) => ErrorCode::Failed,
            };
            let message = error.to_string();
            let exit = if message.contains("connexion au canal") {
                EXIT_UNREACHABLE
            } else {
                exit_code_of(code)
            };
            Outcome::failure(render_error(format, code, &message), exit)
        }
    }
}

fn render_status(format: Format, snapshot: &obliniscor_core::MemorySnapshot) -> String {
    match format {
        Format::Json => format!(
            "{{\"total\":{},\"available\":{},\"free\":{}}}",
            snapshot.total, snapshot.available, snapshot.free
        ),
        Format::Human => format!(
            "mémoire : {:.1} Gio au total, {:.1} Gio disponibles, {:.1} Gio libres",
            gio(snapshot.total),
            gio(snapshot.available),
            gio(snapshot.free)
        ),
    }
}

fn render_error(format: Format, code: ErrorCode, message: &str) -> String {
    match format {
        Format::Json => format!(
            "{{\"error\":{{\"code\":{:?},\"message\":{message:?}}}}}",
            code.as_str()
        ),
        Format::Human => format!("erreur {} : {message}", code.as_str()),
    }
}

fn gio(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

fn config_dir() -> Option<PathBuf> {
    lpdnsoft_settings::config_dir("obliniscor")
}

impl From<String> for Outcome {
    fn from(text: String) -> Self {
        Self::success(text)
    }
}
