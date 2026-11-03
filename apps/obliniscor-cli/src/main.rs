//! `obliniscor` — la commande.
//!
//! Elle ne fait rien elle-même : elle demande au service, ou lit un fichier de réglages. Cette
//! séparation est celle de l'architecture (ADR-004) : un binaire qui toucherait directement au
//! système rendrait le service inutile et la GUI impossible.

mod cli;
mod commands;
mod transport;

use cli::{Format, Invocation};
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    let invocation: Invocation = match cli::parse(&arguments) {
        Ok(invocation) => invocation,
        Err(error) => {
            // Un usage invalide s'explique : le message seul laisserait l'utilisateur deviner.
            eprintln!("obliniscor : {}", error.message);
            eprintln!();
            eprintln!("{}", cli::HELP);
            return ExitCode::from(commands::EXIT_USAGE as u8);
        }
    };

    let outcome = commands::run(&invocation.command, invocation.format);
    match invocation.format {
        // En JSON, la sortie standard ne contient qu'une ligne : un script peut la lire sans
        // filtrer un en-tête.
        Format::Json => println!("{}", outcome.text),
        Format::Human => println!("{}", outcome.text),
    }
    ExitCode::from(outcome.exit as u8)
}
