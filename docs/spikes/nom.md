# Spike — disponibilité du nom « obliNiscor »

- **Tâche :** OBL-002
- **Date de vérification :** 2026-11-01
- **Verdict :** nom **libre** sur crates.io pour les six noms vérifiés ; la partie « marques »
  reste à faire.

## Question

Le nom « obliNiscor » et ses variantes de crates sont-ils disponibles ? Un nom déjà pris sur
crates.io interdirait la publication des bibliothèques ; un nom proche déposé comme marque
exposerait à une contestation.

## Méthode

Requête HTTP `GET https://crates.io/api/v1/crates/<nom>` : un `404` signifie qu'aucune crate
de ce nom n'existe. Le dépôt GitHub a été vérifié par `gh repo view`.

## Résultats — crates.io, 2026-11-01

| Nom | Résultat |
| --- | --- |
| `obliniscor` | libre (404) |
| `obliniscor-core` | libre (404) |
| `obliniscor-proto` | libre (404) |
| `obliniscor-sys` | libre (404) |
| `obliniscord` | libre (404) |
| `obliniscor-cli` | libre (404) |

## Résultats — hébergement

| Élément | Résultat |
| --- | --- |
| Dépôt `LordPhenixDeNetra/obliNiscor` | créé, **public** |
| Organisation ou utilisateur `obliniscor` sur GitHub | non vérifié — sans objet pour la v1 |

## Décision

**Le nom est retenu** pour les crates et le dépôt. Les noms ne sont **pas réservés** par
publication de crates vides : c'est une pratique courante, mais elle pollue le registre et
n'apporte ici aucune garantie, puisque rien n'est encore publié.

## Ce qui reste à faire

- **Recherche d'antériorité de marque** : registres nationaux et régionaux, plus la base de
  l'OAPI pour une diffusion en Afrique francophone. Ce point demande une recherche manuelle
  et, avant un dépôt de marque, un professionnel ; il n'est pas couvert ici.
- **Lecture à voix haute** dans les deux langues et recherche des variantes proches
  (`obliviscor`, `obliNiscor`, `obliniscord`) dans un moteur de recherche.
- **Décision d'enregistrer ou non** la marque : hors périmètre de la v1, à trancher dans
  ADR-0002.

## Sources

- `https://crates.io/api/v1/crates/...` consulté le 2026-11-01.
- `gh repo view LordPhenixDeNetra/obliNiscor` consulté le 2026-11-01.
