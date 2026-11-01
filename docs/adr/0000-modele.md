# ADR 0000 — Modèle d'une fiche de décision

- **Statut :** modèle (ne pas suivre ce numéro)
- **Date :** —
- **Remplace / complète :** —

Ce fichier est un **gabarit**. Copiez-le sous le numéro suivant (`0001-…md`) avant de le
remplir, et supprimez les mentions entre crochets.

## Contexte

[Ce qui oblige à décider : contrainte technique, exigence du cahier des charges, incident.
Citez les exigences concernées (`FR-*`, `NFR-*`, `SEC-*`) et, si la décision en remplace une
autre, l'ADR remplacé.]

## Options examinées

| Option | Avantages | Inconvénients |
| --- | --- | --- |
| [option A] | | |
| [option B] | | |

## Décision

[Décision à l'infinitif, au présent, en une ou deux phrases. Ce qui est retenu, et pourquoi
les autres options sont écartées. Une décision doit être vérifiable : on doit pouvoir dire
plus tard si le code la respecte.]

## Conséquences

- **Dans le code :** [ce que cela impose, ce que cela interdit]
- **Dans l'exploitation :** [service, installation, réversibilité]
- **Dans les tests :** [ce qui devra être prouvé, sur quel système]
- **Ce qui reste ouvert :** [ce que cette décision ne tranche pas]

## Sources

[Documentation consultée, avec sa date — la règle clean room (ADR-001) exige que chaque appel
système puisse être rattaché à une source autorisée. Pour une décision purement interne,
écrire « aucune source externe ».]

---

## Les décisions à écrire

D'après le cahier des charges, section 3. Celles marquées **à valider** attendent une preuve
(phase P0) avant d'être figées.

| ADR | Sujet | État attendu |
| --- | --- | --- |
| 0001 | Clean room et licence | À rédiger en premier : tout le reste en dépend |
| 0002 | Nom et noms de crates | Disponibilité à vérifier avant publication |
| 0003 | Séparation des privilèges (service + IPC) | Retenu dans le cahier |
| 0004 | Multi-plateforme : `obliniscor-sys` et modules par système | **Révisé le 31 octobre 2026** |
| 0005 | Réversibilité par défaut | À rédiger avec les règles de journal |
| 0006 | Mesure avant/après obligatoire | À rédiger avec la définition du gain |
| 0007 | Réseau : pare-feu par système, ou WFP en v2 | À valider par le spike OBL-007 |
| 0008 | Licence MIT ou Apache-2.0 | Dépend d'ADR-0001 |
| 0009 | Disponibilité honnête par système (matrice 6.0) | Retenu |

Une décision remplacée n'est **jamais supprimée** : elle est marquée `Remplacé par ADR-xxxx`
et sa date de remplacement est ajoutée.
