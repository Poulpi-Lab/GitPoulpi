[English](../multiplexers.md) · **Deutsch**

# Multiplexer und Panes

Ein Multiplexer sitzt zwischen GitPoulpi und Ihrem Terminal: Er zeichnet die Zellen
selbst neu und leitet nur die Escape-Sequenzen weiter, die er versteht. Was im Terminal
allein funktioniert ([Ein Terminal wählen](terminals.md)), muss daher in tmux, zellij,
screen oder herdr nicht funktionieren. Diese Seite sagt, was durchkommt und welche
Konfiguration Sie ergänzen. Die Ergebnisse stammen aus demselben Testdurchlauf vom
November 2026 wie die Terminal-Tabellen: tmux 3.8 (MSYS2: 3.7c), zellij 0.45.1 und
0.46.0, GNU screen 5.0 (macOS liefert 4.00.03), herdr 0.9.5, psmux 3.4.0.

GitPoulpi erkennt einen Multiplexer an `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` oder
`tmux*` und `HERDR_*`. In einem solchen sind Bilder standardmäßig Zeichen, außer wo die
Tabelle unten sagt, dass GitPoulpi einen funktionierenden Weg erkennt.
`GITPOULPI_TERMINAL_IMAGE=off` schaltet alle Bilder aus, und `ui.image_protocol`
erzwingt ein Protokoll.

## Auf einen Blick

| | Bilder | Zeigerform (OSC 22) | Zwischenablage (OSC 52) | Tasten |
|---|---|---|---|---|
| **tmux** | Sixel, wenn das Terminal dahinter es kann (erkannt); kitty-Bilder mit `allow-passthrough on` und `ui.image_protocol = "kitty"` | mit `allow-passthrough on` in kitty und foot; nicht in Ghostty | mit `set-clipboard on` | keine kitty-Tastatur; `Ctrl+Enter` und `Ctrl+,` kommen nie an |
| **zellij** | Zeichen | nein | ja | kitty-Tastatur funktioniert |
| **GNU screen** | Zeichen | nein | nein | klassische Tasten |
| **herdr** | kitty-Bilder (erkannt mit `kitty_graphics = true`) | nein | ja | nicht gemessen |
| **psmux** (Windows) | Zeichen (gewollt) | nein (Windows Terminal hat keine) | System-Zwischenablage funktioniert | nicht gemessen |
| **MSYS2 tmux** (Windows) | Zeichen (gewollt) | nein | System-Zwischenablage funktioniert | nicht gemessen; der Bildschirm ist verzerrt |
| **WezTerm-Panes**, **Windows-Terminal-Panes**, **kitty- und Ghostty-Splits**, **iTerm2-Split-Panes** | wie das Terminal selbst | wie das Terminal | wie das Terminal | wie das Terminal |
| **iTerm2 `tmux -CC`** | Zeichen | nein | nein (iTerm2 verweigert es standardmäßig) | nicht gemessen |

## tmux

**Bilder.**

- **Sixel** zeichnet tmux selbst, für einen Client, der es unterstützt, neu kodiert.
  GitPoulpi fragt tmux einmal beim Start (`tmux display-message -p '#{client_termfeatures}'`,
  auf 400 ms begrenzt) und nutzt Sixel, wenn der Client es aufführt. Das ist hinter foot,
  WezTerm, Rio und Tabby der Fall; hinter kitty und Ghostty nicht (tmux gibt dann den Text
  „SIXEL IMAGE“ aus). Braucht ein tmux, das mit Sixel gebaut wurde (das von Arch und von
  Homebrew ist es; prüfen Sie mit `tmux -V` und einem Sixel-Test). Keine Einstellung. Die
  eigene Antwort von tmux auf eine Geräteattribut-Abfrage behauptet Sixel, was auch immer
  dahinter sitzt; deshalb fragt GitPoulpi stattdessen nach den Funktionen des Clients.
- **iTerm2 hinter tmux** meldet Sixel, malt aber ungesetzte Pixel deckend, sodass der
  überwiegend transparente Graph zu einem einfarbigen Rechteck würde. GitPoulpi erkennt
  iTerm2 (`LC_TERMINAL`, `ITERM_SESSION_ID`, den Client-Typ) und zeichnet Zeichen.
- **kitty-Bilder** laufen durch das DCS-Durchreichen von tmux (kitty, Ghostty; im rohen
  Test auch in iTerm2 und Rio gesehen). Sie brauchen `set -g allow-passthrough on` (tmux
  3.3a oder neuer) **und** `ui.image_protocol = "kitty"` in GitPoulpi, das dann die
  Uploads und Löschungen verpackt. Ohne die tmux-Option wird die Verpackung verworfen, und
  der Graph bleibt leer.
- iTerm2-Inline-Bilder laufen durch tmux, aber tmux kann sie nicht positionieren:
  GitPoulpi nutzt sie dort nie.

**Zeigerform.** GitPoulpi verpackt `OSC 22` für das Durchreichen. Es erreicht das Fenster
in kitty und foot mit `allow-passthrough on`. Ghostty zeigt es verpackt nicht an (der
Pfeil bleibt). WezTerm, Rio, Alacritty, Tabby und iTerm2 haben kein `OSC 22` zum Anzeigen.

**Zwischenablage.** `set -g set-clipboard on`. GitPoulpi sendet zusätzlich die verpackte
Form fürs Durchreichen (sie funktioniert mit `allow-passthrough on`). Hinter kitty,
Ghostty, Alacritty und Tabby kommt es an; iTerm2 verweigert es, bis seine
Zwischenablage-Berechtigung an ist.

**Tasten.** tmux beantwortet die Abfrage des kitty-Tastaturprotokolls nicht, und
`extended-keys` bringt `Ctrl+Enter` oder `Ctrl+,` nicht durch: Gemessen mit tmux 3.8 in
kitty kommen beide als einfaches `Enter` und `,` an, mit `extended-keys` aus, an oder
always. Buchstaben-Tastenfolgen wie `Ctrl+Shift+Z` kommen nur an, wenn `extended-keys` an
oder always ist **und** `extended-keys-format csi-u` (tmux 3.5 oder neuer) **und** eine
Anwendung erweiterte Tasten anfordert; GitPoulpi tut das nie, die Einstellung ändert
hier also nichts. Nutzen Sie in tmux die `Alt`-Varianten
([Tastenkürzel](keybindings.md#tasten-die-das-terminal-nicht-senden-kann-und-das-kitty-tastaturprotokoll)).

**Konfiguration** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later: pointer shapes, forced kitty pictures
```

**tmux in WSL.** Nicht gemessen.

## zellij

Bilder: zellij akzeptiert das kitty-Protokoll, aber nicht dessen Unicode-Platzhalter und
transportiert weder Sixel noch iTerm2-Bilder, sodass GitPoulpi Zeichen zeichnet (ein
gewolltes Veto). Zeigerform: verschluckt, daher sendet GitPoulpi sie nicht.
Zwischenablage: `OSC 52` wird weitergeleitet (funktioniert hinter kitty, Ghostty,
Alacritty, Tabby; iTerm2 und Terminal.app verweigern oder ignorieren es). Tasten: zellij
beantwortet die Abfrage des kitty-Tastaturprotokolls, die volle Tastatur funktioniert also.

zellij 0.46 öffnet bei einem frischen Start ein Popup „Welcome to Zellij“: Drücken Sie
`Esc`. Eine beendete Sitzung behält ihren Namen, und der nächste Start desselben Namens
fragt, was zu tun ist. Es ist keine Konfiguration nötig.

## GNU screen

Nichts kommt durch: Bilder und `OSC 22` werden als Zeichen ausgegeben oder verworfen,
`OSC 52` wird ignoriert, und die Tastatur ist die klassische. GitPoulpi zeichnet Zeichen.
screen 4.00.03 (die Version, die macOS liefert) verzerrt in der Oberfläche außerdem
Farben und breite Glyphen; screen 5 wurde nur unter Linux getestet. Nutzen Sie tmux oder
zellij für eine brauchbare Sitzung.

## herdr

herdr bettet ein Terminal ein, das **kitty-Bilder** erneut ausgibt. GitPoulpi nutzt sie,
wenn die `config.toml` von herdr `kitty_graphics = true` sagt (unter `[terminal]`, in
älteren Versionen unter `[experimental]`) und das Terminal um herdr sie zeichnet (kitty,
Ghostty, WezTerm): erkannt, keine Einstellung. Liegt die Konfiguration woanders (zum
Beispiel unter `HERDR_HOME`, das GitPoulpi nicht liest), setzen Sie
`ui.image_protocol = "kitty"`. Über SSH (`herdr --remote`) kann GitPoulpi den lokalen
Client nicht sehen und zeichnet daher Zeichen; siehe
[Fehlerbehebung](troubleshooting.md#der-graph-wird-mit-zeichen-gezeichnet-nicht-als-bild-herdr-tmux-andere-terminals).
`OSC 22` leitet herdr 0.9.5 nicht weiter, daher sendet GitPoulpi es nicht. `OSC 52`
kommt hinter kitty, Ghostty, Alacritty und Tabby an. Im rohen Test zeichnete herdr
kitty-Bilder in kitty, Ghostty und iTerm2, auch mit der Standardkonfiguration. Tasten:
nicht gemessen.

## psmux und MSYS2-tmux (Windows)

- **psmux** (Rust, ConPTY, tmux-Befehlssprache; `winget install psmux`) setzt `TMUX`,
  daher zeichnet GitPoulpi dort gewollt Zeichen. In Windows Terminal läuft es normal, und
  Kopieren funktioniert über die System-Zwischenablage; der Zeiger ändert sich nicht
  (Windows Terminal hat kein `OSC 22`). Andere Host-Terminals wurden nicht gemessen.
- **tmux aus MSYS2 oder Cygwin** setzt ebenfalls `TMUX` (gewollt Zeichen), und im Test
  war der Bildschirm verzerrt: veraltete Bilder übereinander gezeichnet in mintty und in
  Windows Terminal (mit und ohne `-u`; ohne es wurden Rahmenzeichen als `qqqq`
  ausgegeben). Ein natives Windows-Programm in einem MSYS2-Pseudoterminal wird von tmux
  nicht korrekt neu gezeichnet. Das lässt sich in GitPoulpi nicht beheben: Nutzen Sie
  psmux, Windows-Terminal-Panes oder WezTerm-Panes.
- zellij und screen haben kein Windows-Release. herdr 0.9.5 wurde installiert, aber nicht
  ausgeführt.

## Terminal-Panes

Panes, die zum Terminal selbst gehören, sind für GitPoulpi keine Multiplexer:

- **WezTerm-Panes** (`wezterm cli split-pane`, `wezterm connect`, SSH-Domains): WezTerm
  zeichnet alles, daher funktionieren Bilder (Graph und Avatare im Pane), und der Zeiger
  verhält sich wie in WezTerm selbst (Formen funktionieren in einem Build mit dem
  Upstream-WezTerm-PR #8092; der Pane-Teiler zeigt den Größenänderungszeiger). Unter Linux
  und macOS nicht gemessen.
- **Windows-Terminal-Panes** (`wt ; split-pane`): Sixel-Graph und -Avatare funktionieren im
  Pane, Kopieren funktioniert, keine Zeigerformen.
- **kitty-, Ghostty- und iTerm2-Splits**: nativ, wie das Terminal. Unter macOS für kitty
  und Ghostty nicht gemessen (sie brauchen eine GPU).

## iTerm2 `tmux -CC` (Steuermodus)

iTerm2 öffnet für jedes tmux-Fenster ein natives Fenster und behält ein Gateway-Fenster
„[tmux bash]“. `TMUX` ist gesetzt, daher zeichnet GitPoulpi im nativen Fenster Zeichen.
Bilder erreichen iTerm2 über die Steuerverbindung durchaus (der Dialog „Allow
Terminal-Initiated Display?“ erschien bei einem `OSC 1337`-Test), aber GitPoulpi nutzt sie
dort nicht. Die kitty-Nutzlast eines erzwungenen `ui.image_protocol = "kitty"` wird im
Gateway-Fenster ausgegeben; lassen Sie es also auf `auto`. `OSC 22` und `OSC 52` kommen
nicht an.

## Einstellungen, die man kennen sollte

| Einstellung | Verwendung |
|---|---|
| `ui.image_protocol` | `kitty` erzwingt kitty-Bilder durch einen Multiplexer, der sie weiterleitet (tmux mit `allow-passthrough on`, herdr mit einer Konfiguration, die GitPoulpi nicht liest); `none` für Zeichen. |
| `ui.graph_renderer` | `text` ist immer korrekt. |
| `ui.kitty_keyboard` | `off`, wenn ein Multiplexer sich bei der Tastaturabfrage schlecht verhält. |

Siehe [Ein Terminal wählen](terminals.md) für das Terminal hinter dem Multiplexer und
[Fehlerbehebung](troubleshooting.md), wenn der Graph mit Zeichen gezeichnet wird.
