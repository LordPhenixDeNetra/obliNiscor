# Spike — stockage (OBL-006)

- **Date :** 2026-11-01
- **Machine :** Windows 11, build 10.0.26200, x64, deux volumes NTFS
- **Outil :** `spikes/stockage` (binaire jetable, hors workspace, lecture seule)
- **État :** **partiel** — Windows mesuré ; Linux (`/sys/block/*/queue/rotational`, `fstrim`) et
  macOS (`statfs`, `diskutil`) à faire

## Question

1. La propriété « pénalité de recherche » distingue-t-elle réellement HDD, SSD et NVMe ?
2. TRIM est-il exposé, et sous quelle forme ?
3. Quels volumes sont « fixes », et lesquels pourraient mentir (virtuels, réseau) ?

## Méthode

```sh
cd spikes/stockage
cargo run --release
```

Le binaire énumère les racines par `GetLogicalDriveStringsW`, lit type, système de fichiers et
espace par `GetVolumeInformationW` et `GetDiskFreeSpaceExW`, puis interroge chaque volume par
`DeviceIoControl(IOCTL_STORAGE_QUERY_PROPERTY)` pour deux propriétés :
`StorageDeviceSeekPenaltyProperty` (7) et `StorageDeviceTrimProperty` (8). **Aucune écriture,
aucune suppression, aucune défragmentation.**

## Résultats bruts

| Volume | Type | FS | Total | Libre | `incurs_seek_penalty` | `trim_enabled` | Bus | Modèle |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `C:\` | fixe | NTFS | 631 597 166 592 | 176 635 101 184 | `false` | `true` | **`raid`** | `SAMSUNG MZVLB1T0HBLR-000H1` |
| `N:\` | fixe | NTFS | 390 840 971 264 | 209 669 140 480 | `false` | `true` | **`raid`** | idem |

Le modèle est un NVMe (Samsung PM981), mais le bus annoncé est **`raid`** : le disque est
derrière un contrôleur RAID logiciel (Intel RST / VMD), ce qui est le cas par défaut sur
beaucoup de portables et de cartes mères récentes.

## Ce que le spike a établi

1. **La propriété de pénalité de recherche répond, et elle est exploitable.** Les deux volumes
   rendent `false` : ce sont des supports sans déplacement de tête (SSD ou NVMe). La règle
   « jamais de défragmentation sur un support sans pénalité » (FR-STO-9) peut donc s'appuyer
   sur cette propriété, et pas sur une devinette.
2. **TRIM est exposé séparément, et vaut `true`.** `StorageDeviceTrimProperty` donne une
   réponse directe : inutile de déduire la capacité de ré-optimisation du type de média.
3. **Ouvrir un volume ne demande aucun droit — si l'on demande l'accès zéro.** Avec
   `GENERIC_READ`, `CreateFile("\\.\C:")` échoue en **erreur 5 (accès refusé)** sur les deux
   volumes, et les deux propriétés reviennent `null`. Avec un accès `0`, tout fonctionne. Un
   code qui aurait utilisé `GENERIC_READ` aurait donc rendu « type inconnu » pour un simple
   utilisateur, sans erreur visible — c'est le piège le plus coûteux de ce spike.
4. **`GetLogicalDriveStringsW` ne rend que les racines à lettre.** Un volume monté dans un
   dossier (point de montage) n'y figure pas ; `FR-STO-1` demande pourtant « lettre **ou** point
   de montage ». Il faudra `FindFirstVolume` / `GetVolumePathNamesForVolumeName` pour les voir.
5. **Le type de bus ne dit pas NVMe quand un contrôleur RAID s'interpose.** `BusType` rend
   `raid` sur cette machine, alors que le disque est un NVMe Samsung. Ma conclusion du premier
   passage — « il suffit d'ajouter `BusType` pour distinguer NVMe de SATA » — était donc
   fausse : sous RST/VMD, le bus décrit le **contrôleur**, pas le disque.
6. **Les chaînes de `STORAGE_DEVICE_DESCRIPTOR` sont en ASCII, pas en UTF-16.** Ma première
   lecture affichait `䅓卍乕⁇婍䱖…` : un octet sur deux pris pour un caractère. Après correction,
   le même champ donne `SAMSUNG MZVLB1T0HBLR-000H1`. Le nom du disque est lisible — mais il
   n'est pas normalisé : reconnaître « NVMe » par le modèle est une **heuristique**, pas une
   garantie.

## Ce qui reste incertain

- **Distinguer NVMe de SATA reste un problème ouvert.** Ni la pénalité de recherche (les deux
  rendent `false`), ni le type de bus (il dit `raid`), ni le modèle (texte libre) ne donnent
  une réponse fiable sur une machine à contrôleur RAID. La piste sérieuse est
  `IOCTL_STORAGE_QUERY_PROPERTY` avec `StorageAdapterProperty` **et** l'énumération des
  périphériques physiques (`SetupDiGetDeviceRegistryProperty`), hors périmètre de ce spike.
- **Volumes virtuels et réseau** : aucun sur cette machine. Ce sont eux qui peuvent mentir sur
  la pénalité de recherche ; le spike ne peut pas les juger sans eux.
- **Points de reparse** (jonctions, liens symboliques) : non testés. C'est le cœur de
  OBL-053, la tâche la plus dangereuse du projet, et elle mérite son propre spike.
- **Défragmentation et TRIM effectifs** : non exécutés, et ils ne le seront pas sans votre
  accord — ce sont des écritures.

## macOS — deuxième passage

- **Machine :** MacBook Intel (i5-7360U), macOS 13.7.8, APFS sur un SSD de 121 Go
- **Mesuré par :** l'utilisateur, en compte normal, sans `sudo`
- **État :** partiel — le TRIM n'est pas exposé par `diskutil info` sur ce volume APFS

### Résultats bruts

| Élément | Valeur | Source |
| --- | --- | --- |
| Disque physique | `disk0`, 121,3 Go, interne | `diskutil list` |
| Conteneur APFS | `disk1`, 121,0 Go, **synthétisé** | `diskutil list` |
| Volume système | `disk1s5s1`, snapshot APFS, 9,3 Go | `diskutil list` |
| Type de média | **Solid State: Yes** | `diskutil info /` |
| Système de fichiers | APFS | `diskutil info /` |
| Amovible | Non (`Fixed`) | `diskutil info /` |
| Privilège demandé | **aucun** | les trois commandes passent en utilisateur |

### Ce que macOS a établi

7. **Le type de média est donné sans privilège.** `diskutil info` répond `Solid State: Yes` à
   un compte normal : `FR-STO-1` est satisfait sous macOS, sans équivalent du piège Windows de
   l'accès `GENERIC_READ`.
8. **`df` ment sur APFS, et c'est structurel.** Les six volumes du conteneur affichent tous
   **118 181 844 blocs** — la taille du conteneur, pas la leur. Les volumes APFS **partagent**
   leur espace : afficher « 4406988 blocs libres » six fois ferait croire à six fois plus de
   place. L'interface doit montrer l'espace du **conteneur**, une fois, et rattacher les volumes
   à lui.
9. **Les volumes sont hiérarchisés : physique, conteneur, volumes.** `disk0` (disque) contient
   `disk1` (conteneur synthétisé), qui contient `disk1s1`…`disk1s6`. Une énumération qui
   traite les trois niveaux comme des volumes distincts compte trois fois le même disque.
10. **Le TRIM n'est pas visible sur APFS par `diskutil info`.** Le cas HFS+ l'affiche ; ici,
    APFS gère la ré-optimisation en interne. `FR-STO-9` ne peut donc pas s'appuyer sur ce champ
    sous macOS : il faut soit `system_profiler SPStorageDataType`, soit considérer qu'APFS sur
    SSD implique la ré-optimisation.

### Résultats complémentaires (`system_profiler`)

| Élément | Valeur |
| --- | --- |
| Disque | `APPLE SSD SM0128L` |
| **Type de média** | **SSD** (`Medium Type: SSD`) |
| **Bus** | **PCI-Express** (`Protocol: PCI-Express`) |
| Interne | Oui |
| S.M.A.R.T. | *Verified* |
| TRIM | **absent** de la sortie, au niveau disque comme au niveau volume |
| Espace | **Free 4,29 Go / Capacity 121,02 Go — identiques pour les deux volumes** |

11. **`system_profiler` répond à `FR-STO-1` sous macOS, et mieux que Windows.** Il donne à la
    fois le **type de média** (`SSD`) et le **bus** (`PCI-Express`), là où Windows annonce `raid`
    derrière un contrôleur RST et ne sait plus distinguer NVMe de SATA. Sous macOS, la question
    « SSD ou disque dur, et sur quel bus » a une réponse directe.
12. **Le TRIM reste invisible, y compris au niveau du disque physique.** `diskutil info
    /dev/disk0` n'affiche que `Solid State: Yes`. `FR-STO-9` ne peut donc pas s'appuyer sur un
    champ TRIM sous macOS : la règle devra être « APFS sur SSD implique la ré-optimisation
    interne », écrite comme telle.
13. **Le conteneur partagé est confirmé par une seconde source.** `system_profiler` annonce
    `Free: 4,29 Go` et `Capacity: 121,02 Go` pour **les deux** volumes — exactement comme `df`.
    Ce n'est donc pas un artefact de `df` : c'est le modèle APFS, et l'interface doit le refléter.

### Ce qui manque encore pour clore macOS

- **Apple Silicon** : comme pour la mémoire, la machine mesurée est un Intel.
- **Un disque externe ou un HDD** : aucun support à pénalité de recherche n'était disponible, donc
  la branche « disque dur » de `FR-STO-9` n'a pas été exercée sous macOS.

## Conséquences pour le cahier des charges

- `FR-STO-1` doit demander **trois** informations, pas deux : pénalité de recherche
  (SSD ou HDD), type de bus, **et** modèle du disque — parce que le bus seul ne suffit pas
  sous contrôleur RAID. L'interface affichera « NVMe, d'après le modèle » plutôt que de
  prétendre le savoir.
- **Correction du premier passage** : ma recommandation « ajouter `BusType` pour nommer NVMe »
  était erronée sur une machine RST/VMD, c'est-à-dire sur la majorité des portables récents.
  Une décision d'architecture ne doit pas reposer sur ce champ seul.
- La règle « jamais de défragmentation sur SSD » gagne un garde-fou supplémentaire : refuser
  aussi quand la propriété est **indisponible** (volume virtuel, accès refusé), plutôt que de
  supposer un HDD.
- L'énumération des volumes doit passer par `FindFirstVolume`, pas par les lettres.

## Sources

- Microsoft Learn, `IOCTL_STORAGE_QUERY_PROPERTY`, `STORAGE_PROPERTY_QUERY`,
  `DEVICE_SEEK_PENALTY_DESCRIPTOR`, `DEVICE_TRIM_DESCRIPTOR` (consulté le 2026-11-01).
- Microsoft Learn, `GetLogicalDriveStringsW`, `GetVolumeInformationW`, `GetDiskFreeSpaceExW`.
- `CreateFileW` : la valeur `dwDesiredAccess = 0` est documentée comme « interroger les
  attributs sans accéder aux données » ; c'est exactement ce que le spike confirme.
- Aucun code de projet tiers consulté (règle clean room, ADR-001).
