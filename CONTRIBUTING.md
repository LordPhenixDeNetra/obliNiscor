# Contribuer à obliNiscor

Utilitaire d'entretien du poste, en Rust, pour **Windows, Linux et macOS**. Le cahier des
charges (`obliNiscor — Cahier des charges et cahier des tâches.md`) est la référence : les
exigences `FR-*`, `NFR-*` et `SEC-*` y sont numérotées, et chaque tâche `OBL-*` en découle.

## Avant de lire le code

**Règle clean room (ADR-001).** obliNiscor reprend les *comportements* de WinMemoryCleaner,
jamais son code : ce projet est sous GPL-3.0, et une traduction rendrait obliNiscor dérivé,
donc soumis à la GPL. Les sources autorisées sont la documentation Microsoft et Apple, les
crates sous licence MIT ou Apache-2.0, et l'expérimentation personnelle. Chaque appel système
porte en commentaire la source qui l'a documenté, et le **journal des sources** est tenu dans
les ADR.

## Prérequis

- Rust stable **≥ 1.95** (édition 2024, `resolver = "3"`) ;
- `cargo install cargo-deny --locked` ;
- les bibliothèques de la suite clonées **à côté** de ce dépôt : `lpdnsoft-libs`
  (`lpdnsoft-log`, `lpdnsoft-settings`, `lpdnsoft-i18n`, `lpdnsoft-atomic`, `lpdnsoft-ui`) ;
- pour les tests d'intégration : un Windows avec droits administrateur, un Linux avec
  `nftables` et `systemd`, un macOS 12 ou plus récent.

## Construire et tester

```sh
cargo xtask ci        # fmt, clippy -D warnings, tests, cargo deny
cargo run -p obliniscor-cli -- version
```

**`cargo xtask ci` est la seule commande à connaître**, et elle doit sortir en exit 0 **sur
les trois systèmes** : le noyau et la CLI sont portables, et une régression Linux ne doit pas
attendre la phase P6b pour être vue.

Les tests qui touchent le système réel sont marqués `#[ignore]` et lancés à la main ; leur
résultat se consigne dans `docs/spikes/` ou `docs/mesures/`.

## Règles de code

- **`unsafe` est interdit partout, sauf dans `obliniscor-sys`.** Sous Linux et macOS, la
  couche peut et doit rester sans `unsafe` : lire `/proc`, appeler `openat2` par `nix`, lancer
  `nft` ou `fstrim` suffit. Chaque bloc `unsafe` est justifié par un commentaire `// SAFETY:`
  et le lint `undocumented_unsafe_blocks` est en erreur.
- **Une erreur `thiserror` par crate** ; `anyhow` seulement dans `apps/*` et `xtask`.
- **Aucune chaîne visible en dur** : `t!(&loc, "id", …)`, et chaque clé existe en `en` **et**
  en `fr`.
- **Une décision s'écrit** dans `docs/adr/` avant le code : contexte, options écartées,
  conséquences.
- **Une fonction indisponible se déclare.** La matrice 6.0 dit ce qui existe par système ; le
  code renvoie « non disponible » avec la raison, et l'interface l'affiche. On ne simule
  jamais un effet qu'on n'a pas.
- **Le suivi vit dans le document de suivi** : une tâche commencée passe « En cours », une
  tâche livrée « Fait » avec sa preuve, dans le même changement que le code.

## Tests

- **Placer le test selon sa nature** : logique privée dans un `#[cfg(test)] mod tests` au bas
  du fichier ; API publique dans `crates/<crate>/tests/*.rs`.
- **Un test nouveau échoue sans le changement**, sinon il ne prouve rien.
- **Écrire d'abord le test du cas dangereux** : suppression hors racine, règle réseau non
  retirée, privilège non rendu.
- **Ne jamais désactiver un test existant** pour faire passer une modification.
- Sur les chemins et la suppression, doublures et tests de propriétés sont la norme.

## Commits

- **Conventional Commits**, en anglais, avec portée : `feat(mem):`, `fix(disk):`, `docs:`,
  `test(proto):`.
- **Un commit par sujet**, `git add` avec des chemins explicites.
- **Signer** : `git commit -s` (DCO).
- **Un `-m` par paragraphe** : le titre, puis le pourquoi.

## Branches et demande de fusion

- Branche `feat/…`, `fix/…`, `docs/…` ou `test/…`, fusion vers `main`.
- Titre au format des commits ; description selon `.github/pull_request_template.md`.
- Une demande de fusion cite **une** tâche `OBL-*` et démontre son critère d'acceptation.

## Signaler un problème

Décrivez le système et sa version, ce que vous faisiez, ce que vous attendiez, ce qui s'est
passé, et joignez le journal. Pour une faille de sécurité, ne passez pas par une *issue*
publique : écrivez à part, la procédure sera dans `docs/securite.md` quand elle existera.
