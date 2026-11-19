[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · **Français** · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="La pieuvre GitPoulpi" width="128"></p>

**Un client Git pour le terminal qui se prend pour une application de bureau.**
GitPoulpi vous offre un vrai graphe de commits sur lequel cliquer, glisser et
faire un clic droit, une palette de commandes pilotée au clavier qui atteint
chaque fonction, l’intégration des hébergeurs de code (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, dans le cloud ou auto-hébergés) et des outils de suivi de tickets (Jira, Trello, Linear, YouTrack, Redmine, ou un dossier de notes Markdown), et les mêmes
opérations en CLI scriptable et en serveur MCP, pour qu’un agent IA le pilote
exactement comme vous. GitPoulpi s’inspire de GitKraken et a été entièrement
réécrit en Rust : des binaires natifs, légers sur le disque et en mémoire, sans Electron, sans compte, sans télémétrie. Il tourne dans votre terminal préféré ([lequel ?](../../docs/user/fr/terminals.md)), parce que certains d’entre nous y vivent.

![GitPoulpi affichant le graphe de commits avec branches, tags et un stash, le panneau Refs à gauche, et les modifications locales avec le formulaire de commit à droite](../../docs/user/images/hero.png)

- **Souris et clavier, à égalité.** Cliquez, double-cliquez, faites un clic
  droit et glissez-déposez des refs sur des commits ; ou appuyez sur `Ctrl+P` et
  tapez ce que vous voulez. Chaque geste de souris a son jumeau au clavier, et
  chaque commande est dans la palette.
- **Un vrai graphe de commits.** Branches, merges, tags, stashes et copie de
  travail dans un seul Log défilant, dessiné en pixels dans les terminaux qui
  le permettent et en caractères partout ailleurs. De nombreux thèmes de
  couleurs sombres et clairs, avatars des auteurs, recherche.
- **Un stage précis.** Stagez, unstagez ou abandonnez un fichier, un dossier, un
  hunk ou une seule ligne directement depuis le diff. En ligne ou côte à côte,
  avec une minimap.
- **Réécrire l’historique sans risque.** Reformulez ou amendez n’importe quel
  commit (message, auteur, dates), faites un rebase interactif avec un écran de
  revue, réécrivez de nombreux commits d’un coup avec un aperçu et une seule
  annulation, cherry-pickez plusieurs commits, avancez en fast-forward, créez
  des worktrees et des patches.
- **Hébergeurs et suivi de tickets intégrés.** Parcourez, récupérez et créez des pull et merge requests (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) et relisez les changes Gerrit. Parcourez et créez des tickets, work items et cartes (aussi Jira, Trello, Linear, YouTrack, Redmine et des notes Markdown locales), au besoin sur un tableau Kanban. Clonez depuis votre compte et gérez vos clés SSH.
- **CLI et MCP pour les agents.** `gitpoulpi <commande> --json` couvre tout ce
  que fait l’interface, et `gitpoulpi mcp serve` l’expose sous forme d’outils MCP. Un
  démon d’arrière-plan par dépôt garde chaque interface vivante et
  synchronisée.
- **IA facultative, jamais obligatoire.** Désactivée par défaut et sans compte :
  apportez votre propre clé d’API ou une CLI d’agent locale comme `claude -p`.
  Générez des messages de commit, expliquez des modifications, rédigez des
  descriptions de pull request, répartissez vos modifications locales en
  plusieurs commits et préparez des règles de réécriture d’historique.
- **Git LFS, sous-modules, worktrees, patches, blame et historique de fichier,
  diffs d’images, un éditeur intégré.**

## Captures d'écran

| | |
|---|---|
| ![Le graphe du Log avec la palette de voies Material et la teinte de ligne activée](../../docs/user/images/theme-material.png) **Le graphe de commits**, ici avec la palette de voies Material et la teinte de ligne. | ![Le diff d’un fichier modifié avec les boutons Stage du hunk et Abandonner le hunk sur chaque hunk](../../docs/user/images/diff.png) **Le diff**, avec stage et abandon par hunk ou par ligne. |
| ![La palette de commandes filtrée sur le mot stash](../../docs/user/images/palette.png) **La palette de commandes** (`Ctrl+P`) atteint chaque commande. | ![Le compositeur de commits proposant quatre commits pour les modifications locales, prêts à être relus](../../docs/user/images/composer.png) **Le compositeur de commits** fait relire les commits proposés par l’IA avant que quoi que ce soit soit commité. |
| ![Le menu clic droit d’un commit avec les actions cherry-pick, revert, reset, rebase, branche et tag](../../docs/user/images/context-menu.png) **Clic droit** sur n’importe quoi pour ses actions. | ![La page IA des Préférences avec les profils, les instructions par action et l’interrupteur d’activation](../../docs/user/images/settings-ai.png) **Les Préférences**, avec recherche, y compris la page IA facultative. |

D’autres images, avec des explications, se trouvent dans la [documentation utilisateur](../../docs/user/fr/README.md).

## Installation

Il vous faut un `git` système ; `git-lfs` est facultatif.

### Linux et macOS (`install.sh`)

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew (macOS, Linux)

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt (Debian, Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop (Windows)

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Installateur Windows

Téléchargez `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` depuis la
[dernière version](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest) et
lancez-le. Pour les administrateurs, le MSI :

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

Voir [Exécuter dans Docker](../../docs/user/fr/docker.md).

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### Archive de release

Téléchargez-la depuis la
[dernière version](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest),
décompressez-la et suivez `INSTALL.txt` qu’elle contient.

### Depuis les sources

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Votre propre archive de release, un build personnalisé plus petit et les notes par OS :
[docs/user/fr/installation.md](../../docs/user/fr/installation.md) et
[docs/user/fr/custom-builds.md](../../docs/user/fr/custom-builds.md).

## Démarrage rapide

```
cd path/to/your/repository
gitpoulpi
```

- `?` ouvre l’aide avec chaque touche et chaque geste, `Ctrl+P` la palette de
  commandes, `Ctrl+,` les Préférences, `q` quitte.
- Cliquez sur un commit pour voir ses modifications ; cliquez sur la ligne du
  haut du Log (les modifications locales) pour stager des fichiers et commiter.
- Dans un terminal qui le permet (kitty, Ghostty, WezTerm, iTerm2 (ainsi que Windows Terminal et foot, via sixel)), le graphe
  est dessiné en pixels. Dans les autres (une console Windows simple,
  tmux), il bascule automatiquement en caractères.

La version pas à pas est dans [Premiers pas](../../docs/user/fr/getting-started.md),
et [la visite de l’interface](../../docs/user/fr/interface-tour.md) nomme chaque zone
de l’écran.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi mcp serve   # give an AI agent the same tools
```

L’IA est facultative et désactivée par défaut : activez l’IA propre à
GitPoulpi dans *Préférences > IA* ([Assistance IA](../../docs/user/fr/ai.md)) ; laissez
un agent externe piloter GitPoulpi avec [MCP et agents](../../docs/user/fr/mcp-and-agents.md)
(permissions, chaque outil) et la skill d’agent
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md).

## Documentation

La documentation utilisateur complète commence à **[docs/user/fr/README.md](../../docs/user/fr/README.md)** :

| | |
|---|---|
| [Premiers pas](../../docs/user/fr/getting-started.md) | Installer, ouvrir un dépôt, faire un premier commit |
| [Visite de l’interface](../../docs/user/fr/interface-tour.md) | Chaque zone de la fenêtre, nommée |
| [Travailler avec le Log](../../docs/user/fr/log-and-history.md) | Graphe, refs, branches, tags, stashes, rebase, cherry-pick |
| [Stage et commit](../../docs/user/fr/staging-and-committing.md) | Statut, diff, hunks et lignes, le compositeur, l’éditeur |
| [Préférences](../../docs/user/fr/settings.md) | Chaque page et chaque option des Préférences |
| [Assistance IA](../../docs/user/fr/ai.md) | Configurer l’IA intégrée, chaque action IA, confidentialité |
| [Hébergeurs et suivi de tickets](../../docs/user/fr/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine : jetons, lier un dépôt, pull et merge requests |
| [CLI](../../docs/user/fr/cli-reference.md) et [MCP](../../docs/user/fr/mcp-and-agents.md) | Le scripter, ou laisser un agent le piloter (permissions, chaque outil) |
| [Raccourcis clavier](../../docs/user/fr/keybindings.md) (et chaque commande de la palette), [souris](../../docs/user/fr/mouse-and-gestures.md), [configuration](../../docs/user/fr/config-reference.md), [LFS](../../docs/user/fr/lfs.md), [annuler](../../docs/user/fr/undo-redo.md), [dépannage](../../docs/user/fr/troubleshooting.md) | Référence et aide |

Développeurs : [CLAUDE.md](../../CLAUDE.md) et `docs/architecture/overview.md`
décrivent les crates et la conception ; [CHANGELOG.md](../../CHANGELOG.md) liste le
contenu de chaque release.

## Licence

[MIT](../../LICENSE).

## Crédits et licences tierces

GitPoulpi repose sur des bibliothèques open source et utilise des valeurs de couleur issues de thèmes publiés. Leurs licences et crédits, auteurs des thèmes compris, figurent dans [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt). Ce fichier est livré avec chaque version ; la fenêtre À propos et `gitpoulpi licenses` l'affichent aussi.

## Marques

GitKraken est une marque de son propriétaire respectif. GitPoulpi est un projet indépendant, ni affilié à ce propriétaire, ni approuvé ou parrainé par lui ; le nom n'est cité que pour décrire la compatibilité et l'inspiration.
