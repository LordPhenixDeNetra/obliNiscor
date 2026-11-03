# Spike — service et canal nommé (OBL-008)

- **Date :** 2026-11-03
- **Machine :** Windows 11, build 10.0.26200, x64
- **Outil :** `spikes/service` (binaire jetable, hors workspace, aucun fichier écrit)
- **État :** **Windows mesuré** — les autres systèmes viendront avec P6b

## Question

Le service et la GUI vont se parler par un canal nommé. Trois choses doivent être vraies avant
d'écrire le service : l'ACL peut-elle être **explicite** dès la création, le client légitime
passe-t-il, et un client visé par un refus est-il **réellement** rejeté ?

## Méthode

```sh
cd spikes/service
cargo run --release
```

Le binaire crée trois canaux nommés, chacun avec un descripteur de sécurité construit depuis une
chaîne SDDL (`ConvertStringSecurityDescriptorToSecurityDescriptorW`), tente une connexion cliente
sur chacun, puis ferme les handles. Le canal disparaît avec le dernier handle : **rien ne reste
sur le système**, et aucune donnée n'est écrite.

## Résultats bruts

| Cas | SDDL | Créé | Client connecté | `GetLastError` |
| --- | --- | --- | --- | --- |
| Propriétaire seul | `D:P(A;;GA;;;OW)` | oui | **oui** | 0 |
| Refus à tout le monde | `D:(D;;GA;;;WD)` | oui | **non** | **5** (`ACCESS_DENIED`) |
| Système et utilisateur | `D:P(A;;GA;;;SY)(A;;GA;;;IU)` | oui | **oui** | 0 |

## Ce que le spike a établi

1. **L'ACL s'impose à la création, pas après.** `CreateNamedPipeW` accepte un
   `SECURITY_ATTRIBUTES` dont le descripteur vient d'une chaîne SDDL : il n'y a pas de fenêtre
   pendant laquelle le canal serait ouvert, et pas besoin de construire des ACE à la main.
2. **Le refus est réel et son code est exploitable.** Un client couvert par une entrée *refus*
   échoue en **erreur 5** au moment de la connexion. Le service pourra donc traduire ce code en
   `PlatformError::Denied` / `E_DENIED` sans deviner, et le client saura dire « accès refusé »
   au lieu d'« échec ».
3. **Le canal ne survit pas au processus.** Fermer le dernier handle détruit l'instance : un
   service arrêté brusquement ne laisse pas de canal orphelin qui bloquerait le redémarrage.
4. **`OW` est un piège pour un service.** Le cas 3 est le seul réaliste : le service tourne en
   `LocalSystem`, donc le propriétaire du canal est **le système**, et une ACL
   `D:P(A;;GA;;;OW)` exclurait l'utilisateur qui doit précisément piloter le service. Il faut
   nommer explicitement `SY` (système local) et `IU` (utilisateur interactif).

## Ce qui reste incertain

- **Aucun test entre deux comptes distincts.** Le refus mesuré vient d'une entrée *refus* qui
  s'applique aussi à nous-mêmes : cela prouve que l'ACL est appliquée, pas la matrice complète
  « utilisateur A autorisé, utilisateur B refusé ». Une machine à deux comptes le ferait.
- **L'usurpation d'identité** (`ImpersonateNamedPipeClient`) n'est pas testée : le service devra
  vérifier *qui* parle, et pas seulement *qui a le droit d'ouvrir*.
- **Le mode message** n'est pas mesuré : l'IPC d'obliNiscor utilise déjà un cadrage par longueur
  (`obliniscor-proto`), donc le mode octets convient et le mode message n'apporte rien.
- **Le comportement quand toutes les instances sont occupées** (`ERROR_PIPE_BUSY`, 231) n'est pas
  observé : le service devra peut-être attendre et réessayer.

## Conséquences pour le cahier des charges

- `FR-SVC-2` peut affirmer ce que le spike vient de mesurer : l'ACL est posée à la création, et
  le refus remonte en code 5, traduit en `E_DENIED`.
- **Le descripteur du service doit être `D:P(A;;GA;;;SY)(A;;GA;;;IU)`**, pas `OW`. Une ADR ou le
  commentaire du code doit le dire, sinon la première version du service sera inaccessible à son
  propre utilisateur.
- La GUI n'a besoin d'aucun privilège particulier pour se connecter : elle est l'utilisateur
  interactif, et c'est `IU` qui l'autorise.
- `OBL-017`, `OBL-018` et `OBL-019` ne sont plus bloqués par une inconnue : la forme du canal et
  le comportement du refus sont connus.

## Sources

- Microsoft Learn, `CreateNamedPipeW`, `SECURITY_ATTRIBUTES`, `ConvertStringSecurityDescriptorToSecurityDescriptorW`,
  `CreateFileW` (consulté le 2026-11-03).
- Microsoft Learn, « Named Pipe Security and Access Rights » — c'est la page qui documente les
  alias SDDL `SY`, `IU`, `OW`, `WD` utilisés ici.
- Aucun code de projet tiers consulté (règle clean room, ADR-001).
