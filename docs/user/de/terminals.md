[English](../terminals.md) · [Čeština](../cs/terminals.md) · **Deutsch** · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Ein Terminal wählen

GitPoulpi läuft in jedem Terminal mit Mausunterstützung, sieht aber in einem am
besten aus und fühlt sich dort am besten an, das Bilder zeichnet, True Color
darstellt und die volle Tastatur sendet. Diese Seite sagt, was jedes Terminal
auf jedem Betriebssystem bietet und welche Einstellung Sie ändern müssen.

**So lesen Sie die Seite.** Die Tabellen stammen aus einem Testdurchlauf im
**November 2026**: Jedes Terminal wurde mit rohen Escape-Sequenzen angesteuert und,
wo es sich starten ließ, mit GitPoulpi selbst, unter Linux, Windows 11 und macOS 15.
Die Versionen stehen [weiter unten](#getestete-versionen). Jede Zelle ist eines von:

- **funktioniert**;
- **teilweise** (und was fehlt);
- **funktioniert nicht** (und warum: das *Terminal*, die *Betriebssystem-Schicht* oder *GitPoulpi*);
- **entfällt** (das Terminal gibt es auf diesem Betriebssystem nicht);
- **nicht gemessen** (und warum).

Multiplexer (tmux, zellij, screen, herdr, psmux, Panes) haben eine eigene Seite:
[Multiplexer](multiplexers.md).

## Die besten Wahlen

| System | Beste Wahl, in dieser Reihenfolge | Warum |
|---|---|---|
| **Windows** | **Windows Terminal**, dann **WezTerm** und **Rio** | Windows Terminal zeichnet den Graphen und die Avatare als Sixel-Bilder, richtig verankert, und ist der Standard des Starters. WezTerm und Rio zeichnen sie als iTerm2-Bilder, seit der ConPTY-Korrektur vom November 2026 richtig verankert (siehe [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty und Ghostty bieten Bilder, die Zeigerformen und die volle Tastatur. foot zeichnet Sixel-Bilder und hat die volle Tastatur und die Zeigerformen. |
| **macOS** | **iTerm2**, dann **kitty** und **Ghostty** | iTerm2 3.7.7 zeichnet Bilder über das kitty-Protokoll. Terminal.app funktioniert, hat aber weder Bilder noch True Color. kitty und Ghostty verhalten sich wie unter Linux (unter macOS nicht gemessen). |

Auf jedem System verbergen **screen** und **zellij** das Bildprotokoll des
Terminals vor GitPoulpi, sodass es Zeichen zeichnet; tmux reicht Sixel durch. Siehe
[Multiplexer](multiplexers.md).

## Was GitPoulpi nutzt

- **Bilder** (Graph-Spalte, Autoren-Avatare, Bild-Diff und Dateiansicht): das
  **kitty-Grafikprotokoll** mit Unicode-Platzhaltern, **iTerm2-Inline-Bilder**
  (`OSC 1337`) oder **Sixel**. Die Erkennung ist standardmäßig ablehnend und stützt
  sich auf `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`,
  `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` und die Variablen der
  Multiplexer. Nennt die Umgebung niemanden (ein mit `-ti vt340` gestartetes xterm,
  ein VS-Code-Terminal mit eingeschalteten Bildern, jedes Terminal über SSH, wo
  `TERM_PROGRAM` nicht weitergegeben wird), entscheiden die eigenen Antworten des
  Terminals: GitPoulpi fragt einmal beim Start (siehe „Die Start-Abfrage“ unten) und
  nutzt den XTVERSION-Namen des Terminals (kitty, Ghostty, WezTerm, iTerm2, mintty,
  foot) oder Sixel in seinen Geräteattributen. In einem Multiplexer werden die
  Antworten ignoriert, weil tmux Sixel behauptet, was auch immer dahinter sitzt.
- **Welches Protokoll wo.** kitty, Ghostty und iTerm2 ab 3.7.7: das kitty-Protokoll.
  WezTerm und Rio: das iTerm2-Protokoll, das die Transparenz des PNG einmischt.
  mintty und iTerm2 vor 3.7.7: das iTerm2-Protokoll mit undurchsichtigem Hintergrund.
  Windows Terminal, foot, Tabby, Konsole und mlterm: **Sixel**. Alles andere: Zeichen,
  und der Avatar wird zum Initialen-Abzeichen. `ui.image_protocol` erzwingt eines.
- **iTerm2 3.7.7.** Jedes `OSC 1337`-Inline-Bild öffnet einen modalen Dialog „Allow
  Terminal-Initiated Display?“, der das Fenster blockiert, bis Sie „Remember my
  choice“ ankreuzen. Dieselben Bilder über das kitty-Protokoll, Unicode-Platzhalter
  eingeschlossen, werden ohne Rückfrage gezeichnet; deshalb nutzt GitPoulpi dort das
  kitty-Protokoll. `ui.image_protocol = "iterm2"` liefert weiterhin die Inline-Bilder.
- **Sixel** wird für die **Graph-Spalte** gezeichnet (Spuren, Knoten und mit
  `ui.avatars = "graph"` die Avatare in den Commit-Kreisen) und sonst für die
  **Avatare** (Log, Commit-Details, Dateiverlauf, Blame, Bildansicht). Ein Sixel hat
  keinen Alphakanal, keine IDs und keine Skalierung; das Bild ist daher genau
  `Spalten x Zellbreite` mal `Zeilen x Zellhöhe` Pixel groß und wird nur dort
  gezeichnet, wo etwas ist, über leeren Zellen mit erhaltenem Hintergrund; es wird
  nur bei geänderten Spuren, Scrollposition oder Zoom neu kodiert. Ein Sixel lässt
  sich nicht löschen: GitPoulpi schreibt Text über seine Zellen, um es zu entfernen.
  Sixel braucht die echte Zellgröße in Pixeln: Windows Terminal, Tabby und mintty
  beantworten `CSI 16 t` einmal beim Start und nach einem Zoom (und blockieren nie,
  wenn sie schweigen); anderswo kommt sie aus der Fenstergröße des Terminals oder aus
  `GITPOULPI_TERMINAL_CELL_PX=10x20`. Ein Terminal, das Sixel zeichnet, ohne es zu
  sagen, braucht `ui.image_protocol = "sixel"`. Windows Terminal vor 1.22 zeichnet
  kein Sixel; setzen Sie dort `ui.image_protocol = "none"`.
- **True Color**: Die Designs sind RGB. `ui.color_depth = "auto"` fällt bei einem
  `*-256color`-`TERM` ohne `COLORTERM=truecolor` (und in Terminal.app, das kein True
  Color hat) auf 256 Farben zurück und auf der Linux-Konsole auf 16 Farben.
- **Maus**: SGR-Mausberichte mit Tasten, Ziehen und Bewegung (Hover-Rückmeldung,
  Drag & Drop, Rechtsklick, Doppelklick).
- **Die Start-Abfrage.** Vier Fragen gehen in einem Schreibvorgang hinaus: die Flags
  des kitty-Tastaturprotokolls (`CSI ? u`), die Hintergrundfarbe (`OSC 11`, nur für
  `ui.theme = system` genutzt), der Name des Terminals (XTVERSION, `CSI > 0 q`) und
  seine Geräteattribute (`CSI c`, zuletzt: Jedes Terminal beantwortet sie, sie
  beendet also das Lesen). Das Lesen ist eine einzige `poll`-Schleife mit 100 ms
  Frist und endet, bevor der Tastaturleser startet. Ein Terminal, das nichts
  antwortet, kostet einmalig die 100 ms, und alles bleibt, wie die Umgebung es sagt.
  Nur Unix: Unter Windows meldet die Konsole die Tasten selbst, und die Umgebung
  entscheidet. Die Antworten stehen im Umgebungsbericht des Aktivitätsprotokolls
  (Zeile `start-up query:`).
- **Tastatur**: Das **kitty-Tastaturprotokoll** (nur das Flag zum Unterscheiden)
  wird beim Start angefordert. Damit kommen `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N`
  und `Ctrl+Shift+Z` an. Ohne es bleiben die klassischen Tasten (`Ctrl+Y` wiederholt,
  `Alt`-Varianten ersetzen einige Tastenfolgen). Siehe
  [Tastenkürzel](keybindings.md#tasten-die-das-terminal-nicht-senden-kann-und-das-kitty-tastaturprotokoll).
- **Zwischenablage**: Jede Kopieraktion folgt einer Regel. Lokal geht der Text in die
  **System-Zwischenablage** des Rechners, auf dem GitPoulpi läuft, plus per `OSC 52`
  an das Terminal, wenn bekannt ist, dass das Terminal es beachtet (kitty, WezTerm,
  Ghostty, foot, Alacritty, Windows Terminal, Rio, Tabby, iTerm2); `OSC 52` ist auch
  der Ausweg, wenn die System-Zwischenablage versagt. **Über SSH oder ohne Display**
  (eine Linux-Konsole, ein Container) wird nur `OSC 52` gesendet, und in tmux wird es
  für das Durchreichen verpackt ebenso gesendet wie unverpackt. Die Statuszeile sagt
  „kopiert“ nur, wenn die System-Zwischenablage den Text genommen hat; was ans
  Terminal ging, heißt „an die Zwischenablage des Terminals gesendet (OSC 52, nicht
  bestätigt)“, weil nichts bestätigt, dass das Terminal gehorcht hat. Der
  Umgebungsbericht des Aktivitätsprotokolls nennt den Weg in seiner Zeile `clipboard:`.
- **Form des Mauszeigers** (Größenänderungszeiger über Trennern, Hand über
  anklickbaren Elementen, I-Balken über Textfeldern): `OSC 22`, ein Extra, wo das
  Terminal es umsetzt. GitPoulpi sendet es immer (außer auf der Linux-Konsole und
  hinter screen, zellij und herdr, die es verschlucken oder ausgeben; hinter tmux wird
  es für das Durchreichen verpackt). Unter `TERM_PROGRAM=mintty` sendet es die eigenen
  Cursor-Namen von mintty (`hand`, `ibeam`, `sizewe`, ...), die einzigen, die mintty
  liest. **Rio setzt `OSC 22` auf keinem System um**: Sein Quelltext liest die Sequenz
  und verwirft sie, keine Einstellung auf irgendeiner Seite ändert das.
- **Textcursor**: Mit `ui.text_cursor = "terminal"` (Standard) wird der eigene Cursor
  des Terminals an die Einfügemarke des fokussierten Textfelds gesetzt, als blinkender
  Balken (`DECSCUSR`), und sonst verborgen; Ihr eigener Cursorstil kommt beim Beenden
  zurück. `drawn` zeichnet stattdessen die eigene Einfügemarke von GitPoulpi.
- **Fokusereignisse** sind eingeschaltet (Tooltips schließen sich, wenn das Fenster
  den Fokus verliert). **Bracketed Paste** wird beim Start angefordert und bei jedem
  Verlassen freigegeben (Unix), sodass ein mehrzeiliges Einfügen als ein Einfügen
  ankommt: Seine Zeilenumbrüche senden nie ein Feld ab, und seine Buchstaben lösen nie
  Tastenkürzel aus. Unter Windows hat die Konsole keine Einfüge-Klammern; was GitPoulpi
  stattdessen tut, steht unter [Windows](#windows). GitPoulpi fordert weder
  OSC-8-Hyperlinks noch Undercurl an. Nur auf dem Bildpfad wird jedes Bild in eine
  synchronisierte Aktualisierung gehüllt (Modus 2026, von Terminals ignoriert, die
  ihn nicht kennen), damit ein Bild nie über einem Popup aufblitzt.
- **Emoji und Symbole**: GitPoulpi verwendet nur Glyphen mit Emoji-Darstellung als
  Standard oder einbreite BMP-Formen, weil Terminals und Schriften bei den
  Variantenselektoren uneins sind.

## Die Matrix

Die Zeilen sind Terminals, die Spalten Betriebssysteme. „GitPoulpi“ in einer Zelle
heißt, dass GitPoulpi selbst dort lief; „probe“ heißt, dass nur rohe
Escape-Sequenzen liefen. Zellen mit **entfällt**: kitty hat keinen eigenen
Windows-Build (die Releases enthalten Linux und macOS); Ghostty hat keinen
Windows-Build (nur die Bibliothek libghostty ist plattformübergreifend); foot ist ein
Wayland-Terminal (nur Linux); xterm ist ein X11-Client (Linux und über XQuartz
macOS); iTerm2 und Terminal.app gibt es nur unter macOS; Windows Terminal, conhost und
mintty nur unter Windows (mintty auch unter Cygwin und MSYS2).

**Unter macOS nicht gemessen: kitty, Ghostty, WezTerm, Rio und Warp.** Sie brauchen
eine GPU (Metal oder OpenGL 3.3). Ihre Linux-Ergebnisse sind die Referenz dafür, was
dasselbe Programm tut; messen Sie sie auf einem Mac mit GPU nach, bevor Sie sich auf
eine macOS-Zelle verlassen.

### Bilder: Welches Protokoll GitPoulpi wählt und ob es zu sehen ist

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funktioniert**: kitty-Protokoll (GitPoulpi) | entfällt | nicht gemessen (braucht eine GPU) |
| **Ghostty** | **funktioniert**: kitty-Protokoll (GitPoulpi) | entfällt | nicht gemessen (braucht eine GPU) |
| **WezTerm** | **funktioniert**: iTerm2-Protokoll mit Transparenz (GitPoulpi). Platzhalter werden gestaucht gezeichnet, daher wird das kitty-Protokoll nicht genutzt | **funktioniert**: iTerm2-Protokoll mit Transparenz, Graph und Avatare auf ihren Zeilen (offizielles 20240203 und der Build mit dem Upstream-PR #8092). Das offizielle 20240203 setzte die Bilder auf die falschen Zeilen, bevor GitPoulpi den Cursor um sie herum zeigte, siehe „Die ConPTY-Grenze“. | nicht gemessen (braucht eine GPU) |
| **foot** | **funktioniert**: Sixel mit Transparenz (GitPoulpi) | entfällt | entfällt |
| **Rio** | **funktioniert**: iTerm2-Protokoll mit Transparenz (GitPoulpi) | **funktioniert**: iTerm2-Protokoll mit Transparenz, zellgenau bei jeder getesteten Schriftgröße (0.5.28; GitPoulpi). Sixel und kitty kommen nie an (ConPTY verwirft sie), es gibt also nichts zu wählen | nicht gemessen (braucht eine GPU) |
| **Alacritty** | Text: **funktioniert** (keine Bildunterstützung im Terminal) | Text: **teilweise** (keine Bilder; Bild-Diff als Blöcke) | Text: **funktioniert** |
| **Tabby** | **funktioniert**: Sixel mit Transparenz (probe; GitPoulpi unter Linux nicht in Tabby gestartet) | **funktioniert**: Sixel, ausgerichtet (GitPoulpi) | **funktioniert**: Sixel, Transparenz erhalten (GitPoulpi) |
| **Warp** | nicht gemessen (es braucht XWayland und dekodiert synthetische Tastenanschläge falsch, sodass nichts getippt werden konnte) | **funktioniert nicht** (Terminal: weder Sixel noch iTerm2- oder kitty-Bilder); GitPoulpi nicht gemessen (ließ sich aus Warps Eingabefeld nicht starten) | nicht gemessen (braucht eine GPU) |
| **xterm** | probe: Sixel **funktioniert** mit `-ti vt340`; GitPoulpi wählt es anhand der Geräteattribute (die Start-Abfrage kam nach dem Lauf), oder setzen Sie `ui.image_protocol = "sixel"` | entfällt | Text: **funktioniert**; Sixel wird gezeichnet (probe) und braucht `ui.image_protocol = "sixel"` |
| **iTerm2** | entfällt | entfällt | **funktioniert**: kitty-Protokoll ab 3.7.7, ohne Dialog (GitPoulpi). Ältere Versionen bekommen die Inline-Bilder mit undurchsichtigem Hintergrund. Sixel wird gezeichnet, malt aber ungesetzte Pixel deckend, sodass `ui.image_protocol = "sixel"` ein einfarbiges Rechteck ergibt |
| **Terminal.app** | entfällt | entfällt | Text: **funktioniert** (keine Bilder) |
| **Windows Terminal** | entfällt | **funktioniert**: Sixel (GitPoulpi, 1.25) | entfällt |
| **conhost** | entfällt | Text: **teilweise** (Zeichen-Graph, Halbblock-Bild-Diff) | entfällt |
| **mintty** | entfällt | **funktioniert nicht** (Terminal hinter ConPTY: Es wird gar kein Bild gezeichnet, nicht einmal ein einfaches `OSC 1337`-Bild bei sichtbarem Cursor; die Graph-Spalte bleibt leer; Text und Palette sind in Ordnung) | entfällt |

### Form des Mauszeigers (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funktioniert** | entfällt | nicht gemessen (braucht eine GPU) |
| **Ghostty** | **funktioniert** | entfällt | nicht gemessen (braucht eine GPU) |
| **WezTerm** | **funktioniert nicht** (Terminal: im getesteten Build nicht umgesetzt) | offiziell: **funktioniert nicht** (Terminal). Build mit dem Upstream-WezTerm-PR #8092, der Zeigerformen ergänzt: **funktioniert** (Hand, I-Balken, Größenpfeile) | nicht gemessen (braucht eine GPU) |
| **foot** | **funktioniert** | entfällt | entfällt |
| **Rio** | **funktioniert nicht** (Terminal: gelesen und verworfen) | **funktioniert nicht** (Terminal, aus demselben Grund; nicht ConPTY) | **funktioniert nicht** (Terminal, derselbe Quelltext; nur unter Linux und Windows gelaufen) |
| **Alacritty** | **funktioniert nicht** (Terminal) | **funktioniert nicht** (Terminal) | **funktioniert nicht** (Terminal) |
| **Tabby** | **funktioniert nicht** (Terminal: xterm.js) | **funktioniert nicht** (Terminal) | **funktioniert nicht** (Terminal) |
| **Warp** | nicht gemessen | **funktioniert nicht** (Terminal) | nicht gemessen |
| **xterm** | **funktioniert nicht** (Terminal: CSS-Namen unbekannt) | entfällt | nicht gemessen (der Zeiger lag außerhalb des Fensters) |
| **iTerm2** | entfällt | entfällt | **funktioniert nicht** (Terminal) |
| **Terminal.app** | entfällt | entfällt | **funktioniert nicht** (Terminal) |
| **Windows Terminal** | entfällt | **funktioniert nicht** (Terminal) | entfällt |
| **conhost** | entfällt | **funktioniert nicht** (Terminal) | entfällt |
| **mintty** | entfällt | **funktioniert** mit den eigenen Namen von mintty; GitPoulpi sendet sie | entfällt |

Wo „funktioniert nicht“ steht, leuchtet der Teiler oder die Schaltfläche unter der Maus trotzdem auf.

### Tastaturprotokoll

GitPoulpi fragt unter Linux und macOS nach dem kitty-Tastaturprotokoll. Unter Windows
fragt es nicht: Die Konsole meldet die Modifikatoren selbst, und die folgenden
Tastenfolgen sind dadurch begrenzt, was das Terminal für sich behält.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funktioniert** | entfällt | nicht gemessen (braucht eine GPU) |
| **Ghostty** | **funktioniert** | entfällt | nicht gemessen (braucht eine GPU) |
| **WezTerm** | **teilweise**: keine Antwort, bis `enable_kitty_keyboard = true` in `wezterm.lua` steht (Standard `false`) | **teilweise**: WezTerm behält `Ctrl+Shift+N` (öffnet sein Fenster); `Ctrl+,` kommt nicht an (offizieller Build). Der PR-#8092-Build behält keine der beiden Tastenfolgen, aber `Ctrl+,` öffnet die Einstellungen trotzdem nicht (keine Antwort zum kitty-Tastaturprotokoll durch ConPTY) | nicht gemessen (braucht eine GPU) |
| **foot** | **funktioniert** | entfällt | entfällt |
| **Rio** | **funktioniert** (standardmäßig an) | **teilweise**: Rio behält `Ctrl+Shift+N` (öffnet sein Fenster) | nicht gemessen (braucht eine GPU) |
| **Alacritty** | **funktioniert** (ab 0.13) | **teilweise**: `Ctrl+Shift+N` kommt als `Ctrl+N` an, `Ctrl+,` kommt nicht an | **funktioniert** |
| **Tabby** | **funktioniert nicht** (Terminal: keine Antwort; es behält `Ctrl+Shift+N` und `Ctrl+,` für sich) | **teilweise**: Tabby behält `Ctrl+Shift+N` und `Ctrl+,` | **funktioniert nicht** (Terminal: keine Antwort); einige Tastenfolgen kamen nicht an |
| **Warp** | nicht gemessen | nicht gemessen | nicht gemessen (braucht eine GPU) |
| **xterm** | **funktioniert nicht** (Terminal) | entfällt | **funktioniert nicht** (Terminal) |
| **iTerm2** | entfällt | entfällt | **funktioniert** (3.7.7; prüfen Sie *Profiles > Keys*, wenn nicht) |
| **Terminal.app** | entfällt | entfällt | **funktioniert nicht** (Terminal: keine Antwort; klassische Tasten) |
| **Windows Terminal** | entfällt | **teilweise**: WT behält `Ctrl+Shift+N` (neues Fenster) und `Ctrl+,` (seine Einstellungen) | entfällt |
| **conhost** | entfällt | nicht gemessen | entfällt |
| **mintty** | entfällt | **teilweise**: Keine der beiden Tastenfolgen hatte eine sichtbare Wirkung | entfällt |

Wo eine Tastenfolge nicht ankommt, nutzen Sie die `Alt`-Varianten
([Tastenkürzel](keybindings.md#tasten-die-das-terminal-nicht-senden-kann-und-das-kitty-tastaturprotokoll))
oder belegen die Tastenfolge unter *Einstellungen > Tastatur & Maus* neu.

### Zwischenablage (Kopieraktionen)

Ein Kopieren nutzt immer die System-Zwischenablage, wenn es ein lokales Display gibt;
diese Zellen beschreiben also den `OSC 52`-Weg des Terminals, der über SSH, in
Containern und als Ausweg zählt. Jeder GitPoulpi-Kopiertest, der die
System-Zwischenablage nutzte, funktionierte.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **funktioniert** | entfällt | nicht gemessen (braucht eine GPU) |
| **Ghostty** | **funktioniert** (hinter tmux, zellij, herdr gesehen); direkt: nicht gemessen (ein Schreibzugriff auf die Zwischenablage braucht den Tastaturfokus, den der Test nicht gab) | entfällt | nicht gemessen (braucht eine GPU) |
| **WezTerm** | **funktioniert** | **funktioniert** | nicht gemessen (braucht eine GPU) |
| **foot** | nicht gemessen (ein Schreibzugriff braucht den Tastaturfokus, den der Test nicht gab) | entfällt | entfällt |
| **Rio** | nicht gemessen (derselbe Grund) | **funktioniert** | nicht gemessen (braucht eine GPU) |
| **Alacritty** | nicht gemessen (derselbe Grund) | **funktioniert nicht** (Zwischenablage mit der Standardkonfiguration unverändert; nicht untersucht) | **funktioniert** |
| **Tabby** | **funktioniert** | **funktioniert** | **funktioniert** |
| **Warp** | nicht gemessen | **funktioniert nicht** (Zwischenablage unverändert) | nicht gemessen (braucht eine GPU) |
| **xterm** | nicht gemessen | entfällt | **funktioniert nicht** (Zwischenablage unverändert) |
| **iTerm2** | entfällt | entfällt | **funktioniert nicht**, bis *Settings > General > Selection > Applications in terminal may access clipboard* eingeschaltet ist |
| **Terminal.app** | entfällt | entfällt | **funktioniert nicht** (Terminal) |
| **Windows Terminal** | entfällt | **funktioniert** (solange fokussiert) | entfällt |
| **conhost** | entfällt | nicht gemessen (keine probe gelaufen) | entfällt |
| **mintty** | entfällt | **funktioniert nicht** (Zwischenablage mit den Standardeinstellungen unverändert) | entfällt |

### Einfügen

Unter Linux und macOS fordert GitPoulpi Bracketed Paste an, sodass ein mehrzeiliges
Einfügen ein Einfügen ist. Das wurde nicht Terminal für Terminal geprüft (je Terminal
**nicht gemessen**). Unter Windows hat die Konsole keine Einfüge-Klammern; ein
Terminal, das nicht zuerst fragt, liefert den Text als Tastenanschläge, und jeder
Zeilenumbruch wäre ein `Enter`. GitPoulpi behandelt deshalb Tastenanschläge, die
innerhalb von 30 ms aufeinander folgen (ein innerer `Enter` oder acht und mehr
Zeichen), als ein Einfügen. Gehaltene Tasten und von Hand getipptes „abc, `Enter`“
sind keine Einfügungen.

| Terminal | Windows (zweizeiliges Einfügen in die Eingabe für einen neuen Branch) |
|---|---|
| **Windows Terminal** | **funktioniert**: WT fragt zuerst „mehrere Zeilen einfügen?“ (seine eigene Warnung), danach wird kein Branch angelegt |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **funktioniert** (kein Branch angelegt) |
| **Warp** | nicht gemessen (GitPoulpi ließ sich darin nicht starten) |
| **conhost** | nicht gemessen (vom Test gesendetes `Ctrl+V` und `Shift+Insert` fügten nicht in die klassische Konsole ein) |

### Breite von Emoji und Symbolen

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **funktioniert** (breite Zellen ausgerichtet; kitty zeichnet das Zahnrad U+2699 leer, eine bekannte Schriftfrage) | WezTerm, Rio, Tabby, Windows Terminal: **funktioniert**. Alacritty: **teilweise** (Emoji als einfarbige Umrisse) | Alacritty, Tabby: **funktioniert**. Andere: nicht gemessen (braucht eine GPU) |
| **Tabby** | **funktioniert** | **funktioniert** | **funktioniert** |
| **Warp** | nicht gemessen | **funktioniert** | nicht gemessen |
| **xterm** | **teilweise** (Kästchen: die Schrift) | entfällt | **teilweise** (Kästchen: die Schrift) |
| **iTerm2**, **Terminal.app** | entfällt | entfällt | **funktioniert** |
| **conhost** | entfällt | **teilweise**: Farb-Emoji gezeichnet, breite Zellen ausgerichtet | entfällt |
| **mintty** | entfällt | **funktioniert** | entfällt |

### Getestete Versionen

kitty 0.49.2 (macOS 0.50.0 startete nicht), Ghostty 1.4.0, WezTerm 20260716 unter
Linux und 20240203 (das letzte getaggte Release) unter Windows, foot 1.28.0, Rio
0.5.28 (Linux, Windows) und 0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp
0.2026.11.04, xterm 411 (Linux) und 378 (XQuartz), iTerm2 3.7.7, Terminal.app 455.1
(macOS 15.8), Windows Terminal 1.25.2733, mintty 3.8.3. Terminals ändern sich
schnell: Eine neuere Version als diese kann abweichen.

## Die einzelnen Terminals

### Windows

- **Windows Terminal.** Der Standard des Starters und die beste Bildunterstützung
  unter Windows: Sixel-Graph und -Avatare, OSC 52, gute Emoji, und es beantwortet die
  Abfrage der Zellgröße. Wählen Sie eine Nerd Font unter *Settings > Profiles >
  Defaults > Appearance > Font face*. Halten Sie `Shift` gedrückt, um Text zu
  markieren, solange die Maus erfasst ist. Es behält `Ctrl+Shift+N` und `Ctrl+,`:
  Nutzen Sie die `Alt`-Varianten oder belegen Sie neu. Die Zeigerform ändert sich nicht.
- **WezTerm.** Setzen Sie in `wezterm.lua` `config.enable_kitty_keyboard = true`
  (Linux- und macOS-Builds) und eine Schrift mit Emoji-Ersatz, zum Beispiel
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Die Bilder nutzen das iTerm2-Protokoll, und nichts muss eingestellt werden: Das
  letzte getaggte Release, 20240203-110809 (Feb. 2024), verankert sie auf den richtigen
  Zeilen (im November 2026 mit dem aktuellen GitPoulpi erneut geprüft; vor der unten
  beschriebenen ConPTY-Korrektur tat es das nicht). Der Upstream-WezTerm-PR #8092
  ergänzt Zeigerformen.
- **Rio** (MSI, braucht UAC). Bilder (Graph, Avatare, Bild-Diff) nutzen das
  iTerm2-Protokoll mit Transparenz und landen genau auf ihren Zellen: gemessen mit
  0.5.28 bei der Standardschrift und bei 18 und 22,5 px (Zellen 8x16, 11x21 und 13x26
  Pixel, immer ganze Pixel); ein Bild von 20x10 Zellen und eines von 4x27 Zellen
  kamen beide zellgenau heraus. Nichts muss eingestellt werden. Die Pseudokonsole
  verschluckt die Abfrage der Zellgröße, daher nimmt GitPoulpi an, dass Rios Zellen
  doppelt so hoch wie breit sind (bei allen drei Größen zutreffend); bei einer Schrift
  anderer Proportion setzen Sie `GITPOULPI_TERMINAL_CELL_PX=WxH` (zum Beispiel
  `10x24`), um runde Commit-Markierungen zu bekommen. Rio hat auf keinem System
  Zeigerformen.
- **Tabby.** Sixel-Graph und -Avatare, Kopieren und Einfügen funktionieren. Tabby
  behält `Ctrl+Shift+N` und `Ctrl+,` für sich.
- **Alacritty.** Braucht OpenGL 3.3. Zeichen-Graph; das Kopieren über `OSC 52` war mit
  der Standardkonfiguration nicht an.
- **mintty** (Git Bash, MSYS2, Cygwin). Zeigerformen funktionieren (mit den eigenen
  Namen von mintty). Bilder werden nicht angezeigt (die Graph-Spalte bleibt leer;
  mintty 3.8.3 zeichnet hinter ConPTY kein `OSC 1337`-Bild, auch wenn eine einfache
  probe es platziert), und es beantwortet dort keine Abfrage der Zellgröße: Setzen Sie
  `ui.graph_renderer = "text"`.
- **Warp.** Mit GitPoulpi nicht gemessen: Es ließ sich aus Warps Eingabefeld nicht
  starten. Das Terminal hat weder Sixel noch iTerm2- oder kitty-Bilder.
- **conhost, cmd, ein PowerShell-Fenster.** Zeichen-Graph, Halbblock-Bild-Diff.
  Standardmäßig öffnet sich GitPoulpi, aus einem solchen Fenster gestartet, im von
  Ihnen gewählten Terminal neu; siehe „Die Neustart-Regel“.

**Die ConPTY-Grenze.** Windows lässt das Programm jedes Terminals hinter einer
Pseudokonsole (ConPTY) laufen. Windows Terminal und WezTerm-Builds, die eine
aktuelle mitbringen, reichen die Bildsequenzen durch; die in Windows eingebaute (von
Rio und mintty genutzt) und die mit WezTerm 20240203 gelieferte tun es nicht. Was
verloren geht: Die DCS-Sequenzen (Sixel) und APC-Sequenzen (kitty-Grafik) werden
verworfen, es kommen also nur die iTerm2-Bilder (`OSC 1337`) an, weshalb Rio und
WezTerm sie nutzen; `CSI 16 t` (Zellgröße) wird nur in Windows Terminal und Tabby
beantwortet. Was behoben wurde: Ein `OSC 1337` durchquert ConPTY sofort und landet
beim Cursor, den ConPTY zuletzt gemalt hat, und ConPTY malt den Cursor nur, solange
er sichtbar ist. GitPoulpi hielt den Cursor verborgen, sodass seine Bilder auf
veralteten Zeilen landeten (der Graph über der Fußzeile, ein zerrissener Bildschirm
nach dem Scrollen). Es zeigt den Cursor jetzt unter Windows um jedes geschriebene
Bild herum, im selben Schreibvorgang, und verbirgt ihn wieder; Rio und WezTerm
20240203 verankern jedes Bild auf seiner Zelle (`pty_conpty_picture_anchor` liest die
Cursorbewegung hinter ConPTY zurück). Außerhalb von Windows und bei Sixel ändert sich
nichts.

**Einfügen unter Windows.** Siehe die Tabelle „Einfügen“ oben: GitPoulpi fasst eine
Folge von Tastenanschlägen zu einem Einfügen zusammen, sodass ein mehrzeiliges
Einfügen in den getesteten Terminals nie eine Eingabe absendet.

**Die Neustart-Regel.** Ein Doppelklick auf die exe startet sie in einem klassischen
Konsolenfenster (conhost) für sich. Nennt `ui.launch_terminal` ein anderes Terminal
(Standard: Windows Terminal), öffnet GitPoulpi sich dort neu, und das erste Fenster
schließt sich (`ui.relaunch_in_terminal`). Das geschieht **nur**, wenn es allein in
einem klassischen Konsolenfenster ist. Ein Terminal, das die exe direkt startet
(`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty `-e`, Tabby `run`), ist dort, wo Sie
sein wollen, und es wird nichts neu gestartet; aus einer Shell ist es nicht allein,
also geschieht auch dann nichts. Schalten Sie es mit
`ui.relaunch_in_terminal = false` aus.

### Linux

- **kitty.** Alles funktioniert ohne Einstellung. `Shift+Klick` gehört kitty selbst,
  nutzen Sie also `Alt+Klick`, um eine Auswahl zu erweitern
  ([Maus](mouse-and-gestures.md)).
- **Ghostty.** Alles funktioniert ohne Einstellung. Erscheinen Symbole als Löcher,
  fügen Sie `font-family` eine Emoji- oder Nerd Font hinzu (wiederholen Sie den
  Schlüssel, um Ersatzschriften zu ergänzen).
- **WezTerm.** Bilder über das iTerm2-Protokoll, Transparenz eingemischt. Setzen Sie
  `enable_kitty_keyboard = true` für die vollen Tastenfolgen. Keine Zeigerformen.
- **foot.** Sixel-Graph und -Avatare, Zeigerformen, volle Tastatur, schnell. Setzen
  Sie `font=JetBrainsMono Nerd Font:size=11` in `foot.ini`. Das Kopieren über `OSC 52`
  steuert `security.osc52`.
- **Rio.** Bilder über das iTerm2-Protokoll mit Transparenz, volle Tastatur. Keine
  Zeigerformen (Rio verwirft `OSC 22`).
- **Alacritty.** Zeichen-Graph, kitty-Tastatur seit 0.13. Keine Bildunterstützung.
- **Tabby.** Sixel-Bilder; keine kitty-Tastatur, keine Zeigerformen (Electron,
  xterm.js). Es behält `Ctrl+Shift+N` und `Ctrl+,`.
- **xterm.** Starten Sie es als `xterm -u8 -ti vt340` für Sixel (UTF-8 ist für die
  Rahmenzeichen nötig). Kästchen statt Emoji: die Schrift.
- **Warp.** Nicht gemessen: Es braucht XWayland und dekodierte im Test synthetische
  Tastenanschläge falsch.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Nicht Teil des Durchlaufs vom November
  2026. Sie arbeiten mit Zeichen und klassischen Tasten: Nutzen Sie die `Alt`-Varianten
  und `Ctrl+Y` zum Wiederholen. VTE ignoriert `OSC 52`, sodass Kopieren über SSH die
  Zwischenablage nicht erreicht. Eine Eingabemethode (IBus, Fcitx) kann die
  Umschaltung `Ctrl+Space` zum Stagen abfangen; belegen Sie sie unter *Einstellungen >
  Tastatur & Maus* neu. Konsole wird für Sixel anhand der eigenen Dokumentation erkannt.
- **Wayland, Hyprland, Omarchy.** Der Hinweis auf den Mauszeiger (`OSC 22`)
  funktioniert in kitty, Ghostty und foot. Der Starter-Eintrag von Omarchy läuft
  über `xdg-terminal-exec`.

### macOS

- **iTerm2.** Ab 3.7.7 zeichnet es Bilder über das kitty-Protokoll, ohne Rückfrage.
  Einstellungen, die helfen:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    nötig für das Kopieren über `OSC 52` (sonst zeigt iTerm2 „the terminal attempted to
    access the clipboard but it was denied“).
  - *Profiles > Keys > Left Option key: Esc+*: nötig für die `Alt`-Tastenfolgen.
  - Setzen Sie `ui.image_protocol = "iterm2"` (oder nutzen Sie eine Version vor 3.7.7),
    kreuzen Sie im Dialog „Allow Terminal-Initiated Display?“ „Remember my choice“ an.
    Die Bilder haben dann einen undurchsichtigen Hintergrund (ein sichtbares Rechteck,
    wenn Ihr Hintergrund vom Design abweicht; `ui.graph_renderer = "text"` entfernt es).
  - Das kitty-Tastaturprotokoll ist standardmäßig an (3.6.9 und neuer).
- **Terminal.app.** Nur 256 Farben: GitPoulpi wählt `ui.color_depth = 256` selbst, und
  Designs wirken flacher. Keine Bilder, kein kitty-Tastaturprotokoll, keine
  Zwischenablage über `OSC 52`. Kreuzen Sie *Use Option as Meta key* für die
  `Alt`-Tastenfolgen an.
- **Alacritty, Tabby.** Alacritty: Zeichen-Graph, kitty-Tastatur, Kopieren über
  `OSC 52`; setzen Sie `option_as_alt = "Both"` für die `Alt`-Tastenfolgen. Tabby:
  Sixel-Graph und -Avatare.
- **Option als Meta.** Option setzt Zeichen zusammen (Option+B tippt ein Integralzeichen),
  sofern das Terminal sie nicht zu Meta macht: iTerm2 *Left Option key: Esc+*,
  Terminal.app *Use Option as Meta key*, Alacritty `option_as_alt`. Die
  `Alt`-Tastenfolgen von GitPoulpi brauchen das, und nirgends ist es standardmäßig an.
- **kitty, Ghostty, WezTerm, Rio, Warp**: unter macOS nicht gemessen (sie brauchen eine
  GPU). Erwarten Sie das Linux-Verhalten.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; Sixel braucht `ui.image_protocol = "sixel"`.

### VS Code und Warp

Das VS-Code-Terminal (`TERM_PROGRAM=vscode`) ist nicht Teil des Durchlaufs. Es wird
anhand seiner Geräteattribute erkannt, wenn `terminal.integrated.enableImages` an ist,
sonst zeichnet GitPoulpi Zeichen; `ui.image_protocol = "sixel"` erzwingt die Bilder.
Warp: siehe die Hinweise je System oben.

## Multiplexer

tmux, zellij, screen, herdr, psmux und die eigenen Panes der Terminals sitzen zwischen
GitPoulpi und dem Terminal und verändern, was durchkommt. Kurzfassung: tmux reicht Sixel
und, mit `allow-passthrough on`, Zeigerformen und kitty-Bilder durch; zellij und screen
reichen weder Bilder noch Zeigerformen durch; herdr reicht kitty-Bilder durch. Alle
Einzelheiten und die zu ergänzende Konfiguration stehen unter [Multiplexer](multiplexers.md).

## Was ausfällt und was stattdessen kommt

| Fehlt | Was Sie stattdessen bekommen |
|---|---|
| Bildprotokoll | Zeichen-Graph. Der Avatar wird zum farbigen Initialen-Abzeichen (ein Buchstabe), was auch immer `ui.avatar_fallback` sagt: Ein in Textzellen gepresstes Identicon ist nicht zu erkennen. Der Bild-Diff und die Dateiansicht nutzen farbige Halbblöcke oder eine Infokarte. |
| True Color | Die Farben werden auf die 256- oder 16-Farben-Palette abgebildet (`ui.color_depth`). |
| Kitty-Tastaturprotokoll | Klassische Tasten; einige Tastenfolgen kommen als andere Taste an. Die Einstellungen warnen, und Sie belegen neu oder nutzen die `Alt`-Varianten. |
| OSC 52 | Lokal ändert sich nichts (die System-Zwischenablage wird genutzt). Über SSH oder ohne Display legen Kopieraktionen nichts in die Zwischenablage: Markieren Sie mit der Maus (`Shift` gedrückt halten) und nutzen Sie das eigene Kopieren des Terminals. |
| OSC 22 | Der Zeiger ändert sich nicht; der Teiler oder die Schaltfläche unter der Maus leuchtet trotzdem auf. |
| Emoji-Schrift | Kästchen oder Löcher statt Symbolen; Graph und Text funktionieren weiter. Installieren Sie eine Nerd Font oder eine Emoji-Schrift und tragen Sie sie als Ersatz ein. |

## Einstellungen, die man kennen sollte

| Einstellung | Verwendung |
|---|---|
| `ui.image_protocol` | `auto` (Standard), `kitty`, `iterm2`, `sixel`, `none`. Setzen Sie `none`, wenn Sie Müll sehen, wo Bilder sein sollten. Neustart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` ist immer korrekt (nutzen Sie es, wo die ConPTY-Grenze gilt). Neustart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` oder `off`, wenn ein Terminal oder Multiplexer sich schlecht verhält. Neustart. |
| `ui.text_cursor` | `terminal` (Standard) oder `drawn`. |
| `ui.mouse_enabled` | Aus überlässt die Maus dem Terminal. Neustart. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | Das Terminal, das die Verknüpfungen des Betriebssystems öffnen, und ob ein Doppelklick GitPoulpi dort neu öffnet ([Einstellungen](settings.md)). |

Siehe auch [Multiplexer](multiplexers.md), [Bilder](image-view.md),
[Tastenkürzel](keybindings.md) und [Fehlerbehebung](troubleshooting.md).

## Quellen

Dokumentation im Oktober 2026 geprüft, Verhalten im November 2026 getestet. Primärquellen, sofern nicht anders vermerkt.

- kitty: [Changelog](https://sw.kovidgoyal.net/kitty/changelog/), [Zeigerformen](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [Download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [Escape-Sequenzen](https://wezterm.org/escape-sequences.html), [Pull Request zu OSC 22](https://github.com/wezterm/wezterm/pull/6292), [Releases](https://github.com/wezterm/wezterm/releases)
- foot: [Changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [Changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [Releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [Merge Request zu OSC 52](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [Merge Request zur kitty-Tastatur](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [Issue zu OSC 52](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [Issue zur kitty-Tastatur](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [Downloads](https://iterm2.com/downloads.html); kitty-Tastatur aus der [terminfo.dev-Matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) (sekundär)
- Windows Terminal: [Ankündigung 1.22, Sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (sekundär), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [Unterstützung von Escape-Sequenzen](https://rioterm.com/docs/escape-sequence-support) (keine Zeile zu `OSC 22`), [Funktionen](https://rioterm.com/docs/features), [kitty-Tastatur](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [Vollbild-Apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 aus einem [Bericht zur Warp-OSC-52-Sicherheitsmeldung](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (sekundär)
- VS Code: [Erweitertes Terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [Steuersequenzen](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [Tipps](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (kein True Color): [Apple-Diskussion](https://discussions.apple.com/thread/255315820) (sekundär)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Vergleichsseiten (sekundär, nur zum Gegenprüfen genutzt): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
