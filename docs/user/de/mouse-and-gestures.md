[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · **Deutsch** · [Español](../es/mouse-and-gestures.md) · [Français](../fr/mouse-and-gestures.md) · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · [Polski](../pl/mouse-and-gestures.md) · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Maus und Gesten

Alles, was Sie mit der Maus tun können, hat auch ein Tastatur-Gegenstück. Siehe
[Tastenkürzel](keybindings.md). Die Namen der Bereiche stehen im
[Rundgang durch die Oberfläche](interface-tour.md).

## Allgemeine Regeln

- **Klicken** Sie auf eine Zeile, um sie auszuwählen. Das funktioniert in jedem
  Bereich, jeder Liste, jedem Menü und Dialog.
- **Doppelklicken** Sie auf eine Zeile, um sie zu öffnen oder auszuführen. Ein
  Doppelklick auf einen lokalen Branch, einen Tag oder eine Ref-Pille checkt ihn aus.
  Ein Doppelklick auf eine Repository-Zeile öffnet das Repository.
- **Rechtsklick** auf eine Zeile öffnet ihr Menü. `.`, `Shift+F10` oder die Menütaste
  öffnet dasselbe Menü für die ausgewählte Zeile.
- **Das Mausrad** scrollt den Bereich unter dem Zeiger. In einer Liste bewegt es nur
  die Ansicht. Die Auswahl bleibt, wo sie ist, auch wenn sie aus dem Bild scrollt. Nur
  ein Klick oder eine Taste bewegt die Auswahl.
- **Shift+Mausrad** scrollt seitwärts, im Diff, in der Dateiansicht und im Editor,
  wenn der Zeilenumbruch aus ist.
- **Fahren Sie über** eine Schaltfläche, eine Zeile oder den Schieber einer
  Bildlaufleiste, um sie aufleuchten zu lassen. Viele Terminals können den Mauszeiger
  nicht ändern, daher ist dieses Leuchten oft das einzige Zeichen, dass etwas
  anklickbar ist. Lassen Sie den Zeiger auf einer Schaltfläche oder einer Log-Zelle
  ruhen, um einen Tooltip zu sehen.
- **Klicken Sie außerhalb** eines Textfeldes, um es zu verlassen. Was Sie getippt
  haben, bleibt erhalten. Der Refs-Filter übernimmt, was Sie getippt haben, wie `Enter`.
  Die Log-Suchleiste schließt sich, wie bei `Esc`.

## Fenster und Dialoge schließen

Jedes Fenster mit Rahmen und jeder Dialog hat oben rechts im Rahmen ein `✕`, und überall
gelten dieselben Regeln:

- **Klicken Sie auf `✕`**, um es zu schließen. `Esc` tut dasselbe und schließt zuerst das
  Innerste (ein offenes Dropdown, ein Textfeld, eine Bestätigung), dann das Fenster.
- **Klicken Sie außerhalb** eines Dialogs, eines Hilfe- oder Infopanels oder eines Popups,
  um es zu schließen. Ein Klick außerhalb schließt nie eine Vollansicht (Einstellungen,
  Diff-Ansicht, Dateiansicht, Editoren, Review, Board): Sie haben kein Außerhalb.
- Ein Klick **innerhalb** eines Panels schließt es nie.
- Ein Formular mit von Ihnen getipptem Text (ein neues Issue, ein Pull oder Merge
  Request) ignoriert einen Klick außerhalb, damit ein versehentlicher Klick nichts
  verliert. `✕` und `Esc` fragen zuerst **Änderungen verwerfen?**; **Weiter bearbeiten**
  geht zurück. Ein Formular ohne Eingabe schließt sofort, ebenso ein einzeiliger Dialog.
- Die Befehlspalette, die Auswahllisten (Tab, Theme, Sprache), die kleinen Dialoge von
  Review, Board, Merge und Composer sowie die Bestätigungsabfrage haben ebenfalls ein `✕`.
  Kontextmenüs und Dropdownlisten haben keins: Sie schließen mit `Esc` oder einem Klick
  außerhalb.

## Listen in den Einstellungen

Die Listen der Einstellungen (Profile, KI-Profile, Workflow, Öffnen in, Bilder) lassen
sich so mit der Maus bedienen. **Klicken Sie auf einen Untertab** (Allgemein oder die Liste), um zu ihm zu wechseln; `Alt+Left` / `Alt+Right` tut dasselbe.

- **Klicken** Sie auf eine Zeile, um sie auszuwählen, **doppelklicken** Sie, um zu ihren
  Feldern zu wechseln, und nutzen Sie das **Mausrad**, um ohne Auswahl zu scrollen.
- **Drücken Sie auf eine Zeile und ziehen** Sie sie nach oben oder unten, um sie
  umzusortieren: Ein Balken zeigt, wo sie landet, Loslassen legt sie ab, `Esc` bricht
  ab. Das Tastatur-Gegenstück ist `Alt+Up` / `Alt+Down`.
- **Klicken Sie auf `+ Hinzufügen`, `Duplizieren`, `Entfernen`, `↑`, `↓`** unter der
  Liste. Tasten: `Insert`, `Ctrl+D`, `Delete`, `Alt+Up` / `Alt+Down`;
  siehe [Einstellungen](settings.md#listen).

## Listen in Popups

Diese Regeln gelten für jede Liste in einem Popup mit Filterfeld: die Befehlspalette,
ihre Auswahllisten (Design, Sprache, Graph-Farben), die Tab-Auswahl, durchsuchbare
Dropdownlisten, die Liste der Issue-Referenzen und die Hilfe.

- **Klicken** Sie auf eine Zeile, um sie auszuwählen.
- **Doppelklicken** Sie auf eine Zeile, um sie auszuführen oder zu wählen, wie `Enter`.
- **Das Mausrad** scrollt die Liste. Es bewegt die Auswahl nicht.
- Ist die Liste länger als das Popup, erscheint an ihrem rechten Rand eine
  **Bildlaufleiste**. Ziehen Sie den Schieber oder klicken Sie auf die Bahn.

Eine kurze Dropdownliste ohne Filterfeld wählt die Zeile beim ersten Klick. Ein
Kontextmenü führt die Zeile beim ersten Klick aus. Die Tasten für diese Listen stehen
in [Tastenkürzel](keybindings.md#filterbare-popup-listen).

## Bildlaufleisten

Jeder Bereich, der scrollen kann, zeigt an seinem Rand eine dünne Bildlaufleiste, wenn
sein Inhalt nicht passt: das Log, die Refs-Abschnitte, die Dateilisten, der Diff, die
Dateiansicht, der Editor und Popup-Listen.

- **Ziehen Sie den Schieber**, um zu scrollen.
- **Klicken Sie auf die Bahn**, um den Schieber unter den Zeiger zu holen: Die Ansicht
  springt anteilig dorthin, und solange Sie die Taste gedrückt halten, ziehen Sie weiter.
- **Halten Sie die Taste auf einem Endpfeil**, um weiter zu scrollen. Es beginnt nach
  kurzer Verzögerung. Gehen Sie vom Pfeil weg, um zu pausieren.
- Eine Bildlaufleiste bewegt nur die Ansicht. Sie bewegt nie die Auswahl.

Im Diff und im Editor zeigt die vertikale Bildlaufleiste auch die ganze Datei in
kleinen Markierungen: grün für hinzugefügte Zeilen, rot für entfernte Zeilen, bernsteinfarben
für geänderte Zeilen, `◆` für die Cursor-Zeile und `▌` für jede andere Stelle, an der der
ausgewählte Text vorkommt. Der Merge-Editor markiert jeden Konflikt auf seinen
Bildlaufleisten. Klicken Sie auf eine Markierung, um dorthin zu gehen.

## Fehler und das Aktivitätsprotokoll

- **Klick auf den roten Fehler** in der Statuszeile öffnet das Aktivitätsprotokoll bei seinem Eintrag; **Klick auf Kopieren / die vorgeschlagene Schaltfläche** (Wiederholen, Erst Pull, Stash und wiederholen, Integrationen öffnen) am Eintrag.
- **Klick auf Aktivität** am rechten Ende der Statuszeile öffnet das Aktivitätsprotokoll. Klick auf einen Tab wechselt, Klick auf ein Stufen-Kästchen (oder auf **Alle** / **Keine**) blendet diese Zeilen ein oder aus, Klick auf einen Eintrag des Tabs Git-Vorgänge öffnet seine Details, Klick auf **Protokoll leeren** auf dem Tab Anwendung leert es (es fragt zuerst nach). Lange Zeilen scrollen Sie seitlich mit `Shift` + Mausrad oder der horizontalen Scrollleiste. Siehe [Das Aktivitätsprotokoll](troubleshooting.md#das-aktivitätsprotokoll).
- **Klick auf `[x]`** an der Job-Anzeige rechts in der Statuszeile bricht den laufenden Job ab, **ein Klick auf deren Beschriftung** öffnet das Job-Panel. Die Schaltfläche **Aktivität** unten rechts im Home-Tab (oder `Ctrl+N Shift+G`) öffnet das Aktivitätsprotokoll auf Alle Repositories (sein Feld Bereich, eine Auswahlliste unter den Tabs, wählt ein Repository oder einen Workspace).

## Größe ändern

- **Ziehen Sie einen Teiler** zwischen der Refs-Seitenleiste, dem Log und dem rechten
  Bereich. Ziehen Sie ihn über die Mindestbreite hinaus, um diesen Bereich einzuklappen. Die Tastatur verschiebt sie stufenweise: `Ctrl+N Left` / `Ctrl+N Right` bewegen die Trennlinie zwischen Refs und Log, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` die zwischen Log und rechtem Bereich (die Trennlinie geht in die Richtung, in die der Pfeil zeigt); die Palette bietet sie als `Navigation: Refs-Bereich verbreitern`, `Navigation: Refs-Bereich verschmälern`, `Navigation: Status-Bereich verbreitern` und `Navigation: Status-Bereich verschmälern`, dazu `Navigation: Refs-Bereich ein-/ausklappen` und `Navigation: Status-Bereich ein-/ausklappen`.
- **Ziehen Sie den unteren Rand** eines Refs-Abschnitts, um seine Höhe zu ändern.
- **Ziehen Sie den unteren Rand** des Blocks der Commit-Nachricht im rechten Bereich.
- **Ziehen Sie den Rand zwischen zwei Log-Kopfzellen** (standardmäßig die Lücke, von der Kopfzeile oder von irgendwo in der Tabelle; Einstellungen > Verlauf & Graph > Spaltengröße ändern), um eine Spalte in der Größe zu
  ändern (per Tastatur: `Shift+C`, um eine Spalte zu wählen, dann `]` / `[`).
- **Doppelklicken Sie auf den Rand zwischen zwei Log-Kopfzellen**, um die Spalte links davon an ihren Inhalt anzupassen (per Tastatur: `Shift+C`, um eine Spalte zu wählen, dann `=`; `+` passt alle Spalten an). Die letzte Spalte hat einen Griff auf ihrer eigenen letzten Zelle: Ziehen Sie ihn, um diese Spalte in der Größe zu ändern, wobei die Spalte Nachricht den Unterschied aufnimmt.
- **Das Log seitlich scrollen** (das Spaltenlayout `scroll`; Einstellungen > Verlauf & Graph > Spaltenlayout oder das Kontrollkästchen `Seitlich scrollen` der Auswahl): das horizontale Mausrad oder `Shift`+Rad über der Tabelle, die Leiste am unteren Rand (Griff ziehen, auf die leere Schiene klicken, Endpfeile anklicken) oder `Shift+Links` / `Shift+Rechts`. Im Standardlayout `fit` füllen die Spalten den Bereich, und es gibt nichts zu scrollen.
- **Ziehen Sie eine Log-Kopfzelle**, um die Spalte zu verschieben: Ein Schatten ihres Namens folgt dem Zeiger, die verlassene Kopfzelle wird abgedunkelt, und ein Akzentbalken markiert den Platz, an dem sie landen würde. Die Spalten wandern beim Loslassen; `Esc` bricht ab, und der Zeiger zeigt die offene Hand, solange Sie sie tragen (per Tastatur: `Shift+C`, um eine Spalte zu wählen, dann `<` / `>`).

GitPoulpi merkt sich jede Größe. Siehe
[Größen werden gemerkt](interface-tour.md#größen-werden-gemerkt).

## Titelleiste, Tab-Leiste und Symbolleiste

- **Klicken Sie auf <img src="../../../assets/logo/gitpoulpi.png" alt="GitPoulpi-Symbol" height="16">** in der Titelleiste, um **Über** zu öffnen. Im Info-Fenster
  folgen die Pupillen des Oktopus dem Zeiger; ein Klick auf das Logo dreht es zur nächsten
  Farbe, Rechtsklick darauf zeigt die Farbliste, ein Klick auf einen Link öffnet ihn.
- **Klicken Sie auf Öffnen in ▾** in der Titelleiste für die Liste der Programme. Siehe
  [Öffnen in](open-in.md).
- **Klicken Sie auf die Version** (`v1.0.0` oder `vX.Y.Z verfügbar`) in der Titelleiste, um den Tab Versionshinweise zu öffnen.
- **Klicken Sie auf die Lupe und `Globale Suche`** in der Titelleiste, direkt links neben dem Zahnrad, für die
  [globale Suche](search.md).
- **Klicken Sie auf das Zahnrad** in der Titelleiste, links neben dem ×, für die Einstellungen.
- **Klicken Sie auf ×** am rechten Ende der Titelleiste, um zu beenden.
- **Klicken Sie auf einen Tab**, um zu ihm zu wechseln. **Klicken Sie auf sein `x`** (auf
  dem aktiven Tab und auf dem Tab unter dem Mauszeiger), um ihn zu schließen. **Ziehen Sie
  einen Tab** zur Seite, um seine Reihenfolge zu ändern. **Rechtsklick auf einen Tab** öffnet
  das Tab-Menü.
- **Mittelklick auf einen Tab** schließt ihn, wie sein `x`.
- **Klicken Sie auf `‹` oder `›`** an den Enden der Tabs (sichtbar, wenn nicht alle passen), um die Zeile um einen Tab zu scrollen; halten Sie die Taste gedrückt, um weiterzuscrollen, oder nutzen Sie das Mausrad über der Zeile. Auch das Ziehen eines Tabs auf einen Pfeil scrollt die Zeile. Die Zeile scrollt außerdem von selbst, um den aktuellen Tab zu zeigen, sobald er wechselt.
- **Klicken Sie auf das `+`** hinter dem letzten Tab für ein Menü: Neuer Tab (`Alt+T`) und, solange ihre Tabs geschlossen sind, Radar und Workspaces (`Alt+J`).
- **Klicken Sie auf das Haus-Symbol** für den Home-Tab.
- **Klicken Sie auf den Tab Radar oder Workspaces**, um ihn zu öffnen; sein `x`, ein Mittelklick oder `Ctrl+W` schließt ihn. **Klicken Sie auf den Workspace-Chip** (`Alt+W`), um den Workspace zu wechseln.
- **Klicken Sie auf „Beenden & cd hierher“** (rechts neben den Tabs, wenn Sie GitPoulpi über den Shell-Wrapper `gp` gestartet haben), um zu beenden und Ihre Shell im Ordner des aktiven Tabs zu lassen. Siehe [Dem Ordner des aktiven Tabs folgen](following-the-tab-folder.md).
- **Klicken Sie auf eine Schaltfläche der Symbolleiste**, um sie auszuführen.
  **Klicken Sie auf `▾`** neben Pull, Push oder LFS für ihre Liste. **Klicken Sie auf
  `⋯`** für die Schaltflächen, die nicht gepasst haben.
- **Klicken Sie auf das Profil-Kombinationsfeld** am rechten Ende der Symbolleiste, um
  die Commit-Identität für dieses Repository zu wählen.

## Home-Tab

- **Klicken** Sie auf eine Zeile, um sie auszuwählen. **Doppelklicken** Sie darauf, um das
  Repository zu öffnen.
- **Klicken Sie auf den Stern**, um einen Favoriten hinzuzufügen oder zu entfernen.
- **Klicken Sie auf Öffnen, Wechseln, Schließen oder Entfernen** in einer Zeile oder auf
  **`⋮`** für das Menü der Zeile. **Rechtsklick** auf eine Zeile öffnet dasselbe Menü.
- **Klicken Sie auf einen Abschnittstitel**, um den Abschnitt zu falten.
- **Klicken Sie oben auf Suchen, Klonen, Init, Alle einklappen oder Alle ausklappen**
  und auf einem Abschnittstitel auf **Alle Tabs schließen** oder **Alle entfernen**.

In der Ordnerauswahl **Suchen**: Klicken Sie auf einen Ort oder Ordner, um dorthin zu
gehen, doppelklicken Sie auf einen Ordner, um ihn zu öffnen, klicken Sie auf das
Pfadfeld, um einen Pfad zu tippen. Haken Sie
**Nachbarordner anzeigen** an (`Alt+S`, wird gemerkt), und der linke Baum listet auch die
anderen Ordner jeder geöffneten Ebene: Die Ordner auf dem geöffneten Pfad zeigen `▾`,
die anderen `▸`, und ein Klick auf einen geht dorthin. In **Klonen** und **Init**: Klicken Sie auf ein Feld,
um darin zu tippen, und auf eine Schaltfläche oder ein Kontrollkästchen, um es zu
benutzen.

## Refs-Seitenleiste

- **Klicken** Sie auf eine Zeile, um sie auszuwählen. **Doppelklicken** Sie auf einen
  lokalen Branch oder Tag, um ihn auszuchecken. **Doppelklicken** Sie auf einen Worktree
  oder ein Submodul, um es in einem Tab zu öffnen. Das **+** in der Überschrift von
  WORKTREES erstellt einen Worktree; Rechtsklick auf einen Worktree bietet In neuem Tab
  öffnen, Öffnen in, Merge und Entfernen ([Worktrees](worktrees.md)).
- **Fahren Sie über** eine Zeile, um das Symbol links davon zu zeigen. **Klicken Sie**
  darauf, um die Ref im Log auszublenden oder wieder zu zeigen. Ist **Solo** an, stellt
  dieselbe Stelle die Ref solo. In einer Ordnerzeile wirkt das Symbol auf jede Ref im
  Ordner.
- **Klicken Sie auf Solo beenden** oder ** Alle zeigen ** im Titel der Seitenleiste, um
  jedes Solo oder jede ausgeblendete Ref aufzuheben.
- **Rechtsklick** auf eine Zeile öffnet ihr Menü: Checkout, Merge, Fast-Forward, Rebase,
  Worktree erstellen, umbenennen, löschen, ausblenden, Solo und mehr. Die eigene Zeile
  eines Remotes hat **Fetch**, **Bearbeiten**, **Entfernen** und weitere Remote-Aktionen.
  Ein Stash hat **Pop**, **Anwenden** und **Drop**. Eine ausgegraute Zeile sagt, warum sie
  nicht laufen kann.
- **Klicken Sie auf `+`** im Titel REMOTE, um ein Remote hinzuzufügen.
- **Klicken Sie auf eine Ordnerzeile**, um sie zu falten oder zu entfalten.

### Pull Requests und Issues

| Geste | Aktion |
|---|---|
| Klick auf eine Zeile | Sie auswählen. |
| Doppelklick auf eine Zeile | Sie im Browser öffnen. |
| Rechtsklick auf eine Zeile | Ihr Menü: öffnen, URL oder Nummer kopieren, den Quell-Branch auschecken, einen Branch aus einem Issue erstellen. |
| Klick auf `+` im Abschnittstitel | Das Formular **Pull Request erstellen** oder **Neues Issue** öffnen. |
| Klick auf die Filterzeile (`Offen ▾`) | Wählen, welche Requests oder Issues aufgelistet werden. |
| Klick auf einen Gruppentitel | Die Gruppe falten oder entfalten. |
| Klick auf die Zeile von Anbieter oder Repository (ISSUES) | Die Issue-Quelle wählen. **Keiner (dieses Repository trennen)** entfernt sie. |
| Klick auf die Abfragezeile (ISSUES) | Eine JQL- oder WIQL-Abfrage tippen (Jira, Azure DevOps) oder Trello-Listen wählen. |
| Klick auf den Chip der Symbolleiste (`GH #12`, `GL !53`, ...) | Den Pull Request im Browser öffnen. Rechtsklick darauf für **Im Browser öffnen**, **URL kopieren**, **Aktualisieren** und **Profil wechseln**. |
| Klick auf den blassen Chip `+ PR` | Einen Pull Request für den gepushten Branch starten. |

Siehe [Hosting und Issue-Tracker](hosting-integration.md).

## Das Log

- **Klicken** Sie auf einen Commit, um ihn auszuwählen. Seine Details erscheinen im
  rechten Bereich.
- **Ctrl+Klick** fügt einen Commit der Auswahl hinzu oder entfernt ihn. **Shift+Klick**
  wählt einen Bereich (**Alt+Klick** in Terminals, die Shift+Klick für ihre eigene
  Textauswahl behalten, etwa kitty und Windows Terminal). Rechtsklick auf einen
  ausgewählten Commit, um alle zu cherry-picken. Die Auswahl kann die WIP-Zeile und Stashes
  enthalten; zwei oder mehr Zeilen machen den rechten Bereich zu einem Vergleich, siehe
  [Commits vergleichen](log-and-history.md#commits-vergleichen).
- **Klick auf eine Karte** des Vergleichsbereichs setzt den Log-Cursor auf diese Zeile;
  **Klick auf Tauschen** vertauscht die Enden Von und Bis. Ein einfacher Klick auf eine
  Zeile verlässt den Vergleich.
- **Klicken Sie auf die WIP-Zeile** für die Änderungen. **Klicken Sie auf ihren Text
  `// WIP`**, um dort die Commit-Zusammenfassung zu tippen.
- **Fahren Sie über** eine Zelle für den vollen Text: die Nachricht, Autor und E-Mail, das
  volle Datum, den vollen Hash.
- **Fahren Sie über ein Badge `+N`** für die anderen Refs am Commit. Gehen Sie in das
  Popup, um dort auf eine Ref zu klicken. Doppelklick auf eine Ref im Popup checkt sie
  aus.
- **Doppelklicken Sie auf eine Ref-Pille**, um sie auszuchecken.
- **Rechtsklick auf einen Commit** öffnet das Commit-Menü. Es hat ein Untermenü für jede
  Ref am Commit.
- **Rechtsklick auf eine Ref-Pille** öffnet ein Menü mit den Aktionen der Ref und des
  Commits.
- **Rechtsklick auf die Kopfzeile** blendet Spalten ein oder aus: eine Checkliste (`[x]` / `[ ]`) mit einem ✕ im Titel; klicken Sie auf eine Zeile oder nutzen Sie die Pfeiltasten und `Space` / `Enter`, und schließen Sie sie mit dem ✕, `Esc` oder einem Klick daneben (die Palette: `Ansicht: Log: Anzuzeigende Spalten wählen`). Unter den Kontrollkästchen: `<Spalte> an Inhalt anpassen` (die angeklickte Spalte), `Alle Spalten an Inhalt anpassen` und `Spaltenbreiten zurücksetzen`. Das Tastaturgegenstück zum Rechtsklick: `Shift+C` wählt eine Spalte, dann öffnet `.` (oder `Shift+F10`) dasselbe Menü darauf.
- **Die Spalten des Radars** funktionieren genauso: Ziehen Sie den Trenner nach der Repository-, `#`-, Autor- oder Alterspalte, um sie einzustellen, **doppelklicken Sie darauf**, um die Spalte an ihren breitesten Eintrag anzupassen (`C` wählt eine Spalte, dann `=`; `+` passt jede Spalte an), und **klicken Sie mit rechts auf einen Spaltentitel** (oder `C`, dann `.`) für `<Spalte> an Inhalt anpassen` und `Alle Spalten an Inhalt anpassen`.
- **Klicken Sie auf `↑`, `↓` oder `✕`** in der Suchleiste, um zum vorherigen oder nächsten
  Treffer zu gehen oder die Leiste zu schließen.
- **Klicken Sie auf `#12`, `ABC-12` oder `AB#12`** in einer Commit-Nachricht im rechten
  Bereich, um dieses Issue oder diesen Pull Request im Browser zu öffnen.

### Abschnitte der linken Seitenleiste

**Ordner** (Branches und Tags mit Namen wie `a/b/c`, die Dateien des rechten Bereichs in der
Baumansicht) lassen sich auf einmal auf- und zuklappen: **Rechtsklick auf die Kopfzeile oder den
leeren Teil eines Abschnitts**, der Ordner hat, bietet **Alle aufklappen** / **Alle zuklappen**
(und **Bereiche ein-/ausblenden...**, das zur Checkliste unten führt), und **Rechtsklick auf eine
Ordnerzeile** bietet **Alle Unterordner aufklappen** / **Alle Unterordner zuklappen** (der Ordner und
alles darunter). Tasten, im Refs-Bereich, in den Listen Unstaged und Staged und bei den geänderten
Dateien eines Commits gleichermaßen: `=` klappt alles auf, `-` klappt alles zu, `*` (oder
`Shift+Right`) klappt den Ordner unter dem Cursor und alles darunter auf, `_` (oder `Shift+Left`)
klappt ihn zu. Palette: *Refs: Alle Ordner aufklappen*, *Refs: Alle Ordner zuklappen*, *Refs:
Gewählten Ordner und seine Unterordner aufklappen*, *Refs: Gewählten Ordner und seine Unterordner
zuklappen* und die vier Zwillinge *Dateien:* für die fokussierte Dateiliste. Solange ein Namensfilter
aktiv ist (der Refs-Filter, *Dateien filtern* eines Commits), wird jeder Ordner, der einen Treffer
enthält, offen gezeigt, was Sie auch zugeklappt hatten, und Ihre eigenen Faltungen werden weder
geändert noch gespeichert: Löschen Sie den Filter, und sie kommen zurück. Eine Faltung, die Sie
während des Filters vornehmen, wirkt auf das Gezeigte und wird mit dem Filter vergessen.

**Rechtsklick auf die Kopfzeile eines Abschnitts** (ein Abschnitt ohne Ordner, oder der leere Teil der linken Seitenleiste) öffnet eine
Checkliste ihrer Abschnitte: Lokal, Remote, Worktrees, Stashes, Tags, Pull Requests, Issues und Submodule. Ein Klick
entfernt oder setzt das Häkchen eines Abschnitts; das Menü bleibt offen. Ein ausgeblendeter Abschnitt wird nicht
gezeichnet, und die Tastatur überspringt ihn. Der letzte sichtbare Abschnitt lässt sich nicht abwählen. Die Wahl gilt pro Repository und bleibt über Neustarts erhalten; ein Repository ohne eigene Wahl folgt dem Standard (global, `ui.sidebar_sections_hidden`). *Für alle Repositories verwenden* macht die aktuelle Wahl zum Standard, *Auf Standard zurücksetzen* verwirft die eigene Wahl dieses Repositorys (Palette: *Seitenleiste: Diese Abschnitte für alle Repositories verwenden*). Palette: *Seitenleiste: Abschnitte
ein-/ausblenden...*, je ein Befehl *ein-/ausblenden* pro Abschnitt, *Seitenleiste: Abschnitte auf Standard zurücksetzen*.

### Refs ziehen

Drücken Sie auf einen Branch in der Refs-Seitenleiste oder auf eine Ref-Pille im Log und
ziehen Sie ihn auf einen anderen Branch. Die Statuszeile sagt, was Sie tragen. Ein Ziel, das
das Ablegen annehmen kann, leuchtet auf. Lassen Sie los, um ein Menü zu erhalten. `Esc`
bricht das Ziehen ab.

| Ziehen | Menü |
|---|---|
| Lokaler Branch X auf lokalen Branch Y | **Merge von X in Y**, **Rebase von X auf Y**, **Interaktiver Rebase von X auf Y...**, **Fast-Forward von Y auf X** |
| Lokaler Branch auf einen Remote-Branch | **Push X** zu diesem Remote |
| Remote-Branch auf lokalen Branch Y | **Merge**, **Rebase von Y darauf**, **Fast-Forward von Y darauf** und **Pull**, wenn er der Upstream von Y ist |

Ändert die Aktion einen Branch, der nicht ausgecheckt ist, checkt GitPoulpi ihn zuerst
aus. Lehnt git den Checkout ab, ändert sich nichts. Tags lassen sich nicht ziehen und
nehmen kein Ablegen an.

Tastatur: „Markieren, dann ablegen“ tut dasselbe wie das Ziehen, für jedes Paar von Refs.
Drücken Sie im Refs-Bereich `m` auf einem Branch (Palette: `Refs: Als Ablagequelle markieren`).
Die Ref zeigt ein `◆` und ist unterstrichen, auch auf ihren Pills im Log, und die Statuszeile
sagt, wohin Sie gehen müssen. Wählen Sie eine andere Ref in einem beliebigen Abschnitt und
drücken Sie `Shift+M` (Palette: `Refs: Markierte Ref hier ablegen...`): Es erscheint dasselbe
Menü wie beim Ziehen, und seine Zeilen laufen genau wie nach einem Ziehen. Die Markierung ist
verbraucht, sobald das Menü offen ist; `Esc` oder `m` auf der markierten Ref entfernt sie
früher. Der Palettenbefehl `Refs: Ausgewählte Ref auf dem ausgecheckten Branch ablegen...`
öffnet das Menü für die ausgewählte Ref und den aktuellen Branch.

## Der rechte Bereich

### Änderungen

- **Klicken** Sie auf eine Datei, um ihren Diff zu öffnen. **Klicken Sie erneut auf
  dieselbe Datei**, um zum Log zurückzukehren. Ist die Datei im Editor mit ungespeicherten
  Änderungen offen, fragt GitPoulpi zuerst.
- **Fahren Sie über** eine Zeile, um ihre Schaltfläche **Datei stagen** oder **Datei
  unstagen** zu zeigen. Eine Ordnerzeile hat **Ordner stagen** oder **Ordner unstagen**.
- **Klicken Sie auf Alle Änderungen stagen** oder **Alle Änderungen unstagen** in einem
  Abschnittstitel.
- **Klicken Sie auf einen Abschnittstitel** (`Unstaged Dateien`, `Staged Dateien`), um ihn
  zu falten.
- **Klicken Sie auf Pfad oder Baum**, um eine flache Liste oder Ordner zu wählen.
  **Klicken Sie auf Alle einklappen** oder **Alle ausklappen**, um jeden Ordner zu falten.
- **Klicken Sie auf `✗`** links vom Titel, um alle Änderungen zu verwerfen (fragt
  vorher). **Klicken Sie auf `✦`** rechts, um die Änderungen mit der KI zu erklären.
- **Rechtsklick auf eine Datei** öffnet ihr Menü: stagen, verwerfen, ignorieren, LFS, die
  Datei stashen, Verlauf, Blame, in einem externen Programm öffnen, den Pfad kopieren, einen
  Patch erstellen, bearbeiten, löschen. Eine Datei im Konflikt beginnt mit den Zeilen
  **Mit ... lösen**.
- **Rechtsklick auf eine Ordnerzeile** öffnet das Ordnermenü: stagen, verwerfen,
  ignorieren, stashen, einen Patch erstellen, den Ordner öffnen, darin eine Datei erstellen.
- **Rechtsklick auf den leeren Teil** von Unstaged Dateien für **Datei erstellen**.
- **Ignorieren**, **Als Binärdatei behandeln** und **Als Textdatei behandeln** (Datei- und Ordnermenü) öffnen zuerst eine Liste der `.gitignore`- oder `.gitattributes`-Dateien, in die die Regel gehen kann. Zeile anklicken, dann `Enter` drücken (oder die Pfeiltasten benutzen); `Esc` oder ein Klick daneben bricht ab und schreibt nichts.
- **Ctrl+Klick** auf eine Datei- oder Ordnerzeile fügt sie zur Auswahl hinzu oder entfernt sie, **Shift+Klick** (**Alt+Klick**) nimmt einen Bereich; ebenso in den geänderten Dateien eines Commits und in Alle Dateien ansehen. Ein einfacher Klick beginnt neu. **Rechtsklick auf eine gewählte Zeile**: das Menü aller gewählten Dateien.
- **Rechtsklick auf den leeren Teil** einer Liste bietet auch **Repository-Ordner öffnen**; Unstaged ergänzt **Alle Änderungen stagen** und **Alle Änderungen verwerfen...**, Staged ergänzt **Alle Änderungen unstagen**.
- **Konvertieren** (Menüs von Datei, Ordner und Auswahl) schreibt Kodierung, Zeilenenden oder Einrückung der Dateien auf dem Datenträger neu; ein `Ctrl+Z` macht den Stapel rückgängig.

### Das Commit-Formular

- **Klicken** Sie auf die Zusammenfassung oder die Beschreibung, um darin zu tippen.
- **Klicken Sie auf Vorherigen Commit ändern**, um Amend ein- oder auszuschalten.
- **Klicken Sie auf Änderungen an N Dateien committen**, um zu committen. Ist die
  Schaltfläche ausgegraut, sagt ein Klick, warum (zum Beispiel *Änderungen zum Committen
  stagen*).
- **Klicken Sie auf `✦`** rechts von der Zusammenfassung, um die Nachricht mit der KI zu
  schreiben.
- **Klicken Sie auf `#▾`** am Ende der Zusammenfassung für die Liste der Issues. Klicken
  Sie auf eines, um seine Referenz einzufügen.
- **Klicken Sie auf Commits mit KI zusammenstellen**, um den Commit-Composer zu öffnen.

### Commit-Details

- **Klicken** Sie auf eine geänderte Datei, um ihren Diff zu öffnen. **Klicken Sie erneut**,
  um zum Log zurückzukehren.
- **Ziehen** Sie den unteren Rand der Nachricht, um sie in der Größe zu ändern. Das Mausrad
  über der Nachricht scrollt eine lange Beschreibung.
- **Rechtsklick** auf die Zusammenfassung, die Beschreibung oder den Autorenblock für
  *Zusammenfassung ändern...*, *Beschreibung ändern...* oder *Autor ändern...*; ein
  **Doppelklick** auf einen davon (oder auf das Autordatum) öffnet den vollständigen
  Ändern-Dialog mit dem Cursor darin. Die Tastaturäquivalente sind `Ctrl+G Shift+E` (alle
  Felder) und `Ctrl+G Shift+A` (der Autor); Zusammenfassung und Beschreibung stehen in der Palette.
- **Rechtsklick** auf eine geänderte Datei, eine Ordnerzeile, den leeren Teil der Liste oder eine Auswahl für das Menü (öffnen, Verlauf, Blame, in einem Programm öffnen, Pfade kopieren, ignorieren, als Binär- oder Textdatei behandeln, konvertieren, aus dem Commit wiederherstellen, löschen). **Ctrl+Klick** und **Shift+Klick** wählen mehrere Dateien.

Siehe [Staging und Committen](staging-and-committing.md).

## Der Diff und die Dateiansicht

- **Klicken Sie auf eine Schaltfläche** in der Leiste über dem Diff: Dateiansicht,
  Diff-Ansicht, Blame, die Spaltenauswahl `Datei │ Alle`, Bearbeiten, `↑` `↓`, Inline, Split,
  Hunks, WS, Umbr., Map, Kodierung. Eine eingeschaltete Schaltfläche ist hervorgehoben. Ein Klick
  auf `Datei` oder `Alle` zeigt diese Liste in der Commit-Spalte; ein Klick auf das bereits
  gewählte Segment blendet sie aus (siehe [Die Commit-Spalte](interface-tour.md#die-commit-spalte)).
- **Klicken Sie auf `✕`** im Titel, um die Datei zu schließen.
- **Fahren Sie über einen Hunk**, um **Hunk stagen** oder **Hunk unstagen** und **Hunk
  verwerfen** zu zeigen.
- **Fahren Sie über eine geänderte Zeile**, um ihre eigenen Schaltflächen **+ Stage**, **−
  Unstage** und **✗ Verwerfen** am rechten Ende der Zeile zu zeigen.
- **Rechtsklick auf eine Zeile** öffnet das Hunk-Menü: den Hunk oder die ausgewählten
  Zeilen stagen, unstagen oder verwerfen, Dateiverlauf, Blame, kopieren.
- **Klicken Sie auf die Minimap**, um die Ansicht zu dieser Zeile zu bewegen. Halten Sie die
  Taste gedrückt und bewegen Sie, um zu scrollen.

Jedes Verwerfen fragt vorher. Löscht ein Verwerfen nicht verfolgte Dateien, verschwinden auch die Ordner, die dadurch ganz leer werden (nie einer, der noch eine verfolgte, ignorierte oder versteckte Datei enthält, oder der schon vorher leer war); Rückgängig bringt die Dateien samt ihren Ordnern zurück.

### Blame und Dateiverlauf

- **Blame.** Lassen Sie den Mauszeiger auf einer Annotation ruhen, um die Hunk-Karte zu sehen; ein Klick wählt einen Hunk, ein Rechtsklick öffnet sein Menü. Ziehen Sie den Rand der Annotationsspalte, um sie zu verändern (wird gespeichert). Klicken oder ziehen Sie auf der Minimap zum Scrollen.
- **Dateiverlauf.** Ein Klick wählt einen Commit; `Shift+Klick` wählt einen Bereich zum Vergleichen; ein Doppelklick zeigt den Commit im Log; ein Rechtsklick öffnet das Zeilenmenü; ziehen Sie den Rand zwischen Liste und Diff, um die Breite zu ändern; das ✕ an der Liste blendet sie aus (ein Klick auf den Streifen `›` bringt sie zurück); klicken Sie in der Leiste auf `Dateiansicht` / `Diff` / `Blame`, um umzuschalten, was die rechte Seite zeigt, und auf die Auswahl `Datei │ Alle`, um die Liste der Spalte zu wählen. Das Mausrad scrollt die Hälfte unter dem Zeiger. Jedes hat ein Tastaturäquivalent ([Blame](log-and-history.md#blame), [Dateiverlauf](log-and-history.md#dateiverlauf)).

### Die Commit-Spalte (Liste Alle)

- **Fahren Sie über eine Zeile** für den Tooltip des Logs (die ganze Nachricht, Name und E-Mail des Autors, das volle Datum, je nach dem Teil unter dem Zeiger).
- **Rechtsklick auf eine Zeile** öffnet ihr Menü: im Log zeigen, der Diff der Datei in diesem Commit, die Datei in diesem Commit, ihr Blame, dann das Menü des Logs für den Commit. Die Menütaste tut dasselbe mit der Tastatur.
- **Ziehen Sie eine Grenze in der Kopfzeile** (`ui.column_resize`: standardmäßig die Trennzelle zwischen zwei Teilen, in der Kopfzeile durch ein gedämpftes `│` markiert und als Akzentlinie die Spalte entlang gezeichnet, solange der Zeiger darauf steht, wobei ein Ziehen von jeder Stelle daran die Größe ändert; mit `header` der erste Buchstabe des nächsten Namens, hervorgehoben, solange der Zeiger darauf steht), um den Teil links davon einzustellen: Spuren, Nachricht, Autor, Datum, Refs, Änderungen oder SHA. Die Zelle direkt hinter dem letzten Teil ist die Grenze des letzten Teils.
- **Doppelklick auf eine Grenze der Kopfzeile** passt den Teil links davon an seinen Inhalt an, wie die Grenzen des Logs: die Breite, die seine breiteste Zeile ganz zeigt, nie breiter als die Spalte (per Tastatur: eine Zeile anklicken oder `Ctrl+N`, um der Spalte die Tasten zu geben, mit `Tab` den Teil wählen, dann `=`; `+` passt jeden Teil an; die Palette hat `Spalte an Inhalt anpassen` und `Alle Spalten an Inhalt anpassen`). Die Nachricht behält, was die anderen Teile übrig lassen.
- **Ziehen Sie einen Namen der Kopfzeile** über einen anderen, um den Teil dorthin zu verschieben: derselbe Schatten, dieselbe abgedunkelte Quelle und derselbe Akzentbalken wie in der Kopfzeile des Logs, und der Teil wandert beim Loslassen (`Esc` bricht ab; `<` / `>` der Tastatur, nach `Tab`, tun dasselbe um jeweils eine Stelle).
- **Rechtsklick auf die Kopfzeile**, um zu wählen, welche Teile angezeigt werden (Ref, Graph, Autor, Datum, Änder., SHA; die Nachricht ist immer sichtbar). Die Checkliste hat ein ✕ im Titel. Darunter: `<Teil> an Inhalt anpassen` (der angeklickte Teil) und `Alle Spalten an Inhalt anpassen`. Die Palette hat `Spalten wählen` (das Menü per Tastatur, auf dem Teil, den `Tab` gewählt hat) und einen Umschalter pro Teil.
- **Shift+Mausrad** oder ein horizontales Mausrad scrollt die Zeilen seitlich, wenn sie breiter als die Spalte sind.
- **Ziehen Sie den rechten Rand** der Spalte, um ihre Breite zu ändern. **Klicken Sie auf den Streifen `›`** (oder das ✕ an der Spalte, oder **klicken Sie auf das gewählte Segment** der Auswahl `Datei │ Alle` der Leiste), um sie zurückzuholen oder auszublenden.

### Text markieren und kopieren

Sie können den Text des Diffs, der Dateiansicht, von Blame und Dateiverlauf markieren und
kopieren.

- **Drücken und ziehen**, um Text zu markieren. Die Markierung bleibt beim Scrollen auf
  demselben Text.
- **Doppelklick**, um ein Wort zu markieren.
- **Ein einfacher Klick** markiert keinen Text. Er bewegt den Cursor oder wählt die Zeile
  aus und hebt die Textmarkierung auf.
- **`y` oder `Ctrl+C`** kopiert die Markierung. Ohne Markierung kopieren sie die aktuelle
  Zeile.
- **Rechtsklick** im Diff für **Auswahl kopieren**, **Zeile kopieren** und **Hunk
  kopieren**.

GitPoulpi kopiert nur den Text der Datei. Zeilennummern, die Zeichen `+` und `-`,
Hunk-Kopfzeilen und die Spalten von Blame werden nicht kopiert. Im Split bleibt eine
Markierung auf der Seite, auf der Sie das Ziehen begonnen haben.

Im Diff markiert ein Ziehen auch die Zeilen, die es überquert, wie `v`. `Enter` stagt dann
diese Zeilen. `Esc` hebt zuerst die Textmarkierung auf, dann die Zeilenauswahl und
schließt dann den Diff.

Das Markieren von Text hebt auch jede andere Stelle, an der dieser Text im Diff und im
Editor vorkommt, farbig hervor.

## Textfelder

Das gilt für jedes Textfeld: Filter, die Suchleiste, die Palette, die Zusammenfassung und
die Beschreibung, Dialogfelder und Werte der Einstellungen.

- **Klicken**, um den Cursor dorthin zu setzen.
- **Drücken und ziehen**, um Text zu markieren.
- **Doppelklick**, um ein Wort zu markieren.
- Tippen ersetzt die Markierung. `Backspace` oder `Delete` entfernt sie.

Die Tasten für Textfelder stehen in [Tastenkürzel](keybindings.md).

## Der Editor

- **Klicken**, um den Cursor dorthin zu setzen. **Shift+Klick** erweitert die Markierung.
- **Ziehen**, um zu markieren. Über den oberen oder unteren Rand hinaus ziehen, um zu
  scrollen.
- **Doppelklick**, um ein Wort zu markieren.
- **Das Mausrad** scrollt, ohne den Cursor zu bewegen.
- **Klicken Sie auf Speichern, Rückgängig, Wiederholen oder Suchen** in der Leiste. Im
  Suchmodus gehen **`↑`** und **`↓`** zum vorherigen und nächsten Treffer.
- **Klicken Sie auf `✕`**, um zu schließen. GitPoulpi fragt vorher nach, wenn es
  ungespeicherte Änderungen gibt.

## Bilder

Siehe [Bilder](image-view.md) für die Maus in der Bildansicht.

## Dialoge verschieben und in der Größe ändern

Dialoge lassen sich verschieben, die größeren Fenster auch in der Größe ändern:

- **Die Titelleiste ziehen**, um einen Dialog zu verschieben (das Schließkreuz behält seinen Klick). Die Befehlspalette und die Auswahllisten haben keine eigene Titelleiste: Ziehen Sie den oberen Rand.
- **Den linken, rechten oder unteren Rand** oder eine untere Ecke (den Griff `◢`) ziehen, um die Größe eines Fensters zu ändern. Der Dialog bleibt auf dem Bildschirm und wird nie kleiner als sein Minimum (60 x 16 beim Rückgängig-Verlauf, bei den anderen kleiner). Ein vergrößertes Fenster nutzt den Platz: Textbereiche und Listen wachsen, Text bricht an der neuen Breite um.
- Das Ziehen läuft weiter, wenn der Zeiger den Dialog verlässt, und ein Loslassen außerhalb schließt ihn nie.
- **Die Trennlinien** in der Ansicht Reflog / Verlorene Arbeit des Rückgängig-Verlaufs ziehen; ein Doppelklick auf eine setzt sie zurück.
- Die Tastatur-Gegenstücke sind `Ctrl+N` und dann `m` (Pfeile verschieben, `Shift`+Pfeile ändern die Größe, `0` setzt zurück, `Enter` oder `Esc` beenden) und `Alt`+Pfeile für die Trennlinien; die Palette hat `Dialog: Verschieben/Größe ändern` und `Dialog: Position und Größe zurücksetzen`. Sie wirken auf den obersten Dialog. `Ctrl+N` funktioniert über jeder Oberfläche gleich: Es zeigt ein Badge an den Bedienelementen des vordersten Fensters, Dialogs oder Popups (Felder, Listen, Tabs, Schaltflächen und das Schließkreuz), die Taste danach drückt dieses Bedienelement, `m` startet den Anordnungsmodus und `Esc` bricht ab; die Statuszeile nennt `<Badge>`, `m=anordnen` und `Esc`. Das gilt auch für die Befehlspalette, die Auswahllisten, die Workspace-Suche und die Hilfe, wo `Ctrl+N` nicht mehr in der Liste nach unten geht. `Alt+G` (`palette.arrange`) ist ein weiterer Weg, über jedem Dialog und Popup gültig, um den vordersten ohne das Präfix anzuordnen.
- **Eine Einstellung** (`ui.dialog_geometry`, Einstellungen > Darstellung > Layout) gilt für jeden Dialog: Jeder merkt sich seine eigene Position und Größe für diese Sitzung, auf der Festplatte für den nächsten Start oder gar nicht.

| Dialog | Verschieben | Größe ändern |
|---|---|---|
| Rückgängig-Verlauf / Reflog / Verlorene Arbeit, Aktivitätslog, Aufgabenpanel, Ordnerauswahl | ja | ja |
| Befehlspalette, Auswahllisten für Tab / Theme / Sprache, Workspace-Suche | ja | ja (die Liste wächst mit der Höhe, die Zeilen mit der Breite) |
| Clone (die Repository-Liste), Pull Request und Neues Issue (die Beschreibung), "Was gesendet wird" (der Bereich), Mermaid (die Vorschau), Review-Optionen (die Felder, nur die Breite) | ja | ja |
| Langtext-Editoren in den Einstellungen (Vorlagen, Hooks, KI-Prompts), das KI-Feld im Workspace-Formular | ja | ja (der Textbereich) |
| Verlauf umschreiben: die zwei KI-Hilfsdialoge | ja | ja (die Beschreibung, der Sendebereich) |
| Über, Update-Panel, Sicherheitsprüfung, vorhergesagte Konflikte, "Über dem KI-Limit", Tag löschen, sensible Dateien | ja | nein (die Größe folgt dem Text) |
| Init, Worktree, Sparse Checkout, Flow-Dialog, Issue-Flow-Dialoge, Formular der Commit-Nachricht (Umformulieren), Workspace-Formular, Auswahllisten für Labels / Meilensteine | ja | nein |
| Bestätigungen (jedes "Sind Sie sicher?") | ja | nein |
| Review- und Board-Dialoge: ihre kleinen Dialoge (Kommentar, Abschluss, Auswahl, Spalten) | ja | nein |
| Popups der Tastenkürzel-Seite (Zurücksetzen bestätigen, Tastenerfassung), das Popup der KI-Antwort | ja | nein |
| Einstellungen: die Ergebnisse und die Bestätigung der Repository-Authentifizierung | ja | nein |
| Verlauf umschreiben: Purge-, Identitäts- und Frage-Dialoge | ja | nein |
| Popup zum Verschieben einer Datei im Commit-Composer, Merge-Hinweise | ja | nein |

Nicht verschiebbar: die Vollseiten (Einstellungen, Hilfe, die Seiten Review und Board, die schon das Fenster füllen), Kontextmenüs und Dropdownlisten (sie hängen an dem, was sie geöffnet hat), Tooltips und die kleinen Popups innerhalb einer Seite (das Formular der Remotes, der Editor zum Aufteilen eines Commits).

Der Zeiger zeigt eine Verschiebeform über der Titelleiste und eine Größenform über Rändern und Ecken (Terminals mit `OSC 22`).

## Form des Mauszeigers

Manche Terminals lassen GitPoulpi den Mauszeiger ändern: einen Größenänderungszeiger über
Teilern, über einer Spaltengrenze des Logs, des Radars und der Commit-Spalte (auch beim
Ziehen), eine Hand über anklickbaren Elementen. Das funktioniert in kitty, foot, Ghostty,
mintty und einem WezTerm-Build mit dem Upstream-PR #8092 (die Sequenz `OSC 22`). Sie wird
nur gesendet, wenn sich die Form ändert, in tmux nur für das Durchreichen verpackt
(`allow-passthrough on`; kitty und foot zeigen sie, Ghostty nicht) und nie in screen,
zellij oder herdr. Der Standardzeiger wird beim Beenden von GitPoulpi wiederhergestellt.
In anderen Terminals (Rio, Alacritty, Tabby, iTerm2, Terminal.app, Windows Terminal,
xterm) ändert sich der Zeiger nicht; siehe
[Ein Terminal wählen](terminals.md#form-des-mauszeigers-osc-22). Der Teiler oder die Schaltfläche unter
dem Zeiger leuchtet in jedem Terminal auf.

Dieselbe Art von Komponente hat überall in der App denselben Zeiger.

- **Schaltflächen sind echte Schaltflächen**, in den Einstellungen (Integrationen:
  Bearbeiten, Trennen, Entfernen, Durchsuchen, Löschen, die Snippet-Schaltflächen des
  Anweisungseditors ...) wie in Dialogen: Beschriftungen mit Innenabstand auf gefülltem
  Hintergrund, der unter dem Zeiger heller wird, abgedunkelt, wenn deaktiviert, nie Text
  in `[ ]`. Die einzigen verbleibenden Klammern sind die Kästchen `[x]` / `[ ]`, die
  Kontrollkästchen sind und wie jedes andere Bedienelement die Hand bekommen.
- **Die zeigende Hand** steht auf Bedienelementen: Schaltflächen, Kontrollkästchen und
  Schalter, Auswahlfelder (in den Einstellungen, in den Bereichen für Issues und Pull
  Requests, im Radar, in Dialogen), Tabs und ihre Kreuze, die `+`-Schaltflächen, Links und
  Verweise (`#12`, `!45`), Gruppen- und Ordnerkopfzeilen mit ihrem Aufklapppfeil,
  Ref-Pillen, Bildlaufleisten sowie die Schaltflächen Aktivität, Jobs und Version der
  Statusleiste. Ein Kontrollkästchen oder eine Schaltfläche in einer Zeile behält die Hand.
- **Der Pfeil** steht auf Zeilen. Eine Zeile leuchtet als Ganzes auf, wenn der Zeiger darauf
  steht (dasselbe Licht in jeder Liste), und ist für sich kein Bedienelement: die Zeilen der
  Bereiche Refs, Worktrees, Stashes und Submodule, des Logs und der Commit-Spalte (nur eine
  Ref-Pille bekommt die Hand), die Zeilen eines Diffs, die geänderten Dateien, die Einträge
  für Pull Requests, Issues und das Radar sowie die Zeilen der Listen in Dialogen und
  Bereichen (Klonen, Ordnerauswahl, Workspaces, die Karten des Boards, das Jobs-Panel, die
  Auswahlen für Zuständige, Labels und Meilensteine, Rebase-Schritte, Review-Dateien und
  -Commits, Auswahlen in Menüs und Auswahlfeldern).
- **Der Textcursor (I-Balken)** steht auf Eingabefeldern, über dem ganzen Feld: die
  Suchfelder (globale Suche, Einstellungen, Befehlspalette, Start), die Filter (Refs,
  Dateien, Radar, Aktivitätslog, die Suche im Issues-Bereich), die Text-, Zahlen- und
  Bereichszeilen der Einstellungen (auch beim Bearbeiten), die Felder der Dialoge, die
  Review-Kommentarfelder, der Commit-Composer, der Text des Dateieditors und das bearbeitbare
  Ergebnis des Merge-Editors (seine schreibgeschützten Bereiche behalten den Pfeil). In den
  Commit-Details sind Nachricht und Beschreibung bearbeitbar, der I-Balken steht also darauf,
  und der ganze Block leuchtet leicht auf.
- **In den Commit-Details** sind die Infozeilen Bedienelemente: Klicken Sie auf
  `commit: <id>`, um die kurze ID zu kopieren, doppelklicken Sie auf die Zeile von Autor
  oder Datum, um sie zu ändern (die Hand und eine leichte Markierung zeigen es). Die
  Statistikzeile informiert nur und behält den Pfeil.

Die Größenänderungszeiger bleiben auf Teilern und Spaltengrenzen, und die offene Hand
erscheint nur, solange Sie eine Ref tragen.

## In die Zwischenablage kopieren

Kopieraktionen legen den Text in die System-Zwischenablage des Rechners, auf dem
GitPoulpi läuft, und senden ihn, wo das Terminal dies bekanntermaßen beachtet, zusätzlich
an das Terminal (OSC 52). Über SSH oder ohne Display wird nur OSC 52 gesendet: Unterstützt
Ihr Terminal es nicht (iTerm2, bis seine Berechtigung für die Zwischenablage an ist,
Terminal.app, GNU screen), wird nichts kopiert, und die Statuszeile sagt, dass der Text nur
an das Terminal gesendet wurde. Siehe
[Ein Terminal wählen](terminals.md#zwischenablage-kopieraktionen).
