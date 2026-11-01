# ADR 0001 — Réécriture propre (clean room) et licence

- **Statut :** proposé
- **Date :** 2026-11-01
- **Complète :** ADR-008 (licence de la suite)
- **Décide :** OBL-001, et conditionne l'ADR-0002 (nom) et l'ADR-0008 (licence)

## Contexte

obliNiscor reprend l'**idée** de WinMemoryCleaner : mesurer la mémoire, libérer des zones,
régler des processus, nettoyer des fichiers temporaires, bloquer des applications sur le
réseau. Ce projet est distribué sous **GPL-3.0**.

La suite LPDNSoft est sous **MIT** (`lpdnsoft-launcher`) ou **MIT OR Apache-2.0** (`ankou`,
`lpdnsoft-libs`). Publier obliNiscor sous GPL l'exclurait de fait de la suite : un projet
GPL ne peut pas être lié à des bibliothèques permissives puis redistribué sous MIT, et
l'utilisateur d'un launcher ne s'attend pas à ce qu'une application installée change la
licence du logiciel qu'il utilise.

Le risque n'est pas seulement juridique : il est aussi de croire qu'un renommage de variables
suffit à effacer une contrainte de licence. Ce n'est pas le cas — c'est la **structure** et
l'**expression** du code qui sont protégées, pas les noms.

## Options examinées

| Option | Avantages | Inconvénients |
| --- | --- | --- |
| **A. Traduire le code C# en Rust** | Rapide, comportement fidèle | Œuvre dérivée : obliNiscor devient GPL-3.0 (ou doit être publié sous GPL), ce qui casse la cohérence de la suite et la redistribution |
| **B. Réécriture propre (clean room)** | Permet MIT ou Apache-2.0 ; le code est le nôtre, donc maintenable | Plus lent : il faut comprendre les comportements par la documentation et la mesure, pas par la lecture du code |
| **C. Ne réimplémenter que les fonctions non ambiguës** | Réduit le périmètre | Le périmètre perd sa raison d'être : les zones mémoire *sont* le cœur du produit |

## Décision

**Option B — réécriture propre.** Le comportement s'observe, le code ne se lit pas.

Les règles opérationnelles, qui rendent la décision vérifiable plutôt que déclarative :

1. **Sources autorisées** : la documentation Microsoft et Apple, les crates sous licence
   MIT ou Apache-2.0, les outils publics utilisés comme **référence de comportement**
   (RAMMap, Moniteur de ressources, `free`, `vm_stat`), et l'expérimentation propre.
2. **Sources interdites** : le code source de WinMemoryCleaner, ses fichiers `.cs`, ses
   ressources, toute décompilation ou désassemblage. Aucun de ces éléments n'entre dans le
   dépôt, même comme « référence ».
3. **Journal des sources** : chaque appel système porte en commentaire la source qui l'a
   documenté (page Microsoft Learn ou Apple, avec sa date de consultation). Sans source, le
   code n'est pas relu.
4. **Déclaration de contribution** : la case « aucun code copié d'un projet de licence
   incompatible » figure dans `.github/pull_request_template.md`, et le contributeur la coche
   en connaissance de cause (voir aussi `docs/adr/0000-modele.md`, section *Sources*).
5. **Spécifier avant de coder** : le comportement attendu s'écrit d'abord (cahier des
   charges, `docs/spikes/`), le code vient ensuite. On ne rouvre jamais la source d'origine
   pour « vérifier un détail ».
6. **En cas de doute, on remplace** : si un comportement ne peut être établi que par le code
   GPL, il sort du périmètre ou se réimplémente à partir d'une mesure.

## Conséquences

- **Sur le calendrier :** la phase P0 existe pour cela. Les spikes mémoire, stockage,
  pare-feu et service (OBL-005 à OBL-008) sont le prix de la décision, et ils sont estimés
  dans le cahier des tâches.
- **Sur la licence :** ADR-0008 peut choisir MIT ou Apache-2.0, comme le reste de la suite.
  Un avis juridique reste recommandé avant la première publication : la décision est
  documentée, pas garantie.
- **Sur le multi-plateforme :** la révision multi-plateforme (ADR-0004) **réduit** le risque.
  Les backends Linux et macOS sont du code neuf, sans contrepartie GPL : la zone à surveiller
  se limite au module Windows.
- **Sur la revue :** la case du formulaire de PR n'est pas décorative. Un contributeur qui ne
  peut pas l'affirmer ouvre une discussion avant d'écrire la ligne concernée.
- **Ce qui reste ouvert :** la décision de licence définitive (Q1), et l'avis juridique
  éventuel. Si une contamination était constatée, la sortie est assumée à l'avance :
  republier obliNiscor sous GPL-3.0 plutôt que de masquer le problème.

## Sources

- La licence de WinMemoryCleaner (GPL-3.0) est une **observation** du dépôt public, pas une
  lecture de son code.
- Documentation citée pour les comportements : Microsoft Learn (mémoire, listes de pages,
  pare-feu) et Apple Developer (mémoire, `libproc`, Network Extensions) — les pages précises
  sont consignées dans `docs/spikes/` au fur et à mesure des tâches.
- Aucun extrait de code n'est reproduit dans ce document.
