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

| Volume | Type | FS | Total | Libre | `incurs_seek_penalty` | `trim_enabled` |
| --- | --- | --- | --- | --- | --- | --- |
| `C:\` | fixe | NTFS | 631 597 166 592 | 179 672 821 760 | `false` | `true` |
| `N:\` | fixe | NTFS | 390 840 971 264 | 210 188 705 792 | `false` | `true` |

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

## Ce qui reste incertain

- **SSD et NVMe sont indiscernables par la seule pénalité de recherche** : les deux rendent
  `false`. Pour les distinguer (`FR-STO-1`), il faut le type de bus
  (`StorageDeviceProperty` → `BusType`, `BusTypeNvme = 17`). Non mesuré ici.
- **Volumes virtuels, RAID et réseau** : aucun sur cette machine. Ce sont eux qui peuvent
  mentir sur la pénalité de recherche ; le spike ne peut pas les juger sans eux.
- **Points de reparse** (jonctions, liens symboliques) : non testés. C'est le cœur de
  OBL-053, la tâche la plus dangereuse du projet, et elle mérite son propre spike.
- **Défragmentation et TRIM effectifs** : non exécutés, et ils ne le seront pas sans votre
  accord — ce sont des écritures.

## Conséquences pour le cahier des charges

- `FR-STO-1` doit demander **deux** informations, pas une : pénalité de recherche (SSD ou
  HDD) **et** type de bus (NVMe ou SATA). Sans la seconde, l'interface ne peut pas afficher
  « NVMe ».
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
