[English](../merge-conflicts.md) · [Čeština](../cs/merge-conflicts.md) · **Deutsch** · [Español](../es/merge-conflicts.md) · [Français](../fr/merge-conflicts.md) · [Italiano](../it/merge-conflicts.md) · [日本語](../ja/merge-conflicts.md) · [한국어](../ko/merge-conflicts.md) · [Nederlands](../nl/merge-conflicts.md) · [Polski](../pl/merge-conflicts.md) · [Português (Brasil)](../pt-BR/merge-conflicts.md) · [Português (Portugal)](../pt-PT/merge-conflicts.md) · [Русский](../ru/merge-conflicts.md) · [Svenska](../sv/merge-conflicts.md) · [Türkçe](../tr/merge-conflicts.md) · [Українська](../uk/merge-conflicts.md) · [简体中文](../zh-CN/merge-conflicts.md)

# Merge-Konflikte lösen

Hält ein Merge, Cherry-Pick, Revert oder Rebase bei Konflikten an, zeigt das Log
ein Banner (*Merge läuft · feature in main · 2 Dateien im Konflikt*), und die
Dateien im Konflikt erscheinen im Status-Bereich mit einem roten `!`. Öffnen Sie
eine im **Merge-Editor**: A (die aktuelle Seite) und B (die eingehende Seite)
nebeneinander, mit dem **Ergebnis**, der Datei, die Sie schreiben, darunter.
Diese Seite ist die Kurzfassung; die vollständige Tastenliste steht in
[Tastenkürzel](keybindings.md).

## Öffnen

- `Enter` oder ein Doppelklick auf eine `!`-Zeile im Status-Bereich; ihr
  Rechtsklick-Menü beginnt mit **Im Merge-Editor öffnen**.
- Die Schaltfläche **Konflikte lösen...** des Banners (sie öffnet die erste Datei
  im Konflikt).
- Die Diff-Ansicht einer Datei im Konflikt: die Schaltfläche **Merge-Editor**
  (`Shift+E`).
- Die Palette: **Konflikte: Merge-Editor öffnen**.

Nichts öffnet sich von selbst: Ein angehaltener Vorgang zeigt nur das Banner
und die `!`-Zeilen.

![Das Log eines Repositorys, das bei einem Merge mit Konflikten angehalten wurde: das Banner Merge läuft mit Konflikte lösen und Merge abbrechen, und zwei Dateien im Konflikt mit einem roten ! markiert](../images/conflict-banner.png)

Der Editor nimmt die ganze Fensterbreite ein: Die Refs-Seitenleiste
und der rechte Bereich sind ausgeblendet, solange er offen ist, und kommen zurück,
wenn Sie ihn schließen (`F9` zeigt oder verbirgt sie zwischenzeitlich). A und B
stehen nebeneinander, sobald der Editor mindestens
`merge.side_by_side_min_width` Spalten breit ist (standardmäßig 100, Einstellungen
> Diff & Editor; `0` bedeutet immer); schmaler sehen Sie jeweils eine Seite hinter
Tabs. Lange Zeilen scrollen seitwärts, A und B zusammen (`Shift+←` / `Shift+→`,
Shift+Mausrad oder ein seitliches Mausrad), sie werden nie umbrochen.

![Der Merge-Editor: A (aktuell) und B (eingehend) nebeneinander über dem Ergebnis, Konflikt 1 aus A und Konflikt 2 aus B übernommen, Konflikt 3 noch zu wählen](../images/merge-editor.png)

## Auswählen

Jeder Konflikt ist ein umrahmter Block in A und in B, mit einem Kontrollkästchen
in seiner Kopfzeile und einem `+` / `✓` vor jeder Zeile.

| Um... | Taste | Maus |
|---|---|---|
| die Zeile unter dem Cursor zu übernehmen (fokussierte Seite) | `Space` | auf die Zeile klicken |
| den ganzen Block der fokussierten Seite zu übernehmen | `Enter` | auf das `[ ]` in seiner Kopfzeile klicken |
| nur A / nur B für diesen Konflikt zu übernehmen | `a` / `b` | Rechtsklick auf den Block |
| beide zu übernehmen, A dann B / B dann A | `c` / `Shift+C` | Rechtsklick auf den Block |
| nichts zu übernehmen | `x` | Rechtsklick auf den Block |
| die Reihenfolge zweier gewählter Seiten zu tauschen | `s` | auf das Abzeichen `1st` / `2nd` klicken |
| alles aus A / alles aus B zu übernehmen (jeder Konflikt) | `Shift+A` / `Shift+B` | Symbolleiste |
| die Wahl dieses Konflikts aufzuheben (zurück zu ungelöst) | `u` in A / B, `Alt+U` überall | Rechtsklick auf den Block, **Auswahl aufheben (ungelöst)** |
| die ganze Datei auf ihre ursprünglichen Konflikte zurückzusetzen (fragt vorher nach, wenn Sie im Ergebnis getippt haben; `Ctrl+Z` bringt es zurück) | `Shift+U` in A / B, `Alt+R` überall | **Zurücksetzen** neben Alles übernehmen |
| nächster / vorheriger Konflikt | `F7` / `Shift+F7`, `Alt+Down` / `Alt+Up` (`n` / `Shift+N` in A / B) | `▼` / `▲`, oder auf eine Markierung einer Bildlaufleiste klicken |
| eine Zahl wiederholt eine Bewegung | `3j`, `2k`, `2n`, `3N` (in A / B) | |
| nächste / vorherige Datei im Konflikt | `Alt+PageDown` / `Alt+PageUp` | `›` / `‹` in der Kopfzeile |
| den Bereich Basis (gemeinsamer Vorfahre) ein- oder auszublenden | `Alt+D` (`d` in A / B) | `Basis` |
| die ganze Datei in A und B zu zeigen (**Ganze Datei**) | `Alt+F` (`f` in A / B) | `Ganze Datei` |
| A oder B mehr Breite zu geben / zurück zu gleichen Breiten | `Alt+Right` / `Alt+Left`, `Alt+0` | den Teiler ziehen; Doppelklick darauf |
| A / B / Basis seitwärts zu scrollen | `Shift+←` / `Shift+→` (`Shift+H` / `Shift+L`) | Shift+Mausrad, seitliches Mausrad |
| die Bereiche Refs und Status ein- oder auszublenden | `F9` | |
| A, B, das Ergebnis zu fokussieren | `F6` / `Shift+F6` | Klick |
| zu einem Bereich zu springen, auf dem Konflikt, auf dem Sie stehen | `Alt+1` A, `Alt+2` B, `Alt+3` Ergebnis, `Alt+4` Basis; oder `Ctrl+N` dann `1` / `2` / `3` | |
| zum Nachbarbereich zu wechseln (A, Basis, B, Ergebnis), gleicher Konflikt | `h` / `l` oder `←` / `→` in A / B | |
| die Höhe des Ergebnisses durchzuschalten | `z` | seinen Titel ziehen |
| rückgängig / wiederholen (Auswahl und Tippen, ein Stapel) | `Ctrl+Z` / `Ctrl+Y` | |
| die Datei zu schreiben, aber im Konflikt zu lassen | `Alt+S` | **Speichern** |
| die Datei zu schreiben und als gelöst zu markieren | `Ctrl+S` | **Speichern & als gelöst markieren** |
| zu schließen | `Esc` | `✕` |

Jede Bildlaufleiste (A / B / Basis und das Ergebnis) trägt eine Markierung pro
Konflikt: orange, solange noch eine Wahl nötig ist, grün, sobald erledigt, blau
für den aktuellen. Ein Klick auf eine Markierung springt zu diesem Konflikt (eine
Markierung unter dem Schieber der Bildlaufleiste ist schon im Bild, dort zieht der
Klick den Schieber).

**Ganze Datei** zeigt die ganze Datei in A und B mit jedem Unterschied farbig, nicht nur die Konflikte:

![Der Modus Ganze Datei: die ganze Datei in A und B, mit farbig markierten Änderungen](../images/merge-editor-full-file.png)

Das Ergebnis ist ein echter Editor: Klicken Sie hinein und tippen Sie, markieren,
schneiden und einfügen, suchen mit `Ctrl+F`. Eine Auswahl auf einem von Hand
bearbeiteten Konflikt ersetzt Ihre Bearbeitung (`Ctrl+Z` bringt sie zurück).
Speichern mit einem Konflikt, bei dem nichts gewählt ist, oder mit einer Zeile, die
noch wie eine Konfliktmarkierung aussieht, fragt vorher nach.

### ✦ Lösung vorschlagen (KI)

Ist die KI eingeschaltet (Einstellungen > KI; siehe [KI-Unterstützung](ai.md)),
hat die Symbolleiste eine Schaltfläche **✦ Vorschlag**, das Rechtsklick-Menü
eines Konflikts hat *✦ Lösung vorschlagen (KI)*, und `Alt+I` und die Palette
(`KI: Lösung für Merge-Konflikt vorschlagen`) tun dasselbe. Sie arbeitet nur am
aktuellen Konflikt und läuft nur auf Ihren Wunsch. Die Antwort ersetzt den Teil
dieses Konflikts im Ergebnis als eine Änderung, als bearbeitet markiert (`✎`),
sodass `Ctrl+Z` den Konflikt zurückbringt, wie er war. Sie wird nie für Sie
gespeichert oder gestagt. Während sie läuft, zeigt die Hinweiszeile die
verstrichene Zeit; `Esc` bricht ab, und ein Fehler wird dort ebenfalls gezeigt.
Bei ausgeschalteter KI gibt es keines dieser Bedienelemente.

Nach **Speichern & als gelöst markieren** ist die Datei gestagt, und die Liste
der Dateien im Konflikt ist zurück; öffnen Sie die nächste
Datei selbst. `Ctrl+Z` (oder die Schaltfläche Rückgängig) nimmt das letzte „als
gelöst markieren“ zurück. Ist keine Datei mehr übrig, bietet das Banner **Merge
per Commit abschließen** (oder **Fortsetzen** bei einem Cherry-Pick, Revert oder
Rebase) und **Merge abbrechen** an, was vorher nachfragt: Es setzt HEAD, den
Index und das Arbeitsverzeichnis dorthin zurück, wo sie waren.

Ein Merge, dessen Ergebnis HEAD gleicht (Sie haben überall unsere Seite
genommen), hat nichts gestagt und keine geänderte Datei, lässt sich aber
dennoch abschließen: **Merge per Commit abschließen** öffnet die Commit-Eingabe
mit der Merge-Nachricht, und der Commit hat beide Eltern. Ebenso ist die
Commit-Schaltfläche des Composers ohne Gestagtes aktiv, solange ein Merge
angehalten ist.

Ein leerer Cherry-Pick oder Revert (die Änderung ist schon angewendet) lässt sich
nicht committen: git lehnt es ab, und **Fortsetzen** sagt das und verweist auf
**Überspringen**. **Überspringen** (Banner-Schaltfläche, Paletteneintrag *Git:
Cherry-Pick oder Revert überspringen*, CLI `conflicts skip`) fragt vorher nach und
beendet dann den Vorgang, ohne etwas dafür aufzuzeichnen; HEAD, Index und
Arbeitsverzeichnis sind zurück bei HEAD. Überspringen ist nicht rückgängig zu
machen (nichts geht verloren, was nicht absichtlich aufgegeben wurde); ein Merge
hat nichts zu überspringen.

## Konflikte, die kein Text sind

Eine Binärdatei, ein Git-LFS-Zeiger, eine Datei über 8 MiB, eine auf einer Seite
gelöschte Datei, ein Überbleibsel einer Umbenennung, ein Symlink, ein
Moduswechsel oder ein Submodul haben keine Zeilen zum Auswählen. Der Editor zeigt
eine **Karte** mit dem, was jede Seite enthält, und einer Schaltfläche pro Wahl;
die Wahl löst die Datei auf und stagt sie sofort.

| Karte | Wahlmöglichkeiten |
|---|---|
| Binärdatei, LFS-Zeiger, zu groß | **A behalten** (`a`), **B übernehmen** (`b`), **Basis übernehmen** (`o`, wenn es eine gibt), **Im Merge-Tool öffnen** (`m`) |
| Auf einer Seite gelöscht, auf der anderen geändert | **Datei behalten** (`k`), **Löschung akzeptieren** (`Shift+D`) |
| Umbenennungs-Überbleibsel (der Pfad existiert nur auf einer Seite) | **Datei behalten** (`k`), **Datei löschen** (`Shift+D`) |
| Symlink, Submodul, Dateimodus | **A behalten**, **B übernehmen**, **Basis übernehmen** |

Ist die Datei ein Bild (PNG, JPEG, GIF, WebP, BMP, ICO), zeigen die Felder auch
die Bilder A, B und Basis, angepasst und schreibgeschützt, in einem dafür
ausreichend breiten Fenster; siehe [Bilder](image-view.md#im-merge-editor).

![Die Karte eines binären Konflikts: was A, B und die Basis enthalten, und die Schaltflächen A behalten, B übernehmen, Basis übernehmen und Im Merge-Tool öffnen](../images/merge-editor-card.png)

`Esc` schließt die Karte, ohne zu wählen. Das externe Merge-Tool ist
`general.external_merge_tool` unter Einstellungen > Diff & Editor (`{base}`, `{local}`
= A, `{remote}` = B, `{merged}`; leer bedeutet `git mergetool`); die Datei wird
gestagt, wenn das Tool erfolgreich endet.

## Konfliktvermeidung

GitPoulpi kann Ihnen Bescheid geben, *bevor* Sie mergen. Würde der Branch, auf dem Sie stehen, mit dem Branch, von dem er
abzweigte (seiner **Basis**), in Konflikt geraten, erscheint ein Streifen in der obersten Randzeile des Logs:
**Konflikte mit origin/main in 2 Dateien**, mit einer Schaltfläche **Dateien zeigen**. Er nimmt die nächste freie Randzeile
nach den Bannern laufender Vorgänge; solange eines davon angezeigt wird, muss er unter Umständen auf Platz warten. Es
wird nichts gemergt: Der Daemon führt einen Merge der beiden Commits im Arbeitsspeicher aus (libgit2 oder
`git merge-tree --write-tree` ab Git 2.38; ein älteres Git schaltet die Funktion mit einer Zeile im Protokoll ab), sodass
Ihre Dateien, der Index und die Refs unberührt bleiben.

- **Die Basis** ist der Zielbranch des Pull Requests des Branches, wenn GitPoulpi ihn schon kennt (der Cache, den der
  Abschnitt Pull Requests führt; dafür wird keine Netzwerkanfrage gestellt), sonst derjenige Branch unter den Mustern
  für **Basis-Branches** (Einstellungen > Konfliktvermeidung), der seit der Abzweigung die wenigsten Commits hat. Bei
  Gleichstand gewinnt der Branch, dessen Spitze der Abzweigung am nächsten liegt, danach ein Remote-Tracking-Branch vor
  einem lokalen, zuletzt der Name. Der Branch selbst und seine eigene Remote-Kopie sind nie die Basis.
- **Das Popup** (Klick auf den Streifen oder der Paletteneintrag **Konflikte: Vorhergesagte Konflikte zeigen**, der sich
  über die Tastatur bedienen lässt) listet die Dateien und bietet **Vorschau für Merge öffnen** und **Rebase auf Basis**.
  Das sind die eigenen Menüzeilen des Basis-Branches (*Merge von <Basis> in <Branch>*, *Rebase von <Branch> auf
  <Basis>*), für Sie ausgeführt: im Rückgängig-Verlauf erfasst, sodass `Ctrl+Z` sie zurücknimmt, und ein Merge, der
  tatsächlich Konflikte hat, hält wie jeder andere im Merge-Editor an. Links und Rechts wechseln zwischen den
  Schaltflächen, Enter drückt eine, Esc schließt.
- **Wann geprüft wird**: im Hintergrund, etwa 2 Sekunden nachdem im Repository Ruhe eingekehrt ist (ein Commit, ein
  Checkout, ein Fetch, der einen Branch bewegt hat, ein Rebase), einmal je Paar `(HEAD, Basis)`. Der Streifen
  verschwindet, sobald HEAD sich bewegt, und kommt mit der neuen Antwort wieder oder nicht. Ein losgelöster HEAD wird
  nicht geprüft.
- **Ausschalten**: **Proaktive Konflikterkennung aktivieren** unter Einstellungen > Allgemein, oder für ein einzelnes
  Repository auf dessen Seite **Konfliktvermeidung**. In einem Skript gibt `gitpoulpi conflicts check` dieselbe Antwort als
  JSON aus ([CLI-Referenz](cli-reference.md)).

## Agenten

Ein Agent kann Ihnen einen Konflikt zeigen (`gitpoulpi ui show conflict <path>
[--conflict N]`: Der Editor öffnet sich darauf, nichts wird gewählt) oder eine
Lösung **vorschlagen** (`gitpoulpi ui propose resolution <path> --pick 1=a,2=ba`
oder `--content-file F`, was *KI-Agenten Änderungen zur Bestätigung vorschlagen lassen* voraussetzt). Ein Vorschlag öffnet den Editor
mit den Auswahlen (oder dem ganzen Text) bereits im Ergebnis und einem lila Banner
*✦ Vorgeschlagen von claude-code*. Prüfen Sie ihn, ändern Sie, was Sie wollen, und
drücken Sie **Ctrl+S**: Nur das schreibt oder stagt etwas. Den Editor zu
schließen verwirft den Vorschlag. Siehe [MCP und Agenten](mcp-and-agents.md).

Aus einem Skript sind dieselben Vorgänge `gitpoulpi conflicts list | show |
resolve | unresolve | state | abort` ([CLI-Referenz](cli-reference.md)); `conflicts check` sagt sie vor einem Merge voraus.
