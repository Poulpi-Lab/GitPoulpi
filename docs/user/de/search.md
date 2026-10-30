[English](../search.md) · [Čeština](../cs/search.md) · **Deutsch** · [Español](../es/search.md) · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Suche

`Ctrl+Alt+F` (oder `Ctrl+P`, dann `Suche: Workspace...`) öffnet eine Fuzzy-Suche über
**jedes Repository, in dem Sie arbeiten**: die Repositories des aktiven
[Workspace](workspaces.md), wenn es einen gibt, sonst die
offenen Tabs. Die Schaltfläche **Globale Suche** (die Lupe mit ihrer Beschriftung in der Titelleiste, direkt links
neben dem Zahnrad) öffnet sie ebenfalls. Sie findet:

- **Repositories** (nach Name, dann nach Pfad),
- **Branches** (lokale und Remote-Tracking-Branches) und **Tags**,
- **Worktrees** (nach Ordner, dann nach Pfad und Branch),
- **Stashes** (nach Nachricht),
- **Commits** (nach Betreff; der Text eines Commits wird nur durchsucht, wenn sein
  Betreff nicht passt und Sie mindestens drei Zeichen getippt haben),
- **Pull oder Merge Requests** und **Issues** (nach Titel, `#Nummer`, Autor und
  Branch).

![Das Suchfenster über zwei offenen Repositories: Branches, ein Stash und ein Commit, die auf „main“ passen, jeweils mit ihrem Repository rechts](../images/workspace-search.png)

Die Ergebnisse sind nach Art gruppiert, in jeder Gruppe der beste Treffer zuerst.
Jede Zeile hat das Symbol der Art, die passenden Zeichen hervorgehoben, ein
abgedunkeltes Detail (den Upstream eines Branches, Autor und Branch eines Requests,
die Position eines Stashs) und rechts den Namen des Repositorys, sodass zwei
Repositories mit einem Branch `main` nie verwechselt werden. Ein Spinner im Titel
zeigt an, dass ein Repository seine Commits noch indiziert (die erste Suche in einem
großen Repository kann einen Moment dauern); die anderen Arten sind sofort da, und die
Commits kommen dazu, sobald sie bereit sind.

## Hingehen

| Taste | Wirkung |
|---|---|
| `Auf` / `Ab`, `Ctrl+P` / `Ctrl+N`, `Bild auf` / `Bild ab`, `Pos1` / `Ende` | Durch die Ergebnisse bewegen (die Gruppenüberschriften werden übersprungen). |
| `Enter` | Geht zum Ergebnis: Der Tab des Repositorys öffnet sich (oder wird gewechselt), dann wird das Ergebnis angezeigt. |
| `Esc` | Schließt die Suche. |
| `Alt+1` .. `Alt+8` | Eine Art ein- oder ausschalten (die Chips über der Liste, in ihrer Reihenfolge). `Alt+0` schaltet alle ein. |

Was „hingehen“ bedeutet:

| Ergebnis | Enter |
|---|---|
| Branch oder Tag | Wählt ihn im Bereich Refs und seinen Commit im Log aus. |
| Commit | Wählt ihn im Log aus und lädt vorher mehr vom Log, wenn er älter ist als das Geladene. |
| Stash | Wählt seine Zeile im Abschnitt STASHES aus. |
| Worktree | Öffnet ihn als Tab. |
| Pull/Merge Request, Issue | Wählt seine Zeile in der Seitenleiste aus (im Tab dieses Repositorys). |
| Repository | Wechselt zu seinem Tab (und öffnet ihn, wenn er nicht offen ist). |

`Ctrl+N Rücktaste` (oder `Zurück` in der Palette) kehrt dorthin zurück, wo Sie waren,
wie bei jedem Ansichtswechsel, den ein Agent für Sie macht. Durch das Hingehen wird
nichts ausgecheckt, gemergt oder verändert.

Mit der Maus: Ein Klick auf eine Zeile wählt sie, ein Doppelklick geht hin, ein Klick
auf einen Chip schaltet diese Art ein oder aus, das Rad scrollt, und ein Klick
außerhalb schließt das Fenster.

## Einschränken mit einem Präfix

Beginnen Sie die Anfrage mit einem Präfix, um nur eine Art zu durchsuchen, egal was
die Chips sagen:

| Präfix | Art |
|---|---|
| `r:` | Repositories |
| `b:` | Branches |
| `t:` | Tags |
| `w:` | Worktrees |
| `s:` | Stashes |
| `c:` | Commits |
| `pr:` (oder `mr:`, `p:`) | Pull oder Merge Requests |
| `i:` | Issues |
| `#` | Pull/Merge Requests und Issues (`#12` findet Nummer 12) |

`fix: login` ist eine einfache Anfrage, kein Präfix: Nur ein einzelner Buchstabe (oder
`pr:` / `mr:`) mit anschließendem Doppelpunkt oder ein führendes `#` zählt.

## Wie gefunden wird

Dieselben drei Stufen wie in der Befehlspalette: Ihr Text wörtlich, dann wörtlich
irgendwo, dann seine Zeichen in der Reihenfolge, aber getrennt. Ein früherer Treffer
und ein kürzerer Text gewinnen. Groß- und Kleinschreibung sind gleich.

## Pull Requests und Issues

Sie stammen aus dem, was GitPoulpi für jedes Repository bereits geladen hat (die
Listen hinter den Seitenleistenabschnitten PULL REQUESTS und ISSUES), abgelegt in
einer kleinen Cache-Datei im Ordner `.gitpoulpi/` des Repositorys. Die Suche **ruft
nie GitHub oder GitLab auf**: Ein Repository, dessen Requests in einem Fenster nie
geladen wurden, hat keine zu finden, und was Sie sehen, ist so frisch wie die letzte
Aktualisierung der Seitenleiste. Ohne verknüpftes Konto gibt es nichts zu finden und
nichts einzustellen.

## Von der Kommandozeile

`gitpoulpi search <Anfrage> [--workspace ID] [--kind K]... [--limit N]` tut dasselbe
und gibt JSON aus (siehe die [CLI-Referenz](cli-reference.md#workspaces-und-suche));
der MCP-Server hat ein Tool `search`, sodass ein Agent finden kann, was Sie finden.
`search.commit_limit` in den [Einstellungen](config-reference.md#search-die-suche-im-workspace)
begrenzt, wie viele Commits jedes Repositorys indiziert werden (`0` schaltet die
Commit-Suche aus).

## Die Taste ändern

`Suche: Workspace...` ist ein Palettenbefehl, und die Taste ist die Aktion
`global.workspace_search`: Belegen Sie sie unter Einstellungen > Tastatur & Maus neu
oder in der Tabelle `[keybindings.global]` (`workspace_search = "Ctrl+Alt+F"`).
