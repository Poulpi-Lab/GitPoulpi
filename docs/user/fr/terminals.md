[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · **Français** · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Choisir un terminal

GitPoulpi fonctionne dans n'importe quel terminal qui gère la souris, mais il est
le plus agréable dans un terminal qui dessine des images, affiche la vraie
couleur et envoie le clavier complet. Cette page dit ce que chaque terminal
apporte sur chaque système, et quel paramètre changer.

**Comment la lire.** Les tableaux viennent d'une série de tests de **novembre 2026** :
chaque terminal a été piloté avec des séquences d'échappement brutes et, là où il
pouvait l'exécuter, avec GitPoulpi lui-même, sous Linux, Windows 11 et macOS 15. Les
versions sont listées [plus bas](#versions-testées). Chaque cellule est l'une de
ces mentions :

- **fonctionne** ;
- **partiel** (et ce qui manque) ;
- **ne fonctionne pas** (et pourquoi : le *terminal*, la *couche du système* ou *GitPoulpi*) ;
- **n/a** (le terminal n'existe pas sur ce système) ;
- **non mesuré** (et pourquoi).

Les multiplexeurs (tmux, zellij, screen, herdr, psmux, panneaux) ont leur propre
page : [Multiplexeurs](multiplexers.md).

## Les meilleurs choix

| Système | Meilleurs choix, dans l'ordre | Pourquoi |
|---|---|---|
| **Windows** | **Windows Terminal**, puis **WezTerm** et **Rio** | Windows Terminal dessine le graphe et les avatars en images sixel, bien ancrées, et c'est le choix par défaut du lanceur. WezTerm et Rio les dessinent en images iTerm2, ancrées correctement depuis le correctif ConPTY de novembre 2026 (voir [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty et Ghostty donnent des images, les formes de pointeur et le clavier complet. foot dessine des images sixel et a le clavier complet et les formes de pointeur. |
| **macOS** | **iTerm2**, puis **kitty** et **Ghostty** | iTerm2 3.7.7 dessine les images par le protocole kitty. Terminal.app fonctionne, mais n'a ni images ni vraie couleur. kitty et Ghostty se comportent comme sous Linux (non mesuré sous macOS). |

Quel que soit le système, **screen** et **zellij** cachent à GitPoulpi le
protocole d'images du terminal, si bien qu'il dessine des caractères ; tmux laisse
passer le sixel. Voir [Multiplexeurs](multiplexers.md).

## Ce que GitPoulpi utilise

- **Images** (colonne du graphe, avatars des auteurs, diff d'image et vue
  Fichier) : le **protocole graphique de kitty** avec des espaces réservés
  Unicode, les **images en ligne d'iTerm2** (`OSC 1337`) ou **sixel**. La
  détection refuse par défaut ; elle s'appuie sur `TERM`, `TERM_PROGRAM`,
  `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION` et les variables des multiplexeurs. Quand
  l'environnement ne nomme personne (un xterm lancé avec `-ti vt340`, un terminal
  de VS Code avec les images activées, tout terminal atteint par SSH, où
  `TERM_PROGRAM` n'est pas transmis), ce sont les réponses du terminal qui
  décident : GitPoulpi interroge une fois au démarrage (voir « La requête de
  démarrage » plus bas) et utilise le nom XTVERSION du terminal (kitty, Ghostty,
  WezTerm, iTerm2, mintty, foot) ou sixel dans ses attributs d'appareil. Dans un
  multiplexeur, les réponses sont ignorées, car tmux se déclare sixel quoi qu'il
  y ait derrière lui.
- **Quel protocole où.** kitty, Ghostty et iTerm2 3.7.7 ou plus récent : le
  protocole kitty. WezTerm et Rio : le protocole d'iTerm2, qui mélange la
  transparence du PNG. mintty et iTerm2 antérieur à la 3.7.7 : le protocole
  d'iTerm2 avec un fond opaque. Windows Terminal, foot, Tabby, Konsole et
  mlterm : **sixel**. Tout le reste : des caractères, et l'avatar devient le
  badge d'initiales. `ui.image_protocol` en force un.
- **iTerm2 3.7.7.** Chaque image en ligne `OSC 1337` ouvre une boîte de dialogue
  modale « Allow Terminal-Initiated Display? » qui bloque la fenêtre tant que
  vous n'avez pas coché « Remember my choice ». Les mêmes images envoyées par le
  protocole kitty, espaces réservés Unicode compris, s'affichent sans question :
  GitPoulpi y utilise donc le protocole kitty. `ui.image_protocol = "iterm2"`
  donne quand même les images en ligne.
- **Sixel** est dessiné pour la **colonne du graphe** (voies, nœuds et, avec
  `ui.avatars = "graph"`, les avatars dans les ronds des commits) et ailleurs
  pour les **avatars** (Log, détail du commit, Historique du fichier, Blame, vue
  d'image). Un sixel n'a ni alpha, ni identifiant, ni mise à l'échelle : l'image
  fait exactement `colonnes x largeur de cellule` sur `lignes x hauteur de
  cellule` pixels, n'est dessinée que là où il y a quelque chose, sur des
  cellules vides qui gardent leur fond ; elle n'est réencodée que quand les
  voies, le défilement ou le zoom changent. Un sixel ne se supprime pas :
  GitPoulpi écrit du texte sur ses cellules pour l'enlever. Le sixel a besoin de
  la vraie taille de cellule en pixels : Windows Terminal, Tabby et mintty
  répondent une fois à `CSI 16 t` au démarrage et après un zoom (sans jamais
  bloquer s'ils se taisent) ; ailleurs elle vient de la taille de la fenêtre du
  terminal, ou de `GITPOULPI_TERMINAL_CELL_PX=10x20`. Un terminal qui dessine le
  sixel sans le dire demande `ui.image_protocol = "sixel"`. Windows Terminal
  avant 1.22 ne dessine pas le sixel ; réglez-y `ui.image_protocol = "none"`.
- **Vraie couleur** : les thèmes sont en RVB. `ui.color_depth = "auto"` passe à
  256 couleurs sur un `TERM` en `*-256color` sans `COLORTERM=truecolor` (et dans
  Terminal.app, qui n'a pas la vraie couleur), et à 16 couleurs sur la console
  Linux.
- **Souris** : le rapport de souris SGR avec boutons, glissement et mouvement
  (retour au survol, glisser-déposer, clic droit, double-clic).
- **La requête de démarrage.** Quatre questions partent en une seule écriture :
  les indicateurs du clavier kitty (`CSI ? u`), la couleur de fond (`OSC 11`,
  utilisée seulement pour `ui.theme = system`), le nom du terminal (XTVERSION,
  `CSI > 0 q`) et ses attributs d'appareil (`CSI c`, en dernier : tous les
  terminaux y répondent, ce qui clôt la lecture). La lecture est une boucle
  `poll` unique avec une échéance de 100 ms, terminée avant le démarrage du
  lecteur de clavier. Un terminal qui ne répond rien coûte une fois 100 ms et tout
  reste comme le dit l'environnement. Unix seulement : sous Windows, la console
  rapporte elle-même les touches et l'environnement décide. Les réponses figurent
  dans le rapport d'environnement du Journal d'activité (ligne `start-up query:`).
- **Clavier** : le **protocole clavier de kitty** (seulement l'indicateur de
  désambiguïsation) est demandé au démarrage. Il fait arriver `Ctrl+,`,
  `Ctrl+Enter`, `Ctrl+Shift+N` et `Ctrl+Shift+Z`. Sans lui, les touches
  classiques restent (`Ctrl+Y` rétablit, des variantes `Alt` remplacent certains
  enchaînements). Voir
  [Raccourcis clavier](keybindings.md#touches-que-le-terminal-ne-peut-pas-envoyer-et-le-protocole-clavier-kitty).
- **Presse-papiers** : toute action de copie suit une seule règle. En local, le
  **presse-papiers du système** de la machine où tourne GitPoulpi, plus `OSC 52`
  vers le terminal quand on sait que le terminal le respecte (kitty, WezTerm,
  Ghostty, foot, Alacritty, Windows Terminal, Rio, Tabby, iTerm2) ; `OSC 52`
  sert aussi de repli quand le presse-papiers du système échoue. **Par SSH, ou
  sans affichage** (une console Linux, un conteneur), seul `OSC 52` est envoyé, et
  dans tmux il l'est enveloppé pour le passthrough aussi bien que tel quel. La
  ligne d'état n'affiche « copié » que quand le presse-papiers du système a pris
  le texte ; ce qui est parti vers le terminal dit « envoyé ... au presse-papiers
  du terminal (OSC 52, non confirmé) », car rien ne confirme que le terminal a
  obéi. Le rapport d'environnement du Journal d'activité nomme la voie dans sa
  ligne `clipboard:`.
- **Forme du pointeur de souris** (pointeur de redimensionnement sur les
  séparateurs, main sur les éléments cliquables, I-beam sur les champs de texte) :
  `OSC 22`, un plus là où le terminal l'implémente. GitPoulpi l'envoie toujours
  (sauf sur la console Linux et derrière screen, zellij et herdr, qui l'avalent
  ou l'impriment ; derrière tmux, il est enveloppé pour le passthrough). Avec
  `TERM_PROGRAM=mintty`, il envoie les noms de curseur propres à mintty (`hand`,
  `ibeam`, `sizewe`, ...), les seuls que mintty lit. **Rio n'implémente `OSC 22`
  sur aucun système** : son code source analyse la séquence puis la jette, aucun
  réglage d'un côté ni de l'autre n'y change rien.
- **Curseur de texte** : avec `ui.text_cursor = "terminal"` (le défaut), le
  curseur du terminal lui-même est placé au point d'insertion du champ de texte
  actif, en barre clignotante (`DECSCUSR`), et masqué ailleurs ; votre propre
  style de curseur revient à la sortie. `drawn` dessine à la place le curseur
  propre à GitPoulpi.
- **Les événements de focus** sont activés (les infobulles se ferment quand la
  fenêtre perd le focus). Le **collage entre crochets** est demandé au démarrage
  et relâché à chaque sortie (Unix), si bien qu'un collage de plusieurs lignes
  arrive en un seul bloc : ses retours à la ligne ne valident jamais un champ et
  ses lettres ne déclenchent jamais de raccourcis. Sous Windows, la console n'a pas
  de crochets de collage ; voir [Windows](#windows) pour ce que GitPoulpi fait à
  la place. GitPoulpi ne demande ni les liens hypertexte OSC 8 ni le soulignement
  ondulé. Sur le chemin des images seulement, chaque trame est enveloppée dans une
  mise à jour synchronisée (mode 2026, ignoré par les terminaux qui ne le
  connaissent pas) pour qu'une image ne clignote jamais par-dessus une fenêtre
  contextuelle.
- **Emoji et symboles** : GitPoulpi n'utilise que des glyphes à présentation
  emoji par défaut ou des formes BMP de largeur simple, car les terminaux et les
  polices ne s'accordent pas sur les sélecteurs de variation.

## La matrice

Les lignes sont les terminaux, les colonnes les systèmes. « GitPoulpi » dans une
cellule signifie que GitPoulpi lui-même y a tourné ; « sonde » que seules des
séquences d'échappement brutes ont été envoyées. Cellules **n/a** : kitty n'a pas
de version Windows native (les versions publiées couvrent Linux et macOS) ;
Ghostty n'a pas de version Windows (seule sa bibliothèque libghostty est
multiplateforme) ; foot est un terminal Wayland (Linux seulement) ; xterm est un
client X11 (Linux, et macOS via XQuartz) ; iTerm2 et Terminal.app existent
uniquement sous macOS ; Windows Terminal, conhost et mintty uniquement sous Windows
(mintty aussi sous Cygwin et MSYS2).

**Non mesurés sous macOS : kitty, Ghostty, WezTerm, Rio et Warp.** Ils demandent un
GPU (Metal ou OpenGL 3.3). Leurs résultats sous Linux servent de référence pour ce
que fait le même programme ; refaites le test sur un Mac avec GPU avant de vous
fier à une cellule macOS.

### Images : quel protocole GitPoulpi choisit, et s'affiche-t-il

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **fonctionne** : protocole kitty (GitPoulpi) | n/a | non mesuré (demande un GPU) |
| **Ghostty** | **fonctionne** : protocole kitty (GitPoulpi) | n/a | non mesuré (demande un GPU) |
| **WezTerm** | **fonctionne** : protocole iTerm2 avec transparence (GitPoulpi). Les espaces réservés s'affichent écrasés, le protocole kitty n'est donc pas utilisé | **fonctionne** : protocole iTerm2 avec transparence, graphe et avatars sur leurs lignes (la version officielle 20240203 et la version avec la PR amont #8092). La 20240203 officielle plaçait les images sur de mauvaises lignes avant que GitPoulpi n'affiche le curseur autour d'elles, voir « La limite de ConPTY ». | non mesuré (demande un GPU) |
| **foot** | **fonctionne** : sixel avec transparence (GitPoulpi) | n/a | n/a |
| **Rio** | **fonctionne** : protocole iTerm2 avec transparence (GitPoulpi) | **fonctionne** : protocole iTerm2 avec transparence, exact à la cellule à toutes les tailles de police essayées (0.5.28 ; GitPoulpi). Le sixel et kitty n'arrivent jamais (ConPTY les jette), il n'y a donc rien à choisir | non mesuré (demande un GPU) |
| **Alacritty** | texte : **fonctionne** (pas de prise en charge des images dans le terminal) | texte : **partiel** (pas d'images ; diff d'image en blocs) | texte : **fonctionne** |
| **Tabby** | **fonctionne** : sixel avec transparence (sonde ; GitPoulpi non lancé dans Tabby sous Linux) | **fonctionne** : sixel, aligné (GitPoulpi) | **fonctionne** : sixel, transparence conservée (GitPoulpi) |
| **Warp** | non mesuré (il demande XWayland et décode mal les frappes synthétiques : rien n'a pu être saisi) | **ne fonctionne pas** (terminal : ni sixel, ni iTerm2, ni images kitty) ; GitPoulpi non mesuré (impossible à lancer depuis la zone de saisie de Warp) | non mesuré (demande un GPU) |
| **xterm** | sonde : le sixel **fonctionne** avec `-ti vt340` ; GitPoulpi le choisit d'après les attributs d'appareil (la requête de démarrage est venue après le test), ou réglez `ui.image_protocol = "sixel"` | n/a | texte : **fonctionne** ; le sixel s'affiche (sonde) et demande `ui.image_protocol = "sixel"` |
| **iTerm2** | n/a | n/a | **fonctionne** : protocole kitty à partir de la 3.7.7, sans boîte de dialogue (GitPoulpi). Les versions plus anciennes reçoivent les images en ligne avec un fond opaque. Le sixel s'affiche mais peint les pixels non définis en opaque, donc `ui.image_protocol = "sixel"` donne un rectangle plein |
| **Terminal.app** | n/a | n/a | texte : **fonctionne** (pas d'images) |
| **Windows Terminal** | n/a | **fonctionne** : sixel (GitPoulpi, 1.25) | n/a |
| **conhost** | n/a | texte : **partiel** (graphe en caractères, diff d'image en demi-blocs) | n/a |
| **mintty** | n/a | **ne fonctionne pas** (terminal derrière ConPTY : aucune image n'est dessinée, pas même une simple `OSC 1337` placée avec le curseur affiché ; la colonne du graphe reste vide ; le texte et la palette vont bien) | n/a |

### Forme du pointeur de la souris (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **fonctionne** | n/a | non mesuré (demande un GPU) |
| **Ghostty** | **fonctionne** | n/a | non mesuré (demande un GPU) |
| **WezTerm** | **ne fonctionne pas** (terminal : non implémenté dans la version testée) | officiel : **ne fonctionne pas** (terminal). Version avec la PR amont #8092 de WezTerm, qui ajoute les formes de pointeur : **fonctionne** (main, I-beam, flèches de redimensionnement) | non mesuré (demande un GPU) |
| **foot** | **fonctionne** | n/a | n/a |
| **Rio** | **ne fonctionne pas** (terminal : analysée puis jetée) | **ne fonctionne pas** (terminal, même raison ; pas ConPTY) | **ne fonctionne pas** (terminal, même code ; testé seulement sous Linux et Windows) |
| **Alacritty** | **ne fonctionne pas** (terminal) | **ne fonctionne pas** (terminal) | **ne fonctionne pas** (terminal) |
| **Tabby** | **ne fonctionne pas** (terminal : xterm.js) | **ne fonctionne pas** (terminal) | **ne fonctionne pas** (terminal) |
| **Warp** | non mesuré | **ne fonctionne pas** (terminal) | non mesuré |
| **xterm** | **ne fonctionne pas** (terminal : noms CSS inconnus) | n/a | non mesuré (le pointeur était hors de la fenêtre) |
| **iTerm2** | n/a | n/a | **ne fonctionne pas** (terminal) |
| **Terminal.app** | n/a | n/a | **ne fonctionne pas** (terminal) |
| **Windows Terminal** | n/a | **ne fonctionne pas** (terminal) | n/a |
| **conhost** | n/a | **ne fonctionne pas** (terminal) | n/a |
| **mintty** | n/a | **fonctionne** avec les noms propres à mintty ; GitPoulpi les envoie | n/a |

Là où la cellule dit « ne fonctionne pas », le séparateur ou le bouton sous la souris s'allume quand même.

### Protocole clavier

GitPoulpi demande le protocole clavier de kitty sous Linux et macOS. Sous Windows, il ne
le demande pas : la console rapporte elle-même les modificateurs, et les enchaînements
ci-dessous sont limités par ce que le terminal garde pour lui.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **fonctionne** | n/a | non mesuré (demande un GPU) |
| **Ghostty** | **fonctionne** | n/a | non mesuré (demande un GPU) |
| **WezTerm** | **partiel** : pas de réponse tant que `enable_kitty_keyboard = true` n'est pas dans `wezterm.lua` (défaut `false`) | **partiel** : WezTerm garde `Ctrl+Shift+N` (il ouvre sa fenêtre) ; `Ctrl+,` n'est pas transmis (version officielle). La version PR #8092 ne garde aucun des deux, mais `Ctrl+,` n'ouvre toujours pas les Paramètres (pas de réponse du clavier kitty à travers ConPTY) | non mesuré (demande un GPU) |
| **foot** | **fonctionne** | n/a | n/a |
| **Rio** | **fonctionne** (activé par défaut) | **partiel** : Rio garde `Ctrl+Shift+N` (il ouvre sa fenêtre) | non mesuré (demande un GPU) |
| **Alacritty** | **fonctionne** (0.13 et suivantes) | **partiel** : `Ctrl+Shift+N` arrive comme `Ctrl+N`, `Ctrl+,` n'est pas transmis | **fonctionne** |
| **Tabby** | **ne fonctionne pas** (terminal : pas de réponse ; il garde `Ctrl+Shift+N` et `Ctrl+,` pour lui) | **partiel** : Tabby garde `Ctrl+Shift+N` et `Ctrl+,` | **ne fonctionne pas** (terminal : pas de réponse) ; certains enchaînements ne sont pas arrivés |
| **Warp** | non mesuré | non mesuré | non mesuré (demande un GPU) |
| **xterm** | **ne fonctionne pas** (terminal) | n/a | **ne fonctionne pas** (terminal) |
| **iTerm2** | n/a | n/a | **fonctionne** (3.7.7 ; vérifiez *Profiles > Keys* sinon) |
| **Terminal.app** | n/a | n/a | **ne fonctionne pas** (terminal : pas de réponse ; touches classiques) |
| **Windows Terminal** | n/a | **partiel** : WT garde `Ctrl+Shift+N` (nouvelle fenêtre) et `Ctrl+,` (ses paramètres) | n/a |
| **conhost** | n/a | non mesuré | n/a |
| **mintty** | n/a | **partiel** : aucun des deux enchaînements n'a eu d'effet visible | n/a |

Là où un enchaînement n'arrive pas, utilisez les variantes `Alt`
([Raccourcis clavier](keybindings.md#touches-que-le-terminal-ne-peut-pas-envoyer-et-le-protocole-clavier-kitty)),
ou réassignez l'enchaînement dans *Paramètres > Clavier et souris*.

### Presse-papiers (actions de copie)

Une copie utilise toujours le presse-papiers du système quand il y a un affichage local ;
ces cellules décrivent donc la voie `OSC 52` du terminal, qui compte par SSH, dans les
conteneurs et comme repli. Chaque test de copie de GitPoulpi qui utilisait le
presse-papiers du système a fonctionné.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **fonctionne** | n/a | non mesuré (demande un GPU) |
| **Ghostty** | **fonctionne** (vu derrière tmux, zellij, herdr) ; sans multiplexeur : non mesuré (une écriture dans le presse-papiers demande le focus clavier, que le test ne lui a pas donné) | n/a | non mesuré (demande un GPU) |
| **WezTerm** | **fonctionne** | **fonctionne** | non mesuré (demande un GPU) |
| **foot** | non mesuré (une écriture demande le focus clavier, que le test ne lui a pas donné) | n/a | n/a |
| **Rio** | non mesuré (même raison) | **fonctionne** | non mesuré (demande un GPU) |
| **Alacritty** | non mesuré (même raison) | **ne fonctionne pas** (presse-papiers inchangé avec la configuration par défaut ; non étudié) | **fonctionne** |
| **Tabby** | **fonctionne** | **fonctionne** | **fonctionne** |
| **Warp** | non mesuré | **ne fonctionne pas** (presse-papiers inchangé) | non mesuré (demande un GPU) |
| **xterm** | non mesuré | n/a | **ne fonctionne pas** (presse-papiers inchangé) |
| **iTerm2** | n/a | n/a | **ne fonctionne pas** tant que *Settings > General > Selection > Applications in terminal may access clipboard* n'est pas activé |
| **Terminal.app** | n/a | n/a | **ne fonctionne pas** (terminal) |
| **Windows Terminal** | n/a | **fonctionne** (avec le focus) | n/a |
| **conhost** | n/a | non mesuré (aucune sonde lancée) | n/a |
| **mintty** | n/a | **ne fonctionne pas** (presse-papiers inchangé avec ses réglages par défaut) | n/a |

### Collage

Sous Linux et macOS, GitPoulpi demande le collage entre crochets, si bien qu'un collage de
plusieurs lignes est un seul collage. Cela n'a pas été exécuté terminal par terminal
(**non mesuré** par terminal). Sous Windows, la console n'a pas de crochets de collage :
un terminal qui ne les demande pas d'abord livre le texte comme des frappes, et chaque
retour à la ligne serait un `Enter`. GitPoulpi traite donc comme un seul collage les
frappes qui se suivent à moins de 30 ms (un `Enter` au milieu, ou huit caractères ou
plus). Les touches maintenues et « abc, `Enter` » tapés à la main ne sont pas des collages.

| Terminal | Windows (collage de deux lignes dans l'invite de nouvelle branche) |
|---|---|
| **Windows Terminal** | **fonctionne** : WT demande d'abord « coller plusieurs lignes ? » (son propre avertissement), puis aucune branche n'est créée |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **fonctionne** (aucune branche créée) |
| **Warp** | non mesuré (GitPoulpi n'a pas pu y être lancé) |
| **conhost** | non mesuré (`Ctrl+V` et `Shift+Insert` envoyés par le test n'ont rien collé dans la console classique) |

### Largeur des emoji et des icônes

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **fonctionne** (cellules larges alignées ; kitty dessine l'engrenage U+2699 en blanc, une question de police connue) | WezTerm, Rio, Tabby, Windows Terminal : **fonctionne**. Alacritty : **partiel** (emoji en contours monochromes) | Alacritty, Tabby : **fonctionne**. Les autres : non mesuré (demande un GPU) |
| **Tabby** | **fonctionne** | **fonctionne** | **fonctionne** |
| **Warp** | non mesuré | **fonctionne** | non mesuré |
| **xterm** | **partiel** (des cases : la police) | n/a | **partiel** (des cases : la police) |
| **iTerm2**, **Terminal.app** | n/a | n/a | **fonctionne** |
| **conhost** | n/a | **partiel** : emoji en couleur dessinés, cellules larges alignées | n/a |
| **mintty** | n/a | **fonctionne** | n/a |

### Versions testées

kitty 0.49.2 (0.50.0 sous macOS ne démarrait pas), Ghostty 1.4.0, WezTerm 20260716 sous
Linux et 20240203 (la dernière version étiquetée) sous Windows, foot 1.28.0, Rio 0.5.28
(Linux, Windows) et 0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04,
xterm 411 (Linux) et 378 (XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8),
Windows Terminal 1.25.2733, mintty 3.8.3. Les terminaux évoluent vite : une version plus
récente peut différer.

## Terminal par terminal

### Windows

- **Windows Terminal.** Le choix par défaut du lanceur et la meilleure prise en charge
  des images sous Windows : graphe et avatars en sixel, OSC 52, de bons emoji, et il
  répond à la requête de taille de cellule. Choisissez une Nerd Font dans *Settings >
  Profiles > Defaults > Appearance > Font face*. Maintenez `Shift` pour sélectionner du
  texte pendant que la souris est capturée. Il garde `Ctrl+Shift+N` et `Ctrl+,` :
  utilisez les variantes `Alt` ou réassignez. La forme du pointeur ne change pas.
- **WezTerm.** Dans `wezterm.lua`, mettez `config.enable_kitty_keyboard = true` (versions
  Linux et macOS) et une police avec repli emoji, par exemple
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Les images utilisent le protocole iTerm2 et rien n'est à régler : la dernière version
  étiquetée, 20240203-110809 (févr. 2024), les ancre sur les bonnes lignes (revérifié en
  novembre 2026 avec le GitPoulpi actuel ; avant le correctif ConPTY décrit plus bas, ce
  n'était pas le cas). La PR amont #8092 de WezTerm ajoute les formes de pointeur.
- **Rio** (MSI, demande l'UAC). Les images (graphe, avatars, diff d'image) utilisent le
  protocole iTerm2 avec transparence et tombent exactement sur leurs cellules : mesuré
  sur la 0.5.28 à sa police par défaut et à 18 et 22,5 px (cellules de 8x16, 11x21 et
  13x26 pixels, toujours des pixels entiers), une image de 20x10 cellules et une de
  4x27 cellules sont sorties exactes à la cellule. Rien n'est à régler. La
  pseudo-console avale la requête de taille de cellule ; GitPoulpi suppose donc que les
  cellules de Rio sont deux fois plus hautes que larges (vrai aux trois tailles) ; avec
  une police d'une autre proportion, réglez `GITPOULPI_TERMINAL_CELL_PX=WxH` (par
  exemple `10x24`) pour avoir des marqueurs de commit ronds. Rio n'a de formes de
  pointeur sur aucun système.
- **Tabby.** Graphe et avatars en sixel, copier et coller fonctionnent. Tabby garde
  `Ctrl+Shift+N` et `Ctrl+,` pour lui.
- **Alacritty.** Demande OpenGL 3.3. Graphe en caractères ; la copie par `OSC 52` n'était
  pas active avec la configuration par défaut.
- **mintty** (Git Bash, MSYS2, Cygwin). Les formes de pointeur fonctionnent (les noms
  propres à mintty). Les images ne s'affichent pas (la colonne du graphe reste vide ;
  mintty 3.8.3 ne dessine aucune image `OSC 1337` derrière ConPTY, même placée par une
  simple sonde) et il ne répond à aucune requête de taille de cellule : réglez
  `ui.graph_renderer = "text"`.
- **Warp.** Non mesuré avec GitPoulpi : impossible de le lancer depuis la zone de saisie
  de Warp. Le terminal n'a ni images sixel, ni iTerm2, ni kitty.
- **conhost, cmd, une fenêtre PowerShell.** Graphe en caractères, diff d'image en
  demi-blocs. Par défaut, GitPoulpi lancé depuis une telle fenêtre se rouvre dans le
  terminal de votre choix ; voir « La règle de relance ».

**La limite de ConPTY.** Windows fait tourner le programme de chaque terminal derrière une
pseudo-console (ConPTY). Les versions de Windows Terminal et de WezTerm qui en embarquent
une récente laissent passer les séquences d'image ; celle qui est dans Windows (utilisée
par Rio et mintty) et celle qu'embarque WezTerm 20240203 ne le font pas. Ce qui est
perdu : les séquences DCS (sixel) et APC (graphique kitty) sont jetées, seules les images
iTerm2 (`OSC 1337`) arrivent donc, ce qui explique que Rio et WezTerm les utilisent ;
`CSI 16 t` (taille de cellule) n'obtient de réponse que dans Windows Terminal et Tabby.
Ce qui a été corrigé : une `OSC 1337` traverse ConPTY aussitôt et tombe là où ConPTY a
peint le curseur en dernier, et ConPTY ne peint le curseur que tant qu'il est visible.
GitPoulpi gardait le curseur masqué, si bien que ses images tombaient sur des lignes
périmées (le graphe par-dessus le pied de page, un écran déchiré après un défilement).
Il affiche maintenant le curseur autour de chaque image qu'il écrit sous Windows, dans la
même écriture, puis le masque de nouveau ; Rio et WezTerm 20240203 ancrent chaque image
sur sa cellule (`pty_conpty_picture_anchor` relit le déplacement du curseur derrière
ConPTY). Rien ne change hors de Windows ni pour le sixel.

**Collage sous Windows.** Voir le tableau « Collage » ci-dessus : GitPoulpi regroupe une
rafale de frappes en un seul collage, si bien qu'un collage de plusieurs lignes ne valide
jamais une invite dans les terminaux testés.

**La règle de relance.** Un double-clic sur l'exe le démarre dans une fenêtre de console
classique (conhost) à lui. Quand `ui.launch_terminal` nomme un autre terminal (par défaut :
Windows Terminal), GitPoulpi se rouvre là et la première fenêtre se ferme
(`ui.relaunch_in_terminal`). Il ne le fait **que** s'il est seul dans une fenêtre de
console classique. Un terminal qui lance l'exe directement (`wt.exe gitpoulpi.exe`,
WezTerm, Rio, Alacritty `-e`, Tabby `run`) est l'endroit où vous voulez être, et rien
n'est relancé ; depuis un shell, il n'est pas seul, donc rien ne se passe non plus.
Désactivez-le avec `ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Tout fonctionne sans réglage. `Shift+clic` est celui de kitty, utilisez donc
  `Alt+clic` pour étendre une sélection ([Souris](mouse-and-gestures.md)).
- **Ghostty.** Tout fonctionne sans réglage. Si les icônes apparaissent comme des trous,
  ajoutez une police emoji ou Nerd Font à `font-family` (répétez la clé pour ajouter des
  replis).
- **WezTerm.** Images par le protocole iTerm2, transparence mélangée. Réglez
  `enable_kitty_keyboard = true` pour les enchaînements complets. Pas de formes de
  pointeur.
- **foot.** Graphe et avatars en sixel, formes de pointeur, clavier complet, rapide.
  Réglez `font=JetBrainsMono Nerd Font:size=11` dans `foot.ini`. La copie par `OSC 52` est
  gouvernée par `security.osc52`.
- **Rio.** Images par le protocole iTerm2 avec transparence, clavier complet. Pas de
  formes de pointeur (Rio jette `OSC 22`).
- **Alacritty.** Graphe en caractères, clavier kitty depuis la 0.13. Pas de prise en
  charge des images.
- **Tabby.** Images sixel ; pas de clavier kitty, pas de formes de pointeur (Electron,
  xterm.js). Il garde `Ctrl+Shift+N` et `Ctrl+,`.
- **xterm.** Lancez-le avec `xterm -u8 -ti vt340` pour le sixel (l'UTF-8 est nécessaire
  pour les caractères de tracé). Des cases à la place des emoji : la police.
- **Warp.** Non mesuré : il demande XWayland et, pendant le test, a mal décodé les frappes
  synthétiques.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Hors de la série de novembre 2026. Ils
  fonctionnent avec des caractères et les touches classiques : utilisez les variantes
  `Alt` et `Ctrl+Y` pour rétablir. VTE ignore `OSC 52`, donc la copie par SSH n'atteint
  pas le presse-papiers. Une méthode de saisie (IBus, Fcitx) peut s'emparer de la
  bascule d'indexation `Ctrl+Space` ; réassignez-la dans *Paramètres > Clavier et
  souris*. Konsole est reconnu pour le sixel d'après sa propre documentation.
- **Wayland, Hyprland, Omarchy.** L'indice de pointeur (`OSC 22`) fonctionne dans kitty,
  Ghostty et foot. L'entrée du lanceur d'Omarchy passe par `xdg-terminal-exec`.

### macOS

- **iTerm2.** La 3.7.7 et les suivantes dessinent les images par le protocole kitty, sans
  question. Réglages utiles :
  - *Settings > General > Selection > Applications in terminal may access clipboard* :
    nécessaire pour la copie par `OSC 52` (sinon iTerm2 affiche « the terminal attempted
    to access the clipboard but it was denied »).
  - *Profiles > Keys > Left Option key: Esc+* : nécessaire pour les enchaînements `Alt`.
  - Si vous réglez `ui.image_protocol = "iterm2"` (ou utilisez une version antérieure à la
    3.7.7), cochez « Remember my choice » dans la boîte « Allow Terminal-Initiated
    Display? ». Les images ont alors un fond opaque (un rectangle visible si votre fond
    diffère de celui du thème ; `ui.graph_renderer = "text"` le supprime).
  - Le protocole clavier de kitty est activé par défaut (3.6.9 et suivantes).
- **Terminal.app.** 256 couleurs seulement : GitPoulpi choisit `ui.color_depth = 256` de
  lui-même et les thèmes paraissent plus ternes. Pas d'images, pas de clavier kitty, pas
  de presse-papiers par `OSC 52`. Cochez *Use Option as Meta key* pour les enchaînements
  `Alt`.
- **Alacritty, Tabby.** Alacritty : graphe en caractères, clavier kitty, copie `OSC 52` ;
  réglez `option_as_alt = "Both"` pour les enchaînements `Alt`. Tabby : graphe et avatars
  en sixel.
- **Option comme Meta.** Option compose des caractères (Option+B tape un signe intégrale)
  tant que le terminal ne la transforme pas en Meta : iTerm2 *Left Option key: Esc+*,
  Terminal.app *Use Option as Meta key*, Alacritty `option_as_alt`. Les enchaînements
  `Alt` de GitPoulpi en ont besoin, et nulle part elle n'est activée par défaut.
- **kitty, Ghostty, WezTerm, Rio, Warp** : non mesurés sous macOS (ils demandent un GPU).
  Attendez-vous au comportement de Linux.
- **xterm** (XQuartz) : `xterm -u8 -ti vt340` ; le sixel demande
  `ui.image_protocol = "sixel"`.

### VS Code et Warp

Le terminal de VS Code (`TERM_PROGRAM=vscode`) n'a pas fait partie de la série. Il est
reconnu d'après ses attributs d'appareil quand `terminal.integrated.enableImages` est
activé, sinon GitPoulpi dessine des caractères ; `ui.image_protocol = "sixel"` force les
images. Warp : voir les notes par système ci-dessus.

## Multiplexeurs

tmux, zellij, screen, herdr, psmux et les panneaux propres aux terminaux s'intercalent
entre GitPoulpi et le terminal et changent ce qui passe. En bref : tmux laisse passer le
sixel et, avec `allow-passthrough on`, les formes de pointeur et les images kitty ;
zellij et screen ne laissent passer ni images ni formes de pointeur ; herdr laisse passer
les images kitty. Tous les détails et la configuration à ajouter sont dans
[Multiplexeurs](multiplexers.md).

## Ce qui est dégradé, et comment

| Ce qui manque | Ce que vous obtenez à la place |
|---|---|
| Protocole d'images | Graphe en caractères. L'avatar devient le badge d'initiales en couleur (une lettre) quel que soit `ui.avatar_fallback` : une identicon écrasée dans des cellules de texte est méconnaissable. Le diff d'image et la vue Fichier utilisent des demi-blocs colorés, ou une fiche d'information. |
| Vraie couleur | Les couleurs sont ramenées à la palette de 256 ou 16 couleurs (`ui.color_depth`). |
| Protocole clavier kitty | Touches classiques ; certains enchaînements arrivent comme une autre touche. Les Paramètres avertissent, et vous réassignez ou utilisez les variantes `Alt`. |
| OSC 52 | En local, rien ne change (le presse-papiers du système est utilisé). Par SSH ou sans affichage, les actions de copie ne mettent rien dans le presse-papiers : sélectionnez à la souris (maintenez `Shift`) et utilisez la copie du terminal. |
| OSC 22 | Le pointeur ne change pas ; le séparateur ou le bouton sous la souris s'allume quand même. |
| Police emoji | Des cases ou des trous à la place des icônes ; le graphe et le texte fonctionnent quand même. Installez une Nerd Font ou une police emoji et listez-la en repli. |

## Paramètres à connaître

| Paramètre | Usage |
|---|---|
| `ui.image_protocol` | `auto` (défaut), `kitty`, `iterm2`, `sixel`, `none`. Mettez `none` si vous voyez du bruit là où les images devraient être. Redémarrage. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` est toujours correct (utilisez-le là où la limite de ConPTY s'applique). Redémarrage. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` ou `off`, si un terminal ou un multiplexeur se comporte mal. Redémarrage. |
| `ui.text_cursor` | `terminal` (défaut) ou `drawn`. |
| `ui.mouse_enabled` | Désactivé, la souris reste au terminal. Redémarrage. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | Le terminal qu'ouvrent les raccourcis du système, et si un double-clic y rouvre GitPoulpi ([Paramètres](settings.md)). |

Voir aussi [Multiplexeurs](multiplexers.md), [Images](image-view.md),
[Raccourcis clavier](keybindings.md) et [Dépannage](troubleshooting.md).

## Sources

Documentation vérifiée en octobre 2026, comportement testé en novembre 2026. Sources primaires sauf mention contraire.

- kitty : [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty : [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm : [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot : [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty : [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole : [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE : [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2 : [downloads](https://iterm2.com/downloads.html) ; clavier kitty d'après la [matrice terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secondaire)
- Windows Terminal : [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secondaire), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio : [escape sequence support](https://rioterm.com/docs/escape-sequence-support) (pas de ligne `OSC 22`), [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp : [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps) ; OSC 52 d'après un [article sur l'avis de sécurité OSC 52 de Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secondaire)
- VS Code : [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty : [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (pas de vraie couleur) : [discussion Apple](https://discussions.apple.com/thread/255315820) (secondaire)
- tmux : [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES) ; zellij : [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Sites de comparaison (secondaires, utilisés seulement pour recouper) : [Terminal Trove](https://terminaltrove.com/compare/terminals/)
