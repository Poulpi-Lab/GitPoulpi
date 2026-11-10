[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · **Deutsch** · [Español](../es/troubleshooting.md) · [Français](../fr/troubleshooting.md) · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · [Polski](../pl/troubleshooting.md) · [Português (Brasil)](../pt-BR/troubleshooting.md) · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Fehlerbehebung und FAQ

Beginnen Sie mit den kurzen Antworten unten; die längeren Abschnitte behandeln Git LFS und
die Graph-Darstellung. Hilft nichts davon, legen Sie ein Issue im GitHub-Repository an, mit
Ihrem Betriebssystem, Ihrem Terminal, `gitpoulpi --version` und dem, was Sie getan haben.

## Wenn etwas fehlschlägt

Ein fehlgeschlagener Vorgang zeigt kein Pop-up und keine umrahmte Karte: Die **Statuszeile** sagt es, in Rot, mit Titel und Grund (*error: Fetch failed: The remote could not be reached.*), und es bleibt dort, bis eine andere Meldung es ersetzt. Die Schaltfläche Aktivität an ihrem rechten Ende zählt die Fehler, die Sie noch nicht angesehen haben. Ein Klick auf die rote Meldung oder `Aktivitätsprotokoll: Letzten Fehler öffnen` in der Palette öffnet das Aktivitätsprotokoll bei diesem Eintrag. Jeder von Ihnen ausgelöste Fehler funktioniert so, nicht nur Git-Operationen:

- **Git-Operationen**: ein Fetch, ein Push, ein Pull, ein Checkout, ein Merge, ein Rebase, ein Commit, ein Stash...
- **Ein Klon** (und das Anlegen eines neuen Repositorys): „Klonen fehlgeschlagen“.
- **Ein Aufruf an GitHub oder GitLab**: die Repository-Listen der Auswahlfenster und des Dialogs Klonen, das Erstellen eines Pull Requests oder Issues, das Prüfen eines Tokens oder SSH-Schlüssels auf der Seite Integrationen: „Anfrage an den Hoster fehlgeschlagen“.
- **Ein externes Programm**: der Editor, das Diff- oder Merge-Tool, der Datei-Explorer, der Browser, das Terminal oder der Starter, ein Bildeditor, die *Testen*-Schaltflächen der Einstellungen: „Externes Programm fehlgeschlagen“.
- **Eine KI-Anfrage** (eine Commit-Nachricht, eine Erklärung, ein Vorschlag für einen Konflikt...): „KI-Anfrage fehlgeschlagen“.
- **Eine Datei, die nicht geschrieben werden konnte**: Speichern im eingebauten Editor, der Merge-Editor, eine Einstellung, die nicht gespeichert werden konnte: „Speichern fehlgeschlagen“.

Die rote Meldung steht in der Statuszeile, welche Ansicht auch vorne ist: ein Repository-Tab, Home, das Radar, die Workspaces, die Einstellungen (in der untersten Zeile des Bildschirms), unter dem Aktivitätsprotokoll oder einem Dialog. Wo die Statuszeile verdeckt ist (Home, Radar, Workspaces, Einstellungen), wird die Meldung über ihre unterste Zeile geschrieben, bis Sie den Fehler angesehen haben. Sie folgt Ihnen von einem Tab zum anderen. Ein Vorgang, der beim Tabwechsel noch läuft, wird nicht verfolgt: Öffnen Sie das Aktivitätsprotokoll, um zu sehen, wie er endete.

Die Listen der Abschnitte PULL REQUESTS und ISSUES, die sich nicht aktualisieren lassen, behalten stattdessen ihre eigene Zeile im Abschnitt (sie aktualisieren sich von selbst, eine Meldung alle paar Sekunden wäre nur Lärm). Ein noch offener Dialog (Klonen, Pull Request erstellen...) behält ebenfalls seine eigene einzeilige Meldung, damit Sie das Feld korrigieren und es erneut versuchen können.

Der Eintrag im Aktivitätsprotokoll enthält alles, was das Pop-up früher zeigte:

- **Titel und Grund.** „Fetch fehlgeschlagen“, dann ein Satz, der in einfachen Worten sagt, warum, dann, was als Nächstes zu tun ist. Erkannt werden: Das Remote hat Commits, die Sie nicht haben (*erst Pull*), Zugangsdaten abgelehnt (*prüfen Sie Einstellungen > Integrationen oder Ihren SSH-Schlüssel*), ein Halt bei Konflikten, lokale Änderungen, die überschrieben würden (*committen, stashen oder verwerfen*), das Remote nicht erreichbar und ein anderer Git-Prozess, der eine Sperre auf dem Repository hält. Alles andere zeigt die erste Zeile von Git selbst. Ein Klon oder ein Hoster-Aufruf, der genauso scheitert (Zugangsdaten abgelehnt, Host nicht erreichbar), wird gleich formuliert; andere Fehler bekommen einen Satz dazu, was fehlgeschlagen ist („Das Repository konnte nicht geklont werden.“, „Das Programm konnte nicht ausgeführt werden.“).
- **Die Details**, sichtbar, wenn der Eintrag offen ist (`Enter` oder der Klick, der ihn geöffnet hat): den Vorgang, den ausgeführten Git-Befehl, seinen Exit-Code und die letzten Zeilen seiner Ausgabe (bei der eingebauten Git-Bibliothek ihre Fehlerklasse und ihren Code). Zugangsdaten, Tokens und `Authorization`-Header werden maskiert, bevor etwas angezeigt, protokolliert oder kopiert wird.
- **Kopieren** (die Schaltfläche oder `c`) legt einen Textblock in die Zwischenablage, gemacht zum Einfügen in ein Gespräch mit jemandem oder einer KI, die ihn erklären kann: GitPoulpis Version, das System, das Terminal, das Backend, der fehlgeschlagene Vorgang mit seinen Details und die Git-Version.
- **Eine Schaltfläche für den nächsten Schritt**, wenn es einen gibt, neben Kopieren (oder `r`): *Erst Pull* (ein Push, den das Remote ablehnen würde), *Stash und wiederholen* (lokale Änderungen stehen einem Checkout, Merge, Rebase oder Pull im Weg), *Wiederholen* (das Remote war nicht erreichbar), *Integrationen öffnen* (Zugangsdaten). Sie führt den Schritt aus und schließt das Fenster. Bei einem Fehler in einem anderen Repository als dem des vordersten Tabs wird nur *Integrationen öffnen* angeboten: Eine Wiederholung würde im falschen Repository laufen.
- Ohne Maus bietet die Befehlspalette `Aktivitätsprotokoll: Letzten Fehler öffnen`, `Aktivitätsprotokoll: Letzten Fehler kopieren` und `Aktivitätsprotokoll: Vorgeschlagene Aktion ausführen` (die Schaltfläche des Eintrags für den Fehler, den die Statuszeile zeigt).

Ein Push, den das Remote abgelehnt hat, weil es weitergezogen ist, behält seinen eigenen Dialog (erst Pull, Force Push mit Lease, Abbrechen); die Fakten stehen trotzdem im Aktivitätsprotokoll.

### Das Aktivitätsprotokoll

Die Schaltfläche **Aktivität** am rechten Ende der Statuszeile (eine Zahl darauf zählt die Fehler, die Sie noch nicht angesehen haben), `Ctrl+N`, dann `Shift+A`, und `Ansicht: Aktivitätsprotokoll` in der Palette öffnen das Fenster. `Ctrl+N` zeigt ein Badge an jedem Tab, Feld, Kontrollkästchen und jeder Schaltfläche des Fensters (und am ersten Eintrag der Liste); das Drücken des Zeichens tut, was ein Klick darauf tut. Zwei Tabs, **Git-Vorgänge** und **Anwendung**, gewechselt mit `Tab`, per Klick, `g` (Git-Vorgänge) oder `p` (Anwendung), und darunter ein Feld **Bereich**, das für beide gilt (siehe unten):

- **Anwendung**: die eigenen Protokollzeilen des Programms, die des Fensters und des Hintergrund-Daemons, neueste zuerst. Das Feld **Stufe** (eine Auswahlliste: Fehler, Warnung, Info, Debug, Trace; `l` öffnet sie) bestimmt, wie viel gezeigt wird; wer mehr Details verlangt, als aufgezeichnet wird, hebt auch die Aufzeichnungsstufe an (und speichert sie als `log.level`). Darunter hat **Anzeigen** ein Kästchen pro Stufe (Fehler, Warnung, Info, Debug, Trace, jedes mit der Zahl der Zeilen, die es enthält), um die Zeilen dieser Stufe aus- oder einzublenden, zum Beispiel um nur die Fehler zu sehen, und **Alle** / **Keine** setzen oder löschen jedes Kästchen. Die Tasten `1` bis `5` schalten die Stufen in dieser Reihenfolge um, `a` ist Alle, `n` ist Keine; die Palette bietet `Aktivitätsprotokoll: Stufe ein- oder ausblenden: ...`, `Aktivitätsprotokoll: alle Stufen anzeigen` und `Aktivitätsprotokoll: keine Stufe anzeigen`. Stufen, die ausführlicher sind als die Wahl im Feld Stufe, sind ausgegraut (für sie wird nichts aufgezeichnet; fahren Sie mit der Maus darüber, um den Grund zu sehen). Die Haken werden bis zum Beenden gemerkt, und nur der Tab Anwendung hat sie. **Filter** (`/`) behält die Zeilen, die Ihren Text enthalten. **Alles kopieren (gefiltert)** kopiert, was gezeigt wird (Haken und Filter gelten beide); **Protokollordner öffnen** öffnet den Ordner in Ihrem Dateimanager. Eine lange Zeile wird nie abgeschnitten: siehe „Lange Zeilen“ weiter unten.

  ![Das Aktivitätsprotokoll, Tab Anwendung: das Feld Stufe, die Kästchen Anzeigen pro Stufe mit Alle und Keine, der Filter und die Protokollzeilen](../images/activity-log.png)

**Protokoll leeren** (`x`, oder der Palettenbefehl `Aktivitätsprotokoll: Anwendungsprotokoll leeren`) steht auf dem Tab **Anwendung**: es fragt zuerst nach und leert dann die Protokolldateien aller aufbewahrten Tage und die Liste auf dem Bildschirm. Das lässt sich nicht rückgängig machen und steht nicht im Undo-Verlauf.

- **Git-Vorgänge** (der Tab, auf dem sich das Fenster öffnet): die Vorgänge, neueste zuerst, jeder mit der Zeit, zu der er lief, und der Dauer (`Push - 123 ms`): Repository öffnen, Commit-Graph laden, Fetch, Pull, Push, Commit, Checkout, Merge, Rebase, Stash, Klonen. Fehlschläge, Statusmeldungen, ein fehlgeschlagener Klon und das Fetch all eines Workspaces stehen hier ebenfalls; `Enter` oder ein Klick öffnet einen Eintrag (sein Grund, nächster Schritt, Befehl, Exit-Code und seine Ausgabe), `c` oder seine Schaltfläche Kopieren kopiert ihn, `r` oder seine Schaltfläche führt den vorgeschlagenen Schritt aus. `Tab` (oder `p`) wechselt zum Tab Anwendung; `Left`/`Right` scrollen lange Zeilen seitlich.

  ![Das Aktivitätsprotokoll, Tab Git-Vorgänge: jeder Vorgang mit seiner Zeit und Dauer, ein fehlgeschlagener Fetch in Rot](../images/activity-log-repository.png)

- **Lange Zeilen** (beide Tabs): Eine Kopf- oder Detailzeile, die breiter als das Fenster ist, wird nie abgeschnitten. Unter der Liste erscheint eine horizontale Scrollleiste; scrollen Sie seitlich mit `Left` / `Right` (umbelegbar als `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + Mausrad oder einem horizontalen Mausrad, den Palettenbefehlen `Aktivitätsprotokoll: nach links scrollen` / `Aktivitätsprotokoll: nach rechts scrollen` oder der Leiste (den Griff ziehen oder auf die Bahn drücken, um den Griff unter den Zeiger zu holen). `Home` geht zurück zum Anfang, und ein Tab-Wechsel startet den anderen Tab am linken Rand. Umbrochener Text (eine Zusammenfassung, eine vorgeschlagene Aktion) bleibt umbrochen.

- **Bereich** (eine Auswahlliste unter den Tabs, `s` öffnet sie, oder `Aktivitätsprotokoll: Bereich wählen` in der Palette): worum es auf beiden Tabs geht. Die Zeilen sind **Dieses Repository (Name)** (das Repository des Tabs, aus dem Sie das Protokoll geöffnet haben; von Home aus, das keines hat, nicht angeboten), **Alle Repositories**, jeder **Workspace** und jedes Repository, das das Fenster kennt. Aus einem Repository-Tab öffnet sich das Protokoll auf *Dieses Repository*, von Home aus (die Schaltfläche Aktivität, `Ctrl+N` dann `Shift+A`) auf *Alle Repositories*; `Ctrl+N` dann `Shift+G` und `Ansicht: Globales Aktivitätsprotokoll` in der Palette öffnen immer auf *Alle Repositories*, ebenso ein Klick auf den roten Fehler eines Vorgangs, dessen Repository nicht der vorderste Tab ist (etwa ein fehlgeschlagener Klon). Ihre letzte Wahl wird für die Sitzung gemerkt, getrennt für die beiden Arten des Öffnens. Auf dem Tab Anwendung gehört eine Zeile zu einem Repository, wenn das Programm sie über eines geschrieben hat (ein fehlgeschlagener Push, die Zeit eines Vorgangs); unter *Alle Repositories* sagt jede solche Zeile, wessen sie ist (`[name]`). Eine Zeile über gar kein Repository (Start, Einstellungen, der Daemon selbst) und ein Eintrag, der zu keinem gehört (ein fehlgeschlagenes Speichern), sind **anwendungsweit**: Sie erscheinen unter jedem Bereich, mit `[app-wide]` markiert, wenn der Bereich enger ist als *Alle Repositories*, sodass ein Filter nie einen Fehlschlag verbirgt. Die Palette hat außerdem `Aktivitätsprotokoll: Git-Vorgänge` und `Aktivitätsprotokoll: Anwendungsprotokoll` für die Tabs.

- **Diagnose kopieren** (eine Schaltfläche auf beiden Tabs, `d`, immer das ganze Protokoll, was der Bereich auch ist; und `Ansicht: Diagnose kopieren` in der Palette) kopiert Version, System, Terminal und Backend, den letzten Fehlschlag mit seinen Details und die letzten 200 Protokollzeilen auf der gewählten Stufe: den Block, den man einer KI gibt, wenn etwas nicht stimmt.

Die Dauer misst das Fenster (von der Anfrage bis zur Antwort des Daemons, die Laufzeit eingeschlossen); was die CLI oder MCP tut, wird also nicht gemessen. `log.timings` (standardmäßig an) schaltet die Dauer aus, `log.extended` (standardmäßig aus) listet auch die Hintergrundarbeit auf, die periodische Aktualisierung und Auto-Fetch; beide stehen unter Einstellungen > Allgemein neben den übrigen Protokolleinstellungen, und jeder Vorgang wird ohnehin ins Anwendungsprotokoll geschrieben (`info` für das, was Sie verlangt haben, `debug` für den Hintergrund).

Protokolldateien heißen `gitpoulpi-YYYY-MM-DD.log`, werden standardmäßig eine Woche und höchstens zehn Jahre aufbewahrt (`log.retention_days`, 1 bis 3650), beim Start von GitPoulpi bereinigt und bei `log.max_file_mb` (5 MiB) rotiert. Wo sie liegen: [Konfigurationsreferenz](config-reference.md#log-das-anwendungsprotokoll). Aus einem Skript trägt der `--json`-Fehler eines fehlgeschlagenen Befehls dieselben Fakten unter `detail`; siehe die [CLI-Referenz](cli-reference.md).

Ein Klick auf die rote Meldung öffnet das Protokoll auf *Alle Repositories* mit diesem Eintrag geöffnet. Die Liste der Git-Vorgänge gehört zur Sitzung; der Tab Anwendung behält das Protokoll des Programms über Neustarts hinweg.

**Hintergrundjobs.** Fetch, Push, Pull, Klonen oder LFS-Übertragung, das Fetch all / Pull all / Clone missing eines Workspaces, eine KI-Anfrage und ein gerade gesendeter Pull Request oder Issue sind Jobs: Das rechte Ende der Statuszeile zeigt den laufenden mit einem `[x]`, das ihn abbricht; ein Klick auf die Beschriftung (oder `Ctrl+N` dann `Shift+J`, der Paletteneintrag `Aufgaben: Hintergrundaufgaben anzeigen`) öffnet das Panel mit einer Abbrechen-Schaltfläche je Job (auch `c` oder `Entf`). Ein Klon läuft im Hintergrund, eine KI-Anfrage und ein Workspace-Durchlauf überstehen einen Tab-Wechsel und werden angewendet, wenn Sie zu ihrem Repository zurückkehren. Ein Job hinter dem Punkt, ab dem Stoppen nicht mehr sicher ist, ist ausgegraut und nennt den Grund; ein abgebrochener Job lässt das Repository unverändert.

## Kurze Antworten

**Die Symbolleiste und die Seitenleiste zeigen Kästchen oder Fragezeichen statt
Symbolen.** Nutzen Sie in Ihrem Terminal eine Nerd Font oder eine Schrift mit
Emoji-Abdeckung. Der Graph und der Text funktionieren auch ohne sie.

**Der Commit-Graph besteht aus Zeichen, nicht aus Bildern.** Das ist der automatische
Rückfall: Bilder brauchen ein Terminal mit dem Kitty-Grafikprotokoll (kitty, Ghostty, herdr
mit eingeschalteter Kitty-Grafik) oder dem iTerm2-Protokoll (WezTerm, iTerm2, mintty) oder Sixel (Windows Terminal, foot, Konsole). Die Windows-Konsole, tmux und zellij bekommen Zeichen. Siehe den Abschnitt zum Graphen unten,
um das eine oder andere zu erzwingen.

**Die Maus tut nichts, oder das Markieren von Text mit der Maus funktioniert nicht.**
GitPoulpi fängt die Maus ab, damit Klicks es erreichen. Um Text mit dem Terminal selbst zu
markieren, halten Sie `Shift` beim Ziehen gedrückt (in den meisten Terminals). Sie können
auch im Diff markieren (ziehen, dann `y`). Um das Abfangen der Maus abzuschalten, nutzen Sie
*Einstellungen > Tastatur & Maus > Maus aktiviert* (Neustart nötig). Die Tastatur erreicht
jeden Befehl.

**Ein Tastenkürzel funktioniert nicht.** Ihr Terminal, tmux oder ein AutoHotkey-Skript kann
die Taste abfangen, bevor GitPoulpi sie sieht (`Ctrl+,` kommt in Linux- und
macOS-Terminals oft als einfaches `,` an; `Ctrl+Shift+N` sieht aus wie `Ctrl+N`). Nutzen Sie
die Alternativen (`Ctrl+N` dann `,` öffnet die Einstellungen; `Ctrl+P` erreicht jeden Befehl)
oder belegen Sie die Tastenfolge neu: siehe [Einstellungen](settings.md#tastatur--maus).

**Ich weiß ein Tastenkürzel nicht mehr.** Drücken Sie `Ctrl+P` und tippen Sie, was
Sie wollen; jedes Ergebnis zeigt sein Tastenkürzel. `?` listet alles auf.

**Eine Option ist in den Einstellungen ausgegraut.** Sie hängt von einer anderen Option ab.
Die Zeile sagt, von welcher (zum Beispiel braucht *Stärke der Zeilentönung*, dass *Hintergrund
der Graph-Spuren* auf *Zeilentönung* steht).

**„Auto-Stash ausstehend“ über dem Log.** GitPoulpi hat Ihre Änderungen gestasht, um eine
Operation auszuführen, und konnte sie nicht zurückspielen. Nutzen Sie **Jetzt poppen**,
**Stash zeigen**, **Neu beginnen** (die Operation rückgängig machen und Ihre Änderungen
zurückholen) oder **Drop** des Banners; siehe [Auto-Stash](log-and-history.md#auto-stash).

**Ein Agent meldet `no_viewer` oder `deferred`.** `no_viewer`: Für dieses Repository ist kein
GitPoulpi-Fenster offen (oder *KI-Agenten die Ansicht steuern lassen* ist aus). `deferred`:
Das Fenster zeigt einen Chip; `Ctrl+N Enter` nimmt ihn an. Siehe
[MCP und Agenten](mcp-and-agents.md#dem-benutzer-dinge-zeigen).

**Ich habe GitPoulpi geöffnet und sehe ein leeres Fenster oder `log error: backend error`.**
Prüfen Sie, dass Sie es in einem Git-Repository gestartet haben (sonst landen Sie im Home-Tab:
nutzen Sie **Suchen**, **Klonen** oder **Init**). Führen Sie dann `gitpoulpi daemon status`
aus; `gitpoulpi daemon restart` startet den Hintergrundprozess für dieses Repository neu.
Drücken Sie `r` zum Aktualisieren.

**Zwei Fenster auf demselben Repository verhalten sich seltsam.** Sie teilen absichtlich einen
Daemon pro Repository, sodass eine Änderung im einen im anderen erscheint. Scheint ein Daemon
zu hängen, beendet `gitpoulpi daemon stop --all` den Daemon jeder Sitzung; das nächste Fenster
startet einen frischen.

**Ein Commit, Pull oder Push fragt nach einem Passwort oder scheitert an der Authentifizierung.**
GitPoulpi nutzt für das Netz den Credential-Helper und den ssh-agent Ihres `git`. Stellen Sie
zuerst sicher, dass `git fetch` in einem einfachen Terminal funktioniert. Die Tokens für
GitHub/GitLab für Pull Requests sind getrennt und optional: siehe
[Hosting und Issue-Tracker](hosting-integration.md).

**Ein Fetch, Pull oder Push scheitert mit „Der konfigurierte Credential-Helper ist fehlgeschlagen“, „Der SSH-Hostschlüssel des Remotes ist noch nicht bekannt“ oder einer abgelehnten Anmeldung, und sein Eintrag im Aktivitätsprotokoll hat eine Schaltfläche „SSH & Zugangsdaten...“.**
Der Fehler kommt daher, wie sich dieses Repository anmeldet. Die Schaltfläche öffnet
Einstellungen > SSH & Zugangsdaten, wo Sie für dieses Repository eine andere Quelle wählen:
die eigenen Helper von git unter Ignorieren der Helper des Repositorys, ein
Integrationskonto, eine Schlüsseldatei (mit einer Known-Hosts-Datei) oder einen eigenen Helper
oder Token-Befehl; **Verbindung testen** prüft dann jedes Remote. Bei einem Helper, der nicht
lief, zeigt `git config --show-origin --get-all credential.helper`, wo er gesetzt ist.

**„GitPoulpi hat das Konto-Token nicht an ... gesendet“.** Das Remote des Repositorys zeigt auf
einen Server, der nicht die Adresse ist, an die das gewählte Konto gebunden ist (oft eine
Umschreibung `url.<x>.insteadOf` in der eigenen Konfiguration des Repositorys), daher wurde das
Token zurückgehalten. Korrigieren Sie das Remote oder wählen Sie auf der Seite eine andere Quelle.

**„Der Token-Befehl dieses Repositorys ist fehlgeschlagen“, „... hat sich geändert, seit Sie es bestätigt haben“, „Die Anmeldung dieses Repositorys kann nicht verwendet werden“.**
Die Wahl auf der Seite lässt sich nicht ausführen: Der Token-Befehl hat kein Token ausgegeben
(seine letzten Fehlerzeilen werden zitiert, Geheimnisse entfernt), ein Skript, das er ausführt,
hat sich geändert, seit Sie ihn bestätigt haben (bestätigen Sie den Befehl auf der Seite erneut),
oder die Wahl nennt eine Schlüsseldatei oder ein Konto, die es nicht mehr gibt. Jede dieser
Meldungen hat die Schaltfläche **SSH & Zugangsdaten...**.

**OpenSSH meldet „bad permissions“ für eine Schlüsseldatei.** Andere Konten können den
Schlüssel lesen. Auf Einstellungen > SSH & Zugangsdaten zeigt die Zeile **Datei mit privatem
Schlüssel** eine Warnung mit **Auf mein Konto beschränken**.

**„Staging...“ bleibt in der Statuszeile.** Eine Aktualisierung im Hintergrund läuft noch; sie
verschwindet, wenn sich die Dateiliste aktualisiert. `r` erzwingt eine Aktualisierung.

**Ein sehr großes Repository ist langsam.** Senken Sie *Anfängliche Commits im Graph* auf der
Seite Einstellungen > Verlauf & Graph (das Log lädt beim Scrollen mehr, wenn *Commits im Graph
verzögert laden* an ist); blenden Sie die Spalte *Änder.* des Logs aus und schalten Sie Avatare ab;
schalten Sie *Rückgängig aktiviert* ab, wenn Sie Rückgängig nicht brauchen (dann gibt es beim
Staging und bei Commits gar keine Buchführung). `GITPOULPI_TIMING=1` schreibt Zeiten pro
Anfrage nach `gitpoulpi-timing.log` im temporären Ordner, nützlich für einen
Leistungsbericht.

**Die KI-Schaltflächen fehlen oder sind grau.** Die KI ist standardmäßig aus; siehe
[KI-Unterstützung](ai.md#fehlerbehebung).

**GitPoulpi hat sich von selbst geschlossen.** Jedes Beenden wird protokolliert: Suchen Sie
`gitpoulpi-tui-exit.log` (und `gitpoulpi-tui-panic.txt` nach einem Absturz) im
Konfigurationsordner (`gitpoulpi config path` gibt die Einstellungsdatei aus, die dort liegt)
und hängen Sie sie an ein Issue an.

**Wo sind meine Einstellungen?** `gitpoulpi config path`. Der Zustand pro Repository liegt in
`.gitpoulpi/repo-state.toml` im Repository.

**Sendet GitPoulpi irgendetwas irgendwohin?** Nur, was Sie anfordern: Git-Operationen an Ihre
Remotes, Hosting-Anfragen, nachdem Sie ein Konto verbunden haben, KI-Anfragen, nachdem Sie eine
KI-Schaltfläche gedrückt haben, und, wenn Sie Avatare einschalten, ein Hash der
Autoren-E-Mails an Gravatar. Es gibt keine Telemetrie. Siehe
[KI-Unterstützung](ai.md#datenschutz).

## Jedes Schreiben scheitert: „Unable to create index.lock“

Ein abgebrochener Git-Befehl (ein Absturz, ein beendetes Terminal) kann `.git/index.lock`
zurücklassen, und dann scheitert jede Änderung am Repository. Ist die Datei älter als 10
Sekunden, zeigt GitPoulpi einen Streifen über dem Log: **index.lock entfernen** (Palette:
*Repository: Veraltete index.lock entfernen*; CLI: `gitpoulpi index-lock remove`). Er fragt
vorher nach und löscht nur diese Datei. GitPoulpi kann nicht zuverlässig erkennen, ob noch ein
anderes Git-Programm läuft; stellen Sie also sicher, dass keines läuft (eine IDE, ein anderes
Terminal), bevor Sie bestätigen. In einem verknüpften Worktree ist die Sperre die des
Worktrees, in `.git/worktrees/<name>`.

## „git am“ ist angehalten und Rebase abbrechen tut nichts

Ein `am` (Patches aus einer Mailbox anwenden) ist kein Rebase. Ein angehaltenes `git am` hat
seinen eigenen Streifen, **Patches werden angewendet (git am)**, mit Konflikte lösen, Fortsetzen,
Überspringen und Abbrechen (Palette: *git am: Fortsetzen / Diesen Patch überspringen /
Abbrechen*; CLI: `conflicts am-continue|am-skip|am-abort`). Abbrechen fragt vorher nach,
verwirft die bisher angewandten Patches und setzt den Branch dorthin zurück, wo er war. Sie
sind nicht rückgängig zu machen, wie die Rebase-Schaltflächen.

## Rückgängig ist ausgegraut und sagt „... kann nicht rückgängig gemacht werden“ (blockierter Eintrag)

Der neueste Journal-Eintrag steckt fest: Das Repository hat sich außerhalb von GitPoulpi
weiterbewegt, oder der Eintrag wurde unterbrochen. Fahren Sie über Rückgängig für den Grund;
steckt er dauerhaft fest, verweist der Tooltip auf **Bearbeiten: Blockierten Rückgängig-Eintrag vergessen**
(Palette) oder `gitpoulpi undo-forget`. Es fragt vorher nach und entfernt nur den Eintrag aus
dem Journal; das Repository wird nicht berührt. Siehe [Rückgängig und Wiederholen](undo-redo.md).

## Große Dateien erscheinen als drei Zeilen Text („LFS-Dateien erscheinen als Zeiger“)

Eine Datei, die ein Bild, ein Modell oder eine große Binärdatei sein sollte, enthält `version https://git-lfs.github.com/spec/v1` und eine Zeile `oid sha256:...`: Es ist ein Git-LFS-**Zeiger**, und der echte Inhalt wurde nicht heruntergeladen (oder die LFS-Filter sind nicht installiert).

- Sehen Sie in den Status-Bereich: Ein Repository, das LFS nutzt, während `git-lfs` fehlt, zeigt ein bernsteinfarbenes Banner. Installieren Sie das Programm (https://git-lfs.com; Windows: mit Git for Windows geliefert oder `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) und öffnen Sie neu oder drücken Sie `r`.
- `gitpoulpi lfs status` sagt, ob das Programm installiert ist, ob die Filter und der Hook eingerichtet sind und welche Muster verfolgt werden; `gitpoulpi lfs files` listet die Zeigerdateien und welche `local` sind.
- Führen Sie bei installiertem `git-lfs` in der Palette **`LFS: Objekte pullen`** aus (oder `gitpoulpi lfs pull`): Es lädt die Objekte herunter und ersetzt die Zeiger. Es installiert zuerst die LFS-Filter für dieses Repository, falls es keine gibt.
- Schlägt der Download fehl, nennt die Meldung den Grund (Zugangsdaten für das Remote, ein Server, der das Objekt nicht mehr hat). GitPoulpi braucht kein eigenes Konto: LFS nutzt die üblichen Zugangsdaten Ihres Remotes (Credential-Helper oder ssh-agent).
- Ein Klon eines LFS-Repositorys scheitert nie wegen LFS; konnte der Download-Schritt nicht laufen, ist der Klon mit Zeigern da, und `LFS: Objekte pullen` beendet ihn später. `gitpoulpi clone --no-lfs` überspringt den Download absichtlich.
- Sie können alles zu LFS unter Einstellungen > Git & Kodierung > Git LFS abschalten (oder `[git] lfs_enabled = false`).

## „git-lfs not found“ / „Git LFS is not installed“

Die Meldung `Git LFS is not installed. This repository stores large files with Git LFS; ...` (oder ein Dialog mit einer Schaltfläche *Installationsseite öffnen* oder git eigenes `git-lfs: command not found` / `'lfs' is not a git command`) bedeutet, dass das Repository für LFS konfiguriert ist - seine `.gitattributes`, die Filter in `.git/config` oder der `pre-push`-Hook rufen `git-lfs` auf - und das Programm nicht im `PATH` liegt, den GitPoulpi sieht. Installieren Sie es (Link oben). Unter Windows wird das Programm mit Git for Windows geliefert; funktioniert `git lfs version` in einem Terminal, GitPoulpi meldet es aber weiterhin als fehlend, starten Sie GitPoulpi aus einem Terminal, das denselben `PATH` sieht. Um LFS vor GitPoulpi zu verbergen und das eigene Verhalten von git in Ruhe zu lassen, schalten Sie `git.lfs_enabled` ab.

## Der Graph wird mit Zeichen gezeichnet, nicht als Bild (herdr, tmux, andere Terminals)

GitPoulpi zeichnet die Spalte Graph und die Autoren-Avatare nur dort als Pixel, wo es sicher ist, dass das Terminal sie anzeigt: iTerm2-Inline-Bilder in reinem WezTerm, iTerm2 und mintty; das Kitty-Grafikprotokoll in kitty, Ghostty und **herdr, wenn die eigene Konfiguration von herdr `kitty_graphics = true` setzt** (`[terminal]`, oder `[experimental]` bei älterem herdr) und herdr in WezTerm, kitty oder Ghostty läuft. Sixel-Bilder gibt es in Windows Terminal, foot, Konsole und mlterm. Überall sonst zeichnet es Zeichen, was immer korrekt ist.

- In herdr mit eingeschalteter Kitty-Grafik, aber einer Konfiguration, die GitPoulpi nicht findet (oder wenn man sich auf den Standard von herdr verlässt), setzen Sie `ui.image_protocol = "kitty"` (Einstellungen > Verlauf & Graph > Bildprotokoll) und starten Sie neu.
- In einem Terminal, das Kästchen oder Müll zeigt, wo der Graph sein sollte, setzen Sie `ui.image_protocol = "none"` oder `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` schaltet Bilder unabhängig von der Konfiguration ab.

**herdr remote (`herdr --remote <Host>`, oder `ssh` dann `herdr`): Der Text-Graph ist der Standard.** Über SSH läuft GitPoulpi auf dem entfernten Host und kann nur die Umgebung des entfernten Rechners und die `config.toml` von herdr lesen. Ob ein Kitty-Bild erscheint, entscheidet der *lokale* herdr-Client (seine eigene Einstellung `kitty_graphics` und sein eigenes Terminal), den GitPoulpi nicht sehen kann: Zeichnet der Client die Bilder nicht, bleiben die Zellen leer. GitPoulpi zeichnet daher den Text-Graphen, sobald es herdr über SSH erkennt (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Hat Ihr lokales herdr `kitty_graphics = true` und ein Terminal, das Kitty-Bilder zeichnet (kitty, Ghostty), schalten Sie es mit `ui.graph_renderer = "image"` (oder `ui.image_protocol = "kitty"`) auf dem entfernten Rechner ein. Bilder werden inline gesendet (`t=d`), was über jeden Byte-Strom funktioniert. Ist die Spalte leer, setzen Sie `ui.graph_renderer = "text"`.

**Ein helleres Rechteck hinter den Spuren.** Das Bild ist überall dort transparent (PNG-RGBA mit geradem Alpha), wo nichts gezeichnet ist, sodass der Hintergrund des Terminals und das Auswahlband durchscheinen. Terminals mit Kitty-Protokoll und **WezTerm** setzen dieses Alpha zusammen; GitPoulpi reserviert dann gewöhnliche leere Zellen unter dem Bild und sendet es danach. Jedes andere Terminal mit iTerm2-Protokoll (iTerm2 selbst, Rio, mintty) ist nicht dafür bekannt, PNG-Alpha zu mischen, daher trägt das Bild dort einen deckenden Hintergrund in der Farbe des Designs, der auf einem Terminal mit abweichendem Hintergrund als Rechteck sichtbar werden kann; `ui.graph_renderer = "text"` entfernt es. Mit `ui.graph_lane_background = "off"` wird auf den transparenten Pfaden nichts hinter die Spuren gemalt.

Siehe `ui.graph_renderer` und `ui.image_protocol` in der [Konfigurationsreferenz](config-reference.md).
