[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · **Deutsch** · [Español](../es/local-issues.md) · [Français](../fr/local-issues.md) · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# Lokale Issues und Obsidian

Führen Sie Ihre Issues als Ordner mit Markdown-Notizen: kein Konto, kein Token, kein Netzwerk. Der Ordner kann im
Repository liegen (`docs/issues/`, mit dem Code committet) oder irgendwo auf dem Datenträger, zum Beispiel ein Unterordner eines
[Obsidian](https://obsidian.md)-Vaults. GitPoulpi liest und schreibt die Notizen. Obsidian, jeder Editor oder eine Shell
können sie gleichzeitig bearbeiten, und die Listen folgen.

![Lokale Markdown-Issues in der Seitenleiste, mit dem Zeilenmenü einer Notiz: die Notiz öffnen, schließen, ihre Labels oder ihren Meilenstein bearbeiten](../images/local-issues.png)

## Einrichten

1. **Einstellungen > Integrationen > Issue-Tracker > Local Markdown > Profil hinzufügen.** Geben Sie den Ordner ein oder
   drücken Sie **Durchsuchen...** und wählen Sie ihn. Ein relativer Ordner (`docs/issues`) wird gegen jedes Repository gelesen, mit dem er
   verknüpft ist; ein absoluter (`/home/me/Vault/Projects/Issues`) gilt überall gleich. Es ist kein Token einzugeben.
   - CLI: `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. Öffnen Sie in einem Repository den Bereich **ISSUES** der Seitenleiste und wählen Sie den Anbieter **Local Markdown - <Name>** (der
   Ordner wird damit gewählt; die Repository-Auswahlliste listet den Ordner und seine direkten Unterordner, falls Sie einen anderen möchten).
   - CLI: `gitpoulpi hosting link issues --id <Profil>` (der Ordner des Profils ist das Standard-`--project`).

Nichts wird von selbst verknüpft: Wie jede Issue-Quelle ist sie pro Repository freiwillig.

## Die Notizen

Eine Notiz pro Issue, benannt `<id>-<slug>.md` (`12-crash-on-empty-name.md`), mit einem Frontmatter-Block:

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Freies Markdown. GitPoulpi schreibt den Text nie um.
```

- Neue Issues bekommen `max(id) + 1`.
- GitPoulpi schreibt nur die Eigenschaften, die ihm gehören (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Jede andere Eigenschaft, der Text, die Zeilenenden und die Reihenfolge der Schlüssel bleiben Byte für Byte erhalten.** Schreibvorgänge sind
  atomar (eine temporäre Datei, dann ein Umbenennen).
- Das Lesen ist nachsichtig: Eine Notiz ohne `id` nimmt sie aus ihrem Dateinamen, eine Notiz ohne `status` ist `todo`, eine Notiz
  ohne Frontmatter nimmt ihren Titel aus der ersten `# Überschrift`. Eine Datei, die kein Issue ist (`readme.md`), wird ignoriert.
- Ein Status ist freier Text. **Offen** heißt „nicht der Erledigt-Status“.

## Was Sie tun können

| Im Bereich ISSUES | Tastatur / Palette |
|---|---|
| Neues Issue (das `+` oder `n`): Titel, Beschreibung, Status, Zugewiesene, Labels, Meilenstein | `Hosting: Neues Issue...` |
| Zugewiesene / Labels / Meilenstein über das Zeilenmenü bearbeiten. Wählen Sie aus dem, was der Ordner schon verwendet, oder **tippen Sie ein neues Label oder einen Login** und nehmen Sie die Zeile `„...“ hinzufügen` | `Hosting: Labels bearbeiten...` und die übrigen |
| **Issue schließen** / **Issue wieder öffnen** (setzt den Erledigt-Status oder den ersten anderen) | `Lokale Issues: Issue schließen...` / `Issue wieder öffnen...` |
| **Notiz öffnen** (Ihr Standardprogramm für `.md`) | `Integrationen: Issue öffnen...` |
| **In Obsidian öffnen** (nur wenn der Ordner in einem Vault liegt, also ein Ordner `.obsidian/` darüber) | `Lokale Issues: In Obsidian öffnen...` |
| Filter: Offen / Geschlossen / Alle, „Mir zugewiesen“ | die Filter-Auswahlliste |

„Mir zugewiesen“ passt auf die Einstellung `me` des Profils, sonst auf den verbundenen Namen, sonst auf den Profilnamen.

Branches und Commits funktionieren wie bei GitHub: „Branch aus Issue erstellen“ ergibt `12-crash-on-empty-name`, und `#12` in einer Commit-Nachricht
oder `12-...` in einem Branch-Namen wird erkannt.

Änderungen außerhalb von GitPoulpi (Obsidian, eine Shell, ein `git pull`) erscheinen nach etwa einer Sekunde: Dateinamen und
Änderungszeiten des Ordners werden in einem Hintergrund-Thread angesehen, nie im Render-Thread.

## Konfigurationsdatei

In der `config.toml`, neben dem Profil (jeder Schlüssel ist optional):

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # the board's column order
done_status = "done"                            # closes an issue
reference_prefix = "LOC"                         # shown as LOC-12 instead of #12
me = "kai"                                       # who "assigned to me" means
```

Statuswerte, die Notizen verwenden, die aber nicht in `statuses` stehen, folgen den aufgeführten. `statuses` ist die Reihenfolge der Spalten des
[Boards](#das-kanban-board) (`gitpoulpi hosting issue options` gibt sie aus).

## Das Kanban-Board

![Das Board eines Notizordners: eine Spalte pro Status, die Karten mit ihren Zugewiesenen, Labels und ihrem Meilenstein](../images/issue-board.png)

Wenn Sie allein arbeiten oder in einem Team, das seine Issues neben dem Code führt, ist der Ordner auch ein Kanban-Board.
**Kopfzeile von ISSUES > Board** oder der Palettenbefehl **Issues: Board öffnen** öffnet es über dem Fenster; `Esc` schließt es.

- **Spalten** sind die Status: zuerst die in `statuses` aufgeführten, in dieser Reihenfolge, dann jeder andere Status, den eine Notiz verwendet.
  Eine leere Spalte wird trotzdem gezeigt. Jede Kopfzeile trägt die Zahl ihrer Karten.
- **Karten** zeigen die Issue-Nummer, den Titel (zwei Zeilen), die Zugewiesenen als Initialen, die Labels und den Meilenstein.
  Passen nicht alle Spalten hinein, scrollt das Board seitwärts (die Leiste darunter, das Mausrad mit `Shift` oder die Pfeile).
- **Eine Karte verschieben**: auf eine andere Spalte ziehen (die Spalte leuchtet unter dem Zeiger auf), mit
  `Shift+Links` / `Shift+Rechts` oder mit **Board: Karte verschieben nach...** (`m`), einer Auswahlliste der Spalten. Die Karte bewegt sich sofort;
  der `status` der Notiz wird wie im [Abschnitt über die Notizen](#die-notizen) umgeschrieben, sonst nichts in der Datei. Schlägt das Schreiben
  fehl, springt die Karte zurück, und der Fehler steht in der Statuszeile.
- **Öffnen** Sie eine Karte mit `Enter` oder einem Doppelklick (die Notiz öffnet sich in Ihrem Standardprogramm für `.md`). `e` bearbeitet ihre
  Zugewiesenen, Labels oder ihren Meilenstein, `.` oder ein Rechtsklick öffnet das Kartenmenü, `n` beginnt ein neues Issue in der ausgewählten
  Spalte (das Formular kehrt beim Schließen zum Board zurück).
- **Filtern** nach Zugewiesenen (`a`), Label (`l`), Meilenstein (`M`) oder Text (`/`); `x` setzt sie zurück. Die Filter sind
  Auswahllisten: tippen zum Eingrenzen, `Enter` wählt.
- **Live**: Eine in Obsidian oder einer Shell geänderte Notiz bewegt ihre Karte innerhalb einer Sekunde.

Jede Mausgeste hat eine Taste und einen Palettenbefehl (**Board: ...**, aufgelistet auf der Seite
[Tastenbelegung](keybindings.md)).

## Obsidian Bases

**Palette: `Lokale Issues: Obsidian Base erstellen`** (CLI: `gitpoulpi hosting issue base`) schreibt einmalig `issues.base` neben die
Notizen: eine *Tabellen*-Ansicht und eine *Kanban*-Ansicht, nach `status` gruppiert, in der Reihenfolge von `statuses`. Öffnen Sie sie in Obsidian, um
dieselben Issues dort als Board zu sehen. GitPoulpi bearbeitet nie eine vorhandene `.base`-Datei (Ihre oder eine frühere): Es lehnt ab und sagt es.
Das Format folgt der Bases-Syntax von Obsidian
(<https://obsidian.md/help/bases/syntax>); die Kanban-Ansicht braucht ein aktuelles Obsidian.

## CLI und MCP

Dieselben Befehle wie bei jeder Issue-Quelle funktionieren damit: `hosting issues`, `hosting issue create [--list <Status>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Vier weitere sind für Notizen:
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` und `hosting issue base`.
Siehe die [CLI-Referenz](cli-reference.md).

## Grenzen

- Ein neuer **Meilenstein** lässt sich in der Auswahlliste nicht eintippen (wählen Sie einen, den der Ordner schon verwendet), aber `milestone:` in einer
  Notiz kann beliebig sein und erscheint, sobald sie gespeichert ist.
- Nur die Notizen direkt im Ordner sind Issues (keine Unterordner).
- Es gibt kein Löschen: Entfernen Sie die Notiz in Ihrem Dateimanager oder in Obsidian (dessen Papierkorb ist Ihrer).
