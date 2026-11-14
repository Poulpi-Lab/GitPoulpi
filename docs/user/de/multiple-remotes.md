[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · **Deutsch** · [Español](../es/multiple-remotes.md) · [Français](../fr/multiple-remotes.md) · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Mehrere Remotes für einen Branch

Viele Repositories liegen auf mehr als einem Host: ein Haupt-Remote (`origin`), ein Spiegel auf einem anderen Host, ein Backup. GitPoulpi lässt einen Branch mit einem Klick an alle senden und optional von mehr als einem holen, ohne zu ändern, was reines git tut. Nichts davon braucht ein Konto, und alles steht in der eigenen `.git/config` des Repositorys, sodass reines git und andere Werkzeuge weiter funktionieren.

## Push-Ziele und Pull-Quellen

- Der **Upstream** ist gits eigener Begriff (`branch.<name>.remote` und `merge`). Er ist das *primäre* Remote des Branches. GitPoulpi ändert nie, wie er funktioniert.
- Ein **Push-Ziel** ist ein zusätzliches Remote, an das der Branch ebenfalls pusht. Die Push-Schaltfläche sendet den Branch an den Upstream und an jedes Push-Ziel.
- Eine **Pull-Quelle** ist ein zusätzliches Remote, von dem ein Pull ebenfalls Commits holt. Sie ist aus, bis Sie sie einschalten (das Kontrollkästchen **Auch Pull**): Zwei Remotes zusammenzuführen, die Sie nicht gewählt haben, ist keine Voreinstellung, die jemand will.

Ein Ziel kann auf dem Remote einen anderen Branch-Namen verwenden (`backup` bekommt `feat-x`).

## Der Remotes-Dialog

Klicken Sie mit rechts auf einen Branch und wählen Sie **Remotes von <Branch>...** (auch im Push-Dropdown, in der Palette als `Git: Remotes des Branches...` und per Klick auf das Abzeichen `⇉3` in der Zeile des Branches im Abschnitt Refs). Er listet jedes Remote des Repositorys:

| Spalte | Bedeutung |
|---|---|
| Remote, Branch | Das Remote und der Branch-Name dort. **Auf Remote umbenennen...** ändert diesen Namen für die gewählte Zeile. |
| Push | Anhaken, um das Remote zum Push-Ziel dieses Branches zu machen. |
| Auch Pull | Anhaken, damit ein Pull auch dieses Remote integriert. Der Upstream wird immer gepullt. |
| Stand | Wo der Branch dazu steht: synchron, voraus, zurück, neu, ohne Bezug. |

Eine Zeile mit „Remote fehlt“ nennt ein Remote, das der Branch noch erwähnt, das das Repository aber nicht mehr hat; **Entfernen** vergisst es. Derselbe Dialog setzt den Upstream (die Liste **Upstream**), und **Upstream entfernen** löscht ihn. **Übernehmen** schreibt alles als eine rückgängig machbare Änderung.

Ein Repository kann außerdem jedem neuen Branch dieselben zusätzlichen Ziele geben: **Einstellungen > Git & Kodierung** (die repositoryspezifische Seite), „Weitere Remotes für den Push neuer Branches“ (ein Kontrollkästchen je Remote). Sie werden in einen Branch kopiert, sobald er zum ersten Mal einen Upstream bekommt; bestehende Branches bleiben unberührt.

## Pushen

- **Push-Schaltfläche** bei einem Branch mit zusätzlichen Zielen: pusht an alle. Nimmt jedes Remote die neuen Commits einfach an, läuft es sofort, und die Fortschrittsansicht zeigt, wie jedes Remote zu einem Haken wird. Braucht etwas eine Entscheidung (ein Remote hat Commits, die Sie nicht haben), öffnet sich stattdessen der Plan-Dialog.
- **Push ▾** (der Pfeil neben der Schaltfläche; auch die Schaltfläche selbst, wenn **Push-Schaltfläche** in den Einstellungen auf **ask** steht):
  - `Push an alle (origin, gitlab, backup)`
  - `Push nur an <Remote>`, eines je Remote des Branches
  - `Push an ausgewählte Remotes...`: immer vorhanden, für jeden Branch, auch für einen mit nur einem Remote
  - `Remotes von <Branch>...`
- Das Rechtsklickmenü des Branches und die Palette (`Git: Push an alle Ziele`, `Git: Push an ausgewählte Remotes...`) bieten dieselben Aktionen mit denselben Namen.

**Push an ausgewählte Remotes...** öffnet eine Liste aller Remotes des Repositorys, angehakt sind der Upstream und die Ziele des Branches. Sie gilt nur für diesen Push; haken Sie **Für diesen Branch merken** an, um die Auswahl als Push-Ziele des Branches zu speichern.

### Der Plan

Vor einem Push an mehrere Remotes sieht GitPoulpi jedes Remote an (eine kurze Abfrage und ein Download von diesem Remote nur, wenn es abweicht), damit ein Problem beim dritten Remote sichtbar wird, bevor der erste Push startet. Der Dialog listet jedes Remote mit dem Stand des Branches und dem, was geschehen wird:

| Situation | Standardaktion | Zusätzlich angeboten |
|---|---|---|
| Das Remote hat nichts vom Branch oder es fehlen einige Commits | Push | Überspringen |
| Das Remote ist voraus (Sie haben nichts Neues dafür) | Fast-Forward von ihm | Überspringen |
| Das Remote weicht ab, weil Sie Gepushtes neu aufgesetzt oder geändert haben | Force-Push (Lease) | Merge, Rebase, Überspringen |
| Das Remote hat Commits von jemand anderem | Merge von ihm (Rebase, wenn der **Pull-Schaltfläche** auf Rebase steht) | Rebase, Force-Push (Lease), Überspringen |
| Der Verlauf hat nichts mit Ihrem gemeinsam | Überspringen | Force-Push (Lease) |
| Der Host hat den Branch abgelehnt (geschützt) | Überspringen | keine: Erzwingen würde ebenfalls abgelehnt |

Die Aktion jeder Zeile ist eine Liste, die Sie ändern können (Links/Rechts geht sie durch, Enter öffnet sie, `s` überspringt die Zeile). Bei einem Merge sehen Sie, ob er Konflikte erzeugen würde und in welchen Dateien. Ein **Force-Push** wird immer bestätigt, nennt das Remote und wie viele Commits er ersetzt, verwendet immer ein Lease (nie ein blankes Force) und lässt sich nicht rückgängig machen: Das Ergebnis nennt, welcher Commit ersetzt wurde, und den Befehl, der ihn zurückbringt. **Probleme überspringen** setzt jede Zeile mit Problem auf Überspringen. **Ausführen** (Ctrl+Enter) startet.

Zuerst laufen die Merges, Fast-Forwards und Rebases in der Reihenfolge der Liste, sodass Ihr Branch seinen Endzustand nur einmal erreicht; danach gehen die Pushes Remote für Remote hinaus, der Upstream zuerst. Ein atomares „alles oder nichts“ über Remotes hinweg gibt es nicht: Jedes endet mit seinem eigenen Ergebnis, und ein Fehler bei einem (Netzwerk, ein Hook) hält die anderen nicht auf. **Fehlgeschlagene wiederholen** (oder `Git: Push an fehlgeschlagene Remotes wiederholen`) versucht diese erneut. Das Schließen der Fortschrittsansicht beendet den Lauf nie; `Git: Push an mehrere Remotes anzeigen` holt sie zurück.

### Konflikte und Fortsetzen

Bleibt ein Merge oder Rebase des Laufs bei Konflikten stehen, schließt sich der Dialog, und das gewöhnliche Merge-Banner und der Konflikteditor übernehmen. Ein Streifen über dem Log lautet `Push an mehrere Remotes pausiert (gitlab): lösen und abschließen, dann Fortsetzen.` mit den Schaltflächen **Fortsetzen** und **Gruppe abbrechen**.

- Lösen Sie die Konflikte und schließen Sie den Merge oder Rebase ab (in GitPoulpi oder mit reinem git).
- **Fortsetzen** ist ausgegraut, bis das erledigt ist, und es wird **nie für Sie gedrückt**: Nichts läuft von selbst weiter. Drücken Sie es (oder führen Sie `Git: Push an mehrere Remotes fortsetzen` aus), und der Lauf pusht von Ihrer neuen Spitze aus erneut an jedes Remote. Nach einem Rebase kehrt ein Remote, das noch die alten Commits hält, in den Plan-Dialog zurück (es braucht jetzt einen Force-Push), statt stillschweigend erzwungen zu werden.
- **Gruppe abbrechen** (mit Bestätigung; `Git: Push an mehrere Remotes abbrechen`) verwirft den Merge oder Rebase, den der Lauf gestartet hat, und vergisst den Lauf. Bereits erfolgte Pushes bleiben.

Der Streifen überlebt einen Neustart von GitPoulpi, und die CLI sieht denselben gespeicherten Lauf.

## Pullen

- Der **Pull-Schaltfläche** pullt den Upstream genau wie zuvor, außer der Branch hat Pull-Quellen: Dann pullt er den Upstream und jede Quelle der Reihe nach, jeweils im Modus, den **Pull-Schaltfläche** in den Einstellungen vorgibt (Merge, nur Fast-Forward oder Rebase). Eine Quelle, die sich nicht per Fast-Forward nachziehen lässt, wird wie der Upstream gemergt (oder neu aufgesetzt); bei „nur Fast-Forward“ fragt eine Quelle, die mehr braucht, zuerst.
- **Pull ▾** ergänzt `Pull von allen Quellen (origin + gitlab)`, wenn es Quellen gibt, und `Pull nur von <Remote>` für jedes Remote des Branches: ein einmaliger Pull des Branches dieses Remotes, unabhängig von den Kontrollkästchen.
- Die Palette hat `Git: Pull von allen Quellen` und `Git: Pull von einem Remote...`.
- Ein Pull, der bei Konflikten stehen bleibt, hinterlässt denselben Streifen, beschriftet als Pull von mehreren Remotes.

## Einstellungen

Alle vier stehen unter **Einstellungen > Git & Kodierung**.

| Einstellung | Standard | Bedeutung |
|---|---|---|
| **Push-Schaltfläche** | direct | direct pusht sofort; ask öffnet zuerst das Push-Menü. |
| **Pull-Schaltfläche** | merge | Der Modus des Pull-Schaltfläches (Merge, nur Fast-Forward oder Rebase) und der Pull-Quellen. |
| **Jedes Remote vor einem Push prüfen** | an | Vor einem Push an mehrere Remotes jedes Remote ansehen (die CLI folgt dem; `--no-preflight` schaltet es für einen Aufruf aus). |
| **Weitere Remotes für den Push neuer Branches** (je Repository) | keine | Die Remotes, an die ein neuer Branch ebenfalls pusht. |

## Von der Kommandozeile

```
gitpoulpi branch upstream add gitlab --pull      # a push target that is also a pull source
gitpoulpi branch upstream list                    # what the branch is set up for
gitpoulpi push --all-targets                      # upstream plus every target
gitpoulpi push --to gitlab --to backup:feat-x     # exactly these, this once
gitpoulpi pull --all-sources                      # upstream plus every pull source
gitpoulpi remote-group plan                       # what a grouped push would do, nothing changed
gitpoulpi remote-group continue                   # after resolving and concluding a stop
```

Braucht ein Remote eine Entscheidung, antwortet der Befehl mit dem Plan (Exit 3) und ändert nichts; beantworten Sie ihn mit `--resolve gitlab=merge`. Alle Optionen stehen in der [CLI-Referenz](cli-reference.md). Der MCP-Server hat dieselben Werkzeuge.

## Mit reinem git

Die Ziele sind einfache Konfigurationsschlüssel, sodass ein Skript oder ein Alias sie ohne GitPoulpi nutzen kann:

```
git config --get-all branch.feature.gitpoulpi-push-to     # one value per line: remote or remote:branch

# push the current branch to its upstream and to every target
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Ein Wert ist `<remote>` (gleicher Branch-Name) oder `<remote>:<branch>`; das ordnet die Zeile `d=${t#*:}`. Der einfache Alias sieht die Remotes nicht vorher an, bietet keinen Merge an und kann nach einem Konflikt nicht fortsetzen: Dafür ist der Dialog da.

## Gut zu wissen

- Zwei Remotes, die auf dieselbe Adresse zeigen, werden bemerkt („gitlab und mirror haben beim Push dieselbe URL.“); das zweite wird standardmäßig übersprungen.
- Ein Host mit geschütztem Branch lehnt einen Force-Push ab; die Zeile sagt es und bietet nur Überspringen an.
