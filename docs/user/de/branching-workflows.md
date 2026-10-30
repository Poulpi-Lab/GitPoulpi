[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · **Deutsch** · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · [Italiano](../it/branching-workflows.md) · [日本語](../ja/branching-workflows.md) · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# Branching-Workflows

Ein Branching-Workflow sagt GitPoulpi, wie Ihr Team Branches verwendet: welche
ewig leben (`main`, `develop`, `production`), welche kurzlebige Topic-Branches sind
(`feature/...`, `hotfix/...`), wie sie heißen und wo jeder beginnt und endet. Es
ist Git Flow, anpassbar gemacht: feste Namen, Präfixe und Suffixe oder Muster mit
Platzhaltern, damit auch die Branches, die ein Bot aus einem Issue erstellt, wie
Ihre eigenen erkannt werden.

Diese Seite behandelt das Modell, die Datei, die Vorlagen, Erkennung und Import,
die lesenden Befehle, die Aktionen, die Branches starten, abschließen,
synchronisieren und hochstufen (CLI, MCP und App), und die Bildschirme der App:
den Einstellungs-Editor, die gruppierte Seitenleiste, das `Flow`-Menü und seine
Dialoge.

## Konzepte

- Ein **Basis-Branch** ist langlebig. Eine Basis kann einen **Eltern-Branch**
  haben (`develop` geht in `main` über); ein Trunk hat keinen. Eine Basis kann
  mehrere mögliche Namen auflisten (`main` oder `master`, `develop` oder `dev`):
  verwendet wird, was existiert.
- Ein **Topic-Typ** ist eine Art kurzlebiger Branch (Feature, Bugfix, Release,
  Hotfix, Support...). Er hat ein **Namensmuster**, einen **Eltern-Branch** (wo er
  beginnt und worin er abgeschlossen wird) und eine Art, abgeschlossen zu werden.
- Jeder Branch wird **eingeordnet**: eine Basis (nach Name), ein Topic eines
  Typs (nach Muster) oder nicht eingeordnet.
- Der **Eltern-Branch eines Branches** ist der Reihe nach: der, den GitPoulpi beim
  Erstellen des Branches festgehalten hat (`git config
  branch.<name>.gitpoulpi-parent`), sonst Git Towns
  `git-town-branch.<name>.parent`, wenn vorhanden, sonst der Eltern-Branch seines
  Typs.

## Wo er gespeichert wird

| Was | Wo | Eingecheckt |
|---|---|---|
| Der gemeinsame Workflow | `.gitpoulpi/workflow.toml` im Repository | Ja, von Ihnen: GitPoulpi schreibt sie nur auf Ihre Bitte |
| Ihre eigenen Anpassungen | `[workflow]` in `.gitpoulpi/repo-state.toml` | Nein (diese Datei wird ignoriert) |

Das Zustandsverzeichnis ignoriert sich selbst (`.gitpoulpi/.gitignore`), außer
`workflow.toml`, die zum Einchecken gedacht ist. Der wirksame Workflow ist die
gemeinsame Datei mit Ihrer Abweichung obendrauf: eine Basis mit demselben Namen
oder ein Topic mit derselben ID wird ersetzt; neue werden hinzugefügt;
`disable_bases` und `disable_topics` entfernen gemeinsame nur für Sie.

Das Speichern über eine vorhandene Datei behält die Schlüssel, die diese Version
nicht kennt (die einer neueren GitPoulpi-Version oder eines Teamkollegen), aber
keine Kommentare und keine Reihenfolge der Schlüssel.

## Die Datei

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (informativ)

[[base]]
name = "main"
names = ["master"]             # andere Schreibweisen desselben Branches
protected = true               # nie gelöscht, direkte Commits sind verdächtig

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # ein Umgebungs-Branch (GitLab Flow): wird nachgelagert hochgestuft

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # weitere Namen, die ebenfalls passen; nie zum Erstellen verwendet
# regex = "^feat-(?P<slug>.+)$"  # passt ebenfalls; benannte Gruppen werden zu Feldern
parent = "develop"             # wo er beginnt, und das Standard-Abschlussziel
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # Shell-Befehle auf eigenen Wunsch, immer vor dem Ausführen angezeigt
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # vom Host gesteuerte Flows, siehe unten
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode` und `hooks` steuern die [Aktionen](#aktionen)
unten; `issue_flow` steuert die [vom Host gesteuerten Flows](#vom-host-gesteuerte-flows).

### Namensmuster

Text mit `{Platzhaltern}`:

| Platzhalter | Passt auf | Erzeugt aus |
|---|---|---|
| `{slug}` | freien Text; darf `/` enthalten, wenn er zuletzt steht | dem Text, kleingeschrieben, Akzente aufgelöst, `-` zwischen Wörtern, höchstens 50 Zeichen |
| `{issue}` | Ziffern (`123`) | `123` oder `#123` |
| `{issue_key}` | einen Tracker-Schlüssel (`PROJ-123`) | wie angegeben |
| `{user}` | einen Namen ohne `/` | als Slug |
| `{date}` | `2026-10-16` oder `20261016` | wie angegeben |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | wie angegeben |
| `{n}` | Ziffern | wie angegeben |

Alles andere ist wörtlich, Präfixe und Suffixe sind also einfach Text:
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. Jeder Platzhalter
darf einmal vorkommen, und zwei Platzhalter brauchen Text dazwischen. Ein Muster
muss gültige Branch-Namen erzeugen (die Regeln von `git check-ref-format`): keine
Leerzeichen, kein `~ ^ : ? * [ \`, kein `..`, keine Komponente, die mit `.`
beginnt oder auf `.lock` endet.

Ein `regex` dient nur dem Erkennen (er ist an beiden Enden verankert); verwenden Sie
ihn für Namen, die kein Muster ausdrücken kann. Benannte Gruppen werden zu den
Feldern des Branches.

### Welches Topic gewinnt

1. Ein Branch, der wie eine Basis heißt (einer ihrer Namen), ist diese Basis.
2. Sonst ist jedes Muster jedes Topics, das passt, ein Kandidat: `pattern`, dann
   `patterns`, dann `regex`, Topic für Topic in der Reihenfolge der Datei. Es
   gewinnt das Muster, das die meisten Zeichen wörtlich festlegt
   (`release/{version}` schlägt `{user}/{slug}` schlägt ein bloßes `{slug}`); ein
   Regex zählt sein wörtliches Präfix. Bei Gleichstand gewinnt das frühere in der
   Datei.
3. Sonst ist der Branch nicht eingeordnet.

Ein kaputtes Muster oder ein kaputter Regex wird als Warnung gemeldet und
übersprungen; der Rest funktioniert weiter.

## Vorlagen

| Vorlage | Basen | Topics |
|---|---|---|
| `git-flow` | `main` oder `master`; `develop` oder `dev` (Eltern-Branch main) | `feature/` und `bugfix/` (von develop), `release/` (von develop, nach main und develop, getaggt), `hotfix/` (von main, nach main und develop, getaggt), `support/` (von main); die Pluralformen (`features/`, `releases/`...) passen ebenfalls |
| `github-flow` | `main` | `feature/` oder ein beliebiger Name, von main, Merge Request nach main, Squash, danach gelöscht |
| `gitlab-flow` | `main`; `pre-production` und `production` (Umgebungs-Branches, verkettet main, pre-production, production) | `feature/` oder ein beliebiger Name von main; `release/{version}` als Cherry-Pick-Ziel |
| `trunk` | `main` | kurzlebige `{user}/{slug}` von main, rebased |
| `custom` | keine | keine |

Umgebungs-Branches passen auch als `preprod`, `staging` und `prod`. Eine Vorlage
ist ein Ausgangspunkt: mit `flow config init` ausgeben, die Datei bearbeiten.

## Erkennung und Import

`gitpoulpi flow detect` ordnet die Vorlagen nach ihrer Eignung für die Branches des
Repositorys, mit einem Wert von 100 und den gefundenen konkreten Namen (`master`
für `main`, `dev` für `develop`, `features/` für `feature/`). Basen zählen 60
Punkte, der Anteil der übrigen Branches, den die spezifischen Muster der Vorlage
erkennen, 40, und ein langlebiger Branch einer anderen Vorlage (ein `develop` bei
einem GitHub-Flow-Kandidaten) zieht 25 ab.

`gitpoulpi flow import --from git-flow` liest die klassischen git-flow-Einstellungen
(`gitflow.branch.master`, `gitflow.branch.develop`,
`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`) aus der eigenen
`.git/config` des Repositorys; `--from git-town` liest `git-town.main-branch` und
`git-town.perennial-branches` (der Eltern-Branch jedes Branches,
`git-town-branch.<name>.parent`, wird als festgehaltener Eltern-Branch übernommen).
Nichts wird stillschweigend angewandt: Der Befehl gibt das TOML aus, und `--write`
erstellt `.gitpoulpi/workflow.toml` nur, wenn sie nicht existiert.

## Befehle

Alle nehmen `--json`; der MCP-Server stellt sie als Tools bereit (`flow_status`,
`flow_templates`, `flow_detect`, `flow_config_show` lesen; die Aktionen sind unten
beschrieben).

| Befehl | Was er tut |
|---|---|
| `gitpoulpi flow status` | Jeder lokale und Remote-Branch: Art (Basis, Topic, nicht eingeordnet), Topic-Typ und Felder, Eltern-Branch und woher er stammt (`recorded`, `git_town`, `type`, `base`) und bei lokalen Branches, wie viele Commits er seinem Eltern-Branch voraus (`ahead`) und hinter ihm (`behind`) ist. Außerdem die in der Definition gefundenen Warnungen, die geordnete Erkennung und Einstellungen anderer Werkzeuge, die sich importieren ließen. |
| `gitpoulpi flow templates` | Die obigen Vorlagen mit ihrer vollständigen Definition. Braucht kein Repository. |
| `gitpoulpi flow detect` | Die geordneten Vorlagen mit den gefundenen Basen und Topics. |
| `gitpoulpi flow config [--show-effective]` | Die gemeinsame Datei (ihr Pfad, ob sie existiert, der geparste Workflow, unbekannte Schlüssel) oder mit `--show-effective` der Workflow, mit dem Ihre Branches eingeordnet werden. `flow config show` ist dasselbe. |
| `gitpoulpi flow config init --template <t> [--write]` | Gibt das TOML einer Vorlage aus; `--write` erstellt `.gitpoulpi/workflow.toml` und überschreibt keine vorhandene. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | Wie oben beschrieben. |

Die Zähler für voraus/hinter werden für höchstens 300 lokale Branches pro Aufruf
berechnet.

Eine Workflow-Datei, die sich nicht lesen lässt (kaputtes TOML, ein Wert mit
falschem Typ), ist ein Fehler, der die Datei nennt; sie wird nie stillschweigend
ignoriert.

## Aktionen

`start`, `finish`, `sync`, `promote` und `set-parent` wirken auf das Repository.
Jede **plant** zuerst: Sie listet jeden Git-Schritt auf, den sie ausführen wird,
mit dem Git-Befehl, auf den er hinausläuft, die Warnungen, was sich nicht
rückgängig machen lässt und alles, was sie blockiert. Bis Sie bestätigen, ändert
sich nichts. Jeder Lauf ist **ein Eintrag im Rückgängig-Verlauf** (`gitpoulpi
undo` nimmt die Branches, den Tag, den gelöschten Branch, den festgehaltenen
Eltern-Branch und den Checkout zusammen zurück).

| Befehl | Was er tut |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | Benennt den Branch nach dem Muster des Typs (`flow start feature "Login form"` erzeugt `feature/login-form`; `flow start release version=1.2.0` erzeugt `release/1.2.0`; ein einzelnes Wort ist der `slug`; `user` und `date` sind standardmäßig Sie und heute), erstellt ihn aus dem Eltern-Branch des Typs oder aus `--from` (ein Branch, ein Remote-Branch oder ein Tag), checkt ihn aus und hält seinen Eltern-Branch fest. `--publish` pusht ihn mit `-u` (von `undo` nicht zurückgenommen). |
| `gitpoulpi flow finish [<branch>]` | Für jedes `finish`-Ziel des Typs der Reihe nach: das Ziel auschecken und den Branch mit der Strategie des Ziels mergen (`merge`: immer ein Merge-Commit; `squash`: ein Commit mit dem Titel `--title` oder aus dem Branch-Namen; `rebase`: den Branch auf das Ziel rebasen, dann Fast-Forward; `fast-forward`), den Tag erstellen, wenn das Ziel einen hat (`tag = "v{version}"`, annotiert), dann den Eltern-Branch auschecken und den lokalen Branch löschen, wenn der Typ es sagt (`--delete` / `--keep` überschreiben es; `--delete-remote` löscht ihn auch auf dem Remote, nicht rückgängig zu machen). Mit `finish_mode = "merge-request"` (oder `--mode merge-request`) pusht er nur den Branch und öffnet den Pull oder Merge Request in den Eltern-Branch, mit `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone` und `--integration` (nicht rückgängig zu machen; lokal wird nichts gemergt oder gelöscht). `--mode local-merge` überschreibt einen Typ mit `merge-request`, und ein Typ mit `ask` braucht eines von beiden. |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | Bringt die Commits des Eltern-Branches in den Branch: ein Merge oder, wenn die Strategie des Branches `rebase` ist (oder `--rebase`), ein Rebase. Mit `--stack` auch die Branches, deren Eltern-Branch dieser ist (mit `set-parent` festgehalten), rekursiv, der Reihe nach. Kehrt zum Branch zurück, auf dem Sie waren. |
| `gitpoulpi flow promote <env-branch>` | Umgebungs-Branches von GitLab Flow: führt den Branch, dem er nachgelagert ist (`main` in `pre-production`, `pre-production` in `production`), in ihn zusammen, in einem Schritt, und kehrt zurück. Fast-Forward, wenn möglich. |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | Schreibt (oder entfernt) `branch.<name>.gitpoulpi-parent`. Lehnt einen Zyklus ab. Rückgängig zu machen. |

Ein Branch, der von woanders gestartet wurde, wird dort abgeschlossen: Weicht der
festgehaltene Eltern-Branch (aus `--from` oder `set-parent`) von dem des Typs ab,
ist das erste Abschlussziel der festgehaltene Eltern-Branch.

### Bestätigen, planen, skripten

| Option | Bedeutung |
|---|---|
| `--dry-run` | Gibt den Plan als JSON aus (`{"plan": ...}`) und hält an. Exit-Code 1, wenn etwas ihn blockiert. |
| `--yes` | Führt ohne Nachfrage aus. |
| keine | In einem Terminal: Der Plan wird gezeigt und Sie werden gefragt. Sonst (ein Skript, `--json`, ein Agent) kommt der Plan als `{"needs_confirmation": {"plan": ...}}` mit Exit-Code 3 zurück und nichts ändert sich. `start` ohne `--publish` oder Hooks, `set-parent` und `--abort` brauchen keine Bestätigung. |
| `--run-hooks` / `--no-hooks` | Siehe [Hooks](#hooks). |
| `--autostash` | Stasht lokale Änderungen, die einen Checkout oder Merge blockieren, und spielt sie danach zurück, unabhängig von `general.auto_stash`. |

Ein blockierter Plan nennt den Grund: ein Basis-Branch, wo ein Topic nötig ist, ein
Name, der schon existiert, ein fehlendes Feld (`release/{version}` braucht
`version=...`), ein Tag, der existiert, ein Ziel-Branch, der hinter seinem Remote
liegt (`--allow-behind` macht trotzdem weiter), ein bereits laufender Merge oder
Rebase, ein nicht konfigurierter Workflow.

Mit der **Auto-Stash**-Einstellung (`general.auto_stash`) werden Änderungen im
Arbeitsverzeichnis, die einen Checkout oder Merge blockieren, um die Aktion herum
gestasht und zurückgespielt, genau wie bei einem normalen Checkout; mit `never`
hält die Aktion an, bevor sie etwas ändert.

### Konflikte

Ein Merge oder Rebase, der bei Konflikten anhält, hinterlässt den normalen
Konfliktzustand (das Konflikt-Banner, `gitpoulpi conflicts ...`) und der Flow wird
gemerkt. Exit-Code 2, `status: "conflicted"`, die Pfade in `conflicts`. Lösen Sie
die Dateien und markieren Sie sie als gelöst, dann:

- `gitpoulpi flow finish --continue` committet den Merge (oder setzt den Rebase
  fort, oder committet den Squash mit seiner Nachricht) und führt die restlichen
  Schritte aus;
- `gitpoulpi flow finish --abort` bricht den Merge oder Rebase ab, kehrt zum
  Branch zurück, auf dem der Flow begann, und vergisst den Rest. Bereits
  abgeschlossene Schritte (das erste Ziel eines doppelten Abschlusses, mit seinem
  Tag) bleiben: `undo` nimmt sie zurück.

Ein Flow, den ein fehlschlagender Hook angehalten hat, wird genauso fortgesetzt
(`--continue` führt diesen Schritt erneut aus). Der Lauf mit Konflikt und seine
Fortsetzung sind zwei Einträge im Rückgängig-Verlauf.

### Merge Requests

Der Daemon pusht den Branch; das Öffnen des Requests braucht Ihr Hosting-Token,
daher erledigt das die CLI mit demselben Code wie `gitpoulpi hosting pr` (eine
Integration, die zum Remote passt, sonst die vorausgefüllte Webseite des Hosts,
als URL ausgegeben). Das Ergebnis trägt `merge_request` (was angefragt wurde) und
`opened` (was der Host geantwortet hat) oder `open_error`, wenn der Branch gepusht
ist, der Request aber nicht geöffnet werden konnte.

### Hooks

`[hooks]` in der Workflow-Datei listet Shell-Befehle auf (`after_start`,
`before_finish`). Sie stammen aus einer Datei, die Ihr Team teilt, daher laufen
sie **nie ungefragt**: `--run-hooks` oder ein Ja auf die Frage, die ein Terminal
nach ihrer Auflistung stellt. Der Plan listet sie immer auf, mit `hooks_run:
false`, wenn sie nicht laufen werden. Ein Hook läuft in einer Shell mit dem
Repository als Arbeitsverzeichnis und gesetzten `GITPOULPI_FLOW_ACTION` /
`GITPOULPI_FLOW_BRANCH`; sein genauer Befehl, Exit-Code und seine gesamte Ausgabe
stehen im Ergebnis. Ein Hook, der mit einem Wert ungleich null endet, hält den Flow
dort an (ein `before_finish`-Hook läuft, bevor sich etwas geändert hat, ist also
ein einfacher Fehler; nach einem `after_start`-Hook existiert der Branch, und der
Flow lässt sich fortsetzen oder abbrechen). `--no-hooks` überspringt sie ohne
Nachfrage. Ein Hook kann alles tun, daher nimmt `undo` nur die Git-Änderungen
zurück.

### Für Agenten (MCP)

Die Tools sind `flow_start`, `flow_finish`, `flow_sync`, `flow_promote` und
`flow_set_parent` (finish, sync und promote sind als zerstörerisch markiert, daher
sperrt `allow_mutations` sie). Sie arbeiten nach dem Muster Plan, dann Ausführen:
mit `dry_run: true` aufrufen, um den Plan zu erhalten (ein normales Ergebnis),
dann erneut mit `yes: true`, um ihn auszuführen. Ohne beides wird der Aufruf
abgelehnt und trägt den Plan, damit nichts versehentlich läuft. Hooks laufen nur
mit einem ausdrücklichen `run_hooks: true`; der Plan listet sie auf (`hooks`,
`hooks_run`), damit der Agent dem Benutzer zuerst die Befehle zeigen kann. Nach
einem Konflikt `continue_: true` (die Option heißt `--continue`) oder `abort: true`.

### Rückgängig

| Aktion | Was `undo` tut |
|---|---|
| start | Löscht den Branch, vergisst seinen Eltern-Branch, checkt den vorherigen Branch wieder aus. |
| finish | Stellt jeden bewegten Branch, den Tag, den gelöschten Branch und den Checkout in einem Schritt wieder her. Ein gepushter Branch, eine Remote-Löschung und ein geöffneter Merge Request bleiben (der Plan sagt es). |
| sync | Setzt jeden aktualisierten Branch zurück, beim ausgecheckten samt Dateien. |
| promote | Setzt den Umgebungs-Branch zurück. |
| set-parent | Stellt den vorherigen festgehaltenen Eltern-Branch wieder her. |

## Vom Host gesteuerte Flows

Teams, in denen ein Issue die Arbeit anstößt: Das Issue wird aus einer Vorlage
geschrieben, und der Host (GitLab, GitHub, Gitea, Forgejo) oder ein Bot erstellt
den Branch und den Merge Request. GitPoulpi legt das Issue an, wartet auf das, was
der Host erstellt, und greift es auf: ein lokaler Branch, der den Remote-Branch
verfolgt, sein Eltern-Branch festgehalten (das Ziel des Merge Requests),
ausgecheckt. Es braucht den Bereich ISSUES mit einem Tracker verknüpft
(`gitpoulpi hosting link issues`); die Verknüpfung von PULL REQUESTS sagt, wo die
Branches und Requests liegen (das Projekt von ISSUES, wenn sie nicht verknüpft
ist).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # ein Bot reagiert oft auf ein Label
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # eine Datei des Repositorys
branch_from_issue = "{type}/{issue}-{slug}"   # wie der Host oder Bot den Branch nennt
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # wie lange `wait` abfragt
merge_request = true                 # host-creates-branch: auch ein Entwurfs-Request
assignees = []
```

`{type}` in `branch_from_issue` ist die ID des Topic-Typs; der Rest sind die
üblichen Token des [Namensmusters](#namensmuster) (`{issue}`, `{slug}`,
`{issue_key}`...). Ein Branch gehört zu Issue 12, wenn er auf dieses Muster passt
und `{issue}` gleich 12 ist (`feature/12-login` tut es, `feature/120-x` nicht).

| Modus | Nach dem Anlegen des Issues |
|---|---|
| `local` | Nichts auf dem Host. Mit `--pick-up` ein `flow start` des Typs mit der Issue-Nummer und dem Titel (rückgängig zu machen wie jeder Start). |
| `host-creates-branch` | GitPoulpi erstellt den Branch **auf dem Host** aus dem Eltern-Branch des Typs (GitLab: die Branches-API; GitHub: `createLinkedBranch`, das Issue zeigt ihn also unter Development; Gitea/Forgejo: die Branches-API) und einen **Entwurfs**-Request, der `Closes #12` sagt. Ein Host, der einen leeren Request ablehnt (GitHub: noch keine Commits), lässt den Branch stehen und sagt es als Warnung. |
| `bot` | Nichts: Ein Bot (ein Webhook-Handler, ein CI-Job, eine GitHub App) erstellt Branch und Request. `--wait` fragt ab, bis `wait_for` erfüllt ist. |

Worauf `wait` bei jedem Anbieter schaut: die eigenen Verknüpfungen des Hosts
(GitLab `issues/:iid/related_merge_requests` und `closed_by`; GitHub verknüpfte
Branches und die Pull Requests `connected` / `cross-referenced` der Timeline), die
Branches, deren Name auf `branch_from_issue` passt, und die offenen Requests, deren
Quell-Branch passt. Es fragt mit wachsender Pause ab (1, 2, 3, 5, 8, dann alle 10
s), bis `wait_for` erfüllt ist oder `timeout_secs` endet, und ein Host, der nach
einer Antwort langsam oder kurz nicht erreichbar ist, wird erneut abgefragt. Eine
Quelle, die der Host nicht anbietet (ein Gitea hat keine Issue-Verknüpfungen), wird
übersprungen; scheitern alle Quellen, ist der erste Fehler die Antwort.

| Befehl | Was er tut |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | Legt das Issue aus der Vorlage an (`issue_title`, `issue_labels`, die Beschreibungsdatei, `--body` statt ihrer) und macht weiter, wie der Modus es sagt. `--dry-run` gibt aus, was angelegt würde (`{"plan": ...}`), und legt nichts an. `--pick-up` wartet im Modus `bot` zuerst. Exit-Code 4: Das Warten ist abgelaufen (das Issue existiert; `flow issue wait` wartet weiter). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Fragt ab, was der Host für ein Issue erstellt hat, mit Fortschritt auf stderr in einem Terminal; Ctrl+C bricht ab. `--once` schaut ein einziges Mal. Exit-Code 4: noch nichts. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | Holt per Fetch, erstellt den lokalen Branch, der `origin/<branch>` verfolgt, hält seinen Eltern-Branch fest und checkt ihn aus. `#12` schlägt den Branch oder Request des Issues auf dem Host nach, `!5` einen Request, alles andere ist ein Branch-Name. Der Eltern-Branch ist das Ziel des Requests (`--parent` überschreibt es), sonst der Eltern-Branch des Typs. Braucht kein `--yes`: Es ist lokal und rückgängig zu machen, wie `start`. Ein `--dry-run` holt trotzdem per Fetch (es bewegt nur Remote-Tracking-Refs, und der Plan muss den Branch sehen). |

`pick-up` ist ein Eintrag im Rückgängig-Verlauf: `gitpoulpi undo` löscht den Branch,
vergisst den Eltern-Branch und checkt den vorherigen Branch wieder aus (der Fetch
bleibt, wie jeder Fetch). Es wird wie die anderen Aktionen geplant: `--dry-run`
gibt seinen Plan aus (`create_branch`, `set_upstream`, `checkout`, `record_parent`),
und die üblichen Blocker gelten (Änderungen im Arbeitsverzeichnis folgen der
Auto-Stash-Einstellung). Die Anfrage ist `{"action": "pick_up", "branch": ...,
"parent": ...}`.

Von einem Flow erstellte Issues werden **beobachtet** (`watched_issues` in
`.gitpoulpi/repo-state.toml`, nie eingecheckt): Nach jedem Fetch gleicht die App
die Remote-Branches mit `branch_from_issue` ab und sagt Ihnen, wenn für ein
beobachtetes Issue einer erschienen ist. Ein Aufgreifen erledigt es.

### Für Agenten (vom Host gesteuert)

Die Tools sind `flow_issue_new`, `flow_issue_wait` und `flow_pick_up` (schreibend,
Netzwerk; `allow_mutations` sperrt sie). `flow_issue_new` mit `dry_run: true` gibt
den Plan zurück; `flow_issue_wait` blockiert bis zu `timeout` Sekunden, geben Sie
einem Agenten also ein kurzes und rufen Sie es erneut auf (`once: true` antwortet
sofort). Beide liefern JSON mit `found` (`branches`, `requests`) und `timed_out`.

### Vom Host gesteuerte Flows in der App

Mit einem Workflow und einem verknüpften Tracker bekommt das Flow-Menü *Neues
\<Typ\>-Issue...* (eines pro Typ; Palette `Flow: Neues <Typ>-Issue...` und `Flow:
Neues Issue...`) und *Vom Host erstellten Branch aufgreifen...* (Palette `Flow:
Host-Branch aufgreifen...`).

- **Neues Issue** fragt nach dem Typ (ein Dropdown) und dem Titel; es zeigt, woher
  die Beschreibung kommt und wer den Branch erstellt. *Branch aufgreifen, sobald er
  da ist* ist standardmäßig an. *Issue erstellen* legt es an und zeigt dann im
  Modus `bot` die **Wartekarte**: ein Spinner, worauf gewartet wird, die
  verstrichene Zeit und die Zeit, nach der aufgegeben wird, und **Nicht mehr
  warten** (`Esc`). Das Beenden verliert das Issue nicht: Es bleibt beobachtet.
  Ist der Branch da, wird er aufgegriffen und die Statuszeile sagt es.
- **Ein Branch ist erschienen** ist eine Karte, die sich öffnet, nachdem ein Fetch
  den Branch eines beobachteten Issues gebracht hat (wenn kein anderer Dialog
  offen ist): *Aufgreifen* übernimmt ihn, *Später* schließt sie (sie wird bis zu
  einem Neustart der App nicht erneut angeboten).
- **Aufgreifen** nimmt `#12`, `!5` oder einen Branch-Namen und zeigt, was es
  gefunden hat, bevor es den Branch aufgreift.

Alles läuft im Hintergrund; ein Aufruf, der länger als 150 ms dauert, zeigt einen
Spinner.

### Ausprobieren auf einem Prüfstand

`scripts/fake-gitlab-bot.sh` ist ein Stand-in-Bot für das Docker-GitLab
(`scripts/test-bench.sh`): Er beobachtet ein Projekt auf Issues mit einem Label und
erstellt `<type>/<iid>-<slug>` (mit einem Commit) und einen Entwurfs-Request, der
das Issue schließt. `cargo run -p xtask -- mock-host --flow-bare <bare repo>`
liefert dasselbe offline (eine GitLab-förmige API über einem echten Bare-Repository,
mit dem Bot).

## In der App

![Die Seitenleiste mit einem Git-Flow-Workflow: main und develop als Basis-Branches markiert, Feature-Branches in einem Ordner Feature, der Rest unter Sonstige](../images/workflow-sidebar.png)

![Feature starten: der Typ, der Slug, der Startpunkt und der Branch-Name, den es erstellen wird](../images/workflow-start.png)

![Einstellungen > Repository > Workflow: die Vorlage, die Erkennung und der Editor für Basis-Branches](../images/workflow-settings.png)

Alles Folgende erscheint nur bei einem Repository, das einen Workflow hat (eine
`.gitpoulpi/workflow.toml` oder eine Abweichung von Ihnen). Ohne einen ändert sich
nichts: Die Seitenleiste ist der gewöhnliche Baum und es gibt keine
`Flow`-Schaltfläche.

### Einstellungen > Workflow

*Einstellungen > Repository-spezifische Einstellungen > Workflow* bearbeitet die
Definition. Sie arbeitet an einem Entwurf und speichert nichts, bis Sie eine
Speichern-Schaltfläche drücken.

- **Vorlage** ist ein Dropdown der Vorlagen; wer eine wählt, ersetzt die Basen und
  Topic-Typen darunter durch die Vorlage (Ihre Hooks und Ihr Issue-Flow bleiben).
- **Aus den Branches erkennen** liest die Branch-Namen dieses Repositorys und zeigt
  die passendste Vorlage mit ihrem Wert, die gefundenen Basis-Namen (`master` für
  `main`), wie viele Branches jeder Topic-Typ erkennt, mit Beispielen, und den Wert
  jeder Vorlage. **Erkannten Workflow verwenden** lädt ihn in den Entwurf.
- Enthält das Repository die Konfiguration eines anderen Werkzeugs (`gitflow.*` von
  Git Flow, `git-town.*` von Git Town), wird **Importieren** angeboten und lädt die
  importierte Definition in den Entwurf. Nichts wird angewandt, bis Sie es drücken.
- **Basis-Branches**, **Topic-Typen** und ihre **Abschlussregeln** bilden zusammen
  eine [Liste](settings.md#listen) in Gruppen im Tab **Regeln**; der Tab **Allgemein**
  enthält den Rest (Vorlage, Erkennung, Hooks, Issue-Flow). Wählen Sie einen Eintrag, um ihn rechts zu
  bearbeiten (Name und andere Namen, Eltern-Branch, geschützt, Merge-Strategie, Tag
  beim Merge, Umgebungs-Branch; Bezeichnung, Namensmuster, weitere Muster, regulärer
  Ausdruck, Eltern-Branch, Abschlussmodus, nach Abschluss löschen). **+ Hinzufügen ▾**
  wählt, was hinzukommt (eine Abschlussregel gehört zum ausgewählten Topic-Typ);
  Hinzufügen, Duplizieren, Entfernen und Ziehen funktionieren wie in jeder
  Einstellungsliste, und Einträge wandern innerhalb ihrer Gruppe. Die Zeile
  **Namensmuster** listet die Token (`{slug} {issue}
  {issue_key} {user} {date} {version} {n}`); darunter zeigt eine Live-Vorschau die
  vorhandenen Branches, auf die das Muster passt, und den Namen, den es erstellen
  würde, und ein Muster, das sich nicht parsen lässt, sagt in Rot direkt unter dem
  Feld, warum (nichts Eingetipptes geht verloren).
- **Hooks** sind Shell-Befehle, einer pro Zeile (`Nach dem Start`, `Vor dem
  Abschluss`), bearbeitet in einem mehrzeiligen Editor (`Ctrl+S` speichert, `Esc`
  bricht ab). Sie führen Befehle auf Ihrem Rechner aus: Die Seite sagt es, und die
  Dialoge zeigen sie, bevor sie laufen.
- **Issue-Flow** trägt die Felder, die die [vom Host gesteuerten
  Flows](#vom-host-gesteuerte-flows) verwenden (Modus, Issue-Titel und -Labels,
  Branch-Muster, worauf gewartet wird), gespeichert in der Workflow-Datei.
- **Probleme** listet, was die Definition falsch macht (ein Eltern-Branch, der nicht
  existiert, ein kaputtes Muster).
- Eine unten angeheftete Leiste zeigt **● Ungespeichert** (oder **Keine Änderungen**).
  **In die gemeinsame Datei speichern** schreibt den Entwurf nach
  `.gitpoulpi/workflow.toml` (einchecken), behält Schlüssel, die diese Version nicht
  kennt, und arbeitet Ihre eigene Abweichung ein. **Nur für mich** behält nur den
  Unterschied zur gemeinsamen Datei in Ihrem Repository-Zustand
  (`.gitpoulpi/repo-state.toml`, nicht eingecheckt). **Änderungen verwerfen** liest
  das Gespeicherte erneut. Wer die Einstellungen mit ungespeichertem Entwurf schließt,
  wird vorher gefragt.

### Die Seitenleiste

LOKAL und REMOTE werden nach dem Workflow gruppiert: zuerst die Basis-Branches, je
mit einem `[Basis]`-Badge (`[Basis, geschützt]`, wenn geschützt), dann ein Ordner
pro Topic-Typ, benannt nach seiner Bezeichnung mit der Zahl der Branches
(`Feature (3)`), dann die Branches, die kein Typ beansprucht, unter `Sonstige`.
Ordner klappen wie jeder Ordner ein; die einfachen Bäume kehren zurück mit *Flow >
Branches nach Typ gruppieren* (die Zeile des Flow-Menüs) oder dem Palettenbefehl
`Flow: Gruppierte Branches umschalten`. Der Mauszeiger über einem Branch zeigt
seinen Typ, seinen Eltern-Branch und wie weit er hinter (und vor) dem Eltern-Branch
liegt.

### Das Flow-Menü und seine Dialoge

Die Schaltfläche **Flow** der Symbolleiste (`Ctrl+G Shift+W`, Palette `Flow: Menü öffnen`)
listet *\<Typ\> starten...* für jeden Topic-Typ, *Aktuellen Branch abschließen...*,
*Mit Eltern-Branch synchronisieren...*, *Hochstufen...* (Umgebungs-Branches von
GitLab Flow), *Eltern-Branch festlegen...*, den Gruppierungs-Umschalter und
*Workflow-Einstellungen...*. Die Palette hat dieselben Befehle (`Flow: Feature
starten...` wird für jeden Typ erzeugt).

- **Start** ist ein Formular, das aus dem Namensmuster des Typs gebaut ist: ein Feld
  pro Token. `{issue}` und `{issue_key}` kommen mit einem Dropdown der Issues des
  verknüpften Trackers, das auch `{slug}` aus dem Titel des Issues füllt;
  `{version}` schlägt die Version nach Ihrem höchsten Release-Tag vor; `{user}` ist
  Ihr aktives Profil; `{date}` ist heute. Eine Zeile zeigt den Branch-Namen, den das
  Muster erzeugt (oder warum nicht: eine ungültige Ref, ein schon vergebener
  Name). Start bei ist ein Dropdown (zuerst der Eltern-Branch des Typs), und
  *Veröffentlichen* pusht den neuen Branch.
- **Abschließen**, **Synchronisieren**, **Hochstufen** und **Eltern-Branch
  festlegen** fragen nach dem, was sie brauchen, und zeigen dann die **Vorschau**,
  die der Daemon ermittelt (derselbe Plan, den `gitpoulpi flow ... --dry-run`
  ausgibt): jeden Schritt der Reihe nach, Warnungen (ein Ziel hinter seinem Remote,
  auf diesem aufbauende Branches) und **Blocker** (ein Ziel, das nicht existiert, ein
  Ziel hinter seinem Remote), die die Schaltfläche zum Ausführen ausschalten und
  sagen, warum. Schritte, die sich nicht zurücknehmen lassen (ein Push, das Löschen
  eines Remote-Branches, ein Merge Request), sind mit `!` markiert und eingefärbt,
  mit einer Zeile „Nicht rückgängig zu machen: ...“. Hooks, die laufen würden, stehen
  unter einer Warnung, mit einem Kästchen *Hooks ausführen* (standardmäßig aus). Ist
  der Abschlussmodus des Typs `ask`, fragt Abschließen zuerst *Abschließen per*:
  lokal mergen oder einen Pull oder Merge Request öffnen.
- Die Aktion läuft dann mit einer Markierung pro Schritt (ein Spinner nach 150 ms),
  und der Dialog zeigt, wie jeder Schritt verlief. Die ganze Aktion ist **ein
  Rückgängig-Eintrag**: `Ctrl+Z` nimmt Start, Abschluss, Synchronisierung oder
  Hochstufung auf einmal zurück (der Rückgängig-Verlauf liest sich `Flow finish
  feature/login`); Pushes, Remote-Löschungen und Merge Requests bleiben, wie die
  Vorschau sagte.
- Ein **Konflikt** hält die Aktion mit **Fortsetzen** und **Abbrechen** an. Der Dialog
  lässt sich schließen, um den Konflikt mit den üblichen Konflikt-Werkzeugen zu
  lösen, und dann mit dem Palettenbefehl `Flow: Angehaltene Aktion fortsetzen oder
  abbrechen...` wieder öffnen (auch eine Zeile des Flow-Menüs, solange eine Aktion
  angehalten ist); Fortsetzen lehnt ab, solange noch Dateien im Konflikt sind, und
  behält beide Schaltflächen. Abbrechen kehrt zum Branch zurück, auf dem die Aktion
  begann; bereits erledigte Schritte bleiben (Rückgängig nimmt sie zurück).
- Ein Abschluss per **Merge Request** pusht den Branch und öffnet dann das übliche
  Formular *Pull Request erstellen* (*Merge Request erstellen* auf GitLab)
  vorausgefüllt: Das Ziel ist der Eltern-Branch des Branches, der Titel kommt vom
  Branch (und Issue). Die Metadaten-Schaltflächen (Zugewiesene, Reviewer, Labels,
  Meilenstein) funktionieren wie bei jedem Pull Request. Antwortet keine Integration
  für das Remote, wird stattdessen die Vergleichs-URL im Browser geöffnet (und in der
  Statuszeile gezeigt).
- Schritt- und Blocker-Texte sind englisch, wie die der CLI; der eigene Text des
  Dialogs ist übersetzt. Ein Arbeitsverzeichnis mit Änderungen folgt der
  `auto-stash`-Einstellung.

Alles davon ist per Tastatur und Palette erreichbar, und jedes Dropdown ist eine
Combobox (Enter öffnet sie, Tippen filtert, Enter wählt).

Die vom Host gesteuerten Zeilen (*Neues \<Typ\>-Issue...*, *Vom Host erstellten
Branch aufgreifen...*) sind [oben](#vom-host-gesteuerte-flows-in-der-app)
beschrieben.
