//! Analyse des arguments — la partie pure, donc testable sans lancer de commande.
//!
//! Pas de bibliothèque d'analyse d'arguments : la surface est de quatre commandes et deux
//! options. Une dépendance de plus serait un coût de compilation, d'audit et de licence pour
//! une centaine de lignes, dans un binaire qu'on veut petit.

/// Ce que l'utilisateur a demandé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub command: Command,
    pub format: Format,
}

/// La commande du squelette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Affiche la version et le numéro de protocole.
    Version,
    /// Demande l'état de la mémoire au service.
    Status,
    /// Montre les réglages et leur emplacement.
    Config,
    /// L'aide.
    Help,
}

/// Le format de sortie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Texte lisible par un humain.
    Human,
    /// Objet JSON sur une ligne, pour un script.
    Json,
}

/// Erreur d'analyse : une faute de frappe, pas une panne.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError {
    pub message: String,
}

/// Lit les arguments (sans le nom du programme).
///
/// La commande est facultative et vaut `status` par défaut : c'est ce que veut voir un
/// utilisateur qui lance l'outil sans rien préciser.
pub fn parse(args: &[String]) -> Result<Invocation, UsageError> {
    let mut command: Option<Command> = None;
    let mut format = Format::Human;

    for argument in args {
        match argument.as_str() {
            "--json" => format = Format::Json,
            "--help" | "-h" => command = Some(Command::Help),
            "--version" | "-V" => command = Some(Command::Version),
            "version" => set(&mut command, Command::Version)?,
            "status" => set(&mut command, Command::Status)?,
            "config" => set(&mut command, Command::Config)?,
            "help" => set(&mut command, Command::Help)?,
            other => {
                return Err(UsageError {
                    message: format!("argument inconnu : `{other}`"),
                });
            }
        }
    }

    Ok(Invocation {
        command: command.unwrap_or(Command::Status),
        format,
    })
}

/// Refuse deux commandes à la fois : `version status` n'a pas de sens.
fn set(command: &mut Option<Command>, value: Command) -> Result<(), UsageError> {
    if command.is_some() {
        return Err(UsageError {
            message: "une seule commande à la fois".to_owned(),
        });
    }
    *command = Some(value);
    Ok(())
}

/// Texte d'aide, bilingue par construction : il cite les codes de sortie, qui sont le contrat.
pub const HELP: &str = "\
obliniscor — entretien du poste (mémoire, processus, stockage)

Usage : obliniscor [COMMANDE] [OPTIONS]

Commandes :
  status        état de la mémoire (défaut)
  config        réglages et emplacement du fichier
  version       version de l'outil et du protocole
  help          cette aide

Options :
  --json        sortie JSON sur une ligne
  --help, -h    cette aide
  --version, -V version

Codes de sortie :
  0  succès
  1  échec interne
  2  usage invalide
  3  service injoignable
  4  accès refusé
  5  fonction indisponible sur ce système";

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn no_argument_asks_for_the_status() {
        let invocation = parse(&[]).expect("défaut");
        assert_eq!(invocation.command, Command::Status);
        assert_eq!(invocation.format, Format::Human);
    }

    #[test]
    fn a_command_and_a_format_are_both_understood() {
        let invocation = parse(&args(&["status", "--json"])).expect("analyse");
        assert_eq!(invocation.command, Command::Status);
        assert_eq!(invocation.format, Format::Json);
    }

    #[test]
    fn the_options_are_order_independent() {
        let first = parse(&args(&["--json", "config"])).expect("analyse");
        let second = parse(&args(&["config", "--json"])).expect("analyse");
        assert_eq!(first, second);
    }

    #[test]
    fn an_unknown_argument_is_a_usage_error_naming_it() {
        let error = parse(&args(&["--prettier"])).expect_err("inconnu");
        assert!(error.message.contains("--prettier"), "{}", error.message);
    }

    #[test]
    fn two_commands_are_refused() {
        let error = parse(&args(&["version", "status"])).expect_err("deux commandes");
        assert!(error.message.contains("une seule"), "{}", error.message);
    }

    #[test]
    fn the_version_flags_are_equivalent() {
        assert_eq!(
            parse(&args(&["--version"])).expect("analyse").command,
            parse(&args(&["version"])).expect("analyse").command
        );
    }
}
