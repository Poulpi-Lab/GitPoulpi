[English](../history-rewrite.md) · [Čeština](../cs/history-rewrite.md) · **Deutsch** · [Español](../es/history-rewrite.md) · [Français](../fr/history-rewrite.md) · [Italiano](../it/history-rewrite.md) · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · [Polski](../pl/history-rewrite.md) · [Português (Brasil)](../pt-BR/history-rewrite.md) · [Português (Portugal)](../pt-PT/history-rewrite.md) · [Русский](../ru/history-rewrite.md) · [Svenska](../sv/history-rewrite.md) · [Türkçe](../tr/history-rewrite.md) · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Verlauf umschreiben

Das Umschreiben des Verlaufs ändert Commits, die es schon gibt: wer sie wann geschrieben hat,
ihre Nachrichten, die Dateien, die sie enthalten, ihre Signaturen. GitPoulpi erledigt das mit
einem **Rezept** (ein Bereich und eine geordnete Liste von Regeln) und in drei Schritten, von
denen keiner den mittleren überspringt:

1. Die **Vorschau** berechnet den neuen Verlauf, ohne etwas anzufassen: Zähler, was jede Regel
   getan hat, die Branches und Tags, die sich bewegen würden, eine Zeile pro geändertem Commit.
2. **Anwenden** setzt Ihre lokalen Branches und Tags auf diesen Verlauf. Das ist ein Eintrag im
   [Rückgängig-Verlauf](undo-redo.md): **Rückgängig** stellt jede verschobene Ref (und die
   ausgecheckten Dateien) wieder her.
3. **Push** (ein Force-Push mit Lease) und **Bereinigen** sind eigene, bewusste Schritte, weil
   sie sich nicht rückgängig machen lassen: siehe [Das Ergebnis pushen](#das-ergebnis-pushen)
   und [Den alten Verlauf löschen](#den-alten-verlauf-löschen-bereinigen).

Die Werkbank, die Palette, die Befehle `gitpoulpi rewrite` und die MCP-Werkzeuge nutzen
dieselbe Engine, daher wendet das eine an, was das andere in der Vorschau erstellt hat. [Auf der
Befehlszeile](#auf-der-befehlszeile) listet die Flags; [Regeln mit KI
erstellen](#regeln-mit-ki-erstellen) füllt die Werkbank aus einer Beschreibung; [Vorschläge von
KI-Agenten](#vorschläge-von-ki-agenten) sagt, was ein Agent hier darf und was nicht.

**Die Regeln, nach dem, was sie ändern**

| Gruppe | Regeln | Abschnitt |
|---|---|---|
| Autoren | Autor ersetzen, Autoren anonymisieren, .mailmap anwenden | [Autoren und die .mailmap](#autoren-und-die-mailmap) |
| Datum | Datum verschieben, Zeitzone ändern, Auf Uhrzeiten beschränken, Committer-Datum = Autordatum | [Datumsangaben](#datumsangaben) |
| Nachrichten | In Nachrichten suchen und ersetzen, Nachrichtenzeilen löschen, Trailer hinzufügen oder entfernen, Kodierung der Nachrichten korrigieren | [Nachrichten](#nachrichten) |
| Dateiinhalte und Pfade | In Dateien suchen und ersetzen, Dateien aus dem Verlauf entfernen, Große Dateien entfernen, Pfade verschieben oder umbenennen, Zeilenenden umwandeln, Ausführbar-Bit entfernen, Dateikodierung umwandeln | [Dateien und Pfade](#dateien-und-pfade) |
| LFS | Dateien zu LFS migrieren, Dateien aus LFS zurückholen | [Dateien nach Git LFS verschieben oder zurückholen](#dateien-nach-git-lfs-verschieben-oder-zurückholen) |
| Signaturen | Signaturen entfernen (und neu signieren beim Anwenden) | [Signaturen](#signaturen) |
| Skript | Lua, Shell, Python | [Die Skriptregel](#die-skriptregel) |

**Bevor Sie anfangen**

- Eine Vorschau ändert nichts. Anwenden ändert nur Ihre lokalen Refs und pusht nie.
- Umgeschriebene Commits bekommen neue IDs. Alle, die den alten Verlauf haben, müssen
  Bescheid wissen: [Das Ergebnis pushen](#das-ergebnis-pushen) schreibt den Hinweis dafür.
- Ein gepushtes Geheimnis gilt als offengelegt. **Zuerst rotieren**, dann umschreiben,
  force-pushen und bereinigen.
- Commits, die unverändert bleiben, behalten ihre IDs und ihre Signaturen.

## Die Werkbank

`Ctrl+G Shift+H` (oder **Umschreiben: Werkbank für den Verlauf** in der Palette) öffnet die
Werkbank zum Umschreiben über dem Log. Sie hat keine Schaltfläche in der Symbolleiste:
Erweiterte Aktionen liegen in der Palette (jede Zeile `Umschreiben: ...` öffnet die Werkbank
mit der passenden Regel bereits hinzugefügt), im Untermenü **Verlauf umschreiben** des
Commit-Menüs (*Ab diesem Commit bis HEAD...*, *Ausgewählte Commits...* bei mehreren markierten
Commits, *Werkbank öffnen...*) und hinter dem Tastenkürzel.

`Ctrl+N` zeigt ein Badge auf jedem Bedienelement der Werkbank (Bereich, Regeln, Felder, Tabs, Schaltflächen); ein Druck auf ein Badge tut, was ein Klick auf dieses Bedienelement tut, und `Esc` nimmt die Badges weg (siehe [Badges](keybindings.md#badges)).

![Die Werkbank zum Umschreiben der Historie: die Bereichsleiste, eine Liste mit zwei Regeln Autor ersetzen samt den Feldern der gewählten Regel und die Zusammenfassung der Vorschau, wie viele Commits sich ändern](../images/history-rewrite.png)

- **Bereichsleiste:** welche Refs sich bewegen (aktueller Branch, alle lokalen Branches,
  gewählte Branches und Tags, die ausgewählten Commits), das Kästchen *Tags* und, nach
  *Ab diesem Commit bis HEAD...*, ein Kästchen, um nach dem Elternteil dieses Commits zu
  beginnen.
- **Regeln:** eine nummerierte Liste, die der Reihe nach angewendet wird. Das Kästchen
  schaltet eine Regel ein oder aus, die Pfeile verschieben sie, das Kreuz löscht sie
  (*Gelöschte Regel wiederherstellen* oder `Ctrl+Z` holt sie zurück). *Regel hinzufügen*
  listet die Arten nach Gruppen. Eine Regel, die ein Programm braucht, das Sie nicht haben,
  bleibt ausgegraut und nennt den Grund.
- **Einstellungen einer Regel:** Ein Klick auf ein Textfeld bearbeitet es; ein Klick auf ein
  Feld mit mehreren Werten öffnet dessen Liste; Kästchen werden umgeschaltet.
- **Vorschau** (`F5`) berechnet den neuen Verlauf, ohne etwas anzufassen, und zeigt einen
  Fortschrittsbalken. Die Tabs: **Zusammenfassung** (Zähler, was jede Regel getan hat,
  Warnungen), **Commits** (eine Zeile pro geändertem Commit, seitenweise vom Daemon geladen;
  der Filter zeigt geänderte, entfernte oder bereits gepushte), **Branches und Tags** (was sich
  bewegt, und die geschützten Branches, die schon gepusht sind, je mit einem Kästchen),
  **Auswirkung** (was das Umschreiben gepushten Verlaufs bedeutet: ein Force-Push mit Lease,
  Mitwirkende, Signaturen), **Geheimnisse**, **Identitäten** und **Größe** (siehe unten).
- **Dateien eines Commits:** `Enter` (oder ein Doppelklick) auf einer Zeile in Commits öffnet
  die Dateien, die die Inhaltsregeln darin geändert haben, und daneben den Diff der
  ausgewählten. Das Kreuz, `Esc` oder ein Klick daneben schließt sie. Entfernte Zeilen einer
  Geheimnisregel bleiben verborgen.
- **Anwenden** (`Ctrl+S` oder `Ctrl+Enter`) ist möglich, sobald die Vorschau zu den Regeln
  passt. Fragen ohne Standardantwort werden jedes Mal gestellt: was mit den Signaturen
  signierter Commits geschieht und wie ein Ordner herausgelöst wird. Das Ergebnisfenster bietet
  **Rückgängig**, das jede verschobene Ref wieder auf ihren alten Stand setzt.
- **Speichern / Laden** legen ein Rezept im Repository (geteilt) oder nur für Sie ab; geheime
  Werte werden nie gespeichert.

Jede Schaltfläche hat eine Taste: Die Liste steht in Einstellungen > Tastatur & Maus (Kontext
*Werkbank zum Umschreiben des Verlaufs*) und im Leitfaden `keybindings.md`.

### Autoren und die .mailmap

- **Autor ersetzen** erkennt einen Autor oder Committer an Name und/oder E-Mail-Adresse (ein
  Literal oder ein regulärer Ausdruck) und setzt einen neuen Namen und/oder eine neue
  E-Mail-Adresse. *Anwenden auf* wählt den Autor, den Committer oder beide; *Auch die Autoren
  von Tags* sorgt dafür, dass die neu erstellten annotierten Tags folgen. Der ursprüngliche
  Committer bleibt erhalten, solange der Lauf nicht *ich, jetzt* sagt.
- **Autoren anonymisieren** ersetzt jede Identität durch einen nummerierten Platzhalter
  (`Contributor 1`, `contributor-1@anonymous.invalid`; beide Muster lassen sich ändern, mit
  `{n}` für die Nummer). Dieselbe Person bekommt in einem Lauf immer dieselbe Nummer;
  Identitäten, die Sie als *behalten* auflisten, bleiben unverändert.
- **Der Tab Identitäten** (auch **Umschreiben: Identitäten mergen (.mailmap)...** in der
  Palette) behebt den Fall einer Person, die unter mehreren Namen und E-Mail-Adressen committet
  hat, **ohne etwas umzuschreiben**. Er listet jeden Namen und jede E-Mail-Adresse des Bereichs
  der Bereichsleiste, mit der Zahl der Commits als Autor und als Committer und mit den
  Identitäten, die die `.mailmap` des Repositorys schon abbildet (`zugeordnet zu Name <email>`).
  Die Identitäten ankreuzen, die dieselbe Person sind (`Space` oder Klick; *Alle markieren* /
  *Keine markieren*), dann **Mergen in...** (`m`): Ein Dialog fragt, welche davon bleibt (die
  mit den meisten Commits steht oben), und **fügt der `.mailmap` pro weiterer Identität eine
  Zeile hinzu** (`Name <email> Alter Name <alt@email>`; eine bereits vorhandene Zeile wird nicht
  wiederholt, die Datei wird bei Bedarf angelegt). Nichts wird committet: die `.mailmap` selbst
  committen, um sie zu teilen. Log, Blame und Dateiverlauf zeigen den kanonischen Namen sofort
  (Einstellung `log.use_mailmap`, standardmäßig an; die gespeicherten Commits ändern sich nie,
  es gibt also nichts zu force-pushen). Die Statuszeile sagt, was geschrieben wurde, und bietet
  **Rückgängig** (`u`) an, das die vorherige `.mailmap` wiederherstellt (ein Eintrag im
  Rückgängig-Verlauf wie jede andere Aktion). `r` liest die Liste neu.
- **Eine `.mailmap` statt eines Umschreibens.** Ändert eine Autorenregel Commits, die schon auf
  einem Remote liegen, sagt die Zusammenfassung *Eine .mailmap korrigiert die Anzeige, ohne den
  Verlauf umzuschreiben. Stattdessen die .mailmap verwenden?* mit der Schaltfläche **.mailmap
  verwenden** (`m` in der Zusammenfassung). Sie öffnet den Tab Identitäten mit den
  Identitäten, auf die die Regel passt, bereits angekreuzt, und mit der Identität, die die Regel
  setzen würde, als der zu behaltenden, auch wenn sie noch kein Commit trägt.
- **Die `.mailmap`-Regel** (*Umschreiben: .mailmap auf den Verlauf anwenden...* oder `rewrite
  preview --mailmap`) schreibt tatsächlich um: Autoren und Committer (Ihre Wahl) werden zu dem,
  was die `.mailmap` des Worktrees nennt, also dem Ergebnis von `git log --use-mailmap`. Sie ist
  richtig, wenn der Verlauf selbst die kanonischen Namen tragen soll, etwa vor einer
  Veröffentlichung; sie ist ein Umschreiben wie jedes andere (Vorschau, Anwenden, Force-Push).
- **Identitätsregeln und Trailer.** Eine Identitätsregel kann auch die Personen ändern, die in
  Trailern genannt sind: **Auch Trailer umsetzen** ankreuzen (`--map-trailers`). Jeder Trailer,
  dessen Wert genau `Name <email>` lautet und auf *E-Mail ist* / *Name ist* der Regel passt,
  erhält den neuen Namen und die neue E-Mail-Adresse, unabhängig davon, was die Regel mit Autor
  und Committer macht. Standardmäßig ist es aus: Ein `Signed-off-by` hält fest, wer signiert
  hat, also ist seine Änderung eine eigene Entscheidung.

### Datumsangaben

Die vier Datumsregeln wirken auf das Autordatum, das Committer-Datum oder beide (*Datum* im
Bereich der Regel).

- **Datum verschieben** verschiebt jedes Datum um einen Betrag (`+2h`, `-1d`, `90m`, `1d 2h`).
  Die Zeitzone, in der jeder Commit geschrieben wurde, bleibt erhalten.
- **Zeitzone ändern** schreibt die Daten in einem anderen UTC-Versatz. *Beibehalten* sagt, was
  bleibt: der **Zeitpunkt** (derselbe Moment, die Uhr zeigt anderes) oder die **Uhrzeit**
  (dieselbe Ortszeit, der Moment ändert sich).
- **Auf Uhrzeiten beschränken** verschiebt jeden Commit, der außerhalb der erlaubten Zeiten
  liegt (zum Beispiel `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`), auf die nächste erlaubte
  Zeit, oder nur nach früher, oder nur nach später (*Verschieben nach*). *Nie früher als der
  Elternteil* erhält die Reihenfolge des Verlaufs; der *Startwert* macht die kleine zufällige
  Streuung wiederholbar: derselbe Startwert auf demselben Verlauf ergibt dieselben Daten.
- **Committer-Datum = Autordatum** setzt den Committer (Name, E-Mail und Datum) auf den des
  Autors, was ein `git commit --amend` oder ein Rebase rückgängig gemacht hat.

Datumsregeln sind Regeln wie die anderen: Vorschau, den Tab **Commits** prüfen (jede Zeile zeigt
das alte und das neue Datum), dann anwenden. Die ID eines Commits hängt von seinen Daten ab,
darum ändert eine Datumsregel jeden Commit nach dem ersten, den sie berührt.

### Nachrichten

- **In Nachrichten suchen und ersetzen** und **Nachrichtenzeilen löschen** nehmen einen
  literalen Text oder einen regulären Ausdruck (*Regulärer Ausdruck*, *Groß-/Kleinschreibung*);
  ein leerer Ersatz löscht. Eine Nachricht, die kein gültiges UTF-8 ist, wird von ihnen nie
  angefasst: zuerst **Kodierung der Nachrichten korrigieren** ausführen, wenn sie bearbeitet
  werden soll.
- **Trailer.** *Trailer hinzufügen oder entfernen* (*Umschreiben: Trailer hinzufügen oder
  entfernen...* in der Palette, oder `--trailer-add`, `--trailer-remove`, `--trailer-map`)
  bearbeitet die Zeilen `Key: value` am Ende einer Nachricht, die `git interpret-trailers`
  kennt (`Signed-off-by`, `Co-authored-by`, `Reviewed-by`...). Das Auswahlfeld **Aktion** wählt
  *Hinzufügen* (den **Trailer**-Schlüssel, seinen **Wert** und **Falls schon vorhanden**:
  *Hinzufügen (außer bei Duplikat)*, *Vorhandene ersetzen* oder *Commit unverändert lassen*),
  *Entfernen* (jeden Trailer dieses Schlüssels oder nur die, deren Wert **Wert enthält**
  enthält) oder *Wert ändern* (**Im Wert suchen** wird zu **Ersetzen durch**, etwa eine alte
  E-Mail-Adresse durch eine neue in `Signed-off-by`). Schlüssel werden ohne Beachtung der
  Groß-/Kleinschreibung verglichen. Nur die Trailer-Zeilen ändern sich, der Rest der Nachricht
  bleibt genau, wie er war, und eine Nachricht ohne Trailer bekommt eine Leerzeile und einen
  eigenen Block. Der Tab Commits zeigt jede Änderung als `+Signed-off-by: ...` /
  `-Co-authored-by: ...`. Der letzte Absatz einer Nachricht ist ihr Trailer-Block (nie der
  Titel), eine Zeile wie `Note: see the ticket` im Text ist also kein Trailer, es sei denn, sie
  steht zusammen mit anderen Trailern allein im letzten Absatz.
- **Commit-IDs in Nachrichten.** **Commit-IDs aktualisieren** (`--update-message-hashes`,
  standardmäßig aus; am Ende der Bereiche der Regeln, die Nachrichten bearbeiten) lässt in
  Nachrichten geschriebene IDs dem Umschreiben folgen: Eine abgekürzte ID aus 7 oder mehr
  Hex-Zeichen (`This reverts commit 1a2b3c4`, `see 1a2b3c4d5e`), die genau einen Commit des
  umgeschriebenen Verlaufs benennt, wird durch die neue ID dieses Commits ersetzt, auf dieselbe
  Länge gekürzt. Eine, die auf mehrere Commits passt, auf keinen oder auf einen Commit, den das
  Umschreiben erst später erzeugt, bleibt, wie sie ist; die Zusammenfassung sagt, wie viele
  ersetzt wurden und wie viele blieben.
- **Kodierung der Nachrichten korrigieren** (*Umschreiben: Kodierung der Nachrichten
  korrigieren...* oder `--fix-message-encoding [--fix-name-encoding]`) kodiert Nachrichten nach
  UTF-8 um, in zwei Fällen:
  - Ein Commit mit einem `encoding`-Header (`ISO-8859-1`, `Windows-1252`, `Shift_JIS`, alles,
    was Git und die Kodierungstabellen kennen) wird dekodiert und als UTF-8 geschrieben, und der
    Header entfällt: Der Text liest sich überall gleich, ohne Umwandlung. Die Namen von Autor
    und Committer dieses Commits lagen in derselben Kodierung vor und werden mit ihr dekodiert,
    sodass ein Commit nie halb umgewandelt bleibt.
  - **Mojibake**, also UTF-8, das einmal als Latin-1 oder Windows-1252 gelesen wurde
    (`RÃ©Ã©criture` statt `Réécriture`), wird **nur repariert, wenn die Rückumwandlung exakt
    aufgeht**: Der Text muss sich in Bytes zurückkodieren lassen, die gültiges UTF-8 sind, und
    das Ergebnis auf dieselbe Weise zerstört muss das Original ergeben. Eine Nachricht, die nur
    `Ã©` neben echten Buchstaben enthält oder Zeichen außerhalb der alten Alphabete, besteht
    das nicht und bleibt, wie sie ist.

  *Auch Namen korrigieren* (`--fix-name-encoding`) repariert zerstörte Namen von Autor und
  Committer auf dieselbe Weise. Was sich nicht entscheiden lässt (ein unbekannter
  Kodierungsname, Bytes, die in der angegebenen Kodierung ungültig sind, eine Nachricht, die kein
  UTF-8 ist und nichts angibt, Text, der zerstört aussieht, es aber nicht nachweislich ist), wird
  **in Ruhe gelassen und gezählt**: Die Zusammenfassung sagt *n Nachrichten blieben unverändert:
  ihre Kodierung lässt sich nicht bestimmen*, und `counts.encoding_left` hat die Zahl. Der Tab
  Commits zeigt jede Korrektur mit alter und neuer Nachricht und einer Zeile *Kodierung*
  (`ISO-8859-1` wird zu `UTF-8`).

### Dateien und Pfade

- **Regeln für Dateiinhalte** (*In Dateien suchen und ersetzen*, *Dateien aus dem Verlauf
  entfernen*, *Große Dateien entfernen*): Der Bereich prüft jeden Wert beim Tippen und zeigt den
  Grund unter dem Feld (ein Muster, das sich nicht kompilieren lässt, ein ungültiges Pfadmuster,
  eine Größe unter einem Byte). Größen akzeptieren Einheiten (`500 KiB`, `5 MiB`, `1 GiB`). Ein
  Pfadmuster ohne Schrägstrich passt in jeder Tiefe. *Geheime Werte* werden in einem verdeckten
  Feld eingegeben, gelten nur für diesen Lauf und werden nie gespeichert; die Vorschau blendet
  die Zeilen aus, die sie enthielten. *Leere Commits* legt fest, ob ein Commit, den die Regeln
  leer gelassen haben, entfernt (Standard) oder behalten wird. Binärdateien und Dateien über der
  Größengrenze bleiben unverändert und werden in der Zusammenfassung gezählt.
- **Pfade verschieben.** *Pfade verschieben oder umbenennen* (*Umschreiben: Pfade verschieben
  oder umbenennen...* in der Palette) kennt drei Arten, gewählt im Auswahlfeld **Verschieben**:
  - *Pfad umbenennen* (**Von**, **Nach**; `--move-path FROM=TO`): Dieselben Dateien und Ordner
    erscheinen in jedem Commit, der den alten Pfad hatte, am neuen Pfad; sonst ändert sich
    nichts, jede Datei behält ihre ID. Ein Commit, in dem der neue Pfad schon existiert, wird nie
    überschrieben: Die Vorschau wird abgelehnt und listet die Commits und die Pfade, die sich
    beißen (einen freien Namen wählen oder den vorhandenen Pfad zuerst verschieben). Nur exakte
    Pfade, keine Platzhalter.
  - *Alles in einen Ordner verschieben* (**Ordner**; `--into-subdir DIR`): Jeder Pfad jedes
    Commits erhält das Präfix. Praktisch, um Repositorys ohne Namenskonflikte zusammenzuführen.
  - *Ordner als Wurzel herauslösen* (**Ordner**; `--extract DIR`): Der Ordner wird zur Wurzel,
    Commits, die ihn nicht berühren, entfallen (ein Commit, der ihn anlegt, hat keinen
    Elternteil), Merges, die nichts mehr zusammenführen, werden vereinfacht, und Branches und
    Tags, die nichts von dem Ordner haben, werden gelöscht. Es wird **jedes Mal** gefragt, wie es
    endet:
    - *In einen neuen Klon, geöffnet in einem neuen Tab* (`--extract-mode clone`): Von diesem
      Repository wird in einem Ordner Ihrer Wahl eine Kopie angelegt (das neue Repository
      bekommt darin einen Ordner mit dem Namen des herausgelösten), dort umgeschrieben und so
      bereinigt, dass es nur den neuen Verlauf enthält, mit jedem lokalen Branch und Tag, der
      den Ordner hat. Dieses Repository bleibt unberührt. Das neue Repository hat kein Remote
      (das Original ist nicht sein Upstream).
    - *An Ort und Stelle: Der Verlauf dieses Repositorys wird umgeschrieben* (`--extract-mode
      in-place`): das übliche Umschreiben mit Vorschau, Anwenden und Rückgängig. Branches ohne
      den Ordner werden gelöscht (Rückgängig bringt sie zurück); der ausgecheckte Branch muss
      ihn haben.
    - *Als Submodul dieses Repositorys* (`--extract-mode submodule`): Das neue Repository wird
      wie beim Klon angelegt, dann ersetzt **ein neuer Commit** auf dem aktuellen Branch den
      Ordner durch ein Submodul davon (`.gitmodules` und der Verweis); der Ordner bleibt als
      ausgechecktes Submodul bestehen. Der Verlauf dieses Repositorys wird **nicht**
      umgeschrieben: Seine alten Commits behalten die Dateien des Ordners, sodass nichts kaputtgeht,
      was auf sie zeigt (Tags, andere Klone). Die URL in `.gitmodules` ist der lokale Pfad des
      neuen Repositorys: Sie ist zu ändern (`git config -f .gitmodules
      submodule.<Ordner>.url`), sobald es auf einem Host liegt. Bearbeitete oder unversionierte
      Dateien im Ordner lehnen es ab. Rückgängig bringt den einfachen Ordner zurück.

    Die Zusammenfassung sagt, dass die Vorschau den Verlauf des neuen Repositorys zeigt und in
    welchen Ordner es geht. Regeln nach einem Herauslösen (ein Umbenennen, ein Entfernen) wirken
    auf den herausgelösten Baum; bei einem Klon oder Submodul muss das Herauslösen die erste
    Pfadregel sein. Pfadregeln laufen nach den Inhaltsregeln, ein Glob zum Entfernen nennt die
    Pfade also so, wie sie waren.

#### Zeilenenden, Ausführbar-Bit und Dateikodierungen

Drei Regeln in der Gruppe **Inhalt** des Menüs „Regel hinzufügen“ (oder die Palettenzeilen
*Umschreiben: Zeilenenden umwandeln...*, *Ausführbar-Bit entfernen...* und *Umschreiben:
Dateikodierung umwandeln...*). Alle drei nehmen **Dateien** (Globs; ein Muster ohne Schrägstrich
passt in jeder Tiefe; leer heißt jede Datei) und **Ausgenommene Dateien**. Sie ändern nur, was die
Globs nennen, in jedem Commit des Bereichs. Binärdateien (ein NUL-Byte in den ersten 8 KiB)
bleiben unangetastet: Die Zusammenfassung zählt sie als *unverändert gelassen*.

- **Zeilenenden umwandeln** (`--eol lf|crlf|cr --eol-paths GLOB... [--eol-exclude GLOB...]`).
  *Umwandeln in*: LF (Linux, macOS), CRLF (Windows) oder CR (klassisches Mac). Jedes Zeilenende
  einer passenden Textdatei wird zu diesem, sodass eine Datei mit gemischten Zeilenenden
  einheitlich wird; Dateien, die schon stimmen, behalten ihre Version unberührt.
  *.gitattributes beachten* (standardmäßig aus, `--eol-follow-attributes`) liest die
  `.gitattributes` an der Wurzel jedes Commits: Dateien mit `-text` oder `binary` werden
  übersprungen, und Dateien mit `eol=lf` oder `eol=crlf` behalten dieses Zeilenende statt des
  gewählten.
- **Ausführbar-Bit entfernen** (`--strip-exec GLOB... [--strip-exec-exclude GLOB...]`). Passende
  Dateien gehen vom Modus 100755 auf 100644; ihr Inhalt bleibt unberührt. Mindestens ein
  Dateimuster ist Pflicht: Würde man es allen ausführbaren Dateien nehmen, gingen Ihre Skripte
  kaputt.
- **Dateikodierung umwandeln** (`--reencode TARGET [--from SOURCE] [--encoding-fallback ENC]
  --encoding-paths GLOB... [--encoding-exclude GLOB...]`). *Umwandeln in*: UTF-8 (Standard),
  UTF-8 mit BOM, UTF-16 LE oder BE (mit Byte-Order-Mark geschrieben), Windows-1252 oder
  ISO-8859-1. *Dateien liegen vor in*: Standardmäßig wird die Kodierung jeder Datei erkannt (ein
  Byte-Order-Mark, sonst gültiges UTF-8, sonst der *Ersatz*, Windows-1252, wenn Sie nichts
  ändern); wer die Kodierung nennt, erzwingt sie, aber eine falsche Wahl ergibt falschen Text
  (UTF-8 als Latin-1 gelesen verdoppelt jeden Akzent). Eine Datei, die die Quellkodierung nicht
  lesen kann oder die ein Zeichen enthält, das das Ziel nicht speichern kann (ein Euro-Zeichen in
  ISO-8859-1), bleibt genau, wie sie ist, und wird in der Zusammenfassung gezählt.

Die Regeln laufen in der Reihenfolge der Liste, an derselben Datei: Eine Datei erst nach UTF-16
umzuwandeln und dann Zeilenenden zu verlangen, lässt sie unberührt (sie ist dann kein einfacher
Text mehr). Das Anwenden ist ein Eintrag in der Rückgängig-Liste wie jedes Umschreiben.

### Geheimnisse finden

- **Geheimnisse finden:** `gitpoulpi rewrite scan-secrets` (und das MCP-Werkzeug
  `rewrite_scan_secrets`) liest den Verlauf einmal und listet Token-Präfixe (GitHub, GitLab,
  Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic), private Schlüssel, JWTs, Passwörter in
  URLs und zufällig aussehende Zeilen der Art `password = value`, jeweils mit Datei, erstem und
  letztem Commit und einer geschwärzten Vorschau wie `ghp_****abcd`. Die Suche ist nicht
  vollständig, und der Wert wird nie angezeigt, geloggt oder gespeichert. `--remove-finding ID`
  macht aus einem Treffer eine Regel, die ihn aus jedem Commit entfernt. Ein gepushtes Geheimnis
  gilt als offengelegt: zuerst rotieren, dann umschreiben, force-pushen und bereinigen.
- **Der Tab Geheimnisse** erledigt dasselbe in der Werkbank, für den Bereich der Bereichsleiste:
  **Nach Geheimnissen suchen** (`s`) startet die Suche als Job (der Fortschrittsbalken heißt
  *Suche läuft*, **Abbrechen** oder `Esc` stoppt sie). Jeder Treffer ist eine Zeile mit Art,
  geschwärzter Vorschau, Datei und der Zahl weiterer Dateien, die ihn enthalten; unter der Liste
  stehen der erste und der letzte Commit, in dem er auftauchte, und wie viele Commits und
  Dateiversionen ihn enthalten. Der Tab nennt, wie viele Binär- und übergroße Dateien nicht
  gelesen wurden, und dass die Suche nicht vollständig ist. Alle Treffer sind zu Beginn
  angekreuzt; **Aus dem Verlauf entfernen** (`r`) macht aus den angekreuzten eine einzige Regel,
  die sie über ihre ID nennt (nie über den Wert) und die Sie wie jede andere in der Vorschau
  prüfen und anwenden. Treffer veralten nach einem Umschreiben oder einem Bereichswechsel: Der
  Tab sagt es, dann erneut suchen. Die Werkbank hält den Wert eines Geheimnisses nie, kann ihn
  also auch nicht anzeigen.

### Herausfinden, was wiegt

- **Herausfinden, was wiegt:** `gitpoulpi rewrite analyze [--top N]` (und das MCP-Werkzeug
  `rewrite_analyze`) ist die Größenanalyse, wie `filter-repo --analyze`: die größten Dateien mit
  Pfad, Größe, erstem und letztem Commit und der Angabe, ob sie noch in HEAD sind, die größten
  Ordner, die aus HEAD gelöschten Dateien, die der Verlauf noch speichert, und die Summen. Sie
  liest nur Größen und Pfade, nie Dateiinhalte, und ändert nichts.
- **Der Tab Größe** erledigt dasselbe in der Werkbank (`F7` oder *Größe des Verlaufs
  analysieren* in der Palette öffnet ihn und startet die Analyse): **Größen analysieren** (`s`)
  ist ein Job (*Analyse läuft*, **Abbrechen** oder `Esc` stoppt ihn). **Zeigen** wählt die Liste
  (`v`): *Dateien* (die größten eindeutigen Dateiversionen, jede markiert als *in HEAD*, *alte
  Version* einer Datei, die noch da ist, oder *gelöscht*), *Ordner* (alles, was je darunter
  gespeichert wurde) oder *Gelöscht* (Pfade, die aus HEAD verschwunden sind und die der Verlauf
  behält). **Sortierung** (`o`) geht nach Größe, nach Pfad oder gelöschte zuerst. Zeilen
  ankreuzen (`Space`, `Alt+A` / `Alt+U` für alle / keine): Der Tab nennt, wie viel sie enthalten,
  ehrlich als „bis zu etwa“, denn der Platz kommt erst nach dem Umschreiben, einer Bereinigung
  der alten Objekte und einem `gc` zurück. **Aus dem Verlauf entfernen** (`r`) macht aus den
  angekreuzten Pfaden eine Pfadregel (Sonderzeichen werden maskiert, ein Pfad, der schon in einer
  Regel steht, kommt nicht doppelt hinzu); **Alle Dateien über ... entfernen** (`b`, Dateiliste)
  schreibt eine Größenregel knapp unter der kleinsten angekreuzten Datei. Beide in der Vorschau
  prüfen und anwenden wie jede andere Regel. Die Listen zeigen die 200 größten Zeilen; die CLI
  nennt, wie viele weitere es gibt, und `--top` listet sie. Zeilen veralten nach einem
  Umschreiben oder einem Bereichswechsel: erneut analysieren.

### Signaturen

- **Signaturen:** Ein umgeschriebener Commit kann seine Signatur nicht behalten (sie galt für
  die alten Bytes), darum wird bei signierten Commits jedes Mal gefragt: *Signaturen entfernen*
  (die Zahl steht zuerst da), *Nur die signierten Commits neu signieren* oder *Jeden
  umgeschriebenen Commit neu signieren*. Es gibt keine Standardantwort. Commits, die unverändert
  bleiben, behalten ihre. Das Neusignieren nutzt Ihre eigene Einrichtung, so wie ein signierter
  Commit es tut: `gpg.format` (`openpgp`, `ssh` oder `x509`), `user.signingkey` und
  `gpg.program` / `gpg.ssh.program` / `gpg.x509.program`. Die vierte Zeile der Frage,
  **Signierung prüfen** (`4`), signiert einmal eine Testnutzlast und verifiziert sie: Sie sagt,
  wie lange eine Signatur dauert, und schätzt den Lauf für die Zahl der Commits, um die es geht,
  oder nennt, warum es nicht funktioniert (Schlüssel fehlt, Agent gesperrt, Programm nicht
  installiert, mit dem Hinweis, wie man es installiert). Die Vorschau macht denselben Test, bevor
  sie den Verlauf liest, und lehnt mit diesem Grund ab, sodass eine kaputte Einrichtung nie ein
  halb signiertes Umschreiben hinterlässt. Die Commits werden **während der Vorschau** signiert
  (ihre IDs enthalten die Signatur), eine Vorschau, die neu signiert, braucht also die
  Signierzeit, die Zusammenfassung sagt *Neu signierte Commits: n, in 12.5 s*, und dabei kann
  eine Passphrase-Abfrage erscheinen: den Agenten vorher entsperren (Signierung prüfen tut genau
  das). **Abbrechen** hält zwischen zwei Signaturen an und hinterlässt nichts. Nur
  Commit-Signaturen werden neu erstellt; ein signierter Tag verliert seine Signatur weiterhin.
- **Die Regel *Signaturen entfernen*** (Menü „Regel hinzufügen“, Gruppe *Signaturen*;
  *Umschreiben: Signaturen entfernen...* in der Palette; `--remove-signatures`) entfernt die
  Signatur jedes Commits im Bereich, auch wo sich sonst nichts ändert, und fragt nichts zum
  Neusignieren. Auf der CLI: `rewrite check-signing` und
  `--signatures strip|resign-signed|resign-all`.

### Ein Umschreiben rückgängig machen

**Rückgängig** (die Schaltfläche im Ergebnisfenster, *Rückg.* in der Symbolleiste mit `Ctrl+Z`
im Log, oder `gitpoulpi undo`) setzt jeden verschobenen Branch, Tag und jede Notiz zurück, samt
den ausgecheckten Dateien, aus den alten Commits, die das Umschreiben festhält. Es fragt vorher,
wenn ein umgeschriebener Branch schon auf seinem Upstream liegt (lokal rückgängig zu machen
macht keinen Push rückgängig) und wenn Sie nicht committete Änderungen haben: Die bleiben
erhalten, und eine von Ihnen geänderte Datei, die der alte Verlauf überschreiben würde, hält das
Rückgängig stattdessen an (Dateien bewegen sich so wie bei `reset --keep`). Rückgängig ist für
ein Umschreiben nicht mehr möglich, sobald Sie seinen alten Verlauf
[bereinigen](#den-alten-verlauf-löschen-bereinigen); die Verlaufsliste markiert es dann als
*bereinigt*.

## Dateien nach Git LFS verschieben oder zurückholen

Zwei Regeln in der Gruppe *LFS* des Menüs „Regel hinzufügen“ schreiben den Verlauf so um, dass
große Dateien von Anfang an in [Git LFS](lfs.md) liegen (oder es verlassen). Sie brauchen das
Programm `git-lfs`: Ohne es sind die Regeln mit dem Installationshinweis ausgegraut, und eine
Vorschau wird abgelehnt. Es ist unsere eigene Engine, nicht `git lfs migrate`, sodass Sie
Vorschau, Bereich, Rückgängig und den Push-Schutz jeder anderen Regel bekommen.

- **Dateien zu LFS migrieren** (*Umschreiben: Dateien zu LFS migrieren...* oder `--lfs-import
  PATTERN` mit `--lfs-min-size BYTES`). Jede Datei, die zu den Mustern passt (`*.psd`,
  `assets/**`; ein Muster ohne Schrägstrich passt in jeder Tiefe) und mindestens so groß ist wie
  *Nur Dateien ab* (leer: jede passende), wird in jedem Commit, der sie enthält, zu einem
  LFS-Zeiger. Ihr Inhalt kommt beim Anwenden in den lokalen LFS-Speicher
  (`.git/lfs/objects`, einmal pro verschiedenem Inhalt), und die `.gitattributes` an der Wurzel
  erhält in jedem umgeschriebenen Commit, der einen Zeiger enthält, die Zeile `<Muster>
  filter=lfs diff=lfs merge=lfs -text` (angelegt, wenn es keine gibt, die übrigen Zeilen
  unberührt, nie doppelt). Eine Datei, die schon ein Zeiger ist, bleibt unberührt. Die
  Zusammenfassung sagt *3 Dateien (12 MiB) wandern nach LFS. Zu schreibende LFS-Objekte: 3.*, und
  der Tab Commits listet pro Commit *verschiebt big.psd nach LFS*.
- **Dateien aus LFS zurückholen** (*Umschreiben: Dateien aus LFS zurückholen...* oder
  `--lfs-export PATTERN`). Passende Zeiger werden wieder zu den echten Dateien, und die Zeilen in
  `.gitattributes` für diese Muster entfallen. Die Objekte müssen im lokalen LFS-Speicher liegen:
  Fehlen welche, wird die Vorschau mit der Liste abgelehnt, und eine Schaltfläche **LFS-Objekte
  herunterladen** holt sie von `origin`; danach die Vorschau erneut erstellen. Ein Import,
  gefolgt von einem Export derselben Muster, ergibt genau dieselben Commits zurück.

Nach dem Anwenden sind die ausgecheckten Dateien der echte Inhalt (keine Zeiger), und
**Rückgängig** bringt den alten Verlauf zurück. Das Ergebnisfenster hat **LFS-Objekte senden**
(`l`): dasselbe wie `LFS: Push (alle Objekte)`, damit der neue Verlauf gepusht werden kann, ohne
dass ein späterer Push an fehlenden Objekten hängt (ein einfacher Push der umgeschriebenen
Branches lädt sie auch hoch, über den Pre-Push-Hook, den das Anwenden installiert). Der Hinweis
für Mitwirkende sagt, dass sie git-lfs und `git lfs pull` brauchen.

Gut zu wissen: Das Anwenden installiert die LFS-Filter des Repositorys (`git lfs install
--local`), wenn sie fehlen, weil Git sie braucht, um die echten Dateien mit den Zeigern zu
vergleichen. Geschrieben wird nur die `.gitattributes` an der Wurzel, nie eine verschachtelte.
Eine Datei *unter* der Mindestgröße, die zu einem Muster passt, bleibt eine einfache Datei, doch
Git will sie beim nächsten Hinzufügen in einen Zeiger verwandeln, sodass `git status` sie als
geändert zeigt, bis Sie sie committen. Der Import streamt große Dateien, der Export hält jeweils
eine Datei im Speicher.

## Das Ergebnis pushen

Das Anwenden eines Umschreibens pusht nie. Umgeschriebene Branches und Tags, die auf einem
Remote lagen, brauchen einen Force-Push, und der Push erzwingt nie blind: Jede Ref wird mit
einem Lease auf die ID gesendet, die das Remote bei der Vorschau hatte (nicht auf das, was Ihr
Tracking-Branch jetzt sagt). Ein Push eines Kollegen dazwischen wird für diese Ref abgelehnt,
statt überschrieben zu werden, und die Refs gehen in einem einzigen Alles-oder-nichts-Push
(`--atomic`), wo der Server es erlaubt. Auf der Befehlszeile:

```
gitpoulpi rewrite push --dry-run        # fragt das Remote, sendet nichts
gitpoulpi rewrite push --yes            # sendet
gitpoulpi rewrite note                  # was die anderen ausführen müssen
gitpoulpi rewrite maps --out ./maps     # commit-map und ref-map
```

Geschützte Branches (Einstellung `rewrite.protected_branches`, Standard `main`, `master`,
`develop`, `release/*`) bleiben beim Push außen vor, beim Umschreiben und erneut beim Push,
außer Sie nennen sie (`--include-protected main`) oder nehmen alle dazu. Ob der Host einen
Force-Push annimmt, entscheidet weiterhin sein eigener Branch-Schutz; seine Ablehnung wird
pro Ref angezeigt.

In der Werkbank erledigt das Ergebnisfenster nach einem Anwenden all das:

- **Umgeschriebene Refs an das Remote senden** listet die Refs, die das Umschreiben bewegt
  hat und die ein Remote hat, je mit einem Kästchen (`Space`). Geschützte Branches beginnen
  nicht angekreuzt, mit *geschützt* daneben: einzeln ankreuzen oder *Alle markieren* /
  *Keine markieren* (`Alt+A` / `Alt+U`) verwenden. **Umgeschriebene Branches pushen** (`p`)
  sendet die angekreuzten; die Antwort ist eine Zeile pro Ref (*gepusht*, *bereits aktuell*,
  *abgelehnt: Das Remote hat sich seit der Vorschau geändert*, *abgelehnt: vom Host
  geschützt*, *übersprungen: geschützter Branch*, *nicht gepusht*). Eine abgelehnte Zeile
  hat **Fetch und prüfen** (`f`): Die Werkbank schließt sich und die Remotes werden
  gefetcht, sodass das Log zeigt, was die anderen gepusht haben. Nichts wird überschrieben:
  Das Lease schützt deren Arbeit, und Sie erstellen die Vorschau neu, bevor Sie es erneut
  versuchen.
- **Hinweis kopieren** (`c`) legt den Hinweis an Mitwirkende in die Zwischenablage, in der
  Sprache der Oberfläche um die unübersetzten Git-Befehle herum.
- **Commit-Zuordnung öffnen** (`m`) zeigt, wo die Commit-Map und die Ref-Map liegen; ein
  Klick auf einen Pfad (oder `1` / `2`) kopiert ihn.

Ein Push lässt sich nicht rückgängig machen: Wenn Sie das Umschreiben danach mit `undo`
zurücknehmen, ändert sich nur Ihr Repository, und das Rückgängig sagt es, bevor es fragt.
Den Hinweis an alle schicken, die den alten Verlauf haben: Er listet die alte und die neue
Spitze jedes Branches und Tags und die Befehle (`git fetch`, dann
`git reset --hard origin/<branch>` oder `git rebase --onto`). Die beiden Maps werden mit dem
Lauf 24 Stunden aufbewahrt; sie vorher herauskopieren, falls die Repository-Bereinigung von
GitLab oder ein Skript sie braucht.

## Wenn der Host einen Branch schützt

Ihr Host kann einen Branch selbst schützen (GitHub-Branch-Schutz und Rulesets, GitLab Protected
Branches, Azure-DevOps-Branch-Richtlinien, Bitbucket-Branch-Einschränkungen) und den Force-Push
ablehnen, den ein Umschreiben braucht. Ist das Repository mit seinem Host **verknüpft** (der
Bereich der Pull Requests in der Seitenleiste, siehe [Hosting-Integration](hosting-integration.md)),
fragt die Werkbank den Host mit Ihrem eigenen Token, welche Branches er schützt:

- Diese Branches werden **standardmäßig vom Umschreiben ausgenommen**, genau wie die geschützten
  Branches aus `rewrite.protected_branches` und aus dem Workflow: Der Tab Refs listet sie mit je
  einem Kästchen und den Schaltflächen *Alle markieren* / *Keine markieren* auf und nennt den
  Grund: *geschützt auf GitHub*.
- Kann das Token den Schutz nicht lesen (ihm fehlt die Berechtigung, oder Sie sind kein
  Administrator), sagt die Werkbank *Schutz auf dem Host unbekannt* und behandelt nichts auf das
  Wort des Hosts hin als geschützt. Das Umschreiben wird dadurch nie blockiert. Ein Repository,
  das nicht verknüpft ist, wird nie gefragt.

Steht ein geschützter Branch im Weg (die Vorschau wurde abgelehnt, weil jeder Branch geschützt
ist, oder der Host hat den Push abgelehnt: *abgelehnt: vom Host geschützt*), erklärt die
Schaltfläche **So geht es weiter** (`Alt+H`) die drei Auswege, mit der genauen Einstellungsseite
Ihres Anbieters und Repositorys (einen Link wählen und mit `c` kopieren oder mit `o` öffnen).
GitPoulpi erledigt keinen davon für Sie:

1. **Den Schutz vorübergehend aufheben**, in den Einstellungen des Hosts, pushen, dann ihn
   wieder einschalten.
2. **Das Umschreiben unter einem neuen Branch-Namen pushen**, dann diesen Branch zum
   Standard-Branch machen (oder umbenennen) und den alten löschen, sobald ihn niemand mehr
   nutzt.
3. **Als letzten Ausweg das Remote-Repository löschen und neu anlegen** und alles pushen. Dabei
   gehen die Pull / Merge Requests mit ihren Reviews verloren, die Issues, die Verknüpfung der
   Forks mit dem alten Verlauf, Stars und Watcher, die CI-Einstellungen, Secrets und Variablen
   sowie die Webhooks und Deploy Keys.

Auf der CLI kommt derselbe Text als `protected_by_host` im JSON von `rewrite preview` /
`rewrite run` zurück (wenn der Host einen Branch des Bereichs schützt), von einem abgelehnten
`rewrite push` und von `rewrite note`: `provider`, `branches`, `ways` (jeweils mit `steps` und
`links`), `recreate_loses` und die ganze Erklärung als `text`.

## Den alten Verlauf löschen (Bereinigen)

Nach einem Umschreiben liegen die alten Commits noch in Ihrem Repository: Das Rückgängig des
Umschreibens hält sie fest (das macht es erst rückgängig-fähig), ebenso die Reflogs. Wenn Sie
umgeschrieben haben, um ein Geheimnis loszuwerden, steckt es noch im Ordner `.git`. Das
Bereinigen löscht den alten Verlauf endgültig:

```
gitpoulpi rewrite purge                 # fragt: listet, was gelöscht wird und verloren geht (Exit-Code 3)
gitpoulpi rewrite purge --yes           # führt es aus
```

Es entfernt die Undo-Pins des Umschreibens, lässt die Reflog-Einträge ablaufen, die die alten
Commits nennen, führt `git gc --prune=now` aus und prüft dann, ob jeder alte Commit und jede
alte Dateiversion wirklich weg ist. Die Bestätigung nennt vorher, was Sie verlieren:
**Rückgängig ist nicht mehr möglich** für das Umschreiben (und für jede frühere Operation,
deren Rückgängig die alten Commits ebenfalls brauchte); sie bleiben in der Verlaufsliste,
als bereinigt markiert. Eine Bereinigung lässt sich nach dem Start weder anhalten noch
rückgängig machen. Ein Remote fasst sie nicht an.

Alles, was den alten Verlauf noch hält, blockiert die Bereinigung, und dann wird nichts
geändert. Die Liste nennt jeden Fall mit dem, was zu tun ist: einen Branch oder Tag, der nicht
zum Umschreiben gehörte, einen Stash auf dem alten Verlauf (verwerfen, oder behalten und
hinnehmen, dass der alte Verlauf bleibt), einen anderen Worktree, dessen HEAD darauf liegt,
und die Remote-Tracking-Branches, die noch auf den alten Verlauf zeigen (die umgeschriebenen
Branches pushen, dann fetchen). Ihre Stash-Liste und die Reflogs anderer Branches werden nie
angefasst.

Ein gepushtes Geheimnis gilt als offengelegt: **zuerst rotieren**. Nach dem Bereinigen listet
die Antwort pro Remote, was der Host noch braucht: GitHub hält alte Commits in
zwischengespeicherten Ansichten und in den Refs der Pull Requests erreichbar (den GitHub-Support
kontaktieren oder das Repository löschen und neu anlegen), GitLab hat eine
Repository-Bereinigung, die die `commit-map` (`gitpoulpi rewrite maps --out DIR`) plus die
Wartung braucht, Azure DevOps und Bitbucket brauchen eine Support-Anfrage oder ein neues
Repository, und alle mit einem Klon oder einem Fork haben noch den alten Verlauf.

### Der Bildschirm zum Bereinigen

In der Werkbank hat das Ergebnisfenster eines Umschreibens die Schaltfläche **Alten Verlauf
bereinigen...** (Taste `e`), und die Palette hat **Umschreiben: Alten Verlauf bereinigen...**
(sie wirkt auf das neueste Umschreiben, das noch nicht bereinigt ist, auch nach dem Schließen
der Werkbank). Der Bildschirm listet, was das Bereinigen löscht und was Sie verlieren (die
Vorgänge, deren Rückgängig mit verschwindet; die mit *verliert auch sein Rückgängig*
markierten sind keine Umschreiben, brauchten aber die alten Commits), dann, was den alten
Verlauf noch hält, je eine Zeile:

- ein Remote-Tracking-Branch: **Fetch** aktualisiert sie von den Remotes;
- ein Stash: **Verwerfen** fragt vorher, denn ein Stash ist Arbeit, die Sie nie committet
  haben;
- ein Branch, Tag, Worktree oder eine andere Ref: **Name kopieren**, dann dort erledigen, wo
  sie liegt (den Branch umschreiben oder löschen, im Worktree einen umgeschriebenen Branch
  auschecken).

Die Prüfung läuft nach jeder Korrektur erneut, und **Erneut prüfen** (`r`) startet sie auf
Anforderung. **Bereinigen** (`p`) ist nur verfügbar, wenn nichts mehr übrig ist. Es stellt eine
letzte Frage, nummeriert und ohne Standardantwort (`1` bereinigt endgültig, `2` lässt alles,
wie es war), und läuft dann; der Bildschirm zeigt den aktuellen Schritt und sagt, dass es sich
nicht anhalten lässt. Das Ergebnis sagt, dass der alte Verlauf aus diesem Klon verschwunden
ist, oder wie viele alte Objekte noch da sind. Darunter stehen die Schritte für jedes Remote
(ein Link lässt sich kopieren). Das Kreuz, `Esc` und ein Klick daneben schließen den
Bildschirm, außer während die Bereinigung läuft.

Über MCP gibt es das Bereinigen-Werkzeug nur, wenn Sie `allow_history_purge = true` in der
Tabelle `[mcp]` der `config.toml` setzen.

## Die Skriptregel

*Regel hinzufügen > Skript* (oder **Umschreiben: Skript ausführen...** in der Palette) fügt eine
Lua-Skriptregel hinzu und öffnet ihren Editor über der Werkbank; `Enter` auf der Zeile
*Skript* der Regel öffnet ihn erneut. Der Editor beginnt mit einer kleinen Vorlage.

- **Sprache** (eine Zeile im Bereich der Regel): *Lua (eingebaut)*, *Shell-Skript*, *Python*
  oder *Lua (installiert)*. Alles außer dem eingebauten Lua ist ein
  [externes Skript](#externe-skripte-shell-python-lua): ohne Sandbox, und eine Sprache, die in
  den Einstellungen aus ist oder deren Interpreter fehlt, ist in der Liste ausgegraut, mit dem
  Grund daneben (beim Wählen steht, was zu tun ist). Wer die Sprache eines Skripts wechselt, das
  Sie noch nicht geschrieben haben, tauscht die Vorlage aus.
- **Skript** ist ein Texteditor mit Zeilennummern. Was Sie tippen, ist sofort die Regel; es gibt
  nichts zu bestätigen.
- Darunter sagt die **Prüfzeile** `OK: rewrite_commit gefunden` oder listet die Probleme mit
  ihrer Zeile (`script:3: ...`; die Nummer ist am Rand rot). Sie läuft einen Moment, nachdem
  Sie aufgehört haben zu tippen, nie während eine Taste gehalten wird.
- **Parameter** hat eine Zeile pro `--! param`-Zeile im Kopf des Skripts, mit ihrer Beschriftung
  neben dem Feld: Text, eine Zahl oder ein Kästchen für wahr/falsch. Ein geleerter Wert kehrt
  zum Standard aus dem Kopf zurück; ein Textparameter ohne Standard muss vor der Vorschau
  ausgefüllt sein.
- **Datei** (ein Pfad innerhalb des Repositorys, zum Start `.gitpoulpi/rewrite/script.lua`) mit
  **Aus Datei laden...** (ersetzt den Text) und **In Datei speichern...**.
- **Auswahl testen** führt das Skript an den im Log ausgewählten Commits aus, oder an den 20
  neuesten, wenn keiner ausgewählt ist, und schreibt nichts. Die Tabelle listet jeden Commit
  (kurze ID), jedes Feld, das er ändern würde, als `alt -> neu`, und die `log(...)`-Zeilen des
  Skripts. **Abbrechen** hört auf zu warten. Schlägt das Skript fehl, werden Commit und Grund
  angezeigt.
- Das Kreuz, ein Klick außerhalb des Editors und `Esc` schließen ihn; `Tab` und `Shift+Tab`
  wechseln zwischen Editor, Parametern, Datei, Schaltflächen und Ergebnissen; `F5` testet.

Die Vorschau lehnt ein Skript ab, das sich nicht kompilieren lässt: Der Vorschaubereich sagt
*ein Skript ist fehlgeschlagen* mit der Zeile (oder, bei einem Fehler an einem Commit, mit
dessen kurzer ID und dem Grund). In einem Build ohne Skripte bleibt die Art im Menü
„Regel hinzufügen“, ausgegraut.

## Ein Skript mit KI schreiben

Ist die KI eingeschaltet ([KI-Unterstützung](ai.md)), hat der Skript-Editor die Schaltfläche
`✦ Mit KI schreiben...`, und die Palette hat `✦ Umschreiben: Skript mit KI schreiben...`, was die
Werkbank mit einer neuen Skriptregel und geöffnetem Dialog öffnet. Vor dem Drücken von
**Senden** wird nichts gesendet: Der Dialog zeigt beim Tippen den genauen Text, der hinausgeht.

- **Was soll das Skript tun?** Ihre eigenen Worte: *jeden nachts erstellten Commit auf den
  nächsten Morgen verschieben*, *meine alte E-Mail-Adresse durch die neue ersetzen, nur vor
  2024*.
- **Eine Stichprobe aus N Commits senden** (aus): Wenn angekreuzt, werden Autor, Datum und
  Betreff der Commits ergänzt, die „Auswahl testen“ verwenden würde (Ihre Log-Auswahl, sonst die
  neuesten 20), damit das Modell sieht, wie Ihre Daten aussehen. Nie Dateiinhalte oder
  Commit-Nachrichten außer dem Betreff. Bei ausgeschaltetem Kästchen verlassen keine Commit-Daten
  den Rechner.
- **Was gesendet wird** ist ein schreibgeschütztes Feld auf etwas dunklerem Hintergrund, beim
  Öffnen des Dialogs eingeklappt (der Dialog ist dann kurz). Klicken Sie auf seine Kopfzeile,
  drücken Sie `Alt+W` oder wechseln Sie mit `Tab` dorthin und drücken Sie `Enter`, um es
  anzuzeigen: Es listet die Anweisungen (den festen Skriptvertrag, dann Ihre KI-Anweisungen) und
  Ihre Anfrage und hat eine eigene Bildlaufleiste (ziehen Sie sie, nutzen Sie das Mausrad über dem
  Feld oder, solange es den Fokus hat, die Pfeile, `PageUp`, `PageDown`, `Home` und `End`). Das
  Profil, das sie erhält, steht in der Zeile darüber. Der Dialog wächst mit dem Feld, im Rahmen des
  Bildschirms.
- **Senden** (`Tab` dorthin, `Enter`). Während das Modell arbeitet, wartet der Dialog, und `Esc`
  bricht die Anfrage ab. Schlägt sie fehl, bleibt der Dialog mit dem Grund offen, damit Sie es
  erneut versuchen können.

Die Antwort wird mit derselben Daemon-Prüfung wie ein getipptes Skript geprüft; ein
Kompilierfehler geht einmal an das Modell zurück. Das Skript **ersetzt dann den Text im Editor**
(der Dialog sagt es, wenn der Editor mehr als die Startvorlage enthält), und der Titel des Editors
lautet *✦ von der KI geschrieben: vor der Vorschau lesen*, bis Sie den Text ändern. Es wird nie
für Sie getestet, in der Vorschau gezeigt oder angewendet: lesen, **Auswahl testen** drücken, dann
die Vorschau wie bei jedem Skript. Das Skript läuft in derselben Sandbox wie ein selbst
geschriebenes (keine Dateien, keine Uhr, kein Netzwerk).

Auf der Befehlszeile: `gitpoulpi ai rewrite-script "<Anweisung>" [--sample N]` gibt das Skript
und das Ergebnis von `rewrite script check` dafür als JSON aus und führt nichts aus (Exit-Code 2,
wenn es sich noch immer nicht kompilieren lässt). Verwendet wird das Profil *Skript zum
Umschreiben des Verlaufs* aus Einstellungen > KI.

## Regeln mit KI erstellen

Ist die KI eingeschaltet ([KI-Unterstützung](ai.md)), hat die Werkbank neben Speichern und Laden
die Schaltfläche `✦ Regeln mit KI erstellen...` und die Taste `Alt+I` (*Umschreib-Regeln mit KI
erstellen* in Einstellungen > Tastatur & Maus, Kontext *Werkbank zum Umschreiben des Verlaufs*), und die
Palette hat `✦ Umschreiben: Regeln mit KI erstellen...`. Bei ausgeschalteter KI oder in einem
Build ohne KI fehlen die Schaltfläche und die Palettenzeile. Sie beschreiben in eigenen Worten, wie
der Verlauf umgeschrieben werden soll (*meine alte E-Mail a@b.org durch c@d.org ersetzen*,
*secrets.txt aus dem gesamten Verlauf entfernen*, *meine Datumsangaben um zwei Stunden
verschieben*), und das Modell schlägt die Regeln und Einstellungen vor, die **die Werkbank
füllen**. Vor dem Drücken von **Senden** wird nichts gesendet: Der Dialog zeigt beim Tippen den
genauen Text, der hinausgeht.

- **Wie soll der Verlauf umgeschrieben werden?** Ein mehrzeiliges Feld: `Enter` beginnt eine neue
  Zeile, `Ctrl+Enter` oder `Ctrl+S` sendet, `Esc` und das Kreuz brechen ab.
- **Umfang, Namen der Branches und Tags und die Autoren des Verlaufs mitsenden** (aus): Wenn
  angekreuzt, werden der Umfang der Bereichsleiste, die Namen der Branches und Tags und die
  verschiedenen Autoren (Name und E-Mail) der neuesten Commits, die das Log zeigt (höchstens 20
  Commits), ergänzt, damit das Modell Ihre E-Mail genau benennen kann (auf der Kommandozeile
  liest `ai rewrite-rules --context` stattdessen die neuesten 200 Commits). Nie Dateiinhalte
  oder Commit-Nachrichten. Bei ausgeschaltetem Kästchen verlassen nur Ihre Beschreibung und die festen
  Anweisungen den Rechner.
- **Aktuelle Regeln ersetzen** (nur, wenn die Werkbank schon Regeln enthält; aus): Standardmäßig
  werden die neuen Regeln nach Ihren vorhandenen angefügt; angekreuzt ersetzen sie diese (den
  Umfang und die Einstellungen aus der Antwort übernimmt die Werkbank in beiden Fällen).
- **Was gesendet wird** ist ein schreibgeschütztes Feld auf etwas dunklerem Hintergrund, abgesetzt
  von den Feldern darüber, und beim Öffnen des Dialogs **eingeklappt** (der Dialog ist dann kurz).
  Klicken Sie auf seine Kopfzeile, drücken Sie `Alt+W` oder wechseln Sie mit `Tab` dorthin und
  drücken Sie `Enter`, um es anzuzeigen: Es listet die Anweisungen (das Regelvokabular, dann Ihre
  KI-Anweisungen) und Ihre Anfrage und hat eine eigene Bildlaufleiste (ziehen Sie sie, nutzen Sie
  das Mausrad über dem Feld oder, solange es den Fokus hat, die Pfeile, `PageUp`, `PageDown`,
  `Home` und `End`). Das Profil, das sie erhält, steht in der Zeile darüber. Der Dialog wächst mit
  dem Feld, im Rahmen des Bildschirms. Das Beschreibungsfeld bekommt eine eigene Bildlaufleiste,
  wenn der Text es überragt.

Das Modell darf jede Regelart dieser Seite verwenden außer **Skript** (Code: *Ein Skript mit KI
schreiben* zeigt das Skript zur Prüfung) und der Form für geheime Werte von *In Dateien suchen und
ersetzen* (geheime Werte gehen nie an ein Modell: Nutzen Sie den Tab Geheimnisse). Es darf den
Umfang auf den aktuellen Branch, alle lokalen Branches und Tags oder benannte, vorhandene Branches
und Tags setzen, dazu die Optionen *Tags*, *Committer*, *Leere Commits* und *abgekürzte IDs in
Nachrichten*. Was die Werkbank nicht ausdrücken kann (eine Datumsgrenze wie *seit März*, ein
Commit-Bereich), wird nicht geraten: Das Modell sagt es in einer Anmerkung. Was mit Signaturen
geschieht, entscheidet nie das Modell: Sie werden beim Anwenden gefragt.

**Die KI bereitet nur Regeln und Einstellungen vor.** Sie zeigt nie eine Vorschau, wendet nichts
an und führt nichts aus: Die Antwort wird wie ein geladenes Rezept in die Werkbank übernommen, die
Vorschau auf dem Bildschirm ist dann veraltet, und Vorschau und Anwenden (mit ihren Fragen und
Bestätigungen) bleiben wie immer Ihre Sache. Jede Regel wird einzeln mit derselben Prüfung wie im
Editor und in der Engine gelesen und geprüft; eine Regel, die durchfällt, wird ausgelassen. Wurde
etwas ausgelassen oder hat das Modell eine Anmerkung gemacht, zeigt eine Liste es, sobald die
Regeln eingefügt sind (*Ausgelassen (die gültigen Regeln wurden behalten)*, jeweils mit Position,
Name und Grund), und `Enter` oder `Esc` schließt sie. Prüfen und bearbeiten Sie die Regeln, dann
**Vorschau**.

Auf der Befehlszeile: `gitpoulpi ai rewrite-rules "<Beschreibung>" [--context]` gibt den Vorschlag
als JSON aus (`recipe`, das `rewrite preview --recipe-json` liest, `problems`, `notes` und
`applied: false`) und führt nichts aus (Exit-Code 2, wenn etwas ausgelassen wurde). Das
MCP-Werkzeug ist `ai_rewrite_rules`. Verwendet wird das Profil *Regeln zum Umschreiben des
Verlaufs* aus Einstellungen > KI.

## Lua-Skripte

Eine Skriptregel führt Ihre Funktion für jeden Commit aus. Es ist reines Lua 5.4 in einer
Sandbox: keine Dateien, keine Uhr, keine Prozesse, keine Module, kein `pcall`; dasselbe
Skript auf demselben Verlauf liefert immer dasselbe Ergebnis, und ein Aufruf, der über
10 Millionen Anweisungen, 64 MiB oder 5 Sekunden hinausgeht, bricht mit einem Fehler ab,
der den Commit nennt.

```lua
--! name: Replace an email
--! param old_email: text "Email to replace"
--! param new_email: text "New email"
function rewrite_commit(c, ctx)
  if c.author.email == ctx.params.old_email then
    return { author = { email = ctx.params.new_email } }
  end
end
```

`c` enthält `id`, `parents`, `tree`, `author` und `committer` (`name`, `email`, `time`,
`offset_minutes`), `message` und `signed`. `ctx` enthält `index`, `total`, `params`, `state`
(mit `state = {...}` als Rückgabe lassen sich Daten an den nächsten Commit weitergeben) und
`window.before` / `window.after` (mit `--! window: N`). Zurückgegeben wird nur, was sich
ändert: `author`, `committer`, `message`. Hilfsfunktionen: `regex_replace`, `regex_match`,
`stable_hash`, `rand`, `weekday`, `hour`, `log` und für Nachrichten `trailers(message)` (eine Liste von `{key, value}`), `set_trailer(message, key, value)` (ersetzt die Trailer dieses Schlüssels; ein viertes Argument `"add"` oder `"skip"` ändert das), `remove_trailer(message, key)` und `at("2024-03-05 18:30", offset_minutes)` (ein Datum in einem Versatz als Unix-Zeit).

Auf der Befehlszeile: `gitpoulpi rewrite script check my.lua` kompiliert es,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b` zeigt, was es
ändern würde, ohne etwas zu schreiben, und
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b` plant das gesamte
Umschreiben.

## Externe Skripte (Shell, Python, Lua)

Derselbe Vertrag wie bei einem Lua-Skript, ausgeführt von einem Programm auf Ihrem Rechner:
*Shell-Skript* (bash oder sh; PowerShell unter Windows oder für jede `.ps1`), *Python* und
*Lua (installiert)* (ein eigenes `lua`, nicht das eingebaute; es hat keine JSON-Bibliothek,
daher liest die Vorlage die Zeile mit einem Muster). **Sie haben keine Sandbox: Ein externes
Skript läuft mit Ihren Rechten** und kann alles lesen oder ändern, was Sie auch können. Der
Bereich der Regel und der Titel des Editors sagen es.

So spricht es: GitPoulpi startet das Skript einmal pro Lauf und sendet ihm pro Zeile eine
JSON-Anfrage auf der Standardeingabe (die Felder eines Commits, `params`, der `state` aus der
letzten Antwort); es antwortet mit einer JSON-Zeile auf der Standardausgabe, mit denselben
Feldern, die ein Lua-Skript zurückgibt (`author`, `committer`, `message`, `state`, `log`), oder
mit `{"error": "text"}`, um aufzugeben. Diagnosen gehören auf die Standardfehlerausgabe: Sie
erscheinen als `log`-Zeilen des Commits. Die Kopfzeilen beginnen bei Shell und Python mit `#!`
(`#! name: ...`, `#! param old: text "Old"`; eine Shebang-Zeile bleibt eine Shebang-Zeile) und
bei Lua mit `--!`.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**Was es bekommt.** Eine leere Umgebung außer `PATH`, `HOME` / `USERPROFILE`, `SYSTEMROOT`,
`TEMP` / `TMP` und `LANG`; einen frischen temporären Ordner als Arbeitsordner (mit einer eigenen
Kopie des Skripts); nicht den Pfad des Repositorys. **Zeitlimits:** `rewrite.script_timeout_ms`
(5000) für jede Antwort und `rewrite.script_total_timeout_s` (3600) für den ganzen Durchlauf.
Wird eines überschritten, stürzt das Skript ab, gibt etwas aus, das kein JSON ist, oder wird der
Job abgebrochen, beendet GitPoulpi das Skript und alles, was es gestartet hat, und das
Umschreiben hält mit Commit und Grund an; nichts hat sich bewegt.

**Welche Sprachen es gibt.** `rewrite.external_scripts` in der `config.toml` listet sie:
standardmäßig `["shell"]`; `"python"` und `"lua"` selbst in der `config.toml` ergänzen
(`[rewrite]`, `external_scripts = ["shell", "python"]`; eine Liste lässt sich nicht mit
`gitpoulpi config set` setzen). Eine Sprache, die nicht in der Liste steht, ist in der Werkbank
ausgegraut und wird von CLI und MCP abgelehnt (`scripts_disabled`); eine gelistete, deren
Interpreter sich nicht finden lässt, ist mit dem Installationshinweis ausgegraut
(`tool_missing`).

**Vertrauen.** Auch wenn eine Sprache an ist, läuft ein Skript erst, nachdem Sie seinem genauen
Text vertraut haben. Ein Skript, das Sie in der Werkbank tippen, ist Ihres und läuft sofort; eines,
das zu einem geteilten Rezept oder einer Datei des Repositorys gehört, nicht: Die Vorschau und
*Auswahl testen* halten an und fragen *Diesem Skript vertrauen?* mit zwei nummerierten Antworten
und keiner vorgewählten (`1` vertrauen und fortfahren, `2` nicht, `Esc` lässt es). Das Vertrauen
merkt sich den SHA-256 des Texts für dieses Repository, ein geändertes Skript fragt also erneut,
und ein geklontes Repository kann nicht von selbst Code bei Ihnen ausführen. Auf der Befehlszeile
die Datei lesen und dann `gitpoulpi rewrite script trust FILE` (oder `--trust` bei `preview`,
`run`, `script check` und `script test`, was vertraut und weitermacht). Ein Agent über MCP kann
nichts als vertrauenswürdig einstufen: Er kann nur ein Skript ausführen, dem Sie vorher vertraut
haben.

**Prüfung.** Die Prüfzeile unter dem Editor sagt bei einem Skript ohne Vertrauen *Noch nicht als
vertrauenswürdig eingestuft* (nichts wurde gestartet); bei einem vertrauten startet GitPoulpi es
einmal mit einem erfundenen Commit (`"dry_run": true`) und prüft, ob die Antwort dem Vertrag
folgt. `gitpoulpi rewrite script check FILE` tut dasselbe und meldet `trust`, `hash` und
`interpreter`.

## Auf der Befehlszeile

`gitpoulpi rewrite` (und die MCP-Werkzeuge gleichen Namens) nutzt dieselbe Engine, für Skripte
und Agenten. Ein **Rezept** ist eine TOML- oder JSON-Datei, dieselbe, die die Werkbank speichert
(`--recipe FILE`, oder `-` für die Standardeingabe), und Flags fügen Regeln nach denen des Rezepts
hinzu.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # Plan, ändert nichts
gitpoulpi rewrite apply --run ID --digest D           # den geprüften Plan ausführen
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # setzt alles zurück
```

- Regeln als Flags: `--set-name` / `--set-email` mit `--match-email` (Identität), `--mailmap`,
  `--shift-dates`, `--replace-message FIND=WITH`, `--trailer-add`, `--trailer-remove`,
  `--trailer-map`, `--fix-message-encoding` (`--fix-name-encoding`), `--remove-path`,
  `--remove-blobs-over`, `--replace-content`, `--eol` (`--eol-paths`), `--strip-exec`,
  `--reencode` (`--encoding-paths`), `--move-path`, `--into-subdir`, `--extract`, `--lfs-import`,
  `--lfs-export`, `--script`, `--remove-signatures`, `--remove-finding`. Der Bereich ist der
  aktuelle Branch, `--all-local` oder `--ref R` (wiederholbar), ab `--since REV`.
- **Fragen ohne Standardantwort** sind Flags: `--signatures strip|resign-signed|resign-all`,
  wenn signierte Commits sich ändern, und `--extract-mode` für ein Herauslösen. Ohne sie ist die
  Antwort eine Ablehnung oder ein `needs_choice` (Exit-Code 3), nie eine Vermutung.
- **Exit-Codes:** 0 erledigt; 1 ein Fehler (nicht lesbares Rezept, falsche Flags); 2 abgelehnt,
  mit Grund (`stale`, `signature_choice_needed`, `dirty_paths_touched`, `empty_scope`, ...);
  3 eine Frage, die zu beantworten ist, oder gepushte Refs, die `--yes` brauchen.
- `preview` hält den Plan im Repository, sodass `apply` in einem späteren Befehl, nach einem
  Neustart des Daemons oder von einem anderen Frontend aus funktioniert; ein Plan, der nicht mehr
  zu seinem Digest passt, ist `stale`.

Die [CLI-Referenz](cli-reference.md) hat jedes Flag, und [MCP und Agenten](mcp-and-agents.md)
sagt, welche Werkzeuge ein Agent bekommt (das Bereinigen-Werkzeug gibt es nur, wenn Sie es
einschalten, und ein Agent kann einem Skript nicht vertrauen).

## Vorschläge von KI-Agenten

Ein Agent kann ein Umschreiben **vorschlagen**, nie ausführen. `gitpoulpi ui propose rewrite
--recipe FILE|-` (das MCP-Werkzeug `ui_propose_rewrite`, aus, bis `[mcp] allow_ui_propose = true`
gesetzt ist) öffnet die Werkbank in Ihrem laufenden Fenster mit den Regeln des Rezepts bereits in
der Liste und einem lila Banner, *✦ Vorgeschlagen von <Agent>: ein Umschreiben des Verlaufs (3
Regeln) · Ctrl+Enter übernimmt · Esc lehnt ab*.

- **Nichts läuft.** Keine Vorschau, kein Anwenden, kein Push, kein Bereinigen: Der Vorschlag füllt
  nur die Regeln. Sie lesen sie, ändern sie nach Belieben, erstellen die **Vorschau** und wenden
  mit Ihrer eigenen Taste an. Das Schließen der Werkbank lehnt den Vorschlag ab.
- **Was der Agent nicht hineinlegen darf:** ein Skript (es liefe bei der Vorschau), mehr als 50
  Regeln, geheime Werte (sie werden aus dem Rezept entfernt, bevor es gesendet wird). Ein solcher
  Vorschlag wird mit Grund abgelehnt.
- Der Agent kann das Ergebnis in `ui state` lesen (`accepted`, wenn Sie die Regeln wie
  vorgeschlagen angewendet haben, `accepted_edited`, wenn Sie sie vorher geändert haben,
  `rejected`, wenn Sie geschlossen haben). Es gibt immer nur einen Vorschlag; er wird abgelehnt,
  solange ein anderer Dialog von Ihnen offen ist.

## Programme

Das meiste beim Umschreiben des Verlaufs braucht nichts außer Git: Autoren und Committer ändern,
eine `.mailmap` anwenden, anonymisieren, Datumsangaben verschieben oder auf Uhrzeiten
beschränken, Nachrichten und Trailer umschreiben, Inhalte ersetzen, Pfade oder große Blobs
entfernen und Pfade verschieben läuft alles innerhalb von GitPoulpi.

**Lua-Skripte sind eingebaut.** Der Interpreter steckt in GitPoulpi, ein Lua-Skript braucht also
weder ein Programm noch eine Einrichtung. Shell-, Python- und Skripte mit installiertem Lua
brauchen ihren Interpreter (siehe die Tabelle unten) und sind unter [Externe
Skripte](#externe-skripte-shell-python-lua) beschrieben.

Einige Funktionen nutzen ein externes Programm. GitPoulpi prüft, ob es vorhanden ist (das
Programm wird im `PATH` gesucht, unter Windows auch über `PATHEXT` und das eigene `usr/bin` von
Git for Windows, und muss innerhalb von 2 Sekunden auf `--version` antworten); die Antwort wird
eine Minute zwischengespeichert, und *Erneut prüfen* fragt neu. Eine Regel, die ein fehlendes
Programm braucht, bleibt in der Liste, ausgegraut, mit dem Grund und einem Installationshinweis,
und die Vorschau lehnt ab, bevor irgendetwas läuft.

| Funktion | Programm | Hinweise |
|---|---|---|
| Commits neu signieren | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) oder `gpgsm` (`gpg.format = x509`) | SSH- und X.509-Signaturen brauchen außerdem `user.signingkey`. `gpg.program`, `gpg.ssh.program` und `gpg.x509.program` werden beachtet. |
| LFS-Migration (die beiden LFS-Regeln) | `git-lfs` 3.0 oder neuer | Dieselbe Prüfung wie im LFS-Bereich. |
| Python-Skripte | `python` 3.8 oder neuer (`py -3` unter Windows) | Standardmäßig aus: `"python"` zu `rewrite.external_scripts` hinzufügen. |
| Skripte mit installiertem Lua | `lua5.4`, `lua54`, `lua` oder `luajit` | Standardmäßig aus: `"lua"` zu `rewrite.external_scripts` hinzufügen. |
| Shell-Skripte | `bash`, `pwsh` (PowerShell) oder `sh` | Standardmäßig an (`"shell"`). Ein PowerShell-Skript (eine `.ps1`-Datei, ein `pwsh`-Shebang oder jedes Shell-Skript unter Windows) braucht `pwsh` oder `powershell`; ein POSIX-Skript braucht `bash` oder `sh` (unter Windows liefert Git for Windows sie). |

Installationshinweise, Windows zuerst:

- Windows: Git for Windows bringt `gpg`, `ssh-keygen`, `bash`, `sh` und Git LFS mit. Andernfalls
  `winget install GnuPG.GnuPG`, Einstellungen > Optionale Features > OpenSSH-Client, `winget
  install GitHub.GitLFS`, `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux: die Pakete der Distribution (`gnupg`, `openssh`, `git-lfs`, `python3`, `lua5.4`, `bash`,
  `powershell`).
- macOS: `brew install gnupg git-lfs python lua powershell`; `ssh-keygen` und `sh` sind
  eingebaut.
