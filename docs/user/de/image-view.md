[English](../image-view.md) · [Čeština](../cs/image-view.md) · **Deutsch** · [Español](../es/image-view.md) · [Français](../fr/image-view.md) · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · [Polski](../pl/image-view.md) · [Português (Brasil)](../pt-BR/image-view.md) · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · [Türkçe](../tr/image-view.md) · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Bilder in der Diff-Ansicht und der Dateiansicht

Eine Binärdatei, die ein Bild ist, wird als Bild angezeigt, nicht mit dem Hinweis
„(no textual changes - binary content...)“. Die Diff-Ansicht zeigt das
**Vorher** und das **Nachher**; die Dateiansicht zeigt die Datei, wie sie ist;
der Merge-Editor zeigt die Seiten eines Bildes im Konflikt. Alles hier ist nur
lesend: Ein Bild anzusehen ändert nie das Repository, also gibt es nichts
rückgängig zu machen.

Schalten Sie es ab mit **Einstellungen > Bilder > Bilder in der Vorschau zeigen**
(`ui.image_view`): Dann erhalten Sie den einfachen Binärhinweis zurück. Ein
eigener Build ohne das Feature `image-view` hat gar keinen Betrachter, und die
Seite Bilder behält nur die Liste der Editoren (siehe
[Eigene Builds](custom-builds.md)).

## Welche Dateien

PNG und JPEG immer; GIF, WebP (ihr erstes Bild), BMP und ICO/CUR, wenn der Build
`image-view` hat. TIFF, AVIF, HEIC, PSD und RAW werden erkannt, aber nicht
dekodiert: Eine Infokarte nennt Format, Größe und Abmessungen. SVG ist Text und
behält den Text-Diff. Ein Bild über **Max. Vorschaugröße (MiB)** (standardmäßig
32 MiB) oder über 64 Megapixel zeigt stattdessen eine Infokarte. Eine mit Git LFS
gespeicherte Datei wird angezeigt, wenn ihr Objekt im lokalen Speicher liegt;
sonst sagt die Karte, dass man zuerst `LFS > Pull` ausführen soll. Transparenz
wird über einem Schachbrett gezeichnet.

## Wie gezeichnet wird

In einem Terminal mit Bildprotokoll (Kitty und Ghostty, WezTerm und iTerm2, Rio,
mintty und die Sixel-Terminals Windows Terminal, foot und Tabby) ist das Bild ein echtes Bild. Anderswo (die einfache Windows-Konsole,
zellij, ein Terminal, das GitPoulpi nicht kennt) wird es mit farbigen
Halbblock-Zeichen gezeichnet, mit halber vertikaler Auflösung, aber überall
lesbar; **Ohne Bildprotokoll** lässt sich auf *Nur Infokarte* stellen.
**Bildprotokoll** auf der Seite **Verlauf & Graph** erzwingt eines.

## Die Diff-Ansicht

Öffnen Sie den Diff einer Bilddatei (den eines Commits, die Unstaged- oder
Staged-Hälfte der Änderungen). Die Kopfzeile nennt beide Dateien
(`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`), dann die Zoom-Schaltflächen
**Anpassen**, **1:1** und **Bearbeiten in**.

- **Geteilt | Inline** rechts in der Kopfzeile, oder `m`: nebeneinander oder ein
  Bild nach dem anderen. **Tab**, oder ein Klick auf *Vorher | Nachher*, zeigt in
  der Inline-Ansicht das jeweils andere. Eine geteilte Ansicht, die für zwei
  Bereiche zu schmal ist, wird inline gezeichnet, ohne Ihre Einstellung zu
  ändern (`ui.diff_layout`).
- Der **Teiler** einer geteilten Ansicht lässt sich ziehen, mit `Alt+Right` /
  `Alt+Left` verschieben (der Vorher-Bereich nimmt / gibt Platz) und kehrt per
  Doppelklick oder `Alt+0` in die Mitte zurück. Der Anteil bleibt für die
  Sitzung gemerkt.
- Eine neue Datei zeigt einen Bereich mit der Beschriftung *Neue Datei*, eine
  gelöschte *Gelöschte Datei*; beide blenden den Umschalter aus. Ein
  unversioniertes Bild, das in den Änderungen ausgewählt ist, wird wie eine
  neue Datei in der Vorschau gezeigt.
- Die Vorher-Seite ist der Eltern-Commit (Commit-Diff), `HEAD` (staged) oder der
  Index (unstaged); die Nachher-Seite ist der Commit, der Index oder die Datei
  auf dem Datenträger. Ändert sich die Datei auf dem Datenträger, während der Diff offen
  ist (Sie haben sie aus einem Editor gespeichert, ein Werkzeug hat sie neu
  geschrieben), folgt das Bild nach etwa einer halben Sekunde.

## Die Dateiansicht

Öffnen Sie ein Bild mit **Dateiansicht** aus dem Menü einer Datei oder der
Diff-Leiste: ein Bereich, die Datei bei der Revision (oder auf dem Datenträger bei
den Änderungen).

## Zoom und Scrollen

| | |
|---|---|
| `+` / `-` (oder `=`) | Hinein- / Herauszoomen, um die Mitte des Bereichs |
| `0` | **Anpassen** (Standard): das ganze Bild; es bleibt angepasst, wenn Fenster oder Teiler in der Größe geändert werden |
| `1` | Tatsächliche Größe (1:1) |
| Pfeiltasten, `h j k l` | Um ein Zehntel des Bereichs scrollen; mit `Shift` um einen ganzen Bereich; `PageUp`/`PageDown`, `Home`/`End` |
| Mausrad | Vertikal scrollen; `Shift`+Rad horizontal; `Ctrl`+Rad zoomt um den Mauszeiger |
| Ziehen | Das Bild greifen und verschieben; die Bildlaufleisten an den Rändern funktionieren wie überall |

Eine gezoomte Ansicht behält ihren Maßstab und den Punkt in ihrer Mitte, wenn
der Bereich in der Größe geändert wird; Anpassen bleibt Anpassen. In der
Diff-Ansicht teilen sich die beiden Bereiche einen Zoom und eine
Scroll-Position (**Zoom und Scrollen vorher/nachher koppeln**, standardmäßig an),
sodass derselbe Teil beider Bilder zu sehen ist; unterscheiden sich ihre
Größen, bleibt der Zoom relativ zu jedem. Ab **Pixelig zoomen ab** (standardmäßig
2x) wird das Bild in eckigen Pixeln statt geglättet vergrößert, was zu Icons
und Pixelart passt.

## Bearbeiten in

Die Schaltfläche **Bearbeiten in** der Kopfzeile listet die auf diesem Rechner
gefundenen Bildeditoren (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Vorschau,
... und die Standard-App des Systems); `o` startet den Standard und `Shift+O`
öffnet die Liste. Sie steht auch im Rechtsklick-Menü eines Bildes in den
Dateilisten und in der Befehlspalette (*Bild: Bearbeiten in...*). Die Liste und der
Standard werden unter **Einstellungen > Bilder** festgelegt, so wie
[Öffnen in](open-in.md) funktioniert.

Die Datei auf dem Datenträger zu bearbeiten (die Seite der Änderungen) geht direkt:
Der Editor speichert an Ort und Stelle, und das Bild aktualisiert sich. Jede
andere Version (die eines Commits, der Index, eine gelöschte Datei) wird in eine
**schreibgeschützte temporäre Kopie** geschrieben, mit einem Hinweis in der
Statuszeile, sodass ein Editor das Repository nicht ändern kann.

Die Anwendungen, die das System für den Typ des Bildes registriert hat (seine Liste *Öffnen mit*), folgen Ihren Editoren in der Liste, ohne ein schon vorhandenes Programm zu wiederholen. Die übrigen Binärdateien haben dieselbe Liste unter [Öffnen in](open-in.md).

## Im Merge-Editor

Ein Bild im Konflikt zeigt seine Karte mit den Bildern **A**, **B** und (falls
vorhanden) **Basis** in den Feldern, nebeneinander, jedes an sein Feld angepasst
und schreibgeschützt (dort kein Zoom; die Bilder sollen nur bei der Wahl helfen).
Die Schaltflächen und Tasten bleiben gleich: **A behalten** (`a`), **B übernehmen**
(`b`), **Basis übernehmen** (`o`), **Im Merge-Tool öffnen** (`m`). Die Vorschauen
brauchen ein ausreichend breites Fenster (etwa 70 Spalten) und einige freie
Zeilen; schmaler ist die Karte die einfache Textkarte. Siehe
[Merge-Konflikte lösen](merge-conflicts.md).

## Tasten und Einstellungen

Die Tasten stehen unter [Tastenkürzel](keybindings.md#bildansicht); jede lässt sich
neu belegen und ist über die Palette erreichbar (`Ctrl+P`, *Bild* tippen). Die
Optionen stehen auf der Seite **Bilder** der [Einstellungen](settings.md#bilder) und in
der [Konfigurationsreferenz](config-reference.md) (`ui.image_view*`, `ui.image_editors`).
