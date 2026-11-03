//! Le binaire, exécuté pour de vrai : arguments, sortie et codes de sortie.
//!
//! Ces tests lancent le binaire compilé (`CARGO_BIN_EXE_…`) plutôt que d'appeler les fonctions :
//! ce qui doit être prouvé, c'est ce qu'un script voit — la sortie et le code.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_obliniscor-cli"))
        .args(args)
        .output()
        .expect("lancement du binaire")
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn version_prints_the_tool_and_the_protocol() {
    let output = run(&["version"]);
    assert!(output.status.success(), "code {:?}", output.status.code());
    let text = stdout(&output);
    assert!(text.contains("obliniscor"), "{text}");
    assert!(text.contains("protocole 1"), "{text}");
}

#[test]
fn version_in_json_is_one_parseable_line() {
    let output = run(&["version", "--json"]);
    assert!(output.status.success());
    let text = stdout(&output);
    let lines: Vec<&str> = text.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(lines.len(), 1, "une seule ligne attendue, reçu : {text:?}");
    let value: serde_json::Value = serde_json::from_str(lines[0]).expect("JSON valide");
    assert_eq!(value["protocol"], 1);
    assert!(value["version"].is_string());
}

#[test]
fn an_unknown_argument_exits_with_the_usage_code() {
    let output = run(&["--prettier"]);
    assert_eq!(output.status.code(), Some(2));
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(errors.contains("--prettier"), "{errors}");
    // L'aide accompagne l'erreur : elle évite un aller-retour vers la documentation.
    assert!(errors.contains("Codes de sortie"), "{errors}");
}

#[test]
fn help_lists_the_exit_codes() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    let text = stdout(&output);
    for code in ["0", "2", "3", "4", "5"] {
        assert!(text.contains(code), "code {code} absent de l'aide");
    }
}

#[test]
fn config_works_without_the_service() {
    // `config` ne lit qu'un fichier : elle doit réussir même service arrêté, sinon elle ne
    // servirait à rien pour diagnostiquer une installation.
    let output = run(&["config", "--json"]);
    assert!(
        output.status.success(),
        "code {:?}, erreur : {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = stdout(&output);
    let value: serde_json::Value = serde_json::from_str(text.trim()).expect("JSON valide");
    assert!(value["path"].is_string());
    assert_eq!(value["dry_run"], true, "la simulation est le défaut");
}

#[test]
fn status_without_a_service_exits_with_the_unreachable_code() {
    // Aucun service ne tourne pendant les tests : le client doit le dire avec son code dédié,
    // et non avec un échec générique.
    let output = run(&["status", "--json"]);
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(
        code,
        3,
        "code {code}, erreur : {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_str(stdout(&output).trim()).expect("JSON");
    assert_eq!(value["error"]["code"], "E_FAILED");
}
