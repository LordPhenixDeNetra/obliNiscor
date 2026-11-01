# obliNiscor — Suivi des tâches et guide d'exécution

*Suite LPDNSoft · Document compagnon du cahier des charges · Version 0.2 · 31 octobre 2026*<br>*Révision du 31 octobre 2026 : la v1 couvre Windows, Linux et macOS (ADR-004) ; phase P6b ajoutée.*

## 1. Mode d'emploi

Ce document sert à deux choses : **suivre** l'avancement (tableaux, blocages, journal) et **bien exécuter** chaque tâche (guide ingénieur, section 7 et suivantes). Les identifiants `OBL-nnn`, les dépendances et les estimations viennent du cahier des tâches (section 14 du cahier des charges).

### 1.1 Les statuts

| Statut | Sens | On y entre quand… | On en sort quand… |
| --- | --- | --- | --- |
| **À faire** | Tâche connue, pas commencée | Elle est au plan | Ses dépendances sont « Fait » et on la prend |
| **En cours** | Quelqu'un travaille dessus | Dépendances satisfaites ; la limite de travail en cours est respectée | Le code, les tests et la doc sont prêts pour relecture |
| **En revue** | Relecture ou validation en attente | Demande de tirage ouverte, CI verte | Relecture terminée : retour « En cours » ou passage à « Fait » |
| **Bloqué** | Impossible d'avancer sans aide extérieure | Une entrée existe au registre des blocages (section 4) | Le blocage est levé : retour « En cours » |
| **Fait** | Terminée et démontrée | La définition de « terminé » (1.3) est entièrement cochée | — |
| **Reporté** | Volontairement repoussée ou abandonnée | Décision datée avec motif (surtout tâches C) | Reprise décidée : retour « À faire » |

### 1.2 Règles de fonctionnement

1. **Limite de travail en cours** : au plus **2** tâches « En cours » par personne ; **1 seule** pour les tâches à risque (OBL-005, OBL-017, OBL-053, OBL-061). Finir vaut mieux que commencer.
2. **Pas de démarrage sans dépendances** : une tâche ne passe « En cours » que si ses dépendances sont « Fait ». Exception : une tâche de spike dont le but est précisément de lever le doute.
3. **Revue sous 48 h** : une tâche « En revue » plus de deux jours est un signal à traiter (relecteur absent, demande trop grosse).
4. **Un blocage a un propriétaire et une échéance.** Si une tâche reste « Bloquée » plus de **3 jours**, on la redécoupe ou on contourne : on ne laisse pas le temps passer.
5. **Une tâche = au plus 3 jours ouvrés.** Au-delà, la découper en sous-tâches `OBL-nnn.a`, `.b`… (elles héritent du statut le plus avancé de leurs parents).
6. **L'écart d'estimation s'enregistre.** Si le réel dépasse 1,5 × l'estimation, noter la cause en « Notes » et corriger les estimations des tâches voisines du même type (calibrage).
7. **Un statut ne ment jamais.** « Fait » sans preuve est une dette ; on le repasse « En cours ».

### 1.3 Définition de « terminé » (à cocher avant « Fait »)

- [ ] Le code est fusionné et `cargo xtask ci` est vert.
- [ ] Les tests de la tâche existent et passent (unitaires, intégration, sécurité selon le cas).
- [ ] Le critère d'acceptation de la tâche est **démontré** (capture, mesure, journal ou commande rejouable), et la preuve est liée dans les notes.
- [ ] La documentation est à jour (ADR, runbook, page d'aide, `docs/api-utilisees.md` si un appel système est ajouté).
- [ ] Les chaînes d'interface sont traduites en français et en anglais.
- [ ] Chaque bloc `unsafe` a un commentaire `SAFETY:` et a été relu.
- [ ] Aucun `TODO` sans entrée de suivi associée.

### 1.4 Rituels

| Rituel | Fréquence | Durée | Contenu |
| --- | --- | --- | --- |
| Point quotidien | Chaque jour de travail | 10 min (ou 3 lignes écrites en solo) | Hier, aujourd'hui, blocages |
| Revue hebdomadaire | Une fois par semaine | 30 min | Mettre à jour le tableau de bord et le registre des blocages, relire les risques, replanifier |
| Rétrospective de phase | Fin de chaque phase | 45 min | Ce qui a marché, ce qui a coûté, ajustement des estimations et du processus |
| Démonstration de jalon | Fin de chaque jalon J0 à J8 | 30 min | Montrer la preuve de chaque critère d'acceptation |

## 2. Tableau de bord

À mettre à jour à chaque revue hebdomadaire (compter les lignes par statut dans la section 3).

| Phase | Tâches | À faire | En cours | En revue | Bloqué | Fait | Reporté | Avancement |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| P0 Cadrage et spikes | 10 | 6 | 0 | 0 | 0 | 4 | 0 | 33 % |
| P1 Fondations | 10 | 6 | 1 | 1 | 0 | 2 | 0 | 24 % |
| P2 Mémoire | 9 | 9 | 0 | 0 | 0 | 0 | 0 | 0 % |
| P3 Processus | 8 | 8 | 0 | 0 | 0 | 0 | 0 | 0 % |
| P4 Interface | 7 | 7 | 0 | 0 | 0 | 0 | 0 | 0 % |
| P5 Stockage | 7 | 7 | 0 | 0 | 0 | 0 | 0 | 0 % |
| P6 Réseau | 10 | 10 | 0 | 0 | 0 | 0 | 0 | 0 % |
| P6b Backends Linux et macOS | 9 | 9 | 0 | 0 | 0 | 0 | 0 | 0 % |
| P7 Durcissement | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 % |
| P8 Distribution | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 % |
| Tâches C | 5 | 5 | 0 | 0 | 0 | 0 | 0 | — |
| **Total** | **87** | **79** | **1** | **1** | **0** | **6** | **0** | **12 %** |

**Avancement** = jours estimés des tâches « Fait » ÷ jours estimés totaux de la phase (et non le nombre de lignes : une tâche de 4 jours pèse plus qu'une tâche d'un demi-jour).

### Jalons

| Jalon | Contenu | Date prévue | Date réelle | État |
| --- | --- | --- | --- | --- |
| J0 | Go/no-go, hypothèses levées | — | — | À venir |
| J1 | Service installable, IPC, CLI squelette | — | — | À venir |
| J2 | Alpha : mémoire en CLI avec mesure | — | — | À venir |
| J3 | Processus en CLI | — | — | À venir |
| J4 | Bêta 1 : interface mémoire et processus | — | — | À venir |
| J5 | Stockage sûr | — | — | À venir |
| J6 | Bêta 2 : réseau avec restauration garantie | — | — | À venir |
| J6b | Les trois systèmes couverts (matrice 6.0 respectée) | — | — | À venir |
| J7 | Version candidate | — | — | À venir |
| J8 | v1.0.0 publiée pour Windows, Linux et macOS | — | — | À venir |

## 3. Tableaux de suivi par phase

Colonnes : **Est.** = estimation en jours-personne ; **Réel** = jours consommés ; **Resp.** = responsable. Mettre à jour le statut, les jours réels et un lien vers la preuve dans « Notes ».

### P0 — Cadrage et validation technique

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-001 | ADR-001 clean room et licence | Fait | — | 1 | 1 | — | `docs/adr/0001-clean-room-et-licence.md` (PR #2) |
| OBL-002 | Disponibilité du nom | Fait | — | 0,5 | 0,5 | — | `docs/spikes/nom.md` : six noms libres (PR #2) |
| OBL-003 | Dépôt, outillage, `xtask ci` | Fait | — | 2 | 2 | 001 | `cargo xtask ci` + contrôle i18n (PR #2, #3) |
| OBL-004 | CI Windows x64 et ARM64 | Fait | — | 1,5 | 1,5 | 003 | trois OS + MSRV **verts** (4 jobs, PR #4) ; ARM64 natif reste à ajouter |
| OBL-005 | Spike mémoire | À faire | — | 3 | — | 003 |  |
| OBL-006 | Spike stockage | À faire | — | 1,5 | — | 003 |  |
| OBL-007 | Spike pare-feu | À faire | — | 2 | — | 003 |  |
| OBL-008 | Spike service et canal nommé | À faire | — | 1,5 | — | 003 |  |
| OBL-009 | Protocole de mesure, machines A–H | À faire | — | 1,5 | — | 003 |  |
| OBL-010 | Revue de phase, go/no-go | À faire | — | 0,5 | — | 005–009 |  |

### P1 — Fondations

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-011 | `obliniscor-core` | En revue | — | 2 | 1,5 | 010 | modèles (`MemorySnapshot`, `CleanReport`, `ProcessInfo`, `Volume`), `MemoryZone` avec noms d'IPC, `ErrorCode` stable ; 7 tests |
| OBL-012 | `obliniscor-platform` | Fait | — | 1,5 | 1,5 | 011 | 4 traits de capacité, 3 doublures, 4 tests (PR #4) |
| OBL-013 | `obliniscor-proto` | Fait | — | 2,5 | 2,5 | 011 | cadrage u32+JSON, bornes 1 Mio, 12 tests (PR #4) |
| OBL-014 | Garde RAII de privilèges | À faire | — | 1 | — | 012 |  |
| OBL-015 | Intégration des libs, réglages v1 | À faire | — | 2 | — | 011 |  |
| OBL-016 | Historique JSONL | À faire | — | 1,5 | — | 015 |  |
| OBL-017 | Service Windows | À faire | — | 3 | — | 008, 014 |  |
| OBL-018 | Serveur et client IPC | À faire | — | 3 | — | 013, 017 |  |
| OBL-019 | CLI squelette | À faire | — | 1,5 | — | 018 |  |
| OBL-100 | Socle multi-plateforme (`obliniscor-sys`, CI trois OS) | En cours | — | 3 | 2 | 011 | `sys` : trois modules et matrice 6.0 exécutable, 6 tests ; manquent ARM64 et l'empaquetage continu |

### P2 — Mémoire

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-020 | Lecture de l'état mémoire | À faire | — | 3 | — | 005, 012 |  |
| OBL-021 | Défauts de page et pression disque | À faire | — | 1,5 | — | 020 |  |
| OBL-022 | Nettoyage : working set, cache système, modifiée | À faire | — | 3 | — | 014, 020 |  |
| OBL-023 | Nettoyage : standby, combinée, fichiers, registre | À faire | — | 3,5 | — | 022 |  |
| OBL-024 | Mesure avant/après et coût | À faire | — | 2 | — | 021, 023 |  |
| OBL-025 | Politiques et garde-fous | À faire | — | 3 | — | 024 |  |
| OBL-026 | Exclusions de processus | À faire | — | 1 | — | 022 |  |
| OBL-027 | CLI `mem` | À faire | — | 1,5 | — | 018, 024 |  |
| OBL-028 | Campagne de mesures A, B, C | À faire | — | 3 | — | 024, 009 |  |

### P3 — Processus

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-030 | Énumération des processus | À faire | — | 3 | — | 012 |  |
| OBL-031 | Working set, priorité, règles | À faire | — | 2 | — | 030, 018 |  |
| OBL-032 | Liste de processus protégés | À faire | — | 1,5 | — | 030 |  |
| OBL-033 | Arrêt en deux temps | À faire | — | 1,5 | — | 032 |  |
| OBL-034 | EcoQoS et règles d'arrière-plan | À faire | — | 2,5 | — | 031 |  |
| OBL-035 | Détection de dérive mémoire | À faire | — | 1,5 | — | 030 |  |
| OBL-036 | Services : état d'origine | À faire | — | 2 | — | 018 |  |
| OBL-037 | CLI `proc` | À faire | — | 1 | — | 031–033 |  |

### P4 — Interface

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-040 | Squelette egui et thème | À faire | — | 2 | — | 018 |  |
| OBL-041 | Tableau de bord et onglet Mémoire | À faire | — | 3 | — | 024, 040 |  |
| OBL-042 | Onglet Processus | À faire | — | 3 | — | 031–033, 040 |  |
| OBL-043 | Notification, raccourci global | À faire | — | 2,5 | — | 041 |  |
| OBL-044 | Mode compact, réglages, journal | À faire | — | 2 | — | 041 |  |
| OBL-045 | i18n français et anglais | À faire | — | 1 | — | 040 |  |
| OBL-046 | Accessibilité et DPI | À faire | — | 1,5 | — | 041, 042 |  |

### P5 — Stockage

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-050 | Inventaire des volumes | À faire | — | 2 | — | 006 |  |
| OBL-051 | Analyse d'espace | À faire | — | 3 | — | 050 |  |
| OBL-052 | Catalogue de cibles | À faire | — | 3 | — | 015 |  |
| OBL-053 | Suppression sûre | À faire | — | 4 | — | 052 |  |
| OBL-054 | Aperçu, jeton, journal | À faire | — | 2 | — | 053, 018 |  |
| OBL-055 | Corbeille, TRIM, défragmentation HDD | À faire | — | 2 | — | 050 |  |
| OBL-056 | Onglet Stockage et CLI | À faire | — | 3 | — | 054, 040 |  |

### P6 — Réseau

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-060 | Inventaire réseau et débit | À faire | — | 3,5 | — | 012 |  |
| OBL-061 | Contrôle du pare-feu | À faire | — | 3 | — | 007 |  |
| OBL-062 | Identification des applications | À faire | — | 2,5 | — | 061 |  |
| OBL-063 | Profils et liste d'immunité | À faire | — | 2,5 | — | 062 |  |
| OBL-064 | Filet de sécurité et restauration | À faire | — | 2 | — | 061, 017 |  |
| OBL-065 | Détection d'environnement | À faire | — | 1,5 | — | 061 |  |
| OBL-066 | Services réseau Windows | À faire | — | 2 | — | 036 |  |
| OBL-067 | Planification | À faire | — | 1,5 | — | 063, 025 |  |
| OBL-068 | Onglet Réseau et CLI | À faire | — | 3 | — | 063, 040 |  |
| OBL-069 | Coexistence avec ankou | À faire | — | 1,5 | — | 063 |  |

### P6b — Backends Linux et macOS

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-101 | Linux `mem` : lecture `/proc` | À faire | — | 3 | — | 100, 020 |  |
| OBL-102 | Linux `mem` : nettoyage et privilèges | À faire | — | 4 | — | 101, 014 |  |
| OBL-103 | Linux `proc` | À faire | — | 5 | — | 100, 030 |  |
| OBL-104 | Linux `disk` (openat2, fstrim) | À faire | — | 6 | — | 100, 053 |  |
| OBL-105 | Linux `net` (nftables par cgroup) | À faire | — | 7 | — | 100, 063 |  |
| OBL-106 | Linux `svc` (systemd, socket Unix) | À faire | — | 4 | — | 100, 018 |  |
| OBL-107 | macOS `mem` et `proc` | À faire | — | 7 | — | 100, 020 |  |
| OBL-108 | macOS `disk` | À faire | — | 5 | — | 100, 053 |  |
| OBL-109 | macOS `svc` (launchd) | À faire | — | 4 | — | 100, 018 |  |
### P7 — Durcissement et validation

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-080 | Revue de sécurité | À faire | — | 3 | — | P2–P6 |  |
| OBL-081 | Fuzzing étendu et chaos | À faire | — | 2,5 | — | 013, 053, 064 |  |
| OBL-082 | Matrice matérielle A–H | À faire | — | 3 | — | P2–P6 |  |
| OBL-083 | Profilage de l'empreinte | À faire | — | 2 | — | P4 |  |
| OBL-084 | Audit des dépendances, SBOM | À faire | — | 1 | — | P1 |  |
| OBL-085 | Soumissions antivirus, SmartScreen | À faire | — | 1 | — | 090 |  |

### P8 — Distribution et version 1.0

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-090 | Signature Authenticode | À faire | — | 2 | — | 004 |  |
| OBL-091 | Installateur, archive portable, `SHA256SUMS` | À faire | — | 3 | — | 090 |  |
| OBL-092 | Intégration au launcher | À faire | — | 1,5 | — | 091 |  |
| OBL-093 | Documentation utilisateur et runbooks | À faire | — | 2 | — | P2–P6 |  |
| OBL-110 | Installateurs des trois systèmes (MSI, deb/rpm, pkg) | À faire | — | 5 | — | 091 |  |
| OBL-094 | Version 1.0.0 | À faire | — | 1 | — | 080–093 |  |

### Tâches C (optionnelles)

| ID | Tâche | Statut | Resp. | Est. | Réel | Dép. | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OBL-038 | Programmes au démarrage, réversible | Reporté | — | 2,5 | — | 036 | Hors v1 |
| OBL-039 | Affinité et groupes > 64 processeurs | Reporté | — | 2 | — | 030 | Hors v1 |
| OBL-057 | Doublons par hachage | Reporté | — | 3 | — | 053 | Hors v1 |
| OBL-058 | Lecture directe de la MFT | Reporté | — | 4 | — | 051 | Hors v1 |
| OBL-070 | Mode WFP | Reporté | — | 6 | — | 063 | Hors v1 |

## 4. Registre des blocages

Toute tâche « Bloqué » a une ligne ici. Une ligne résolue reste visible (statut « Levé ») pour garder la mémoire du projet.

| Réf. | Tâche | Description du blocage | Depuis | Propriétaire | Action pour débloquer | Échéance | État |
| --- | --- | --- | --- | --- | --- | --- | --- |
| B-001 | *(exemple — à remplacer)* OBL-007 | Le pare-feu de la machine de test est géré par une stratégie de groupe | 2026-11-10 | — | Utiliser une machine virtuelle isolée | 2026-11-12 | Ouvert |

## 5. Décisions en attente

Questions ouvertes du cahier des charges, avec les tâches qu'elles bloquent si elles ne sont pas tranchées. Une décision prise devient un ADR.

| Réf. | Question | Tâches concernées | À trancher avant | Décision | État |
| --- | --- | --- | --- | --- | --- |
| Q1 | Licence : clean room ou GPL | OBL-001, OBL-010, tout le code | Début de P1 | — | À décider |
| Q2 | Réseau v1 : pare-feu Windows seul ou WFP | OBL-007, OBL-061, OBL-070 | Fin de P0 | — | À décider |
| Q3 | Disponibilité du nom « obliNiscor » | OBL-002, OBL-003 | Début de P0 | — | À décider |
| Q4 | Signature : certificat commercial ou programme pour projets ouverts | OBL-090 | Début de P7 | — | À décider |
| Q5 | Installateur Windows : MSI ou MSIX (Linux `.deb`/`.rpm`, macOS `.pkg` sont tranchés) | OBL-091, OBL-110 | Début de P8 | — | À décider |
| Q6 | Désactivation des services de mise à jour : permise ou pause limitée | OBL-066 | Début de P6 | — | À décider |
| Q7 | Extension Linux prévue ou non | OBL-012 | Fin de P1 | **Tranché le 31 octobre 2026 : multi-plateforme dès la v1 (ADR-004), disponibilités de la matrice 6.0** | Décidé |
| Q8 | Équipe : une personne ou plusieurs | Planning global | Début de P0 | — | À décider |

## 6. Journal de suivi

Une ligne par changement notable (statut, estimation, décision). Les plus récentes en haut.

| Date | Tâche | Changement | Auteur | Preuve |
| --- | --- | --- | --- | --- |
| 2026-11-01 | OBL-011 | Noyau complété : les modèles y sont rapatriés (ils vivaient dans `platform`, qui ne garde que les traits), noms de zones pour l'IPC et codes d'erreur stables. 30 tests. | — | PR « feat(core): own the models and the stable codes » |
| 2026-11-01 | OBL-004, OBL-012, OBL-013 | PR #4 fusionnée ; **CI verte sur les trois OS et la MSRV** (4 jobs). OBL-011 et OBL-100 restent *En cours* : le noyau et le socle sont partiels, les marquer « Fait » mentirait. | — | PR #4 |
| 2026-11-01 | OBL-004, OBL-011 à OBL-013, OBL-100 | Premier lot P1 mené **en équipe** : trois crates indépendants en parallèle (`proto`, `platform`, `sys`), 26 tests, `cargo xtask ci` vert. Revue du Lead : `encode` rend son erreur au lieu d`'un vecteur vide. | — | PR « feat: add the protocol, the platform traits and the system tables » |
| 2026-11-01 | OBL-003, OBL-015 | Contrôle i18n dans `cargo xtask ci` : complétude des deux langues et présence des clés produites par `Availability::label_key`. Première partie d'OBL-015. | — | PR « feat: check the translations » |
| 2026-11-01 | OBL-001 à OBL-003 | ADR-0001 (clean room), disponibilité du nom vérifiée, espace de travail et `cargo xtask ci` en état de marche | — | PR « feat: start the project » |
| 2026-10-31 | OBL-100 à OBL-110 | Révision multi-plateforme : ADR-004 révisé, matrice de disponibilité 6.0, phase P6b (45 j) et installateurs des trois systèmes ; total 202 j, 87 tâches | — | Cahier des charges v0.2 |
| 2026-10-31 | — | *(exemple)* Création du suivi ; toutes les tâches à « À faire » | — | Ce document |

## 7. Guide d'exécution des tâches

Chaque fiche donne : **Approche** (comment procéder dans le bon ordre), **Pièges** (ce qui fait perdre du temps ou de la sûreté) et **Preuve** (ce qui permet d'écrire « Fait »). Les points marqués *à valider* reposent sur des API Windows peu documentées : on les confirme par l'expérience avant de s'y fier.

### Principes transverses (valables pour toutes les tâches)

- **Tester avant de coder le cas nominal** : écrire d'abord le test de l'échec dangereux (suppression hors racine, règle non retirée, privilège non rendu).
- **Un seul risque nouveau par tâche** : si une tâche cumule une API inconnue, un nouveau format et une interface, la découper.
- **Mesurer plutôt que croire** : toute affirmation de performance ou de gain porte un chiffre, une machine et une date.
- **Écrire ce qu'on sait faire et ce qu'on ignore** dans le README et dans `docs/spikes/` ; l'honnêteté est une fonction du produit.
- **Journaliser avant d'agir** pour tout effet durable (suppression, règle réseau, changement de service).
- **Isoler l'OS** : tout appel système vit dans `obliniscor-windows`, derrière un trait de `obliniscor-platform`.

### Phase P0 — Cadrage et validation technique

**OBL-001 — ADR-001 : clean room et licence** **Approche.** Écrire l'ADR en trois temps : contexte (WinMemoryCleaner est sous GPL-3.0), décision, conséquences. Rendre la règle opérationnelle : les spécifications de comportement viennent de l'observation de l'application et de la documentation Microsoft ; le code est écrit sans recopier ni transposer ligne à ligne les fichiers `.cs`. Tenir un **journal des sources** (document consulté, date, module concerné). En solo, la séparation des rôles se fait dans le temps : spécifier d'abord, coder ensuite, sans rouvrir le code d'origine. **Pièges.** Croire qu'un renommage de variables suffit ; reproduire l'ordre exact des opérations d'un fichier source ; oublier d'ajouter une case « aucun code copié » au modèle de demande de tirage. **Preuve.** ADR fusionné, avis écrit ou décision assumée de passer en GPL, case ajoutée au modèle de PR.

**OBL-002 — Disponibilité du nom** **Approche.** Vérifier chaque nom de crate (`obliniscor-*`) sur crates.io, le nom d'utilisateur ou d'organisation sur GitHub, un nom de domaine éventuel, puis rechercher des marques proches. Pour une diffusion en Afrique francophone, consulter la base de l'OAPI en plus des registres habituels ; consulter un professionnel avant un dépôt de marque. Lire le nom à voix haute dans les deux langues. **Pièges.** Ne contrôler que le nom exact et oublier les variantes proches ; publier une crate vide sans nécessité (c'est une pratique courante de réservation, à décider en connaissance de cause). **Preuve.** Rapport d'une page avec date de vérification, liens et décision.

**OBL-003 — Dépôt, outillage, `xtask ci`** **Approche.** Partir des conventions d'ankou et du launcher : `deny.toml`, `rustfmt.toml`, `clippy.toml`, `rust-toolchain.toml` (version épinglée), `[workspace.lints]` avec `unsafe_code = "forbid"` par défaut et dérogation dans `obliniscor-windows` seulement, `undocumented_unsafe_blocks` en erreur, `.editorconfig` et `.gitattributes` (fins de ligne). Écrire `xtask ci` avant tout autre code : fmt, clippy, tests, deny. **Pièges.** La dépendance par chemin relatif vers `lpdnsoft-libs` : la documenter dans le README et faire vérifier sa présence par `xtask`. Laisser la dérogation `unsafe` se propager à d'autres crates. **Preuve.** `cargo xtask ci` vert sur un clone propre, en local et en CI.

**OBL-004 — CI Windows x64 et ARM64** **Approche.** Jobs séparés : lint, tests, deny, compilation ARM64 (cible `aarch64-pc-windows-msvc`). Mettre en cache les dépendances, épingler les actions par empreinte, donner au jeton le minimum de permissions. Marquer les tests exigeant des droits administrateur avec une fonctionnalité `admin-tests` (le runner est administrateur, pas forcément votre poste). **Pièges.** Tests qui passent en CI mais échouent en local faute de droits ; durée de pipeline qui dérive. **Preuve.** Pipeline vert, durée inférieure à 15 minutes, jobs lisibles.

**OBL-005 — Spike mémoire** **Approche.** Travailler dans un binaire d'exploration jetable, hors du code de production. Pour chaque zone : appeler, lire le code de retour, vérifier le résultat avec un outil indépendant (RAMMap, compteurs de performance), consigner la version de Windows. Tester aussi l'**absence de privilège** pour connaître l'erreur attendue. Répéter chaque mesure au moins 5 fois et retenir la médiane. Dériver les structures de la documentation ou de sources dont la licence est compatible : ne pas les reconstituer de mémoire. *À valider* : tout ce qui touche à la liste mémoire, au cache de fichiers, aux pages combinées et au cache du registre. **Pièges.** Mesurer sur machine inactive (gains trompeurs) ; confondre « mémoire libre » et « mémoire disponible » ; ignorer l'effet de Defender et de Windows Update pendant la mesure. **Preuve.** `docs/spikes/memoire.md` : tableau zone × build × privilège × résultat × coût observé.

**OBL-006 — Spike stockage** **Approche.** Ouvrir un handle sur un volume et interroger la propriété de pénalité de recherche pour distinguer HDD et SSD ; essayer sur disque USB, disque virtuel, volume RAID et espace de stockage. Déterminer la voie de ré-optimisation (TRIM) et les droits requis. Créer des jonctions, liens symboliques et liens physiques de test et observer le comportement de l'énumération et de la résolution du chemin final. **Pièges.** Écrire sur un volume réel pendant l'expérience ; conclure d'un seul disque à tous. **Preuve.** `docs/spikes/stockage.md` avec cas détectés, cas ambigus et comportements des points de reparse.

**OBL-007 — Spike pare-feu** **Approche.** Créer par COM une règle de blocage, la rattacher à un **groupe** (pour la retrouver en un coup), tester le ciblage par chemin, par nom de service et par identifiant de paquet d'application. Vérifier le résultat avec les outils du système. Tester : stratégie de groupe qui ignore les règles locales, pare-feu tiers, service du pare-feu arrêté. Les directions entrante et sortante sont des règles distinctes. **Pièges.** Règle sans profil explicite ; nom non unique qui empêche un retrait propre ; tester sur la machine de travail au lieu d'une machine virtuelle. **Preuve.** Scripts de création et de suppression idempotents ; zéro règle résiduelle ; `docs/spikes/pare-feu.md`.

**OBL-008 — Spike service et canal nommé** **Approche.** Mini-service avec `windows-service`, canal nommé en instance unique, clients distants refusés, ACL explicite (administrateurs et groupe dédié). Récupérer l'identité de l'appelant côté serveur. Écrire les tests d'abus : client non autorisé, second serveur qui tente de créer le même canal, message gigantesque, déconnexion brutale, client lent. **Pièges.** Faire confiance à une identité fournie par le client ; oublier de borner la taille des messages. **Preuve.** Démonstration exécutable et tests d'abus verts.

**OBL-009 — Protocole de mesure et machines A–H** **Approche.** Scripter le scénario 10.3 du cahier des charges : démarrage propre, peuplement du cache, relevés à +0 s, +30 s, +5 min, sortie CSV. Fixer l'environnement : mises à jour et analyses antivirus suspendues, même image de départ (instantané de machine virtuelle). Décrire chaque machine : processeur, RAM, disque, version de Windows. **Pièges.** Résultats non comparables d'un jour à l'autre ; changement d'environnement non noté. **Preuve.** Le scénario s'exécute de bout en bout deux fois, avec moins de 10 % d'écart entre les deux séries.

**OBL-010 — Revue de phase et go/no-go** **Approche.** Dresser la liste des hypothèses « à valider » du cahier des charges et leur donner un verdict : confirmée, modifiée ou abandonnée. Si une zone mémoire est inutilisable, la déclasser en W. Mettre à jour le cahier des charges (v0.2) et réviser les estimations de P1 à P6 à la lumière des spikes. **Pièges.** Passer à la suite en laissant des hypothèses ouvertes « pour plus tard ». **Preuve.** Cahier des charges v0.2 et décision go/no-go datée au journal.

### Phase P1 — Fondations

**OBL-011 — `obliniscor-core`** **Approche.** Types forts (taille en octets, identifiant de zone en énumération extensible), instantanés immuables, erreurs avec **codes stables** utilisables par l'IPC et la CLI. L'horloge est injectée par un trait pour tester le planificateur sans attendre. Aucune E/S. **Pièges.** Laisser fuiter des types Windows dans le noyau ; multiplier les `Option` à la place de vrais états. **Preuve.** Tests unitaires, `forbid(unsafe_code)` actif, couverture mesurée.

**OBL-012 — `obliniscor-platform`** **Approche.** Traits petits et orientés capacité (lire, nettoyer, énumérer), une doublure de test par trait avec des scénarios réglables : zone indisponible, accès refusé, lenteur. **Pièges.** Un « trait dieu » qui rend les doublures pénibles à écrire. **Preuve.** Le noyau est testé sur une machine sans Windows.

**OBL-013 — `obliniscor-proto`** **Approche.** Messages typés avec champ de version, cadrage par longueur (limite 1 Mio vérifiée **avant** toute allocation), refus des champs inconnus côté serveur, ajout de champs uniquement optionnels pour la compatibilité. Fichiers de référence (*golden files*) pour chaque message et fuzz du décodeur. **Pièges.** Allouer d'après une longueur annoncée avant de la valider ; casser la compatibilité sans le noter. **Preuve.** Fuzz sans plantage, test de compatibilité entre versions, limites vérifiées.

**OBL-014 — Garde RAII de privilèges** **Approche.** Une valeur dont la création active le privilège et dont la destruction restitue l'état antérieur. Attention : l'ajustement d'un jeton peut renvoyer un succès tout en n'attribuant pas tous les privilèges ; lire le code d'erreur associé après l'appel. **Pièges.** Oublier la restitution en cas d'erreur ou de panique ; activer un privilège pour toute la durée de vie du service. **Preuve.** Test qui provoque une panique et vérifie le retrait du privilège.

**OBL-015 — Intégration des libs et réglages v1** **Approche.** Lire les README des quatre libs, définir le schéma de réglages avec un numéro de version, écrire une migration de test dès la version 1 (même vide) pour prouver le mécanisme. Le service crée ses dossiers de données avec des droits d'écriture réservés au système et aux administrateurs, et de lecture pour les utilisateurs. **Pièges.** Dossier de données modifiable par un utilisateur standard ; journaux dans un dossier différent selon le processus sans l'avoir documenté. **Preuve.** Réglages relus après redémarrage ; remise aux réglages d'usine testée.

**OBL-016 — Historique JSONL** **Approche.** Écriture en fin de fichier, une ligne par événement, rotation par taille, lecture paginée. Tolérer une dernière ligne tronquée après un plantage (l'ignorer et le noter). **Pièges.** Réécrire tout le fichier à chaque ajout ; supposer qu'une ligne est toujours complète. **Preuve.** Test de troncature ; 100 000 entrées lues en moins de 200 ms.

**OBL-017 — Service Windows** **Approche.** Gestionnaire de contrôle qui ne bloque jamais (arrêt, alimentation, session) ; états rapportés avec indications d'attente ; actions de reprise sur échec ; démarrage automatique différé ; installation et désinstallation idempotentes. Réduire les privilèges requis du service à la liste utile (*à valider* : configuration des privilèges requis du service). **Pièges.** Travail long dans le gestionnaire de contrôle ; panique non rattrapée qui tue le service sans trace ; perte des opérations en cours à l'arrêt. **Preuve.** Installation sans redémarrage, reprise correcte après veille et hibernation, désinstallation propre.

**OBL-018 — Serveur et client IPC** **Approche.** Poignée de main de version, autorisation par catégorie de message (lecture, mutation, opération lourde, destructif), identité de l'appelant relevée par le service et inscrite au journal, délais d'attente et nombre de connexions bornés. Tests de clients concurrents. **Pièges.** Une lecture bloquante qui garde un client indéfiniment ; autorisation vérifiée à l'ouverture seulement et non à chaque message. **Preuve.** Client non autorisé refusé ; latence inférieure à 50 ms ; tests de concurrence verts.

**OBL-019 — CLI squelette** **Approche.** `clap` en mode dérivé, résultat sur la sortie standard (humain ou JSON versionné), messages d'état et journaux sur l'erreur standard, codes de sortie documentés. Aucune élévation de droits : la CLI passe toujours par le service. **Pièges.** Mélanger sortie machine et texte décoratif ; changer le format JSON sans incrémenter sa version. **Preuve.** Tests de sortie figés ; codes de sortie listés dans l'aide.

### Phase P2 — Module Mémoire

**OBL-020 — Lecture de l'état mémoire** **Approche.** Séparer la lecture « simple » (`GlobalMemoryStatusEx`, `GetPerformanceInfo`) de la lecture des listes de pages (requête noyau, *à valider*). Produire un `Snapshot` horodaté ; les valeurs dérivées (disponible = libre + standby, pourcentages) se calculent dans `obliniscor-core`, pas dans la couche Windows. Convertir les pages en octets avec la taille de page réelle du système. Comparer avec le Moniteur de ressources et RAMMap sur chaque machine de référence. **Pièges.** Confondre pages et octets ; oublier que la lecture n'est pas atomique (deux valeurs lues à quelques microsecondes d'écart peuvent être incohérentes : le documenter) ; supposer que les compteurs existent sur toutes les versions. **Preuve.** Écart inférieur à 2 % avec les outils de référence sur les machines A, B et C.

**OBL-021 — Défauts de page et pression disque** **Approche.** Une seule requête PDH ouverte à l'initialisation, échantillonnée à intervalle régulier, jamais recréée à chaque lecture. Ajouter les compteurs avec la variante qui prend les **noms anglais** (les noms de compteurs sont traduits selon la langue de Windows et casseraient un système en français). Les compteurs de type « taux » exigent deux échantillons pour produire une valeur. **Pièges.** Noms localisés ; première valeur nulle prise pour une mesure ; requête jamais fermée (fuite). **Preuve.** Valeurs cohérentes avec l'outil système sur une machine en français et une machine en anglais.

**OBL-022 — Nettoyage : working set, cache système, liste modifiée** **Approche.** Pour le working set : ouvrir chaque processus avec les droits minimaux, réduire, compter réussites et échecs, ignorer sans bruit les processus protégés. **Étaler** les appels (petite pause entre lots) pour ne pas provoquer une rafale de défauts de page. Annuler entre deux processus via un jeton partagé. Les autres zones passent par le privilège dédié, obtenu par la garde RAII (OBL-014). *À valider* : appels noyau pour le cache système et la liste modifiée. **Pièges.** Réduire son propre working set ; boucle serrée sur 500 processus ; traiter un refus d'accès comme une erreur fatale. **Preuve.** Gain mesuré et journalisé ; interface fluide pendant l'opération ; échec d'un processus sans effet sur les autres.

**OBL-023 — Nettoyage : standby, combinée, fichiers modifiés, registre** **Approche.** Une structure par zone, avec `is_supported()` (selon le **numéro de build**, lu de façon fiable : ne pas se fier à une API de version que le manifeste peut fausser) et `run()`. Traduire les codes de retour natifs en trois catégories : non supporté, accès refusé, autre. La purge *standby* s'exécute en dernier (voir 8.5 du cahier des charges). Pour le vidage du cache de fichiers modifiés, limiter aux volumes fixes et prévoir un garde-fou sur disque dur. **Pièges.** Considérer « non supporté » comme une erreur ; lancer le vidage de cache sur un disque dur sous charge ; coder les numéros de build en dur sans test. **Preuve.** Zone indisponible grisée avec explication, aucune erreur fatale, tableau de disponibilité par build joint aux notes.

**OBL-024 — Mesure avant/après et indicateur de coût** **Approche.** Une structure `Measurement` : états avant et après, delta par zone, durée, indicateur de coût. Laisser une brève stabilisation (environ 500 ms) avant la mesure « après ». Le coût compare les défauts de page durs des 30 s suivantes à la **moyenne des 60 s précédentes**. Marquer la mesure comme « bruitée » si l'activité disque de fond est élevée. **Pièges.** Attribuer à l'optimisation un effet dû à une autre activité ; afficher un gain sans le coût associé. **Preuve.** Entrée d'historique complète sur la machine A, avec cas bruité reconnu.

**OBL-025 — Politiques et garde-fous** **Approche.** Machine à états : *Repos* → *Armé* (condition vraie pendant N secondes) → *Exécution* → *Refroidissement*. Seuil d'armement X et seuil de réarmement Y > X (hystérésis). Plafond d'exécutions par heure. Garde-fous : état de notification de l'utilisateur (plein écran ou jeu), pression disque, batterie. Tester par **simulation d'horloge** sur 24 h de scénarios synthétiques. **Pièges.** Le « ping-pong » : purge → cache perdu → relectures disque → mémoire libre qui rechute → nouvelle purge. L'hystérésis, le refroidissement et le plafond doivent l'interdire ; le test dédié le prouve. **Preuve.** Aucune boucle de purge sur 24 h simulées, garde-fous vérifiés un par un.

**OBL-026 — Exclusions de processus** **Approche.** Identifier un processus par **chemin canonique** (et éventuellement signataire), pas par le seul nom, falsifiable. Normaliser la casse et les chemins courts (8.3). Les exclusions sont stockées dans les réglages machine. **Pièges.** Exclusion par nom trivialement contournée ; chemin non normalisé qui ne correspond plus. **Preuve.** Test : le processus exclu n'est jamais touché, même renommé en copie à un autre emplacement.

**OBL-027 — CLI mémoire** **Approche.** Sous-commandes `status`, `clean`, `history` ; `--dry-run` n'appelle aucune API noyau ; progression sur l'erreur standard ; JSON versionné sur la sortie standard ; code de sortie 3 si les droits manquent ; délai maximal configurable. **Pièges.** Attendre la fin sans délai maximal ; mélanger progression et résultat. **Preuve.** Le scénario S5 (script d'administration) s'exécute et son JSON est validé par un test.

**OBL-028 — Campagne de mesures** **Approche.** Trois profils × 5 répétitions × machines A, B, C, avec le protocole d'OBL-009. Pour chaque combinaison : gain, coût, temps de démarrage d'une application témoin. Publier les **données brutes** (CSV) avec l'analyse. Régler les valeurs par défaut sur les résultats, pas sur l'intuition. **Pièges.** Généraliser à partir d'une seule machine ; retenir le meilleur essai au lieu de la médiane. **Preuve.** `docs/mesures/memoire.md` publié, valeurs par défaut justifiées une à une.

### Phase P3 — Module Processus

**OBL-030 — Énumération des processus** **Approche.** Choisir la source (instantané système ou requête noyau de la liste des processus) **après mesure** du coût. Identifier un processus par le couple *PID + heure de création* (les PID sont réutilisés). CPU % = variation du temps processeur ÷ (variation de temps réel × nombre de processeurs logiques), donc deux échantillons. Vérifier la signature numérique en différé et la mettre en cache par chemin et date de modification : c'est coûteux. Ouvrir les processus avec le droit d'interrogation limité. **Pièges.** Signature vérifiée à chaque rafraîchissement ; PID recyclé pris pour le même processus ; accès refusé sur les processus protégés traité comme erreur. **Preuve.** 500 processus en moins de 100 ms (hors signature) ; CPU conforme à l'outil système.

**OBL-031 — Working set ciblé, priorité, règles persistantes** **Approche.** Refuser la classe « temps réel ». Une règle porte sur un chemin signé et une action ; elle est réappliquée au démarrage du processus. Préférer un abonnement aux événements de création de processus à un sondage (*à valider*), avec repli sur un balayage espacé. Mémoriser la priorité d'origine pour restaurer. **Pièges.** Règle réappliquée en boucle ; priorité élevée qui affame le reste du système. **Preuve.** Une règle survit au redémarrage du processus ; retour à l'état d'origine vérifié.

**OBL-032 — Liste de processus protégés** **Approche.** Deux niveaux : la **protection détectée par le système** (processus protégés) et une **liste codée en dur** (processus système essentiels, antivirus, `obliniscord`, `ankoud`). L'utilisateur peut ajouter des entrées, jamais en retirer du noyau. Identifier par chemin système et signature, pas par le seul nom. **Pièges.** Nom falsifiable (un faux `csrss.exe` ailleurs) ; liste modifiable par l'IPC. **Preuve.** Test : aucune action possible sur un processus protégé, y compris par message forgé.

**OBL-033 — Arrêt en deux temps** **Approche.** Demander la fermeture des fenêtres du processus, attendre un délai configurable, puis seulement proposer l'arrêt forcé avec **confirmation portée par le message**. Sans fenêtre, aller directement à l'avertissement avant l'arrêt forcé. Préciser si l'arrêt vise le processus seul ou son arbre. **Pièges.** Arrêter l'arbre sans le dire ; confirmation implicite côté interface mais absente du message. **Preuve.** Message d'arrêt forcé rejeté sans confirmation ; test d'un éditeur avec document non enregistré.

**OBL-034 — Mode efficacité et règles d'arrière-plan** **Approche.** Activer la limitation d'alimentation de processus (disponible selon la version de Windows). Détecter le premier plan par la fenêtre active et son arbre. Exclure explicitement les applications d'appel et de lecture audio. Prévoir un délai avant application et un retour immédiat au premier plan. **Pièges.** Ralentir un appel vidéo ou une lecture audio ; effet invisible faute de mesure. **Preuve.** Effet mesuré sur la machine C (cœurs P et E) ; liste d'exclusion testée.

**OBL-035 — Détection de dérive mémoire** **Approche.** Fenêtre glissante par processus, pente de la mémoire privée par régression, seuils de croissance minimale et de linéarité pour limiter les faux positifs ; ignorer les cinq premières minutes d'un processus ; dédoublonner les alertes. **Pièges.** Prendre une montée en charge légitime pour une fuite ; alertes répétées. **Preuve.** Fuite simulée détectée ; moins de 5 % de faux positifs sur un scénario normal d'une heure.

**OBL-036 — Services : état d'origine** **Approche.** Écrire l'état d'origine sur disque (de façon atomique) **avant** toute modification, puis modifier. Vérifier les services dépendants avant un arrêt. Liste noyau de services intouchables. **Pièges.** Modifier d'abord, mémoriser ensuite : un plantage perdrait l'état d'origine ; ignorer les dépendances. **Preuve.** Retour à l'état initial vérifié après arrêt brutal pendant la modification.

**OBL-037 — CLI processus** **Approche.** Filtres (nom, utilisateur, seuil de mémoire), tri, formats humain et JSON ; actions destructives via `--yes` explicite. **Pièges.** Une commande destructive silencieuse par défaut. **Preuve.** Scénarios scriptés et codes de sortie vérifiés.

### Phase P4 — Interface egui

**OBL-040 — Squelette egui et thème** **Approche.** État d'application central, messages reçus d'un fil client IPC par canal ; **aucun appel bloquant dans la fonction de dessin**. Demander un nouveau dessin seulement quand l'état change. Thème `lpdnsoft-ui`. Mode dégradé clair quand le service est absent. **Pièges.** Dessin continu qui consomme du processeur ; état dupliqué entre interface et service. **Preuve.** Ouverture en moins de 1 s ; CPU quasi nul au repos ; message clair sans service.

**OBL-041 — Tableau de bord et onglet Mémoire** **Approche.** Jauges, graphique court d'historique, bouton d'optimisation, résultat avant/après avec **coût** et explication. Désactiver une zone indisponible avec la raison. Afficher disponible et libre séparément, avec une aide courte. **Pièges.** Afficher « X Mo libérés » sans dire ce que cela coûte ; boutons actifs alors que le service est occupé. **Preuve.** Parcours S1 réalisable ; aucun gel supérieur à 100 ms.

**OBL-042 — Onglet Processus** **Approche.** Tableau **virtualisé** (dessiner seulement les lignes visibles), tri stable, arbre repliable, sélection conservée au rafraîchissement, actions avec fenêtre de confirmation. Rafraîchir à 1 s. **Pièges.** Retrier à chaque trame ; sélection perdue ; confirmation absente pour un arrêt forcé. **Preuve.** 500 lignes fluides ; confirmation vérifiée pour toute action destructive.

**OBL-043 — Zone de notification, notifications, raccourci global** **Approche.** Choisir des crates maintenues et de licence compatible. L'icône ne se redessine qu'au changement de palier de couleur. Le raccourci global signale clairement un conflit au lieu d'échouer en silence. Les notifications indiquent raison, gain et coût. **Pièges.** Minuteur d'icône permanent ; icône floue en haute densité de pixels. **Preuve.** Conflit de raccourci signalé ; icône nette à 100 et 200 % ; aucun sondage actif.

**OBL-044 — Mode compact, réglages, journal** **Approche.** Les réglages machine passent par le message `UpdateSettings` du service, jamais par une écriture directe depuis l'interface. Validation côté service. Journal filtrable par niveau et par module. **Pièges.** Interface qui écrit des fichiers machine ; réglage non validé qui casse le service. **Preuve.** Réglages persistés après redémarrage ; valeur invalide refusée avec message clair.

**OBL-045 — Français et anglais complets** **Approche.** Toutes les chaînes passent par `t!`. Test automatique : mêmes clés dans les deux langues. **Pseudo-localisation** (chaînes allongées de 30 %) pour repérer les troncatures avant qu'elles n'apparaissent. **Pièges.** Chaîne en dur oubliée ; texte tronqué dans une langue plus longue. **Preuve.** Compilation en échec si une chaîne manque ; captures sans troncature.

**OBL-046 — Accessibilité et DPI** **Approche.** Ordre de tabulation logique, focus visible, contraste AA vérifié avec un outil, tests à 100, 150, 200 et 250 % de mise à l'échelle. Vérifier l'état de la prise en charge des lecteurs d'écran par egui (*à valider*) et le documenter. **Pièges.** Information portée par la seule couleur ; contrôles trop petits au tactile. **Preuve.** Liste de contrôle NFR-A11Y-1 signée.

### Phase P5 — Module Stockage

**OBL-050 — Inventaire des volumes** **Approche.** Énumérer les volumes (y compris ceux sans lettre), lire système de fichiers, capacité et espace libre, puis le type de média par la propriété de pénalité de recherche. Ne nettoyer que les volumes fixes locaux. Rendre un état **« inconnu »** plutôt que deviner (RAID, disques virtuels). Désactiver les erreurs bloquantes du système sur lecteur absent et fixer des délais. **Pièges.** Bloquer l'interface sur un lecteur réseau déconnecté ou un lecteur optique vide ; conclure « SSD » faute de pénalité détectée sur un volume virtuel. **Preuve.** SSD, NVMe et HDD correctement identifiés sur A et B ; cas ambigus documentés.

**OBL-051 — Analyse d'espace** **Approche.** Parcours parallèle à nombre de fils borné, sans suivre les points de reparse, agrégation en flux par dossier (jamais toute l'arborescence en mémoire). Distinguer taille logique et taille sur disque (fichiers compressés ou épars). Compter à part les éléments inaccessibles. Annulation par jeton, progression régulière. **Pièges.** Explosion de mémoire sur des millions de fichiers ; liens physiques comptés deux fois (le noter dans l'interface). **Preuve.** 1 To analysé sans gel ; annulation en moins d'une seconde.

**OBL-052 — Catalogue de cibles** **Approche.** Schéma TOML versionné : `id`, racine exprimée par **dossier connu** (jamais un chemin en dur), motif, âge minimal, risque, droits requis, service à arrêter, texte explicatif français et anglais. À la charge : racine dans une liste blanche, motif sans remontée de dossier, aucune racine de lecteur, signature ou empreinte vérifiée. Documenter la **source de chaque cible** (documentation Microsoft ou observation). **Pièges.** Cibles trop larges (`*`) ; dossiers dépendant de la langue ou de la version ; cible ajoutée sans relecture. **Preuve.** Catalogue invalide refusé au chargement ; chaque cible relue par une seconde personne ou une seconde passe.

**OBL-053 — Suppression sûre** *(tâche la plus dangereuse : une seule en cours, revue renforcée)* **Approche.** Écrire d'abord la suite « arborescences pièges » (tests rouges) : liens symboliques, jonctions, liens physiques, chemins longs, noms réservés (CON, NUL), flux alternatifs, nom à espace final, dossier remplacé par un lien en cours de route. Ensuite le code : ouvrir la racine par **handle**, obtenir son chemin final et le comparer à la racine déclarée ; énumérer sans suivre les points de reparse ; supprimer un lien lui-même, jamais sa cible ; pour chaque fichier, revérifier le chemin final **à partir du handle** avant de supprimer ; ignorer ce qui est ouvert par ailleurs ; préfixer les chemins longs. **Pièges.** Canonicaliser par simple manipulation de texte ; vérifier le chemin puis supprimer par nom (course) ; traiter l'attribut lecture seule en forçant sans vérifier. **Preuve.** La suite pièges passe ; aucune suppression hors racine ; revue indépendante consignée.

**OBL-054 — Aperçu, jeton, journal** **Approche.** L'aperçu produit la liste et une **empreinte** (chemins, tailles, dates). Le jeton est à usage unique et à durée limitée ; à l'exécution, chaque élément est revalidé ; si trop d'éléments ont changé, refus et nouvel aperçu. **Journaliser l'intention avant** de supprimer, puis le résultat. **Pièges.** Jeton rejouable ; journal écrit après coup (un plantage efface la trace). **Preuve.** Exécution impossible sans aperçu ; journal complet même après arrêt brutal.

**OBL-055 — Corbeille, TRIM, défragmentation HDD** **Approche.** La corbeille est **propre à chaque utilisateur** : le service (compte système) ne peut pas y envoyer les fichiers d'un utilisateur ; l'envoi à la corbeille passe donc par le client non privilégié. Pour la ré-optimisation, passer par l'API d'optimisation de volume (*à valider*). Règle dure : refuser toute défragmentation si le média est SSD **ou inconnu**. **Pièges.** Croire que le service peut utiliser la corbeille d'un utilisateur ; défragmenter un SSD mal détecté. **Preuve.** Test : défragmentation refusée sur SSD et sur média inconnu.

**OBL-056 — Onglet Stockage et CLI** **Approche.** Volumes, analyse, cibles groupées par risque avec case à cocher, aperçu obligatoire, progression, résultat. Cibles de risque élevé **masquées** par défaut. CLI `disk scan` et `disk clean --dry-run`. **Pièges.** Tout pré-cocher ; masquer le coût (redémarrage nécessaire, service arrêté). **Preuve.** Parcours S4 réalisable ; sortie JSON stable.

### Phase P6 — Module Réseau

**OBL-060 — Inventaire réseau et débit** **Approche.** Connexions TCP et UDP avec PID par les tables étendues du système ; débit par processus par le suivi d'événements noyau du réseau (session à droits élevés, donc dans le service). Agréger dans le rappel (octets par PID par seconde) sans journaliser chaque événement. Pas de résolution DNS inverse bloquante. **Pièges.** Volume d'événements à haut débit qui sature le processeur ; PID recyclé. **Preuve.** Débit conforme à l'outil système à ± 5 % ; coût processeur inférieur à 0,5 %.

**OBL-061 — Contrôle du pare-feu** **Approche.** COM `INetFwPolicy2` dans un fil dédié à l'apartment correctement initialisé. Noms de règles : préfixe `obliNiscor:` + identifiant stable. Création **idempotente** (même nom = mise à jour). Lecture de l'état réel pour le comparer à l'état attendu. Suppression par groupe. **Pièges.** COM utilisé depuis un fil non initialisé ; collision de noms ; règle créée mais non activée ; profil non précisé. **Preuve.** Création puis retrait : zéro règle résiduelle, vérifié par un second outil.

**OBL-062 — Identification des applications** **Approche.** Programme classique : chemin normalisé. Service : **nom de service** (jamais `svchost.exe` seul). Application du Store : identifiant de paquet, car son dossier change à chaque mise à jour. **Pièges.** Bloquer tous les services d'un même hôte ; règle devenue caduque après mise à jour. **Preuve.** Une règle sur un service ne bloque pas les autres services du même hôte (test).

**OBL-063 — Profils et liste d'immunité** **Approche.** Un profil est un ensemble déclaratif de règles. **Point clé : dans le pare-feu de Windows, une règle de blocage l'emporte sur une règle d'autorisation.** L'immunité ne se garantit donc pas en ajoutant des règles « autoriser », mais en **ne générant jamais** de blocage qui couvre loopback, DNS, DHCP, ankou et obliNiscor (filtre à la génération, test à chaque profil). Pour « Strict », décider par ADR s'il s'agit de blocages ciblés sur toutes les applications de l'inventaire hors liste blanche (recommandé en v1) ou d'un changement de l'action sortante par défaut du profil (effet global, plus risqué). **Pièges.** Croire qu'une règle « autoriser » sauvera DNS d'un blocage large ; modifier la politique par défaut sans restauration garantie. **Preuve.** Sous le profil Strict, DNS et tunnel fonctionnent ; le test d'immunité est automatisé.

**OBL-064 — Filet de sécurité et restauration** **Approche.** Avant d'appliquer un profil large, écrire le **plan de restauration** sur disque, puis lancer un minuteur de 60 s **dans le service** (jamais dans l'interface : si elle plante, l'annulation doit quand même avoir lieu). La confirmation annule le minuteur ; `net restore` supprime le groupe et rétablit les services. Au démarrage du service : réconcilier l'état réel et l'état attendu ; si un profil était en attente de confirmation, restaurer. **Pièges.** Minuteur côté interface ; restauration qui dépend d'un fichier non écrit de façon atomique. **Preuve.** Arrêt du service pendant l'application : état restauré au redémarrage (test de chaos).

**OBL-065 — Détection d'environnement** **Approche.** Lire l'état d'activation de chaque profil, détecter une politique qui ignore les règles locales et un produit pare-feu tiers déclaré au centre de sécurité (*à valider*). Le service du pare-feu arrêté est un cas à part. L'interface explique la situation et **ne promet aucun blocage** quand il n'est pas garanti. **Pièges.** Afficher « bloqué » alors que la règle est ignorée. **Preuve.** Message explicite dans chaque cas simulé.

**OBL-066 — Services réseau de Windows** **Approche.** Cibler d'abord les services de mise à jour, d'optimisation de livraison et de télémétrie ; **vérifier leurs noms et comportements sur chaque build** (*à valider*). Arrêter et désactiver avec état d'origine mémorisé (voir OBL-036). Détecter une relance par déclencheur ou tâche planifiée et la signaler. Par défaut : **pause limitée** (24 h) avec rappel visible, pas désactivation indéfinie. **Pièges.** Laisser des mises à jour de sécurité coupées sans rappel ; croire qu'un service désactivé le reste. **Preuve.** Rétablissement automatique à l'échéance ; relance détectée et signalée.

**OBL-067 — Planification** **Approche.** Plages horaires en heure locale avec gestion des changements d'heure et de fuseau ; session de jeu détectée par plein écran ou liste d'applications. Réappliquer l'état attendu à la reprise de veille. Journaliser chaque début et chaque fin. **Pièges.** Plage qui chevauche minuit ; règle laissée active après une veille longue. **Preuve.** Règles appliquées et levées aux heures prévues, y compris après veille.

**OBL-068 — Onglet Réseau et CLI** **Approche.** Liste d'applications avec débit en direct, interrupteur par application, choix de profil, bouton **« Tout rétablir » toujours visible**. État de chaque règle : appliquée, en attente, divergente. Avertissement clair sur les effets et confirmation pour les profils larges. CLI `net apps|rules|block|allow|profile|restore`. **Pièges.** Cacher le bouton de rétablissement dans un menu ; états ambigus. **Preuve.** Parcours S3 réalisable ; confirmation vérifiée.

**OBL-069 — Coexistence avec ankou** **Approche.** Appliquer chaque profil pendant qu'un tunnel WireGuard est actif et vérifier la poignée de main et le trafic. Si le client ankou n'existe pas encore sous Windows, tester avec un tunnel WireGuard standard et le noter. Répéter avec un pare-feu tiers (machine F). **Pièges.** Tester seulement sans tunnel ; oublier le trafic UDP sortant du tunnel. **Preuve.** Tunnel intact sous tous les profils ; résultats consignés.

### Phase P6b — Backends Linux et macOS

**Principe commun.** Ces tâches n''ajoutent aucune fonctionnalité : elles implémentent, pour un autre système, les traits déjà figés en P1. Le test de référence est donc **le même** que sous Windows, relancé sur la machine G ou H ; une différence de comportement se tranche dans le noyau partagé, jamais dans le backend. La matrice 6.0 dit ce qui est complet, partiel ou indisponible : une fonction indisponible se **déclare**, elle ne s''approxime pas.

**OBL-101 — Linux `mem`, lecture** **Approche.** Séparer la lecture de fichier (derrière un trait) du parseur (pur, couvert par des fixtures de plusieurs noyaux). Convertir les pages en octets avec la taille réelle du système. **Pièges.** Prendre `MemFree` pour la mémoire disponible ; oublier `SReclaimable` ; supposer que toutes les clés existent. **Preuve.** Écart < 2 % avec `free -b` sur la machine G ; fixtures de trois versions de noyau.

**OBL-102 — Linux `mem`, nettoyage** **Approche.** `/proc/sys/vm/drop_caches` exige root : passer par le service, jamais par la GUI, et écrire d''abord le texte d''avertissement, ensuite l''action. Désactivé par défaut. **Pièges.** Présenter `drop_caches` comme un entretien recommandé — c''est un outil de diagnostic ; oublier de rendre le privilège. **Preuve.** Désactivé par défaut, avertissement visible, test de panique vérifiant le retrait.

**OBL-103 — Linux `proc`** **Approche.** `/proc/<pid>/stat` et `status` ; identifier un processus par PID **et** heure de démarrage ; pondération par cgroups v2, `nice` pour la priorité fine ; arrêt en deux temps (SIGTERM, attente, SIGKILL) au même contrat que Windows. **Pièges.** Confondre RSS et mémoire privée ; toucher aux processus noyau ; tuer un processus d''un autre utilisateur sans le dire. **Preuve.** 500 processus en < 100 ms ; SIGTERM respecté ; protections intactes.

**OBL-104 — Linux `disk`** **Approche.** `openat2` avec `RESOLVE_BENEATH|RESOLVE_NO_SYMLINKS` pour garantir la racine ; `unlinkat` relatif au descripteur ; `fstrim` par volume monté ; défragmentation refusée par construction. **Pièges.** Canonicaliser puis supprimer par nom (rouvre la course) ; oublier que les montages peuvent changer pendant le parcours. **Preuve.** Suite « arborescences pièges » verte ; `fstrim` seulement sur volumes montés.

**OBL-105 — Linux `net`** **Approche.** nftables avec une table dédiée et des ensembles par cgroup, en reprenant la structure du kill switch d''aNkou ; l''immunité se fait en **n''émettant jamais** de règle couvrant loopback, DNS, DHCP et le tunnel. **Pièges.** Casser le tunnel WireGuard ; oublier que cgroups v2 doit être monté ; règle sans `meta nfproto` qui attrape IPv6 par surprise. **Preuve.** Tunnel intact sous tous les profils ; `net restore` : zéro règle ; test de coupure du service.

**OBL-106 — Linux `svc`** **Approche.** Unité systemd durcie (SEC-13), socket Unix en 0660 sur le groupe `obliniscor`, `SO_PEERCRED` pour l''identité. **Pièges.** Socket accessible à tous ; unité qui redémarre en boucle ; `RestartSec` oublié. **Preuve.** Client non autorisé refusé ; arrêt propre ; désinstallation sans résidu.

**OBL-107 — macOS `mem` et `proc`** **Approche.** `host_statistics64` pour la mémoire, `libproc` pour les processus, `nice` pour la priorité. Déclarer le nettoyage mémoire et la réduction de working set **indisponibles** plutôt que de les approximer. **Pièges.** Demander une Network Extension ou un droit Apple non nécessaire ; simuler un nettoyage sans effet. **Preuve.** Mesures conformes à `vm_stat` et `ps` ; l''interface annonce l''indisponibilité.

**OBL-108 — macOS `disk`** **Approche.** `statfs` pour les volumes, `diskutil` pour la santé, suppression sûre par `openat` + `O_NOFOLLOW`, aucune défragmentation. **Pièges.** Confondre volume APFS et volume physique ; oublier les volumes montés sous `/Volumes`. **Preuve.** Suite pièges verte ; volumes APFS correctement listés.

**OBL-109 — macOS `svc`** **Approche.** `launchd` daemon, socket Unix, `LOCAL_PEERPID` ; l''installation demande une approbation de l''utilisateur, et l''interface doit le dire. **Pièges.** Promettre une installation silencieuse ; oublier que l''approbation peut être refusée. **Preuve.** Client non autorisé refusé ; désinstallation propre ; refus d''approbation annoncé clairement.
### Phase P7 — Durcissement et validation

**OBL-080 — Revue de sécurité** **Approche.** Pour chaque menace de la section 9 du cahier des charges, écrire un test ou une preuve. Relire chaque bloc `unsafe` avec une grille fixe : invariant, durée de vie des pointeurs, taille des tampons (appels « demander la taille puis allouer » avec nouvelle tentative), handle fermé par un type RAII, chaînes UTF-16 terminées par NUL. Contrôler les droits réels du canal et des dossiers avec les outils du système, puis **jouer l'attaquant** : depuis un compte standard, tenter les messages interdits. **Pièges.** Revue faite par l'auteur seul ; droits vérifiés sur le papier et non sur la machine. **Preuve.** Rapport sans écart critique ouvert, relecteur indépendant si possible.

**OBL-081 — Fuzzing étendu et chaos** **Approche.** Cibles de fuzz : décodeur IPC, analyseur du catalogue, canonicalisation de chemins ; corpus de départ = fichiers de référence. Pour le chaos : instrumenter les **points d'arrêt** des séquences critiques (fonctionnalité `chaos`) et tuer le processus à chacun, puis vérifier les invariants : aucune règle orpheline, journal cohérent, état restaurable. **Pièges.** Tuer le processus seulement à un endroit « commode » ; invariants non écrits. **Preuve.** 24 h de fuzz sans plantage ; tous les points d'arrêt rejoués avec invariants verts.

**OBL-082 — Matrice matérielle A à H** **Approche.** Dérouler la liste de contrôle complète sur chaque machine, en **français et en anglais** (les noms de compteurs et de dossiers diffèrent), à plusieurs échelles d'affichage. Corriger ou documenter chaque écart. **Pièges.** Ne tester que la dernière version de Windows ; ignorer la langue du système. **Preuve.** Matrice verte, écarts acceptés consignés avec raison.

**OBL-083 — Profilage de l'empreinte** **Approche.** Enregistrer avec les outils de trace du système (WPR et WPA) : réveils par seconde du service au repos, mémoire privée, handles, fils. Faire un **test d'endurance de 24 h** : la courbe mémoire doit être plate. Toujours mesurer le binaire de release. **Pièges.** Mesurer en mode debug ; ignorer les réveils périodiques qui empêchent la mise en veille du processeur. **Preuve.** Cibles NFR-PERF-1 à 3 atteintes, courbe d'endurance jointe.

**OBL-084 — Audit des dépendances et SBOM** **Approche.** `cargo deny check`, audit des avis de sécurité, liste des doublons de versions, génération d'une nomenclature logicielle (outil à choisir), build avec `--locked`. Décider d'une politique de mise à jour (mensuelle, plus immédiate pour un avis de sécurité). **Pièges.** Dépendance transitive à licence incompatible découverte tard. **Preuve.** Aucun avis ouvert ; SBOM publiée avec la release.

**OBL-085 — Soumissions antivirus et SmartScreen** **Approche.** Soumettre les binaires **signés** aux principaux éditeurs et au service d'analyse de Microsoft, avec une description de ce que fait le service. Ne pas supposer que la signature supprime tout avertissement dès le premier jour : la réputation se construit avec le temps et les téléchargements. **Pièges.** Soumettre des binaires non signés ; publier sans plan de réponse aux faux positifs. **Preuve.** Aucun blocage sur les produits principaux, ou dossier de contestation déposé et suivi.

### Phase P8 — Distribution et version 1.0

**OBL-090 — Signature Authenticode** **Approche.** Deux voies : certificat commercial, ou programme de signature pour projets ouverts (WinMemoryCleaner en utilise un, d'après son README ; à comparer). Signer dans la CI avec des secrets protégés, **horodater** chaque signature pour qu'elle reste valide après l'expiration du certificat, signer tous les binaires et l'installateur. **Pièges.** Oublier l'horodatage ; clé privée dans le dépôt ; binaire signé puis modifié. **Preuve.** Signature valide et horodatée, vérifiable avec `Get-AuthenticodeSignature` et `signtool verify`.

**OBL-091 — Installateur, archive portable, `SHA256SUMS`** **Approche.** Trancher MSI ou MSIX (Q5). L'installateur crée le service, le groupe local `obliniscor` et les droits des dossiers ; la désinstallation arrête le service, exécute la **restauration réseau**, retire service et tâches, et propose de conserver ou supprimer les données. Écrire un **script d'audit de résidus** (service, règles de pare-feu, tâches, clés de registre, dossiers) et l'exécuter après chaque test d'installation sur machine virtuelle propre. **Pièges.** Désinstallation qui laisse des règles de blocage : l'utilisateur resterait coupé du réseau. C'est le défaut le plus grave de cette phase. **Preuve.** Après désinstallation : 0 service, 0 règle, 0 tâche, 0 résidu hors données choisies.

**OBL-092 — Intégration au launcher** **Approche.** Suivre le format décrit dans le dépôt du launcher pour l'entrée de l'application ; faire détecter la version installée par la clé de désinstallation ; tester de bout en bout l'installation, la mise à jour et la vérification SHA-256 par le launcher. **Pièges.** Version affichée différente de la version installée ; mise à jour qui interrompt un service en cours d'opération. **Preuve.** Le launcher installe et met à jour obliNiscor sur une machine propre.

**OBL-093 — Documentation utilisateur et runbooks** **Approche.** Écrire pour quelqu'un qui n'a **pas lu** ce document. Runbooks prioritaires : « Je n'ai plus Internet » (restauration réseau), « Le service ne démarre pas », « Désinstaller proprement ». Publier `api-utilisees.md` et `securite.md`. Faire relire par une personne extérieure en lui demandant de suivre les pas sans aide. **Pièges.** Documentation qui suppose le vocabulaire du projet. **Preuve.** Un relecteur extérieur réussit les trois runbooks sans aide.

**OBL-094 — Version 1.0.0** **Approche.** Checklist de release : tous les critères de la section 16 du cahier des charges, étiquette, journal des modifications, signatures, empreintes, launcher. Préparer un **plan de retrait** (comment retirer une release, quoi annoncer) et surveiller les retours pendant la première semaine. **Pièges.** Publier un vendredi soir ; ne pas avoir de plan si un défaut grave apparaît. **Preuve.** Release publiée, checklist signée, plan de retrait écrit.

### Tâches C (si reprises)

- **OBL-038 — Programmes au démarrage.** Lister les clés de démarrage, dossiers et tâches planifiées ; désactiver en **déplaçant** vers un espace d'attente réversible, jamais en supprimant ; afficher l'éditeur signé.
- **OBL-039 — Affinité et groupes.** Utiliser les ensembles de processeurs et les groupes ; tester sur une machine à plus de 64 processeurs logiques (virtuelle si besoin) ; ne jamais fixer une affinité qui exclurait tous les processeurs.
- **OBL-057 — Doublons.** Regrouper par taille, puis empreinte partielle, puis empreinte complète ; lecture seule ; aucune suppression automatique ; choisir une crate de hachage maintenue.
- **OBL-058 — Lecture directe de la MFT.** Droits élevés, format NTFS à respecter strictement, repli sur le parcours standard ; analyse en lecture seule uniquement.
- **OBL-070 — Mode WFP.** Fournisseur et sous-couche dédiés (GUID fixés), poids documentés, **filtres dynamiques** qui disparaissent à la fermeture de la session du moteur (garde-fou), tests de coexistence avec le pare-feu ; ADR avant de commencer.

## 8. Annexes

### A. Modèle de fiche de tâche (à copier avant de démarrer)

| Champ | Contenu |
| --- | --- |
| ID et titre | `OBL-nnn` — … |
| Objectif en une phrase | Ce qui sera vrai à la fin |
| Exigences couvertes | `FR-…`, `NFR-…`, `SEC-…` |
| Dépendances vérifiées | Liste, avec statut « Fait » confirmé |
| Risque principal | Le seul risque nouveau de la tâche |
| Plan en 3 à 6 étapes | Chacune rejouable et vérifiable |
| Tests à écrire d'abord | Le cas dangereux en premier |
| Preuve attendue | Mesure, capture, journal, commande |
| Documentation touchée | ADR, runbook, `api-utilisees.md`, aide |
| Estimation / réel | j / j, et cause d'un écart supérieur à 1,5 × |

### B. Modèle de rapport de spike (`docs/spikes/…`)

1. **Question** posée, en une phrase.
2. **Environnement** : machine, version et build de Windows, droits, outils.
3. **Méthode** : ce qui a été exécuté, de façon rejouable.
4. **Résultats** bruts (tableau, mesures, codes de retour).
5. **Interprétation** : ce qui est établi, ce qui reste incertain.
6. **Décision** : confirmée, modifiée ou abandonnée, et l'impact sur le cahier des charges.
7. **Sources** consultées (clean room) avec dates.

### C. Liste de relecture d'une demande de tirage

- [ ] La demande répond à **une** tâche et la cite.
- [ ] Les tests de l'échec dangereux existent et sont écrits en premier.
- [ ] Pas de `unwrap` ni de panique possible sur une entrée externe.
- [ ] Toute ressource système est libérée par un type RAII, y compris en cas d'erreur.
- [ ] Chaque `unsafe` est isolé dans `obliniscor-windows` avec `SAFETY:` précis.
- [ ] Toute entrée IPC est validée (type, bornes, chemin canonique).
- [ ] Tout effet durable est journalisé **avant** d'être exécuté.
- [ ] Aucune chaîne en dur ; français et anglais fournis.
- [ ] Aucun code issu d'un projet de licence incompatible (clean room).
- [ ] La documentation et le tableau de suivi sont mis à jour.

### D. Quand on est bloqué : l'arbre de décision

1. **15 minutes** de recherche ciblée (documentation, `docs/spikes/`, journal du projet).
2. **Réduire** : un exemple minimal qui reproduit le problème, seul, sans le reste du code.
3. **Changer de point de vue** : essayer l'appel avec un outil indépendant pour savoir si le défaut vient du code ou du système.
4. **Découper** : isoler la partie qui avance de celle qui bloque.
5. **Contourner** avec une doublure ou un stub pour poursuivre la suite, en le notant.
6. **Demander** : forum ou ticket, avec l'exemple minimal, la version de Windows et les codes d'erreur.
7. **Consigner** au registre des blocages (section 4) avec propriétaire et échéance.
8. **Décider** au bout de 3 jours : reporter (W ou C), changer d'approche ou réduire le périmètre. Une décision écrite vaut mieux qu'un blocage silencieux.

### E. Indicateurs de santé du projet (à relever à la revue hebdomadaire)

| Indicateur | Vert | Orange | Rouge |
| --- | --- | --- | --- |
| Écart réel / estimé (moyenne glissante) | ≤ 1,2 | 1,2 à 1,5 | > 1,5 |
| Âge du plus ancien blocage | ≤ 2 jours | 3 à 5 jours | > 5 jours |
| Tâches « En cours » par personne | ≤ 2 | 3 | ≥ 4 |
| Durée de `cargo xtask ci` | < 10 min | 10 à 15 min | > 15 min |
| Tests instables (qui échouent au hasard) | 0 | 1 à 2 | ≥ 3 |
| Blocs `unsafe` sans relecture | 0 | 1 à 3 | > 3 |
| Hypothèses « à valider » encore ouvertes après P0 | 0 | 1 à 2 | > 2 |
| Tâches « En revue » depuis plus de 48 h | 0 | 1 | ≥ 2 |

Un indicateur rouge déclenche une action écrite au journal (section 6) dans la même semaine.

### F. Avant d'écrire « Fait » : contrôles spécifiques par type de tâche

| Type | Contrôles supplémentaires |
| --- | --- |
| Appel système | Test sans privilège, test sur deux versions de Windows, ligne ajoutée à `api-utilisees.md`, comportement documenté en cas de « non supporté » |
| Suppression de fichiers | Suite « pièges » exécutée, aperçu et jeton vérifiés, journal lu après un arrêt brutal |
| Réseau | Restauration testée, immunité vérifiée, essai sous pare-feu tiers, état lisible dans l'interface |
| Service et IPC | Client non autorisé refusé, message trop gros refusé, reprise après veille |
| Interface | Navigation clavier, échelle 100 et 200 %, deux langues, aucun gel supérieur à 100 ms |
| Mesure et performance | Machine et version notées, au moins 5 répétitions, médiane, données brutes archivées |
