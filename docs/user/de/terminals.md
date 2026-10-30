[English](../terminals.md) · [Čeština](../cs/terminals.md) · **Deutsch** · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Ein Terminal wählen

GitPoulpi läuft in jedem Terminal mit Mausunterstützung, sieht aber in einem am
besten aus und fühlt sich dort am besten an, das Bilder zeichnet, True Color
darstellt und die volle Tastatur sendet. Diese Seite reiht die Terminals je
Betriebssystem, sagt, was jedes davon bietet, und nennt die Einstellung, die Sie
ändern müssen. Die Angaben wurden im Oktober 2026 an der Dokumentation, dem
Changelog oder dem Issue-Tracker des jeweiligen Projekts geprüft (siehe
[Quellen](#quellen)); nichts davon wurde in jedem Terminal getestet.
Alles, was mit **(ungeprüft)** markiert ist, ließ sich nicht aus einer Primärquelle
bestätigen.

## Die besten Wahlen

| System | Beste Wahl, in dieser Reihenfolge | Warum |
|---|---|---|
| **Windows** | **WezTerm**, dann **Windows Terminal** | WezTerm zeichnet den Graphen und die Avatare als Bilder, mit Transparenz. Windows Terminal hat die beste Integration ins System (es ist der Standard des Starters), zeichnet aber die Avatare als Sixel-Bilder und den Graphen als Zeichen. |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, dann **foot** | kitty und Ghostty bieten Bilder und die volle Tastatur. WezTerm bietet ebenfalls Bilder. foot zeichnet die Avatare als Sixel-Bilder, ist schnell und hat die volle Tastatur. Unter Omarchy nehmen Sie für Bilder Ghostty oder kitty. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, dann **iTerm2** | Dieselbe Begründung. Terminal.app funktioniert, hat aber weder Bilder noch True Color. |

Auf jedem System verbergen **tmux, zellij und screen** das Bildprotokoll des
Terminals vor GitPoulpi, sodass es Zeichen zeichnet (siehe
[Multiplexer](#multiplexer)).

## Was GitPoulpi nutzt

- **Bilder** (Graph-Spalte, Autoren-Avatare, Bild-Diff und Dateiansicht): das
  **kitty-Grafikprotokoll** mit Unicode-Platzhaltern, **iTerm2-Inline-Bilder**
  (`OSC 1337`) oder **Sixel**, in dieser Rangfolge. Die Erkennung ist
  standardmäßig ablehnend und stützt sich auf `TERM`, `TERM_PROGRAM`,
  `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`,
  `KONSOLE_VERSION` und die Variablen der Multiplexer; die Tastatur liest sie nie
  (sie fragt das Terminal also auch nicht nach seinen Geräteattributen). kitty und
  Ghostty nutzen das kitty-Protokoll. WezTerm nutzt absichtlich das iTerm2-Protokoll
  und mischt die Transparenz des PNG ein. iTerm2, Rio und mintty nutzen das
  iTerm2-Protokoll mit undurchsichtigem Hintergrund. Windows Terminal, foot, Konsole
  und mlterm nutzen **Sixel**. Ohne eines davon sind die Avatare das Initialen-Abzeichen.
- **Sixel** wird für die **Graph-Spalte** gezeichnet (Spuren, Knoten und mit `ui.avatars = "graph"` die Avatare in den Commit-Kreisen) und sonst für die **Avatare** (Log, Commit-Details, Dateiverlauf, Blame, Bildansicht). Ein Sixel hat keinen Alphakanal, keine IDs und keine Skalierung; das Bild ist daher genau `Spalten x Zellbreite` mal `Zeilen x Zellhöhe` Pixel groß und wird nur dort gezeichnet, wo etwas ist, über leeren Zellen mit erhaltenem Hintergrund. Das kodierte Bild wird zwischengespeichert, nur bei geänderten Spuren, Scrollposition oder Zoom neu kodiert und nur erneut gesendet, wenn eine Zelle darunter neu gezeichnet wurde. Ein Sixel lässt sich nicht löschen: GitPoulpi schreibt Text über seine Zellen, um es zu entfernen. Sixel braucht die echte Zellgröße in Pixeln: Windows Terminal wird einmal beim Start und nach einem Zoom danach gefragt (`CSI 16 t`, Antwort in etwa 10 ms, blockiert nie, wenn es schweigt); anderswo kommt sie aus der Fenstergröße des Terminals oder aus `GITPOULPI_TERMINAL_CELL_PX=10x20`. Ein Terminal, das Sixel zeichnet, aber nicht erkannt wird (ein VS-Code-Terminal mit `terminal.integrated.enableImages`, ein mit `-ti vt340` gestartetes xterm), braucht `ui.image_protocol = "sixel"`. Windows Terminal vor 1.22 zeichnet kein Sixel; setzen Sie dort `ui.image_protocol = "none"` (der Graph bleibt dann Zeichen).
- **True Color**: Die Designs sind RGB. `ui.color_depth = "auto"` fällt bei einem
  `*-256color`-`TERM` ohne `COLORTERM=truecolor` (und in Terminal.app) auf 256 Farben
  zurück und auf der Linux-Konsole auf 16 Farben. kitty, WezTerm und Windows Terminal
  gelten als True-Color-fähig.
- **Maus**: SGR-Mausberichte mit Tasten, Ziehen und Bewegung (Hover-Rückmeldung,
  Drag & Drop, Rechtsklick, Doppelklick).
- **Tastatur**: Das **kitty-Tastaturprotokoll** (nur das Flag zum Unterscheiden)
  wird beim Start mit einem Timeout von 100 ms angefordert. Damit kommen
  `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` und `Ctrl+Shift+Z` an. Ohne es bleiben die
  klassischen Tasten (`Ctrl+Y` wiederholt, `Alt`-Varianten ersetzen einige
  Tastenfolgen). Siehe
  [Tastenkürzel](keybindings.md#tasten-die-das-terminal-nicht-senden-kann-und-das-kitty-tastaturprotokoll).
- **Zwischenablage**: Kopieraktionen schreiben `OSC 52`.
- **Form des Mauszeigers** (Größenänderungszeiger über Trennern, Hand über
  anklickbaren Elementen): `OSC 22`, ein Extra, wo es umgesetzt ist.
- **Fokusereignisse** sind eingeschaltet (Tooltips schließen sich, wenn das Fenster
  den Fokus verliert). GitPoulpi fordert weder Bracketed Paste noch
  OSC-8-Hyperlinks noch Undercurl an. Nur auf dem Bildpfad wird jedes Bild in eine
  synchronisierte Aktualisierung gehüllt (Modus 2026, von Terminals ignoriert, die
  ihn nicht kennen), damit ein Bild nie über einem Popup aufblitzt.
- **Emoji und Symbole**: GitPoulpi verwendet nur Glyphen mit Emoji-Darstellung als
  Standard oder einbreite BMP-Formen, weil Terminals und Schriften bei den
  Variantenselektoren uneins sind.

## Fähigkeitstabelle

„Bilder“ ist das, was GitPoulpi heute in diesem Terminal zeichnet (Protokoll
**kitty**, **iTerm2** oder **Sixel**, oder **Text**), nicht das, was das Terminal könnte.

| Terminal | System | Bilder (Graph, Avatare, Bild-Diff) | True Color | kitty-Tastatur (volle Tastenfolgen) | Kopieren per OSC 52 | Zeiger per OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | ja | ja | ja | ja |
| **Ghostty** (1.3) | Linux, macOS (kein Windows-Build) | kitty | ja | ja | ja | ja, seit 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, mit Transparenz | ja | nur mit `enable_kitty_keyboard = true` (Standard `false`) | ja (nur Schreiben) | nein (der Pull Request ist noch offen) |
| **foot** (1.28) | Linux (Wayland) | Sixel (Graph und Avatare; kein kitty-Grafikprotokoll) | ja | ja, seit 1.10.3 | ja (`security.osc52`) | ja, seit 1.12 |
| **Alacritty** (0.17) | alle | Text (keine Bildunterstützung im Projekt) | ja | ja, seit 0.13 | ja (Schreiben; Einfügen standardmäßig aus) | nein (nicht umgesetzt) |
| **Konsole** | Linux | Sixel (Graph und Avatare) | ja | wird 2026 ergänzt (Merge Request !1188); Release (ungeprüft) | nur Schreiben, Mitte 2024 gemergt | (ungeprüft) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | Text | ja | nein (nicht in einem veröffentlichten VTE) | nein (VTE ignoriert es) | nein |
| **iTerm2** (3.7) | macOS | iTerm2, undurchsichtiger Hintergrund | ja | ja, seit 3.6.9 (Profiles > Keys prüfen) | standardmäßig aus: „Applications in terminal may access clipboard“ einschalten | (ungeprüft) |
| **Terminal.app** | macOS | Text | nein (256 Farben) | nein | (ungeprüft) | (ungeprüft) |
| **Windows Terminal** (1.25) | Windows | Sixel (Graph und Avatare) | ja | ja, stabil seit 1.25 (Okt. 2026); Preview ab 1.25 (März 2026) | ja (Schreiben; nur solange fokussiert) | nein (nicht umgesetzt) |
| **Rio** | Windows, Linux, macOS | iTerm2, undurchsichtiger Hintergrund (spricht auch kitty-Grafik) | ja | ja (standardmäßig an) | ja | nein (nicht umgesetzt) |
| **Warp** | macOS, Linux, Windows | Text (nicht erkannt) | ja | ja | ja (es beachtet die Sequenz) | (ungeprüft) |
| **VS-Code-Terminal** (xterm.js) | alle | Text (Sixel und iTerm2-Bilder gibt es hinter `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` nutzt das Sixel) | ja | (ungeprüft) | (ungeprüft) | nein |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, undurchsichtiger Hintergrund | ja | (ungeprüft; nicht dokumentiert) | (ungeprüft; nicht dokumentiert) | ja |
| **Windows-Konsole** (conhost, cmd, PowerShell-Fenster) | Windows | Text | nur neuere Windows 10/11 | nicht nötig: Sie meldet die Modifikatoren selbst | nein | nein |
| **ConEmu** | Windows | Text | (ungeprüft) | (ungeprüft) | (ungeprüft) | (ungeprüft) |
| **tmux, zellij, screen** | alle | Text, immer | braucht Einstellungen | braucht Einstellungen | braucht Einstellungen | nein |

Die Zellen „nein (nicht umgesetzt)“ bei OSC 22 stammen aus dem Fehlen in Changelog
und Dokumentation eines Projekts sowie aus Berichten von Nutzern, nicht aus einem
Test; lesen Sie sie als „nicht dokumentiert“. Die Seite tabelliert nicht die Breite
von Emoji und Variantenselektoren: Sie unterscheidet sich je nach Terminal und
Schrift, weshalb GitPoulpi VS16-Glyphen meidet (siehe
[Was GitPoulpi nutzt](#was-gitpoulpi-nutzt)).

## Die einzelnen Terminals

### Windows

- **WezTerm.** Die beste Bildunterstützung unter Windows. Lassen Sie
  `ui.image_protocol` auf `auto`. Setzen Sie in `wezterm.lua`
  `config.enable_kitty_keyboard = true` für die vollen Tastenfolgen und eine
  Schrift mit Emoji-Ersatz, zum Beispiel
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Das letzte getaggte stabile Release von WezTerm ist 20240203-110809-5046fc22
  (Feb. 2024); neuere Korrekturen erscheinen in Nightlies. WezTerm hat noch keine
  Zeigerform per OSC 22.
- **Windows Terminal.** True Color, Maus, OSC 52 und gute Emoji. GitPoulpi zeichnet
  die Avatare als Sixel-Bilder (Windows Terminal 1.22 oder neuer); der Graph besteht
  aus Zeichen. Wählen Sie eine Nerd Font unter *Settings > Profiles
  > Defaults > Appearance > Font face*. Halten Sie `Shift` gedrückt, um Text zu
  markieren, solange die Maus erfasst ist. Die Zeigerform ändert sich nicht. Bei
  einem Build vor 1.25 (ohne kitty-Tastaturprotokoll) nutzen Sie die
  `Alt`-Varianten.
- **Rio** funktioniert mit undurchsichtigem Bildhintergrund; weniger getestet.
- **mintty** (Git Bash, MSYS2, Cygwin) zeichnet Bilder über das iTerm2-Protokoll
  mit undurchsichtigem Hintergrund; weniger getestet.
- **Windows-Konsole, ConEmu, cmd**: besser meiden. Bilder werden schlecht oder gar
  nicht gezeichnet. Ein Doppelklick auf die exe öffnet GitPoulpi erneut in dem
  Terminal Ihrer Wahl (*Einstellungen > Allgemein > Starter*).

### Linux

- **kitty.** Alles funktioniert ohne Einstellung. `Shift+Klick` gehört kitty selbst,
  nutzen Sie also `Alt+Klick`, um eine Auswahl zu erweitern
  ([Maus](mouse-and-gestures.md)).
- **Ghostty.** Alles funktioniert ohne Einstellung. Erscheinen Symbole als Löcher,
  fügen Sie `font-family` eine Emoji- oder Nerd Font hinzu (wiederholen Sie den
  Schlüssel, um Ersatzschriften zu ergänzen).
- **WezTerm.** Bilder über das iTerm2-Protokoll, Transparenz eingemischt. Derselbe
  Hinweis zu `enable_kitty_keyboard` wie unter Windows.
- **foot.** Zeichen-Graph, volle Tastatur, schnell. Avatare sind Sixel-Bilder,
  der Graph besteht aus Zeichen. Setzen Sie `font=JetBrainsMono Nerd Font:size=11` in
  `foot.ini`.
- **Alacritty.** Zeichen-Graph, kitty-Tastatur seit 0.13. Alacritty hat keine
  Bildunterstützung.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Arbeiten mit Zeichen und klassischen
  Tasten: Nutzen Sie die `Alt`-Varianten und `Ctrl+Y` zum Wiederholen (eine neuere
  Konsole kann das kitty-Tastaturprotokoll ergänzen). Kopieraktionen von VTE
  (`OSC 52`) legen nichts in die Zwischenablage. Eine Eingabemethode (IBus, Fcitx)
  kann die Umschaltung `Ctrl+Space` zum Stagen abfangen - **(ungeprüft)**; belegen
  Sie sie unter *Einstellungen > Tastatur & Maus* neu.
- **Wayland, Hyprland, Omarchy.** Der Hinweis auf den Mauszeiger (`OSC 22`)
  funktioniert in kitty, Ghostty und foot. Der Starter-Eintrag von Omarchy läuft
  über `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: wie unter Linux.
- **iTerm2.** Bilder mit undurchsichtigem Hintergrund (ein sichtbares Rechteck, wenn
  Ihr Hintergrund vom Design abweicht; `ui.graph_renderer = "text"` beseitigt es).
  Schalten Sie *Settings > General > Selection > Applications in terminal may access
  clipboard* für das Kopieren ein und *Profiles > Keys > Left Option key: Esc+* für
  die `Alt`-Tastenfolgen.
- **Terminal.app.** Nur 256 Farben: GitPoulpi wählt `ui.color_depth = 256` selbst, und
  die Designs wirken flacher. Keine Bilder. Schalten Sie *Use Option as Meta key* für
  `Alt`-Tastenfolgen ein.

### VS Code und Warp

Das VS-Code-Terminal (`TERM_PROGRAM=vscode`) und Warp werden nicht als Terminals mit
Bildunterstützung erkannt, daher zeichnet GitPoulpi Zeichen. Beide sind brauchbar. Im VS-Code-Terminal schalten Sie
`terminal.integrated.enableImages` ein und setzen `ui.image_protocol = "sixel"` für
Avatar-Bilder.
Warp dokumentiert Unterstützung für das kitty-Tastaturprotokoll; beim
VS-Code-Terminal ist sie **(ungeprüft)**.

## Multiplexer

GitPoulpi erkennt tmux, zellij, screen und herdr (`TMUX`, `ZELLIJ`, `STY`,
`TERM=screen*` oder `tmux*`, `HERDR_*`) und zeichnet **Zeichen**, weil ein
Multiplexer Zellen weiterreicht, keine Bildsequenzen. Für die übrigen Funktionen
gehört Folgendes in `~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

Um Bilder durch einen Multiplexer zu erzwingen, der sie weiterreicht, setzen Sie
`ui.image_protocol` auf `kitty` oder `iterm2`.
herdr mit `kitty_graphics = true` in kitty, Ghostty oder WezTerm wird automatisch
erkannt; siehe
[Fehlerbehebung](troubleshooting.md#der-graph-wird-mit-zeichen-gezeichnet-nicht-als-bild-herdr-tmux-andere-terminals).
`GITPOULPI_TERMINAL_IMAGE=off` schaltet alle Bilder ab.

tmux reicht das kitty-Tastaturprotokoll nicht selbst weiter: `extended-keys` lässt
es modifizierte Tasten als `CSI u`- oder `xterm`-Sequenzen senden. zellij ab 0.41
spricht laut Changelog das kitty-Tastaturprotokoll und reicht OSC 52 weiter.

## Was ausfällt, und was dann geschieht

| Fehlt | Was Sie stattdessen bekommen |
|---|---|
| Bildprotokoll | Zeichen-Graph. Der Avatar wird das farbige Initialen-Abzeichen (ein Buchstabe), unabhängig von `ui.avatar_fallback`: ein in Textzellen gequetschtes Identicon ist nicht erkennbar. Bild-Diff und Dateiansicht nutzen farbige Halbblöcke oder eine Infokarte. |
| True Color | Die Farben werden auf die Palette mit 256 oder 16 Farben abgebildet (`ui.color_depth`). |
| kitty-Tastaturprotokoll | Klassische Tasten; manche Tastenfolgen kommen als andere Taste an. Die Einstellungen warnen, und Sie belegen neu oder nutzen die `Alt`-Varianten. |
| OSC 52 | Kopieraktionen legen nichts in die Zwischenablage: Markieren Sie mit der Maus (`Shift` gedrückt halten) und nutzen Sie das eigene Kopieren des Terminals. |
| OSC 22 | Der Zeiger ändert sich nicht; der Trenner oder die Schaltfläche unter der Maus leuchtet trotzdem auf. |
| Emoji-Schrift | Kästchen oder Löcher statt Symbolen; Graph und Text funktionieren weiter. Installieren Sie eine Nerd Font oder eine Emoji-Schrift und tragen Sie sie als Ersatz ein. |

## Einstellungen, die Sie kennen sollten

| Einstellung | Verwendung |
|---|---|
| `ui.image_protocol` | `auto` (Standard), `kitty`, `iterm2`, `sixel`, `none`. Setzen Sie `none`, wenn Sie Müll sehen, wo Bilder sein sollten. Neustart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` ist immer korrekt. Neustart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` oder `off`, falls sich ein Terminal oder Multiplexer schlecht verhält. Neustart. |
| `ui.mouse_enabled` | Aus überlässt die Maus dem Terminal. Neustart. |
| `ui.launch_terminal` | Das Terminal, das die Verknüpfungen des Systems öffnen ([Einstellungen](settings.md)). |

Siehe auch [Bilder](image-view.md), [Tastenkürzel](keybindings.md) und
[Fehlerbehebung](troubleshooting.md).

## Quellen

Geprüft im Oktober 2026. Primärquellen, sofern nicht anders vermerkt.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); Kitty-Tastaturprotokoll aus der [terminfo.dev-Matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) (sekundär)
- Windows Terminal: [1.22 announcement, Sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (sekundär), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 aus einem [Warp-OSC-52-Sicherheitshinweis](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (sekundär)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (no truecolor): [Apple discussion](https://discussions.apple.com/thread/255315820) (sekundär)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Vergleichsseiten (sekundär, nur zum Gegenprüfen genutzt): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Weiterhin ungeprüft: Sixel in Konsole und mlterm (aus den Funktionslisten der Projekte übernommen, nicht gegen ein Release geprüft und in keinem Terminal in einem echten Fenster ausprobiert); OSC 22 in Konsole, iTerm2, Terminal.app und Warp; OSC 52 in
Terminal.app, VS Code und mintty; das kitty-Tastaturprotokoll in VS Code, mintty und
ConEmu; jede ConEmu-Zelle; und die Emoji-Breite je Terminal. In welchem Release
Konsole das kitty-Tastaturprotokoll ausliefert, wurde nicht bestätigt.
