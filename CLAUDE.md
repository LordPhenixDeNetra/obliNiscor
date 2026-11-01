# obliNiscor — consignes pour les agents

Utilitaire d'entretien du poste (mémoire, processus, stockage, réseau), en Rust, pour
**Windows, Linux et macOS**. Décisions : `docs/adr/`. Référence : le cahier des charges.

## Disposition des dépôts

Les trois dépôts de la suite sont clonés **côte à côte** dans le même dossier parent ;
`obliNiscor` dépend de `../lpdnsoft-libs/crates/<nom>`.

```
LPDNSoft/
├── lpdnsoft-libs/        (log, settings, i18n, atomic, ui)
├── lpdnsoft-launcher/    (installe et met à jour les applications)
├── ankou/                (client VPN)
└── obliNiscor/           dépend de ../lpdnsoft-libs/crates/<nom>
```

## Structure prévue (cahier des charges §8.2)

- `crates/core` — `obliniscor-core` : modèles, politiques, garde-fous, planificateur.
  **Aucune E/S**, `#![forbid(unsafe_code)]`, testable sur toutes plateformes.
- `crates/proto` — `obliniscor-proto` : messages IPC, cadrage par longueur (1 Mio), versions.
- `crates/platform` — `obliniscor-platform` : traits (`MemoryProbe`, `MemoryCleaner`,
  `ProcessControl`, `DiskInventory`, `FirewallControl`, `ServiceControl`, `PowerEvents`) et
  doublures de test.
- `crates/sys` — `obliniscor-sys` : la seule couche qui parle au système, un module par
  système (`windows/`, `linux/`, `macos/`) puis par domaine. **Seul crate où `unsafe` est
  permis** ; sous Linux et macOS il peut et doit rester absent.
- `crates/store` — `obliniscor-store` : historique JSONL, règles, catalogue de cibles, état
  d'origine pour rétablissement.
- `crates/service` — `obliniscor-service` : boucle du service, serveur IPC, planificateur.
- `crates/client` — `obliniscor-client` : client IPC partagé (GUI et CLI).
- `apps/obliniscor` — interface egui ; `apps/obliniscord` — service ; `apps/obliniscor-cli`.
- `xtask` — `cargo xtask ci`, empaquetage.

## Commande unique

```sh
cargo xtask ci   # fmt --check, clippy -D warnings, tests, cargo deny check
```

Prérequis : Rust stable ≥ 1.95, `cargo install cargo-deny --locked`. La CI tourne sur
**Windows, Linux et macOS** : une régression Unix ne doit pas attendre la phase P6b.

## Règles

- Édition 2024, `resolver = "3"`, licence `MIT OR Apache-2.0`.
- `unsafe_code = "forbid"` par défaut, dérogation **uniquement** dans `obliniscor-sys` ;
  `undocumented_unsafe_blocks` en erreur ; clippy `unwrap_used`, `expect_used`, `dbg_macro`,
  `todo` en `deny` (autorisés dans les tests).
- Une erreur `thiserror` par crate de bibliothèque ; `anyhow` seulement dans `apps/*` et
  `xtask`.
- Préfixe des crates : `obliniscor-`.
- Langues : `en` et `fr`, repli `en`.
- **Clean room (ADR-001)** : jamais de code recopié de WinMemoryCleaner (GPL-3.0). Les
  comportements s'observent, la source se cite.

## Disponibilité par système (ADR-009, matrice 6.0)

Une fonction absente **n'est pas simulée** : elle est déclarée indisponible avec sa raison.
Notamment : sous macOS, le nettoyage mémoire, la réduction de working set et le blocage
réseau par application n'existent pas ; sous Linux, le nettoyage mémoire est réservé à root et
désactivé par défaut.

## i18n

Aucune chaîne visible en dur dans `apps/*` ni dans l'interface : tout passe par
`t!(&loc, "id", arg = valeur)`. Ajouter chaque clé en `en` **et** en `fr` — une clé absente
casse la compilation.

## Tests : définition de « terminé »

- **Ajouter les tests du comportement modifié**, dans le même changement que le code.
- **`cargo xtask ci` en exit 0**, sur les trois systèmes.
- **Un test nouveau échoue sans le changement.**
- **Écrire d'abord le test du cas dangereux** : suppression hors racine, règle réseau non
  retirée, privilège non rendu.
- **Placer les tests selon leur nature** : logique privée en `#[cfg(test)] mod tests`, API
  publique en `crates/<crate>/tests/*.rs`. On n'élargit jamais une API pour la tester.
- **Indiquer dans la livraison** la commande lancée et son résultat exact, avec les commandes
  git et le formulaire de demande de fusion.

## État et périmètre

**Le code s'arrête au squelette.** L'espace de travail existe et `cargo xtask ci` est vert :
`fmt --check`, `clippy -D warnings`, trois tests, `cargo deny check`, et le contrôle i18n
(les deux langues contiennent les mêmes clés, et les clés produites par le code existent).
Le crate `obliniscor-core` ne contient que des types purs. Rien ne parle encore au système :
la couche `obliniscor-sys` et les binaires arrivent avec leurs tâches.

Les prochains travaux sont **OBL-004** (intégration continue sur les trois systèmes, qui doit
cloner `../lpdnsoft-libs`) puis les **spikes OBL-005 à OBL-009**, qui lèvent les hypothèses
« à valider » du cahier des charges avant tout code de production.
