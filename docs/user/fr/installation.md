[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · **Français** · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Installation

GitPoulpi est un seul programme, `gitpoulpi`. Lancé sans sous-commande, il ouvre l'interface dans le terminal ; avec une sous-commande, c'est l'interface en ligne de commande (`--json` pour les scripts) ; `gitpoulpi mcp serve` est le serveur MCP pour les agents IA (voir [MCP et agents](mcp-and-agents.md)). À côté, l'archive contient `gitpoulpi-updater`, le petit programme séparé qui met à jour une installation (voir [Mises à jour](updates.md)).

Par compatibilité, l'archive contient aussi trois petits relais qui ne font que démarrer `gitpoulpi` : `gitpoulpi-tui` (ouvre l'interface ; il sera retiré dans une version ultérieure), `gitpoulpi-mcp` (démarre le serveur MCP ; il reste pendant toute la ligne 1.x, donc les configurations d'agents existantes continuent de fonctionner) et `gitpoulpi-launch` (ce que les raccourcis exécutent pour afficher une fenêtre GitPoulpi déjà ouverte, `gitpoulpi launch` ; sous Windows il n'a pas de fenêtre de console, donc un clic n'en fait jamais clignoter une). Tout est installé, mis à jour et supprimé ensemble.

## Ce qu'il vous faut

- Un **`git` système** dans votre `PATH`. GitPoulpi utilise libgit2 pour la
  plupart des opérations et votre `git` pour le reste.
  La version **2.34 ou plus récente** est requise. Sous macOS, le `git` des Command Line Tools d'Apple convient, tout comme celui de Homebrew.
- Un **terminal** avec prise en charge de la souris. kitty, Ghostty, WezTerm, Rio et
  iTerm2 (ainsi que Windows Terminal, foot et Tabby, via sixel) dessinent le graphe des commits sous forme d'images. Les autres
  terminaux (la console Windows, zellij, screen et la plupart des autres) affichent
  un graphe en caractères. Voir [Choisir un terminal](terminals.md) et [Multiplexeurs](multiplexers.md).
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
l'archive avec le fichier `.sha256`, décompressez-la et placez ses
exécutables (`gitpoulpi`, `gitpoulpi-updater` et les relais `gitpoulpi-tui`, `gitpoulpi-mcp` et `gitpoulpi-launch`)
dans un dossier de votre `PATH`, avec le marqueur
`.gitpoulpi-dist` (`gitpoulpi-updater` et le marqueur doivent rester à côté des autres : le marqueur dit `channel=archive`, pour que cette version puisse se
mettre à jour elle-même, voir [Mises à jour](updates.md)). Le fichier `INSTALL.txt` de l'archive
donne les mêmes étapes.

### Vérifier un téléchargement

Chaque fichier de version est signé avec la clé [minisign](https://jedisct1.github.io/minisign/) du projet, et GitPoulpi vérifie lui-même cette signature avant d'installer une mise à jour. Pour un fichier que vous avez téléchargé à la main, le `.minisig` à côté de lui est la signature. La clé publique (identifiant de clé `B461FFCBF6F6A4F2`) est commitée sous `packaging/minisign.pub` dans le dépôt :

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Lisez le commentaire signé qu'affiche la commande : il doit être `file:<le nom du fichier> version:<la version attendue>`. Une signature valide mais qui désigne un autre fichier ou une autre version n'est pas une signature pour le vôtre.

Quand la version a été construite avec des attestations de provenance de build (la variable `ATTESTATIONS` du dépôt), la CLI GitHub vérifie où un fichier a été construit :

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

Le fichier `.sha256` ne détecte qu'un téléchargement abîmé. La signature est ce qui montre que le fichier est bien celui du projet.

## Avec un installateur

Chaque version publiée propose aussi un installateur par système. Tous deux font
les mêmes choix (raccourcis, PATH, alias de commande comme `gp`) en appelant le
`gitpoulpi` installé lui-même, le code derrière Paramètres > Lanceur :
ce que l'installateur met en place et ce que les Paramètres affichent concordent
donc toujours.

### Windows : `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

La première boîte demande pour qui est l'installation :

| Choix | Où | Droits d'administrateur | Ce que cela change pour les mises à jour |
|---|---|---|---|
| **Seulement pour moi** (par défaut) | `%LOCALAPPDATA%\Programs\GitPoulpi` | aucun | GitPoulpi remplace ses propres fichiers : **Mettre à jour et redémarrer** dans le [panneau de mise à jour](updates.md), sans demande d'autorisation. |
| **Pour tous les utilisateurs** | `Program Files\GitPoulpi` | une demande à l'installation | Chaque mise à jour demande l'accord d'un administrateur (une invite Windows), et les sessions GitPoulpi des autres utilisateurs de l'ordinateur sont d'abord fermées. |

Le même choix fonctionne en silence : `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` ou `/ALLUSERS`.

Cases à cocher : raccourci du menu Démarrer, raccourci du Bureau, ajout au PATH. Une page demande l'**alias de commande** : `gp` par défaut, le nom que vous voulez, ou vide pour aucun (`/ALIAS=nom` ou `/ALIAS=` en ligne de commande). PowerShell utilise déjà `gp` pour `Get-ItemProperty` : choisissez un autre nom pour lui, ou utilisez `gitpoulpi`.

L'installateur copie les exécutables (dont `gitpoulpi-updater`, le petit programme qui remplace les fichiers pendant une mise à jour) et le marqueur `.gitpoulpi-dist`. Le marqueur enregistre comment GitPoulpi a été installé (`inno-user` ou `inno-machine`), ce que le panneau de mise à jour affiche comme **Installé avec**. Relancer l'installateur conserve vos choix (portée, raccourcis, alias).

Pour désinstaller, passez par Paramètres Windows > Applications : cela lance `gitpoulpi uninstall` (alias, entrées du PATH, raccourcis) et supprime les fichiers, y compris les restes `.old` du programme de mise à jour. Vos paramètres et vos dépôts restent.

### Linux et macOS : `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Il choisit l'archive adaptée à votre système et à votre processeur, vérifie son
`.sha256`, s'installe (canal `archive` : GitPoulpi se met à jour sur place) dans `~/.local/opt/gitpoulpi` et crée des liens
`gitpoulpi` (qui ouvre l'interface terminal quand on le lance sans sous-commande) et `gitpoulpi-mcp` (le relais) dans `~/.local/bin`. Aucun accès root n'est
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

## Avec un gestionnaire de paquets

Chaque canal ci-dessous écrit le même marqueur `.gitpoulpi-dist` à côté des programmes, avec son propre nom de canal. GitPoulpi le lit pour savoir qui s'occupe des mises à jour : c'est le gestionnaire de paquets, donc GitPoulpi ne remplace jamais des fichiers dans son dos. Le panneau de mise à jour (cliquez sur la version dans la barre de titre) lance alors la commande propre du gestionnaire de paquets, ou l'affiche pour que vous la copiiez. Voir [Mises à jour](updates.md).

| Canal | Installation | Remarques |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Installe pour tous les utilisateurs : les mises à jour demandent donc l'accord d'un administrateur. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` puis `scoop install gitpoulpi` | Par utilisateur, sans droits d'administrateur. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | Le tap est `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | voir [le dépôt apt](#apt-debian-et-ubuntu) | Dépôt signé ; un `.deb` est aussi joint à chaque version. |
| **MSI** (Windows, géré) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Pour les administrateurs : tous les utilisateurs, pas de mise à jour dans l'application. Voir [Distribution en entreprise](enterprise-distribution.md#msi-avec-intune-ou-une-stratégie-de-groupe). |
| **cargo** | `cargo binstall gitpoulpi-cli` | Télécharge l'archive de version au lieu de compiler. |
| **cargo, depuis crates.io** | `cargo install --locked gitpoulpi-cli` | Compile ; nécessite une chaîne d'outils Rust. Installe `gitpoulpi`, qui ouvre l'interface terminal quand on le lance sans sous-commande et sert aussi MCP (`gitpoulpi mcp`). |
| **cargo, depuis les sources** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Compile ; nécessite une chaîne d'outils Rust. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Voir [Exécuter dans Docker](docker.md). |

Les noms ci-dessus sont ceux vers lesquels le processus de publication publie (voir [Releasing](../../developer/releasing.md)) : un canal qui n'est pas encore en ligne répond « not found », et l'archive de version ou l'installeur fonctionne toujours.

La gestion du PATH propre à chaque gestionnaire de paquets s'applique : Chocolatey, Scoop et Homebrew placent eux-mêmes les programmes dans le PATH, et aucun ne crée l'alias `gp` (lancez `gitpoulpi alias create gp`, ou Paramètres > Lanceur).

### apt (Debian et Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

Le dépôt prend en charge `amd64` et `arm64` et est signé avec une clé qui appartient au projet. Les programmes vont dans `/usr/lib/gitpoulpi`, avec des liens dans `/usr/bin`. Le paquet dépend de `git` et recommande `git-lfs`. Sans le dépôt, installez le `.deb` d'une version avec `sudo apt install ./gitpoulpi_<version>_<arch>.deb` : GitPoulpi vous dit alors de télécharger vous-même le `.deb` suivant, car il ne propose des commandes `apt` que s'il trouve le dépôt dans `/etc/apt/sources.list.d/gitpoulpi.*`.

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

Les scripts `build-opti` à la racine du dépôt lancent cette compilation et
affichent la taille des exécutables :

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
systèmes). L'archive contient les exécutables, `INSTALL.txt`,
`README.md`, `LICENSE` et `CHANGELOG.md`. Un fichier `.sha256` est écrit à
côté.

## Mises à jour

La barre de titre affiche la version en cours (`v1.0.0`) entre le nom de l'application et le chemin du dépôt. Un clic dessus ouvre le panneau de mise à jour seulement quand une version plus récente existe ou qu'une mise à jour est en cours ; sinon il ouvre l'onglet **Notes de version**. Vous pouvez aussi taper `Ctrl+N` puis `u`, ou lancer **Mises à jour : Afficher la mise à jour** depuis la palette de commandes (`Ctrl+P`) : le panneau de mise à jour s'ouvre avec la version, la façon dont GitPoulpi a été installé et ce qu'il faut faire pour une version plus récente. **Aide : Notes de version** ouvre le journal des modifications.

Une fois par jour, en arrière-plan, GitPoulpi cherche une version plus récente à l'endroit où ses versions sont publiées (GitHub par défaut) (`updates.mode`, [Paramètres > Synchronisation](settings.md#synchronisation)). Quand il en trouve une, la pastille affiche `v0.2.0 disponible` dans la couleur d'accent.

Ce que fait le panneau dépend de la façon dont GitPoulpi a été installé : une archive de version ou l'installeur Windows remplace ses propres fichiers, un gestionnaire de paquets lance sa propre commande, une installation gérée ou en conteneur dit comment les mises à jour lui parviennent. Une version compilée avec `cargo build` ne se met jamais à jour. La description complète, avec le retour en arrière, `auto` face à `notify` et ce qu'un administrateur peut verrouiller, est dans [Mises à jour](updates.md).

## Vérifier que ça marche

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi
```

`--version` affiche la version et le commit à partir duquel le build a été
fait. Suivez ensuite [Premiers pas](getting-started.md).

## Plusieurs installations côte à côte

Un paquet, un dépôt de développement, un build cargo et une archive décompressée peuvent être installés en même temps. Chacun est une installation à part entière : son propre nom de lanceur (`GitPoulpi (Chocolatey)`, `GitPoulpi (dev)`, ...), sa couleur de pieuvre, ses alias de commande, son entrée de menu « Ouvrir dans » et ses démons, si bien que deux versions tournent en parallèle sans jamais se prendre leurs fenêtres. Les paramètres, les dépôts et les jetons sont partagés. Personne ne possède le nom de lanceur simple `GitPoulpi` tant que vous ne l'avez pas choisi avec **Devenir l'installation principale** dans Paramètres > Lanceur. [Paramètres](settings.md#lanceur) liste les autres installations et propose un bouton **Oublier** pour celle qui a disparu.

## Désinstaller

`gitpoulpi uninstall` supprime les alias de commande (et leurs clés App Paths
sous Windows), les entrées du PATH, les raccourcis et les entrées du menu du
gestionnaire de fichiers **de cette installation seulement** : celles des autres
restent, et le `PATH` garde leurs dossiers ainsi que le dossier d'alias partagé
tant que l'une d'elles y a encore un alias. Les épingles de la barre des tâches
et du Dock restent (Windows ne permet pas de désépingler de l'extérieur), et vos
paramètres conservent l'entrée de cette installation : réinstaller dans le même
dossier lui rend son apparence. Il supprime ensuite les fichiers du programme
d'une installation depuis une version publiée ; cette partie refuse de s'exécuter
sur un build sans le marqueur `.gitpoulpi-dist` (un build cargo). Avec
`--keep-files`, il laisse les fichiers du programme et fonctionne sur n'importe
quel build, un dépôt de développement compris. Il est indisponible pour les
agents MCP.

| Votre mode d'installation | Ce qui supprime les lanceurs et les alias |
|---|---|
| Installateur Windows (utilisateur ou tous les utilisateurs), winget | Le désinstallateur lance `gitpoulpi uninstall --keep-files`, puis supprime les fichiers. |
| Chocolatey | Son désinstallateur automatique lance le désinstallateur de l'installateur. |
| Scoop | `scoop uninstall gitpoulpi` lance d'abord `gitpoulpi uninstall --keep-files`. |
| MSI | La suppression le lance sous le compte qui a installé. Supprimé depuis un autre compte, les lanceurs de l'utilisateur restent et apparaissent comme orphelins dans les Paramètres des autres installations (**Oublier**). |
| apt, AUR, Homebrew, cargo, un dépôt de développement | Aucun crochet ne s'exécute pour vous : lancez `gitpoulpi uninstall --keep-files` avant de supprimer le paquet. |
| Archive, `install.sh --uninstall` | Lancez `gitpoulpi uninstall` ou `install.sh --uninstall`, puis supprimez le dossier. |

Sans installateur, supprimez les fichiers du programme. GitPoulpi conserve
aussi ces fichiers :

| Quoi | Où |
|---|---|
| Paramètres (`config.toml`) et sessions | Le dossier de configuration de votre OS. `GITPOULPI_CONFIG_DIR` le change. |
| État par dépôt | `gitpoulpi/` dans le dossier `.git` du dépôt par défaut (`session.per_repo_state_dir` peut le déplacer vers `.gitpoulpi/` dans l'arbre de travail ou vers le dossier de données de l'utilisateur). |
| Caches (avatars) | Le dossier de cache de votre OS. |
| Jetons et clés d'API | Votre assistant d'identifiants. **Paramètres > SSH et identifiants > Oublier tous les noms d'utilisateur et mots de passe** les supprime. |

Voir la [Référence de configuration](config-reference.md) pour les chemins exacts.
