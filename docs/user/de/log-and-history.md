[English](../log-and-history.md) · [Čeština](../cs/log-and-history.md) · **Deutsch** · [Español](../es/log-and-history.md) · [Français](../fr/log-and-history.md) · [Italiano](../it/log-and-history.md) · [日本語](../ja/log-and-history.md) · [한국어](../ko/log-and-history.md) · [Nederlands](../nl/log-and-history.md) · [Polski](../pl/log-and-history.md) · [Português (Brasil)](../pt-BR/log-and-history.md) · [Português (Portugal)](../pt-PT/log-and-history.md) · [Русский](../ru/log-and-history.md) · [Svenska](../sv/log-and-history.md) · [Türkçe](../tr/log-and-history.md) · [Українська](../uk/log-and-history.md) · [简体中文](../zh-CN/log-and-history.md)

# Mit dem Log arbeiten

Das Log ist die Mitte von GitPoulpi: der ganze Verlauf als Graph, mit jedem Branch,
Tag und Stash darauf. Diese Seite behandelt, wie man es liest, wie man darin etwas
findet und alles, was Sie mit Commits und Refs tun können. Die Bereiche sind im
[Rundgang durch die Oberfläche](interface-tour.md) benannt; die genauen Tasten
stehen unter [Tastenkürzel](keybindings.md) und die Mausgesten unter
[Maus und Gesten](mouse-and-gestures.md).

![Das Log mit ausgewähltem Merge-Commit: rechts seine Details und Dateien](../images/log-graph.png)

## Den Graphen lesen

Jede Zeile ist ein Commit, der neueste zuerst. Ein Punkt sitzt auf der Spur des
Commits; Linien verbinden einen Commit mit seinen Eltern; ein Merge zeigt zwei
Linien, die zusammenlaufen. Die Pillen links nennen die Branches und Tags an
diesem Commit, in der Farbe der Spur, mit einer dünnen Linie zum Punkt. Ein
Laptop-Symbol markiert einen lokalen Branch, ein Globus einen entfernten und `✓`
den ausgecheckten Branch. `+N` heißt, dass weitere Refs auf diesen Commit zeigen;
fahren Sie darüber, um sie aufzulisten, oder nutzen Sie das Menü unten.

Die erste Zeile sind die **Änderungen** (ein gestrichelter Kreis: Ihre nicht
committete Arbeit, verbunden mit dem Commit, auf dem sie sitzt). Stashes erscheinen
als kleine Quadrate auf einer Spur neben dem Commit, auf dem sie erstellt wurden.

### Farben, Zeilentönung und Avatare

Die Spurfarben stammen aus dem Design. *Einstellungen > Darstellung > Farbschema des
Graphen* kombiniert jedes Design mit jeder Spurpalette, und die Auswahl der Palette
*App: Farbschema des Graphen wechseln...* (`Ctrl+N Shift+T`) zeigt jede Palette live
in der Vorschau; siehe [Designs](themes.md). *Hintergrund der Graph-Spuren* auf
**Zeilentönung** gestellt malt ein schwaches Band in der Spurfarbe hinter die
Graph-Spalte und einen dünnen Balken an ihren Rand. *Stärke der Zeilentönung*
(**Stark**, **Mittel** (Standard), **Dezent**) legt fest, wie blass das Band ist.

| Spurpalette Serie mit Zeilentönung | Spurpalette Material mit Zeilentönung |
|---|---|
| ![Der Log-Graph im Farbschema Serie mit Zeilentönung](../images/theme-serie.png) | ![Der Log-Graph im Farbschema Material mit Zeilentönung](../images/theme-material.png) |

*Autoren-Avatare* (ein Bild im Graph-Punkt oder in der Spalte Autor) sind
standardmäßig aus. Sie einzuschalten sendet einen Hash der E-Mail-Adresse jedes
Autors an Gravatar und fragt Ihren Hosting-Anbieter. *Avatar-Quellen* und *Avatar-
Ersatz* auf derselben Seite bestimmen, woher die Bilder kommen und was gezeichnet
wird, wenn es keines gibt.

Kann Ihr Terminal keine Bilder zeigen, wird der Graph mit Zeichen gezeichnet.
*Graph-Darstellung* und *Bildprotokoll* erzwingen das eine oder das andere; siehe
[Fehlerbehebung](troubleshooting.md).

Die Hervorhebung der ausgewählten oder überfahrenen Zeile beginnt am Kreis des
Commits. Die Spuren links davon behalten den normalen Hintergrund.

### Spalten

Rechtsklick auf die Kopfzeile wählt die Spalten: **Ref**, **Graph**, **Nachricht**
(immer an), **Autor**, **Datum**, **Änder.** und **SHA**. Die optionale Spalte
**Änder.** zeigt für jeden Commit `n Dateien +hinzugefügt -entfernt`. Ziehen Sie den
Rand zwischen zwei Kopfzellen, um eine Spalte in der Größe zu ändern; die Wahl
wird gemerkt, in Terminalzellen. Mit dem Standard `ui.column_resize = "line"` ist der Rand die einzellige Lücke am Ende einer Spalte: Eine kleine Markierung ruht darin in der Kopfzeile, und solange der Zeiger auf der Lücke steht (in der Kopfzeile oder irgendwo in der Tabelle), wird über die volle Höhe der Tabelle eine dünne Akzentlinie gezeichnet, und ein Ziehen von jedem Punkt daran ändert die Größe (in Terminals, die es unterstützen, wird der Zeiger zum Größenänderungszeiger). Mit `"header"` gibt es weder Markierung noch Linie: Das erste Zeichen der Kopfzelle nach dem Rand leuchtet beim Überfahren auf, und das Ziehen beginnt dort, nur in der Kopfzeile (Einstellungen > Verlauf & Graph > Spaltengröße ändern). Die Zeilen sehen in beiden Modi gleich aus: Die Lücke gehört zur Spalte links davon, sodass Auswahl und Zeilenfärbung durch sie hindurchlaufen. Ref, Graph und die übrigen Spalten behalten ihre Breite, wenn das Fenster verkleinert, vergrößert oder maximiert wird: Nur **Nachricht** wächst oder schrumpft (in einem sehr schmalen Fenster weichen die anderen Spalten vorübergehend aus, zuerst der Graph, und kehren zu Ihren Breiten zurück, sobald das Fenster wieder wächst). Ziehen Sie die Spalte Graph auf ihre kleinste Breite, einen Commit-Punkt, wird der Graph kompakt: Jeder Commit liegt auf einer einzigen Spur, einer unter dem anderen, ohne Branch- oder Merge-Linien. Eine Zelle breiter, und der Baum ist zurück. Mit den Bildern der Autoren in den Commit-Kreisen (Einstellungen > Darstellung > Autoren-Avatare: graph) ist der Kreis zwei Zellen breit, sodass die Spalte eine Zelle früher (bei drei Zellen) haltmacht und nie ein Bild abschneidet.

Beim Ziehen einer Kopfzelle folgt ein Schatten ihres Namens dem Zeiger, und ein Akzentbalken zeigt, wo die Spalte landen würde; die Reihenfolge ändert sich beim Loslassen, und `Esc` bricht ab. Die Auswahl, die der Rechtsklick öffnet, ist eine Checkliste (`[x]` / `[ ]`) mit einem ✕ im Titel; sie ist auch über die Palette erreichbar (`Ansicht: Log: Anzuzeigende Spalten wählen`), wird mit den Pfeiltasten bewegt und mit `Space` / `Enter` umgeschaltet.

Per Tastatur wählt `Shift+C` (oder in der Palette `Ansicht: Log-Spalte: Spalte zum Verschieben oder Anpassen wählen`) eine Spalte, und ihre Kopfzelle leuchtet auf. Dann wählen `Tab` / `Shift+Tab` eine andere Spalte, `<` / `>` verschieben sie um einen Platz nach links oder rechts unter den angezeigten Spalten (das Tastaturäquivalent zum Ziehen ihrer Kopfzelle), `]` / `[` verbreitern oder verschmälern sie (das Äquivalent zum Ziehen ihrer Grenze; die letzte Spalte bewegt ihren linken Rand), `0` setzt alle Breiten auf automatisch zurück, und `Esc` oder `Enter` beendet. Die Pfeiltasten bewegen sich währenddessen weiter durch die Commits. Reihenfolge und Breiten werden wie nach dem Ziehen gemerkt und gehören nur dem Log: Die Commit-Spalte neben einem Diff führt eine eigene Liste.

*An den Inhalt anpassen.* Ein **Doppelklick** auf die Grenze einer Spalte passt die Spalte links davon an ihren breitesten sichtbaren Inhalt und ihre Kopfzeile an (breite Zeichen und Emoji zählen mit den Zellen, die sie belegen). Im Layout Anpassen füllt das Log immer seinen Bereich, eine Spalte hört also bei dem auf, wofür der Bereich Platz hat, sobald die **Nachricht** ihren Anteil und jede andere Spalte ihre Mindestbreite behält: Die Statuszeile sagt dann `Autor braucht 34 Zellen, 20 passen.` Dasselbe per Tastatur: Bei gewählter Spalte (`Shift+C`, `Tab`) passt `=` sie an und `+` alle Spalten; die Palette hat `Ansicht: Log-Spalte: Spalte an Inhalt anpassen` und `... Alle Spalten an Inhalt anpassen`; die Rechtsklick-Auswahl listet unter ihren Kontrollkästchen `<Spalte> an Inhalt anpassen` (die Spalte unter dem Zeiger), `Alle Spalten an Inhalt anpassen` und `Spaltenbreiten zurücksetzen`. Die **Nachricht** ist die flexible Spalte und hat keine eigene Breite zum Anpassen: Sie nimmt den Platz, den die anderen lassen.

*Seitlich scrollen.* Standardmäßig passt jede Spalte in den Bereich. Im Layout **Seitlich scrollen** (das Kontrollkästchen `Seitlich scrollen (Spalten behalten ihre Breiten)` der Auswahl, Einstellungen > Verlauf & Graph > Spaltenlayout, in der Palette `Ansicht: Log: Seitliches Scrollen der Spalten umschalten` oder `s` bei gewählter Spalte) behalten die Spalten ihre Breiten, auch die **Nachricht** (sie hat dort eine eigene Breite, und das Anpassen nimmt ihre breiteste Zusammenfassung), bei schmalem Fenster wird nichts aufgegeben, und die Tabelle kann breiter als der Bereich sein. Eine Leiste am unteren Rand scrollt sie dann: Ziehen Sie den Griff, klicken Sie auf die leere Schiene, um den Griff unter den Zeiger zu holen, oder nutzen Sie ihre Endpfeile. Per Tastatur scrollen `Shift+Links` / `Shift+Rechts` um vier Zellen (die Palette hat `Ansicht: Log: Spalten nach links scrollen` / `rechts`); das horizontale Mausrad oder `Shift`+Rad über der Tabelle tun dasselbe, und das Wählen einer Spalte (`Shift+C`, `Tab`) scrollt sie ins Bild. Eine Tabelle, die schmaler als der Bereich ist, lässt rechts Leerraum, und das Band einer gewählten Zeile reicht bis zum Rand. Jedes Layout behält seine eigenen Breiten (`ui.log_column_widths` und `ui.log_column_widths_scroll`); das erste Ziehen, die erste Taste oder das erste Anpassen im Layout Seitlich scrollen speichert jede Spalte mit der Breite, die sie hat, sodass sich nur die eine bewegt, die Sie ändern. Im Layout Anpassen sagt ein Anpassen, das nicht in den Bereich passt, das und verweist auf dieses Layout.

*Der Griff der letzten Spalte.* Auch die letzte Spalte hat einen Griff, auf ihrer eigenen letzten Zelle (nicht der Rand des Bereichs, der weiterhin die Bereiche in der Größe ändert): Ein Ziehen verschiebt sie um die Strecke des Zeigers, wobei die **Nachricht** den Unterschied aufnimmt. **SHA** ist standardmäßig 8 Zellen breit (die 7 Ziffern und die Griffzelle).

![Das Log mit eingeblendeter Spalte Änder.: Jeder Commit nennt die Zahl seiner Dateien und der hinzugefügten und entfernten Zeilen](../images/log-changes-column.png)

### Autoren und `.mailmap`

Hat das Repository eine `.mailmap` (Gits Datei, die einer Person, die unter mehreren Namen und
E-Mail-Adressen committet hat, einen einzigen gibt), zeigen die Spalte **Autor** des Logs, die
Commit-Details, Blame und der Dateiverlauf die **kanonische** Identität, wie `git log`. Die Commits
selbst bleiben unberührt. Die Einstellung `log.use_mailmap` (Einstellungen > Verlauf & Graph,
standardmäßig an) schaltet es aus und zeigt die gespeicherten Namen. Die Suche nach Autor findet
Name und E-Mail-Adresse, wie sie angezeigt werden, sodass eine Suche nach dem kanonischen Namen
jeden Commit der Person findet. Der Tab **Identitäten** der Werkbank für den Verlauf listet die
Namen und E-Mail-Adressen des Verlaufs und schreibt `.mailmap`-Zeilen für Sie, siehe
`history-rewrite.md`; im Terminal: `gitpoulpi mailmap list|check|add`.

## Dinge finden

- **Suche** (`f`, die Schaltfläche **Suchen** der Symbolleiste oder
  *Navigation: Commits suchen...* in der Palette): durchsucht die ganze
  Commit-Nachricht, Name und E-Mail des Autors, den Hash und die Ref-Namen. Was
  nicht passt, wird abgedunkelt, und die Form des Graphen bleibt. `Enter` /
  `Shift+Enter` springen zum nächsten / vorherigen Treffer.
- **Refs-Filter** (`/` in der Seitenleiste): grenzt die Listen der Branches, Tags
  und Remotes nach Namen ein.
- **Solo** (die Taste `o` oder das Menü der Ref, auf einer Ref, einem Ordner oder
  Remote): zeigt nur die Commits der Solo-Refs; mehrere im Solo zeigen ihre
  Vereinigung. `Shift+O` beendet das Solo.
- Eine Ref **ausblenden** (ihr Menü), um sie aus dem Graphen zu nehmen: Ihre Pille verschwindet,
  und mit ihr die Commits, die nur sie erreicht (der ausgecheckte Commit bleibt immer, mit der
  Arbeitsbaum-Zeile). *Ausgeblendet n* im Titel von Refs zählt sie, **Alle zeigen** (`Shift+H`)
  holt sie zurück. Die Gesamtzahl der Statuszeile zählt den sichtbaren Verlauf.

## Branches, Tags und die Refs-Seitenleiste

Doppelklick auf einen lokalen Branch oder Tag checkt ihn aus. Rechtsklick auf eine
Ref (in der Seitenleiste oder ihre Pille im Log) öffnet ihr Menü. Die nützlichsten
Einträge:

| Eintrag | Wirkung |
|---|---|
| Im Graph anzeigen | Scrollt das Log zum Commit der Ref (lädt bei Bedarf zuerst ältere Commits) und lässt die Zeile aufblinken; die Auswahl bleibt. Ein einfacher Klick auf eine Ref bewirkt dasselbe, solange *Klick auf eine Ref zeigt sie im Graph* in den Einstellungen nicht ausgeschaltet ist. Eine im Graph (oder durch Solo) ausgeblendete Ref meldet das stattdessen in der Statuszeile |
| Checkout | Wechselt auf den Branch oder löst sich am Tag |
| Merge, Rebase auf | Merget diese Ref in den ausgecheckten Branch oder rebased den ausgecheckten Branch auf sie |
| Fast-Forward `<Branch>` auf `<aktuell>` / `<aktuell>` auf `<Branch>` | Schiebt einen Branch vor, wenn das ein Fast-Forward ist; sonst ausgegraut mit dem Grund (`diverged, not a fast-forward`) |
| Worktree erstellen aus... | Öffnet einen verknüpften Worktree in einem eigenen Tab |
| Pull, Push, Fetch, Remotes von <Branch>... | Netzwerkvorgänge auf diesem Branch (Remotes von <Branch>... setzt den Upstream, dazu je Remote ein Kontrollkästchen Push und Auch Pull) |
| Im Graphen ausblenden / zeigen, Solo | Nur Anzeige; blenden eine Ref aus (das Ausblenden nimmt auch die Commits mit, die nur diese Ref erreicht) oder stellen sie solo, ohne das Repository zu berühren |
| Umbenennen, Löschen | Löschen fragt vorher; ein Remote-Zwilling kann ebenfalls gelöscht werden |
| Branch-Namen kopieren, Link zum Branch auf dem Remote kopieren | In die Zwischenablage |

**Wo ein Tag liegt.** Ein Tag kann auf diesem Computer sein, auf einem Remote oder auf beidem,
und die Zeilen TAGS der Seitenleiste (und die Pille des Tags im Log) zeigen es mit einer Markierung
hinter dem Namen. Wer einen Tag überfährt, bekommt dasselbe in Worten.

| Markierung | Bedeutung |
|---|---|
| Pfeil nach oben | Nur hier: Kein Remote hat ihn. Sein Menü bietet *Push* |
| Pfeil nach links und rechts | Hier und auf einem Remote, am selben Commit |
| Pfeil nach unten (Zeile abgedunkelt) | Nur auf einem Remote: nicht in diesem Repository (nie gefetcht, oder hier gelöscht). Sein Menü bietet *Fetch `<tag>` von `<remote>`* und *`<tag>` auf `<remote>` löschen...*; `Git: Fetch für den ausgewählten Tag` in der Palette |
| Pfeil nach oben und unten | Auch auf einem Remote, aber an einem anderen Commit (ein Push würde abgelehnt) |

Git führt für Tags keine Remote-Tracking-Refs, darum erfährt GitPoulpi, was jedes Remote hat, aus
Ihrem letzten **Fetch** (jedem Fetch, auch dem automatischen) und aus den eigenen Tag-Pushes und
-Löschungen. Beim Stöbern wird nie das Netzwerk gefragt; hat jemand anderes die Tags eines Remotes
geändert, hilft ein Fetch, es zu sehen. Dieselbe Information steht in `gitpoulpi tag list` (Felder
`local`, `remote`, `differs_on`, `where`).

**Einen Tag löschen.** Hat ein Remote den Tag, öffnet das rote **Löschen** einen Dialog mit drei
nummerierten Zeilen (`1`-`9`, oder Auf/Ab und `Enter`; `Esc` bricht ab), und die hervorgehobene
Zeile erklärt sich darunter selbst:

1. **Lokal löschen**: entfernt ihn nur aus diesem Repository. Es läuft sofort, und die Meldung sagt,
   dass das Remote ihn noch hat. **Der nächste Fetch bringt ihn zurück** (Git folgt den Tags des
   Remotes), es sei denn, Sie löschen ihn auch auf dem Remote.
2. **Auf `<remote>` löschen**: entfernt ihn dort für alle (`git push <remote> --delete`). Es verlangt
   eine Bestätigung, die das Remote nennt. Ihr lokaler Tag bleibt. Das lässt sich nicht rückgängig
   machen.
3. **Lokal und auf `<remote>` löschen**: beides, nach derselben Bestätigung, sodass ein Fetch ihn
   nicht zurückbringen kann. **Rückgängig stellt nur den lokalen Tag wieder her** (dasselbe
   Tag-Objekt); er ist dann wieder nur lokal, und *Push* legt ihn auf dem Remote neu an.

![Der Dialog Tag löschen mit seinen drei Zeilen Lokal löschen, Auf origin löschen und Lokal und auf origin löschen, über dem Bereich Tags der Refs-Spalte, in dem jeder Tag eine Markierung trägt, ob er nur lokal oder auch auf origin liegt](../images/tag-delete.png)

Bei mehreren Remotes bekommt jedes seine eigenen Zeilen. Ein Tag nur hier behält die einfache
Bestätigung; ein Tag nur auf einem Remote geht direkt zur Bestätigung dieses Remotes.
CLI: `gitpoulpi tag delete <name> [--local | --remote <r> | --both [--remote <r>]]`.

**Fetch und Tags.** Ein Fetch aktualisiert auch, was jedes Remote hat (die Markierungen), und das
eigene Tag-Folgen von Git gilt weiter: Ein Tag, der nur hier gelöscht wurde, wird wieder gefetcht.
Ein Tag, den jemand **auf dem Remote gelöscht** hat, verschwindet nach jedem Fetch von der
Remote-Seite der Markierung, bleibt hier aber als Ihr lokaler Tag (jetzt *nur hier*), weil Git
Tags standardmäßig nicht bereinigt. **Einstellungen > Synchronisierung > Auto-Prune Tags** (standardmäßig aus;
`gitpoulpi fetch --prune-tags`) löscht auch einen solchen lokalen Tag, aber nur einen, der auf dem
Remote gelegen hatte: Ein Tag, den Sie nie gepusht oder den Sie verschoben haben, wird nie
angefasst. Um zu erfahren, was bereinigt wurde, braucht es Git 2.41 oder neuer.

**Neuer Branch oder Tag.** Die Schaltflächen **Branch** und **Tag** der
Symbolleiste, `Ctrl+G b` und `Ctrl+G t`, oder *Branch hier erstellen...* eines
Commits fragen den Namen in einem Inline-Feld in der Spalte Ref ab, in der Zeile des
Ziel-Commits. `Enter` legt ihn an, `Esc` bricht ab, und ein Name, den git ablehnen
würde, wird rot gemeldet, ohne das Feld zu schließen.

Bei einem Commit ohne Ref zeigt das Überfahren seiner Zeile außerdem eine blasse Pille in der Spalte Ref mit dem Namen des Branches, zu dem er gehört (sonst Remote-Branch), aus dem Verlauf abgeleitet; Tags werden nie als Platzhalter gezeigt. Ein nach einem Merge gelöschter Branch erscheint weiterhin, durchgestrichen, unter dem Namen aus dem Betreff des Merge-Commits (`Merge branch 'x'`, `Merge pull request #N from owner/x`...); Squash-Merges, Rebases und Fast-Forwards hinterlassen keine solche Spur. Fährt man über die Pille, erscheint ihre Art. Sie legt nichts an: nutzen Sie den Befehl zum Erstellen eines Branches oben. **Platzhalter für Branch am überfahrenen Commit zeigen** unter Einstellungen > Verlauf & Graph schaltet sie ab; wo die Spalte Ref zu schmal ist, wird nichts gezeichnet.

**Remotes.** Das `+` im Titel REMOTE fügt eines hinzu; Rechtsklick auf den Namen
eines Remotes, um es zu fetchen, seinen Weblink zu kopieren, es zum Standard-Remote
zu machen, es im Graph auszublenden, seine URLs zu bearbeiten oder es zu
entfernen. **Ein Remote ausblenden** (das ⊙ seiner Zeile, im Menü *Remote im Graph ausblenden* oder in der Palette `Remote: Remote im Graph ausblenden...`) blendet jeden Branch aus, den es jetzt hat oder später bekommt, und die Commits, die nur es erreicht, verlassen das Log: so durchsuchen Sie einen Verlauf ohne den eines fremden Archiv-Remotes und holen ihn mit *Remote im Graph einblenden* zurück. Das Einblenden eines Remotes zeigt auch die Branches von ihm wieder, die einzeln ausgeblendet waren. Ein Branch eines ausgeblendeten Remotes sagt das in der Statuszeile, wenn sein ⊙ benutzt wird: Blenden Sie zuerst das Remote ein.

**Stashes.** **Stash** stasht alles (unversionierte Dateien eingeschlossen);
`Ctrl+G m` fragt zuerst nach einer Nachricht (bei eingeschalteter KI schlägt
`Ctrl+G` in dieser Eingabe eine vor). Rechtsklick auf eine Stash-Zeile, im Log oder
im Abschnitt STASHES, für **Pop**, **Anwenden** (behält den Stash) oder **Drop**.

**Worktrees.** *Worktree erstellen aus...* (auf einem Branch, Tag oder Commit)
öffnet einen Dialog, der einen Ordner neben dem Repository vorschlägt, einen
optionalen neuen Branch-Namen und „Den neuen Worktree als Tab öffnen“. Worktrees
erscheinen im Abschnitt WORKTREES; Doppelklick auf einen öffnet ihn, und sein Tab
schließt sich von selbst, wenn der Worktree entfernt wird. Die ganze Funktion
(in einem anderen Werkzeug öffnen, den Branch eines Worktrees mergen, entfernen)
steht in [Worktrees](worktrees.md); der Abschnitt SUBMODULE desselben Bereichs in
[Submodule](submodules.md).

## Refs auf Commits ziehen

Sie können auf einen Branch (in der Seitenleiste oder seine
Pille im Log) drücken und ihn ziehen. Ein Geisteretikett folgt dem Zeiger, und
gültige Ziele leuchten auf; lassen Sie los, um ein Menü zu erhalten:

- ein lokaler Branch **auf einen anderen lokalen Branch**: Merge, Rebase,
  Interaktiver Rebase, Fast-Forward;
- ein lokaler Branch **auf einen Remote-Branch**: Push zu diesem Remote;
- ein Remote-Branch **auf einen lokalen Branch**: Merge, Rebase, Fast-Forward, Pull.

`Esc` bricht das Ziehen ab. Das Tastatur-Gegenstück ist der Palettenbefehl
*Refs: Ausgewählte Ref auf dem ausgecheckten Branch ablegen...*.

**Merge oder Fast-Forward.** Hat das Ziel keinen eigenen Commit (der gezogene Branch ist ihm nur voraus), könnte ein
Merge das Ziel einfach vorrücken. Standardmäßig (Einstellungen > Git > **Merge: Fast-Forward** = *never*, oder die eigene Wahl eines
Repositorys auf seiner Seite Merge-Optionen) lautet die Menüzeile *Merge von x in y (Merge-Commit)* und erzeugt einen echten Merge-Commit,
damit der Verlauf den Branch behält; für das schlichte Vorrücken gibt es die getrennte Zeile *Fast-Forward von y auf x* (für eine gerade
Linie ist *Rebase* da). Mit *allowed* lautet die Merge-Zeile *(Fast-Forward)*, mit *only* *(nur Fast-Forward)*, abgeblendet und mit Begründung,
wenn die Branches auseinandergelaufen sind. Dieselben Zeilen und Beschriftungen stehen im eigenen Menü eines Branches, und
`gitpoulpi merge --ff | --no-ff | --ff-only` wählt je Merge.

## Das Commit-Menü

Rechtsklick auf einen Commit (oder `.` darauf) öffnet sein Menü:

![Das Commit-Menü: Cherry-Pick, Revert, Reset, Nachricht bearbeiten, mit KI erklären, interaktiver Rebase, Branch erstellen, Tag, Checkout, Worktree, Patches und Kopieraktionen](../images/context-menu.png)

| Eintrag | Wirkung |
|---|---|
| **Cherry-Pick auf `<Branch>`** | Spielt den Commit auf dem ausgecheckten Branch erneut ab |
| **Commit rückgängig machen (Revert)...** | Legt einen Commit an, der ihn aufhebt. Bei einem Merge-Commit fragt zuerst ein Dropdown, welcher Elternteil die Hauptlinie ist (die Seite, die bleibt; Elternteil 1 ist der Branch, auf dem der Merge erstellt wurde), danach folgt die übliche Bestätigung. Rückgängig stellt den früheren Zustand wieder her |
| **Reset von `<Branch>` auf diesen Commit** | Ein Untermenü für die Art des Resets; ein Hard-Reset fragt nach Bestätigung |
| **Commit-Nachricht bearbeiten...** | Siehe unten |
| **Ändern...** | Ändert Nachricht sowie Name, E-Mail und Datum des Autors des Commits in einem Dialog; siehe [Einen Commit ändern](#einen-commit-ändern-nachricht-autor-datum) |
| **Commit aufteilen...** | Teilt den Commit in mehrere auf; siehe [Einen Commit aufteilen](#einen-commit-aufteilen) |
| **Commit aufteilen (KI)...** | Dasselbe, mit einer vom Modell vorgeschlagenen Gruppierung; nur wenn die [KI](ai.md) an ist |
| **Änderungen erklären (KI)** | Ein Popup mit einer KI-Erklärung; nur wenn die [KI](ai.md) an ist |
| **Interaktiver Rebase ab hier...** | Siehe unten |
| **Branch hier erstellen...**, **Tag hier erstellen...**, **Annotierten Tag hier erstellen...** | Inline-Namensfeld |
| **Diesen Commit auschecken**, **Worktree hier erstellen...** | Losgelöster HEAD oder ein Worktree |
| **Patch aus diesem Commit erstellen...**, **von hier bis `<Branch>`...** | Schreibt `.patch`-Dateien; *Patch anwenden* steht im Menü einer Ref |
| **Link zum Commit auf dem Remote kopieren**, **Kopieren** > | In die Zwischenablage (Hash, Nachricht und mehr) |

## Eine Commit-Nachricht bearbeiten (Reword)

*Commit-Nachricht bearbeiten...* (oder `Ctrl+G e`) öffnet die ganze Nachricht: eine
einzeilige Zusammenfassung mit Zähler, darunter eine Beschreibung, **Abbrechen** und
**Nachricht aktualisieren**. Jeder Commit des ausgecheckten Branches kann umformuliert
werden (HEAD oder ein Vorfahre); die Commits darüber werden mit ihren eigenen
Trees, Autoren und Daten neu erzeugt, Merges bleiben Merges, und Ihre gestagten und
ungestagten Änderungen bleiben unberührt. Wurde der Commit schon gepusht, warnt das
Popup in Bernstein, dass Sie einen Force-Push (mit Lease) brauchen, und verlangt eine
zweite Bestätigung. `Ctrl+Z` (Rückgängig der Symbolleiste) nimmt das Reword zurück;
siehe [Rückgängig und Wiederholen](undo-redo.md). Die Statuszeile nennt auch die
vorherige Spitze des Branches.

Ein Commit, der nicht auf dem ausgecheckten Branch liegt (er lebt auf einem anderen Branch),
lässt sich dort, wo Sie sind, weder umformulieren noch ändern, deshalb öffnen *Commit-Nachricht bearbeiten...* und
*Ändern...* stattdessen **Einen Commit eines anderen Branches bearbeiten**, eine Liste von Möglichkeiten
(`Up` / `Down` oder `Ctrl+J` / `Ctrl+K` bewegen, `Enter` oder ein Klick wählt, `Esc`, das
Kreuz oder **Abbrechen (A)** schließt sie):

- **`<Branch>` auschecken und bearbeiten (1)**: eine Zeile je lokalem Branch, dessen Verlauf
  den Commit enthält (`1` bis `8`, die ersten acht, der Rest wird mitgezählt). Der
  Branch wird auf die übliche Weise ausgecheckt, dann öffnet sich die Bearbeitung darauf. Die Rückgängig-Geschichte
  ist die der beiden Aktionen: `Ctrl+Z` nimmt die Bearbeitung zurück, das Auschecken bleibt ein
  Auschecken.
- **Mit erweitertem Umschreiben bearbeiten (U)**: öffnet die
  [Werkbank zum Umschreiben des Verlaufs](history-rewrite.md) an diesem Commit, in Builds, die
  sie haben.
- **Branch hier erstellen... (B)**: nur wenn kein lokaler Branch den Commit enthält
  (er liegt etwa nur auf einem Remote-Branch); öffnet das Namensfeld für einen Branch daran.

## Einen Commit ändern (Nachricht, Autor, Datum)

*Ändern...* im Commit-Menü (oder `Ctrl+G Shift+E`, oder *Commit: Ändern...* in der Palette) öffnet einen einzigen Dialog für die Eigenschaften des Commits: Zusammenfassung, Beschreibung sowie Name, E-Mail und Datum des Autors, jeweils mit den Werten des Commits vorbelegt. Ändern Sie beliebige Felder und drücken Sie **Anwenden**; es werden nur die geänderten Felder gesendet. Ein Datum wird als `YYYY-MM-DD HH:MM:SS +HH:MM` geschrieben und in UTC angezeigt, bis Sie es bearbeiten. Als Committer werden Sie mit dem aktuellen Zeitpunkt eingetragen (wie bei einem Reword). Jeder Commit des ausgecheckten Branches lässt sich ändern, mit denselben Regeln, derselben Warnung für bereits gepushte Commits und derselben zweiten Bestätigung wie *Commit-Nachricht bearbeiten...*; die Commits darüber werden mit ihren eigenen Autoren und Daten neu erstellt, und Ihre gestagten und ungestagten Änderungen bleiben unberührt. `Ctrl+Z` (Rückgängig in der Symbolleiste) nimmt es zurück. In Builds mit Verlaufs-Umschreibung schließt **Verlauf weitergehend ändern...** (`Alt+H`) unten links im Dialog diesen, ohne etwas anzuwenden, und öffnet die [Werkbank zum Umschreiben des Verlaufs](history-rewrite.md) mit dem Bereich von diesem Commit bis zum neuesten; haben Sie ein Feld geändert, fragt es vorher (`Esc` bestätigt). `Ctrl+N` zeigt ein Badge auf jedem Feld und jeder Schaltfläche des Dialogs; drücken Sie es, um den Cursor in dieses Feld zu setzen oder die Schaltfläche zu drücken (siehe [Badges](keybindings.md#badges)).

Jeder Weg in den Dialog öffnet ihn vollständig, mit allen Feldern sichtbar und bearbeitbar; nur der Cursor steht anders. Im Detailbereich des Commits (rechts) Rechtsklick auf die Zusammenfassung, die Beschreibung oder den Autorblock und *Zusammenfassung ändern...*, *Beschreibung ändern...* oder *Autor ändern...* wählen, oder die Zusammenfassung, die Beschreibung, den Autor oder das Autordatum doppelt anklicken: Der Dialog öffnet sich mit dem Cursor in diesem Feld. Die Tastaturentsprechungen funktionieren im Log und im Detailbereich: `Ctrl+G Shift+E` (alle Felder), `Ctrl+G Shift+A` (der Autor) und in der Palette *Commit: Ändern...*, *Commit: Autor ändern...*, *Commit: Zusammenfassung ändern...* und *Commit: Beschreibung ändern...*. Beide Kürzel lassen sich neu belegen (`git_prefix.amend`, `git_prefix.edit_author`; siehe [Tastenkürzel](keybindings.md)). Aus einem Skript oder von einem Agenten: `gitpoulpi amend <commit>` mit `--message` (oder `--summary` / `--description`), `--author-name`, `--author-email`, `--author-date` und den entsprechenden `--committer-*`-Optionen (das MCP-Werkzeug heißt `amend`).

## Einen Commit aufteilen

*Commit aufteilen...* im Commit-Menü (oder in der Palette: *Git: Ausgewählten Commit aufteilen...*)
teilt einen Commit in zwei oder mehr auf, älteste zuerst. Jeder Nicht-Merge-Commit des
ausgecheckten Branches lässt sich aufteilen (HEAD oder ein Vorfahre, auch ein Root-Commit).
Der Aufteilungseditor hat zwei Bereiche: links die **Neuen Commits** (je eine Nachricht) und
rechts die **Änderungen** des Commits, eine Zeile pro Datei und Hunk, jede mit einem
Feld `[N v]`, das den neuen Commit nennt, der sie erhält.

| Taste | Maus | Wirkung |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | eine Zeile anklicken | über die Änderungen gehen (`Shift+Up` / `Shift+Down` erweitert eine Auswahl) |
| `1`-`9` | das Feld `[N v]` der Zeile, dann eine Auswahl | die ausgewählte Änderung diesem neuen Commit zuordnen; `0` lässt sie unzugeordnet |
| `Enter` / `m` | das Feld `[N v]` | die Liste der Commits für die ausgewählte Änderung öffnen |
| `n` | `+ Neuer Commit` | die ausgewählte Änderung in einen neuen Commit legen |
| `v` | | **Zeilenmodus**: einen Hunk Zeile für Zeile aufteilen (über Zeilen ziehen, um einen Bereich zu wählen) |
| `Tab` | einen Bereich anklicken | zwischen den Commits und den Änderungen wechseln |
| `e` / `Enter` (im Commit-Bereich) | der Stift | die Nachricht dieses Commits bearbeiten |
| `d` / `Delete` | das Kreuz | einen neuen Commit entfernen (seine Änderungen werden unzugeordnet) |
| `Alt+Up` / `Alt+Down` | die Pfeile | einen neuen Commit nach oben oder unten verschieben |
| `s` | das Kontrollkästchen | die neuen Commits signieren |
| `Shift+R` | `Zurücksetzen` | zurück zum ursprünglichen Plan |
| `Ctrl+S` | `In N aufteilen` | aufteilen |
| `Esc` | das Kreuz, `Abbrechen` | verlassen; bei ausstehenden Änderungen bestätigt ein zweiter Druck |

Jede Änderung muss genau einem Commit zugeordnet sein; der Editor zählt, was noch
*Nicht zugeordnet* ist, und die Schaltfläche bleibt gesperrt, bis es null ist. Eine
Datei, die binär ist, nur ihren Modus ändert, im Commit neu oder gelöscht ist, ein
Submodul oder eine LFS-Datei ist, lässt sich nur als Ganzes zuordnen. Der erste Commit
beginnt mit der ursprünglichen Nachricht; geben Sie jedem neuen eine Nachricht. Ein
neuer Commit behält den ursprünglichen Autor und das Datum; Committer sind Sie, jetzt.

**In Ihrem Repository ändert sich nur der Branch.** Der letzte neue Commit hat genau den
Inhalt des ursprünglichen Commits, daher werden die Commits darüber mit denselben
Bäumen, Autoren und Nachrichten neu erzeugt (Merges bleiben Merges), und Ihre
gestageten und ungestageten Änderungen bleiben unberührt: Ein unsauberes Arbeitsverzeichnis
ist kein Problem, und ein Konflikt ist ausgeschlossen. Hooks laufen nicht. Solange ein
Auto-Stash aussteht, ist das Aufteilen gesperrt.

**Signieren.** Das Kontrollkästchen *Commits signieren* startet mit dem Wert von
`commit.gpgSign` des Repositorys. Ist es nicht angehakt, entfallen die Signaturen der
umgeschriebenen Commits (der neuen und derer darüber), und der Editor sagt das unter dem
Kontrollkästchen. Ist es angehakt, werden alle signiert; schlägt das Signieren fehl,
ändert sich nichts.

**Rückgängig.** Ein `Ctrl+Z` (das Rückgängig der Symbolleiste) bringt den ursprünglichen
Commit zurück, auch bei unsauberem Arbeitsverzeichnis; siehe
[Rückgängig und Wiederholen](undo-redo.md). Ein bereits gepushter Commit braucht danach
einen Force-Push (mit Lease).

**Aus dem interaktiven Rebase.** Jede Zeile der Rebase-Prüfung hat eine kleine Schaltfläche
**aufteilen...** (`t` auf der Zeile): Sie öffnet denselben Editor für diesen Commit, und
nach dem Aufteilen (oder dem Abbrechen) sind Sie zurück auf dem Prüfbildschirm, mit der
neuen Historie. Wenn Sie den Plan bearbeitet hatten, drücken Sie ein zweites Mal
aufteilen, um zu bestätigen, dass diese Änderungen verloren gehen.

Für Skripte und Agenten: `split-source` und `split-commit` in der
[CLI-Referenz](cli-reference.md); ein Agent kann diesen Editor auch vorausgefüllt zur
Prüfung für Sie öffnen (`ui propose split`, siehe [MCP und Agenten](mcp-and-agents.md)).

## Interaktiver Rebase

Wählen Sie **Interaktiver Rebase ab hier...** auf einem Commit, der ein Vorfahre des
ausgecheckten Branches ist, oder ziehen Sie einen Branch auf einen anderen und
wählen Sie *Interaktiver Rebase*. Das Log wird durch einen Prüfbildschirm ersetzt:
eine Zeile pro Commit, der neueste oben, jede mit einem Ziehgriff und einer
Aktions-Dropdownliste.

![Der Bildschirm des interaktiven Rebase: vier Commits, der zweite auf Squash und der dritte auf Drop gestellt, mit den Schaltflächen Zurücksetzen, Rebase abbrechen und Rebase starten](../images/rebase.png)

| Aktion | Taste | Wirkung |
|---|---|---|
| Pick | `p` | Den Commit behalten |
| Reword | `r` | Ihn mit neuer Nachricht behalten |
| Squash | `s` | In den Commit darunter einarbeiten, die Nachrichten kombinieren |
| Fixup | `f` | In den Commit darunter einarbeiten, seine Nachricht verwerfen |
| Drop | `d` | Ihn weglassen |
| Edit | `e` | Dort anhalten, damit Sie ihn ändern können |

Ziehen Sie eine Zeile (oder `Ctrl+Up` / `Ctrl+Down`), um umzuordnen. `a` zeigt eine
Vorschau des Autosquash (`squash!`- und `fixup!`-Commits rücken neben ihre Ziele).
Nichts geschieht, bis Sie **Rebase starten** (`Ctrl+S`) drücken; **Zurücksetzen**
stellt den Plan wieder her und **Rebase abbrechen** (`Esc`) verlässt ihn, ohne etwas
zu ändern. Hält ein Schritt bei einem Konflikt oder einem `edit` an, bietet ein
Banner über dem Log **Fortsetzen**, **Überspringen** und **Abbrechen** an. Ein
Bereich, der einen Merge-Commit enthält, lässt sich nicht interaktiv rebasen; der
Menüeintrag nennt den Grund.

## Mehrere Commits cherry-picken

`Ctrl+Klick` schaltet einen Commit um, `Shift+Klick` (oder `Alt+Klick`) wählt einen Bereich, `Space`
schaltet den aktuellen um, `Esc` löscht. Rechtsklick auf einen ausgewählten Commit
für **N Commits per Cherry-Pick auf `<Branch>`...**: Derselbe Prüfbildschirm öffnet
sich, mit denselben Aktionen und Umordnungen, und **Cherry-Pick starten** spielt sie
der Reihe nach ab.

## Commits als Mermaid gitGraph kopieren

![Das Popup „Als Mermaid gitGraph kopieren“ über drei ausgewählten Commits: Auswahl von Umfang, Detail und Richtung, die Option für den Codeblock und die Live-Vorschau](../images/log-mermaid.png)

Wählen Sie im Log Commits aus (`Ctrl+Klick`, `Shift+Klick`, `Space`; oder lassen Sie den Cursor einfach auf einem
stehen) und öffnen Sie **Als Mermaid gitGraph kopieren...**: `Shift+Y` im Log, das Untermenü **Kopieren** des
Commit-Menüs oder in der Palette `Log: Als Mermaid gitGraph kopieren...`. Ein Popup zeigt das erzeugte
[Mermaid](https://mermaid.js.org/syntax/gitgraph.html)-Skript, live und scrollbar; `Enter`
(oder die Schaltfläche **Kopieren**) legt genau diesen Text in die Zwischenablage. Fügen Sie ihn in eine README, die
Beschreibung eines Pull Requests oder [mermaid.live](https://mermaid.live) ein.

Das Popup hat drei Comboboxen, ein Kästchen und die Vorschau (`Tab` wechselt zwischen ihnen; jede
Combobox öffnet sich mit `Enter`, `Space` oder `Down`):

- **Commits**: die Auswahl oder - wenn ein Commit ausgewählt ist und HEAD von ihm abstammt - **Von der
  Auswahl bis HEAD**, die Folge der Commits dazwischen (beantwortet aus dem geladenen Teil des Logs).
- **Detail**: `structure` (Commits, Branches und Merges; die IDs sind kurze SHAs), `refs` (fügt die echten
  Branch-Namen als Mermaid-Branches hinzu, Tags als `tag:` und den HEAD-Commit als `type: HIGHLIGHT`),
  `subjects` (die Commit-ID wird zu `<sha> <Betreff>`, maskiert und auf `ui.mermaid_subject_width`
  Zeichen gekürzt) und `full` (fügt der ID den Autor und das UTC-Datum hinzu). In der Oberfläche heißen die Stufen
  **Struktur**, **Refs**, **Betreffzeilen** und **Vollständig**.
- **Richtung**: von links nach rechts (`gitGraph LR:`) oder von oben nach unten (`gitGraph TB:`).
- **Den ```mermaid-Block einschließen** umschließt das Skript mit einem Markdown-Codeblock.

Detail, Richtung und Codeblock werden gemerkt (`ui.mermaid_detail`, `ui.mermaid_orientation`,
`ui.mermaid_fence`). Derselbe Generator läuft auf der Kommandozeile:
`gitpoulpi log --format mermaid [--detail structure|refs|subjects|full] [--orientation lr|tb] [--fence]
[--range A..B | --limit N]` (siehe die [CLI-Referenz](cli-reference.md)); der MCP-Server stellt ihn als
Argument `format` des Tools `log` bereit.

Mermaids `gitGraph` ist eine Wiedergabesprache (commit, branch, checkout, merge), keine Graphbeschreibung,
daher trifft der Generator ein paar Entscheidungen, die man kennen sollte:

- Ein Commit setzt den Branch seines ersten ausgewählten Elternteils fort; die anderen Kinder eröffnen einen neuen
  Branch, nach einem echten Branch darauf benannt (`b1`, `b2`... bei `structure`). Der erste Branch ist `main`
  (oder der echte Name, über einen `mainBranchName`-Header) und folgt der First-Parent-Kette von HEAD.
- Ein Merge verbindet sich mit dem Branch des anderen Elternteils. Ist dieser Branch nach dem Elternteil
  weitergelaufen, erzeugt der Generator am Elternteil einen kleinen Zeiger-Branch (`<Branch>-<sha>`) und mergt
  diesen, damit der Merge weiter auf den richtigen Commit zeigt.
- Ein Elternteil außerhalb der Auswahl entfällt: Ein Commit ohne ausgewählten Elternteil beginnt eine eigene
  Linie, Lücken in einer Auswahl erscheinen also als getrennte Linien.
- Ein Octopus-Merge (drei oder mehr Elternteile) wird zu aufeinanderfolgenden Merges; die mittleren tragen das
  Suffix ` (octopus 1/2)`, der letzte trägt den echten Commit.
- Cherry-Picks werden nicht erkannt; doppelte Anführungszeichen in Betreffzeilen werden zu Apostrophen (Mermaid
  kann sie nicht maskieren).

## Commits vergleichen

Wählen Sie zwei oder mehr Zeilen des Logs aus: `Ctrl+Klick` schaltet eine Zeile um, `Shift+Klick` (oder `Alt+Klick`, `Shift+Up/Down`, `Alt+Up/Down`) wählt einen Bereich, `Space` schaltet die aktuelle Zeile um. Commits, die **WIP**-Zeile und **Stash**-Zeilen lassen sich alle auswählen. Der rechte Bereich wird zu einem **Vergleich**-Bereich:

- Die Kopfzeile hat eine Schaltfläche **Tauschen**, und darunter ist jede ausgewählte Zeile in Log-Reihenfolge (neueste zuerst) als zweizeilige **Karte** aufgelistet: der Avatar des Autors (nach den Avatar-Einstellungen des Logs: ein Bild auf Terminals mit Bildunterstützung, sonst Initialen), Name, wie lange es her ist, kurze ID und Zusammenfassung. Die **WIP**-Zeile ist „Arbeitsverzeichnis / Nicht committete Änderungen“, ein Stash zeigt das Stash-Symbol, `stash@{n}` und seine Nachricht. Das älteste Ende trägt ein Etikett **Von** und das neueste ein Etikett **Bis** (Tauschen vertauscht sie); bei drei oder mehr Zeilen weist ein Hinweis darauf hin, dass die Änderungen zusammengefasst sind. Die Kartenliste nimmt höchstens ein Drittel des Bereichs ein und scrollt dann (Mausrad, Scrollleiste oder den Teiler darunter ziehen, um die Größe zu ändern);
- Ein Klick auf eine Karte setzt den Log-Cursor auf diese Zeile und lässt sie im Graphen aufblitzen; die Auswahl bleibt unberührt, `Space` nimmt diese Zeile also anschließend aus ihr heraus;
- darunter listet die gewohnte Liste der geänderten Dateien (Pfad- oder Baumansicht, Filter, Sortierung) die Dateien auf, die sich zwischen ihnen unterscheiden. Klicken Sie auf eine Datei oder drücken Sie `Enter`, um ihr Diff in der Mitte zu öffnen, in denselben Layouts (vereint oder nebeneinander), mit Blame, Dateiverlauf und Dateiansicht am neueren Ende.

![Drei ausgewählte Log-Zeilen: Der Bereich Vergleich listet sie als Karten mit Avataren und Etiketten Von / Bis auf, über den geänderten Dateien](../images/compare-cards.png)

Was verglichen wird, hängt von der Auswahl ab:

| Auswahl | Verglichen |
|---|---|
| zwei Commits (oder ein Stash und ein Commit) | der ältere mit dem neueren, in Log-Reihenfolge, egal in welcher Reihenfolge geklickt wurde |
| die WIP-Zeile und ein Commit oder Stash | dieser Commit mit dem Arbeitsverzeichnis: gestagte und ungestagte Änderungen der versionierten Dateien (unversionierte Dateien sind nicht Teil eines Diffs) |
| drei oder mehr Commits | ihre zusammengefassten Änderungen: von kurz vor dem ältesten bis zum neuesten (Merges folgen ihrem ersten Elternteil) |

Eine umbenannte oder verschobene Datei erscheint als eine Datei, mit dem alten Namen und der Ähnlichkeit über ihrem Diff. **Tauschen** (`x`) vertauscht die Enden eines Vergleichs aus zwei Zeilen; mit der WIP-Zeile, die immer das neuere Ende ist, oder bei einem Bereich aus drei oder mehr Zeilen gibt es nichts zu tauschen. Ein Vergleich mit der WIP-Zeile folgt dem Arbeitsverzeichnis, während Sie Dateien speichern.

Tastatur: `v` (oder **Log: Auswahl vergleichen** in der Befehlspalette) setzt den Fokus in die Dateiliste des Vergleichs; `x` (oder **Log: Vergleichsenden tauschen**) tauscht; `n` (oder **Log: Zur nächsten verglichenen Zeile**) ist das Tastatur-Gegenstück zum Klick auf eine Karte: Der Log-Cursor springt zur nächsten ausgewählten Zeile, am Ende wieder von vorn. `Esc` oder ein einfacher Klick auf eine Zeile führt zurück zum einzelnen Commit. Der Eintrag **Cherry-Pick** zählt nur die Commits der Auswahl; die WIP-Zeile und Stashes werden dort ignoriert.

Dieselben Listen stehen Skripten und Agenten zur Verfügung: `gitpoulpi diff --from <rev> [--to <rev|worktree>] [--name-only]` (die CLI und das MCP-Tool `diff`; `--to` ist standardmäßig das Arbeitsverzeichnis).

## Konflikte und andere Banner

Hält ein Merge, Rebase, Cherry-Pick oder Revert bei einem Konflikt an, erscheint ein
Banner über dem Log, und die Dateien im Konflikt tragen im Status-Bereich ein rotes
`!`. Lösen Sie sie im Merge-Editor und nutzen Sie dann **Fortsetzen** oder **Merge per
Commit abschließen** des Banners, oder **Abbrechen**, um auszusteigen. Siehe
[Merge-Konflikte lösen](merge-conflicts.md).

Weitere Banner über dem Log:

| Banner | Schaltflächen | Palette |
|---|---|---|
| *Rebase von `<Branch>` auf `<Commit>` · Schritt/Gesamt · angehalten bei ...* | **Fortsetzen**, **Überspringen**, **Abbrechen** | *Rebase: Fortsetzen / Überspringen / Abbrechen* |
| *Patches werden angewendet (git am) · Patch n von m* (ein angehaltenes `git am`) | **Konflikte lösen...** (wenn ein Patch Konflikte hat), **Fortsetzen**, **Überspringen**, **Abbrechen** (fragt vorher) | *git am: Fortsetzen / Diesen Patch überspringen / Abbrechen* |
| *Eine übrig gebliebene index.lock (n s alt) blockiert jede Änderung an diesem Repository* (ein Git-Befehl starb und ließ `.git/index.lock` zurück) | **index.lock entfernen** (fragt vorher; nur angeboten, wenn die Datei mindestens 10 Sekunden alt ist) | *Repository: Veraltete index.lock entfernen* |
| *Auto-Stash ausstehend: ...* | Siehe [Auto-Stash](#auto-stash) | *Banner: ...* |

## Blame

Blame zeigt die ganze Datei mit Zeilennummern und Syntaxfarben und rechts vom Code eine Annotationsspalte (`ui.blame_side = "left"` setzt sie nach links). Öffnen Sie es über die Leiste einer Datei (`Blame`), das Dateimenü, die Palette oder aus einer Verlaufszeile (`b`). Aus dem Arbeitsverzeichnis werden noch nicht committete Zeilen gesondert dargestellt (siehe Streifen).

- **Hunks.** Eine Folge aufeinanderfolgender Zeilen, die zuletzt derselbe Commit geändert hat, ist ein Hunk. Seine erste Zeile zeigt den Avatar des Autors (ein Bild, wo das Terminal Bilder zeichnet, sonst das Initialen-Abzeichen, nichts bei ausgeschaltetem `ui.avatars`), den Autor, das Alter und die Zusammenfassung des Commits; die übrigen Zeilen zeigen nur den Farbbalken des Autors. Eine feine Linie trennt Hunks. Beginnt ein Hunk oberhalb des Bildschirms, wird seine Annotation in der ersten sichtbaren Zeile wiederholt.
- **Streifen.** Neben der Annotation: der Farbbalken des Autors und ein **Alters-Hitzestreifen**: warm für junge, kalt für alte Zeilen, bezogen auf die älteste und die neueste Zeile der Datei (die Legende steht im Titel). Noch nicht committete Zeilen sind gepunktet und sagen „Noch nicht committet“.
- **Karte.** Lassen Sie den Mauszeiger kurz auf einer Annotation ruhen oder drücken Sie `i`, dann erscheint die Detailkarte des Hunks: vollständige Commit-ID, Autor und E-Mail, Datum, Zusammenfassung und ihre Aktionen. Der gewählte Hunk ist getönt.

  ![Blame mit eingeschalteter Minimap und der Karte am gewählten Hunk](../images/blame-card.png)
- **Aktionen** (Kartenzeilen, Rechtsklickmenü auf einem Hunk, Tasten und `Blame: ...` in der Palette): den Commit im Log zeigen (`Enter`), seine ID kopieren (`Shift+Y`), den Datei-Diff des Commits zeigen (`d`), Blame beim Eltern-Commit (`b`, folgt Umbenennungen), den Dateiverlauf öffnen (`Shift+H`).
- **Bewegen.** `j` / `k`, PageUp/PageDown, Home, `Shift+G` und das Mausrad wie überall; `n` / `p` nächster / vorheriger Hunk, `Shift+N` / `Shift+P` nächster / vorheriger Hunk desselben Commits, `g` zu einer Zeilennummer springen.
- **Hohe Hunks.** Unter der Zusammenfassung zeigt ein Hunk ab drei Zeilen die kurze Commit-ID und das volle Datum (gedämpft), damit die Spalte nicht leer bleibt.
- **Minimap.** Bei eingeschaltetem `ui.minimap` (die Schaltfläche `Map`, `M`) sitzt die Minimap der Datei neben dem Code, gefärbt mit derselben Alters-Hitze wie der Streifen; klicken oder ziehen Sie entlang, um zu scrollen.
- **Breite.** Ziehen Sie den Rand der Annotationsspalte oder drücken Sie `>` / `<`; die Breite wird als `ui.blame_width` gespeichert.

## Dateiverlauf

Öffnen Sie ihn über das Segment `Datei` der Auswahl `Datei │ Alle` in der Diff-Leiste, ein Dateimenü oder die Palette (`Dateiverlauf: ...`). Die Mitte teilt sich in zwei Hälften: links die Commits der Datei, neueste zuerst, mit Umbenennungen; rechts der Diff der Datei für den gewählten Commit, mit allen Modi der Diff-Ansicht (Hunks / ganze Datei, inline / nebeneinander, Wort-Diff, Minimap, Umbruch).

- **Die Liste.** Zwei Zeilen pro Commit. Die erste: der Änderungsbuchstabe (A hinzugefügt, M geändert, D gelöscht, R umbenannt), die Zusammenfassung und das farbige `+hinzugefügt -entfernt`. Die zweite, gedämpft: der Avatar des Autors (genau das, was die Autorenspalte des Logs je nach *Autoren-Avataren* zeigt), der Autor, das relative Datum, die kurze Commit-ID und `alt -> neu` beim Commit, der die Datei umbenannt hat. Eine schmale Liste lässt zuerst die Umbenennung weg, dann die ID, dann das Ende des Autornamens, nie aber Autor oder Datum. Seiten mit je 100 Commits werden nachgeladen, wenn Sie sich dem Ende nähern; währenddessen steht eine Zeile `weitere werden geladen...` da.
- **Bewegen.** `j` / `k`, Pfeile, PageUp/PageDown, Home / End, das Mausrad und ein Klick; der Diff folgt sofort (Diffs werden im Hintergrund geholt, pro Zeile zwischengespeichert, und eine gehaltene Taste fordert nur einen an). Eine Zeile vor einer Umbenennung wird unter dem damaligen Dateinamen verglichen.
- **Diff / Datei / Blame.** Die Schaltflächen **Dateiansicht**, **Diff** und **Blame** der Ansichtsleiste wählen, was die rechte Seite für den gewählten Commit zeigt: seine **Diff-Ansicht**, die **Dateiansicht** (die Datei, wie sie bei diesem Commit war) oder sein **Blame**, alles innerhalb der Teilung, die Liste bleibt links, und das Segment `Datei` der Spaltenauswahl bleibt hervorgehoben. Der gewählte Modus bleibt erhalten, wenn Sie die Zeile wechseln; die rechte Seite folgt der Auswahl. Tasten `Alt+1` / `Alt+2` / `Alt+3` (in der Liste auch `d` / `o` / `b`), aus beiden Hälften, und die `Dateiverlauf: ...`-Zeilen der Palette tun dasselbe; `Esc` führt zum Diff zurück.
- **Die Spalte.** Die Liste ist die *Commit-Spalte* mit ihrer Liste **Datei**: Aus- und Einblenden, Größen und die Auswahl `Datei │ Alle` sind an einer Stelle beschrieben, unter [Die Commit-Spalte](interface-tour.md#die-commit-spalte). Ein Klick auf `Alle` verlässt den Dateiverlauf und lässt die Ansicht beim gewählten Commit.

  ![Dateiverlauf mit einem mittleren Commit gewählt: sein Diff rechts](../images/file-history.png)

  ![Dateiverlauf mit der rechten Seite im Blame-Modus, dem Blame der Datei beim gewählten Commit](../images/file-history-blame.png)
- **Zwei Commits vergleichen.** `Shift+Up` / `Shift+Down` oder `Shift+Klick` wählen einen Bereich; die rechte Seite zeigt den Diff der Datei zwischen den beiden Enden, jedes mit seinem eigenen Pfad. `Esc` verwirft den Bereich.
- **Fokus.** `Tab` (oder ein Klick) gibt die Tastatur an den Diff und zurück; der Diff behält seine eigenen Tasten.
- **Aktionen** (Rechtsklickmenü, Tasten und `Dateiverlauf: ...` in der Palette): im Log zeigen (`Enter`, Doppelklick; das Log lädt so weit zurück wie nötig), die ID (`y`) oder die Zusammenfassung (`Shift+Y`) kopieren, den Diff zeigen (`d`), die Datei bei diesem Commit (`o`) oder ihren Blame (`b`) rechts zeigen, mit dem Arbeitsverzeichnis vergleichen (`c`, nochmals zum Verlassen), die Datei auf diese Version wiederherstellen (`Shift+R`, mit Rückfrage).
- **Rückgängig.** Das Wiederherstellen einer Version ist ein protokollierter Schritt: Rückgängig (`Ctrl+Z`) bringt die alten Bytes der Datei zurück.
- **Teiler.** Ziehen Sie den Rand zwischen den Hälften oder nutzen Sie `Alt+Right` / `Alt+Left`, `Alt+0` zum Zurücksetzen; die Breite wird pro Sitzung gemerkt.

## Fetch, Pull und Push

- **Pull** (`Ctrl+G p`) pullt so, wie es *Einstellungen > Git & Kodierung >
  Pull-Schaltfläche* festlegt (`git.pull_mode`: Merge, nur Fast-Forward, Rebase;
  eine Einstellung pro Repository). Sein `▾` führt *Fetch All*, *Pull (Fast-Forward,
  wenn möglich)*, *Pull (nur Fast-Forward)* oder *Pull (Rebase)* einmal aus, ohne die
  Schaltfläche zu ändern. Der Modus wird git immer ausdrücklich übergeben, sodass
  das `pull.rebase` Ihrer Git-Konfiguration Sie nicht überraschen kann.
- **Push** (`Ctrl+G Shift+P`) sendet den ausgecheckten Branch an das Standard-Remote
  und setzt beim ersten Mal den Upstream. Sein `▾` pusht einmal zu einem anderen
  Remote Ihrer Wahl. Hat das Remote Commits, die Sie nicht haben, bietet ein Dialog
  **Pull (Merge), dann Push**, **Force-Push (mit Lease)...** (nie ein bloßes
  `--force`) oder **Abbrechen** an.
- `Ctrl+G f` fetcht jedes Remote. Unter *Einstellungen > Synchronisierung* entscheidet
  *Auto-Prune*, ob Fetches bereinigen, und *Intervall für Auto-Fetch (Minuten)* fetcht im
  Hintergrund.

All das gibt es auch als `gitpoulpi fetch`, `pull` und `push` in der
[CLI](cli-reference.md).

## Auto-Stash

Blockieren nicht committete Änderungen einen Checkout, Pull, Merge oder Rebase, kann
GitPoulpi sie für Sie beiseitelegen: Es stasht sie, führt den Vorgang aus und poppt
dann den Stash. *Einstellungen > Synchronisierung > Auto-Stash vor Checkout, Pull,
Merge oder Rebase* (`general.auto_stash`):

- **fragen** (Standard): Ein Dialog bietet *Änderungen stashen und fortfahren*, *Immer automatisch stashen* (dasselbe, und es setzt `general.auto_stash = "always"`, damit Sie nicht mehr gefragt werden; Einstellungen > Synchronisierung schaltet es zurück) oder *Abbrechen* an.
  Die CLI und MCP scheitern stattdessen und nennen `--autostash`.
- **immer**: ohne Nachfrage erledigt. **nie**: Der Vorgang wird abgelehnt.

*Unversionierte Dateien in den Auto-Stash aufnehmen* nimmt auch nicht verfolgte
Dateien mit. Bei einem sauberen Arbeitsverzeichnis entsteht nie ein Stash. Der Stash, der Vorgang
und der Pop lassen sich jeweils rückgängig machen.

Lässt sich der Stash nicht zurückspielen (er kollidiert mit dem Ergebnis, oder ein
Merge oder Rebase hielt bei einem eigenen Konflikt an), bleibt er in der Stash-Liste.
Ein **Banner über dem Log** sagt *Auto-Stash ausstehend: Ihre Änderungen von vor dem
Checkout liegen in stash@{n}*. Seine Schaltflächen:

| Schaltfläche | Wirkung |
|---|---|
| **Konflikte lösen...** | Öffnet den Merge-Editor (nur nach einem Pop mit Konflikten) |
| **Ergebnis behalten** | Behält das Arbeitsverzeichnis, wie es ist, und verwirft die Stash-Kopie (nur nach einem Pop mit Konflikten, sobald keine Datei mehr im Konflikt ist) |
| **Jetzt poppen** (**Erneut poppen** nach einem fehlgeschlagenen Pop) | Spielt den Stash zurück |
| **Stash zeigen** | Zeigt die Änderungen des Stashs |
| **Neu beginnen** | Siehe unten |
| **Drop** | Verwirft den Stash (fragt vorher) |

Jede Schaltfläche hat einen Palettenbefehl (*Banner: Ausstehenden Auto-Stash poppen*, *Banner:
Neu beginnen...*, ...). Nichts wird für Sie wiederholt oder verworfen.

**Neu beginnen** fragt zuerst nach. Dann:

1. bewahrt es eine Kopie des aktuellen Zustands der gestashten Dateien unter
   `refs/gitpoulpi-backup/autostash-start-over-<Zeit>` auf;
2. verwirft es die Änderungen nur an diesen Dateien (es lehnt ab und nennt sie, wenn
   andere Dateien lokale Änderungen haben, die verloren gehen könnten);
3. macht es den Vorgang rückgängig, der den Auto-Stash erstellt hat, über das
   Rückgängig-Journal (dieser Vorgang muss also der letzte sein);
4. poppt es den Stash zurück auf den ursprünglichen Commit und räumt das Banner ab.

Schlägt ein Schritt fehl, hält es mit intaktem Stash an und sagt, wo. Solange das
Banner angezeigt wird und der letzte Vorgang der ist, der den Auto-Stash erstellt
hat, tut **Rückgängig** (`Ctrl+Z`, `gitpoulpi undo`) dasselbe, nach einer Bestätigung.
Skripte nutzen `gitpoulpi autostash start-over [--dry-run] [--yes]` (Exit-Code 3 ohne
`--yes`).

**Solange ein Auto-Stash aussteht, werden Vorgänge, die den Verlauf verschieben,
abgelehnt:** Checkout eines Branches oder Commits, Merge, Rebase (auch interaktiv),
Pull, Fast-Forward, Cherry-Pick, Revert, Reword, KI-Commit-Zusammenstellung und
Wiederholen. Das Fenster graut diese Schaltflächen und Paletteneinträge aus und nennt
den Grund. Die CLI und MCP erhalten einen Fehler `PendingAutoStash` (Exit-Code 1).
Alles andere bleibt erlaubt: Lesen, Staging, Verwerfen, Commit, Branches, Tags,
Remotes, Fetch, Push, Rebase fortsetzen / überspringen / abbrechen, Konfliktlösung,
`stash push`, `reset` und `stash pop` / `apply` / `drop`. Rückgängig ist ebenfalls
erlaubt.

Wartet der Auto-Stash auf einen Merge oder Rebase, der bei Konflikten angehalten hat,
wird er von selbst gepoppt, sobald dieser Vorgang endet (in GitPoulpi oder mit reinem
git gefolgt von einer Aktualisierung).
