[English](../getting-started.md) · [Čeština](../cs/getting-started.md) · **Deutsch** · [Español](../es/getting-started.md) · [Français](../fr/getting-started.md) · [Italiano](../it/getting-started.md) · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · [Nederlands](../nl/getting-started.md) · [Polski](../pl/getting-started.md) · [Português (Brasil)](../pt-BR/getting-started.md) · [Português (Portugal)](../pt-PT/getting-started.md) · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · [简体中文](../zh-CN/getting-started.md)

# Erste Schritte

Diese Seite führt Sie von einer frischen Installation bis zu Ihrem ersten
Commit. Der [Rundgang durch die Oberfläche](interface-tour.md) benennt jeden
Bereich des Fensters. Die [Einstellungen](settings.md) listen jede Option auf.

## 1. Installieren und starten

Folgen Sie [Installation](installation.md). Öffnen Sie dann ein Terminal in
einem Git-Repository und starten Sie:

```
gitpoulpi
```

GitPoulpi öffnet das Repository,
das den aktuellen Ordner enthält. Außerhalb eines Repositorys öffnet es den
**Home**-Tab, in dem Sie ein Repository durchsuchen, klonen oder initialisieren
können (siehe [Schritt 7](#7-repositories-öffnen-klonen-und-initialisieren)).

Die Symbole brauchen eine Nerd Font oder eine Schrift mit Emoji. Sehen Sie
Kästchen, lesen Sie [Fehlerbehebung](troubleshooting.md).

![Das Hauptfenster: links die Refs-Seitenleiste, in der Mitte der Log-Graph, rechts die Änderungen mit dem Commit-Formular](../images/hero.png)

## 2. Drei Tasten, um alles andere zu lernen

| Taste | Öffnet |
|---|---|
| `?` | Die **Hilfe**: jede Taste und jede Mausgeste. Tippen Sie, um zu filtern. `Ctrl+D` öffnet diese Dokumentation im Browser. |
| `Ctrl+P` | Die **Befehlspalette**: Tippen Sie, um einen Befehl zu finden, `Enter` führt ihn aus. Jede Zeile zeigt ihr Tastenkürzel. |
| `Ctrl+,` | Die **Einstellungen**. Das Zahnrad in der Titelleiste, links neben dem ×, öffnet sie ebenfalls. |

Sie können alles mit der Tastatur oder mit der Maus erledigen. Die Palette
erreicht jeden Befehl. Siehe [Tastenkürzel](keybindings.md) und
[Maus und Gesten](mouse-and-gestures.md).

![Die Befehlspalette, gefiltert nach dem Wort stage, mit den Stage-Befehlen und ihren Tastenkürzeln](../images/palette.png)

## 3. Den Verlauf ansehen

Die Mitte des Fensters ist das **Log**: eine Zeile pro Commit, der neueste
zuerst. Es zeigt Branches und Tags als farbige Pillen, den Graphen, die
Nachricht, den Autor, das Datum und den kurzen Hash.

- Klicken Sie auf einen Commit. Seine Details und geänderten Dateien erscheinen
  rechts.
- Klicken Sie auf eine geänderte Datei. Ihr Diff öffnet sich in der Mitte.
  `Esc` schließt ihn.
- `j` / `k` oder die Pfeiltasten bewegen die Auswahl. `f` durchsucht die
  Commits.
- Rechtsklick auf einen Commit öffnet sein Menü.

![Das Log mit ausgewähltem Merge-Commit: rechts seine Nachricht, sein Autor und die zwei Dateien, die er hinzufügt](../images/log-graph.png)

## 4. Etwas ändern und stagen

Ändern Sie eine Datei in Ihrem Editor. Die **oberste Zeile des Logs** zeigt
jetzt die Änderungen: eine gestrichelte Zeile mit der Anzahl geänderter
Dateien. Klicken Sie darauf. Der rechte Bereich zeigt **Unstaged Dateien** und
**Staged Dateien**.

- Fahren Sie über eine Datei und klicken Sie auf **Datei stagen**. Oder
  klicken Sie auf **Alle Änderungen stagen**.
- Klicken Sie auf einen Dateinamen, um den Diff zu öffnen. Dort können Sie
  einen **Hunk** (**Hunk stagen**) oder eine einzelne **Zeile** stagen.

Siehe [Staging und Committen](staging-and-committing.md).

![Der Diff einer geänderten Datei, mit den Schaltflächen Hunk stagen und Hunk verwerfen über jedem Hunk](../images/diff.png)

## 5. Committen

Geben Sie im Commit-Formular unten rechts eine Zusammenfassung und optional
eine Beschreibung ein. Klicken Sie dann auf die grüne Schaltfläche
**Änderungen an N Dateien committen** oder drücken Sie `Ctrl+Enter` (oder `Ctrl+G Enter`, von überall aus). Haken Sie
**Vorherigen Commit ändern** an, um Ihre Änderungen dem letzten Commit
hinzuzufügen.

Haben Sie die [KI](ai.md) eingeschaltet, schreibt die Schaltfläche `✦` neben
der Zusammenfassung eine Nachricht aus den gestagten Änderungen. GitPoulpi
committet nie, bevor Sie auf **Änderungen an N Dateien committen** klicken.

## 6. Ihre Arbeit teilen

Die Symbolleiste hat die Schaltflächen **Pull** und **Push**, jeweils mit
einer `▾`-Liste. Über die Tastatur drücken Sie `Ctrl+G` und dann einen
Buchstaben: `Ctrl+G p` führt einen Pull aus, `Ctrl+G Shift+P` einen Push.
Nach `Ctrl+G` listet die Statuszeile die Buchstaben.

GitPoulpi nutzt die Authentifizierung, die Ihr `git` bereits verwendet
(Credential-Helper, ssh-agent). Es braucht kein eigenes Konto. Um Pull
Requests zu sehen und zu erstellen, verbinden Sie einmal ein Hosting-Konto.
Siehe [Hosting und Issue-Tracker](hosting-integration.md).

## 7. Repositories öffnen, klonen und initialisieren

Klicken Sie auf das Haus-Symbol am linken Ende der Tab-Leiste oder drücken Sie
`Ctrl+O`. Der **Home**-Tab listet Ihre geöffneten, favorisierten und zuletzt
verwendeten Repositories, mit den Schaltflächen **Durchsuchen**, **Klonen** und
**Init**. Jedes Repository öffnet sich in einem eigenen Tab. `Ctrl+W` schließt
den aktuellen Tab.

![Der Home-Tab mit den geöffneten und zuletzt verwendeten Repositories und den Schaltflächen Durchsuchen, Klonen und Init](../images/home.png)

**Klonen** fragt nach einer URL und einem Ordner oder lässt Sie ein Repository aus einem verbundenen Konto (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) auswählen (siehe [Von einem verbundenen Konto klonen](hosting-integration.md#von-einem-verbundenen-konto-klonen)). **Init** macht aus einem
Ordner ein neues Repository.
In allen drei Dialogen (und in der Ordnerauswahl, die sie öffnen) zeigt `Ctrl+N` ein Badge an jedem Feld und jeder Schaltfläche: Drücken Sie das Zeichen, um den Cursor in das Feld zu setzen oder die Schaltfläche zu drücken.

| ![Der Klonen-Dialog: wohin geklont wird, die URL, der Ordnername sowie die Schaltflächen Abbrechen und Klonen](../images/clone.png) | ![Der Dialog Repository initialisieren: der Ordner, der Name und der Start-Branch](../images/init.png) |
|---|---|

### Sparse Checkout

Ein großes Repository muss nicht vollständig auf den Datenträger gelangen. Haken Sie im Klonen-Dialog **Sparse Checkout (nur einige Verzeichnisse)** an und tippen Sie die gewünschten Verzeichnisse, getrennt durch Leerzeichen oder Kommas (`src docs/guide`). Nur diese Verzeichnisse und die Dateien im Wurzelverzeichnis des Repositorys werden ausgecheckt; der gesamte Verlauf wird trotzdem heruntergeladen, Log, Blame und Branches funktionieren also wie gewohnt. Solange der Haken gesetzt ist, ist mindestens ein Verzeichnis nötig.

![Der Sparse-Checkout-Dialog mit zwei eingetippten Verzeichnissen und den Schaltflächen Abbrechen, Deaktivieren und Anwenden](../images/sparse-checkout.png)

Jedes Repository lässt sich auch später umschalten: Öffnen Sie die Befehlspalette (`Ctrl+P`) und führen Sie **Repository: Sparse Checkout...** aus, oder nutzen Sie **Sparse Checkout...** im Menü des Repository-Tabs. Der Dialog listet die aktuell ausgecheckten Verzeichnisse in einem Feld: Fügen Sie Namen hinzu oder entfernen Sie sie und drücken Sie **Anwenden** (`Enter`), oder **Deaktivieren (vollständiger Checkout)**, um jede Datei zurückzubringen. Solange ein Arbeitsverzeichnis sparse ist, zeigt die Statuszeile ein kleines Abzeichen `sparse`. Dieselben Bedienelemente sind eine Seite unter *Einstellungen > Sparse Checkout* (eine Liste von Pfaden, einer pro Zeile, mit den Schaltflächen **Starten**, **Beenden** und **Auffrischen**).

Hinweise:

- Es ist der Cone-Modus von Git (Verzeichnisse, keine Muster); `*`, `..` und absolute Pfade werden abgelehnt.
- Rückgängig zeichnet die Änderung der Auswahl nicht auf: nichts geht verloren, und **Deaktivieren** stellt jede Datei wieder her. Git kann sich weigern, ein Verzeichnis mit nicht committeten Änderungen zu entfernen (der Fehler erscheint in der Statuszeile); committen oder stashen Sie sie zuerst.
- Dasselbe aus Skripten und von Agenten: `gitpoulpi clone <url> --sparse src docs` und `gitpoulpi sparse-checkout status | set | add | disable` (siehe [CLI-Referenz](cli-reference.md)); der MCP-Server hat dieselben Tools.

## Wie es weitergeht

- [Rundgang durch die Oberfläche](interface-tour.md): der Name von allem, was
  Sie sehen.
- [Mit dem Log arbeiten](log-and-history.md) und
  [Staging und Committen](staging-and-committing.md): die tägliche Arbeit.
- [Designs](themes.md) und [Einstellungen](settings.md).
- [KI-Unterstützung](ai.md) und [Hosting und Issue-Tracker](hosting-integration.md):
  die beiden optionalen Integrationen.
- [CLI-Referenz](cli-reference.md) und [MCP und Agenten](mcp-and-agents.md):
  dieselben Vorgänge aus Skripten und von Agenten.
- [Fehlerbehebung](troubleshooting.md).
