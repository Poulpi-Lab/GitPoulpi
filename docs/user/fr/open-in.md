[English](../open-in.md) · [Čeština](../cs/open-in.md) · [Deutsch](../de/open-in.md) · [Español](../es/open-in.md) · **Français** · [Italiano](../it/open-in.md) · [日本語](../ja/open-in.md) · [한국어](../ko/open-in.md) · [Nederlands](../nl/open-in.md) · [Polski](../pl/open-in.md) · [Português (Brasil)](../pt-BR/open-in.md) · [Português (Portugal)](../pt-PT/open-in.md) · [Русский](../ru/open-in.md) · [Svenska](../sv/open-in.md) · [Türkçe](../tr/open-in.md) · [Українська](../uk/open-in.md) · [简体中文](../zh-CN/open-in.md)

# Ouvrir dans

Le bouton **Ouvrir dans ▾** de la barre de titre, à côté du chemin du dépôt, ouvre le dossier de l'onglet actif
ailleurs : votre terminal, un nouvel onglet ou volet de multiplexeur, un éditeur,
le gestionnaire de fichiers. Vous choisissez ce qui est dans la liste. Rien
n'est câblé en dur : chaque entrée est un libellé et une commande.

## L'utiliser

- **Cliquez sur Ouvrir dans ▾** : une liste se déroule. Cliquez sur une entrée
  pour la lancer. L'entrée par défaut est marquée d'un `•` ; la dernière ligne,
  **Configurer...**, ouvre les [Paramètres](settings.md#ouvrir-dans) sur la page Ouvrir dans.

  ![La liste Ouvrir dans : Terminal, deux éditeurs, un outil Git et le gestionnaire de fichiers, le défaut marqué d'une puce, et Configurer à la fin](../images/open-in-menu.png)

- **Clavier** : `Alt+O` ouvre la même liste (`Up`/`Down`, `Enter`, `Esc` ;
  au-delà de sept entrées, tapez pour chercher). `Ctrl+Alt+O` lance
  immédiatement l'entrée par défaut. Palette (`Ctrl+P`) : `Onglet : Ouvrir dans...` et
  `Onglet : Ouvrir dans l'entrée par défaut`. Les commandes de la palette fonctionnent même
  quand la barre de titre n'a pas de place pour le bouton. Les deux touches sont
  réaffectables ; voir [Raccourcis clavier](keybindings.md).
- **Clic droit sur un onglet de dépôt** : le menu de l'onglet a un sous-menu
  **Ouvrir dans** avec les mêmes entrées.
- La liste démarre sur l'entrée que vous avez utilisée en dernier dans cette
  session.
- La ligne d'état affiche aussitôt « ouverture dans ... ». Si le programme échoue
  en moins de 1,5 seconde environ (par exemple tmux sans serveur, ou une
  option incorrecte), la première ligne de sa sortie d'erreur apparaît sur la
  ligne d'état. Un programme qui n'est pas installé est nommé aussitôt.

Tant que vous n'avez pas enregistré votre propre liste, l'entrée par défaut est choisie d'après ce qui est détecté : dans herdr, « Herdr: new tab » ; sur Omarchy, la nouvelle fenêtre de kitty (kitty est son terminal) ; sinon le gestionnaire de fichiers.

Le bouton s'affiche sur chaque onglet de dépôt, y compris le dossier où GitPoulpi
a été lancé, et en mode `ui.tab_dir_follow = "auto"`. En mode `auto`, l'entrée par
défaut s'exécute aussi toute seule quand vous ouvrez un nouvel onglet (pas pour
les onglets restaurés au démarrage). Voir
[Suivre le dossier de l'onglet actif](following-the-tab-folder.md).

## À quoi ressemble une entrée

Dans `config.toml`, une table `[[ui.open_in]]` par entrée :

```toml
[ui]
open_in_default = "Herdr: new tab"   # a label; empty means the first entry

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Vous l'écrivez rarement à la main : la page des Paramètres la modifie. Chaque clé
est dans la [référence de configuration](config-reference.md).

### Commandes et variables

La commande est d'abord découpée en arguments (les espaces, `'...'` et `"..."`
regroupent les mots), puis les variables sont remplacées dans chaque argument.
Un dossier dont le nom contient une espace reste donc un seul argument.
**Aucun shell n'intervient** sauf si votre commande en lance un.

| Variable | Remplacée par |
|---|---|
| `${cwd}` | Le dossier de l'onglet. |
| `${repo}` | La racine du dépôt (diffère de `${cwd}` pour un onglet de worktree). |
| `${name}` | Le nom de base du dossier. |
| `${branch}` | La branche extraite ; vide en HEAD détaché. |
| `${web_url}` | La page web du remote origin quand c'est GitHub ou GitLab ; vide sinon. |
| `${env:NAME}` | La variable d'environnement `NAME` ; vide si elle n'est pas définie. |

L'ancien `{repo}` fonctionne encore et signifie `${cwd}`. Tout autre `${...}`
est une erreur qui nomme la variable.

Le programme est lancé avec le dossier de l'onglet comme dossier de travail, et
reçoit trois variables d'environnement, pratiques pour un extrait de shell :

| Variable | Valeur |
|---|---|
| `GITPOULPI_CWD` | Le dossier de l'onglet. |
| `GITPOULPI_REPO` | La racine du dépôt. |
| `GITPOULPI_BRANCH` | La branche, ou vide. |

```toml
[[ui.open_in]]
label = "Terminal here, then status"
command = "sh -c 'cd \"$GITPOULPI_CWD\" && git status; exec $SHELL'"
```

Sous Windows, le programme est recherché via `PATH` et `PATHEXT`, si bien que
`code` et `cursor` (des fichiers `.cmd`) fonctionnent, sans qu'une fenêtre de
console ne clignote.

## Préréglages

Paramètres > Ouvrir dans > **Ajouter un préréglage** liste des entrées toutes
prêtes pour votre système d'exploitation seulement (les préréglages Windows
n'apparaissent jamais sous Linux, et ainsi de suite), par catégorie : la ligne
commence par la catégorie, `Explorateur : ...`, `Multiplexeur : ...`, `Terminal : ...`,
`Éditeur : ...`, `Outil Git : ...`, `Agent IA : ...`, `Web : ...`. En haut, **Détecté** regroupe les
entrées de la liste par défaut calculée. Dans chaque catégorie, les préréglages
utilisables tout de suite viennent en premier (vous tournez dans ce
multiplexeur ou ce terminal, et le programme est dans le `PATH`), puis les
installés, puis le reste. Tapez dans la liste déroulante pour chercher, par
exemple `lazygit` ou `Outil Git :`. **Tester sur l'onglet actuel** essaie une
entrée avant de vous y fier.

![La liste + Ajouter de la page Ouvrir dans : les entrées détectées d'abord, puis Explorateur, Multiplexeur et les autres catégories, chaque préréglage préfixé de sa catégorie](../images/settings-open-in-presets.png)

Les préréglages « lancer un outil dans un volet » (`lazygit in Herdr tab`,
`yazi in tmux window`, `Claude Code in Herdr split`...) ne sont proposés que
quand l'hôte (herdr, tmux, zellij, WezTerm, kitty, `xdg-terminal-exec`,
Windows Terminal) et l'outil sont installés. `$EDITOR in ...` lance l'éditeur
nommé par votre variable `EDITOR` (avec repli sur `vi` dans herdr) dans herdr
ou tmux.

Les préréglages d'outils de herdr créent l'onglet ou la séparation, puis
saisissent la commande dans son shell, de sorte que l'outil démarre dans le
dossier et que le volet reste ouvert après sa sortie. Ceux d'`Agent IA`
utilisent `herdr agent start`, si bien que herdr suit l'état de l'agent.

| Catégorie | Préréglages (`V` = vérifié en l'exécutant ou d'après l'aide installée) |
|---|---|
| Explorateur | Windows `explorer.exe` (V), Finder macOS (V), Linux `xdg-open` (V), Nautilus, Dolphin, Thunar, Nemo, PCManFM ; yazi, superfile, nnn, lf, ranger dans un volet/onglet/terminal |
| Multiplexeur | herdr onglet, espace de travail, séparation droite/bas (V) ; tmux fenêtre, séparation droite/bas, session détachée (V) ; zellij volet, onglet, flottant ; WezTerm onglet, séparation, nouvelle fenêtre ; kitty onglet, fenêtre séparée, fenêtre de l'OS (V : options d'après l'aide ; demande `allow_remote_control`) ; screen ; cmux ; psmux |
| Terminal | `xdg-terminal-exec` (V), kitty (V), foot (V), Alacritty (+ `alacritty msg`), WezTerm, Ghostty (Linux, macOS), GNOME Terminal, Ptyxis, Konsole, Xfce Terminal, Tilix, Terminator, MATE Terminal, iTerm2, Terminal.app, Warp ; Windows : Windows Terminal (fenêtre, onglet, séparation, et un onglet chacun pour PowerShell 7, Windows PowerShell, Invite de commandes, WSL), fenêtres PowerShell 7 / Windows PowerShell / Invite de commandes / WSL, Git Bash |
| Éditeur | VS Code (V), VSCodium, Cursor, Windsurf, Zed, Sublime Text, Emacs, IntelliJ IDEA, RustRover, PyCharm, WebStorm, CLion, GoLand, Rider, PhpStorm, Android Studio, Visual Studio (Windows) ; Neovim, Vim, Helix, `$EDITOR` dans un volet/onglet/terminal |
| Outil Git | gitk, `git gui`, gitg, Sublime Merge, GitKraken, GitHub Desktop ; lazygit, tig, gitui dans un volet/onglet/terminal (ceux de herdr, tmux, kitty et `xdg-terminal-exec` en V) |
| Agent IA | Claude Code, Codex, Gemini CLI, opencode dans un onglet/séparation herdr (V, avec Claude), tmux, zellij, WezTerm, kitty, nouveau terminal |
| Web | `Repository page on the web` (`${web_url}`), `gh repo view --web` (V), `glab repo view --web` |

WSL sous Windows : `wsl.exe --cd ${cwd}` accepte tel quel le chemin Windows, et
l'onglet `wsl.exe` de Windows Terminal démarre dans le dossier, que WSL place
sous `/mnt/`.

L'explorateur, Windows Terminal, PowerShell, `cmd`, WSL, Warp, psmux, Visual
Studio et Zed (Zed est réservé à Unix) n'apparaissent que là où ils existent ;
tmux, zellij, cmux, screen, le contrôle à distance de kitty et les préréglages
herdr pilotés par `sh` sont réservés à Unix ; herdr (préréglages simples),
WezTerm, Alacritty, VS Code, Cursor et les IDE JetBrains sont proposés partout.

### Variable `${web_url}`

La page web du remote `origin` du dépôt, quand c'est un projet GitHub ou GitLab
(github.com, gitlab.com, ou un hôte auto-hébergé qui a une entrée dans
`[[integrations]]`) ; vide sinon, et alors une entrée qui l'utilise s'arrête avec
un message au lieu d'ouvrir une adresse vide. Elle est facultative : aucun
préréglage n'en a besoin sauf `Repository page on the web`.

## La page des Paramètres

Paramètres > **Ouvrir dans** modifie la liste. Les entrées forment une [liste](settings.md#listes) à
gauche (l'entrée par défaut est marquée `★`), les champs de l'entrée sélectionnée sont à droite ; `+ Ajouter`,
`Dupliquer`, `Supprimer`, `↑` / `↓` et le glissement d'une ligne gèrent la liste.

| Ligne | Ce qu'elle fait |
|---|---|
| **Entrée par défaut** | L'entrée que lance **Ouvrir dans par défaut**. Elle est marquée `•` dans la liste Ouvrir dans. |
| **Libellé** | Le nom affiché dans la liste. Unique, non vide. |
| **Commande** | La commande, avec les pastilles de variables (`${cwd}`, `${repo}`, `${branch}`, `${name}`, `${web_url}`) à insérer. |
| **Tester sur l'onglet actuel** | Lance l'entrée maintenant sur le dossier de l'onglet actif. |
| **+ Ajouter ▾** | Ajoute une entrée toute prête (les détectées d'abord), ou **Entrée personnalisée**, une entrée vide dont la commande se saisit ensuite. |
| **Restaurer les valeurs détectées** | Remplace la liste par ce qui est détecté sur cette machine et efface le défaut. |

Les modifications sont enregistrées au fur et à mesure. La première modification
écrit tout le tableau `[[ui.open_in]]` dans `config.toml` ; jusque-là la liste
est calculée (voir [Anciennes clés](#anciennes-clés)) et le fichier n'a pas de
clé `open_in`. La recherche des Paramètres trouve aussi la page.

## Fichiers binaires : Ouvrir dans

Un fichier binaire (un PDF, un document Word, un tableur, une archive...) n'a pas de texte à montrer dans la vue Diff ou la vue Fichier. Quand le système d'exploitation connaît une application pour son type, la vue affiche un bouton **Ouvrir dans** à côté de l'avis habituel. Sa liste contient l'application avec laquelle le système ouvre ce type, en premier et marquée d'une puce, puis toutes les autres applications que le système y associe (la liste *Ouvrir avec* du système). Sous Linux, c'est le type MIME du nom de fichier, `mimeapps.list` pour le défaut et les fichiers `.desktop` pour les autres ; sous Windows, le registre (le choix de l'utilisateur et les programmes associés à l'extension) ; sous macOS, l'application par défaut seulement.

`o` lance l'application par défaut et `Shift+O` ouvre la liste. Les deux sont dans la palette de commandes (*Fichier : Ouvrir dans l'application par défaut*, *Fichier : Ouvrir avec...*), et **Ouvrir avec...** est dans le menu clic droit d'un fichier du volet Statut et de la liste des fichiers d'un commit. La recherche tourne en arrière-plan, une fois par extension : la vue affiche « recherche des applications... » pendant ce temps. Un type pour lequel le système ne connaît aucune application, et le texte brut, gardent l'avis simple et n'ont pas de bouton.

![Un PDF dans la vue Fichier avec sa liste Ouvrir dans : l'application par défaut en premier, marquée d'une puce, puis les autres applications que le système associe au type](../images/open-with.png)

Le fichier de l'arbre de travail est transmis tel quel à l'application. La version d'un commit (ou de l'index) est écrite dans une **copie temporaire en lecture seule**, avec un message dans la ligne d'état, pour qu'une application ne puisse pas modifier le dépôt. Aucun des deux ne modifie le dépôt : pas d'entrée de journal, l'Annulation n'est pas concernée.

## Anciennes clés

Les anciennes clés `ui.terminal_launch` et `ui.terminal_launch_command` ne sont
lues que tant que `ui.open_in` est absente. La liste calculée est alors :

1. votre ancien choix de terminal, s'il n'est pas `auto` (`custom` devient
   « Terminal (custom) » avec `{repo}` réécrit en `${cwd}` ; un identifiant de
   préréglage devient ce préréglage). Une valeur `auto` devient la première
   entrée, « Default terminal ».
2. l'explorateur de fichiers de l'OS,
3. les multiplexeurs et terminaux détectés d'après l'environnement,
4. les éditeurs trouvés dans le `PATH`.

Quand vous enregistrez la liste depuis les Paramètres, `ui.open_in` prend le
relais et les anciennes clés sont ignorées. Les commandes de palette **Onglet : Ouvrir dans un terminal** et **Onglet : Ouvrir dans
le gestionnaire de fichiers** n'utilisent pas cette liste.

## Dépannage

- *« {label} : programme X introuvable (PATH vérifié) »* : le programme n'est pas dans le `PATH` vu par
  GitPoulpi. Lancez GitPoulpi depuis un shell qui l'a, ou écrivez le chemin
  complet.
- *l'entrée tmux/zellij/herdr échoue aussitôt* : ils parlent à un serveur en
  cours d'exécution, ils ne fonctionnent donc que si GitPoulpi tourne dedans.
- *l'entrée d'onglet kitty échoue* : elle demande `allow_remote_control` dans
  `kitty.conf`.
- *Explorateur Windows* : `explorer.exe` se termine avec le code 1 même en cas de
  succès ; GitPoulpi le traite comme un succès.
