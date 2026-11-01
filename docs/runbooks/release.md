# Publier une version d'obliNiscor

- **Portée :** dépôt `obliNiscor`
- **Décisions :** ADR-008 (licence), ADR-009 (disponibilité), OBL-090 à OBL-110
- **État :** ce runbook décrit la cible. Aucune version n'a encore été publiée : le dépôt ne
  contient que la spécification.

## Ce que la release doit produire

| Système | Fichier | Contenu |
| --- | --- | --- |
| Windows | `obliniscor-<v>-windows-x86_64.msi` | `obliniscord` (service), `obliniscor` (GUI), `obliniscor-cli`, groupe local `obliniscor`, droits des dossiers |
| Linux | `obliniscor-<v>-linux-x86_64.deb` et `.rpm` | mêmes binaires, unité systemd durcie, socket Unix |
| macOS | `obliniscor-<v>-macos-aarch64.pkg` et `-x86_64.pkg` | mêmes binaires, démon launchd |
| tous | `SHA256SUMS` | une ligne par fichier |

Les noms d'actifs sont un contrat : le launcher les cherche avec le motif d'`apps.json`
(`obliniscor-{version}-{os}-{arch}.{ext}`), et la détection Windows lit `DisplayName` et
`DisplayVersion` dans les clés de désinstallation. Un MSI dont le `DisplayName` ne contient
pas « obliNiscor » rendra l'application invisible au launcher.

## Étapes

1. **Vérifier les critères de la section 16** du cahier des charges, en particulier la matrice
   A à H et l'absence de fonction simulée.
2. **Vérifier** :

   ```
   cargo xtask ci
   ```

3. **Signer** (OBL-090) : Authenticode sous Windows, horodaté ; rien d'équivalent côté Linux
   (la distribution se fait par dépôt signé, clé GPG), Developer ID et notarisation côté
   macOS.
4. **Étiqueter** `v<version>`, laisser la CI construire les paquets sur leurs systèmes
   respectifs, calculer `SHA256SUMS` et publier la release.
5. **Auditer les résidus** (OBL-110) sur une machine virtuelle propre : après désinstallation,
   0 service, 0 règle de pare-feu, 0 tâche planifiée, 0 dossier hors données conservées.

## Vérifier un paquet publié

```
gh release download v<version> -R LordPhenixDeNetra/obliNiscor -p "SHA256SUMS" -D %TEMP%\obliniscor
gh release download v<version> -R LordPhenixDeNetra/obliNiscor -p "*.msi" -D %TEMP%\obliniscor
certutil -hashfile "%TEMP%\obliniscor\obliniscor-<version>-windows-x86_64.msi" SHA256
```

L'empreinte doit correspondre à la ligne du fichier dans `SHA256SUMS`. Ensuite installer,
vérifier que le service démarre, puis relancer le launcher : sa propre ligne doit passer à
« à jour ».

## Signature — état actuel

**Aucun certificat n'est disponible.** Le cahier des charges prévoit deux voies (R12,
OBL-090) : certificat commercial, ou programme de signature pour projets ouverts. Sans
signature :

- Windows : SmartScreen avertit au premier lancement, et un navigateur affiche en plus le
  message « pas fréquemment téléchargé » — c'est de la **réputation de fichier**, pas un avis
  de malware ;
- macOS : Gatekeeper refuse le `.pkg` sans clic droit → *Ouvrir*, ou sans
  `xattr -d com.apple.quarantine`.

Ces avertissements sont assumés et documentés, jamais contournés. La garantie d'intégrité est
`SHA256SUMS`, pas l'avis du navigateur.

## Désinstallation : le cas le plus grave

Une désinstallation qui laisse des règles de blocage **coupe Internet à l'utilisateur**. C'est
pourquoi OBL-091 impose que l'installateur exécute la restauration réseau avant de retirer le
service, et qu'un script d'audit de résidus soit lancé après chaque essai. Une release ne
sort pas si ce script n'est pas vert.
