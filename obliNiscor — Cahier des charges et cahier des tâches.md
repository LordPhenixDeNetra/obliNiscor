# obliNiscor — Cahier des charges et cahier des tâches

*Suite LPDNSoft · Document de référence v0.2 · 31 octobre 2026 · Statut : brouillon pour validation*<br>*Révision du 31 octobre 2026 : ouverture multi-plateforme (Windows, Linux, macOS).*

## 0. Comment lire ce document

Ce document réunit en un seul endroit la spécification d'obliNiscor (le « quoi » et le « pourquoi ») et son plan de réalisation (le « comment » et le « quand »). Il suit les conventions de la suite LPDNSoft : décisions tracées en ADR, contrôles par `cargo xtask ci`, honnêteté sur ce qui n'existe pas encore.

- **Identifiants d'exigences** : `FR-<domaine>-<n>` pour les fonctions, `NFR-<domaine>-<n>` pour les qualités, `SEC-<n>` pour la sécurité.
- **Priorités (MoSCoW)** : **M** = indispensable à la v1, **S** = souhaitable, **C** = possible, **W** = explicitement exclu de la v1.
- **« À valider »** : point reposant sur une API Windows peu ou pas documentée, ou sur une hypothèse matérielle. Il doit être confirmé par une expérience mesurée avant d'être figé (voir le jalon P0).
- **Cahier des tâches** : section 14. Chaque tâche a un identifiant `OBL-nnn`, des dépendances, une estimation en jours-personne et un critère d'acceptation vérifiable.

## 1. Synthèse

**obliNiscor** (du latin *obliviscor*, « j'oublie ») est une application de bureau écrite en Rust, pour **Windows, Linux et macOS**, destinée à la suite LPDNSoft. Elle regroupe quatre fonctions d'entretien du poste, pilotées par un service système et une interface légère :

1. **Mémoire** : mesurer la mémoire et récupérer sur demande les zones de cache et de working set, avec preuve chiffrée avant/après.
2. **Processus** : observer, régler (priorité, mode efficacité, working set) et arrêter proprement des processus, avec une liste de protections.
3. **Stockage** : inventorier les volumes, analyser l'espace et nettoyer des cibles connues, en simulation par défaut.
4. **Réseau** : empêcher des applications choisies d'envoyer ou de télécharger, de façon réversible, via le pare-feu de Windows.

Le projet reprend l'esprit de WinMemoryCleaner (utilitaire C# sous GPL-3.0) mais est une **réécriture indépendante** (voir ADR-001), et s'appuie sur les bibliothèques `lpdnsoft-libs`.

**Disponibilité par système** : chaque fonction n'a pas le même sens partout, et le document le dit plutôt que de le masquer. La mesure est universelle ; le nettoyage mémoire est complet sous Windows, partiel sous Linux (réservé à root, désactivé par défaut, et le noyau récupère seul) et **indisponible** sous macOS ; le blocage réseau par application existe sous Windows et Linux, pas sous macOS, qui exige une extension système signée par Apple. La matrice complète est en 6.0, et l'interface grise une fonction indisponible avec sa raison au lieu de la simuler.

**Principes directeurs** : (1) mesurer avant d'agir, (2) rien de destructeur sans aperçu ni journal, (3) tout changement réseau est réversible, (4) privilèges minimaux dans le temps et dans l'espace, (5) aucune télémétrie, (6) une fonction absente est annoncée, jamais simulée.

## 2. Contexte, objectifs, périmètre

### 2.1 Contexte

La suite LPDNSoft comprend `lpdnsoft-libs` (journalisation, réglages TOML versionnés, i18n Fluent, écriture atomique), `ankou` (client VPN WireGuard avec service `ankoud`, kill switch et CLI) et `lpdnsoft-launcher` (installation et mise à jour des applications depuis leurs releases GitHub). obliNiscor est la première application de la suite exclusivement Windows.

### 2.2 Objectifs mesurables

| Réf. | Objectif | Mesure de réussite |
| --- | --- | --- |
| O1 | Récupération mémoire vérifiable | Pour chaque zone, la variation mesurée (avant/après) est affichée et journalisée ; l'écart avec les compteurs Windows est < 2 % |
| O2 | Aucune régression de réactivité | Durée d'une optimisation standard < 3 s sur machine de référence ; l'interface reste fluide (aucun gel > 100 ms) |
| O3 | Sûreté du stockage | 0 suppression hors des racines déclarées sur la suite de tests d'intégration (liens symboliques, jonctions, chemins longs) |
| O4 | Réseau réversible | `net restore` supprime 100 % des règles créées ; test de désinstallation propre |
| O5 | Faible empreinte | GUI < 60 Mo de mémoire privée en repos ; service < 15 Mo ; CPU moyen < 0,5 % hors action |
| O6 | Qualité industrielle | `cargo xtask ci` vert (fmt, clippy `-D warnings`, tests, `cargo deny`) à chaque fusion |

### 2.3 Périmètre de la v1

Windows 10 (22H2), Windows 11, Windows Server 2019 et suivants — x64 et ARM64 ; Linux x86_64 et aarch64, noyau 5.15 ou plus récent (Debian/Ubuntu, Fedora/RHEL, Arch) ; macOS 12 et suivants, Intel et Apple Silicon. La disponibilité des fonctions varie selon le système (matrice 6.0) : Windows est la cible la plus complète, macOS la plus restreinte. Français et anglais.

### 2.4 Hors périmètre de la v1 (W)

- Nettoyage du registre et « boosters » aux effets non démontrables.
- Effacement sécurisé de données, récupération de fichiers.
- Inspection de paquets, filtrage par nom de domaine, VPN (c'est le rôle d'ankou).
- Désinstallation d'applications, gestion des pilotes.
- Android, iOS, les systèmes BSD et toute plateforme hors de la section 2.3.

## 3. Décisions structurantes (ADR initiaux)

| ADR | Décision | Justification | Statut |
| --- | --- | --- | --- |
| 001 | **Réécriture propre (clean room)** : on reprend les comportements observables, jamais le code de WinMemoryCleaner. Sources autorisées : documentation Microsoft, crates sous licence MIT/Apache-2.0, expérimentation propre. Chaque appel système porte en commentaire sa source. | WinMemoryCleaner est sous GPL-3.0 ; une traduction du code rendrait obliNiscor dérivé et soumis à la GPL | À valider (juridique) |
| 002 | Nom : « obliNiscor » ; binaires `obliniscor` (GUI), `obliniscor-cli`, `obliniscord` (service) ; crates `obliniscor-*` | Cohérent avec aNkou/ankoud | À valider (disponibilité du nom) |
| 003 | **Séparation des privilèges** : un service `obliniscord` (LocalSystem) exécute les opérations privilégiées ; la GUI et la CLI tournent sans droits administrateur et lui parlent par un canal nommé protégé | Évite un exécutable permanent en administrateur (source de faux positifs antivirus et de risque) ; même logique qu'`ankoud` | Retenu |
| 004 | **Multi-plateforme dès la v1** : Windows, Linux et macOS ; accès système confiné dans `obliniscor-sys`, un module par système ; traits et doublures dans `obliniscor-platform` | Un utilisateur de la suite ne dépend plus de son système ; le noyau reste testable partout ; l'ajout d'un système est un module, pas une refonte | Retenu (révisé le 31 octobre 2026) |
| 005 | **Réversible par défaut** : simulation avant suppression, journal de chaque effet, état d'origine mémorisé avant tout changement | Réduit le risque pour l'utilisateur | Retenu |
| 006 | **Mesure avant/après** obligatoire pour toute action mémoire | Se démarquer des utilitaires aux promesses invérifiables | Retenu |
| 007 | Réseau v1 : API du pare-feu Windows (`INetFwPolicy2`) ; v2 : Windows Filtering Platform (WFP) si un contrôle plus fin est nécessaire | Simplicité, visibilité pour l'utilisateur dans le pare-feu, désinstallation propre | À valider (choix de l'auteur) |
| 008 | Licence : MIT ou Apache-2.0, comme le reste de la suite (sous réserve d'ADR-001) | Cohérence | Retenu |
| 009 | **Disponibilité honnête par système** : chaque fonction est classée complète, partielle ou indisponible selon l'OS (matrice 6.0) ; une fonction indisponible est grisée avec sa raison | Évite un « optimiseur » sans effet sur macOS et des promesses invérifiables | Retenu |

## 4. Utilisateurs et scénarios

| Persona | Profil | Besoin principal |
| --- | --- | --- |
| Joueuse | 16 Go de RAM, jeux gourmands | Libérer le cache avant une session, sans gêner le jeu |
| Développeur | Machines virtuelles, navigateurs ouverts | Voir quel processus fuit, régler les priorités, nettoyer les caches de build |
| Poste modeste | 8 Go, disque dur | Éviter la pagination, retrouver de la place disque |
| Administrateur | Plusieurs postes | CLI scriptable, sortie JSON, journal exploitable |

**Scénarios clés** : (S1) l'utilisateur clique sur « Optimiser » et voit le gain par zone ; (S2) la mémoire libre passe sous un seuil et le service optimise de lui-même, avec une notification expliquant la raison ; (S3) l'utilisateur coupe l'accès réseau d'un logiciel de mise à jour qu'il ne veut pas, puis le rétablit ; (S4) l'utilisateur lance l'analyse de stockage, examine l'aperçu, confirme ; (S5) l'administrateur exécute `obliniscor mem clean --zones working-set,standby --json` depuis un script.

## 5. Fondamentaux matériels et système

Cette section fixe le modèle mental sur lequel reposent les exigences. Elle explique aussi pourquoi certaines actions sont parfois contre-productives.

### 5.1 Mémoire

- **Pages et tailles.** La mémoire est gérée par pages de 4 Ko ; les grandes pages (2 Mo) existent mais ne sont ni paginables ni gérées par ces mécanismes. Le processeur traduit les adresses par la MMU et un cache de traduction (TLB) ; vider des working sets provoque des défauts de page « soft » (la page est encore en RAM) ou « hard » (lecture disque).
- **Working set** : pages d'un processus actuellement résidentes. Le réduire les fait passer dans les listes *standby* ou *modifiée* ; elles ne sont donc pas perdues, mais le processus devra les redemander.
- **Listes de pages.** Windows distingue les pages actives, la liste *standby* (propres, réutilisables, réparties en **8 niveaux de priorité**, 0 à 7), la liste *modifiée* (sales, à écrire sur disque avant réutilisation), la liste *libre* et la liste *à zéro*. Les pages *standby* sont un **cache chaud** : les purger libère de la RAM « officiellement libre » mais détruit un cache utile, ce qui coûte des lectures disque ensuite.
- **Compression mémoire** (Windows 10 et suivants) : des pages peu utilisées sont compressées en RAM avant la pagination. Elle apparaît dans les compteurs du système et doit être affichée à part.
- **Fichier d'échange et commit.** La charge de commit (mémoire promise) est bornée par RAM + fichier d'échange ; c'est ce seuil, et non la RAM libre, qui provoque les échecs d'allocation.
- **NUMA** : sur les machines multi-nœuds (serveurs, certaines stations), la mémoire est locale à un nœud. Les compteurs doivent pouvoir être lus par nœud ; v1 : agrégé, avec structure prête pour le par-nœud.
- **Cache de fichiers** : vider le cache modifié force l'écriture de données en attente ; sur disque dur, c'est une rafale d'E/S perceptible.

### 5.2 Stockage

- **HDD, SSD SATA, NVMe** ont des profils très différents : le disque dur subit un coût de déplacement des têtes (la défragmentation y est utile) ; un SSD ne doit **jamais** être défragmenté (usure) mais bénéficie de la commande **TRIM**, qui informe le contrôleur des blocs libérés.
- **Détection** : le type de média se déduit de la propriété « pénalité de recherche » (*seek penalty*) du périphérique de stockage ; les volumes sur matrices RAID ou disques virtuels peuvent la mentir (à valider).
- **NTFS** : la table de fichiers maîtresse (MFT), le journal USN et les clichés instantanés (VSS) influencent la place réellement récupérable. Supprimer un fichier protégé par un cliché ne libère pas toujours l'espace immédiatement.
- **Points de reparse** : jonctions, liens symboliques et points de montage peuvent faire sortir d'une racine de nettoyage. Toute traversée doit les détecter et les refuser (SEC-4).

### 5.3 Processeur et ordonnancement

- Classes de priorité de processus, priorités de threads et **mode efficacité (EcoQoS)** : indique à l'ordonnanceur de privilégier l'économie d'énergie ; sur processeurs hybrides (cœurs P et E), il oriente le travail vers les cœurs économes.
- Au-delà de 64 processeurs logiques, Windows utilise des **groupes de processeurs** : l'affinité par masque simple ne suffit plus.
- Le SMT (hyper-threading) double les processeurs logiques sans doubler la puissance ; les pourcentages de CPU doivent être normalisés par le nombre de processeurs logiques.

### 5.4 Réseau

- Le filtrage de Windows s'organise en couches (ALE, flux, transport). Le pare-feu applique des règles par programme, par service, par paquet d'application du Store (identifié par son SID de conteneur) et par profil (domaine, privé, public).
- Un même exécutable de service (`svchost.exe`) héberge plusieurs services : bloquer le programme bloquerait tout. Il faut cibler le **service** par son nom court.
- Une règle de blocage l'emporte sur une règle d'autorisation ; les stratégies de groupe ou un pare-feu tiers peuvent prendre la main (à détecter et signaler).

### 5.5 Conséquences de conception

1. Le système gère déjà bien la mémoire ; obliNiscor est un **outil de contrôle à la demande**, pas un correctif magique. Les valeurs par défaut sont prudentes, et la purge complète des listes *standby* n'est jamais automatique sans seuil, hystérésis et délai minimal (FR-MEM-8).
2. Toute action est accompagnée de sa **mesure** et d'une explication de son coût possible.
3. Les actions sur disque dur et sur SSD sont distinguées.

### 5.6 Fondamentaux sous Linux et macOS

- **Mémoire.** Linux expose `/proc/meminfo` (`MemTotal`, `MemFree`, `MemAvailable`, `Buffers`, `Cached`, `SReclaimable`, `SwapCached`) et `/proc/vmstat` (défauts de page, activité de récupération). `MemAvailable` est déjà l'estimation du noyau : c'est la seule valeur à présenter comme « disponible ». Vider les caches se fait par `/proc/sys/vm/drop_caches`, réservé à root : c'est un outil de diagnostic, pas d'entretien, et le noyau récupère seul. macOS expose `host_statistics64` (pages libres, actives, inactives, câblées, compresseur) ; il n'existe **aucune API publique** pour purger les caches — `purge` est un outil interne d'Apple. Conséquence : sous macOS, obliNiscor mesure et n'agit pas.
- **Processus.** Linux expose `/proc/<pid>/` (état, mémoire, E/S, threads). La priorité est celle de `nice` ; la pondération réelle passe par cgroups v2 (`cpu.weight`), qui remplace la notion de classe de priorité. La réduction du working set existe (`process_madvise`, `MADV_PAGEOUT`, ou `/proc/<pid>/clear_refs`) mais exige des droits. macOS expose `libproc` (`proc_listallpids`, `proc_pidinfo`) ; il n'y a **pas** d'équivalent de réduction de working set.
- **Stockage.** Linux expose les volumes par `/proc/mounts` et `statfs` ; le type de média se lit dans `/sys/block/<périphérique>/queue/rotational` (`0` = SSD/NVMe), et le TRIM se déclenche par `fstrim` (ioctl `FITRIM`). La défragmentation n'a pas de sens sur ext4, XFS ou Btrfs. macOS expose `statfs` et `diskutil` ; APFS gère le TRIM seul.
- **Suppression sûre.** Unix offre de meilleures primitives que Windows : `openat2` avec `RESOLVE_BENEATH` (Linux 5.6+) garantit qu'on ne sort pas d'une racine, `O_NOFOLLOW` refuse un lien symbolique, `unlinkat` supprime relativement à un descripteur de dossier. La course entre vérification et suppression se ferme donc plus simplement — mais elle existe encore entre l'énumération et la suppression, et la revalidation reste obligatoire.
- **Réseau.** Linux filtre par nftables ; le ciblage par application passe par les cgroups v2 (`meta cgroup`, `socket cgroupv2`) ou par identifiant utilisateur. aNkou utilise déjà nftables pour son kill switch : la connaissance est acquise et réutilisable. macOS filtre par PF (`pfctl`), mais bloquer une application précise exige une **Network Extension**, c'est-à-dire une extension système signée et autorisée par Apple : hors de portée d'une application indépendante. Sous macOS, le blocage se fait donc par utilisateur et par port, jamais par programme.
- **Service.** Trois modèles pour un même rôle : SCM sous Windows, systemd sous Linux, launchd sous macOS. Le canal nommé Windows a son équivalent direct en socket Unix, et l'identité de l'appelant s'obtient par `SO_PEERCRED` (Linux) ou `LOCAL_PEERPID` (macOS) — plus simple et plus sûr que les ACL de canal.

## 6. Exigences fonctionnelles

### 6.0 Disponibilité par système

La v1 couvre trois systèmes, et toutes les exigences ne s'y appliquent pas au même degré. Une fonction absente est **affichée comme indisponible avec sa raison**, jamais simulée. Cette matrice fait foi pour l'interface, les tests et les critères d'acceptation.

| Fonction | Windows 10/11/Server | Linux | macOS |
| --- | --- | --- | --- |
| Mesure de la mémoire (FR-MEM-1, 2) | complet | complet (`/proc`) | complet (`host_statistics64`) |
| Nettoyage mémoire (FR-MEM-3 à 9) | complet | **partiel** : caches système et working set, réservé à root, désactivé par défaut | **indisponible** (aucune API publique) |
| Liste et métriques des processus (FR-PRC-1, 2) | complet | complet (`/proc`) | complet (`libproc`) |
| Réduction du working set (FR-PRC-3) | complet | possible (`process_madvise`, droits requis) | **indisponible** |
| Priorité, mode efficacité (FR-PRC-4, 5) | classes + EcoQoS | `nice` + cgroups v2 | `nice`, portée limitée |
| Processus protégés, arrêt en deux temps (FR-PRC-7, 8) | complet | complet | complet |
| Services (FR-PRC-10) | SCM | systemd | launchd |
| Volumes et espace (FR-STO-1, 2) | complet | complet (`/sys`, `statfs`) | complet (`statfs`) |
| Nettoyage sûr (FR-STO-3 à 8) | complet | complet (`openat2`) | complet (`O_NOFOLLOW`) |
| TRIM (FR-STO-9) | — | `fstrim` | automatique (APFS) |
| Défragmentation (FR-STO-9) | HDD seulement | **sans objet** | **sans objet** |
| Inventaire réseau (FR-NET-1) | complet | complet | complet |
| Blocage par application (FR-NET-2, 3) | pare-feu Windows | nftables par cgroup | **indisponible** (extension système Apple requise) |
| Profils et immunité (FR-NET-4 à 6) | complet | complet | partiel : par utilisateur et par port |
| Service privilégié (FR-SVC-1) | SCM | systemd | launchd |
| IPC (FR-SVC-2) | canal nommé | socket Unix (`SO_PEERCRED`) | socket Unix (`LOCAL_PEERPID`) |

### 6.1 Module Mémoire (FR-MEM)
| ID | Exigence | Prio |
| --- | --- | --- |
| FR-MEM-1 | Lire l'état : RAM physique (totale, libre, disponible), charge de commit, fichier d'échange, taille des listes (active, standby par priorité, modifiée, libre/zéro), mémoire compressée. Sources : `GlobalMemoryStatusEx`, `GetPerformanceInfo`, requête noyau de la liste mémoire, compteurs PDH (à valider) | M |
| FR-MEM-2 | Rafraîchissement configurable (par défaut 1 s fenêtre visible, aucun sondage lorsque l'interface est masquée) | M |
| FR-MEM-3 | Zones nettoyables : working set (tous processus), cache système, liste standby complète, liste standby basse priorité, liste modifiée, pages combinées (Windows 8 et suivants), cache de fichiers modifiés (vidage des volumes fixes), cache du registre (Windows 8.1 et suivants) | M |
| FR-MEM-4 | Détecter la disponibilité de chaque zone selon la version et le numéro de build de Windows ; une zone indisponible est grisée avec explication, jamais en erreur | M |
| FR-MEM-5 | Activer les privilèges nécessaires (profil d'un processus unique, quotas) uniquement pendant l'opération puis les retirer | M |
| FR-MEM-6 | **Mesure avant/après** : octets libérés par zone, durée, variation de la liste standby, défauts de page durs observés dans les 30 s suivantes (indicateur de coût) | M |
| FR-MEM-7 | Raisons d'exécution tracées : manuelle, planifiée, mémoire basse, raccourci, CLI | M |
| FR-MEM-8 | Déclenchement automatique par seuil : RAM libre < X % **et** liste standby > Y Mo, avec hystérésis, délai minimal entre exécutions (10 min par défaut) et plafond d'exécutions par heure | S |
| FR-MEM-9 | Garde-fous : suspendre l'optimisation automatique en plein écran ou jeu (état de notification de l'utilisateur), sous forte activité disque, ou sur batterie faible | S |
| FR-MEM-10 | Liste d'exclusion de processus pour la réduction de working set | M |
| FR-MEM-11 | Profils : *léger* (standby basse priorité), *standard*, *complet* (toutes zones) | C |
| FR-MEM-12 | Annulation coopérative entre deux zones ; aucune opération ne bloque le fil d'interface | M |
| FR-MEM-13 | Historique des exécutions (JSON Lines, écriture atomique) consultable dans l'interface et par CLI | S |
| FR-MEM-14 | Mode simulation : indique les zones qui seraient traitées et l'estimation de gain, sans appel noyau | M |

> **Disponibilité** : tout est applicable aux trois systèmes, sauf FR-PRC-5 (mode efficacité : approximatif sous Linux par cgroups, limité sous macOS) et FR-PRC-3 (réduction du working set : indisponible sous macOS). FR-PRC-10 change de modèle selon le système (SCM, systemd, launchd) mais garde le même contrat.

### 6.2 Module Processus (FR-PRC)

| ID | Exigence | Prio |
| --- | --- | --- |
| FR-PRC-1 | Énumérer les processus : PID, nom, chemin, utilisateur, session, working set, mémoire privée, commit, CPU % normalisé, E/S, threads, handles, priorité, état EcoQoS, signature numérique | M |
| FR-PRC-2 | Affichage en arbre parent/enfant et recherche | M |
| FR-PRC-3 | Réduction du working set d'un processus choisi | M |
| FR-PRC-4 | Changer la classe de priorité ; règles persistantes par exécutable (chemin signé, pas seulement le nom) | M |
| FR-PRC-5 | Activer le mode efficacité (EcoQoS) pour les processus d'arrière-plan | S |
| FR-PRC-6 | Règles d'arrière-plan : après N minutes sans fenêtre au premier plan, réduire priorité et/ou working set | S |
| FR-PRC-7 | Arrêt en deux temps : demande de fermeture de fenêtre, puis terminaison forcée **après confirmation explicite**, avec avertissement si des données non enregistrées sont probables | M |
| FR-PRC-8 | **Liste de processus protégés** : processus système et protégés (PPL), antivirus, `obliniscord`, `ankoud`, gestionnaire de session ; noyau immuable dans le code, extensible par l'utilisateur | M |
| FR-PRC-9 | Détection de dérive mémoire : croissance quasi monotone de la mémoire privée sur une fenêtre configurable, avec alerte | S |
| FR-PRC-10 | Services : lister, démarrer, arrêter, changer le type de démarrage, en mémorisant l'état d'origine pour rétablissement | S |
| FR-PRC-11 | Programmes au démarrage (clés Run, dossiers de démarrage, tâches planifiées) : lister, désactiver et réactiver de façon réversible | C |
| FR-PRC-12 | Affinité et ensembles de processeurs, y compris groupes > 64 processeurs logiques | C |

> **Disponibilité** : l'inventaire, l'analyse et le nettoyage sûr sont applicables partout ; sous Unix, `openat2` et `O_NOFOLLOW` rendent OBL-053 plus simple que sous Windows. FR-STO-9 (TRIM) se limite à Linux et Windows ; la défragmentation n'a de sens que sous Windows, sur disque dur.

### 6.3 Module Stockage (FR-STO)

| ID | Exigence | Prio |
| --- | --- | --- |
| FR-STO-1 | Inventaire des volumes : lettre ou point de montage, type de média (HDD, SSD, NVMe), capacité, espace libre, système de fichiers, état de santé si le périphérique l'expose (à valider) | M |
| FR-STO-2 | Analyse d'espace : plus gros dossiers et fichiers, parcours parallèle, interruptible | M |
| FR-STO-3 | **Catalogue déclaratif de cibles** (TOML, versionné) : identifiant, dossier connu ou chemin, motif, âge minimal, niveau de risque (faible, moyen, élevé), description, droits requis, redémarrage nécessaire | M |
| FR-STO-4 | Cibles v1 : fichiers temporaires utilisateur et système, caches de miniatures, caches de navigateurs (Edge, Chrome, Firefox), corbeille, cache de téléchargement de Windows Update (service arrêté pendant l'opération), rapports d'erreurs, caches de shaders, cache d'optimisation de livraison | M |
| FR-STO-5 | **Simulation par défaut** : aperçu (liste, tailles, risque) puis confirmation explicite pour exécuter | M |
| FR-STO-6 | Suppression sûre : canonicalisation des chemins, refus de franchir un point de reparse, refus de sortir de la racine déclarée, fichiers en usage ignorés sans erreur fatale, aucun effet de bord sur les dossiers personnels | M |
| FR-STO-7 | Journal détaillé des suppressions (chemin, taille, résultat) et rapport d'espace récupéré | M |
| FR-STO-8 | Envoi à la corbeille pour les cibles de risque moyen et élevé quand c'est possible | S |
| FR-STO-9 | SSD : déclencher la ré-optimisation (TRIM) ; HDD : défragmentation optionnelle ; jamais de défragmentation sur SSD | S |
| FR-STO-10 | Recherche de doublons (hachage) en lecture seule, suppression au choix de l'utilisateur | C |
| FR-STO-11 | Lecture directe de la MFT pour accélérer l'analyse (droits élevés) | C |
| FR-STO-12 | Nettoyage du registre, effacement sécurisé | W |

> **Disponibilité** : Windows et Linux sont complets (pare-feu Windows d'un côté, nftables de l'autre). Sous macOS, FR-NET-2 et FR-NET-3 sont **indisponibles** : bloquer par programme exige une Network Extension signée par Apple. Les profils s'y appliquent par utilisateur et par port, et l'interface le dit.

### 6.4 Module Réseau (FR-NET)

| ID | Exigence | Prio |
| --- | --- | --- |
| FR-NET-1 | Inventaire des applications actives sur le réseau : connexions TCP/UDP avec PID, débit par processus (suivi d'événements noyau réseau), adresses distantes | M |
| FR-NET-2 | Règles par application : bloquer envoi, réception ou les deux ; identification par chemin signé, par nom de service ou par identifiant de paquet d'application du Store | M |
| FR-NET-3 | Mise en œuvre v1 par le pare-feu Windows, en groupe de règles « obliNiscor » à noms normalisés | M |
| FR-NET-4 | Profils : *Normal*, *Silencieux* (liste maintenue de composants de télémétrie et de mise à jour), *Strict* (liste blanche seule), *Hors ligne* par application | M |
| FR-NET-5 | **Liste d'immunité** non modifiable : trafic de loopback, DNS, DHCP, trafic d'ankou (service `ankoud`, UDP WireGuard) et d'obliNiscor lui-même | M |
| FR-NET-6 | Filet de sécurité : annulation automatique en 60 s si un profil à effet large n'est pas confirmé, commande `net restore` et bouton de rétablissement | M |
| FR-NET-7 | Détection d'environnement : pare-feu désactivé, géré par stratégie de groupe ou remplacé par un produit tiers → avertissement clair, aucune promesse de blocage | M |
| FR-NET-8 | Contrôle de services Windows liés au réseau (mises à jour, optimisation de livraison, télémétrie) avec avertissement de sécurité et état d'origine mémorisé ; gérer les relances par déclencheurs | S |
| FR-NET-9 | Planification : plages horaires, session de jeu | S |
| FR-NET-10 | Journal horodaté de chaque changement de règle | M |
| FR-NET-11 | Mode WFP (filtres au niveau ALE, fournisseur et sous-couche dédiés, filtres dynamiques disparaissant à l'arrêt du service comme garde-fou) | C |
| FR-NET-12 | Inspection de paquets, filtrage par domaine | W |

> **Disponibilité** : même architecture partout — un démon privilégié, un client non privilégié, un canal local — avec trois implémentations (SCM, systemd, launchd) et deux transports (canal nommé, socket Unix).

### 6.5 Service et automatisation (FR-SVC)

| ID | Exigence | Prio |
| --- | --- | --- |
| FR-SVC-1 | Service Windows `obliniscord` (LocalSystem), démarrage automatique différé, reprise sur échec | M |
| FR-SVC-2 | Canal nommé protégé par une liste de contrôle d'accès (administrateurs et groupe `obliniscor`) ; protocole versionné, messages typés, limites de taille | M |
| FR-SVC-3 | Planificateur interne (intervalles, seuils, événements d'alimentation) ; reprise correcte après veille et hibernation | M |
| FR-SVC-4 | Chien de garde : redémarrage du service, détection de blocage d'une opération | S |
| FR-SVC-5 | Installation et désinstallation propres, sans redémarrage, avec retrait de toutes les règles réseau et tâches | M |

### 6.6 CLI (FR-CLI)

Commandes : `mem status|clean|history`, `proc list|trim|prio|kill`, `disk volumes|scan|clean`, `net apps|rules|block|allow|profile|restore`, `service install|uninstall|status`, `config get|set|reset`, `version`. Options communes : `--dry-run`, `--json`, `--lang`, `--yes`. Codes de sortie documentés (0 succès, 1 erreur, 2 usage, 3 droits insuffisants, 4 opération partielle). Les commandes privilégiées passent par le service, sans élévation de la CLI (priorité M).

### 6.7 Interface (FR-UI)

egui avec le thème `lpdnsoft-ui` : fenêtre principale à onglets (Tableau de bord, Mémoire, Processus, Stockage, Réseau, Journal, Réglages), mode compact, icône de zone de notification avec usage mémoire et seuils de couleur, raccourci global configurable (avec détection de conflit), notifications indiquant la raison et le gain, confirmations pour toute action destructive, fonctionnement au clavier, mise à l'échelle DPI, contraste conforme WCAG AA, français et anglais (priorité M ; mode compact et icône S).

### 6.8 Réglages, journal, langues (FR-CFG)

Réglages TOML versionnés avec migration via `lpdnsoft-settings` (machine : `%ProgramData%\obliNiscor` ; utilisateur : `%APPDATA%\obliNiscor`) ; écritures atomiques (`lpdnsoft-atomic`) ; journal `tracing` avec rotation et masquage des secrets (`lpdnsoft-log`) ; traductions Fluent vérifiées à la compilation (`lpdnsoft-i18n`) ; commande de remise aux réglages d'usine qui préserve la langue ; export et import des réglages (priorité M sauf export/import, S).

## 7. Exigences non fonctionnelles

| ID | Exigence | Cible |
| --- | --- | --- |
| NFR-PERF-1 | Empreinte au repos | GUI < 60 Mo mémoire privée ; service < 15 Mo ; CPU moyen < 0,5 % |
| NFR-PERF-2 | Pas de sondage actif : attentes événementielles (événements, ports de complétion), aucun cœur occupé en boucle | Vérifié par profilage |
| NFR-PERF-3 | Démarrage de la GUI | < 1 s sur machine de référence |
| NFR-PERF-4 | Latence d'un appel au service | < 50 ms hors opération longue |
| NFR-REL-1 | Aucune opération destructive sans journal préalable ; reprise cohérente après plantage | Test d'injection de pannes |
| NFR-REL-2 | Un échec d'une zone ou d'une cible n'interrompt pas les autres | Test d'intégration |
| NFR-SEC-1 | Voir section 9 | — |
| NFR-COMP-1 | Windows 10 22H2 et suivants ; Linux noyau 5.15+ ; macOS 12+ | Matrice de tests |
| NFR-COMP-2 | Windows x64 et ARM64 ; Linux x86_64 et aarch64 ; macOS Intel et Apple Silicon. Chaque architecture est compilée en CI et testée sur machine réelle avant publication | — |
| NFR-PRIV-1 | Aucune télémétrie ni appel réseau initié par l'application ; la recherche de mises à jour est assurée par le launcher | Revue de code et test de blocage |
| NFR-I18N-1 | Aucune chaîne en dur dans l'interface ; français et anglais complets | Compilation échouée sinon |
| NFR-A11Y-1 | Navigation clavier complète, contraste AA, échelle DPI 100 à 250 % | Liste de contrôle |
| NFR-QUAL-1 | `fmt`, `clippy -D warnings`, tests, `cargo deny` verts ; couverture du noyau ≥ 80 % | CI |
| NFR-MAINT-1 | Chaque appel système isolé dans `obliniscor-sys` (et sous Linux ou macOS, dans le module du système), documenté : source, privilèges, versions, comportement si absent | Revue |
| NFR-DIST-1 | Binaires signés, empreintes SHA-256 publiées dans `SHA256SUMS`, build reproductible dans la mesure du possible | CI de release |

## 8. Architecture

### 8.1 Vue d'ensemble

| Composant | Contexte d'exécution | Rôle |
| --- | --- | --- |
| `obliniscord` (service) | Privilégié : LocalSystem (Windows), systemd système (Linux), launchd daemon (macOS) | Seul composant privilégié : appels noyau, suppression, règles réseau, planification, journal d'audit |
| `obliniscor` (GUI egui) | Utilisateur standard | Affichage, saisie, notifications ; ne fait aucune opération privilégiée |
| `obliniscor-cli` | Utilisateur standard | Pilotage scriptable ; même client IPC que la GUI |
| Canal nommé `\\.\pipe\obliniscor` | Entre les trois | Protocole versionné, ACL restrictive, clients distants refusés |

Les lectures non privilégiées (liste des processus, état de la mémoire) peuvent être faites directement par la GUI en mode dégradé si le service est absent ; toute mutation exige le service.

### 8.2 Espace de travail Cargo

| Crate | Contenu | Particularités |
| --- | --- | --- |
| `obliniscor-core` | Modèles (instantané mémoire, zones, profils, règles, cibles), politiques et garde-fous, planificateur | Aucune E/S ; `#![forbid(unsafe_code)]` ; testé sur toutes plateformes |
| `obliniscor-proto` | Messages IPC, cadrage par longueur (limite 1 Mio), versions | `serde`, fuzzé |
| `obliniscor-platform` | Traits `MemoryProbe`, `MemoryCleaner`, `ProcessControl`, `DiskInventory`, `FirewallControl`, `ServiceControl`, `PowerEvents`, plus doublures de test | Frontire testable de l'OS |
| `obliniscor-sys` | Implémentations par système, puis par domaine : `windows/`, `linux/`, `macos/`, chacun avec `mem`, `proc`, `disk`, `net`, `svc`, `power`. Windows utilise la crate `windows` ; Linux lit `/proc` et `/sys` et appelle `nft`, `fstrim` ; macOS utilise `libproc` et `statfs` | Seul crate où `unsafe` est permis, et sous Linux ou macOS il peut n'y en avoir **aucun**. Chaque bloc commente `// SAFETY:` ; lint `undocumented_unsafe_blocks` en erreur |
| `obliniscor-store` | Historique JSONL, règles, catalogue de cibles, état d'origine pour rétablissement | Écritures atomiques via `lpdnsoft-atomic` |
| `obliniscor-service` | Boucle du service, serveur IPC, planificateur, chien de garde | Crate `windows-service` |
| `obliniscor-client` | Client IPC partagé | Reconnexion, délais, versions |
| `obliniscor-cli`, `apps/obliniscor` | Binaires | CLI avec `clap` ; GUI egui |
| `xtask` | `cargo xtask ci`, empaquetage (MSI, `.deb`/`.rpm`, `.pkg`), génération du manifeste | Comme ankou et le launcher |

Dépendances de la suite : `lpdnsoft-log`, `lpdnsoft-settings`, `lpdnsoft-i18n`, `lpdnsoft-atomic`, clonées à côté du dépôt comme pour ankou. Toute autre dépendance est validée par `cargo deny` (licences, avis, doublons).

### 8.3 Erreurs, concurrence, ressources

- Erreurs typées avec `thiserror`, codes stables exposés par l'IPC et la CLI ; les erreurs Win32 sont traduites en `HRESULT` lisibles et en messages localisés.
- Service : runtime asynchrone `tokio` à un seul fil pour l'IPC ; les opérations lourdes (suppression, analyse) tournent dans un pool borné avec annulation coopérative (jeton partagé).
- GUI : fil d'interface séparé des fils de travail ; aucune attente bloquante sur le fil d'interface.
- Toute ressource système (handle, jeton, privilège) est portée par un type RAII qui la libère en cas d'erreur.

### 8.4 Protocole IPC

| Catégorie | Exemples de messages | Droit requis |
| --- | --- | --- |
| Lecture | `GetStatus`, `ListProcesses`, `ListVolumes`, `ListNetApps`, `GetHistory` | Membre du groupe `obliniscor` ou administrateur |
| Mutation réversible | `SetPriority`, `SetNetRule`, `SetProfile`, `UpdateSettings` | Idem, journalisé avec l'identité de l'appelant |
| Opération lourde | `RunMemoryClean`, `ScanDisk`, `RunDiskClean` | Idem ; `RunDiskClean` nécessite un aperçu préalable validé (jeton d'aperçu à usage unique) |
| Destructif | `KillProcess` (forcé), `RunDiskClean` après aperçu, `StopService` | Confirmation explicite incluse dans le message |
| Événements | Abonnement à la progression et aux alertes | Lecture |

Poignée de main : version du protocole et jeu de fonctions ; refus clair si incompatible. L'identité de l'appelant est obtenue côté service (jeton du client du canal), jamais déclarée par le client.

### 8.5 Séquence d'une optimisation mémoire

1. Le client envoie `RunMemoryClean { zones, dry_run, reason }`.
2. Le service vérifie l'autorisation, les garde-fous (FR-MEM-9) et la disponibilité de chaque zone (FR-MEM-4).
3. Capture de l'état **avant** (listes, disponible, commit, défauts de page cumulés).
4. Activation du privilège nécessaire (garde RAII), exécution zone par zone dans un ordre fixe : working set, cache système, liste modifiée, pages combinées, cache de fichiers modifiés, cache du registre, puis standby (basse priorité ou complète) en dernier.
5. Pause de stabilisation courte (≈ 500 ms) puis capture de l'état **après**.
6. Écriture d'une entrée d'historique, événement de fin, notification.
7. Fenêtre d'observation de 30 s : les défauts de page durs sont comparés à la base de référence et ajoutés à l'entrée (indicateur de coût, FR-MEM-6).

L'ordre place la purge *standby* en dernier : les étapes précédentes y déplacent des pages, que la purge libère ensuite en une seule fois.

### 8.6 Définition du « gain »

Le gain affiché est la variation de la mémoire **disponible** (libre + standby) et, séparément, de la mémoire **libre** réelle. L'interface explique la différence : libérer du cache standby augmente la mémoire libre mais diminue le cache chaud. L'indicateur de coût complète le gain.

### 8.7 Persistance

| Fichier | Emplacement | Contenu | Format |
| --- | --- | --- | --- |
| `settings.toml` | `%ProgramData%\obliNiscor` | Réglages machine (service, seuils, profils) | TOML versionné |
| `user.toml` | `%APPDATA%\obliNiscor` | Préférences d'interface, langue, thème | TOML versionné |
| `targets.toml` | Embarqué + surcharge machine | Catalogue de cibles de nettoyage | TOML, schéma versionné |
| `history.jsonl` | `%ProgramData%\obliNiscor` | Historique des optimisations et nettoyages | JSON Lines, rotation |
| `net-state.json` | `%ProgramData%\obliNiscor` | Règles créées et état d'origine des services (pour `net restore`) | JSON, écriture atomique |
| Journaux | `%ProgramData%\obliNiscor\logs` | Journal `tracing` rotatif | Texte structuré |

Les journaux d'événements Windows reçoivent un résumé des opérations notables (source `obliNiscor`), pour l'audit.

## 9. Sécurité

### 9.1 Actifs et hypothèses

Actifs : intégrité des fichiers de l'utilisateur, disponibilité du réseau, intégrité du système, confidentialité du journal. Hypothèse : un utilisateur standard peut être malveillant ou compromis ; le service est donc la frontière de confiance.

### 9.2 Menaces principales

| Menace | Scénario | Parade |
| --- | --- | --- |
| Élévation de privilèges via l'IPC | Un processus non privilégié pousse le service à supprimer ou arrêter ce qu'il ne devrait pas | ACL du canal, droits par message, validation stricte, jetons d'aperçu, listes de protection non modifiables par IPC |
| Usurpation du canal | Un programme crée le canal avant le service | `FILE_FLAG_FIRST_PIPE_INSTANCE`, refus des clients distants, vérification de l'identité du serveur par le client |
| Traversée de chemin et liens | Un lien symbolique ou une jonction redirige la suppression hors de la racine | Ouverture sans suivre les points de reparse, vérification du chemin final, opérations relatives à un handle de dossier |
| Course (TOCTOU) | Le contenu change entre l'aperçu et la suppression | Revalidation au moment de la suppression, jeton d'aperçu lié à l'empreinte de la liste |
| Chargement de DLL piégée | Chargement depuis un dossier modifiable | Restriction du chemin de recherche aux dossiers système, aucun `LoadLibrary` dynamique étrangers |
| Catalogue de cibles malveillant | Une cible à risque ajoutée par un utilisateur | Catalogue embarqué et signé ; cibles utilisateur limitées à son profil, sans droits élevés |
| Détournement des règles réseau | Suppression ou ajout de règles par un tiers | Règles regroupées, état attendu comparé à l'état réel au démarrage, réconciliation journalisée |
| Faux positifs antivirus | Signalement du service ou de la persistance | Binaires signés, manifeste et métadonnées complètes, documentation publique des API, aucune persistance cachée, soumission aux éditeurs |

### 9.3 Exigences de sécurité

- **SEC-1** Le service retire les privilèges inutiles de son jeton au démarrage et n'active les autres que le temps d'une opération.
- **SEC-2** Canal nommé : instance unique, ACL explicite, clients distants refusés, taille de message bornée, délais d'attente.
- **SEC-3** Toute entrée IPC est validée (types, bornes, chemins canoniques) avant usage.
- **SEC-4** Suppression sans suivre les points de reparse ; chemin final vérifié ; noms réservés, flux de données alternatifs et chemins longs testés.
- **SEC-5** Catalogue de cibles embarqué et signé ; cibles de risque élevé impossibles à ajouter par l'utilisateur.
- **SEC-6** Aucune bibliothèque chargée hors des dossiers système et du dossier d'installation protégé.
- **SEC-7** Binaires signés (Authenticode), empreintes SHA-256 publiées ; vérification par le launcher avant installation.
- **SEC-8** Journaux sans données personnelles inutiles ; masquage des secrets ; chemins d'utilisateurs conservés mais exportables sous forme anonymisée.
- **SEC-9** `unsafe` limité à `obliniscor-sys`, et absent des modules Linux et macOS lorsque l'appel d'un outil système suffit ; chaque bloc est justifié et relu.
- **SEC-10** `cargo deny` et audit des avis de sécurité dans la CI ; mise à jour des dépendances planifiée.
- **SEC-11** Aucune écriture hors des dossiers prévus ; le dossier d'installation n'est pas modifiable par un utilisateur standard.
- **SEC-12** Mode strict du réseau : l'immunité d'ankou et d'obliNiscor est vérifiée à chaque application de profil.
- **SEC-13** Sous Linux, l'unité systemd du service déclare un durcissement explicite (`NoNewPrivileges`, `ProtectSystem`, `ProtectHome`, liste des capacités) ; la configuration est vérifiée par un test qui lit l'unité installée.
- **SEC-14** Sous macOS, aucune extension système ni droit Apple n'est demandé : l'application n'exige que les droits qu'un utilisateur peut accorder, et une fonction qui en exigerait davantage est déclarée indisponible (matrice 6.0).

## 10. Stratégie de qualité et de tests

### 10.1 Niveaux de tests

| Niveau | Contenu | Outils | Cible |
| --- | --- | --- | --- |
| Unitaires | Politiques, planificateur, calculs de gain, canonicalisation de chemins | `cargo test`, `proptest` | Couverture du noyau ≥ 80 % |
| Propriétés | Aucune suppression hors racine pour tout chemin généré ; viabilité de la sérialisation IPC | `proptest` | Invariants documentés |
| Fuzzing | Analyseur IPC, catalogue TOML, noms de fichiers | `cargo-fuzz` | 1 h cumulée par nuit sans plantage |
| Intégration par système | Appels réels sur machine de CI : Windows (listes de processus, volumes, règles de pare-feu créées puis supprimées, service installé puis retiré), Linux (`/proc`, `fstrim`, nftables, unité systemd), macOS (`libproc`, `statfs`, launchd) | Runners Windows, Ubuntu et macOS | Verts à chaque fusion |
| Suppression sûre | Arborescences piégées : liens symboliques, jonctions, liens physiques, chemins > 260 caractères, noms réservés (CON, NUL), flux alternatifs, nom avec espace final | Tests dédiés | 0 suppression hors racine |
| Chaos | Arrêt brutal du service pendant suppression, coupure pendant l'application d'un profil réseau | Injection de pannes | État cohérent, restauration possible |
| Matériel et performance | Mesures sur machines réelles ou virtuelles (voir 10.2) | Scripts, `criterion` | Cibles de la section 2.2 |
| Interface | Captures de référence, navigation clavier, DPI, langues | Liste de contrôle + tests de capture | Aucune chaîne tronquée |

### 10.2 Matrice matérielle de référence

| Machine | Configuration | Objet principal |
| --- | --- | --- |
| A | Windows 10 22H2, 8 Go, disque dur | Pression mémoire, coût d'E/S de la purge, pagination |
| B | Windows 11, 16 Go, NVMe | Cas nominal des joueurs et développeurs, TRIM |
| C | Windows 11, 32 Go ou plus, processeur hybride (cœurs P et E) | EcoQoS et priorités |
| D | Windows Server 2022 en machine virtuelle (plusieurs nœuds NUMA simulés si possible) | Compteurs par nœud, service sans session |
| E | Windows 11 ARM64 | Compilation et comportement natif |
| F | Windows 11 avec antivirus tiers et ankou installé | Coexistence, faux positifs, immunité réseau |
| G | Ubuntu 24.04, 8 Go, ext4 | Équivalents Linux : `/proc`, `openat2`, `fstrim`, nftables par cgroup, systemd |
| H | macOS 14 sur Apple Silicon | Mesure seule (mémoire), `libproc`, `statfs`, launchd ; vérifie qu'une fonction indisponible est annoncée et non simulée |

### 10.3 Scénario de mesure de référence (« preuve de valeur »)

1. Redémarrer la machine, ouvrir un jeu ou un jeu de 30 onglets de navigateur, puis le fermer pour peupler la liste *standby*.
2. Relever disponible, libre, standby par priorité, défauts de page durs.
3. Lancer l'optimisation (profils *léger*, *standard*, *complet*) ; relever à +0 s, +30 s et +5 min.
4. Démarrer une application gourmande ; mesurer le temps de démarrage et les lectures disque.
5. Conclure par profil : gain, coût, recommandation. Ces résultats alimentent les valeurs par défaut de FR-MEM-8 et sont publiés dans `docs/mesures/`.

### 10.4 Définition de « terminé »

Une tâche est terminée quand : le code est fusionné avec `cargo xtask ci` vert ; les tests de la tâche existent ; la documentation (ADR, runbook, page de l'interface) est à jour ; les chaînes sont traduites en français et en anglais ; les blocs `unsafe` sont relus ; et le critère d'acceptation de la tâche est démontré.

## 11. Intégration continue, livraison, distribution

- **CI à chaque fusion** : `cargo xtask ci` (fmt, clippy `-D warnings`, tests, `cargo deny`) sur **trois systèmes** — Windows x64, Linux x86_64, macOS arm64 — car le noyau et la CLI doivent être testés partout ; compilation croisée Windows ARM64 et Linux aarch64 ; tests d'intégration spécifiques à chaque système ; fuzzing nocturne ; rapport de couverture du noyau.
- **Release automatisée** : workflow GitHub Actions déclenché par une étiquette `vMAJEUR.MINEUR.PATCH` ; compilation reproductible dans la mesure du possible (`--locked`, version de Rust fixée), signature Authenticode, génération de `SHA256SUMS`, publication de la release GitHub avec notes issues du changelog.
- **Formats** : installateur par système — MSI (WiX) sous Windows, `.deb` et `.rpm` sous Linux, `.pkg` sous macOS — plus une archive portable pour la GUI et la CLI seules (sans service : mode lecture et opérations utilisateur uniquement). Chaque désinstallation retire service, tâches et règles réseau.
- **Launcher** : entrée dans `apps.json` ; le nom du paquet figure au manifeste pour que la version installée soit détectée via le registre de désinstallation.
- **Confiance** : les API système utilisées sont listées publiquement dans `docs/api-utilisees.md` ; les binaires sont soumis aux principaux éditeurs antivirus et à l'analyse de Microsoft avant chaque version majeure.
- **Versionnage** : SemVer ; le protocole IPC a son propre numéro, compatible sur une version mineure.

## 12. Documentation à produire

| Document | Contenu | Public |
| --- | --- | --- |
| `README.md` | État honnête (ce qui marche, ce qui manque), compilation, commandes | Tous |
| `docs/adr/` | ADR-001 à ADR-008 puis suivants | Contributeurs |
| `docs/spikes/` | Résultats des expérimentations matérielles (P0) | Contributeurs |
| `docs/mesures/` | Campagnes de mesure et valeurs par défaut justifiées | Tous |
| `docs/runbooks/` | Installation du service, restauration réseau, dépannage, désinstallation | Utilisateurs avancés |
| `docs/api-utilisees.md` | Liste des API Windows, privilèges, versions, effets | Tous (confiance) |
| `docs/securite.md` | Modèle de menaces, signalement de vulnérabilités | Tous |
| `CLAUDE.md`, `AGENTS.md` | Règles pour les agents de code, comme dans les autres dépôts | Contributeurs |
| `CONTRIBUTING.md`, `CHANGELOG.md` | Processus, historique | Contributeurs |

## 13. Gouvernance du projet

- Branche `main` protégée ; fusion par demande de tirage avec CI verte sur les trois systèmes ; revue obligatoire pour tout code touchant `obliniscor-sys`, la suppression de fichiers et le pare-feu.
- Une décision structurante = un ADR ; un ADR remplacé n'est pas supprimé mais marqué obsolète.
- Toute exigence `FR-*` est reliée à au moins une tâche `OBL-*` et à un test ; une matrice de traçabilité est générée par `xtask`.
- Règle clean room (ADR-001) : les contributeurs déclarent n'avoir pas recopié de code GPL ; le journal des sources est tenu à jour dans les ADR et les commentaires.

## 14. Cahier des tâches

### 14.1 Mode d'emploi et vue d'ensemble

Les estimations sont en **jours-personne** d'un ingénieur expérimenté en Rust et en programmation système Windows, hors imprévus. Elles couvrent les tâches de priorité **M** et **S** ; les tâches **C** sont listées à part et ne comptent pas dans le total. Colonnes : `Dép.` = dépendances, `j` = jours-personne.

| Phase | Objet | Charge (j) | Jalon de sortie |
| --- | --- | --- | --- |
| P0 | Cadrage et validation technique | 15 | **J0** : go/no-go, hypothèses « à valider » levées |
| P1 | Fondations : socle multi-plateforme, service, IPC, CLI squelette | 21 | **J1** : service installable, `version` et `config` via IPC, sur les trois systèmes |
| P2 | Module Mémoire | 22 | **J2 (alpha)** : `obliniscor mem clean` avec mesure avant/après |
| P3 | Module Processus | 15 | **J3** : contrôle des processus en CLI |
| P4 | Interface egui | 15 | **J4 (bêta 1)** : GUI mémoire et processus |
| P5 | Module Stockage | 19 | **J5** : nettoyage sûr, aperçu obligatoire |
| P6 | Module Réseau | 23 | **J6 (bêta 2)** : blocage par application, restauration garantie |
| P6b | Backends Linux et macOS | 45 | **J6b** : les trois systèmes couverts, disponibilités de la matrice 6.0 respectées |
| P7 | Durcissement et validation | 12,5 | **J7 (RC)** : revue sécurité, fuzzing, matrice A–H verte |
| P8 | Distribution et version 1.0 | 14,5 | **J8** : v1.0.0 signée, empaquetée pour les trois systèmes, publiée via le launcher |
| **Total** |  | **≈ 202** | ≈ 9,5 mois pour une personne à temps plein, ≈ 11,5 mois avec 20 % de marge ; 87 tâches |

### 14.2 Phase P0 — Cadrage et validation technique (15 j)

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-001 | Rédiger ADR-001 (clean room, licence) et la procédure de journal des sources ; obtenir un avis sur la compatibilité de licence | — | M | 1 | ADR fusionné ; checklist de contribution en place |
| OBL-002 | Vérifier la disponibilité du nom (crates.io, GitHub, marques) et réserver les noms de crates | — | M | 0,5 | Rapport de vérification ; noms réservés |
| OBL-003 | Créer le dépôt : espace de travail, `.cargo`, `deny.toml`, `clippy.toml`, `rustfmt.toml`, licences, `CLAUDE.md`, `AGENTS.md`, `CONTRIBUTING.md`, `xtask ci` | OBL-001 | M | 2 | `cargo xtask ci` vert sur un projet vide |
| OBL-004 | CI GitHub Actions : Windows x64 (tests) et ARM64 (compilation) | OBL-003 | M | 1,5 | Pipelines verts ; durée < 15 min |
| OBL-005 | **Spike mémoire** : expérimenter lecture des listes et chaque zone (working set, standby ×2, modifiée, combinée, cache de fichiers, registre) sur Windows 10 et 11 ; privilèges, structures, valeurs de retour, effets mesurés | OBL-003 | M | 3 | `docs/spikes/memoire.md` : tableau zone × build avec résultat observé et mesures |
| OBL-006 | **Spike stockage** : détection du type de média, volumes sur RAID et virtuels, TRIM, points de reparse | OBL-003 | M | 1,5 | `docs/spikes/stockage.md` ; comportements consignés |
| OBL-007 | **Spike pare-feu** : création et suppression de règles par COM, ciblage par chemin, service et paquet du Store, effet des stratégies de groupe et d'un pare-feu tiers | OBL-003 | M | 2 | `docs/spikes/pare-feu.md` ; règles retirées à 100 % |
| OBL-008 | **Spike service et canal nommé** : ACL, instance unique, refus des clients distants, identité de l'appelant | OBL-003 | M | 1,5 | Démonstration exécutable ; tests d'abus (client non autorisé refusé) |
| OBL-009 | Protocole de mesure (scénario 10.3), scripts et préparation des machines A–H | OBL-003 | M | 1,5 | Scripts versionnés ; machines documentées |
| OBL-010 | Revue de fin de phase : lever ou reformuler chaque hypothèse « à valider » ; mise à jour de ce document | OBL-005 à 009 | M | 0,5 | Document v0.2 publié ; décision go/no-go consignée |

### 14.3 Phase P1 — Fondations (21 j)

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-011 | `obliniscor-core` : modèles, erreurs typées, codes stables | OBL-010 | M | 2 | Tests unitaires ; `forbid(unsafe_code)` |
| OBL-012 | `obliniscor-platform` : traits et doublures de test | OBL-011 | M | 1,5 | Noyau testable sans Windows |
| OBL-013 | `obliniscor-proto` : messages, cadrage, versions, limites ; tests de propriétés et premier fuzz | OBL-011 | M | 2,5 | Aucun plantage au fuzz ; compatibilité de version testée |
| OBL-014 | `windows::privilege` : garde RAII d'activation et de retrait de privilège | OBL-012 | M | 1 | Privilège retiré même en cas d'erreur ou de panique |
| OBL-015 | Intégration de `lpdnsoft-log`, `settings`, `i18n`, `atomic` ; schéma de réglages v1 et migrations | OBL-011 | M | 2 | Réglages relus après redémarrage ; migration testée |
| OBL-016 | `obliniscor-store` : historique JSONL, rotation, lecture paginée | OBL-015 | S | 1,5 | 100 000 entrées lues < 200 ms |
| OBL-017 | Service : installation, désinstallation, reprise sur échec, événements d'alimentation | OBL-008, OBL-014 | M | 3 | Installation sans redémarrage ; reprise correcte après veille et hibernation |
| OBL-018 | Serveur IPC et client : poignée de main, ACL, identité de l'appelant, autorisation par catégorie | OBL-013, OBL-017 | M | 3 | Client non autorisé refusé ; latence < 50 ms |
| OBL-019 | CLI squelette : `version`, `service`, `config`, `--json`, codes de sortie | OBL-018 | M | 1,5 | Sortie JSON stable ; codes documentés |
| OBL-100 | **Socle multi-plateforme** : `obliniscor-sys` avec un module par système, CI sur les trois, compilation croisée ARM64/aarch64, packaging continu des artefacts | OBL-011 | M | 3 | `cargo xtask ci` vert sur Windows, Linux et macOS ; un trait de `platform` a au moins une implémentation réelle par système |

### 14.4 Phase P2 — Module Mémoire (22 j)

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-020 | `mem::probe` : état physique, commit, fichier d'échange, listes, mémoire compressée | OBL-005, OBL-012 | M | 3 | Écart < 2 % avec les compteurs de l'outil Moniteur de ressources sur A, B, C |
| OBL-021 | Compteurs de défauts de page et de pression disque (PDH ou équivalent) | OBL-020 | M | 1,5 | Valeurs cohérentes avec l'outil système |
| OBL-022 | `mem::clean` : working set, cache système, liste modifiée | OBL-014, OBL-020 | M | 3 | Gain mesuré et journalisé ; aucun gel de l'interface |
| OBL-023 | `mem::clean` : standby (basse priorité et complète), pages combinées, cache de fichiers modifiés, cache du registre ; détection par build | OBL-022 | M | 3,5 | Zone indisponible grisée avec explication ; aucune erreur fatale |
| OBL-024 | Moteur de mesure avant/après et indicateur de coût (défauts durs à 30 s) | OBL-021, OBL-023 | M | 2 | Entrée d'historique complète ; test sur machine A |
| OBL-025 | Politiques : seuils, hystérésis, délai minimal, plafond horaire, garde-fous (plein écran, E/S, batterie) | OBL-024 | S | 3 | Test de simulation : aucune boucle de purge sur 24 h de scénario |
| OBL-026 | Exclusions de processus pour la réduction de working set | OBL-022 | M | 1 | Processus exclu jamais touché (test) |
| OBL-027 | CLI \`mem status | clean | history`avec`--dry-run`et`--json\` | OBL-018, OBL-024 | M |
| OBL-028 | Campagne de mesures (10.3) sur A, B, C ; réglage des valeurs par défaut | OBL-024, OBL-009 | M | 3 | `docs/mesures/memoire.md` publié ; défauts justifiés |

### 14.5 Phase P3 — Module Processus (15 j)

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-030 | `proc::enum` : métriques, CPU normalisé, signature, arbre | OBL-012 | M | 3 | 500 processus listés en < 100 ms ; CPU conforme à l'outil système |
| OBL-031 | Réduction de working set ciblée, priorité, règles persistantes par chemin signé | OBL-030, OBL-018 | M | 2 | Règle réappliquée après redémarrage du processus |
| OBL-032 | Liste de processus protégés (noyau immuable plus extensions) | OBL-030 | M | 1,5 | Aucune action possible sur un processus protégé (test) |
| OBL-033 | Arrêt en deux temps avec confirmation | OBL-032 | M | 1,5 | Arrêt forcé impossible sans confirmation dans le message |
| OBL-034 | EcoQoS et règles d'arrière-plan | OBL-031 | S | 2,5 | Effet vérifié sur machine C (processeur hybride) |
| OBL-035 | Détection de dérive mémoire et alerte | OBL-030 | S | 1,5 | Fuite simulée détectée ; faux positifs < 5 % sur scénario normal |
| OBL-036 | Services : liste, état d'origine mémorisé, rétablissement | OBL-018 | S | 2 | Retour à l'état initial vérifié |
| OBL-037 | CLI `proc` | OBL-031 à 033 | M | 1 | Scénarios scriptés |

*Tâches C de la phase : OBL-038 (programmes au démarrage, réversible, 2,5 j), OBL-039 (affinité et groupes de plus de 64 processeurs, 2 j).*

### 14.6 Phase P4 — Interface egui (15 j)

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-040 | Squelette egui, thème `lpdnsoft-ui`, navigation par onglets, connexion au service avec mode dégradé | OBL-018 | M | 2 | Fenêtre s'ouvre en < 1 s ; message clair si le service est absent |
| OBL-041 | Tableau de bord et onglet Mémoire : jauges, listes, optimisation, résultat avant/après, historique | OBL-024, OBL-040 | M | 3 | Parcours S1 réalisable ; aucun gel > 100 ms |
| OBL-042 | Onglet Processus : tableau virtualisé, arbre, actions confirmées | OBL-031 à 033, OBL-040 | M | 3 | 500 lignes fluides ; confirmation pour toute action destructive |
| OBL-043 | Zone de notification (usage, seuils de couleur), notifications avec raison et gain, raccourci global avec détection de conflit | OBL-041 | S | 2,5 | Conflit de raccourci signalé ; icône mise à jour sans sondage actif |
| OBL-044 | Mode compact, réglages, journal | OBL-041 | S | 2 | Réglages persistés ; journal filtrable |
| OBL-045 | i18n français et anglais complet, vérification à la compilation | OBL-040 | M | 1 | Aucune chaîne en dur (build échoue sinon) |
| OBL-046 | Accessibilité, DPI 100 à 250 %, contraste AA, navigation clavier | OBL-041, OBL-042 | M | 1,5 | Liste de contrôle NFR-A11Y-1 signée |

### 14.7 Phase P5 — Module Stockage (19 j)

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-050 | `disk::inventory` : volumes, type de média, capacité, santé si exposée | OBL-006 | M | 2 | SSD, NVMe et HDD correctement identifiés sur A, B ; limites sur RAID documentées |
| OBL-051 | Analyse d'espace parallèle et interruptible | OBL-050 | M | 3 | 1 To analysé sans bloquer l'interface ; annulation < 1 s |
| OBL-052 | Catalogue de cibles : schéma, validation, signature, cibles de la v1 | OBL-015 | M | 3 | Catalogue invalide refusé au chargement ; chaque cible a risque et explication |
| OBL-053 | **Suppression sûre** : canonicalisation, refus des points de reparse, handle de dossier, revalidation à l'exécution | OBL-052 | M | 4 | Suite « arborescences pièges » : 0 suppression hors racine |
| OBL-054 | Aperçu, jeton à usage unique lié à l'empreinte de la liste, confirmation, journal des suppressions | OBL-053, OBL-018 | M | 2 | Exécution impossible sans aperçu préalable ; journal complet |
| OBL-055 | Corbeille, TRIM (SSD), défragmentation (HDD seulement) | OBL-050 | S | 2 | Aucune défragmentation sur SSD (test) |
| OBL-056 | Onglet Stockage et CLI `disk` | OBL-054, OBL-040 | M | 3 | Parcours S4 réalisable ; sortie JSON |

*Tâches C de la phase : OBL-057 (doublons par hachage, lecture seule, 3 j), OBL-058 (lecture directe de la MFT, 4 j).*

### 14.8 Phase P6 — Module Réseau (23 j)

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-060 | `net::inventory` : connexions avec PID et débit par processus (événements noyau réseau) | OBL-012 | M | 3,5 | Débit conforme à l'outil système à ± 5 % ; coût CPU < 0,5 % |
| OBL-061 | `FirewallControl` : création, lecture, suppression du groupe de règles « obliNiscor » | OBL-007 | M | 3 | Création puis retrait : 0 règle résiduelle |
| OBL-062 | Identification des applications : chemin signé, nom de service, paquet du Store | OBL-061 | M | 2,5 | Règle sur un service seul ne bloque pas les autres services du même hôte |
| OBL-063 | Profils (Normal, Silencieux, Strict, Hors ligne) et liste d'immunité (loopback, DNS, DHCP, ankou, obliNiscor) | OBL-062 | M | 2,5 | Profil Strict : DNS et ankou toujours fonctionnels |
| OBL-064 | Filet de sécurité : annulation à 60 s, `net restore`, réconciliation de l'état au démarrage du service | OBL-061, OBL-017 | M | 2 | Coupure du service pendant l'application : état restauré au redémarrage |
| OBL-065 | Détection d'environnement : stratégie de groupe, pare-feu tiers, pare-feu désactivé | OBL-061 | M | 1,5 | Avertissement explicite dans tous les cas simulés |
| OBL-066 | Services Windows liés au réseau : état d'origine, rétablissement, gestion des relances | OBL-036 | S | 2 | Service relancé par déclencheur : détecté et signalé |
| OBL-067 | Planification (plages horaires, session de jeu) | OBL-063, OBL-025 | S | 1,5 | Règles appliquées et levées aux heures prévues |
| OBL-068 | Onglet Réseau et CLI `net` | OBL-063, OBL-040 | M | 3 | Parcours S3 réalisable ; confirmation pour les profils larges |
| OBL-069 | Tests de coexistence avec ankou et un pare-feu tiers (machine F) | OBL-063 | M | 1,5 | Tunnel WireGuard intact sous tous les profils |

*Tâche C : OBL-070 (mode WFP : fournisseur, sous-couche, filtres dynamiques, 6 j).*

### 14.9 Phase P6b — Backends Linux et macOS (45 j)

Chaque backend implémente les traits de `obliniscor-platform` déjà figés en P1 : rien du noyau ne change, et la matrice 6.0 dit ce qui est complet, partiel ou indisponible. Les tâches sont regroupées par système pour qu'une personne puisse travailler sur Linux pendant qu'une autre traite macOS.

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-101 | Linux `mem` : lecture `/proc/meminfo` et `/proc/vmstat`, conversion en `Snapshot`, tests sur fixtures | OBL-100, OBL-020 | M | 3 | Écart < 2 % avec `free -b` et `smem` sur la machine G |
| OBL-102 | Linux `mem` : nettoyage (`drop_caches`, `clear_refs`), garde de privilège, avertissements d'usage | OBL-101, OBL-014 | M | 4 | Désactivé par défaut ; avertissement explicite ; privilège rendu même en cas d'erreur |
| OBL-103 | Linux `proc` : énumération `/proc`, `nice`, cgroups v2 pour la pondération, arrêt en deux temps, protections | OBL-100, OBL-030 | M | 5 | 500 processus en < 100 ms ; processus protégés intouchables ; priorité restaurée |
| OBL-104 | Linux `disk` : volumes par `/sys` et `statfs`, type de média (`rotational`), suppression sûre `openat2`/`O_NOFOLLOW`, `fstrim` | OBL-100, OBL-053 | M | 6 | Suite « arborescences pièges » : 0 suppression hors racine ; aucune défragmentation tentée |
| OBL-105 | Linux `net` : nftables par cgroup, profils, immunité, restauration, détection d'environnement | OBL-100, OBL-063 | M | 7 | Tunnel WireGuard intact sous tous les profils ; `net restore` : 0 règle résiduelle |
| OBL-106 | Linux `svc` : unité systemd durcie, socket Unix, `SO_PEERCRED`, installation et désinstallation | OBL-100, OBL-018 | M | 4 | Client non autorisé refusé ; après désinstallation, 0 unité et 0 fichier |
| OBL-107 | macOS `mem` et `proc` : `host_statistics64`, `libproc`, `nice`, protections ; disponibilité déclarée | OBL-100, OBL-020 | M | 7 | Mesures cohérentes avec `vm_stat` et `ps` ; nettoyage mémoire déclaré indisponible, pas simulé |
| OBL-108 | macOS `disk` : `statfs`, `diskutil`, suppression sûre `O_NOFOLLOW` | OBL-100, OBL-053 | M | 5 | Suite pièges : 0 suppression hors racine ; APFS non défragmenté |
| OBL-109 | macOS `svc` : launchd, socket Unix, `LOCAL_PEERPID`, installation et désinstallation | OBL-100, OBL-018 | M | 4 | Client non autorisé refusé ; désinstallation propre sans droit administrateur non accordé |

*Hors périmètre de cette phase, et documenté comme tel : nettoyage mémoire sous macOS (aucune API publique) et blocage réseau par application sous macOS (extension système Apple requise).*
### 14.10 Phase P7 — Durcissement et validation (12,5 j)

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-080 | Revue de sécurité : menaces de la section 9, relecture de tous les blocs `unsafe` | P2 à P6 | M | 3 | Rapport sans écart critique ouvert |
| OBL-081 | Fuzzing étendu (IPC, catalogue, noms de fichiers) et tests de chaos | OBL-013, OBL-053, OBL-064 | M | 2,5 | 24 h de fuzz sans plantage ; chaos : états cohérents |
| OBL-082 | Matrice matérielle complète A à H (trois systèmes) ; correction des écarts | P2 à P6 | M | 3 | Matrice verte ou écarts documentés |
| OBL-083 | Profilage de l'empreinte (CPU, mémoire, réveils) | P4 | M | 2 | Cibles NFR-PERF atteintes |
| OBL-084 | Audit des dépendances, génération d'une nomenclature logicielle (SBOM) | P1 | M | 1 | `cargo deny` sans avis ; SBOM publiée |
| OBL-085 | Soumissions aux éditeurs antivirus, test de SmartScreen sur binaire signé | OBL-090 | S | 1 | Aucun blocage sur les principaux produits ou dossier de contestation déposé |

### 14.11 Phase P8 — Distribution et version 1.0 (14,5 j)

| ID | Tâche | Dép. | Prio | j | Critère d'acceptation |
| --- | --- | --- | --- | --- | --- |
| OBL-090 | Signature Authenticode (certificat ou programme de signature pour projets ouverts) | OBL-004 | M | 2 | Signature valide, horodatée ; vérifiable par `Get-AuthenticodeSignature` |
| OBL-091 | Installateur et archive portable ; `SHA256SUMS` ; test de désinstallation propre | OBL-090 | M | 3 | Après désinstallation : 0 service, 0 règle réseau, 0 tâche |
| OBL-092 | Intégration au launcher : `apps.json`, nom du paquet, détection de version | OBL-091 | M | 1,5 | Le launcher installe et met à jour obliNiscor |
| OBL-093 | Documentation utilisateur FR/EN, runbooks, `api-utilisees.md`, `securite.md` | P2 à P6 | M | 2 | Relecture par un utilisateur extérieur |
| OBL-110 | **Installateurs des trois systèmes** : MSI (WiX), `.deb` et `.rpm`, `.pkg` ; script d'audit de résidus par système | OBL-091 | M | 5 | Après désinstallation : 0 service, 0 règle, 0 tâche sur les trois systèmes |
| OBL-094 | Version 1.0.0 : checklist de release, notes de version, étiquette | OBL-080 à 093 | M | 1 | Critères de la section 16 tous satisfaits |

### 14.12 Chemin critique et parallélisation

- **Chemin critique** : P0 → P1 (socle multi-plateforme, service et IPC) → P2 (mémoire) → P3 (processus, dont l'onglet de P4 dépend) → P4 (interface) → P6b (backends) → P7 → P8. Les modules Stockage et Réseau n'ont besoin que de P1 et peuvent avancer en parallèle de P2.
- **Premier jalon utile** : la fin de P2 donne déjà un outil mémoire complet en ligne de commande, avec preuve chiffrée, avant toute interface — d'abord sous Windows, puis sous Linux avec P6b.
- **Ordre de risque** : les spikes P0 (mémoire, pare-feu, service) viennent en premier parce que ce sont eux qui peuvent invalider une hypothèse ; la suppression de fichiers (OBL-053) est la tâche la plus dangereuse et reçoit la plus forte couverture de tests. Les backends Unix (P6b) héritent des mêmes traits, donc du même noyau testé, mais la suppression sûre y est réécrite avec `openat2` : elle est revue comme une tâche neuve, pas comme un portage.
- Avec des agents de code, la mise en place de squelettes, de tests et de documentation peut être accélérée ; les spikes matériels, la revue des blocs `unsafe` et les mesures sur machines réelles restent des étapes humaines.

## 15. Risques et parades

| Réf. | Risque | Proba. | Impact | Parade |
| --- | --- | --- | --- | --- |
| R1 | API noyau non documentées : comportement différent ou rupture selon les builds de Windows | Haute | Haut | Spikes P0, détection par build, repli propre, tests sur matrice, veille des versions Insider |
| R2 | Contamination par du code GPL (licence) | Moyenne | Très haut | Clean room, journal des sources, revue, avis juridique avant publication |
| R3 | Purge mémoire contre-productive (cache perdu, rafales d'E/S) | Haute | Moyen | Mesure du coût, valeurs par défaut prudentes, garde-fous, pas d'automatisme agressif |
| R4 | Suppression de fichiers hors périmètre | Faible | Très haut | Suppression sûre par handle, aperçu obligatoire, tests pièges, revue renforcée |
| R5 | Blocage réseau coupant l'utilisateur (ou ankou) | Moyenne | Haut | Liste d'immunité, annulation à 60 s, `net restore`, réconciliation au démarrage |
| R6 | Faux positifs antivirus et SmartScreen | Haute | Moyen | Signature, transparence des API, soumissions, pas de persistance cachée |
| R7 | Pare-feu géré par stratégie de groupe ou produit tiers | Moyenne | Moyen | Détection (FR-NET-7), message clair, pas de promesse de blocage |
| R8 | Faux sentiment de sécurité des profils « Silencieux » | Moyenne | Moyen | Documentation honnête des limites, listes maintenues et datées |
| R9 | Désactiver des services de mise à jour expose à des failles | Moyenne | Haut | Avertissement, durée limitée par défaut, rappel visible |
| R10 | Surcharge de travail : périmètre très large pour une seule personne | Haute | Haut | Livrer module par module (jalons J2, J3, J5, J6), tâches C reportées |
| R11 | Dérive d'ARM64 (comportements non testés) | Moyenne | Moyen | Machine E réelle avant publication ; sinon ARM64 marqué expérimental |
| R12 | Certificat de signature coûteux ou refusé | Moyenne | Moyen | Programme de signature pour projets ouverts, sinon certificat commercial ; budget prévu |

## 16. Critères d'acceptation de la version 1.0

- [ ] Toutes les exigences **M** sont couvertes par des tests automatisés ou une procédure manuelle documentée, avec la matrice de traçabilité à jour.
- [ ] `cargo xtask ci` est vert ; couverture du noyau ≥ 80 % ; aucun avis `cargo deny`.
- [ ] Les valeurs de la section 2.2 (O1 à O6) sont atteintes et consignées dans `docs/mesures/`.
- [ ] La suite « arborescences pièges » passe : 0 suppression hors racine.
- [ ] `net restore` et la désinstallation retirent 100 % des règles, tâches et services créés, sur chacun des trois systèmes.
- [ ] La matrice 6.0 est respectée : chaque fonction indisponible est annoncée comme telle dans l'interface et dans la documentation, et aucun test ne prétend l'avoir exercée.
- [ ] Aucune action destructive sans aperçu, confirmation et journal.
- [ ] La matrice A à H est verte ou chaque écart est documenté et accepté.
- [ ] Revue de sécurité terminée, sans écart critique ouvert ; tous les blocs `unsafe` relus.
- [ ] Binaires signés ; `SHA256SUMS` publié ; installation et mise à jour par le launcher validées.
- [ ] Français et anglais complets ; navigation clavier et DPI vérifiés.
- [ ] Le README indique honnêtement ce qui est fait et ce qui ne l'est pas.

## 17. Questions ouvertes

1. **Licence** : confirmer la voie clean room (ADR-001) et obtenir un avis ; sinon, accepter la GPL pour obliNiscor.
2. **Réseau v1** : règles du pare-feu Windows seules (recommandé), ou WFP dès la v1 ?
3. **Nom** : disponibilité de « obliNiscor » et de ses variantes (crate, dépôt, marque).
4. **Signature** : certificat commercial ou programme de signature pour projets ouverts ?
5. **Installateur Windows** : MSI classique ou MSIX ? (MSIX complique l'installation d'un service.) Les deux autres systèmes ont leur réponse : `.deb`/`.rpm` et `.pkg`.
6. **Services de mise à jour** : proposer ou non leur désactivation, ou seulement une pause limitée dans le temps ?
7. ~~**Portage** : prévoir dès maintenant une extension Linux ?~~ **Tranché le 31 octobre 2026** : la v1 est multi-plateforme (ADR-004), avec les disponibilités de la matrice 6.0.
8. **Équipe** : une seule personne ou plusieurs ? Cela change l'ordonnancement de la section 14.11.

## 18. Glossaire

| Terme | Définition |
| --- | --- |
| ADR | Fiche de décision d'architecture, versionnée avec le code |
| ALE | Couche de la plateforme de filtrage Windows qui applique les règles par application et par connexion |
| cgroups v2 | Groupes de contrôle Linux : hiérarchie qui porte les limites et la pondération des processus (équivalent fonctionnel des classes de priorité) |
| launchd | Gestionnaire de services de macOS, équivalent du SCM Windows et de systemd |
| nftables | Filtre réseau du noyau Linux, piloté par `nft`, utilisé par aNkou et repris ici |
| PF | Filtre réseau de macOS, piloté par `pfctl` |
| socket Unix | Canal local d'Unix, équivalent du canal nommé Windows ; l'identité du client est fournie par le noyau |
| Clean room | Réécriture à partir du comportement et de la documentation, sans reprise du code d'un projet de licence incompatible |
| Commit (charge de) | Quantité de mémoire promise aux processus, bornée par RAM + fichier d'échange |
| EcoQoS | Mode d'efficacité énergétique d'un processus, qui oriente l'ordonnanceur vers les cœurs économes |
| IPC | Communication entre processus (ici un canal nommé) |
| MFT | Table de fichiers maîtresse de NTFS |
| NUMA | Architecture où la mémoire est locale à un nœud de processeurs |
| PDH | Interface de lecture des compteurs de performance de Windows |
| PPL | Processus protégé : les autres processus ne peuvent pas le manipuler librement |
| Point de reparse | Mécanisme NTFS derrière les liens symboliques, jonctions et points de montage |
| Standby (liste) | Pages propres conservées en cache et réutilisables, réparties en huit priorités |
| TOCTOU | Course entre la vérification d'un état et son utilisation |
| TRIM | Commande qui informe un SSD des blocs libérés |
| Working set | Ensemble des pages d'un processus actuellement présentes en RAM |
| WFP | Plateforme de filtrage de Windows, plus fine que l'API du pare-feu |

## 19. Références à consulter

À lire avant les spikes P0 (documentation Microsoft Learn et outils Sysinternals) : gestion de la mémoire et listes de pages ; fonctions `GlobalMemoryStatusEx`, `GetPerformanceInfo`, `EmptyWorkingSet` ; privilèges de jeton et `AdjustTokenPrivileges` ; API du pare-feu Windows avec fonctions avancées (`INetFwPolicy2`) ; Windows Filtering Platform ; canaux nommés et sécurité ; services Windows ; `SetProcessInformation` (limitation d'alimentation) ; propriétés de périphérique de stockage (pénalité de recherche) ; documentation du programme RAMMap pour l'observation des listes. Côté Unix, à lire avant P6b : `proc(5)` et `vmstat(8)` pour Linux ; `openat2(2)` et `RESOLVE_BENEATH` ; `process_madvise(2)` ; `fstrim(8)` et l'ioctl `FITRIM` ; `nft(8)` et les correspondances cgroups ; `systemd.exec(5)` pour le durcissement d'unité ; `SO_PEERCRED` dans `unix(7)`. Côté macOS : `host_statistics64`, `libproc.h`, `launchd.plist(5)`, `pfctl(8)`, et la documentation Apple sur les Network Extensions — pour constater ce qui **n'est pas** accessible à une application indépendante.

Les liens exacts et les numéros de version sont à consigner dans `docs/spikes/` lors de la phase P0 et de P6b, car plusieurs interfaces citées sont partiellement documentées.
