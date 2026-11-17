[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · **Deutsch** · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

Ein Worktree ist eine zweite ausgecheckte Kopie des Repositorys: ein anderer
Ordner auf einem anderen Branch, mit demselben Verlauf. Der Abschnitt WORKTREES
des Refs-Bereichs listet sie auf; der Haupt-Worktree ist mit `(Haupt)` markiert.

![Der Refs-Bereich eines Repositorys mit zwei verknüpften Worktrees: Der Abschnitt WORKTREES listet main (Haupt), hotfix/axis-labels und feature/legend, darunter der Abschnitt SUBMODULE](../images/worktrees.png)

## Den Abschnitt anzeigen

In einem Repository ohne verknüpften Worktree ist der Abschnitt ausgeblendet.
Um ihn trotzdem zu sehen (er lautet dann „Noch kein Worktree: + erstellt
einen“), nutzen Sie den Paletten-Befehl **Seitenleiste: Abschnitt Worktrees
anzeigen**, haken **WORKTREES** in der Ein-/Ausblend-Liste der Seitenleiste an
(Rechtsklick auf eine Abschnittsüberschrift) oder führen einen der folgenden
Befehle zum Erstellen aus. GitPoulpi merkt sich das für dieses Repository (siehe
**Abschnitte der Seitenleiste** in den [Einstellungen](settings.md)); ein
anderes Repository ohne Worktrees startet weiterhin mit ausgeblendetem Abschnitt.

## Einen Worktree erstellen

Das **+** in der Überschrift von WORKTREES (Tastatur: `+` im Abschnitt; Palette:
**Refs: Worktree erstellen...**) öffnet ein Menü:

- **Aus dem aktuellen Branch**: Der Branch ist im Hauptordner ausgecheckt, daher
  schlägt der Dialog einen neuen Branch darauf vor (`worktree-<id>`).
- **Aus dem ausgewählten Commit**: ausgegraut, bis im Log ein Commit ausgewählt
  ist. Der Dialog schlägt einen neuen Branch `worktree-<id>` vor, der dort
  startet; leeren Sie das Branch-Feld für einen losgelösten Worktree.
- **Branch wählen...**: Der Dialog öffnet sich auf seiner Liste **Worktree-Branch**,
  um einen schon vorhandenen Branch zu öffnen.

Die Palette hat dieselben drei (**Refs: Worktree aus aktuellem Branch
erstellen**, **Git: Worktree am ausgewählten Commit erstellen...**, **Refs:
Worktree mit Branch-Auswahl erstellen**). Die älteren Einträge funktionieren
weiterhin: **Worktree aus ... erstellen...** im Menü eines Branches, Tags oder
Commits.

![Der Dialog Worktree erstellen: die Felder Name, Ordner, Ref und Worktree-Branch, die Zeile „neuer Branch ab main“, das Kästchen Den neuen Worktree als Tab öffnen und die Schaltflächen Abbrechen und Erstellen](../images/worktree-create.png)

Der Dialog schlägt alles vor und lässt Sie es ändern. **Name** und neuer Branch
beginnen beide als `worktree-<id>`, wobei `<id>` 8 zufällige Hexziffern sind
(bei beiden dieselben), sodass zwei Worktrees nie kollidieren.

| Feld | Bedeutung |
|---|---|
| **Name** | Der Ordnername wird daraus gebildet. **Name nach Branch** (`Alt+N`) setzt ihn auf `worktree-<Branch>`: Schrägstriche werden zu `-`, und das Wort wird nicht verdoppelt (`worktree-foo` und `feature/worktree-x` behalten ihren eigenen, `feature/login` ergibt `worktree-feature-login`). |
| **Ordner** | Ein Nachbar des Repositorys, `<Repository>-<Name>`. Er folgt dem Namen, bis Sie im Feld Ordner selbst tippen; **Durchsuchen** wählt den Ort. |
| **Ref** | Wo ein *neuer* Branch startet: ein Branch oder ein Commit (eine Liste; zum Suchen tippen). Sie wird nicht verwendet, und das steht dort, solange ein vorhandener Branch gewählt ist. |
| **Worktree-Branch** | Ein Feld, zwei Bedeutungen, darunter benannt. Ein Name, der kein lokaler Branch ist, erstellt einen neuen Branch ab der Ref (**neuer Branch ab main**). Ein lokaler Branch, der anderswo nicht ausgecheckt ist (in der Liste wählen oder seinen Namen tippen), wird dort unverändert ausgecheckt (**checkt diesen Branch aus**). Die erste Zeile der Liste oder ein leeres Feld checkt den Branch der Ref aus (oder löst sich auf einem Commit); es wird abgelehnt, wenn dieser Branch bereits ausgecheckt ist. |
| **Neu würfeln** | `Alt+R`. Würfelt eine neue `<id>` und ersetzt damit Name und Branch, außer was Sie selbst getippt haben (ein von Ihnen geschriebener Name oder Branch bleibt) und einen vorhandenen Branch (wird nie ersetzt). Ein Name aus **Name nach Branch** oder von der KI gilt als erzeugt. |
| **Name per KI** | Nur bei eingeschalteter KI (siehe unten). |
| **Den neuen Worktree als Tab öffnen** | Standardmäßig an. |

Ein Ordner, der nicht leer ist, ein ungültiger Name, ein Branch-Name, den Git
ablehnt, oder ein Branch, den ein anderer Worktree hat, wird in der eigenen Zeile
des Dialogs gemeldet, und **Erstellen** bleibt ausgegraut.

### Mit KI benennen

Bei eingeschalteter [KI](ai.md) öffnet **Name per KI** (`Alt+A`) ein Feld im
Dialog: *Feature, Bug oder Aufgabe beschreiben*. **Benennen** bittet das Modell
um einen kurzen Worktree-Namen und einen Branch-Namen, der den eigenen
Branch-Namen des Repositorys folgt (`feature/`, `fix/`, Kleinbuchstaben mit
Bindestrichen). Sie füllen den Namen (als `worktree-<name>`) und das Branch-Feld,
weiter bearbeitbar; ein Branch-Name, den es schon gibt, bekommt das Suffix `-2`,
damit die Antwort nie stillschweigend etwas auscheckt. Das Feld zeigt den
Zustand „beschäftigt“, **Stopp** (oder `Esc`) bricht ab, und ein Fehler wird im
Dialog angezeigt. Gesendet werden nur Ihre Beschreibung und die Namen der
lokalen Branches.

## Das Menü der Zeile

Rechtsklick auf einen Worktree (oder ihn auswählen und die Menütaste, `Shift+F10`
oder `.` drücken):

- **In neuem Tab öffnen**: wie ein Doppelklick oder `Enter`.
- **Öffnen in**: die Einträge von [Öffnen in](open-in.md), deren Kästchen
  **Kann einen Worktree öffnen** angehakt ist (Einstellungen > Öffnen in;
  standardmäßig ist jeder Eintrag angehakt). Sie laufen mit dem Ordner des
  Worktrees als `${cwd}`.
- **Merge von <Branch> in...**: den Branch wählen, in den zusammengeführt wird
  (standardmäßig der Branch des Haupt-Worktrees). Der gewöhnliche Merge läuft in
  dem Worktree, in dem dieser Branch ausgecheckt ist, mit Ihren Merge-Optionen;
  ein Konflikt wird dort im üblichen Konflikt-Ablauf gelöst. Ein Branch, der
  nirgends ausgecheckt ist, wird mit Begründung abgelehnt. Bei einem losgelösten
  Worktree nicht angeboten.
- **Worktree entfernen...**: nie der Haupt-Worktree.

![Das Rechtsklickmenü einer Worktree-Zeile: In neuem Tab öffnen, Öffnen in, Merge von hotfix/axis-labels in... und Worktree entfernen...](../images/worktree-menu.png)

**In neuem Tab öffnen** (oder ein Doppelklick, oder `Enter`) öffnet den Ordner des
Worktrees als eigenen Tab, mit eigenem Log, Status und Branch:

![Der Worktree hotfix/axis-labels als eigener Tab neben dem Haupt-Tab, mit seinem Commit und seiner geänderten Datei](../images/worktree-tab.png)

Die Palette hat dieselben Aktionen für die ausgewählte Zeile (**Refs: Worktree
öffnen in...**, **Refs: Merge des Worktree-Branches in...**, **Refs: Worktree
entfernen...**, **Refs: Worktree in Tab öffnen**).

## Einen Worktree entfernen

Ein sauberer Worktree bekommt eine einfache Bestätigung. Ist sein Branch bereits
übernommen (in seinen Upstream, den Branch des Haupt-Worktrees oder
`main` / `master`), ist das Kästchen **Auch Branch <b> löschen** standardmäßig
angehakt; ein nicht übernommener Branch meldet, dass er bleibt. **Merge in...,
dann entfernen** führt zuerst zusammen und entfernt erst, wenn der Merge ohne
Konflikte durchgelaufen ist.

Bei nicht committeten Änderungen (getrackt oder ungetrackt) listet der Dialog sie
auf (Anzahl, scrollbar) und bietet an:

- **Committen und entfernen**: ein Nachrichtenfeld im Dialog; jede Änderung wird
  gestaget (`.gitignore` wird beachtet) und auf dem Branch des Worktrees
  committet, danach wird der Worktree entfernt. Ein losgelöster Worktree benennt
  im selben Dialog einen Branch für den Commit.
- **Stash anlegen und entfernen**: jede Änderung, ungetrackte Dateien
  eingeschlossen, wird mit einer bearbeitbaren Nachricht (Standard „WIP on
  <Ordner>“) gestasht, danach wird der Worktree entfernt. Die Stash-Liste gehört
  allen Worktrees gemeinsam: Der Stash liegt danach in den STASHES des
  Haupt-Repositorys, bereit zum Pop.
- **Zum Committen öffnen**: öffnet den Worktree in einem Tab mit fokussierter
  Commit-Nachricht; nichts wird entfernt.
- **Trotzdem entfernen**: verwirft die Änderungen nach einer zweiten,
  ausdrücklichen Bestätigung.

## Rückgängig

Das Entfernen eines Worktrees lässt sich nicht rückgängig machen: Der Ordner ist
weg. Was zuvor gesichert wurde, bleibt: der Commit auf seinem Branch, der Stash
in der Liste. Das Löschen des Branches, das **Auch Branch ... löschen** hinzufügt,
läuft über das gewöhnliche Journal: **Rückgängig** bringt den Branch zurück (nicht
den Ordner). Das Erstellen eines Worktrees gehört ebenfalls nicht zum Verlauf.

## Aus einem Skript oder von einem Agenten

`gitpoulpi worktree changes <path>` listet die nicht committeten Änderungen eines
Worktrees auf. `gitpoulpi worktree remove <path> [--force] [--commit-message M
[--commit-branch B]] [--stash-message M] [--delete-branch]` entfernt ihn; die
MCP-Tools heißen `worktree_changes` und `worktree_remove`. `gitpoulpi ai
worktree-name --describe "<task>"` gibt die Worktree- und Branch-Namen aus, die die
KI vorschlägt (`{ name, worktree, branch }`), und erstellt nichts; das MCP-Tool ist
`ai_worktree_name`. Siehe die [CLI-Referenz](cli-reference.md).
