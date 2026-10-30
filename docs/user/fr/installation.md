[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · **Français** · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Installation

GitPoulpi fournit trois exécutables :

| Exécutable | Rôle |
|---|---|
| `gitpoulpi-tui` | L'interface dans le terminal. |
| `gitpoulpi` | L'interface en ligne de commande (`--json` pour les scripts). Sans sous-commande, elle ouvre l'interface. |
| `gitpoulpi-mcp` | Le serveur MCP pour les agents IA. Voir [MCP et agents](mcp-and-agents.md). |

À côté d'eux, l'archive contient `gitpoulpi-launch`, un minuscule programme que les raccourcis exécutent pour afficher une fenêtre GitPoulpi déjà ouverte (`gitpoulpi launch`). Sous Windows il n'a pas de fenêtre de console, donc un clic n'en fait jamais clignoter une. Il est installé, mis à jour et supprimé avec les autres.

## Ce qu'il vous faut

- Un **`git` système** dans votre `PATH`. GitPoulpi utilise libgit2 pour la
  plupart des opérations et votre `git` pour le reste.
  La version **2.34 ou plus récente** est requise. Sous macOS, le `git` des Command Line Tools d'Apple convient, tout comme celui de Homebrew.
- Un **terminal** avec prise en charge de la souris. kitty, Ghostty, WezTerm et
  iTerm2 (ainsi que Windows Terminal et foot, via sixel) dessinent le graphe des commits sous forme d'images. Les autres
  terminaux (la console Windows, tmux et la plupart des autres) affichent
  un graphe en caractères. Voir [Choisir un terminal](terminals.md).
- Une **Nerd Font** ou une police avec emoji, pour les icônes de la barre
  d'outils et du panneau latéral.
- Facultatif : **`git-lfs`** 3.0 ou plus récent, pour les dépôts Git LFS. Sans
  lui, les fichiers LFS apparaissent comme de petits fichiers pointeurs et
  GitPoulpi vous en explique la raison. Voir [Git LFS](lfs.md) pour
  l'installer.
- Facultatif : un **assistant d'identifiants** (Git Credential Manager,
  libsecret, `pass`, Keychain), pour stocker les jetons des hébergeurs et les
  clés d'API de l'IA. Voir [Hébergeurs et suivi de tickets](hosting-integration.md).

## Depuis une archive de release

Une archive de release s'appelle `gitpoulpi-<version>-<target>.zip` (Windows)
ou `.tar.gz` (Linux, macOS), avec un fichier `.sha256` à côté. Vérifiez
l'archive avec le fichier `.sha256`, décompressez-la et placez les trois
exécutables dans un dossier de votre `PATH`. Le fichier `INSTALL.txt` de
l'archive donne les mêmes étapes.

## Avec un installateur

Chaque version publiée propose aussi un installateur par système. Tous deux font
les mêmes choix (raccourcis, PATH, alias de commande comme `gp`) en appelant le
`gitpoulpi` installé lui-même, le code derrière Paramètres > Général > Lanceur :
ce que l'installateur met en place et ce que les Paramètres affichent concordent
donc toujours.

### Windows : `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Il s'installe pour l'utilisateur courant, sans demande d'élévation, dans
`%LOCALAPPDATA%\Programs\GitPoulpi` (la première boîte propose une installation
pour tous les utilisateurs et le choix du dossier). Cases à cocher : raccourci du
menu Démarrer, raccourci du Bureau, ajout au PATH. Une page demande l'**alias de
commande** : `gp` par défaut, le nom que vous voulez, ou vide pour aucun.
PowerShell utilise déjà `gp` pour `Get-ItemProperty` : choisissez un autre nom
pour lui, ou utilisez `gitpoulpi`.

L'installateur copie les trois exécutables et le marqueur `.gitpoulpi-dist` ;
[Mettre à jour maintenant](#mettre-à-jour-sur-place) fonctionne donc exactement
comme avec le zip. Une installation par utilisateur est accessible en écriture,
elle peut donc se mettre à jour elle-même ; une installation pour tous les
utilisateurs dans Program Files demande une élévation pour être remplacée :
réinstallez-la plutôt avec le nouvel installateur. Pour désinstaller, passez par
Paramètres Windows > Applications : cela lance `gitpoulpi uninstall` (alias,
entrées du PATH, raccourcis) et supprime les fichiers, y compris les restes
`.old` du programme de mise à jour. Vos paramètres et vos dépôts restent.

### Linux et macOS : `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Il choisit l'archive adaptée à votre système et à votre processeur, vérifie son
`.sha256`, s'installe dans `~/.local/opt/gitpoulpi` et crée des liens
`gitpoulpi` et `gitpoulpi-mcp` dans `~/.local/bin`. Aucun accès root n'est
nécessaire et rien en dehors de votre dossier personnel n'est touché. Sans
`--yes`, il pose la question de l'alias (`gp` par défaut), du PATH et des
raccourcis.

| Option | Effet |
|---|---|
| `--prefix DIR` | Installe les fichiers du programme dans `DIR` (chemin absolu). |
| `--version X.Y.Z` | Installe cette version plutôt que la dernière. |
| `--archive FILE` | Utilise une archive locale ; `FILE.sha256` doit se trouver à côté (`--no-verify` ignore cette vérification). Aucun accès réseau. |
| `--alias NAME` / `--no-alias` | Fait de `NAME` la commande qui démarre GitPoulpi (`gitpoulpi alias create`), ou n'en crée aucune. |
| `--no-path` | N'ajoute pas GitPoulpi au PATH des nouveaux shells (sinon `gitpoulpi path add`). |
| `--desktop` | Crée aussi les raccourcis du menu des applications et du Bureau. |
| `--yes` | Ne pose aucune question : les options et les valeurs par défaut s'appliquent (alias `gp`, PATH oui, raccourcis non). |
| `--uninstall` | Lance `gitpoulpi uninstall`, puis supprime les liens et le dossier. |

La recherche de la dernière version suppose que le dépôt est public ; pour un
dépôt privé, téléchargez l'archive dans un navigateur et utilisez `--archive`.

## Depuis les sources

Il vous faut une chaîne d'outils Rust stable récente (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Les exécutables se trouvent dans `target/release/`. Le profil `release` se
compile vite. Pour des exécutables les plus petits et les plus rapides,
utilisez le profil `dist`, plus long à compiler :

```
cargo build --profile dist
```

Les scripts `build-opti` à la racine du dépôt lancent cette compilation pour
les trois exécutables et affichent leur taille :

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd ou PowerShell)
```

Pour ne compiler que les langues et les fonctions que vous utilisez, voir
[Builds personnalisés](custom-builds.md).

## Construire soi-même une archive de release

```
cargo run -p xtask -- dist
```

Cette commande compile le profil `dist` et écrit
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) ou `.tar.gz` (autres
systèmes). L'archive contient les trois exécutables, `INSTALL.txt`,
`README.md`, `LICENSE` et `CHANGELOG.md`. Un fichier `.sha256` est écrit à
côté.

## Mises à jour

La barre de titre affiche la version en cours (`v1.0.0`) entre le nom de
l'application et le chemin du dépôt. Cliquez dessus, ou lancez **Aide : Notes de version** depuis la palette
de commandes (`Ctrl+P`) : un onglet Notes de version s'ouvre avec le journal des
changements, positionné sur la version en cours.

Une fois par jour, en arrière-plan, GitPoulpi demande à GitHub s'il existe une
version plus récente (`updates.mode`,
[Paramètres > Synchronisation](settings.md#synchronisation)). Quand c'est le cas,
la pastille affiche `v0.2.0 disponible` dans la couleur d'accent et un clic dessus
ouvre l'onglet Notes de version, avec les notes de cette version en haut. L'onglet
a trois boutons, chacun avec une touche et une commande de palette :

- **Télécharger** (`d`, *Mises à jour : Télécharger*) ouvre la page de la version
  dans votre navigateur.
- **Ignorer cette version** (`s`, *Mises à jour : Ignorer cette version*) masque
  l'avis jusqu'à l'arrivée d'une version plus récente.
- **Rechercher** (`r`, *Mises à jour : Rechercher une mise à jour maintenant*)
  interroge de nouveau tout de suite.

`updates.mode = "off"` ne fait aucune requête. Sans connexion réseau, ou tant que
le dépôt est privé et qu'aucun jeton github.com n'est enregistré, la vérification
échoue discrètement (une ligne dans le journal de l'application, rien dans
l'interface). `GITPOULPI_UPDATE_API` dirige la vérification vers une autre racine
d'API (utilisé par les tests).

### Mettre à jour sur place

Un build installé depuis une archive de version peut se remplacer lui-même.
L'archive contient un petit fichier `.gitpoulpi-dist` à côté des exécutables ; un
build en cours n'est « mettable à jour » que si ce fichier se trouve à côté de son
exécutable (et nomme sa plateforme). Un build fait avec `cargo`, un build
personnalisé copié ailleurs, ou un paquet qui a laissé tomber le fichier ne se met
jamais à jour tout seul : son onglet Notes de version le dit et ne propose que
**Télécharger**.

Pour un build qui peut se mettre à jour, l'onglet ajoute :

- **Mettre à jour maintenant** (`u`, *Mises à jour : Mettre à jour maintenant*)
  télécharge l'archive et son `.sha256` en arrière-plan (une tâche dans
  l'indicateur de la ligne d'état, avec son pourcentage, annulable comme toute
  tâche), vérifie l'empreinte, décompresse l'archive et place chaque nouveau
  fichier à côté de l'ancien sous le nom `<nom>.new`. Rien de ce qui tourne n'est
  touché. En mode `auto`, la même chose se produit toute seule dès qu'une
  vérification trouve une version plus récente.
- **Redémarrer pour mettre à jour** (`R`, *Mises à jour : Redémarrer pour mettre
  à jour* ; la pastille affiche la même chose une fois le téléchargement préparé)
  est toujours une action de votre part, jamais automatique. Elle arrête le démon
  autonome de la session (un démon intégré part avec la fenêtre), remplace les
  fichiers et démarre la nouvelle version avec les mêmes arguments et la même
  session.
- **Revenir à la version précédente** (`v`, *Mises à jour : Revenir à la version
  précédente*) remet les fichiers précédents en place et redémarre. Les fichiers
  de la version précédente restent sous le nom `<nom>.old` jusqu'à ce que la
  nouvelle version tourne depuis une minute, puis ils sont supprimés : le bouton
  est donc là pour ce premier lancement. Il ne fait pas partie du journal
  d'annulation git : il modifie les fichiers de GitPoulpi, pas un dépôt.

Comment les fichiers sont remplacés : sous Linux et macOS, chaque nouveau fichier
est renommé par-dessus l'ancien (opération atomique ; un programme en cours garde
son ancienne copie jusqu'à sa fin). Sous Windows, un `.exe` en cours ne peut pas
être écrasé : l'ancien est donc renommé en `<nom>.old` et le nouveau mis à sa
place ; un `.old` encore en cours d'exécution est supprimé à un démarrage
ultérieur. Le remplacement est tout ou rien : si un autre programme tient l'un
des exécutables (un client MCP qui lance `gitpoulpi-mcp`, par exemple), GitPoulpi
nomme le fichier, ne change rien et laisse le téléchargement préparé pour un
nouvel essai.

Ce qui est refusé, chaque fois avec un message : une version sans fichier
`.sha256`, une empreinte qui ne correspond pas, un fichier d'archive dont le nom
n'est pas celui de cette plateforme (`gitpoulpi-<version>-<target>.<zip|tar.gz>`),
une archive dont le marqueur nomme une autre version ou une autre plateforme, et
tout lien de téléchargement hors de ces hôtes : `github.com`, `api.github.com`,
`objects.githubusercontent.com`, `release-assets.githubusercontent.com` et
`github-releases.githubusercontent.com` de GitHub (la première requête et chaque
redirection ; une redirection vers ailleurs arrête le téléchargement). Les
archives d'un dépôt privé demandent une session de navigateur : le téléchargement
y échoue donc tant que le dépôt n'est pas public.

## Vérifier que ça marche

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` affiche la version et le commit à partir duquel le build a été
fait. Suivez ensuite [Premiers pas](getting-started.md).

## Désinstaller

`gitpoulpi uninstall` (ce que lancent les deux installateurs) supprime les alias
de commande, les entrées du PATH et les raccourcis créés par GitPoulpi, puis les
fichiers du programme d'une installation depuis une version publiée ; il refuse
de s'exécuter sur un build sans le marqueur `.gitpoulpi-dist` (un build cargo), et
`--keep-files` laisse les fichiers du programme. Il est indisponible pour les
agents MCP. Sans installateur, supprimez les trois exécutables. GitPoulpi conserve
aussi ces fichiers :

| Quoi | Où |
|---|---|
| Paramètres (`config.toml`) et sessions | Le dossier de configuration de votre OS. `GITPOULPI_CONFIG_DIR` le change. |
| État par dépôt | `.gitpoulpi/` dans chaque dépôt. |
| Caches (avatars) | Le dossier de cache de votre OS. |
| Jetons et clés d'API | Votre assistant d'identifiants. **Paramètres > SSH et identifiants > Oublier tous les noms d'utilisateur et mots de passe** les supprime. |

Voir la [Référence de configuration](config-reference.md) pour les chemins exacts.
