//! Contrôle du document de suivi.
//!
//! Le suivi est un fichier texte, et un fichier texte se désynchronise : une tâche oubliée dans
//! une colonne, un blocage sans ligne au registre, un journal daté à l'envers. Ces erreurs ne
//! cassent rien, elles **mentent** — et un tableau de bord qui ment est pire qu'aucun tableau.
//!
//! Ce module rend la discipline mécanique : les mêmes règles qu'un relecteur appliquerait à
//! l'œil, vérifiées à chaque `cargo xtask ci`.

use anyhow::{Result, bail};

/// Les six statuts autorisés, dans l'ordre des colonnes du tableau de bord.
pub const STATUSES: [&str; 6] = [
    "À faire", "En cours", "En revue", "Bloqué", "Fait", "Reporté",
];

/// Une ligne de tâche : son identifiant et son statut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub status: String,
}

/// Une ligne du tableau de bord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseRow {
    pub label: String,
    pub tasks: usize,
    pub counts: [usize; 6],
}

/// Le document analysé.
#[derive(Debug, Default)]
pub struct Tracking {
    pub phases: Vec<PhaseRow>,
    pub total: Option<PhaseRow>,
    pub tasks: Vec<Task>,
    /// Références de tâches citées au registre des blocages.
    pub blocked_registry: Vec<String>,
    /// Dates du journal, dans l'ordre du fichier.
    pub journal_dates: Vec<String>,
}

/// Lit le document et rend l'analyse, sans encore juger.
pub fn parse(content: &str) -> Result<Tracking> {
    let mut tracking = Tracking::default();
    let mut section = Section::Other;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            section = Section::of(heading);
        }

        if let Some(row) = parse_dashboard_row(line) {
            if row.label.contains("Total") {
                tracking.total = Some(row);
            } else {
                tracking.phases.push(row);
            }
            continue;
        }

        if section == Section::Tasks
            && let Some(task) = parse_task_row(line)
        {
            tracking.tasks.push(task);
            continue;
        }

        if section == Section::Blockages
            && trimmed.starts_with("| B-")
            && let Some(reference) = first_task_reference(trimmed)
        {
            tracking.blocked_registry.push(reference);
            continue;
        }

        if section == Section::Journal
            && let Some(date) = parse_journal_date(trimmed)
        {
            tracking.journal_dates.push(date);
        }
    }

    Ok(tracking)
}

/// Juge l'analyse : chaque règle du mode d'emploi qui peut être vérifiée mécaniquement.
pub fn verify(tracking: &Tracking) -> Result<()> {
    // 1. Des identifiants uniques : deux lignes pour une même tâche, et les comptes mentent.
    let mut ids: Vec<&str> = tracking.tasks.iter().map(|task| task.id.as_str()).collect();
    let before = ids.len();
    ids.sort_unstable();
    ids.dedup();
    if ids.len() != before {
        bail!("une tâche apparaît plusieurs fois dans les tableaux de phase");
    }

    // 2. Des statuts connus : un statut inventé n'a pas de colonne, donc fausse le total.
    for task in &tracking.tasks {
        if !STATUSES.contains(&task.status.as_str()) {
            bail!("statut inconnu `{}` pour {}", task.status, task.id);
        }
    }

    // 3. Chaque ligne du tableau de bord est cohérente avec elle-même.
    for row in tracking.phases.iter().chain(tracking.total.iter()) {
        let sum: usize = row.counts.iter().sum();
        if sum != row.tasks {
            bail!(
                "`{}` annonce {} tâches mais ses colonnes en comptent {sum}",
                row.label,
                row.tasks
            );
        }
    }

    // 4. Le total est la somme des phases.
    let Some(total) = &tracking.total else {
        bail!("ligne « Total » absente du tableau de bord");
    };
    let tasks_sum: usize = tracking.phases.iter().map(|row| row.tasks).sum();
    if total.tasks != tasks_sum {
        bail!(
            "le total annonce {} tâches, les phases en annoncent {tasks_sum}",
            total.tasks
        );
    }
    for (index, status) in STATUSES.iter().enumerate() {
        let sum: usize = tracking.phases.iter().map(|row| row.counts[index]).sum();
        if total.counts[index] != sum {
            bail!(
                "colonne `{status}` : total {}, phases {sum}",
                total.counts[index]
            );
        }
    }

    // 5. Les tableaux de phase décrivent exactement les tâches du total.
    if tracking.tasks.len() != total.tasks {
        bail!(
            "le tableau de bord annonce {} tâches, {} lignes de phase existent",
            total.tasks,
            tracking.tasks.len()
        );
    }

    // 6. Chaque statut du tableau de bord correspond au nombre réel de tâches dans ce statut.
    let mut counted = [0usize; 6];
    for task in &tracking.tasks {
        if let Some(index) = STATUSES.iter().position(|status| *status == task.status) {
            counted[index] += 1;
        }
    }
    if counted != total.counts {
        bail!(
            "le tableau de bord annonce {:?}, les lignes de tâche donnent {counted:?}",
            total.counts
        );
    }

    // 7. Un blocage a une ligne au registre, et le registre ne cite que des tâches bloquées.
    for task in tracking.tasks.iter().filter(|task| task.status == "Bloqué") {
        if !tracking.blocked_registry.contains(&task.id) {
            bail!(
                "{} est bloquée sans ligne au registre des blocages (section 4)",
                task.id
            );
        }
    }
    for reference in &tracking.blocked_registry {
        let known = tracking
            .tasks
            .iter()
            .any(|task| &task.id == reference && task.status == "Bloqué");
        if !known {
            bail!(
                "le registre cite {} qui n'est pas bloquée dans les tableaux",
                reference
            );
        }
    }

    // 8. Le journal est **monotone** : du plus récent au plus ancien, ou l'inverse, mais jamais
    // mélangé. Le sens importe peu — un journal se lit dans les deux sens — tandis qu'un ordre
    // incohérent cache une entrée ajoutée au mauvais endroit, donc oubliée à la relecture.
    let ascending = tracking
        .journal_dates
        .windows(2)
        .all(|pair| pair[0] <= pair[1]);
    let descending = tracking
        .journal_dates
        .windows(2)
        .all(|pair| pair[0] >= pair[1]);
    if !ascending && !descending {
        bail!("le journal de suivi n'est trié dans aucun sens : ordre incohérent");
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Section {
    Tasks,
    Blockages,
    Journal,
    Other,
}

impl Section {
    fn of(heading: &str) -> Self {
        if heading.starts_with("3.") {
            Self::Tasks
        } else if heading.starts_with("4.") {
            Self::Blockages
        } else if heading.starts_with("6.") {
            Self::Journal
        } else {
            Self::Other
        }
    }
}

/// Lit `| Phase | tâches | à faire | en cours | en revue | bloqué | fait | reporté | … |`.
fn parse_dashboard_row(line: &str) -> Option<PhaseRow> {
    let trimmed = line.trim();
    // Les phases commencent par `P` ; la table des tâches optionnelles par `Tâches C`. Les deux
    // comptent dans le total, et les oublier fait tomber la somme — ce que ce contrôle a
    // justement signalé au premier passage (88 annoncées, 83 comptées).
    let is_phase = trimmed.starts_with("| P") || trimmed.starts_with("| Tâches");
    if !is_phase && !trimmed.starts_with("| **Total**") {
        return None;
    }
    let cells: Vec<&str> = trimmed
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect();
    if cells.len() < 8 {
        return None;
    }
    let numbers: Option<Vec<usize>> = cells[1..8]
        .iter()
        .map(|cell| cell.trim_matches('*').parse::<usize>().ok())
        .collect();
    let numbers = numbers?;
    Some(PhaseRow {
        label: cells[0].trim_matches('*').to_owned(),
        tasks: numbers[0],
        counts: [
            numbers[1], numbers[2], numbers[3], numbers[4], numbers[5], numbers[6],
        ],
    })
}

/// Lit `| OBL-011 | tâche | statut | … |`.
fn parse_task_row(line: &str) -> Option<Task> {
    let trimmed = line.trim();
    if !trimmed.starts_with("| OBL-") {
        return None;
    }
    let cells: Vec<&str> = trimmed
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect();
    if cells.len() < 3 {
        return None;
    }
    Some(Task {
        id: cells[0].to_owned(),
        status: cells[2].to_owned(),
    })
}

/// Première référence `OBL-nnn` d'une ligne du registre.
fn first_task_reference(line: &str) -> Option<String> {
    line.split('|')
        .map(str::trim)
        .find(|cell| cell.starts_with("OBL-"))
        .map(str::to_owned)
}

/// Date d'une ligne de journal, si elle en porte une (`| 2026-11-03 | …`).
fn parse_journal_date(line: &str) -> Option<String> {
    if !line.starts_with('|') {
        return None;
    }
    let cell = line.trim_matches('|').split('|').next()?.trim();
    let bytes = cell.as_bytes();
    let looks_like_date = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && cell[..4].chars().all(|c| c.is_ascii_digit())
        && cell[5..7].chars().all(|c| c.is_ascii_digit())
        && cell[8..10].chars().all(|c| c.is_ascii_digit());
    looks_like_date.then(|| cell.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCUMENT: &str = "\
## 2. Tableau de bord

| Phase | Tâches | À faire | En cours | En revue | Bloqué | Fait | Reporté | Avancement |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| P0 Cadrage | 2 | 0 | 0 | 0 | 1 | 1 | 0 | 50 % |
| P1 Fondations | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 % |
| **Total** | **3** | **1** | **0** | **0** | **1** | **1** | **0** | **33 %** |

## 3. Tableaux de suivi par phase

| OBL-001 | un | Fait | — | 1 | 1 | — |  |
| OBL-005 | deux | Bloqué | — | 3 | 1 | 003 |  |
| OBL-011 | trois | À faire | — | 2 | — | 010 |  |

## 4. Registre des blocages

| Réf. | Tâche | Description |
| --- | --- | --- |
| B-002 | OBL-005 | attend une machine |

## 6. Journal de suivi

| Date | Tâches | Résumé |
| --- | --- | --- |
| 2026-11-01 | OBL-001 | fait |
| 2026-11-03 | OBL-005 | bloqué |
";

    #[test]
    fn a_coherent_document_passes() {
        let tracking = parse(DOCUMENT).expect("analyse");
        verify(&tracking).expect("document cohérent accepté");
        assert_eq!(tracking.tasks.len(), 3);
        assert_eq!(tracking.blocked_registry, ["OBL-005"]);
    }

    #[test]
    fn a_dashboard_that_does_not_sum_is_refused() {
        let broken = DOCUMENT.replace(
            "| **Total** | **3** | **1** | **0** | **0** | **1** | **1** | **0**",
            "| **Total** | **4** | **2** | **0** | **0** | **1** | **1** | **0**",
        );
        let tracking = parse(&broken).expect("analyse");
        let error = verify(&tracking).expect_err("total incohérent");
        assert!(error.to_string().contains("total annonce"), "{error}");
    }

    #[test]
    fn a_blocked_task_without_registry_line_is_refused() {
        let broken = DOCUMENT.replace("| B-002 | OBL-005 | attend une machine |", "");
        let tracking = parse(&broken).expect("analyse");
        let error = verify(&tracking).expect_err("blocage sans registre");
        assert!(
            error.to_string().contains("sans ligne au registre"),
            "{error}"
        );
    }

    #[test]
    fn a_registry_line_without_blocked_task_is_refused() {
        let broken = DOCUMENT.replace(
            "| B-002 | OBL-005 | attend une machine |",
            "| B-002 | OBL-005 | attend une machine |\n| B-004 | OBL-011 | citée à tort |",
        );
        let tracking = parse(&broken).expect("analyse");
        let error = verify(&tracking).expect_err("registre orphelin");
        assert!(error.to_string().contains("registre cite"), "{error}");
    }

    #[test]
    fn an_unknown_status_is_refused() {
        let broken = DOCUMENT.replace(
            "| OBL-011 | trois | À faire",
            "| OBL-011 | trois | En attente",
        );
        let tracking = parse(&broken).expect("analyse");
        let error = verify(&tracking).expect_err("statut inconnu");
        assert!(error.to_string().contains("statut inconnu"), "{error}");
    }

    #[test]
    fn a_journal_mixing_both_orders_is_refused() {
        let broken = DOCUMENT.replace(
            "| 2026-11-01 | OBL-001 | fait |\n| 2026-11-03 | OBL-005 | bloqué |",
            "| 2026-11-03 | OBL-005 | bloqué |\n| 2026-11-01 | OBL-001 | fait |\n| 2026-11-02 | OBL-011 | trois |",
        );
        let tracking = parse(&broken).expect("analyse");
        let error = verify(&tracking).expect_err("journal à l'envers");
        assert!(error.to_string().contains("aucun sens"), "{error}");
    }

    #[test]
    fn a_duplicated_task_is_refused() {
        let broken = DOCUMENT.replace(
            "| OBL-011 | trois | À faire | — | 2 | — | 010 |  |",
            "| OBL-011 | trois | À faire | — | 2 | — | 010 |  |\n| OBL-011 | trois | À faire | — | 2 | — | 010 |  |",
        );
        let tracking = parse(&broken).expect("analyse");
        // Le total ne correspond plus non plus : c'est la première règle qui parle.
        let error = verify(&tracking).expect_err("doublon");
        assert!(!error.to_string().is_empty());
    }
}
