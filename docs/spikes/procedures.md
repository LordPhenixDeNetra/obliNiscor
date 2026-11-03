# Procédures de mesure — machines que le développement n'a pas

- **Date :** 2026-11-01
- **Portée :** OBL-005 (mémoire) et OBL-006 (stockage), machines manquantes
- **Pourquoi ce document :** les spikes sont mesurés sur Windows 11 et Linux. Il manque
  **macOS**, **Windows 10 22H2**, et les mesures Linux de stockage. Chaque procédure tient en
  quelques commandes, et aucune ne modifie le système.

## Ce qu'il faut faire des résultats

Copier la sortie brute dans le rapport de spike correspondant (`docs/spikes/memoire.md`,
`docs/spikes/stockage.md`), dans une section datée. Ne pas arrondir, ne pas résumer : le
rapport publie les nombres, l'interprétation vient après.

---

## 1. macOS — mémoire et stockage (OBL-005, OBL-006)

Sur un Mac (Intel ou Apple Silicon, macOS 12 ou plus récent) :

```sh
# Version réellement exécutée
sw_vers
sysctl -n machdep.cpu.brand_string

# Mémoire, deux mesures espacées de 2 s
vm_stat 1 2
sysctl -n hw.memsize
sysctl -n hw.pagesize

# Pression mémoire (macOS 10.9+)
memory_pressure -Q

# Volumes : type, montage, espace
df -k
diskutil list
diskutil info / | grep -E 'Solid State|TRIM|File System|Volume Name'
```

**Ce que je cherche dans la sortie :**

| Question | Réponse attendue |
| --- | --- |
| `host_statistics64` est-il exploitable par `vm_stat` ? | oui, les pages actives/inactives/wired/compressor |
| macOS mesure-t-il sans droit particulier ? | oui, aucune commande ne demande `sudo` |
| Peut-on purger ? | **non** — vérifier que `purge` existe mais qu'il est documenté comme outil de diagnostic Apple |
| TRIM est-il exposé par volume ? | `diskutil info` doit le dire |
| Type de média | `Solid State: Yes` |

**Point d'attention :** `memory_pressure -Q` peut demander un instant ; ne pas le confondre avec
une mesure instantanée.

---

## 2. Windows 10 22H2 (OBL-005, OBL-006)

Sur une machine Windows 10 22H2 (build 19045), **en utilisateur normal** :

```bat
REM État mémoire, trois mesures espacées d'une seconde
wmic OS get Version,FreePhysicalMemory,TotalVisibleMemorySize /value
powershell -NoProfile -Command "1..3 | %% { Get-Counter '\Memory\Available MBytes','\Memory\Cache Bytes','\Memory\Committed Bytes','\Memory\Commit Limit' | Select-Object -ExpandProperty CounterSamples | Select-Object Path,CookedValue; Start-Sleep 1 }"

REM Volumes et type de média
powershell -NoProfile -Command "Get-Volume | Select-Object DriveLetter,FileSystem,Size,SizeRemaining,DriveType"
powershell -NoProfile -Command "Get-PhysicalDisk | Select-Object FriendlyName,MediaType,BusType,Size"

REM Défauts de page (coût d'une purge, base de comparaison)
powershell -NoProfile -Command "Get-Counter '\Memory\Pages Input/sec','\Memory\Page Faults/sec' -MaxSamples 5"
```

**Ce que je cherche :**

| Question | Réponse attendue |
| --- | --- |
| Les mêmes API existent-elles en 22H2 ? | oui, mais `RegistryCache` n'existe qu'à partir de 8.1 — vérifier que `GetPerformanceInfo` répond encore |
| `MediaType` de `Get-PhysicalDisk` distingue-t-il SSD de HDD ? | oui, `SSD`/`HDD`/`Unspecified` |
| Le build 19045 change-t-il les zones ? | comparer avec le rapport Windows 11 |
| Les compteurs PDH sont-ils nommés en français ? | **oui sur un Windows français** — c'est le piège `OBL-021` |

**Piège à noter :** `Get-Counter` avec des noms de compteurs anglais **échoue sur un Windows
français**. Si la commande renvoie `Le chemin du compteur de performance est introuvable`,
c'est une information précieuse pour `OBL-021`, pas une erreur de manipulation.

---

## 3. Linux — stockage (OBL-006)

Dans le WSL existant, ou sur une distribution quelconque :

```sh
# Type de média de chaque disque
for d in /sys/block/*/queue/rotational; do echo "$d = $(cat "$d")"; done

# Capacité TRIM
lsblk --discard
# ou
sudo fstrim -v /

# Volumes montés et espace
df -hT
findmnt -o TARGET,SOURCE,FSTYPE,SIZE,AVAIL
```

**Ce que je cherche :**

| Question | Réponse attendue |
| --- | --- |
| `rotational` distingue-t-il HDD et SSD ? | `0` = SSD/NVMe, `1` = HDD |
| TRIM est-il visible sans l'exécuter ? | `lsblk --discard` donne `DISC-GRAN`/`DISC-MAX` |
| Un volume monté dans un dossier apparaît-il ? | oui dans `findmnt`, non dans `df` selon les cas |

**Point d'attention :** `fstrim` **modifie** le système (il envoie des commandes TRIM). Le
lancer seulement si vous l'acceptez ; sinon `lsblk --discard` suffit pour la question posée.

---

## 4. Ce qu'il ne faut pas faire dans ces procédures

- **Ne pas purger** les caches mémoire (Windows `EmptyWorkingSet`, Linux `drop_caches`, macOS
  `purge`) : les procédures ci-dessus mesurent l'état, elles ne le modifient pas. Un spike de
  purge viendra séparément, avec un accord explicite et une machine dédiée.
- **Ne pas défragmenter**, ni lancer `Optimize-Volume`, ni `fstrim` sans le vouloir.
- **Ne pas exécuter en administrateur** par défaut : ce que peut faire un utilisateur normal
  fait partie de la réponse.
