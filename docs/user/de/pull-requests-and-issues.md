[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · **Deutsch** · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull Requests und Issues

Pull-/Merge-Requests und Issues lesen, erstellen und verwalten, aus der Seitenleiste, dem Board und der Symbolleiste. Verbinden Sie zuerst ein Konto und verknüpfen Sie ein Repository: siehe [Hosting- und Issue-Tracker-Integrationen](hosting-integration.md). Alles, was zu einem einzelnen Issue oder Request gesagt wird (Kommentare, Reviews, Threads), steht in [Unterhaltungen](conversations.md); einen Request lesen und reviewen steht in [Einen Pull oder Merge Request prüfen](review.md).

## Pull Requests und Issues in der Seitenleiste

### PULL REQUESTS

Der Abschnitt hat eine Kopfzeile mit der Zahl und einem `+` (einen Pull Request
erstellen), den Dropdownlisten für Anbieter und Projekt, einem Suchfeld und einer
Zustands-Dropdownliste (*Offen*, *Gemergt / geschlossen*, *Alle*). Dann folgen vier
Gruppen, jede mit ihrer Zahl:

- **Meine Pull Requests**: Sie sind der Autor.
- **Mir zugewiesen**.
- **Wartet auf mein Review**: Sie sind angeforderter Reviewer.
- **Alle Pull Requests**: anfangs eingeklappt.

Das Suchfeld filtert jede Gruppe (Titel, Nummer, Autor, Branch). Jede Änderung der
Zustands-Dropdownliste fragt den Host erneut.

Eine Zeile beginnt mit einem Zustandssymbol: `●` offen (grün), `○` Entwurf, `◆` gemergt
(lila), `✕` geschlossen (rot). Der Request des ausgecheckten Branches ist fett. Zeilen
lauten `#12  Titel` (GitHub, Bitbucket) oder `!53  Titel` (GitLab, Azure DevOps).

### ISSUES

Der Abschnittstitel nennt den Anbieter: `GITHUB ISSUES`, `JIRA ISSUES` und so weiter. Er
lautet `... WORK ITEMS` bei Azure DevOps und `... CARDS` bei Trello. Unter den
Dropdownlisten: ein Suchfeld, die Zustands-Dropdownliste (*Offen*, *Mir zugewiesen*,
*Geschlossen*, *Alle*) und die Issues. Eine Jira-Zeile lautet `ABC-12  Titel`, mit
abgedunkeltem Typ und Status, wenn die Seitenleiste breit genug ist.

![Der Abschnitt ISSUES der Seitenleiste mit geöffnetem Rechtsklickmenü von Issue #7, Unterhaltung öffnen als erste Zeile](../images/issues-sidebar.png)

### Aktionen auf einer Zeile

- `Enter` oder ein Doppelklick öffnet die Zeile: einen Pull Request im [Review](review.md) (GitHub, GitLab, Gitea, Forgejo, Gerrit), ein Issue und einen Request bei Azure DevOps oder Bitbucket als [Unterhaltung](conversations.md) im Fenster, alles andere in Ihrem Browser. **Im Browser öffnen** im Zeilenmenü öffnet immer den Browser.
- `Enter` oder ein Doppelklick öffnet die Zeile in Ihrem Browser.
- Rechtsklick oder `.` öffnet das Zeilenmenü:
  - **Unterhaltung öffnen** (Issues und Requests, die keine Review-Ansicht haben), **Im Browser öffnen**, **URL kopieren**, **Nummer kopieren**.
  - **Im Browser öffnen**, **URL kopieren**, **Nummer kopieren**.
  - Für einen Pull Request: **Auf <Anbieter> bearbeiten**, **Quell-Branch auschecken**,
    **Commits des Quell-Branches ansehen**. Bei einem Request aus einem Fork lautet der
    Checkout **Pull Request auschecken (Fork)**, und **Commits des Quell-Branches ansehen** ist ausgegraut.
  - Für ein Issue: **Branch aus Issue erstellen**. Es öffnet die Eingabe für einen neuen
    Branch mit einem Namen wie `12-fix-the-thing` (oder `ABC-123-fix-the-thing` bei
    Jira).
  - Bei GitHub, GitLab, Gitea und Forgejo hat eine Pull-Request-Zeile außerdem **Review...**
    (auch **Enter** auf der Zeile): das Review in GitPoulpi, siehe
    [Einen Pull Request prüfen](review.md).
  - Bei GitHub, GitLab, Gitea und Forgejo haben beide Zeilen außerdem **Zugewiesene bearbeiten...**,
    **Labels bearbeiten...** und **Meilenstein festlegen...** (und **Reviewer bearbeiten...** bei
    einem Pull Request): siehe
    [Zugewiesene, Reviewer, Labels und Meilenstein](#zugewiesene-reviewer-labels-und-meilenstein).
- **Auf <Anbieter> bearbeiten** hat drei weitere Zugänge: den Stift `✎` am rechten Ende
  einer überfahrenen oder ausgewählten Pull-Request-Zeile, `e` auf der ausgewählten Zeile
  und den Palettenbefehl *Integrationen: Verknüpften Pull Request auf dem Host bearbeiten* (für den
  Request des ausgecheckten Branches).
- Ist der Quell-Branch noch nicht im Repository, sagt GitPoulpi das. Fetchen Sie zuerst
  (`Ctrl+G f`).

**Checkout eines Pull Requests.** GitPoulpi holt den Request in einen lokalen Branch
`pr/<n>` (`mr/<n>` bei GitLab), der ihm folgt, und wechselt darauf. Bei GitHub, GitLab, Gitea und Forgejo
funktioniert das auch für Forks (GitPoulpi holt die besondere Head-Ref des Hosts). Azure
DevOps und Bitbucket holen den Quell-Branch des Requests. Ein Request aus einem Fork wird
dort abgelehnt: Fügen Sie den Fork als Remote hinzu und checken Sie seinen Branch selbst
aus. Rückgängig kehrt zum vorherigen Branch zurück. CLI: `gitpoulpi hosting checkout-pr <n>`.

**Einen Pull Request von der Kommandozeile prüfen (und durch einen Agenten).** Bei GitHub
(auch Enterprise), GitLab (auch selbst gehostet), Gitea und Forgejo lässt sich ein Request
lesen und prüfen, ohne GitPoulpi zu verlassen: `hosting review show <n>` (Dateien,
Merge-Status, Checks), `hosting review threads <n>` (Review-Threads und
Unterhaltung), `hosting review comment`, `hosting review reply`, `hosting review resolve`
und `hosting review submit <n> --event approve|request-changes|comment
--comment Pfad:Zeile[:Seite]:Text`. Der Diff kommt aus Ihrem Repository, nicht vom
Host: `hosting review fetch <n>` holt den Head des Requests nach
`refs/gitpoulpi/pr/<n>` und seinen Zielbranch nach `refs/gitpoulpi/pr/<n>-base`
(beides sind keine Branches, nichts wird ausgecheckt). Die Zeilennummern sind die dieses
Diffs: `new` ist die Zeile im Head, `old` die an der Merge-Basis. GitLab
hat keinen Review-Status „Änderungen anfordern“: Er wird dort abgelehnt, ohne dass etwas gesendet wird.
Gitea und Forgejo nehmen einen Kommentar nur zu einer Zeile (keine Bereiche); ein Thread sind die
Kommentare zu einer Zeile einer Datei. Gitea kann einen Thread auflösen, Forgejo 11 nicht
(der Host hat dafür keinen Endpunkt: Der Aufruf sagt es). Eine Antwort auf einen Server
ohne den Antwort-Endpunkt wird als Ein-Kommentar-Review an derselben Zeile gesendet.
Gitea lehnt es ab, den eigenen Request zu genehmigen oder abzulehnen.
Ohne Klon: Fügen Sie `--project owner/name` (GitLab: `group/project`) zu `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` und `suggest` hinzu, um ein Projekt zu prüfen, das nicht das aktuelle Repository ist;
`hosting review diff <n> --project P` liest den Diff (Hunks pro Datei) über die API des Hosts, und
`hosting file <path> --project P [--ref R]` liest eine Datei in einer Revision (GitHub, GitLab, Gitea, Forgejo und Gerrit; bei Gerrit wird der Diff eines
Patch Sets Datei für Datei gelesen, und die Datei kommt vom Branch oder, bei einer Commit-ID, vom Commit). Eine Datei, die der Host
nicht im Diff zeigt (zu groß), ist als `omitted` markiert.
Die MCP-Tools sind dieselben Befehle (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Zugewiesene, Reviewer, Labels und Meilenstein

Bei GitHub (auch Enterprise), GitLab (auch selbst gehostet), Gitea und
Forgejo können Sie ändern, wem ein Pull Request oder Issue zugewiesen ist, wer ihn prüft, seine Labels
und seinen Meilenstein, ohne GitPoulpi zu verlassen. Andere Anbieter zeigen nichts davon.

- **Zeilenmenü**: **Zugewiesene bearbeiten...**, **Reviewer bearbeiten...** (nur Pull
  Requests), **Labels bearbeiten...**, **Meilenstein festlegen...**.
- **Palette**: *Hosting: Zugewiesene bearbeiten...*, *Hosting: Reviewer bearbeiten...*,
  *Hosting: Labels bearbeiten...*, *Hosting: Meilenstein festlegen...* listen jeweils zuerst die
  Requests und Issues der Seitenleiste zur Auswahl. Im
  [Radar](radar.md) wirken dieselben vier auf die ausgewählte Zeile.
- Die Auswahlliste öffnet sich sofort und lädt die Personen oder Labels des Projekts (und was
  das Element jetzt hat) vom Host. Nach etwa 150 ms erscheint eine Spinner-Zeile.
  Zum Filtern tippen (unscharfer Treffer), `Auf` / `Ab` bewegen, `Leertaste` setzt oder
  entfernt das Häkchen, `Enter` übernimmt, `Esc` bricht ab, `Tab` erreicht die Schaltflächen. Ein Klick
  auf eine Zeile setzt das Häkchen, und das Mausrad scrollt. Personen zeigen eine Initialen-Plakette,
  Labels ihre Farbe.
- Der Meilenstein ist eine Dropdownliste (mit **Kein Meilenstein**, um ihn zu entfernen).
- Nach **Übernehmen** wird der Host aktualisiert, und die Listen und das Radar zeigen die
  Änderung nach ihrer nächsten Aktualisierung, die sofort beginnt.

Die Formulare zum Erstellen (Pull Request, Issue) haben dieselben Auswahlen als Reihe von
Schaltflächen: `Zugewiesene: keine`, `Reviewer: keine`, `Labels: keine`,
`Meilenstein: keine`. Bei GitHub, GitLab, Gitea und Forgejo ersetzt die Schaltfläche Labels das getippte
Labels-Feld des Issue-Formulars. Nimmt ein Host den neuen Pull Request an, lehnt aber
einige seiner Labels oder Personen ab, sagt es die Statuszeile.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>` und `--assignee`, `--reviewer`, `--label`,
`--milestone` bei `hosting pr` und `hosting issue create`; siehe die
[CLI-Referenz](cli-reference.md).

### Tasten in den Abschnitten

| Taste | Aktion |
|---|---|
| `Enter` / Doppelklick | die Unterhaltung in GitPoulpi öffnen, wo der Host eine hat (ein Pull Request bei GitHub, GitLab, Gitea oder Gerrit öffnet das Review), sonst den Browser |
| `o` | die Zeile im Browser öffnen |
| `Enter` / Doppelklick | die Zeile im Browser öffnen |
| `.` / Rechtsklick | Zeilenmenü |
| `e` | den ausgewählten Pull Request beim Host bearbeiten |
| `n` | neuer Pull Request (PULL REQUESTS) oder neues Issue (ISSUES) |
| `v` | die Zustands-Dropdownliste öffnen |
| `r` | aktualisieren |
| `Enter` / `Space` auf einer Kopfzeile | den Abschnitt ein- oder ausklappen |

### Neu verbinden und offline

Fehlt das Token, wird es abgelehnt oder hat es eine Berechtigung nicht, lautet die erste
Zeile des Abschnitts `Erneut verbinden (Token abgelehnt)`. Klicken Sie darauf oder drücken Sie
`Enter`, um das Profil in Einstellungen > Integrationen zu öffnen.

Ist der Host nicht erreichbar, bleiben die letzten Listen auf dem Bildschirm, und die
Kopfzeile zeigt ihr Alter (`· 5m alt`). Die letzte Antwort wird außerdem auf dem Datenträger
zwischengespeichert (`.gitpoulpi/hosting-cache.json`), sodass die Listen erscheinen, bevor
das Netz antwortet.

## Das Issue-Board

Die Schaltfläche **Board** in der Kopfzeile von ISSUES (Palette: **Issues: Board öffnen**) zeigt die Issues der verknüpften Quelle als Kanban-
Board über dem Fenster. Die Spalten hängen von der Quelle ab:

| Quelle | Spalten | Eine Karte verschieben |
|---|---|---|
| Lokale Markdown-Notizen | die Status (`statuses` des Profils, dann jeder andere in Gebrauch) | schreibt den `status` der Notiz um |
| GitHub, GitLab, Gitea, Forgejo | **Offen**, dann die von Ihnen gewählten Labels, dann **Geschlossen** | tauscht das Status-Label; nach **Geschlossen** schließt es das Issue, in eine Spalte zurück öffnet es es wieder |
| Trello | die offenen Listen des Boards, von links nach rechts | verschiebt die Karte in die Liste |
| Redmine | die Issue-Status | ändert den Status des Issues (Redmines Workflow-Regeln gelten) |
| Jira (Cloud und Data Center) | die Status des Projekts, zuerst To-do, zuletzt erledigt | führt den Workflow-Übergang aus, der zur Spalte führt (`GET`, dann `POST .../transitions`); gibt es keinen solchen Übergang, springt die Karte zurück und die Statuszeile sagt `no transition from X to Y` |
| Azure DevOps (Services und Server) | die Zustände der Work-Item-Typen des Projekts, nach Zustandskategorie | setzt `System.State`; ein Work Item, dessen Typ den Zustand der Spalte nicht hat, springt mit der Begründung zurück (`a Bug has no state Done`) |

Bei einem GitHub- oder GitLab-Projekt fragt **Spalten** (oder `c`, oder **Board: Spalten-Labels festlegen...**) nach den Labels, die die
Spalten bilden, der Reihe nach, zum Beispiel `status::todo, status::doing, status::review`. Es schlägt die Labels des Projekts vor, die
wie Status aussehen. Die Wahl wird für das Repository gespeichert. Ohne Labels ist das Board Offen / Geschlossen. Eine Karte zu verschieben
berührt nur die aufgeführten Status-Labels: Ihre anderen Labels bleiben. Ein Host-Board listet die offenen Issues und die 60 zuletzt
geschlossenen.

Ein Jira-Übergang, der Pflichtfelder verlangt (etwa eine Lösung), wird von Jira abgelehnt, und die Karte springt mit Jiras
eigener Meldung zurück: Erledigen Sie diesen auf der Website.

Dasselbe Board steht Skripten und Agenten zur Verfügung: `gitpoulpi hosting board` gibt die Spalten mit ihren Karten aus
und `gitpoulpi hosting board move <card> <column>` verschiebt eine (siehe [die CLI-Referenz](cli-reference.md)).

Eine Karte zeigt die Nummer, den Titel, die Zugewiesenen als Initialen, die Labels und den Meilenstein. Ziehen Sie sie in eine andere Spalte,
oder `Shift+Links` / `Shift+Rechts`, oder **Board: Karte verschieben nach...** (`m`). Sie bewegt sich sofort; nach etwa 150 ms zeigt ein Spinner,
dass der Host noch arbeitet, und lehnt der Host ab, springt die Karte zurück, und die Statuszeile sagt, warum. `Enter` öffnet
die [Unterhaltung](conversations.md) des Issues in GitPoulpi (bei lokalen Issues die Notiz), und das Board kommt zurück, wenn Sie sie schließen; `e` bearbeitet Zugewiesene, Labels und Meilenstein; `n` beginnt ein neues Issue in
der ausgewählten Spalte; `a`, `l`, `M` und `/` filtern nach Zugewiesenen, Label, Meilenstein und Text, `x` setzt die Filter zurück; `r`
liest den Host erneut. Das Board liest erneut, wenn sich die Listen der Seitenleiste aktualisieren. Das Radar hat kein Board.

## Einen Pull Request erstellen

Öffnen Sie das Formular mit dem `+` in der Kopfzeile PULL REQUESTS, `n` im Abschnitt oder
dem Palettenbefehl *Pull Requests: Pull Request erstellen...*. Ein Branch-Menü hat auch
**Pull Request von <Branch> starten...**.

Das Formular hat:

- Einen Tab pro verbundenem Profil, das Pull Requests hat (`Left` / `Right` auf der
  Tab-Leiste).
- Kombinationsfelder **Quell-Repo** / **Branch** und **Ziel-Repo** / **Branch**. Lange Listen
  sind durchsuchbar. Sie können einen Branch oder ein Repository eintippen, das die
  Listen nicht zeigen.
- **Titel**, gefüllt mit dem Betreff des obersten Commits.
- **Beschreibung**, mehrzeilig (`Enter` fügt eine Zeile hinzu, `Tab` verlässt das Feld).
- Schaltflächen **Zugewiesene**, **Reviewer**, **Labels** und **Meilenstein** (GitHub,
  GitLab, Gitea und Forgejo), die die Auswahlliste öffnen.
- **Als Entwurf einreichen**.
- Bei eingeschalteter [KI](ai.md): **Titel und Beschreibung erzeugen**.

Das Formular öffnet sich mit dem Profil, Repository und den Branches des ausgecheckten
Branches (`feature -> main`). Ein anderes Quell-Repository erzeugt einen Request aus einem
Fork.

**Pull Request erstellen** (oder `Ctrl+S`) legt den Request beim Host an. Das Ergebnis
öffnet sich in Ihrem Browser, und die Statuszeile nennt es.

## Ein Issue erstellen

Hat der Abschnitt ISSUES einen Anbieter **und** ein Projekt, hat seine Kopfzeile ein `+`
(*+ Neues Issue*). Außerdem: `n` im Abschnitt oder der Palettenbefehl *Hosting: Neues
Issue...*.

Das Formular hat **Titel** (Pflicht), **Beschreibung** (mehrzeilig) und **Labels**
(durch Kommas getrennt, zum Beispiel `bug, ui`; bei GitHub, GitLab, Gitea und Forgejo öffnen stattdessen die
Schaltflächen Labels, Zugewiesene und Meilenstein die Auswahlliste). Jira und Azure DevOps fügen eine
Dropdownliste **Typ** hinzu. Trello fügt eine Dropdownliste **Liste** hinzu. Linear und YouTrack
fügen eine Dropdownliste **Status** hinzu (der Workflow-Zustand oder der Wert `State`, in dem das Issue
beginnt). Redmine fügt eine Dropdownliste **Typ** mit den Trackern des Projekts hinzu (Bug, Feature,
Support...); es hat keine Labels, also keine Schaltfläche Labels, und ein auf der Kommandozeile
angegebenes Label wird abgelehnt. Das Formular füllt diese Dropdownlisten beim Öffnen vom Host.

**Issue erstellen** (oder `Ctrl+S`) legt es beim Host an. `Esc` oder **Abbrechen**
schließt das Formular. Lehnt der Host ab (Token abgelehnt, Berechtigung fehlt, unbekanntes
Projekt, unbekanntes Label), zeigt das Formular den Grund und behält, was Sie getippt
haben. Bei Erfolg nennt die Statuszeile das neue Issue, und der Abschnitt ISSUES wird
aktualisiert.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Auf der eigenen Webseite des Hosts erstellen

Das Formular Neues Issue und das Formular Pull Request erstellen haben oben rechts eine
Schaltfläche **Auf <Anbieter> erstellen**. Die Taste ist `Alt+W`. Sie öffnet das
Webformular des Hosts in Ihrem Browser, ausgefüllt mit dem, was Sie bisher getippt haben.
Der Dialog bleibt offen. Sie braucht kein Token und macht keinen API-Aufruf, funktioniert
also auch nach einem API-Fehler.

- GitHub hat keinen URL-Parameter für *Entwurf*, daher wird das Entwurf-Kästchen nicht
  übernommen. GitLab erhält das Titelpräfix `Draft: `.
- GitLab nimmt Labels nur per ID, daher werden sie am Ende der Beschreibung als
  Schnellaktion `/label ~"x"` angehängt.
- Eine sehr lange Beschreibung wird gekürzt, damit die URL unter 6000 Zeichen bleibt,
  mit einem Hinweis `[...truncated: paste the rest]`.
- Bitbucket Data Center hat keine Adresse für ein Webformular, also auch keine
  Schaltfläche **Auf <Anbieter> erstellen**.

CLI: `gitpoulpi hosting issue create ... --web` und `gitpoulpi hosting pr --web` geben
dieselbe URL aus.

## Der Chip der Symbolleiste

Am Ende der Symbolleiste (standardmäßig rechts) zeigt ein Chip den Request des ausgecheckten Branches:
`GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) oder `BB #12` (Bitbucket).
Seine Farbe zeigt den Zustand: grün offen, grau Entwurf, lila gemergt, rot geschlossen.

- Fahren Sie darüber für den Titel, den Zustand und `Quelle → Ziel`.
- Klicken Sie darauf oder drücken Sie `Ctrl+G g`, um den Request im Browser zu öffnen.
- Rechtsklick darauf für **Im Browser öffnen**, **URL kopieren**, **Aktualisieren** und
  **Profil wechseln**.

Ein gepushter Branch ohne Request zeigt einen blassen Chip `+ PR`. Klicken Sie darauf, um
einen Pull Request zu starten. Es gibt keinen Chip, wenn kein Profil passt, der Branch
nicht gepusht ist oder der Host nicht erreichbar war. In einem schmalen Fenster wird der
Chip ausgeblendet, um Platz für die Schaltflächen der Symbolleiste zu lassen.

## Wann sich die Listen aktualisieren

Die Listen aktualisieren sich:

- wenn Sie ein Repository öffnen;
- wenn sich der ausgecheckte Branch ändert;
- nach einem Push oder einem Fetch;
- alle *Aktualisierung von Pull Requests und Issues (Minuten)*
  (`general.hosting_refresh_minutes`, Standard 5, `0` schaltet den Timer ab;
  Einstellungen > Integrationen);
- wenn Sie die Einstellungen schließen;
- wenn Sie `r` drücken, *Integrationen: Pull Requests und Issues aktualisieren* ausführen oder
  einen Filter ändern.

Anfragen blockieren nie das Fenster.

Hosts begrenzen, wie viele Aufrufe Sie machen dürfen. GitPoulpi sendet den `ETag` der
letzten Antwort zurück, sodass eine unveränderte Liste wenig kostet (bei GitHub nichts).
Meldet ein Host, dass nur noch wenige Aufrufe übrig sind, sagt die Statuszeile das. Sind
keine mehr übrig, fragt GitPoulpi diesen Host nicht mehr, bis die Grenze zurückgesetzt
wird, und die Listen bleiben, wie sie sind.

## Terminologie

GitHub, Azure DevOps und Bitbucket sagen „Pull Request“. GitLab sagt „Merge Request“.
*Begriff für Review-Anfragen* (`ui.pr_terminology`, unter Einstellungen > Integrationen)
wählt das Wort für jede Beschriftung, jedes Menü, jeden Palettenbefehl und jede Meldung:

- **Auto (je nach Host)**: folgt dem verknüpften Anbieter des Repositorys („Pull
  Request“, solange nichts verknüpft ist).
- **Pull Request** oder **Merge Request**: immer dieses Wort.

Die Änderung gilt sofort. Die CLI und MCP sagen immer „pull/merge request“. Diese Seite
sagt „Pull Request“.

## Referenzen in Commit-Nachrichten und Branch-Namen

Sind die Issues oder Pull Requests des Repositorys verknüpft, werden Referenzen in der
Commit-Nachricht der Commit-Details unterstrichen. Ein Klick öffnet die verknüpfte Seite.
Tastatur: der Palettenbefehl *Integrationen: Commit-Referenz öffnen (#Issue / !Request)...* (ein
Menü, wenn die Nachricht mehrere Referenzen hat). Ein Repository ohne Verknüpfung hat
keine Links.

Was als Referenz zählt, hängt vom Anbieter des Abschnitts ISSUES ab:

| ISSUES verknüpft mit | In einer Nachricht | In einem Branch-Namen |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` und `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, nur wenn `ABC` der Schlüssel des verknüpften Projekts ist. Text wie `UTF-8`, `SHA-256` oder `ISO-8601` ist nie ein Link. | `ABC-123` irgendwo im Namen (`feature/ABC-123-x`), in Großbuchstaben |
| Linear | `ENG-123`, nur wenn `ENG` der Schlüssel des verknüpften Teams ist (oder eines Teams der Teamliste des Profils). `UTF-8` ist nie ein Link. | Linears eigener `branchName`, wenn Sie den Branch aus einem Issue erstellen; `ENG-123` irgendwo im Namen wird zurückgelesen |
| YouTrack | `ABC-123`, nur wenn `ABC` das Kürzel des verknüpften Projekts ist (oder eines Projekts der Projektliste des Profils) | `ABC-123` irgendwo im Namen |
| Redmine | `#123`, nur wenn Redmine die verknüpfte ISSUES-Quelle ist (eine Pull-Request-Quelle behält ihr eigenes Zeichen) | `123-fix-login` |
| Trello | eine Adresse `https://trello.com/c/<shortLink>`; `#12` nur, wenn eine zwischengespeicherte Karte die Nummer 12 hat | `12-fix-login` |

| PULL REQUESTS verknüpft mit | In einer Nachricht |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Passen beide Listen auf `#N`, gewinnt das Issue. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**In Unterhaltungen.** Der Text eines Issues oder Requests wird mit der Grammatik seines Hosts gelesen, die weiter ist als die Tabelle oben: `owner/repo#7`, `!7`, `GH-7`, `@login`, Commit-IDs und die anderen unter [Querverweise](conversations.md#querverweise) genannten Formen. Die verknüpften Quellen des Repositorys werden zuerst abgeglichen, sodass ein Jira-Schlüssel in einem GitHub-Kommentar sein Issue weiterhin in der App öffnet. `gitpoulpi hosting markdown --body TEXT --id I` zeigt, was die App in einem Text findet.

### Eine Issue-Referenz einfügen

Im Commit-Composer öffnet die Schaltfläche `#▾` am rechten Ende der Zusammenfassung eine
filterbare Liste der verknüpften Issues. Das vom aktuellen Branch genannte Issue steht
zuerst, markiert mit *(vom Branch)*. `Enter` oder ein Klick fügt seine Referenz (`ABC-123`,
`#12`, `AB#12`) am Cursor ein. Tastatur: `Alt+R` im Composer. Palette: *Commit:
Issue-Referenz einfügen...*. Ohne verknüpfte Issue-Liste ist die Schaltfläche ausgegraut
und sagt *zuerst in der Seitenleiste eine Issue-Liste verknüpfen*. GitPoulpi fügt nie von
selbst eine Referenz hinzu.
