[English](../review.md) · [Čeština](../cs/review.md) · **Deutsch** · [Español](../es/review.md) · [Français](../fr/review.md) · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · [Polski](../pl/review.md) · [Português (Brasil)](../pt-BR/review.md) · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Einen Pull oder Merge Request prüfen

Prüfen Sie einen Pull Request (GitHub, Gitea, Forgejo), einen Merge Request (GitLab) oder einen Gerrit-Change
(siehe [Gerrit](#gerrit)) **ohne das Fenster des Repositorys zu verlassen**. Die Review-Ansicht zeigt die
Beschreibung, die Unterhaltung, die geänderten Dateien mit den Kommentar-Threads unter ihren Zeilen und die
Commits - und hier schreiben Sie das Review: Kommentare, Genehmigung und Änderungswünsche. Sie braucht ein Profil
für einen dieser Hosts (siehe [Hosting- und Issue-Tracker-Integrationen](hosting-integration.md)) und ein damit
verknüpftes Repository; andere Anbieter bieten keine Reviews an. Die Grenzen eines Hosts werden eingehalten:
GitLab kennt keine Beurteilung *Änderungen anfordern* und nimmt eine Zeile pro Kommentar. Diese Seite nennt
alle zusammen eine *Anfrage*.

![Die Review-Ansicht, Seite Übersicht: die Kopfzeile der Anfrage mit ihren Metadaten-Schaltflächen, die Beschreibung, ein offener und ein gelöster Code-Thread, die Kommentare und das Kommentarfeld](../images/review-overview.png)

![Die Seite Dateien: die geänderten Dateien mit ihren Kästchen für „angesehen“, der Diff mit einem Kommentar-Thread unter seiner Zeile und ein aufgelöster Thread außerhalb der gezeigten Zeilen](../images/review-files.png)

## Ein Review öffnen

Jeder Weg führt in dieselbe Ansicht:

- die Liste **PULL REQUESTS** der Seitenleiste: eine Zeile auswählen und **Enter** drücken, oder sie
  rechtsklicken und **Review...** wählen;
- das **Radar**: das Menü der Zeile, **Review...**, oder die Taste `e` (das Repository öffnet sich und
  das Review mit ihm; bei einer Anfrage eines Repositorys, das Sie nicht geklont haben, fragt es, ob es geklont
  oder über die API geprüft werden soll, siehe [Review ohne Klon](#review-ohne-klon));
- die **Workspace-Suche** (`Ctrl+Alt+F`): einen Pull oder Merge Request hervorheben und **`Alt+R`** drücken;
- die Palette: **Hosting: Anfrage prüfen...** listet die Anfragen zur Auswahl.

Die Ansicht füllt das Fenster über dem Repository (sie ist ein Modus des Repository-Tabs, kein eigenes Fenster).
**`Esc`** oder das `✕` in ihrer Ecke verlässt sie. Beim Öffnen wird die Anfrage vom Host gelesen und ihr Code im
Hintergrund mit git abgerufen; ein Spinner erscheint, wenn das länger als einen Moment dauert.

## Die Kopfzeile

Immer sichtbar: Nummer und Titel, der Zustand (Open, Draft, Merged, Closed), die Branches
(`main ← feature/moons`), der Autor, die Checks (`✓ Checks bestanden`, `✗ Checks fehlgeschlagen`,
`● Checks laufen`), ob sie sauber zusammengeführt werden kann oder Konflikte hat, die Größe und
**`3 ausstehende Kommentare`**, sobald Sie welche geschrieben haben.

Die vier Metadaten-Schaltflächen - **Zugewiesene**, **Reviewer**, **Labels**, **Meilenstein** - öffnen dieselben
Auswahllisten wie die Seitenleiste (siehe [Zugewiesene, Reviewer, Labels und Meilenstein](pull-requests-and-issues.md#zugewiesene-reviewer-labels-und-meilenstein));
die Kopfzeile zeigt das Ergebnis, sobald der Host es angenommen hat. Die Aktionsschaltflächen: **Auschecken**
(das übliche Auschecken des Branches der Anfrage, das auch die Ansicht schließt), **Im Browser öffnen**,
**Aktualisieren** (**`F5`**) und **Review abschließen**.

## Drei Seiten

Ein Umschalter im oberen Rand wechselt die Seite: **Übersicht**, **Dateien**, **Commits**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`** oder ein Klick).

### Übersicht

Die Übersicht ist **die Unterhaltung der Anfrage**, dieselbe Zeitleiste und dasselbe Kommentarfeld wie bei einem Issue
([Unterhaltungen](conversations.md)): zuerst die Beschreibung (*hat dies geöffnet*), dann die Kommentare,
die Threads am Code mit ihrem Diff-Ausschnitt, die abgesendeten Reviews mit ihrer Beurteilung (`genehmigt`, `Änderungen
angefordert`, `kommentiert`, `verworfen`; bei GitLab sind die Beurteilungen seine Genehmigungen) und die Ereignisse, älteste zuerst,
im neuesten Fenster mit **Ältere laden** an dessen Anfang. Das Kommentarfeld ist unter der Zeitleiste angedockt (**Neuer Kommentar**):
**Schreiben**, **Vorschau** und **Geteilt**, **Abbrechen** und **Senden**. **Ein hier geschriebener Kommentar geht sofort an den Host**
(nichts ist ausstehend: ausstehend sind die Kommentare am Code, die mit dem Review gesendet werden), erscheint gleich in der Liste
und nimmt seinen Platz ein, wenn der Host antwortet; lehnt der Host ihn ab, öffnet sich das Feld wieder mit Ihrem Text und dem Grund.

Die Tasten sind die der Unterhaltungsansicht: **`Up`** / **`Down`** (und `Ctrl+J` / `Ctrl+K`) wechseln zwischen den Einträgen,
**`r`** antwortet in den Thread des ausgewählten Eintrags, **`Q`** zitiert ihn in einem neuen Kommentar, **`c`** schreibt einen neuen
Kommentar, **`e`** / **`d`** bearbeiten und löschen Ihre eigenen Kommentare (vor dem Löschen kommt eine Rückfrage), **`l`** oder `Enter`
öffnet die Links eines Eintrags, **`g`** sein Diagramm, **`y`** kopiert seine Adresse, **`o`** öffnet ihn im Browser, **`n`**
springt zum nächsten ungelesenen Eintrag, **`v`** zeigt oder blendet Ereignisse aus, **`m`** lädt ältere Einträge, `Tab` wechselt zwischen
Zeitleiste und Feld, und **Interne Notiz** (`Alt+I`) gibt es bei den Hosts, die solche Notizen haben. Die Schaltflächen des
ausgewählten Eintrags (**Antworten**, **Zitieren**, **Bearbeiten**, **Löschen**, **Links**) tun dasselbe, und die Palette hat die
Zeilen *Unterhaltung: ...* neben den Zeilen *Review: ...*. Vor dem Öffnen eines Links wird gefragt, wie überall im
Review.

Ein Thread auf Code zeigt sein `Pfad:Zeile` als Link (unterstrichen, eine echte Schaltfläche). Ein Klick darauf, **`G`** auf dem
ausgewählten Thread oder *Unterhaltung: Code des Threads öffnen* in der Palette wechselt zur Seite **Dateien**, öffnet diese Datei
und setzt den Cursor auf die kommentierte Zeile (auf der alten oder der neuen Seite, wie sie geschrieben wurde), mit dem darunter
geöffneten Thread (ein eingeklappter wird aufgeklappt). **Esc**, die Schaltfläche **Zurück zur Übersicht** links neben den
Seiten-Tabs oder *Review: Zurück zum Kommentar in der Übersicht* führen zum selben Eintrag der Übersicht zurück. Ein Thread, dessen
Zeile nicht mehr im Diff steht (veraltet), öffnet die Datei an der nächstgelegenen Zeile, und die Statuszeile sagt es; bei einem
Thread auf einer Datei, die die Anfrage nicht mehr ändert, sagt sie, dass die Datei nicht im Diff ist. In einer Issue- oder
Radar-Ansicht derselben Anfrage gibt es keinen Code zu öffnen: `G` antwortet, dass das Review ihn hat.

Wo sich eine Taste der Unterhaltung und eine des Reviews treffen, behält auf dieser Seite die Unterhaltung den Buchstaben: `l` (Labels), `m`
(Meilenstein), `Q` (Abandon bei Gerrit), `e` / `d`, `r`. Labels, Meilenstein und Abandon bleiben auf den Schaltflächen der Kopfzeile und
in der Palette; `a` (Zuständige), `p` (Reviewer), `f` (**Review abschließen**), `b` (**Auschecken**), `P`, `D`, `U` und
`Alt+1` / `Alt+2` / `Alt+3` behalten hier ihre Bedeutung. Solange das Feld die Tastatur hat, gehört jede Taste dem Feld, außer
`Alt+1` / `Alt+2` / `Alt+3`; `Esc` gibt die Tastatur an die Zeitleiste zurück, und ein zweites `Esc` schließt das Review.

Ein Kommentar, der in einem anderen Fenster oder per CLI geschrieben wurde, erscheint hier von selbst: Jeder Schreibzugriff von GitPoulpi
(dieses Fenster eingeschlossen) meldet es der Sitzung, und die Fenster, die dieselbe Anfrage zeigen, lesen sie neu.

Das Kommentarfeld ist das gemeinsame Markdown-Textfeld der [Unterhaltungsansicht](conversations.md#schreiben)
(auch das Feld einer Antwort auf einen Thread, eines Kommentars an einer Zeile und der Zusammenfassung von **Review abschließen**): `Ctrl+Enter` (auch
`Ctrl+S`, `Alt+Enter`) sendet, und wo KI an ist, öffnet `Alt+E` **Mit KI schreiben**
([das KI-Kapitel](ai.md#schreibhilfe-auf-einem-host)). Die Vorschau zeigt den Text so, wie der Host ihn zeigt, Verweise eingeschlossen
(`#7`, `!12`, `ABC-12`, Commit-IDs); ein ungesendeter Text bleibt je Request erhalten.

### Dateien

Links die geänderten Dateien (Statusbuchstabe, `+hinzugefügt -entfernt`, eine Zahl `●n` offener Threads, eine Zahl
`✎n` Ihrer ausstehenden Kommentare) und rechts der Diff der ausgewählten Datei. Der Diff kommt von git: Der Head
der Anfrage wird nach `refs/gitpoulpi/pr/<n>` abgerufen (nie ein Branch, nichts wird ausgecheckt) und mit der
Merge-Basis verglichen, sodass die Zeilennummern am Rand genau die sind, die Kommentare verwenden.

- **`[ ]` vor einer Datei**: sie als *angesehen* markieren (Klick, oder **`v`** / `Leertaste` in der Liste). Der Titel
  zählt sie. Die Markierungen werden pro Repository gespeichert und überstehen einen Neustart.
- **Threads** stehen direkt unter ihrer Zeile: erster Kommentar, Antworten und die Schaltflächen **Antworten** und
  **Auflösen** / **Wieder öffnen**. Ein aufgelöster Thread klappt zu einer Zeile zusammen; **`z`** (oder `Enter` darauf,
  oder ein Klick auf das `▾`) klappt jeden Thread ein und aus.
- **Veraltete Threads** und Threads an Zeilen, die der Diff nicht zeigt, stehen nach den Hunks unter
  *Kommentare außerhalb der gezeigten Zeilen*, mit der Markierung `veraltet`, wenn sich der Code unter ihnen geändert hat.
- **`c`** kommentiert die Zeile, auf der der Cursor steht (das kleine `+` am Zeilenanfang tut dasselbe mit der Maus).
  **`Shift+Auf` / `Shift+Ab`** (oder ein Ziehen über die Zeilen) wählen zuerst mehrere Zeilen aus, und der
  Kommentar umfasst den Bereich. GitLab nimmt eine Zeile pro Kommentar, und die Ansicht sagt es, statt auszuwählen.
- **`r`** antwortet auf den Thread unter dem Cursor, **`x`** löst ihn auf oder öffnet ihn wieder. Beides wird
  **sofort** an den Host gesendet.
- Tasten: `Auf` / `Ab` / `Bild auf` / `Bild ab` / `Pos1` / `Ende` bewegen; `Tab`, `Rechts` und `Enter` gehen von der Dateiliste
  in den Diff, `Links` und `Tab` zurück; `]` und `[` springen zur nächsten und vorherigen Datei.

### Die Anzeigeoptionen des Diffs

Der Diff-Bereich von **Dateien** und **Commits** ist der der [Diff-Ansicht](staging-and-committing.md), mit derselben Leiste
in seiner ersten Zeile und denselben gemerkten Einstellungen (es ist eine Einstellung: was Sie hier wählen, ist das,
womit die Diff-Ansicht öffnet, und umgekehrt):

- **Inline / Split**: eine Spalte oder vorher und nachher nebeneinander. In Split steht ein Thread, ein ausstehender
  Kommentar oder ein Vorschlag unter der Hälfte, zu der er gehört: ein Kommentar an einer entfernten Zeile unter der
  linken Hälfte, ein Kommentar an einer hinzugefügten oder unveränderten Zeile unter der rechten. Klicken Sie die
  gemeinte Hälfte an, bevor Sie in einer Zeile mit Entfernung und Hinzufügung **`c`** drücken; die Tastatur
  kommentiert die neue Seite.
- **Hunks**: nur die Änderungen mit ihrem Kontext oder die ganze Datei mit den Änderungen an Ort und Stelle. In der
  ganzen Datei steht ein Thread an einer Zeile außerhalb der Änderungen unter seiner Zeile statt unter
  *Kommentare außerhalb der gezeigten Zeilen*. Der Text der Datei wird im Hintergrund gelesen; die Hunks bleiben
  mit einem erklärenden Hinweis auf dem Bildschirm, bis er da ist (auf der Seite Commits werden bis zu 40 Dateien
  eines Commits so gelesen).
- **WS**: Änderungen, die sich nur im Leerraum unterscheiden, zeigen oder ignorieren.
- **Umbr.**: lange Zeilen umbrechen.
- **Pfeile auf / ab**: der vorherige und der nächste Hunk.
- **Map** (**`M`**): die Minimap neben der Bildlaufleiste, eine Einstellung mit der der Diff-Ansicht. Anklicken
  oder darauf ziehen scrollt.
- **Auto** (**`E`** oder *Review: Codierung wählen...*): die Codierung, in der die Datei gelesen wird, wie in der
  Diff-Ansicht: Automatisch erkennen oder eine der Codierungen der Liste, gewählt für die Datei, die der Bereich
  zeigt (auf der Seite Commits die Datei unter dem Cursor). Beide Seiten der Datei werden in dieser Codierung
  im Hintergrund neu gelesen, und die Wahl gilt auch für die Diff-Ansicht desselben Pfads. Eine Datei, die sich so nicht
  lesen lässt, sagt über ihrem Diff, warum.
- **`⋯`**: Ist der Bereich zu schmal für alle Schaltflächen, stehen die, die nicht passen, hinter einem `⋯` am Ende der
  Leiste; seine Liste führt jede aus, als wäre sie angeklickt worden. Ihre Tasten und Palettenzeilen funktionieren wie
  immer.

Das Lineal der Bildlaufleiste zeigt, wo etwas ist: die Änderungen (nur ganze Datei, wie in der Diff-Ansicht), einen
Punkt `●` für jeden Thread, ausstehenden Kommentar und Vorschlag, und den Cursor.

Lange Zeilen werden am Rand des Bereichs abgeschnitten (schalten Sie **Umbr.** ein, um sie ganz zu sehen) und
scrollen seitwärts: **`Rechts`** und **`Links`** im Diff (`Links` am linken Rand geht zurück zur Dateiliste), das
seitliche Mausrad oder `Shift` mit dem Mausrad, die Bildlaufleiste am unteren Rand oder
*Review: Diff nach links / rechts scrollen*.

### Text und Zeilen auswählen

Wo ein Ziehen beginnt, bestimmt, was es auswählt. Ein Ziehen, das **im Code** beginnt, wählt Text aus, wie in der
Diff-Ansicht (Doppelklick wählt ein Wort); **`Ctrl+C`** (oder *Review: Markierten Text oder Zeile kopieren*) kopiert genau
diese Zeichen, ohne Zeilennummern oder Markierungen, und kopiert ohne Textauswahl die Zeile unter dem Cursor. Ein Ziehen,
das **in der Rinne** beginnt (die Zeilennummern, die Markierung, das `+`), wählt Zeilen aus, um sie wie bisher zu
kommentieren, und ein Doppelklick in der Rinne beginnt weiterhin einen Kommentar an dieser Zeile.

Die Tasten `n`, `p`, `m`, `w`, `s` der Diff-Ansicht gehören im Review ihm selbst, daher sind die Umschalter dort
Großbuchstaben: **`V`** Layout, **`H`** Hunks oder ganze Datei, **`S`** Leerraum, **`W`** Umbruch, **`J`** / **`K`** nächster und
vorheriger Hunk, **`E`** die Codierung, **`M`** die Minimap. Jede hat eine Palettenzeile (*Review: Layout vereint oder nebeneinander*, ...).

### Commits

Nur die Commits der Anfrage (nicht die Historie dahinter), neueste zuerst, und der Diff des ausgewählten (seine
Änderungen gegenüber seinem Elternteil), mit allen seinen Dateien untereinander und denselben Anzeigeoptionen.

## Ein Review schreiben

Ein Kommentar am Code ist **ausstehend**: Er wird auf Ihrem Rechner gehalten (in der Zustandsdatei des Repositorys,
sodass er einen Neustart überlebt) und in der Kopfzeile gezählt. Gesendet wird nichts, bevor Sie das Review
abschließen. Ein ausstehender Kommentar erscheint unter seiner Zeile mit der Markierung `Ausstehend - Zeile 12` und zwei
Schaltflächen: **`e`** (Bearbeiten) und **`d`** (Löschen).

**Review abschließen** (**`f`**, die Schaltfläche oder die Palette) öffnet ein kleines Formular: eine Zusammenfassung
(optional) und eine Liste **Beurteilung**: *Kommentar*, *Genehmigen*, *Änderungen anfordern*. Aufgelistet ist nur, was der
Host erlaubt: GitLab hat kein *Änderungen anfordern*. **Review absenden** (`Ctrl+S`) sendet die Zusammenfassung, die
Beurteilung und alle ausstehenden Kommentare in einem Zug. **Review verwerfen...** wirft die ausstehenden Kommentare nach
einer Rückfrage weg.

Sofort gesendet wird, ohne auf das Review zu warten: Antworten, Auflösen und Wieder-Öffnen sowie Kommentare zur
Unterhaltung. Das Kommentarfeld sagt, welche Art es ist (*Dieser Kommentar wird sofort gesendet* oder *Dieser Kommentar
kommt ins ausstehende Review*). Lehnt der Host eine Antwort ab, öffnet sich das Feld erneut mit Ihrem Text.

Hat die Anfrage neue Commits bekommen, nachdem Sie ausstehende Kommentare geschrieben haben, warnt die Ansicht, dass sich
deren Zeilen verschoben haben können (GitHub und GitLab setzen einen Kommentar auf den Commit, den Sie gelesen haben; ein Review
eines älteren Heads wird abgelehnt).

## Gerrit

Ein Gerrit-Server wird wie die anderen Hosts verbunden (siehe [Hosting- und Issue-Tracker-Integrationen](hosting-integration.md):
Server-URL, Benutzername, HTTP-Passwort) und unter **PULL REQUESTS** mit einem *Projekt* verknüpft. Seine Changes werden als Pull
Requests gelistet (`#7`, Chip `GR`) und in derselben Ansicht geprüft, mit den Unterschieden, die Gerrit mitbringt:

- **Patch Sets.** Ein Change hat nummerierte Patch Sets (jeder Upload des Commits ist eines). Die Kopfzeile hat eine
  Auswahlliste **`Patch Set N`** im oberen Rand, neben den Seiten-Tabs (**`P`** oder die Palettenzeile *Review: Patch Set
  wählen...*). Sie steht zunächst auf dem neuesten; wählen Sie ein anderes, werden dessen Dateien und Code gelesen, und die
  auf anderen Patch Sets geschriebenen Threads werden als *veraltet* gelistet. Das Review, das Sie abschließen, wird auf dem
  angezeigten Patch Set gesendet.
- **Mit einem anderen Patch Set vergleichen.** Eine zweite Combobox links davon, **`Vergleich: Basis`** (**`D`**, oder
  *Review: Patch Sets vergleichen...*), listet die Basis und jedes Patch Set, das älter ist als das angezeigte. Wählen
  Sie Patch Set 1, während Patch Set 3 angezeigt wird, listet und zeigt die Seite Dateien nur, was Patch Set 3 seit
  Patch Set 1 geändert hat (der eigene Unterschied des Servers, ein Rebase auf ein neueres Ziel erscheint also nicht
  als Änderung); die Seite Commits zeigt diesen einen Unterschied. Beide Patch Sets werden nach
  `refs/gitpoulpi/pr/<n>` und `refs/gitpoulpi/pr/<n>-ps<k>` geholt. Threads, die auf anderen Patch Sets als dem
  angezeigten geschrieben wurden, sind *veraltet*. Wer ein älteres Patch Set zum Lesen wählt als das verglichene,
  setzt den Vergleich auf die Basis zurück. Das CLI-Gegenstück ist `review show|fetch --compare-with K`.
- **Einreichen und Aufgeben.** Sagt der Server, dass sich der Change einreichen lässt (die nötigen Bewertungen und
  Checks sind da), kommt eine Schaltfläche **Einreichen** zu denen der Kopfzeile; **Aufgeben** ist da, solange Sie den
  Change aufgeben dürfen. Beide (Tasten **`U`** und **`Q`**, Palette *Review: Change einreichen...* / *Review: Change
  aufgeben...*) fragen zuerst, in einem Popup, das sagt, dass die Aktion auf dem Server geschieht und **von GitPoulpi
  aus nicht rückgängig zu machen ist** (ein gemergter Change bleibt gemergt; ein aufgegebener lässt sich in der
  Weboberfläche von Gerrit wiederherstellen). Ablehnen sendet nichts; bei Erfolg liest die Ansicht den Change erneut.
  Die CLI-Gegenstücke sind `gitpoulpi hosting review submit-change <n>` und
  `hosting review abandon <n> [--message M]`.
- **Bewertungen (Votes) statt einer Beurteilung.** **Review abschließen** zeigt eine Auswahlliste pro Label, für das Sie
  bewerten dürfen - `Code-Review` (-2 bis +2) und `Verified`, wo das Projekt es hat - und listet nur die Werte, die der Server
  **für diesen Change und Sie** erlaubt (ein Entwickler ohne das Recht kann +2 nicht wählen). Sie startet auf Ihrer aktuellen
  Bewertung; nur eine geänderte Bewertung wird gesendet. Die Zusammenfassung und die ausstehenden Kommentare gehen im selben Aufruf
  mit. Entwürfe, die Sie in der Gerrit-Weboberfläche gespeichert haben, werden mit dem Review veröffentlicht.
- **Threads** sind Gerrits Kommentarketten. Ein Thread ist aufgelöst, wenn sein neuester Kommentar `unresolved: false` sagt;
  **Auflösen** sendet eine Antwort `Done`, die das tut, **Wieder öffnen** eine Antwort `Reopened`. Antworten und Auflösen gehen
  sofort an den Server, wie bei den anderen Hosts.
- **Zum Review senden.** Gerrit legt einen Change an, wenn Sie nach `refs/for/<branch>` pushen. Ist der Bereich PULL REQUESTS mit einem
  Gerrit-Projekt verknüpft, fragen die Schaltfläche **`+`** des Bereichs und die Palettenzeile **Hosting: Zum Review senden (Push)...**
  nach dem Zielbranch und pushen `HEAD` nach `refs/for/<branch>` (ein Topic lässt sich auf der Kommandozeile angeben). Der Push lässt sich
  aus GitPoulpi nicht rückgängig machen (den Change in Gerrit abandonen); die Meldung sagt es. Das Gegenstück auf der Kommandozeile
  ist `gitpoulpi hosting review push`.
- Die Metadaten-Schaltflächen sind bei Gerrit nur lesend (die Reviewer werden angezeigt), es gibt keine Beurteilung *Änderungen
  anfordern* (eine Bewertung von -1 oder -2 ist dasselbe), und der Checks-Chip ist das Label `Verified`.

## ✦ KI-Review-Entwurf

![Zwei ✦-Vorschläge unter ihren Zeilen auf der Seite Dateien, jeder mit Übernehmen, Bearbeiten und Ablehnen, neben einem bestehenden Thread](../images/review-ai.png)

Bei eingeschalteter KI (*Einstellungen > KI*, siehe [KI](ai.md)) hat die Kopfzeile eine Schaltfläche **`✦ Review entwerfen...`** (auch **`s`**
und die Palettenzeile *Review: ✦ Review entwerfen...*). Sie öffnet die [Review-Optionen](#review-optionen), und
**`Enter`** dort fragt das Modell. Sie ist das Einzige, was das Modell fragt: Ein Review zu öffnen tut es nie. Das
Modell liest Titel und Beschreibung der Anfrage, die bereits vorhandenen Kommentare (damit es sie nicht wiederholt) und den
Diff und antwortet mit einem **Review-Entwurf**: dem Text zum Absenden, der Beurteilung, die es empfiehlt (Genehmigen, Änderungen anfordern oder Kommentar), einer Gesamtzusammenfassung und einer Liste vorgeschlagener Kommentare. Ein langer Diff wird für das Modell
gekürzt - pro Datei und insgesamt - und die Übersicht sagt, wie viele Dateien gekürzt oder weggelassen wurden. Während es arbeitet, zeigt die
Statuszeile nach einem Moment einen Spinner, und **`Esc`** (oder dieselbe Schaltfläche) bricht ab. Es gelten die *Code-Review*-Anweisungen der
[vier Ebenen](ai.md#anweisungen), sodass ein Team oder ein Workspace festlegen kann, wie ein gutes Review aussieht.

### Review-Optionen

`s` öffnet den Dialog **✦ Review-Optionen**, gefüllt mit den Standardwerten des Repositorys (den globalen, wo das
Repository keine hat). `Enter` (oder die Schaltfläche **Starten**) startet den Lauf mit dem, was der Dialog zeigt; `Esc`, **Abbrechen**
oder das Kreuz schließt ihn und fragt nichts.

| Steuerelement | Wirkung |
|---|---|
| **Agent im Repository ausführen** | *Profileinstellung* (der Arbeitsordner des KI-Profils entscheidet), *Ja* oder *Nein*. Ausgegraut, mit dem Grund darunter, bei einem HTTP-Profil (es hat keinen Arbeitsordner) und wenn die Anfrage ohne Klon über die API geprüft wird. |
| **CLAUDE.md**, **AGENTS.md** | Dem Modell diese Datei des Repositorys geben, gelesen auf dem Branch, auf den die Anfrage zielt. Läuft der Agent im Repository, liest er sie selbst, und der Dialog sagt das. |
| **Zusätzliche Datei** | Eine weitere Datei: ein Pfad im Repository, ein absoluter Pfad oder ein Link auf eine Datei eines Code-Hosts. **Durchsuchen...** (`Ctrl+B`, oder `Enter` auf der Schaltfläche) öffnet eine Dateiauswahl, die an der Repository-Wurzel beginnt (oder neben der bereits genannten Datei); eine Datei im Repository wird als Pfad relativ zur Wurzel geschrieben, eine außerhalb als absoluter Pfad. Das Tippen im Feld funktioniert wie zuvor. |
| **Keine zusätzliche Datei** | Keine zusätzliche Datei für diesen Lauf, auch wenn die globale Einstellung oder das Repository eine nennt; wer eine Datei tippt oder mit **Durchsuchen...** wählt, hebt den Haken auf. Mit **Als Standard dieses Repositorys speichern** wird daraus der Schalter *Keine zusätzliche Datei* des Repositorys. Ein leeres Feld gilt hier als keine, weil der Dialog mit dem beginnt, was der Lauf erben würde. |
| **Als Standard dieses Repositorys speichern** | Behält die Werte auch für die nächsten Reviews: in der Zustandsdatei des Repositorys oder, bei einem nicht geklonten Repository, in `remote-reviews.toml`. Nur ein Wert, der vom globalen abweicht, wird behalten; der Rest folgt weiter *Einstellungen > KI*. |

Alles ist mit `Tab` erreichbar, `Space` schaltet ein Kästchen um, und das Pfadfeld ist ein gewöhnliches Textfeld. Dieselben
vier Einstellungen gibt es in *Einstellungen > KI* (*Code-Review*) und in *Repository-spezifische Einstellungen > KI*; siehe
[Review-Kontext](ai.md#review-kontext). Was dem Modell nicht übergeben werden konnte (eine zusätzliche Datei, die fehlt oder
nicht lesbar ist) und jede Datei, die auf die Limits gekürzt wurde, steht auf der Entwurfskarte.

Vorschläge sind **keine Kommentare**. Sie erscheinen an zwei Stellen:

- **Unter ihrer Zeile in Dateien**, als Geisterkommentare mit `✦` und einem Schweregrad (`Hinweis`, `Warnung`, `Problem`) und drei
  Schaltflächen: **Übernehmen**, **Bearbeiten** und **Ablehnen**. Ein Vorschlag zu einer Zeile, die der Diff nicht zeigt, wird
  aufbewahrt und nie verworfen: Er steht nach den Hunks unter *KI-Vorschläge zur ganzen Datei* und landet, einmal übernommen, an der
  ersten geänderten Zeile der Datei, mit den Worten *(zu Zeile N)* davor.
- **In der Übersicht**, in einem Block über der Unterhaltung (nur nach einem Lauf): die Zusammenfassung, dann die Liste mit denselben Schaltflächen und **Alle übernehmen** /
  **Alle ablehnen**.

**Übernehmen** (`y`) macht aus einem Vorschlag einen gewöhnlichen **ausstehenden Kommentar** (den Sie weiter bearbeiten oder löschen
können); **Bearbeiten** (`w`) öffnet das Kommentarfeld mit den Worten des Modells zuerst und übernimmt, was Sie stehen lassen; **Ablehnen** (`n`)
verwirft ihn. **Alle übernehmen** (`A`) füllt weiterhin nur die Liste der ausstehenden Kommentare. Auf der Seite Dateien wirken die Tasten
auf den Vorschlag unter dem Cursor, in der Übersicht auf den ersten der Liste (`y` und `n` sind wieder *Link kopieren* und *Nächster ungelesener* der Unterhaltung, sobald die Liste leer ist). **KI-Zusammenfassung verwenden** (`i` oder die Schaltfläche im
Formular Review abschließen) setzt die Zusammenfassung des Modells in den Review-Text; sie wird nie von selbst eingefügt.
**Nichts wird an den Host gesendet, bevor Sie das Review absenden.**

Die Übersicht zeigt den Entwurf als Karte am Anfang dieses Blocks, über den Vorschlägen: den Chip mit der Beurteilung, den Review-Text und drei
Schaltflächen.

- **✦ Entwurf verwenden...** (`u`) übernimmt jeden offenen Vorschlag als ausstehenden Kommentar und öffnet *Review abschließen*
  bereits mit dem Text und der Beurteilung des Entwurfs gefüllt. Sie lesen es weiterhin selbst, ändern es und senden es selbst ab;
  bricht man das Formular ab, bleibt der Entwurf erhalten. Die Grenzen des Hosts bleiben gewahrt: GitLab hat kein
  *Änderungen anfordern*, das Formular sagt das und fällt auf *Kommentar* zurück; bei Gerrit wird die Beurteilung zu einer Code-Review-Stimme von +1 oder -1,
  nur wenn der Server diesen Wert für Sie erlaubt (sonst wird keine Stimme gesetzt und das Formular sagt das).
- **Text bearbeiten...** (`t`) ändert den Review-Text des Entwurfs, bevor Sie ihn verwenden.
- **Entwurf verwerfen** (`N`) wirft den Text, die Beurteilung und die übrigen Vorschläge weg.

Der Entwurf wird pro Anfrage aufbewahrt und ist noch da, wenn Sie das Review schließen und wieder öffnen. Hat sich der Head der Anfrage
bewegt, seit das Modell geschrieben hat, sagt die Karte *Die Anfrage hat sich geändert, seit dieser Entwurf geschrieben wurde.*
Das Absenden des Reviews verbraucht den Entwurf.

## Alles hat eine Taste und eine Palettenzeile

| Aktion | Taste | Palette |
|---|---|---|
| Übersicht / Dateien / Commits | `Alt+1` / `Alt+2` / `Alt+3` | *Review: Übersicht zeigen*, *Dateien zeigen*, *Commits zeigen* |
| Die Zeile oder den Bereich kommentieren | `c` | *Review: Zeile kommentieren...* |
| Die Unterhaltung kommentieren | `c` in der Übersicht, `C` | *Review: Unterhaltung kommentieren...* (öffnet das Feld der Übersicht), *Unterhaltung: Neuer Kommentar* |
| Auf den Thread antworten | `r` | *Review: Auf Thread antworten...* |
| Auflösen oder wieder öffnen | `x` | *Review: Thread auflösen oder wieder öffnen* |
| Den Code eines Threads öffnen (Übersicht) / zurück zum Kommentar | `G` / `Esc` | *Unterhaltung: Code des Threads öffnen*, *Review: Zurück zum Kommentar in der Übersicht* |
| Einen Thread ein- oder ausklappen | `z` | *Review: Thread ein- oder ausklappen* |
| Ausstehenden Kommentar bearbeiten / löschen | `e` / `d` | *Review: Ausstehenden Kommentar bearbeiten...*, *Ausstehenden Kommentar löschen* |
| Die Datei als angesehen markieren | `v` | *Review: Datei als angesehen markieren* |
| Inline / Split | `V` | *Review: Layout vereint oder nebeneinander* |
| Ganze Datei / nur die Hunks | `H` | *Review: Ganze Datei oder nur Hunks* |
| Reine Leerraum-Änderungen | `S` | *Review: Reine Leerraum-Änderungen zeigen oder ausblenden* |
| Zeilenumbruch | `W` | *Review: Zeilenumbruch umschalten* |
| Nächster / vorheriger Hunk | `J` / `K` | *Review: Nächster Hunk*, *Vorheriger Hunk* |
| Nächste / vorherige Datei | `]` / `[` | *Review: Nächste Datei*, *Vorherige Datei* |
| Das Review abschließen / verwerfen | `f` | *Review: Review abschließen...*, *Ausstehendes Review verwerfen...* |
| Aktualisieren | `F5`, `Ctrl+R` | *Review: Aktualisieren* |
| Im Browser öffnen / auschecken | `o` / `b` | *Review: Im Browser öffnen*, *Branch auschecken* |
| Zugewiesene / Reviewer / Labels / Meilenstein | `a` / `p` / `l` / `m` | *Review: Zugewiesene bearbeiten...* und ihre drei Geschwister |
| Das Patch Set wählen (Gerrit) | `P` | *Review: Patch Set wählen...* |
| Mit einem älteren Patch Set vergleichen (Gerrit) | `D` | *Review: Patch Sets vergleichen...* |
| Den Change einreichen / aufgeben (Gerrit) | `U` / `Q` | *Review: Change einreichen...*, *Review: Change aufgeben...* |
| ✦ Review entwerfen... (KI an; öffnet die Review-Optionen) | `s` | *Review: ✦ Review entwerfen...* |
| Einen Vorschlag übernehmen / bearbeiten und übernehmen / ablehnen | `y` / `w` / `n` | *Review: Vorschlag übernehmen*, *Vorschlag bearbeiten und übernehmen...*, *Vorschlag ablehnen* |
| Alle Vorschläge übernehmen / ablehnen | `A` / `N` | *Review: Alle Vorschläge übernehmen*, *Alle Vorschläge ablehnen* |
| Die KI-Zusammenfassung in den Review-Text übernehmen | `i` | *Review: KI-Zusammenfassung in den Review-Text übernehmen* |
| Den ✦-Entwurf verwenden (füllt Review abschließen) | `u` | *Review: ✦ Entwurf verwenden...* |
| Den Text des Entwurfs bearbeiten | `t` | *Review: Text des ✦ Entwurfs bearbeiten...* |
| Den ✦-Entwurf verwerfen | `N` | *Review: ✦ Entwurf verwerfen* |
| Schließen | `Esc` | *Review: Schließen* |

Öffnen Sie die Palette (`Ctrl+P`), während die Ansicht offen ist: Ihre Zeilen wirken auf sie und sagen, warum eine gerade
nichts tut („zuerst den Cursor auf eine Codezeile setzen“).

## Review ohne Klon

Auch eine Anfrage eines Repositorys, das nicht auf Ihrem Rechner liegt, lässt sich prüfen. Im Radar fragt **Review**
(`e`) auf ihrer Zeile, wie (siehe [Das Radar](radar.md#einen-request-prüfen)); **Über die API prüfen** öffnet
dieselbe Ansicht im Home-Fenster, über dem Radar. Alles auf dieser Seite funktioniert gleich, mit diesen
Unterschieden:

- Der Code (die geänderten Dateien, der Diff jeder Datei, die Commits der Anfrage, der Diff eines Commits, die ganze Datei) wird
  über die API des Hosts gelesen; der erste Blick auf eine große Anfrage dauert also einen Moment, und eine Datei, für die der Host
  keinen Diff liefert (eine binäre oder sehr große), sagt das, statt einen zu zeigen.
- Ihre ausstehenden Kommentare liegen in `remote-reviews.toml` im Konfigurationsordner, abgelegt unter dem Host
  und dem Projekt (`host/project`), statt im eigenen Zustand eines Repositorys. Sie sind noch da, wenn Sie das Review
  schließen und wieder öffnen, und das Absenden des Reviews verbraucht sie.
- Es gibt keine Kodierungswahl (*Ohne Klon nicht verfügbar*) und kein **Auschecken** (es sagt, zuerst das
  Repository zu klonen).
- `Esc` oder das `✕` in der Ecke verlässt die Ansicht und gibt das Radar zurück. Die Palette (`Ctrl+P`) erreicht die
  Zeilen der Ansicht wie im Tab eines Repositorys.

Um stattdessen den Klon zu behalten, wählen Sie **Klonen, dann prüfen** (oder in der Palette *Radar: Klonen und prüfen...*):
Der Dialog Klonen öffnet sich auf dem verbundenen Konto der Anfrage mit ihrem ausgewählten Repository, und das Review öffnet sich im Tab des neuen Repositorys, sobald der Klon da ist.

## Von der Kommandozeile und für Agenten

Dasselbe Review gibt es als `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(und die daraus gebauten MCP-Tools): siehe [CLI-Referenz](cli-reference.md). `threads` listet auch die bereits
abgesendeten Beurteilungen (`reviews`). `suggest <n>` fragt das Modell (KI an) und gibt sein `verdict`, seinen `body`, seine Zusammenfassung und seine
Vorschläge als JSON aus, ohne etwas zu senden, sodass ein Skript oder ein Agent die Kommentare auswählen kann, die er an
`submit --comment` übergibt.

Nichts davon braucht einen Klon: `--project owner/name` (mit `--id`, wenn mehrere Integrationen konfiguriert sind)
führt `show`, `threads`, `comment`, `reply`, `resolve`, `submit` und `suggest` für ein Projekt aus, von dem das aktuelle
Verzeichnis nichts weiß. `hosting review diff <n>` gibt den Diff mit seinen Hunks aus (`--api` liest ihn auch innerhalb eines Klons
vom Host), und `hosting file <path> --project P [--ref R]` liest eine Datei in einer Revision, zum Beispiel die `CLAUDE.md`
des Projekts. Beide lesen die API des Hosts: GitHub, GitLab, Gitea, Forgejo und Gerrit.
