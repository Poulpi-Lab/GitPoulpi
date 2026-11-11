[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · **Français** · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Choisir un terminal

GitPoulpi fonctionne dans n'importe quel terminal qui gère la souris, mais il est
le plus agréable dans un terminal qui dessine des images, affiche la vraie
couleur et envoie le clavier complet. Cette page classe les terminaux par
système, dit ce que chacun apporte et quel paramètre changer. Les faits ont été
vérifiés dans la documentation, le journal des changements ou le suivi de
tickets de chaque projet en octobre 2026 (voir [Sources](#sources)) ; tout
n'a pas été testé sur chaque terminal. Tout ce qui est marqué **(non vérifié)**
n'a pas pu être confirmé par une source primaire.

## Les meilleurs choix

| Système | Meilleurs choix, dans l'ordre | Pourquoi |
|---|---|---|
| **Windows** | **WezTerm**, puis **Windows Terminal** | WezTerm dessine le graphe et les avatars sous forme d'images, avec transparence. Windows Terminal a la meilleure intégration au système (c'est le choix par défaut du lanceur), il dessine le graphe et les avatars en images sixel (sans transparence). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, puis **foot** | kitty et Ghostty donnent des images et le clavier complet. WezTerm donne aussi des images. foot dessine le graphe et les avatars en images sixel, il est rapide et a le clavier complet. Sous Omarchy, choisissez Ghostty ou kitty pour avoir des images. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, puis **iTerm2** | Même raisonnement. Terminal.app fonctionne, mais n'a ni images ni vraie couleur. |

Quel que soit le système, **tmux, zellij et screen** cachent à GitPoulpi le
protocole d'images du terminal, si bien qu'il dessine des caractères (voir
[Multiplexeurs](#multiplexeurs)).

## Ce que GitPoulpi utilise

- **Images** (colonne du graphe, avatars des auteurs, diff d'image et vue
  Fichier) : le **protocole graphique de kitty** avec des espaces réservés
  Unicode, les **images en ligne d'iTerm2** (`OSC 1337`) ou **sixel**, dans cet
  ordre de préférence. La détection refuse par défaut ; elle s'appuie sur
  `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`,
  `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` et les variables des
  multiplexeurs, et ne lit jamais le clavier (elle ne demande donc pas non plus
  ses attributs d'appareil au terminal). kitty et Ghostty utilisent le
  protocole kitty. WezTerm utilise le protocole d'iTerm2 (volontairement) et
  mélange la transparence du PNG. iTerm2, Rio et mintty utilisent le protocole
  d'iTerm2 avec un fond opaque. Windows Terminal, foot, Konsole et mlterm
  utilisent **sixel**. Sans aucun de ces protocoles, les avatars sont le badge
  d'initiales.
- **Sixel** est dessiné pour la **colonne du graphe** (voies, nœuds et, avec `ui.avatars = "graph"`, les avatars dans les ronds des commits) et ailleurs pour les **avatars** (Log, détail du commit, Historique du fichier, Blame, vue d'image). Un sixel n'a ni alpha, ni identifiant, ni mise à l'échelle : l'image fait exactement `colonnes x largeur de cellule` sur `lignes x hauteur de cellule` pixels, n'est dessinée que là où il y a quelque chose, sur des cellules vides qui gardent leur fond ; l'image encodée est mise en cache, réencodée seulement quand les voies, le défilement ou le zoom changent, et renvoyée seulement quand une cellule dessous a été repeinte. Un sixel ne se supprime pas : GitPoulpi écrit du texte sur ses cellules pour l'enlever. Le sixel a besoin de la vraie taille de cellule en pixels : on la demande une fois à Windows Terminal au démarrage et après un zoom (`CSI 16 t`, réponse en 10 ms environ, jamais bloquant s'il se tait) ; ailleurs elle vient de la taille de la fenêtre ou de `GITPOULPI_TERMINAL_CELL_PX=10x20`. Un terminal qui dessine le sixel sans être reconnu (un terminal de VS Code avec `terminal.integrated.enableImages`, un xterm lancé avec `-ti vt340`) demande `ui.image_protocol = "sixel"`. Windows Terminal avant 1.22 ne dessine pas le sixel ; réglez-y `ui.image_protocol = "none"` (le graphe reste alors en caractères).
- **Vraie couleur** : les thèmes sont en RVB. `ui.color_depth = "auto"` passe à
  256 couleurs sur un `TERM` en `*-256color` sans `COLORTERM=truecolor` (et dans
  Terminal.app), et à 16 couleurs sur la console Linux. kitty, WezTerm et
  Windows Terminal comptent comme vraie couleur.
- **Souris** : le rapport de souris SGR avec boutons, glissement et mouvement
  (retour au survol, glisser-déposer, clic droit, double-clic).
- **Clavier** : le **protocole clavier de kitty** (seulement l'indicateur de
  désambiguïsation) est demandé au démarrage avec un délai de 100 ms. Il fait
  arriver `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` et `Ctrl+Shift+Z`. Sans lui, les
  touches classiques restent (`Ctrl+Y` rétablit, des variantes `Alt` remplacent
  certains enchaînements). Voir
  [Raccourcis clavier](keybindings.md#touches-que-le-terminal-ne-peut-pas-envoyer-et-le-protocole-clavier-kitty).
- **Presse-papiers** : les actions de copie écrivent `OSC 52`.
- **Forme du pointeur de souris** (pointeur de redimensionnement sur les
  séparateurs, main sur les éléments cliquables) : `OSC 22`, un plus là où il
  est implémenté.
- **Les événements de focus** sont activés (les infobulles se ferment quand la
  fenêtre perd le focus). GitPoulpi ne demande ni le collage entre crochets, ni
  les liens hypertexte OSC 8, ni le soulignement ondulé. Sur le chemin des
  images seulement, chaque trame est enveloppée dans une mise à jour
  synchronisée (mode 2026, ignoré par les terminaux qui ne le connaissent pas)
  pour qu'une image ne clignote jamais par-dessus une fenêtre contextuelle.
- **Emoji et symboles** : GitPoulpi n'utilise que des glyphes à présentation
  emoji par défaut ou des formes BMP de largeur simple, car les terminaux et les
  polices ne s'accordent pas sur les sélecteurs de variation.

## Tableau des capacités

« Images » désigne ce que GitPoulpi dessine aujourd'hui dans ce terminal
(protocole **kitty**, **iTerm2** ou **sixel**, ou **texte**), pas ce que le terminal saurait
faire.

| Terminal | Système | Images (graphe, avatars, diff d'image) | Vraie couleur | Clavier kitty (enchaînements complets) | Copie OSC 52 | Pointeur OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | oui | oui | oui | oui |
| **Ghostty** (1.3) | Linux, macOS (pas de version Windows) | kitty | oui | oui | oui | oui, depuis 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, avec transparence | oui | seulement avec `enable_kitty_keyboard = true` (par défaut `false`) | oui (écriture seule) | non (la pull request est toujours ouverte) |
| **foot** (1.28) | Linux (Wayland) | sixel (graphe et avatars ; pas d'images kitty) | oui | oui, depuis 1.10.3 | oui (`security.osc52`) | oui, depuis 1.12 |
| **Alacritty** (0.17) | tous | texte (pas de prise en charge des images en amont) | oui | oui, depuis 0.13 | oui (écriture ; collage désactivé par défaut) | non (non implémenté) |
| **Konsole** | Linux | sixel (graphe et avatars) | oui | en cours d'ajout en 2026 (merge request !1188) ; version (non vérifié) | écriture seule, fusionné mi-2024 | (non vérifié) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | texte | oui | non (pas dans une version publiée de VTE) | non (VTE l'ignore) | non |
| **iTerm2** (3.7) | macOS | iTerm2, fond opaque | oui | oui, depuis 3.6.9 (vérifiez Profiles > Keys) | désactivée par défaut : activez « Applications in terminal may access clipboard » | (non vérifié) |
| **Terminal.app** | macOS | texte | non (256 couleurs) | non | (non vérifié) | (non vérifié) |
| **Windows Terminal** (1.25) | Windows | sixel (graphe et avatars) | oui | oui, stable depuis 1.25 (oct. 2026) ; Preview depuis 1.25 (mars 2026) | oui (écriture ; seulement avec le focus) | non (non implémenté) |
| **Rio** | Windows, Linux, macOS | iTerm2, fond opaque (il parle aussi les images kitty) | oui | oui (activé par défaut) | oui | non (non implémenté) |
| **Warp** | macOS, Linux, Windows | texte (non détecté) | oui | oui | oui (il respecte la séquence) | (non vérifié) |
| **Terminal de VS Code** (xterm.js) | tous | texte (sixel et les images iTerm2 existent derrière `terminal.integrated.enableImages` ; `ui.image_protocol = "sixel"` utilise le sixel) | oui | (non vérifié) | (non vérifié) | non |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, fond opaque | oui | (non vérifié ; non documenté) | (non vérifié ; non documenté) | oui |
| **Console Windows** (conhost, cmd, fenêtre PowerShell) | Windows | texte | seulement Windows 10/11 récents | inutile : elle rapporte elle-même les modificateurs | non | non |
| **ConEmu** | Windows | texte | (non vérifié) | (non vérifié) | (non vérifié) | (non vérifié) |
| **tmux, zellij, screen** | tous | texte, toujours | demande des réglages | demande des réglages | demande des réglages | non |

Les cellules « non (non implémenté) » pour OSC 22 viennent de l'absence dans le
journal des changements et la documentation d'un projet, plus des retours
d'utilisateurs, pas d'un test ; lisez-les comme « non documenté ». La page ne
tabule pas la largeur des emoji et des sélecteurs de variation : elle diffère
selon le terminal et la police, c'est pourquoi GitPoulpi évite les glyphes VS16
(voir [Ce que GitPoulpi utilise](#ce-que-gitpoulpi-utilise)).

## Terminal par terminal

### Windows

- **WezTerm.** La meilleure prise en charge des images sous Windows. Laissez
  `ui.image_protocol` sur `auto`. Dans `wezterm.lua`, mettez
  `config.enable_kitty_keyboard = true` pour avoir les enchaînements complets, et
  une police avec repli emoji, par exemple
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  La dernière version stable étiquetée de WezTerm est 20240203-110809-5046fc22
  (févr. 2024) ; les correctifs plus récents arrivent dans les versions
  nightly. WezTerm n'a pas encore de forme de pointeur OSC 22.
- **Windows Terminal.** Vraie couleur, souris, OSC 52 et de bons emoji.
  GitPoulpi y dessine les avatars en images sixel (Windows Terminal 1.22 ou
  plus récent) ; le graphe est en caractères. Choisissez une Nerd Font
  dans *Settings > Profiles > Defaults > Appearance > Font face*. Maintenez
  `Shift` pour sélectionner du texte pendant que la souris est capturée. La forme
  du pointeur ne change pas. Sur une version antérieure à 1.25 (sans protocole
  clavier kitty), utilisez les variantes `Alt`.
- **Rio** fonctionne avec un fond d'image opaque ; moins testé.
- **mintty** (Git Bash, MSYS2, Cygwin) dessine des images par le protocole
  d'iTerm2 avec un fond opaque ; moins testé.
- **Console Windows, ConEmu, cmd** : à éviter. Les images sont mal dessinées ou
  pas du tout. Un double-clic sur l'exe rouvre GitPoulpi dans le terminal que
  vous avez choisi (*Paramètres > Lanceur*).

### Linux

- **kitty.** Tout fonctionne sans réglage. `Shift+clic` est celui de kitty
  lui-même, utilisez donc `Alt+clic` pour étendre une sélection
  ([Souris](mouse-and-gestures.md)).
- **Ghostty.** Tout fonctionne sans réglage. Si des icônes apparaissent comme des
  trous, ajoutez une police emoji ou Nerd Font à `font-family` (répétez la clé
  pour ajouter des polices de repli).
- **WezTerm.** Des images par le protocole d'iTerm2, transparence mélangée. Même
  remarque `enable_kitty_keyboard` que sous Windows.
- **foot.** Graphe en caractères, clavier complet, rapide. Les avatars
  sont des images sixel, le graphe est en caractères. Mettez
  `font=JetBrainsMono Nerd Font:size=11` dans `foot.ini`.
- **Alacritty.** Graphe en caractères, clavier kitty depuis 0.13. Alacritty ne
  prend pas en charge les images.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Fonctionnent avec des caractères et
  les touches classiques : utilisez les variantes `Alt` et `Ctrl+Y` pour
  rétablir (un Konsole récent peut ajouter le protocole clavier kitty). Les
  actions de copie de VTE (`OSC 52`) ne mettent rien dans le presse-papiers. Une
  méthode de saisie (IBus, Fcitx) peut s'emparer de la bascule de stage
  `Ctrl+Space` - **(non vérifié)** ; réaffectez-la dans *Paramètres > Clavier et souris*.
- **Wayland, Hyprland, Omarchy.** L'indication de pointeur de souris (`OSC 22`)
  fonctionne dans kitty, Ghostty et foot. L'entrée du lanceur Omarchy passe par
  `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm** : comme sous Linux.
- **iTerm2.** Des images avec un fond opaque (un rectangle visible si votre
  arrière-plan diffère de celui du thème ; `ui.graph_renderer = "text"` le
  supprime). Activez *Settings > General > Selection > Applications in terminal
  may access clipboard* pour la copie, et *Profiles > Keys > Left Option key:
  Esc+* pour les enchaînements `Alt`.
- **Terminal.app.** 256 couleurs seulement : GitPoulpi choisit lui-même
  `ui.color_depth = 256` et les thèmes paraissent plus plats. Pas d'images.
  Cochez *Use Option as Meta key* pour les enchaînements `Alt`.

### VS Code et Warp

Le terminal de VS Code (`TERM_PROGRAM=vscode`) et Warp ne sont pas reconnus comme
des terminaux à images, si bien que GitPoulpi dessine des caractères. Les deux
sont utilisables. Warp documente la prise en charge du protocole clavier kitty ;
celle du terminal de VS Code est **(non vérifié)**.

## Multiplexeurs

GitPoulpi détecte tmux, zellij, screen et herdr (`TMUX`, `ZELLIJ`, `STY`,
`TERM=screen*` ou `tmux*`, `HERDR_*`) et dessine des **caractères**, car un
multiplexeur transmet des cellules, pas des séquences d'images. Pour les autres
fonctions, dans `~/.tmux.conf` :

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

Pour forcer les images à travers un multiplexeur qui les transmet, mettez
`ui.image_protocol` sur `kitty` ou `iterm2`.
herdr avec `kitty_graphics = true` dans kitty, Ghostty ou WezTerm est détecté
automatiquement ; voir
[Dépannage](troubleshooting.md#le-graphe-est-dessiné-avec-des-caractères-pas-comme-une-image-herdr-tmux-autres-terminaux).
`GITPOULPI_TERMINAL_IMAGE=off` désactive toutes les images.

tmux ne transmet pas lui-même le protocole clavier kitty : `extended-keys` lui
fait envoyer les touches modifiées sous forme de séquences `CSI u` ou `xterm`.
zellij 0.41 et suivants parle le protocole clavier kitty et transmet OSC 52,
d'après son journal des changements.

## Ce qui se dégrade, et comment

| Ce qui manque | Ce que vous obtenez à la place |
|---|---|
| Protocole d'images | Graphe en caractères. L'avatar devient la pastille d'initiales colorée (une lettre), quoi que dise `ui.avatar_fallback` : un identicon écrasé dans des cellules de texte est méconnaissable. Le diff d'image et la vue Fichier utilisent des demi-blocs colorés, ou une fiche d'information. |
| Vraie couleur | Les couleurs sont ramenées à la palette de 256 ou de 16 couleurs (`ui.color_depth`). |
| Protocole clavier kitty | Touches classiques ; certains enchaînements arrivent comme une autre touche. Les Paramètres avertissent, et vous réaffectez ou utilisez les variantes `Alt`. |
| OSC 52 | Les actions de copie ne mettent rien dans le presse-papiers : sélectionnez à la souris (en maintenant `Shift`) et utilisez la copie propre au terminal. |
| OSC 22 | Le pointeur ne change pas ; le séparateur ou le bouton sous la souris s'allume quand même. |
| Police emoji | Des cases ou des trous à la place des icônes ; le graphe et le texte fonctionnent encore. Installez une Nerd Font ou une police emoji et listez-la comme repli. |

## Paramètres à connaître

| Paramètre | Usage |
|---|---|
| `ui.image_protocol` | `auto` (par défaut), `kitty`, `iterm2`, `sixel`, `none`. Mettez `none` si vous voyez du charabia là où devraient être des images. Redémarrage. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` est toujours correct. Redémarrage. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` ou `off`, si un terminal ou un multiplexeur se comporte mal. Redémarrage. |
| `ui.mouse_enabled` | Désactivé, laisse la souris au terminal. Redémarrage. |
| `ui.launch_terminal` | Le terminal que les raccourcis du système ouvrent ([Paramètres](settings.md)). |

Voir aussi [Images](image-view.md), [Raccourcis clavier](keybindings.md) et
[Dépannage](troubleshooting.md).

## Sources

Vérifiées en octobre 2026. Sources primaires sauf mention contraire.

- kitty : [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty : [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm : [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot : [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty : [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole : [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE : [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2 : [downloads](https://iterm2.com/downloads.html) ; clavier kitty d'après la [matrice terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secondaire)
- Windows Terminal : [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secondaire), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio : [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp : [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps) ; OSC 52 d'après une [analyse de l'avis de sécurité OSC 52 de Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secondaire)
- VS Code : [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty : [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (pas de vraie couleur) : [discussion Apple](https://discussions.apple.com/thread/255315820) (secondaire)
- tmux : [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES) ; zellij : [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Sites de comparaison (secondaires, utilisés seulement pour recouper) : [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Toujours non vérifiés : sixel dans Konsole et mlterm (repris des listes de fonctionnalités des projets, sans revérification sur une version, et jamais essayé dans une vraie fenêtre, quel que soit le terminal) ; OSC 22 dans Konsole, iTerm2, Terminal.app et Warp ; OSC 52
dans Terminal.app, VS Code et mintty ; le protocole clavier kitty dans VS Code,
mintty et ConEmu ; chaque cellule de ConEmu ; et la largeur des emoji par
terminal. La version dans laquelle Konsole livre le protocole clavier kitty n'a
pas été confirmée.
