# obliNiscor

Entretien du poste de travail — mémoire, processus, stockage, réseau — pour **Windows, Linux
et macOS**. Septième projet de la suite **LPDNSoft**, après `lpdnsoft-libs`, `ankou` et
`lpdnsoft-launcher`.

> **État : spécification seulement.** Aucune ligne de code, aucun binaire, aucune version
> publiée. Ce dossier contient aujourd'hui le cahier des charges et le suivi des tâches, et
> rien d'autre. Le README le dit parce que c'est sa seule fonction honnête pour l'instant.

## Ce qui existe

| Élément | Fichier | État |
| --- | --- | --- |
| Cahier des charges | `obliNiscor — Cahier des charges et cahier des tâches.md` | v0.2, multi-plateforme |
| Suivi des tâches | `obliNiscor — Suivi des tâches et guide d'exécution.md` | v0.2, 87 tâches, 3 en revue |
| Espace de travail | `Cargo.toml`, `crates/core`, `xtask`, `locales` | `cargo xtask ci` vert : 3 tests, contrôle i18n |
| Binaires, service, interface | — | **aucun** |

Le code s'arrête au noyau : des types purs (`MemoryZone`, `Availability`) et leur règle
centrale — une fonction indisponible n'est jamais proposée. Rien ne parle encore au système.


## Ce que le produit fera

| Module | Windows | Linux | macOS |
| --- | --- | --- | --- |
| Mesure de la mémoire | complet | complet | complet |
| Nettoyage mémoire | complet | partiel (root, désactivé par défaut) | **indisponible** |
| Processus (liste, priorité, arrêt) | complet | complet | complet |
| Réduction de working set | complet | possible (droits requis) | **indisponible** |
| Stockage (inventaire, analyse, nettoyage sûr) | complet | complet | complet |
| Blocage réseau par application | complet | complet (nftables) | **indisponible** |

La matrice complète, avec les raisons et les mécanismes, est la section 6.0 du cahier des
charges. Une fonction indisponible est **annoncée avec sa raison**, jamais simulée — c'est
l'ADR-009.

## Principes

1. Mesurer avant d'agir, et afficher le gain **et** son coût.
2. Rien de destructeur sans aperçu, confirmation et journal.
3. Tout changement réseau est réversible (`net restore`).
4. Privilèges minimaux : un service les porte, l'interface et la CLI n'en ont aucun.
5. Aucune télémétrie.
6. Une fonction absente est annoncée, jamais simulée.

## Prochaines étapes

Le cahier des tâches est la référence : commencer par **OBL-003** (dépôt, outillage,
`cargo xtask ci`) puis les spikes **P0**, qui sont là pour lever les hypothèses marquées
« à valider » avant d'écrire le reste.

## Contribuer

`CONTRIBUTING.md`, et `docs/adr/0000-modele.md` pour le format d'une décision. Les documents
de référence restent la source de vérité : un changement de comportement se discute d'abord
dans le cahier des charges, ensuite dans le code.
