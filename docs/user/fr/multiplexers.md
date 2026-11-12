[English](../multiplexers.md) · [Čeština](../cs/multiplexers.md) · [Deutsch](../de/multiplexers.md) · [Español](../es/multiplexers.md) · **Français** · [Italiano](../it/multiplexers.md) · [日本語](../ja/multiplexers.md) · [한국어](../ko/multiplexers.md) · [Nederlands](../nl/multiplexers.md) · [Polski](../pl/multiplexers.md) · [Português (Brasil)](../pt-BR/multiplexers.md) · [Português (Portugal)](../pt-PT/multiplexers.md) · [Русский](../ru/multiplexers.md) · [Svenska](../sv/multiplexers.md) · [Türkçe](../tr/multiplexers.md) · [Українська](../uk/multiplexers.md) · [简体中文](../zh-CN/multiplexers.md)

# Multiplexeurs et panneaux

Un multiplexeur s'intercale entre GitPoulpi et votre terminal : il redessine lui-même les
cellules et ne transmet que les séquences d'échappement qu'il comprend. Ce qui marche dans
le terminal seul ([Choisir un terminal](terminals.md)) peut donc ne pas marcher dans tmux,
zellij, screen ou herdr. Cette page dit ce qui passe, et la configuration à ajouter. Les
résultats viennent de la même série de tests de novembre 2026 que les tableaux des
terminaux : tmux 3.8 (MSYS2 : 3.7c), zellij 0.45.1 et 0.46.0, GNU screen 5.0 (macOS livre
la 4.00.03), herdr 0.9.5, psmux 3.4.0.

GitPoulpi détecte un multiplexeur d'après `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` ou
`tmux*` et `HERDR_*`. Dans l'un d'eux, les images deviennent des caractères par défaut,
sauf là où le tableau ci-dessous dit que GitPoulpi détecte une voie qui marche.
`GITPOULPI_TERMINAL_IMAGE=off` désactive toutes les images, et `ui.image_protocol` force un
protocole.

## En un coup d'œil

| | Images | Forme du pointeur (OSC 22) | Presse-papiers (OSC 52) | Touches |
|---|---|---|---|---|
| **tmux** | sixel quand le terminal derrière l'accepte (détecté) ; images kitty avec `allow-passthrough on` et `ui.image_protocol = "kitty"` | avec `allow-passthrough on`, dans kitty et foot ; pas dans Ghostty | avec `set-clipboard on` | pas de clavier kitty ; `Ctrl+Enter` et `Ctrl+,` n'arrivent jamais |
| **zellij** | caractères | non | oui | le clavier kitty fonctionne |
| **GNU screen** | caractères | non | non | touches classiques |
| **herdr** | images kitty (détectées avec `kitty_graphics = true`) | non | oui | non mesuré |
| **psmux** (Windows) | caractères (voulu) | non (Windows Terminal n'en a pas) | le presse-papiers du système fonctionne | non mesuré |
| **tmux de MSYS2** (Windows) | caractères (voulu) | non | le presse-papiers du système fonctionne | non mesuré ; l'écran est brouillé |
| **Panneaux de WezTerm**, **panneaux de Windows Terminal**, **séparations de kitty et Ghostty**, **panneaux partagés d'iTerm2** | comme le terminal lui-même | comme le terminal | comme le terminal | comme le terminal |
| **`tmux -CC` d'iTerm2** | caractères | non | non (iTerm2 le refuse par défaut) | non mesuré |

## tmux

**Images.**

- Le **sixel** est dessiné par tmux lui-même, réencodé pour un client qui le gère.
  GitPoulpi interroge tmux une fois au démarrage (`tmux display-message -p
  '#{client_termfeatures}'`, borné à 400 ms) et utilise le sixel quand le client le liste.
  C'est le cas derrière foot, WezTerm, Rio et Tabby ; ce n'est pas le cas derrière kitty
  et Ghostty (tmux affiche alors le texte « SIXEL IMAGE »). Il faut un tmux compilé avec
  le sixel (ceux d'Arch et de Homebrew le sont ; vérifiez avec `tmux -V` et un test
  sixel). Aucun réglage. La propre réponse de tmux à une requête d'attributs d'appareil
  se déclare sixel quoi qu'il y ait derrière lui, ce pourquoi GitPoulpi demande plutôt
  les fonctions du client.
- **iTerm2 derrière tmux** annonce le sixel mais peint les pixels non définis en opaque,
  si bien que le graphe, surtout transparent, deviendrait un rectangle plein. GitPoulpi
  reconnaît iTerm2 (`LC_TERMINAL`, `ITERM_SESSION_ID`, le type du client) et dessine des
  caractères.
- Les **images kitty** passent par le passthrough DCS de tmux (kitty, Ghostty ; vues aussi
  dans iTerm2 et Rio avec une sonde brute). Elles demandent `set -g allow-passthrough on`
  (tmux 3.3a ou plus récent) **et** `ui.image_protocol = "kitty"` dans GitPoulpi, qui
  enveloppe alors les envois et les suppressions. Sans l'option de tmux, l'enveloppe est
  jetée et le graphe reste vide.
- Les images en ligne d'iTerm2 passent par tmux mais tmux ne sait pas les positionner :
  GitPoulpi ne les utilise jamais là.

**Forme du pointeur.** GitPoulpi enveloppe `OSC 22` pour le passthrough. Elle atteint la
fenêtre dans kitty et foot avec `allow-passthrough on`. Ghostty ne l'affiche pas quand elle
est enveloppée (la flèche reste). WezTerm, Rio, Alacritty, Tabby et iTerm2 n'ont pas
d'`OSC 22` à afficher.

**Presse-papiers.** `set -g set-clipboard on`. GitPoulpi envoie aussi la forme enveloppée
pour le passthrough (elle marche avec `allow-passthrough on`). Derrière kitty, Ghostty,
Alacritty et Tabby, elle arrive ; iTerm2 la refuse tant que sa permission d'accès au
presse-papiers n'est pas activée.

**Touches.** tmux ne répond pas à la requête du clavier kitty, et `extended-keys` ne fait
pas passer `Ctrl+Enter` ni `Ctrl+,` : mesuré avec tmux 3.8 dans kitty, les deux arrivent
comme un simple `Enter` et `,` avec `extended-keys` sur off, on ou always. Les
enchaînements de lettres comme `Ctrl+Shift+Z` n'arrivent qu'avec `extended-keys` sur on
ou always **et** `extended-keys-format csi-u` (tmux 3.5 ou plus récent) **et** une
application qui demande les touches étendues ; GitPoulpi ne le fait jamais, donc ce
réglage ne change rien ici. Utilisez les variantes `Alt` dans tmux
([Raccourcis clavier](keybindings.md#touches-que-le-terminal-ne-peut-pas-envoyer-et-le-protocole-clavier-kitty)).

**Configuration** (`~/.tmux.conf`) :

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a ou plus récent : formes de pointeur, images kitty forcées
```

**tmux dans WSL.** Non mesuré.

## zellij

Images : zellij accepte le protocole kitty mais pas ses espaces réservés Unicode, et ne
transporte ni sixel ni images iTerm2, si bien que GitPoulpi dessine des caractères (un
veto délibéré). Forme du pointeur : avalée, GitPoulpi ne l'envoie donc pas.
Presse-papiers : `OSC 52` est transmise (marche derrière kitty, Ghostty, Alacritty,
Tabby ; iTerm2 et Terminal.app la refusent ou l'ignorent). Touches : zellij répond à la
requête du clavier kitty, donc le clavier complet fonctionne.

zellij 0.46 ouvre une fenêtre « Welcome to Zellij » à un démarrage neuf : appuyez sur
`Esc`. Une session qui s'est terminée garde son nom et le démarrage suivant du même nom
demande quoi faire. Aucune configuration n'est nécessaire.

## GNU screen

Rien ne passe : les images et `OSC 22` sont imprimées comme des caractères ou jetées,
`OSC 52` est ignorée, et le clavier est le clavier classique. GitPoulpi dessine des
caractères. screen 4.00.03 (la version livrée par macOS) brouille aussi les couleurs et
les glyphes larges de l'interface ; screen 5 n'a été testé que sous Linux. Utilisez tmux
ou zellij pour une session utilisable.

## herdr

herdr embarque un terminal qui réémet les **images kitty**. GitPoulpi les utilise quand le
`config.toml` de herdr dit `kitty_graphics = true` (sous `[terminal]`, ou `[experimental]`
sur les versions plus anciennes) et que le terminal autour de herdr les dessine (kitty,
Ghostty, WezTerm) : détecté, aucun réglage. Si la configuration est ailleurs (par exemple
sous `HERDR_HOME`, que GitPoulpi ne lit pas), réglez `ui.image_protocol = "kitty"`. Par
SSH (`herdr --remote`), GitPoulpi ne voit pas le client local et dessine donc des
caractères ; voir
[Dépannage](troubleshooting.md#le-graphe-est-dessiné-avec-des-caractères-pas-comme-une-image-herdr-tmux-autres-terminaux).
`OSC 22` n'est pas transmise par herdr 0.9.5, GitPoulpi ne l'envoie donc pas. `OSC 52`
arrive derrière kitty, Ghostty, Alacritty et Tabby. Dans la sonde brute, herdr a dessiné
des images kitty dans kitty, Ghostty et iTerm2, avec la configuration par défaut aussi.
Touches : non mesuré.

## psmux et tmux de MSYS2 (Windows)

- **psmux** (Rust, ConPTY, langage de commandes de tmux ; `winget install psmux`) définit
  `TMUX`, si bien que GitPoulpi y dessine des caractères volontairement. Dans Windows
  Terminal, il tourne normalement et la copie passe par le presse-papiers du système ; le
  pointeur ne change pas (Windows Terminal n'a pas d'`OSC 22`). Les autres terminaux
  hôtes n'ont pas été mesurés.
- **tmux de MSYS2 ou de Cygwin** définit aussi `TMUX` (caractères, volontairement), et
  pendant le test l'écran était brouillé : des trames périmées dessinées les unes sur les
  autres dans mintty et dans Windows Terminal (avec et sans `-u` ; sans lui, le tracé de
  cadres s'imprimait en `qqqq`). Un programme Windows natif dans un pseudo-terminal MSYS2
  n'est pas redessiné correctement par tmux. GitPoulpi n'y peut rien : utilisez psmux, les
  panneaux de Windows Terminal ou ceux de WezTerm.
- zellij et screen n'ont pas de version Windows. herdr 0.9.5 a été installé mais pas lancé.

## Panneaux des terminaux

Les panneaux qui appartiennent au terminal lui-même ne sont pas des multiplexeurs pour
GitPoulpi :

- **Panneaux de WezTerm** (`wezterm cli split-pane`, `wezterm connect`, domaines SSH) :
  WezTerm dessine tout, donc les images fonctionnent (graphe et avatars dans le panneau)
  et le pointeur se comporte comme dans WezTerm lui-même (les formes fonctionnent dans
  une version avec la PR amont #8092 de WezTerm ; le séparateur de panneau donne le
  pointeur de redimensionnement). Non mesuré sous Linux ni macOS.
- **Panneaux de Windows Terminal** (`wt ; split-pane`) : le graphe et les avatars en sixel
  fonctionnent dans le panneau, la copie aussi, pas de formes de pointeur.
- **Séparations de kitty, Ghostty et iTerm2** : natives, comme le terminal. Non mesuré sous
  macOS pour kitty et Ghostty (ils demandent un GPU).

## `tmux -CC` d'iTerm2 (mode contrôle)

iTerm2 ouvre une fenêtre native pour chaque fenêtre tmux et garde une fenêtre passerelle
« [tmux bash] ». `TMUX` est défini, GitPoulpi dessine donc des caractères dans la fenêtre
native. Les images atteignent bien iTerm2 par la connexion de contrôle (la boîte « Allow
Terminal-Initiated Display? » est apparue pour un test `OSC 1337`), mais GitPoulpi ne les
y utilise pas. Les données kitty d'un `ui.image_protocol = "kitty"` forcé sont imprimées
dans la fenêtre passerelle : laissez donc `auto`. `OSC 22` et `OSC 52` n'arrivent pas.

## Paramètres à connaître

| Paramètre | Usage |
|---|---|
| `ui.image_protocol` | `kitty` force les images kitty à travers un multiplexeur qui les transmet (tmux avec `allow-passthrough on`, herdr avec une configuration que GitPoulpi ne lit pas) ; `none` pour des caractères. |
| `ui.graph_renderer` | `text` est toujours correct. |
| `ui.kitty_keyboard` | `off` si un multiplexeur se comporte mal avec la requête du clavier. |

Voir [Choisir un terminal](terminals.md) pour le terminal derrière le multiplexeur, et
[Dépannage](troubleshooting.md) quand le graphe est dessiné avec des caractères.
