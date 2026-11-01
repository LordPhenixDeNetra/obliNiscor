# Spike — mémoire (OBL-005)

- **Date :** 2026-11-01
- **Machine :** celle du développement — Windows 11, build **10.0.26200**, x64, 32 Gio de RAM
- **Outil :** `spikes/memoire` (binaire jetable, hors workspace, lecture seule)
- **État :** **partiel** — Windows 11 x64 et Linux x86_64 mesurés ; Windows 10 et macOS à faire

## Question

Quelles valeurs la documentation permet-elle réellement d'obtenir sur l'état de la mémoire,
sont-elles cohérentes entre elles, et que manque-t-il pour répondre aux exigences `FR-MEM-1`
et `FR-MEM-6` du cahier des charges ?

## Méthode

```sh
cd spikes/memoire
cargo run --release -- --repeat 2 --interval-ms 400
```

Le binaire appelle `GlobalMemoryStatusEx` (kernel32), `GetPerformanceInfo` (psapi) et
`RtlGetVersion` (ntdll), puis imprime une ligne JSON par mesure. Il ne nettoie **rien** : la
purge n'est pas implémentée, précisément pour que le spike ne puisse pas modifier le système
par accident.

## Résultats bruts

Mesure 1, puis mesure 2 à +400 ms :

| Champ | Mesure 1 | Mesure 2 |
| --- | --- | --- |
| `load_pct` | 56 | 56 |
| `total` (octets) | 34 160 984 064 | 34 160 984 064 |
| `available` (octets) | 14 990 217 216 | 14 981 636 096 |
| `commit_available` | 19 718 012 928 | 19 725 996 032 |
| `commit_total` | 30 012 227 584 | 30 004 244 480 |
| `commit_limit` | 49 730 240 512 | 49 730 240 512 |
| `commit_peak` | 39 379 582 976 | 39 379 582 976 |
| `system_cache` | 10 163 122 176 | 10 153 906 176 |
| `kernel_total` | 2 705 092 608 | 2 705 526 784 |
| `kernel_paged` | 1 347 219 456 | 1 347 244 032 |
| `kernel_nonpaged` | 1 357 873 152 | 1 358 282 752 |
| `page_size` | 4096 | 4096 |
| `processes` | 361 | 361 |
| `threads` | 4622 | 4622 |
| `handles` | 163 701 | 163 723 |

Sur les deux mesures, les valeurs structurelles (`total`, `commit_limit`, `page_size`) sont
identiques et les valeurs mouvantes bougent de moins de 0,1 % : la lecture est stable à
400 ms d'intervalle, ce qui satisfait le critère de répétabilité du protocole (moins de 10 %
d'écart).

## Ce que le spike a établi

1. **`PERFORMANCE_INFORMATION` compte en pages, pas en octets.** Ma première version
   affichait `system_cache = 2 461 968` — un cache système de 2,4 Mo, visiblement faux. La
   valeur est en pages : × 4096 = **10,16 Gio**. Le piège est silencieux : rien dans le type
   ne l'indique, seule la documentation le dit. C'est exactement le genre d'erreur qu'un test
   unitaire ne peut pas attraper, et que `FR-MEM-1` doit interdire en imposant une conversion
   nommée.
2. **`GlobalMemoryStatusEx` ne donne pas la mémoire « libre ».** Elle expose `AvailPhys`
   (disponible, cache compris) et `AvailPageFile` (charge de commit disponible) — mais pas la
   liste libre réelle. Ma première version avait étiqueté `AvailPageFile` comme « free » :
   c'était faux. **La mémoire libre et les listes de pages exigent la requête noyau**
   (`NtQuerySystemInformation`), ce qui confirme le marquage *à valider* du cahier des charges
   et interdit de fonder le « gain » (section 8.6) sur `GlobalMemoryStatusEx` seul.
3. **Les totaux se recoupent.** `commit_total` (30,0 Gio) reste sous `commit_limit`
   (49,7 Gio) ; `available` + charge utilisée ≈ `total` à 1 % près. Les compteurs ne se
   contredisent pas, ce qui valide le choix de les exposer ensemble (O1 du cahier des charges :
   écart < 2 %).
4. **`GetPerformanceInfo` vit dans `psapi.dll`, pas `kernel32`.** Le compilateur ne le voit
   pas ; seul l'éditeur de liens le signale (`LNK2019`). À noter pour la couche
   `obliniscor-sys`, qui devra lier `psapi`.

## Linux — deuxième passage

Machine : WSL2, noyau `6.6.87.2-microsoft-standard-WSL2`, x86_64, 16 Gio. Binaire **statique
musl** compilé depuis Windows (`-C linker=rust-lld`), copié dans la distribution : aucune
installation n'a été faite dans le WSL pour ce spike.

| Champ | Mesure | `free -b` |
| --- | --- | --- |
| `total` | 16 670 490 624 | 16 670 490 624 |
| `free` | 14 042 603 520 | 14 042 603 520 |
| `available` | 15 651 020 800 | 15 651 287 040 |
| `swap_total` / `swap_free` | 4 294 967 296 / 4 294 967 296 | identiques |
| `buffers + cached + sreclaimable` | 1 870 405 632 | `buff/cache` = 1 870 938 112 |

L'écart est de **0,002 %** sur `available` et **0,03 %** sur `buff/cache` — les deux
commandes ne mesurent pas au même instant. C'est très en deçà du critère de 2 % de `O1`.

### Ce que Linux a établi

5. **`MemAvailable` existe et fait le travail.** Depuis Linux 3.14, le noyau publie déjà
   l'estimation de mémoire récupérable : sous Linux, obliNiscor n'a **pas** à recalculer
   « disponible = libre + cache », et ne doit surtout pas le faire autrement que le noyau.
6. **`buff/cache` demande trois clés.** `Cached` **n'inclut pas** `SReclaimable` : `free`
   additionne `Buffers + Cached + SReclaimable`. Ne lire que `Cached` sous-estime le cache de
   273 Mio ici — et bien davantage sur un système qui monte des systèmes de fichiers.
7. **Le format de `/proc/meminfo` n'est pas régulier.** `MemTotal:` est suivi d'espaces, pas
   d'un `=`. Ma première version passait la clé sans les deux-points : elle lisait `":"` comme
   valeur et affichait **zéro pour tout**, sans erreur. Le spike l'a montré ; un test unitaire
   sur un extrait réel aurait suffi, et trois tests le couvrent maintenant (`linux.rs`).



- **La liste *standby* n'est pas lisible par ces appels.** Elle sera lue par la requête noyau
  (`SystemMemoryListInformation`), non tentée ici : elle demande `SeProfileSingleProcessPrivilege`
  et sort du périmètre « lecture seule » de ce premier passage.
- **Le coût d'une purge** (défauts de page durs dans les 30 s) n'a pas été mesuré : cela
  demande d'exécuter une purge, donc un accord explicite et une machine dédiée.
- **Windows 10 22H2** n'a pas été mesuré : le build change les zones disponibles (`FR-MEM-4`).
- **Linux et macOS** sont à faire : `free -b` / `/proc/meminfo` et `vm_stat` /
  `host_statistics64`.

## Conséquences pour le cahier des charges

- `FR-MEM-1` doit nommer **trois** sources, pas deux : `GlobalMemoryStatusEx` (RAM physique et
  charge de commit), `GetPerformanceInfo` (cache système, noyau, objets — **en pages**), et la
  requête noyau (listes de pages).
- La définition du gain (section 8.6) est confirmée : elle ne peut pas venir de
  `GlobalMemoryStatusEx`, qui ne distingue pas libre et disponible.
- `O1` (« écart < 2 % avec les compteurs Windows ») reste atteignable : les totaux concordent
  déjà à 1 %.

## Sources

- Microsoft Learn, `MEMORYSTATUSEX` et `GlobalMemoryStatusEx` (consulté le 2026-11-01).
- Microsoft Learn, `PERFORMANCE_INFORMATION` et `GetPerformanceInfo` (même date) — c'est la
  page qui indique l'unité en pages.
- Aucun code de projet tiers consulté (règle clean room, ADR-001).
