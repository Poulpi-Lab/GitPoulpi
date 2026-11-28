Eine **Sequenz** ist eine benannte Liste von Befehlen, die nacheinander laufen:
zum Beispiel *Alle Änderungen stagen*, *KI: Commit-Nachricht erzeugen*, *Commit*,
*Push* als eine **Sicherung**-Taste. Jeder Schritt ist ein Befehl der
[Befehlspalette](keybindings.md); eine Sequenz ist eine eigene Palettenzeile
(`Sequenz: <Name>`), lässt sich an eine Taste binden und wird unter
**Einstellungen > Sequenzen** oder in `config.toml` bearbeitet.

## Die Seite Sequenzen

**Einstellungen > Sequenzen** ist eine [Listenseite](settings.md#listen): links die
Sequenzen, rechts die Zeilen der ausgewählten. Jede Änderung wird sofort angewendet
und geschrieben, und Palette und Tastenbelegung folgen ohne Neustart.

```
 Sequenzen
+- Sequenzen ------------+  Sequenz 1/2 wird bearbeitet
| Diese Installation      |  Name          Sicherung
| > Sicherung     Alt+B   |  ID            backup
|   Worktree und Agent    |  Beschreibung  Alles stagen, ...
| Dieses Repository       |  Geltungsber.  Diese Installation
|   Release               |  Tastenkürzel  [ Alt+B ] [ Entfernen ]
+-------------------------+  Schritte      Stagen > KI: ... > Commit > Push
| [+ Hinzufügen v] [Dupl.]|              [ Schritte bearbeiten (4) ] [ Jetzt ausführen ]
| [Entfernen] [Auf] [Ab]  |
+-------------------------+
```

* **+ Hinzufügen** bietet **Leere Sequenz** und die drei Vorlagen **Aus Vorlage >
  Sicherung**, **Zusammenstellen und Push** und **Worktree und Agent**. Eine Vorlage
  ist ein Ausgangspunkt: Nichts ist aktiv, bis Sie sie hinzufügen, und was Sie
  hinzufügen, ist eine Kopie, die Sie bearbeiten können. Vorlagen werden beim
  Hinzufügen in der Sprache der Oberfläche benannt.
* **Duplizieren**, **Entfernen** (fragt vorher), **Auf** / **Ab** oder Ziehen ordnen
  die Liste um (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`). Die Reihenfolge
  der Liste ist die Reihenfolge in `config.toml`.
* Eine Zeile mit einem Problem ist rot markiert und nennt den Grund (keine Schritte,
  ein Schritt, den diese Version nicht kennt, eine Pause `Nie`, der die nötige Option
  fehlt). Eine Sequenz mit Problemen **lässt sich speichern**, unbekannte künftige
  Schritte bleiben unverändert erhalten, aber ihre Palettenzeile bleibt abgedunkelt,
  bis das Problem behoben ist.

### Die Zeilen einer Sequenz

| Zeile | Was sie tut |
|---|---|
| **Name** | Was die Palette zeigt: `Sequenz: <Name>`. |
| **ID** | Der Kurzname in `config.toml`, auf der Kommandozeile und als Taste des Tastenkürzels (`a-z`, `0-9`, `_`, `-`). Beim Umbenennen zieht das Tastenkürzel im selben Schreibvorgang mit um. |
| **Beschreibung** | Optional, für Sie selbst. |
| **Geltungsbereich** | Siehe unten. |
| **Tastenkürzel** | Eine Schaltfläche mit der Taste. `Enter` oder ein Klick öffnet dieselbe Tastenerfassung wie **Tastatur & Maus**: Konflikte und Terminal-Warnungen erscheinen, während Sie drücken, und eine Taste, die eine andere Aktion nutzt, wird genauso abgelehnt. **Entfernen** löscht sie. Die Taste wird als `[keybindings.sequence] <id>` gespeichert. |
| **Schritte** | Die Schritte der Reihe nach. **Schritte bearbeiten** öffnet sie. |
| **Jetzt ausführen** | Führt die Sequenz im aktuellen Fenster aus. |

### Geltungsbereich

Jede Sequenz hat einen Geltungsbereich, der mit ihr gespeichert wird:

| Geltungsbereich | Wo sie gespeichert wird | Wer sie sieht |
|---|---|---|
| **Diese Installation** (Standard) | `[[sequences]]` in `config.toml`, mit dieser Installation als Eigentümer | Nur diese Installation von GitPoulpi. |
| **Alle Installationen auf diesem Computer** | `[[sequences]]` in `config.toml`, ohne Eigentümer | Jede Installation, die die Datei teilt. |
| **Nur dieses Repository** | `.gitpoulpi/sequences.toml` im aktuellen Repository | Alle, die das Repository haben. Sie zählt erst, wenn Sie ihr vertrauen (siehe unten). |

Ein Wechsel des Geltungsbereichs verschiebt die Definition: **Nur dieses Repository**
schreibt sie in die Datei des Repositorys und entfernt sie aus `config.toml`, und
umgekehrt. Ein Repository-Geltungsbereich setzt ein geöffnetes Repository voraus.

### Repository-Sequenzen und Vertrauen

Die Datei `.gitpoulpi/sequences.toml` kann committet werden und daher von anderen
stammen, und eine Sequenz führt Befehle auf Ihrem Rechner aus. Die Sequenzen eines
Repositorys werden deshalb **erst angeboten, wenn Sie ihnen vertrauen**, so wie bei
den Befehlen einer committeten Workflow-Datei.

Hat ein Repository eine solche Datei, der noch niemand vertraut hat, erscheint über
dem Log ein Hinweis: *Dieses Repository definiert 2 Sequenzen* mit **Prüfen**,
**Vertrauen** und **Ignorieren**. Er blockiert nie etwas.

- **Prüfen** listet jede Sequenz mit jedem Schritt, seiner Pause und seinen Optionen
  auf, nur lesbar. **Vertrauen** dort vertraut dem, was aufgelistet wurde;
  **Schließen** (oder `Esc`) vertraut nichts. Ist die Liste länger als das Fenster,
  wird Vertrauen dort nicht angeboten (vergrößern Sie das Fenster): Sie vertrauen nie
  etwas, das Sie nicht lesen konnten.
- **Vertrauen** (die Schaltfläche oder die Palettenzeile *Sequenz: Sequenzen dieses
  Repositorys vertrauen*) vertraut der Datei genau so, wie sie jetzt ist.
- **Ignorieren** blendet den Hinweis aus, bis sich die Datei ändert.
- Über die Tastatur: Die Palette hat *Sequenz: Sequenzen dieses Repositorys prüfen /
  vertrauen / ignorieren*.

Festgehalten wird ein Digest aus den IDs, den Befehlen, den Pausen und den Optionen,
im lokalen Zustand des Repositorys gespeichert (nie im Repository). Ändert sich
eines davon, zum Beispiel nach einem Pull, kommt der Hinweis zurück als *Die
Sequenzen dieses Repositorys haben sich geändert*, und sie verschwinden aus Palette
und Tasten, bis Sie ihnen erneut vertrauen. Das Umbenennen einer Sequenz oder das
Bearbeiten ihrer Beschreibung ändert nichts. Eine Sequenz, die Sie selbst in den
Einstellungen schreiben, bleibt vertrauenswürdig, wenn der Datei vor Ihrer Änderung
bereits vertraut wurde (oder sie leer war).

Einmal vertraut, erscheinen Repository-Sequenzen wie Ihre eigenen als
`Sequenz: <Name>` in der Palette und können in den Einstellungen ein Tastenkürzel
erhalten. Eine Sequenz mit derselben ID wie eine von Ihnen ist ein Konflikt: Ihre
gewinnt, und die andere wird als Duplikat gemeldet.

Von der Kommandozeile: `gitpoulpi sequence list` zeigt die Sequenzen der Datei mit
`trusted` und einem Zustand `repo_trust` (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (und `untrust`, `--repo PATH` für ein anderes Repository)
ist dafür gedacht, dass ein Mensch es nach dem Lesen von `sequence show` ausführt.
`sequence run` lehnt eine Repository-Sequenz ohne Vertrauen ab. KI-Agenten können
nicht vertrauen: Dafür gibt es kein MCP-Werkzeug.

## Schritte

**Schritte bearbeiten** ersetzt die Liste durch die Schritte der Sequenz (der Titel
lautet **Schritte von <Name>**) und die Zeilen durch die des ausgewählten Schritts.
**Zurück** (das Kreuz oben bei den Zeilen, „Zurück zu den Sequenzen“) oder `Esc` kehrt
zu den Sequenzen zurück.

* **+ Hinzufügen** öffnet **Schritt hinzufügen**: jeden Befehl, den eine Sequenz
  ausführen darf, unter seinem Palettennamen (die KI-Befehle mit dem Funkeln), beim
  Tippen mit dem Abgleich der Palette gefiltert. Ein Schritt, den diese Version nicht
  ausführen kann, ist mit dem Grund abgedunkelt. `Enter` oder ein Klick fügt ihn am
  Ende hinzu; `Esc` oder das Kreuz schließt das Popup.
* **Duplizieren**, **Entfernen**, **Auf** / **Ab** und Ziehen funktionieren wie bei
  den Sequenzen.
* Ein zerstörerischer Befehl (einen Branch löschen, Änderungen verwerfen...) wird nie
  angeboten: Eine Sequenz kann keinen ausführen.

### Pause

Jeder Schritt hat eine **Pause**:

| Pause | Bedeutung |
|---|---|
| **Standard** | Der Schritt hält nur an, wenn er eine Antwort braucht, in seinem üblichen Dialog (ein Branch-Name, eine noch leere Commit-Nachricht...). |
| **Immer** | Vor diesem Schritt anhalten und um Fortsetzen bitten. |
| **Nie** | Nie anhalten. Die Optionen des Schritts müssen ihm alles geben, was er braucht, ein Schritt, der nach einem Namen fragt, braucht also seine ausgefüllte Option **Name**. |

Ein Schritt, der vor der Ausführung immer fragt (ein Force-Push), bietet **Nie** nicht
an: Die Zeile nennt den Grund. **Nie** beim Schritt der KI-Nachricht ist als
ausdrückliche Wahl erlaubt.

### Optionen

Die Zeilen unter **Pause** sind die Optionen des Schritts: ein Text **Name** oder
**Nachricht**, ein **Branch**, ein Kästchen wie **In neuem Tab öffnen** oder eine
Dropdownliste, die aus Ihrer Konfiguration gefüllt wird (ein **Remote**, ein
**KI-Profil**, ein Eintrag von **Öffnen in**). Texte akzeptieren die Variablen
`${branch}`, `${date}`, `${time}` und `${n}` (eine Zahl, die den Namen eindeutig
macht). Einen Text zu leeren entfernt die Option.

## Mit KI erstellen und bearbeiten

Bei [eingeschalteter KI](ai.md#sequenzentwurf) (Einstellungen > KI) erscheinen zwei
Einträge: **Hinzufügen > Mit KI erstellen...** (auch die Palettenzeile
`Sequenz: Mit KI erstellen...`, die die Einstellungen auf dieser Seite mit dem Dialog
öffnet) und **Mit KI bearbeiten...** neben **Jetzt ausführen** für die ausgewählte
Sequenz. Bei ausgeschalteter KI gibt es keinen von beiden.

```
+- Sequenz mit KI erstellen -------------------------------- x -+
| Beschreiben Sie die gewünschte Sequenz                        |
| +-----------------------------------------------------------+ |
| | alles stagen, die Commit-Nachricht mit der KI schreiben,  | |
| | committen, dann pushen                                    | |
| +-----------------------------------------------------------+ |
| Enter fügt eine Zeile ein. Ctrl+Enter sendet. Esc schließt.   |
| [ ] Kontext senden (Remote-Namen, KI-Profile, Einträge von ...|
|    Aus: Es wird nur Ihre Beschreibung gesendet.               |
| Die KI schlägt nur vor. Vor Ihrer Zustimmung wird nichts ...  |
| [ Senden ] [ Abbrechen ]                                      |
+---------------------------------------------------------------+
```

Sie schreiben in eigenen Worten, was Sie möchten; **Senden** (oder `Ctrl+Enter`,
`Ctrl+S`) fragt das Modell, über dasselbe Profil, dieselbe Jobanzeige und dasselbe
`Esc` wie jede andere KI-Aktion (und dieselbe Frage „Was gesendet wird“). Die Antwort
ist ein **Vorschlag**, den Sie lesen, bevor etwas geschieht:

* der Name, die ID, die Beschreibung und die Schritte mit ihrer Pause und ihren
  Optionen;
* bei **Mit KI bearbeiten** ein Vergleich mit der aktuellen Sequenz: `+` ein Schritt
  hinzugefügt, `-` ein Schritt entfernt, `~` ein Schritt, dessen Pause oder Optionen
  sich geändert haben, `~ Name: "a" -> "b"` für ein Feld;
* **Ausgelassen**: ein vom Modell vorgeschlagener Schritt, der nicht erlaubt ist
  (unbekannt, zerstörerisch, eine Sequenz in einer Sequenz, eine unmögliche Pause),
  mit dem Grund; die gültigen Schritte bleiben erhalten;
* die **Hinweise** des Modells und ein **vorgeschlagenes Tastenkürzel**, das nur
  angezeigt wird, mit der Angabe, ob die Taste schon belegt ist. Es wird nie
  gebunden: Nutzen Sie danach die Schaltfläche **Tastenkürzel**.

```
+- Vorgeschlagene Änderung --------------------------------- x -+
| + hinzugefügt   - entfernt   ~ geändert                       |
| ~ Name: "Täglich" -> "Täglich 2"                              |
| Schritte (3)                                                  |
| + 1. Git: Alle Remotes abrufen                                |
|   2. Git: Alle Änderungen stagen                              |
| - Git: Commit...                                              |
| ~ 3. Git: Push: Standard -> Pause: Immer                      |
| Ausgelassen                                                   |
| Schritt 4 `reset_hard`: zerstörerische Schritte sind nicht ...|
| [ Annehmen ] [ Vor dem Speichern bearbeiten ] [ Ablehnen ] [ Zurück ] |
+---------------------------------------------------------------+
```

* **Annehmen** schreibt den Vorschlag mit einem Speichervorgang (eine neue Sequenz
  oder den Ersatz der bearbeiteten, die ihre ID, ihren Geltungsbereich und ihre Taste
  behält) und wählt sie in der Liste aus.
* **Vor dem Speichern bearbeiten** tut dasselbe und öffnet ihre Schritte, damit Sie
  die Zeilen anpassen können (diese Seite speichert jede Änderung sofort, die Sequenz
  ist also bereits gespeichert).
* **Ablehnen** schreibt nichts. **Zurück** kehrt zur Beschreibung zurück, um erneut
  zu senden.
* **Annehmen** ist nicht verfügbar, wenn es nichts anzunehmen gibt: kein brauchbarer
  Schritt, ein Problem, das die Seite markieren würde, oder eine Änderung, die nichts
  ändert.

Vor **Annehmen** wird nichts geschrieben, und eine Sequenz wird von hier aus nie
ausgeführt. Das Schließen des Dialogs (das Kreuz, `Esc`, ein Klick außerhalb) fragt
„Änderungen verwerfen?“, solange eine Beschreibung getippt oder ein Vorschlag offen
ist; das Schließen der Einstellungen fragt ebenfalls.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Sicherung"
scope = "installation"
owner = "<ID dieser Installation>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Schlüssel, die diese Version nicht kennt, und Schritte, die sie nicht kennt, bleiben
erhalten, wenn die Seite speichert.

## Eine Sequenz ausführen

Eine Sequenz ist eine benannte Liste von Schritten, jeder ein Befehl der Palette
(alles stagen, die Commit-Nachricht mit der KI schreiben, committen, pushen...).
Führen Sie sie aus der Palette aus (`Sequenz: <Name>`), mit der Taste, die Sie ihr
unter Einstellungen > Tastatur & Maus gegeben haben (`[keybindings.sequence]` in
`tui.toml`), oder von der Kommandozeile (`gitpoulpi sequence run`).

Während sie läuft, zeigt eine Leiste über dem Log `<Name> k/n`, den Schritt und
worauf gewartet wird. Der Lauf erscheint auch im Jobs-Panel als
`Sequenz <Name> k/n: <Schritt>`. Es läuft immer nur eine Sequenz; wer eine zweite
startet, erfährt, welche läuft.

**Wo sie zum Fragen anhält.** Jeder Schritt hat eine Einstellung `pause` (`default`,
`always`, `never`):

| Schritt | Standard | `pause = "never"` |
|---|---|---|
| KI-Commit-Nachricht | die Nachricht landet im Nachrichtenfeld; die Leiste sagt „Nachricht prüfen, dann Fortsetzen“ | geht sofort weiter |
| Commit | eine leere Nachricht hält immer an: Das Feld hat die Tastatur, und Ihr Commit (oder Fortsetzen) macht weiter | nicht verfügbar, wenn es keine Nachricht zum Committen gibt |
| KI-Commits zusammenstellen | das Review öffnet sich; Anwenden macht weiter, Schließen bricht ab | wendet den Vorschlag unverändert an |
| Worktree erstellen | der Dialog öffnet sich mit ausgefülltem Namen | erstellt ihn ohne Dialog aus `options.name` |
| Push | bei `git.push_behavior = ask` öffnet sich die Auswahl „Push nach...“ | pusht nach `options.remote` |
| Force-Push mit Lease, Stagen von Dateien, die wie Geheimnisse aussehen | fragt immer | nicht erlaubt |

`pause = "always"` bei einem beliebigen Schritt zeigt vor seiner Ausführung „Als
Nächstes: <Schritt>“ mit **Fortsetzen** und **Abbrechen**. Wird der Push abgelehnt,
weil der Branch abgewichen ist, öffnet sich der übliche Dialog, und die Leiste sagt
„Push abgelehnt: im Dialog wählen“; Pull und dann Push (oder ein Force-Push mit
Lease) setzt den Lauf fort, das Schließen des Dialogs bricht ihn ab.

**Fortsetzen** ist die Schaltfläche der Leiste, `Sequenz: Fortsetzen` in der Palette
oder die Taste, die an `sequence.continue` gebunden ist. **Abbrechen** ist das
Schließkreuz der Leiste, `Sequenz: Abbrechen` (`sequence.cancel`), das Jobs-Panel
oder `Esc`, wenn nichts anderes es braucht (ein offener Dialog schließt zuerst, was
ebenfalls abbricht). Beim Abbrechen bleiben die bereits ausgeführten Schritte
erhalten, und das wird gesagt: „<Name> nach Schritt k/n (<Schritt>) abgebrochen;
frühere Schritte bleiben erhalten“.

**Schlägt ein Schritt fehl**, hält der Lauf dort an: Die Statuszeile nennt die
Sequenz, den Schritt und den Fehler, die Leiste wird rot mit **Details** (dem Eintrag
des Aktivitätsprotokolls, der die Optionen des Schritts auflistet) und einem
Schließkreuz, und die früheren Schritte bleiben erledigt.

**Worktrees.** Ein Schritt „Worktree erstellen“ mit `open_tab = true` verlegt den
Lauf in den Tab des neuen Worktrees: Die nächsten Schritte (`tab_dir_terminal`,
`tab_open_agent_here`, `tab_open_in`) wirken dort. Ist ein anderer Tab zu sehen, sagt
die Leiste „wartet auf Tab <Name>“ mit **Dorthin**. Das Schließen dieses Tabs bricht
den Lauf ab. Ein Lauf überlebt das Beenden von GitPoulpi nicht.

### Ein Agent schlägt einen Lauf vor

Bei eingeschaltetem *KI-Agenten Änderungen zur Bestätigung vorschlagen lassen*
(`[mcp] allow_ui_propose`) kann ein Agent Ihr Fenster bitten, eine Sequenz
auszuführen (`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Es erscheint
ein Banner: *✦ Vorgeschlagen von <Agent>: Sequenz <Name> ausführen* mit
**Ausführen** und **Verwerfen**. Nichts läuft, bis Sie Ausführen drücken (anklicken
oder `Ctrl+N`, dann `Enter`); Verwerfen, `Esc` (wenn nichts anderes es braucht) oder
die Palettenzeile *Agent: Ausstehende Ansicht verwerfen* wirft ihn weg. Der Lauf geht
dann genau so, als hätten Sie ihn gestartet, und hält dort an, wo die Sequenz es
vorsieht. Der Vorschlag wird abgelehnt, wenn die Sequenz nicht existiert, ungültig
ist oder eine Repository-Sequenz ist, der Sie nicht vertraut haben.

## Einen Lauf rückgängig machen

Jeder Schritt, der das Repository ändert (stagen, committen, Tag, Branch, Stash,
Worktree...), hinterlässt einen **eigenen gewöhnlichen Rückgängig-Eintrag**, mit dem
Lauf gekennzeichnet, sodass `Ctrl+Z` weiterhin einen Schritt nach dem anderen
rückgängig macht und der [Rückgängig-Verlauf](undo-redo.md#der-rückgängig-verlauf)
`⟳ <Name>` an ihnen zeigt. Um den ganzen Lauf zurückzunehmen, gibt es **Ganze Sequenz
rückgängig**: die Schaltfläche des Rückgängig-Verlaufs (`x`), die Palettenzeile
`Bearbeiten: Ganze Sequenz rückgängig` oder `gitpoulpi undo --sequence`. Es macht die
Einträge des Laufs rückgängig, den neuesten zuerst, jeden mit seinen eigenen
Prüfungen und Rückfragen (ein bereits gepushter Commit fragt vorher), hält bei der
ersten Ablehnung mit Nennung des Schritts an und lässt die älteren im Journal. Die
KI-Nachricht und ein Push hinterlassen keinen Eintrag: Ein Push wird nie
rückgängig gemacht. Nur das Journal des aktuellen Repositorys wird angefasst; in
einem Worktree ausgeführte Schritte werden von diesem Repository aus rückgängig
gemacht. Wiederholen bleibt pro Eintrag. Details: [Sequenzläufe](undo-redo.md#sequenzläufe).
