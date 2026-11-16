[English](../ai.md) · [Čeština](../cs/ai.md) · **Deutsch** · [Español](../es/ai.md) · [Français](../fr/ai.md) · [Italiano](../it/ai.md) · [日本語](../ja/ai.md) · [한국어](../ko/ai.md) · [Nederlands](../nl/ai.md) · [Polski](../pl/ai.md) · [Português (Brasil)](../pt-BR/ai.md) · [Português (Portugal)](../pt-PT/ai.md) · [Русский](../ru/ai.md) · [Svenska](../sv/ai.md) · [Türkçe](../tr/ai.md) · [Українська](../uk/ai.md) · [简体中文](../zh-CN/ai.md)

# KI-Unterstützung

GitPoulpi braucht nie ein Konto, und **die KI ist standardmäßig aus**. Ist die KI an,
kann sie Commit- und Stash-Nachrichten schreiben, Änderungen erklären,
Pull-Request-Beschreibungen verfassen, Lösungen für Merge-Konflikte vorschlagen und
Ihre Änderungen auf mehrere Commits verteilen. Das Modell gehört Ihnen: ein API-Schlüssel,
den Sie besitzen, ein lokaler Server oder eine bereits installierte Agenten-CLI (etwa
`claude -p`).

Jedes KI-Bedienelement zeigt das lila `✦`. Solange die KI aus ist:

- Wird nirgendwohin etwas gesendet.
- Wird kein KI-Bedienelement angezeigt: keine Schaltfläche `✦`, kein Palettenbefehl
  `KI:`, keine KI-Menüzeile und kein KI-Tastenhinweis. Die KI-Tasten tun nichts.
- Zeigt die KI-Seite des Repositorys nur „KI ist ausgeschaltet (Einstellungen > KI)“.
- Listet der MCP-Server die `ai_*`-Tools nicht auf. Die CLI-Befehle `gitpoulpi ai ...`
  lehnen mit derselben Meldung ab.

**KI aktivieren** wirkt sofort. Ein Neustart ist nicht nötig.

Es gibt zwei Wege, die KI mit GitPoulpi zu nutzen:

| | Eingebaute KI (diese Seite) | Ein externer Agent |
|---|---|---|
| Was | GitPoulpi fragt ein Modell nach einer Commit-Nachricht, einer Erklärung, ... | Claude Code, Cursor, Codex... steuern GitPoulpi als Tool |
| Eingerichtet in | Einstellungen > KI (`[ai]`) | der MCP-Konfiguration des Agenten, Grenzen in `[mcp]` |
| Anleitung | unten | [MCP und Agenten](mcp-and-agents.md) |

## Die KI konfigurieren

1. `Ctrl+,` > **KI** > **KI aktivieren** anhaken.
2. Die Profile stehen als Liste im Tab **Profile (Verbindungen)**, links auf der Seite (bei einem schmalen Terminal
   darüber): Wählen Sie eines aus, um es rechts zu bearbeiten (`Enter` wechselt in seine
   Zeilen; der Palettenbefehl **KI: Profil bearbeiten...** springt zur Liste). Ein Profil
   namens *Default* existiert immer. **+ Hinzufügen** legt ein weiteres an, **Duplizieren**
   kopiert das ausgewählte (ohne seinen Schlüssel), **Entfernen** fragt vorher nach.
   Wählen Sie den **Anbieter** und füllen Sie seine Felder:

   | Anbieter | Felder | Standardwerte |
   |---|---|---|
   | **Lokaler Agent** (kein Schlüssel, kein eigenes Konto) | **Agententyp** (um welchen Agenten es sich handelt), **Befehl** (das genaue Programm: im `PATH`, ein Alias oder Wrapper-Skript oder ein vollständiger Pfad), **Argumente**, die ihn nicht interaktiv machen, und optionale **Umgebungsvariablen**. Der Prompt geht auf seinen stdin, die Antwort wird von stdout gelesen. | Claude Code: `claude`, `-p --model haiku` |
   | **Anthropic** | **API-Schlüssel**, optional **Modell** und **Basis-URL** | `claude-sonnet-5`, `https://api.anthropic.com/v1` |
   | **OpenAI** | **API-Schlüssel**, Modell, Basis-URL | `gpt-5`, `https://api.openai.com/v1` |
   | **OpenRouter** | **API-Schlüssel**, Modell, Basis-URL | `openai/gpt-5`, `https://openrouter.ai/api/v1` |
   | **OpenAI-kompatibel** (Ollama, LM Studio, vLLM, ein Firmen-Gateway) | **Basis-URL** (`.../v1`), **Modell**, optional Schlüssel | `http://localhost:11434/v1` (Ollama; LM Studio ist `http://localhost:1234/v1`), `llama3` |

   **Voreinstellungen** (Sie können immer einen anderen Wert eintippen):

   ![Die Dropdownliste Agententyp eines Profils für einen lokalen Agenten, mit Claude Code, OpenAI Codex CLI, Gemini CLI, opencode und den anderen Agenten-CLIs](../images/settings-ai-presets.png)

   - **Lokaler Agent > Agententyp** ist eine Dropdownliste der gängigen Agenten-CLIs,
     die meistgenutzten zuerst: Claude Code, OpenAI Codex CLI, Gemini CLI, opencode,
     Cursor Agent, GitHub Copilot CLI, Pi, Qwen Code, Aider, Goose, Crush, Amp oder
     *Anderer (keine Vorschläge)*. Der Typ ist **vom Befehl getrennt**: Er wählt nur,
     welche Vorschläge Sie bekommen. Die Wahl eines Typs füllt **Befehl** (solange er
     noch der Standard des vorherigen Typs ist) und **Argumente** mit dem Aufruf ohne
     Oberfläche vor (`claude -p`, `codex exec -`, `gemini -p ...`, ...). **Befehl** ist
     freier Text: Schreiben Sie `/opt/bin/work-claude`, ein Wrapper-Skript oder das Ziel
     eines Alias, und der Typ mit seinen Optionen, seiner Modellliste und seinen
     Variablen bleibt. Ist **Agententyp** leer, leitet GitPoulpi ihn aus dem Befehl ab.
   - **Argument-Optionen** fügt Optionen hinzu oder entfernt sie, die zur einmaligen
     Nutzung durch GitPoulpi passen, jede mit einer einzeiligen Begründung: keine
     Tools, keine gespeicherte Sitzung, Ausgabe als Klartext, geringer
     Denkaufwand, schreibgeschützter Modus, keine Berechtigungsabfragen, keine
     Update-Prüfung oder Analytik. Die Liste unterscheidet sich je Agent, weil sich die
     Agenten unterscheiden (nur manche können Tools über die Kommandozeile
     abschalten; bei opencode ist es eine Umgebungsvariable). **Modell** listet die
     Modelle dieses Agenten und schreibt die richtige Option (`--model haiku` für Claude
     Code, `-m gpt-6-luna` für Codex, `--model provider/model` für opencode);
     *(Standard - keine Option)* entfernt sie. Amp hat keine Modelloption. `aider` liest
     seinen Prompt aus `/dev/stdin` und funktioniert daher nur unter Linux und macOS.
   - **Umgebungsvariablen** werden nur für den Prozess dieses Agenten gesetzt, nie für
     GitPoulpi oder andere Profile. **Umgebungsvariable hinzufügen** bietet die
     dokumentierten Variablen an, die für diesen Agenten wichtig sind, jede mit dem, was
     sie tut (Telemetrie und Update-Prüfungen abschalten, Aufwandsstufe,
     Konfigurationsordner, API-Schlüssel, Basis-URL oder lokaler Host, ...); Sie können
     auch `NAME` oder `NAME=value` eintippen. Jede Variable hat dann ihre eigene Zeile
     **Env NAME**: ihr Wert steht als Schaltfläche (drücken Sie sie, um einen neuen
     einzutippen; er wird gespeichert, sobald Sie das Feld verlassen), dazu eine
     Schaltfläche **[x]**, die die Variable löscht. Ein Wert darf `${env:NAME}` sein, um sich eine der eigenen
     Variablen von GitPoulpi zu borgen (praktisch für API-Schlüssel, damit sie nicht in
     die `config.toml` geschrieben werden). Werte, die wie Geheimnisse aussehen (Namen,
     die auf `_KEY`, `_TOKEN`, `_SECRET`, ... enden, oder bekannte Schlüsselpräfixe),
     erscheinen als Punkte und werden von Grund auf neu eingetippt. Werte werden nie
     protokolliert.
   - **HTTP-Anbieter > Anbieter-Voreinstellung** füllt die Basis-URL und ein Modell für
     OpenAI, Anthropic, OpenRouter, Google Gemini, xAI, DeepSeek, Mistral, Groq,
     Together, Fireworks, Cerebras, Perplexity, Moonshot, Qwen (DashScope), Zhipu (GLM)
     und die lokalen Server Ollama, LM Studio, llama.cpp, vLLM, Jan und LocalAI.
     **Modell** ist eine Dropdownliste der Modelle dieses Anbieters, freier Text erlaubt.

3. Drücken Sie **Verbindung testen**: *Letzter Test* zeigt `verbunden: das Modell hat geantwortet`
   oder `fehlgeschlagen: <Grund>`.
4. Wählen Sie bei mehreren Profilen unten das **Standardprofil**; jede Aktion kann
   dennoch ihr eigenes nennen.

![Der Block Profile der KI-Seite für einen lokalen Agenten: Agententyp (Claude Code), Befehl, Argumente, Modell, zwei Env-Zeilen, Umgebungsvariable hinzufügen und entfernen, Verbindung testen und Standardprofil](../images/settings-ai-agent.png)

**API-Schlüssel** werden mit `Enter` auf der Zeile **API-Schlüssel** eingegeben und im
Credential-Helper Ihres Systems (Git Credential Manager, libsecret / `pass`, Keychain)
unter `ai:<Profil-ID>` gespeichert, nie in der `config.toml`. Ohne Helper sagt
GitPoulpi das, statt so zu tun, als behielte es ihn. *Einstellungen > SSH &
Zugangsdaten > Alle Benutzernamen und Passwörter vergessen* entfernt sie.

Dasselbe in der `config.toml` (alles andere steht in der [Konfigurationsreferenz](config-reference.md#ai-eingebaute-ki-standardmäßig-aus)):

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # free: an alias, a wrapper script, an absolute path
harness = "claude"        # which agent it is; empty = inferred from `command`
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # this agent's process only; ${env:NAME} borrows ours
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## Ausführungsmodus: Headless oder virtuelles Terminal

Ein Profil für einen lokalen Agenten startet seinen Agenten auf eine von zwei Arten, gewählt über den **Ausführungsmodus** (angezeigt für Claude Code, OpenAI Codex CLI und Gemini CLI):

- **Headless (nicht interaktiv)**, der Standard: der Einmalmodus des Agenten (`claude -p`), Prompt über stdin, Antwort über stdout.
- **Interaktiv (virtuelles Terminal)**: GitPoulpi startet den eigenen Bildschirm des Agenten in einem versteckten Terminal, tippt die Anfrage ein wie ein Mensch, wartet auf die Antwort und schließt den Agenten. Manche Agenten rechnen ihren Headless-Modus anders ab oder begrenzen ihn anders als den interaktiven; was für Sie gilt und was die Bedingungen des Anbieters für beide Modi sagen, liegt beim Anbieter, informieren Sie sich also dort.

Jeder Modus behält seine eigenen **Argumente**, **Argument-Optionen** und sein eigenes **Modell**: Der Wechsel des Modus verliert nichts, und der interaktive Modus bietet nur die Optionen an, die ohne die Headless-Option sinnvoll sind. Zwei weitere Zeilen gelten für beide Modi:

- **Arbeitsordner**: *Eigener Ordner* ist ein leerer Ordner, der GitPoulpi gehört; der Agent sieht also kein Projekt und bittet Sie einmal um Vertrauen. *Aktuelles Repository* gibt dem Agenten den eigenen Kontext des Projekts (zum Beispiel seine `CLAUDE.md`), und er bittet einmal pro Repository um Vertrauen. *Standard* ist der eigene Ordner im interaktiven Modus und im Headless-Modus, wie bisher, der Ort, an dem GitPoulpi läuft.
- **Zeitlimit (Sekunden)**: wie lange der Agent brauchen darf, bevor GitPoulpi aufgibt und ihn beendet. Leer bedeutet 120 Sekunden im Headless-Modus und 180 im interaktiven.

**Fragen beim ersten Start beantworten Sie selbst.** Ein Agent, der in einem Ordner noch nie lief, fragt zuerst etwas (diesem Ordner vertrauen, anmelden, ein Farbschema wählen). GitPoulpi antwortet nie für Sie: Die Anfrage endet mit einer Meldung, und **Agent öffnen...** (am Profil, oder *KI: Agent öffnen* in der Befehlspalette) öffnet den Agenten einmal in einem sichtbaren Terminal in dem Arbeitsordner, den er verwenden wird. Beantworten Sie die Frage, schließen Sie den Agenten und versuchen Sie es erneut; die Antwort merkt sich der Agent.

Die Antwort wird bei Claude Code aus dem Sitzungsprotokoll des Agenten gelesen (sein genauer Text, danach gelöscht) und bei Codex und Gemini vom Bildschirm zwischen zwei Markierungszeilen, die die Anfrage verlangt. Höchstens zwei Läufe im virtuellen Terminal gleichzeitig, und `Esc` während einer Anfrage beendet den Agenten.

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # the headless set, kept
run_mode = "terminal"               # headless (default) | terminal
terminal_args = ["--effort", "low"] # the interactive set
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 or absent: 120 headless, 180 terminal
```

## Anweisungen

Text, den Sie dem hinzufügen, was dem Modell gesagt wird. Der eingebaute Prompt jeder
Aktion (der das Format der Antwort festlegt) kommt immer zuerst und lässt sich nicht
bearbeiten; dann folgen vier Ebenen, von der allgemeinsten zur spezifischsten, jede nur,
wenn sie nicht leer ist:

1. **Globale Anweisungen** (jede Aktion). Die Chips *Schnell einfügen* des Editors fügen
   gängige Sätze ein: *Auf `<Sprache>` antworten*, *Antworten kurz halten*,
   *Imperativ verwenden*, *Technischer Ton*, *Lockerer Ton*.
2. **Workspace-Anweisungen** (jede Aktion, jedes Repository des
   [Workspace](workspaces.md#ki-anweisungen-eines-workspace)): für alle Aktionen
   und optional pro Aktion. Bearbeitet im Formular Bearbeiten des Workspace.
3. **Repository-Anweisungen** (jede Aktion, nur dieses Repository), mit *Mit global
   kombinieren*: **Zu global hinzufügen** (nach ihnen) oder **Global ersetzen** (statt
   ihrer).
4. **Anweisungen für die Aktion** (*Erzeugung von Commit-Nachrichten*, *Änderungen
   erklären*, *Erzeugung von Stash-Nachrichten*, *Pull Request*, *Commit-Composer*,
   *Konfliktlösung*, *Code-Review*, *Commit aufteilen*), jeder Block mit eigenen Chips in seinem Editor (z. B. *Format Conventional
   Commits*, *Grenze von 50 Zeichen*) und einer eigenen Dropdownliste **Profil**
   (`Standard (<Name>)` folgt dem Standardprofil).

`Enter` auf einer Zeile *Eigene Anweisungen* öffnet einen mehrzeiligen Editor (`Ctrl+S`
speichert, `Esc` bricht ab). Über dem Text enthält eine Zeile *Schnell einfügen* die Chips
dieses Abschnitts: Ein Klick auf einen fügt seinen Satz an der Einfügemarke ein. Mit der
Tastatur wechselt `Tab` in die Zeile, `Left`/`Right` wählen einen Chip, `Enter` oder
`Leertaste` fügt ihn ein und kehrt zum Text zurück (`Tab` oder `Esc` kehren ohne Einfügen
zurück). Ein Chip, dessen Satz schon in den Anweisungen steht, ist abgedunkelt, und ein
erneutes Einfügen bewirkt nichts. Der Text ist ein vollwertiger Editor: Ein Klick setzt die
Einfügemarke, Ziehen oder `Shift`+Pfeiltasten markieren, `Ctrl+A` markiert alles,
`Ctrl+C` / `Ctrl+X` kopieren und schneiden die Markierung, `Ctrl`+Pfeiltasten springen
wortweise, `Home` / `End`, `PageUp` / `PageDown` und das Mausrad bewegen sich durch langen
Text. Die Seite selbst zeigt von jeder Anweisung nur eine Vorschau.

![Die KI-Seite: globale Anweisungen, dann die Blöcke Erzeugung von Commit-Nachrichten und Änderungen erklären mit einer Profil-Dropdownliste und eigenen Anweisungen](../images/settings-ai.png)

**Pro Repository**: *Einstellungen > Repository-spezifische Einstellungen > KI* (solange
ein Repository geöffnet ist) enthält die Repository-Anweisungen und, pro Aktion, ein
Profil und Anweisungen, die die globalen überschreiben. Ein leeres Feld nutzt die
globale Einstellung (dort ausgegraut gezeigt); **Auf global zurücksetzen** entfernt
eine Überschreibung. Gespeichert in der `.gitpoulpi/repo-state.toml` des Repositorys und
auch von der CLI beachtet.

![Die KI-Seite des Repositorys: Repository-Anweisungen mit der Dropdownliste Mit global kombinieren, dann ein Profil und Anweisungen für jede Aktion](../images/settings-repo-ai.png)

### Wie sich die Ebenen verbinden

Die Reihenfolge beim Senden: eingebauter Prompt, **global**, **Workspace**, **Repository**,
dann die Anweisungen der Aktion selbst. Der Modus einer Ebene bestimmt, was mit den
darüberliegenden geschieht:

- **Hinzufügen** (der Standard): Der Text der Ebene wird nach den allgemeineren angefügt.
- **Ersetzen**: Der Text der Ebene wird *statt* jeder allgemeineren Ebene verwendet
  (global, und der Workspace, wenn das Repository ersetzt). Eine Ebene ohne Text
  ersetzt nie etwas.

Die Anweisungen der Aktion folgen derselben Idee: die globalen für diese
Aktion, dann die des Workspace für diese Aktion (danach angefügt oder sie
ersetzend), und eine eigene Überschreibung des Repositorys für die Aktion gewinnt gegen beide. Der
verwendete Workspace ist der aktive Workspace des Fensters, wenn er das
Repository enthält, sonst der erste Workspace, der es enthält; auf der Kommandozeile
ist es der aus `GITPOULPI_WORKSPACE`, sonst der erste.

Um genau zu sehen, was gesendet wird, Ebene für Ebene, führen Sie
`gitpoulpi ai prompt --action <Aktion> --repo .` aus (`base`, `global`, `workspace`,
`repository`, `task` und den endgültigen `system_prompt`). Es ruft kein Modell auf und
funktioniert bei ausgeschalteter KI. Die Anweisungen eines Workspace setzen Sie aus einem Skript mit
`gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode
append|replace] --text ...` (oder `--clear`).

### Prompt-Variablen

Jeder Anweisungstext (global, Workspace, Repository, pro Aktion) darf `${name}`-Variablen
enthalten. Sie werden unmittelbar vor dem Senden des Prompts durch ihren Wert ersetzt
(der gespeicherte Text behält das `${...}`). Ein Name, der nicht in der Tabelle steht,
wird unverändert gesendet, und der Editor warnt davor; schreiben Sie `$${` für ein
wörtliches `${`.

| Variable | Wird zu | Beispiel |
|---|---|---|
| `${uilang}` | Die Sprache der Oberfläche, mit ihrem englischen Namen | `French` |
| `${repo}` | Der Ordnername des Repositorys | `gitpoulpi` |
| `${branch}` | Der aktuelle Branch | `main` |
| `${ref}` / `${commit}` | Kurzer Sha des Commits, um den es in der Aktion geht (*Erklären*), sonst HEAD | `4c0443d` |
| `${commit_subject}` | Betreffzeile dieses Commits | `feat: add search` |
| `${date}` | Lokales Datum, ISO | `2026-10-03` |
| `${time}` | Lokale Uhrzeit, `HH:MM` | `14:05` |
| `${datetime}` | Lokales Datum und Uhrzeit | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | Das Standard-Remote (`origin`, sonst das erste) | `origin` |
| `${host}` | `github`, `gitlab`, `other` (ein anderer Host) oder `none` | `github` |
| `${files}` | Zahl der Dateien im Diff der Aktion | `3` |
| `${action}` | Die ausgeführte Aktion | `commit_message` |

Im Anweisungs-Editor: Tippen Sie `${`, und ein Popup listet die passenden Variablen
(`Up`/`Down` wählen, `Enter` oder `Tab` fügt ein, `Esc` schließt es); oder nutzen Sie die
Dropdownliste **Variablen** unten links (`Ctrl+T`, oder *KI: Prompt-Variable einfügen* in
der Befehlspalette), die an der Cursorposition einfügt. Jeder Eintrag hat eine
Beschreibung, die in der Dropdownliste und in der Zeile über den Schaltflächen angezeigt
wird, während Sie darüber fahren oder ihn hervorheben.

## Was sie kann

| Aktion | Im Fenster | CLI |
|---|---|---|
| **Commit-Nachricht** | Änderungen stagen, das `✦` rechts vom Feld Zusammenfassung drücken, `Ctrl+G i`, oder Palette `KI: Commit-Nachricht erzeugen`. Zusammenfassung und Beschreibung werden zum Bearbeiten ausgefüllt; nichts wird committet. | `ai commit-message` (gibt aus), `commit --ai` (committet) |
| **Änderungen erklären** | Das `✦` oben rechts im Bereich der Änderungen (`Ctrl+G w`: der ausgewählte Commit, sonst die Änderungen), oder der Menüeintrag eines Commits *Änderungen erklären (KI)*. Ein Popup: Pfeiltasten scrollen, `c` kopiert, `Esc` schließt. | `ai explain [rev] [--wip]` |
| **Stash-Nachricht** | `Ctrl+G m` öffnet den Stash-Dialog; seine Schaltfläche **KI-Nachricht** (`Ctrl+G` im Dialog) fragt das Modell. Die Palette `KI: Stash-Nachricht erzeugen` tut beides. | `stash push --ai` |
| **Titel und Beschreibung des Pull Requests** | *Titel und Beschreibung erzeugen* im Formular Pull Request erstellen (die Commits und der Diff des Branches werden lokal gelesen). | |
| **Lösung von Merge-Konflikten** | Im Merge-Konflikt-Editor: die Symbolleisten-Schaltfläche `✦ Vorschlag`, der Rechtsklick-Menüeintrag eines Konflikts *✦ Lösung vorschlagen (KI)*, `Alt+E` oder die Palette `KI: Lösung für Merge-Konflikt vorschlagen`. Der Vorschlag ersetzt den Bereich des *aktuellen Konflikts* im Ergebnis als eine Änderung (als bearbeitet markiert, ein Rückgängig-Schritt); nichts wird gespeichert oder gestagt, bis Sie speichern. | `ai resolve-conflict <path> [--conflict N]` (gibt aus) |
| **Code-Review** | Im [Review eines Pull oder Merge Requests](review.md#-ki-review-entwurf): die Schaltfläche `✦ Review entwerfen...`, `s` oder die Palette `Review: ✦ Review entwerfen...` (sie öffnet die Review-Optionen, `Enter` startet). Das Modell liest den Diff, die Beschreibung und die vorhandenen Kommentare der Anfrage (und standardmäßig die `CLAUDE.md` und `AGENTS.md` des Projekts: siehe [Review-Kontext](#review-kontext)) und schlägt Kommentare vor; Sie übernehmen, bearbeiten oder lehnen jeden ab. Nichts erreicht den Host, bevor Sie das Review absenden. | `hosting review suggest <n>` (gibt JSON aus, sendet nichts), `ai review-context` (die Standardwerte) |
| **Commit aufteilen** | Das Menü eines Commits *Commit aufteilen (KI)...* oder die Palette `KI: Ausgewählten Commit aufteilen...`. Der Aufteilungseditor öffnet sich mit der Gruppierung des Modells; siehe unten. | `ai split-plan <rev>` (gibt aus) |
| **Skript zum Umschreiben des Verlaufs** | In der [Werkbank zum Umschreiben des Verlaufs](history-rewrite.md#ein-skript-mit-ki-schreiben): die Schaltfläche `✦ Mit KI schreiben...` des Skript-Editors oder die Palette `Umschreiben: Skript mit KI schreiben...`. Ein Dialog zeigt genau, was gesendet wird; das Skript landet markiert im Editor und wird nie für Sie ausgeführt oder angewendet. | `ai rewrite-script "<Anweisung>" [--sample N]` (gibt aus) |
| **Regeln zum Umschreiben des Verlaufs** | In der [Werkbank zum Umschreiben des Verlaufs](history-rewrite.md#regeln-mit-ki-erstellen): die Schaltfläche `✦ Regeln mit KI erstellen...`, `Alt+E` oder die Palette `Umschreiben: Regeln mit KI erstellen...`. Ein mehrzeiliger Dialog zeigt genau, was gesendet wird; die Regeln und Einstellungen landen in der Werkbank, und nichts wird für Sie in der Vorschau gezeigt oder angewendet. | `ai rewrite-rules "<Beschreibung>" [--context]` (gibt aus) |
| **Antwort auf einem Host** und **Issue-Beschreibung** | Die Schaltfläche **Mit KI schreiben** (`Alt+E`) jedes Markdown-Textfelds, das an einen Host sendet: ein Kommentar, eine Antwort oder eine Bearbeitung in einer Unterhaltung, ein Review-Kommentar oder eine Zusammenfassung, die Beschreibung eines neuen Issues. Der Entwurf landet im Feld; nichts wird gesendet. Siehe [Schreibhilfe auf einem Host](#schreibhilfe-auf-einem-host). | `ai hosting-reply` (gibt einen Entwurf aus) |
| **Commit-Composer** | Das Menü der WIP-Zeile *Commits zusammenstellen (KI)...*, `Ctrl+G C`, die Palette `KI: Commits aus Änderungen im Arbeitsverzeichnis zusammenstellen...` | |
| **Worktree-Name** | Im [Dialog Worktree erstellen](worktrees.md#mit-ki-benennen): die Schaltfläche `✦ Name per KI` oder `Alt+A` öffnet ein Feld, in dem Sie das Feature, den Bug oder die Aufgabe beschreiben; das Modell benennt den Worktree und seinen Branch nach den eigenen Branch-Namen des Repositorys, und beides landet zum Bearbeiten in den Feldern des Dialogs. Gesendet werden nur Ihre Beschreibung und die Namen der lokalen Branches, daher gibt es keinen Dialog „Was gesendet wird“. | `ai worktree-name --describe "<task>"` (gibt aus) |

`Esc` bricht eine laufende Anfrage ab (der Prozess eines lokalen Agenten wird beendet).

### Lösung von Merge-Konflikten

Das Modell erhält Pfad und Sprache der Datei, die Blöcke Basis, aktuell (A) und
eingehend (B) des Konflikts mit den Bezeichnungen, die der Editor zeigt, bis zu 20
Zeilen bereits gemergten Text davor und danach sowie Ihre Anweisungen (global,
Repository und die für *Konfliktlösung* unter Einstellungen > KI). Es antwortet mit dem
gelösten Text dieses einen Konflikts in einem Codeblock; eine Antwort mit
Konfliktmarkierungen oder Geplauder statt Code wird einmal zurückgeschickt, dann
abgelehnt. Ein Konflikt über 40 KB (`ai.limits.conflict_bytes`, *Einstellungen > KI >
Limits*) wird hinter einer Warnung zurückgehalten, die seine Größe und das Limit nennt,
mit *Trotzdem senden* (nur diese Anfrage, weiterhin von den Datenschutzprüfungen
bereinigt), *Einstellung öffnen* und *Abbrechen*; das Limit ist GitPoulpis eigene
Schranke, erhöhen Sie es also, wenn Ihr Modell genug Platz hat. Die Bedienelemente sind
ausgeblendet, solange die KI aus ist, und ein Vorschlag speichert, stagt oder löst nie
etwas: Prüfen Sie das Ergebnis, bearbeiten Sie es und speichern Sie wie gewohnt.

![Das Commit-Formular nach einem Klick auf die Schaltfläche ✦: eine vom Modell geschriebene Zusammenfassung und Beschreibung, bereit zum Bearbeiten](../images/ai-commit-message.png)

![Das Popup Änderungen erklären: eine Erklärung und eine Liste von Dingen zum Gegenprüfen](../images/ai-explain.png)

### Der Commit-Composer

Das Modell verteilt die Änderungen auf mehrere Commits (jede geänderte Datei in genau
einem; ein ungültiger Plan wird einmal zurückgeschickt, dann abgelehnt). Solange Dateien
noch Konflikte haben, wird nichts angefragt: Ein Dialog listet sie auf und öffnet die
Konfliktlösungsansicht, denn der Composer braucht einen Baum ohne Konflikte. Sie prüfen den
Plan, bevor etwas committet wird:

![Die Composer-Prüfung: vier vorgeschlagene Commits mit ihren Dateien und die Schaltflächen Zurücksetzen, Abbrechen und 4 Commits erstellen](../images/composer.png)

| Taste | Maus | Tut |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | auf eine Zeile klicken | über Commits und Dateien bewegen |
| `Alt+Up` / `Alt+Down` (`K` / `J`), `m` | die Pfeile, Drag & Drop | einen Commit verschieben; eine Datei in einen anderen Commit verschieben |
| `Right` / `Left`, `*` | der Pfeil der Zeile | einen Commit, alle auf- / zuklappen (`ui.composer_files` legt den Anfang fest) |
| `e` / `Enter` | der Stift | die Nachricht bearbeiten |
| `d` / `Delete` | `Drop` / `wiederherst.` | einen Commit weglassen (seine Dateien bleiben nicht committet) |
| `n` | `+ eigener Commit` | eine Datei in einen eigenen Commit verschieben |
| `r` | `Zurücksetzen` | zurück zum Plan des Modells |
| `Ctrl+S` / `a` | `N Commits erstellen` | sie der Reihe nach erstellen |
| `Esc` | `Abbrechen` | verlassen, nichts geschieht |

Das Erstellen stagt die Dateien jedes Commits und committet sie nacheinander; das
Arbeitsverzeichnis wird nie angetastet. Eine nur teilweise gestagte Datei wird ganz
committet. Schlägt ein Schritt fehl, hält es an und sagt, wie viele Commits erstellt
wurden.

### Einen Commit aufteilen

Das Modell liest die Änderungen des Commits, nummeriert so, wie der
[Aufteilungseditor](log-and-history.md#einen-commit-aufteilen) sie nummeriert, und
schlägt mehrere Commits mit je einer Nachricht vor; es kann Hunks und einzelne Zeilen
gruppieren. Eine Antwort, die eine Änderung auslässt, eine doppelt zuordnet oder etwas
nennt, das nicht existiert, wird abgelehnt. In beiden Fällen öffnet sich der
Aufteilungseditor: mit dem Vorschlag, oder, wenn das Modell gescheitert ist, mit dem
Grund in einem Hinweis, damit Sie von Hand aufteilen können. Aufgeteilt wird erst, wenn
Sie **In N aufteilen** drücken (`Ctrl+S`); verschieben Sie vorher Änderungen, benennen Sie
Commits um oder fügen Sie einen hinzu, falls die Gruppierung nicht passt. Profil und
Anweisungen sind die von *Commit aufteilen* unter Einstellungen > KI.

### Ein Skript zum Umschreiben des Verlaufs

Sie beschreiben, was das Umschreiben tun soll („jeden nachts erstellten Commit auf den
nächsten Morgen verschieben“, „meine alte E-Mail-Adresse durch die neue ersetzen, nur vor
2024“), und das Modell schreibt ein [Lua-Skript](history-rewrite.md#lua-skripte) für die
Skriptregel, mit Kopf, `rewrite_commit(c, ctx)` und den Hilfsfunktionen der Sandbox. Der
Daemon kompiliert die Antwort; lässt sie sich nicht kompilieren, geht der Fehler einmal an das
Modell zurück. Was übrig bleibt, erscheint im Editor: lesen, testen, dann die Vorschau.
Einzelheiten und der Dialog stehen unter
[Verlauf umschreiben](history-rewrite.md#ein-skript-mit-ki-schreiben). Profil und Anweisungen sind
die von *Skript zum Umschreiben des Verlaufs* unter Einstellungen > KI.

### Regeln zum Umschreiben des Verlaufs

Sie beschreiben in eigenen Worten, wie der Verlauf umgeschrieben werden soll („meine alte E-Mail
a@b.org durch c@d.org ersetzen“, „secrets.txt aus dem gesamten Verlauf entfernen“), und das Modell
schlägt einen Umfang, einige Einstellungen und eine Liste von
[Umschreib-Regeln](history-rewrite.md#regeln-mit-ki-erstellen) im Vokabular der Werkbank vor.
**Es bereitet nur vor**: Die Antwort füllt die Werkbank, nichts wird in der Vorschau gezeigt oder
angewendet, und ein unbrauchbarer Eintrag wird mit dem Grund aufgelistet, während die gültigen
behalten werden. Eine Skriptregel entsteht auf diesem Weg nie. Auf Wunsch gehen der Umfang, die
Namen der Branches und Tags und die Autoren der neuesten Commits mit (ein Kästchen, standardmäßig
aus, im Bereich „Was gesendet wird“ sichtbar). Profil und Anweisungen sind die von *Regeln zum
Umschreiben des Verlaufs* unter Einstellungen > KI.

### Schreibhilfe auf einem Host

*Antwort auf einem Host* entwirft einen Kommentar, eine Antwort in einem Thread, einen Kommentar zu einer Codezeile oder die Zusammenfassung eines Reviews; *Issue-Beschreibung* entwirft die Beschreibung eines neuen Issues. **Die KI entwirft nur.** Der Text kommt in das Textfeld (oder wird von der CLI ausgegeben), damit Sie ihn lesen, bearbeiten und selbst senden. Von dieser Funktion führt kein Codepfad zu einem Schreibzugriff auf einen Host: Sie kann nichts senden, bearbeiten, lösen oder löschen, die Antwort von CLI und MCP sagt `"posted": false`, und ein lokaler Agent läuft nur lesend (unten). Der Text verlässt GitPoulpi nur über die eigene Schaltfläche **Senden** (`Ctrl+Enter`) oder **Kommentieren** des Textfelds.

**Im Markdown-Textfeld.** Solange KI an ist, hat jedes Textfeld, dessen Text an einen Host geht (ein Kommentar, eine Antwort oder eine Bearbeitung in der [Unterhaltungsansicht](hosting-integration.md#5c-unterhaltungen), ein Review-Kommentar, die Zusammenfassung von Review abschließen, die Beschreibung eines neuen Issues und die Beschreibung eines Pull Requests), am rechten Ende seiner Modusleiste eine Schaltfläche **Mit KI schreiben** (`Alt+E`, in allen dieselbe Taste). Sie öffnet ein Panel unter der Leiste:

- ein Feld, *Was in diese Antwort soll*, das Sie im Moment der Nutzung ausfüllen (`Enter` entwirft). Sein Platzhalter gibt ein Beispiel dieser Art („zustimmen und einen fehlschlagenden Test erbitten“). Eine Liste **Zuletzt** behält frühere Anweisungen (klicken Sie eine an, oder `Up` / `Down` im Feld; gespeichert in `ai-hints.toml`);
- *Meinen Entwurf verbessern*, angekreuzt, wenn das Feld bereits einen Text enthält: Die KI geht von ihm aus, statt einen neuen zu schreiben;
- *Die KI das Repository lesen lassen*, **jedes Mal nicht angekreuzt**, außer Sie haben seinen Standard eingeschaltet (siehe unten);
- **Entwurf**, **Stopp** (auch `Esc`) und **Schließen** sowie eine Zeile für Fortschritt oder Fehler.

Der übliche Dialog „Was gesendet wird“ (`[ai.sent_preview] hosting_reply`, `issue_body`, `pull_request_description`) zeigt die Anfrage, bevor sie hinausgeht. Neben dem System- und Benutzertext zeigt er eine Liste **Repository-Kontext**, wenn das Repository gelesen wird: jeden Teil mit seiner Größe, die Summe gegen das Limit (*31.2 KB von 48 KB*), was zum Einpassen gekürzt wurde, was als sensibel ausgelassen wurde und welche Teile wiederverwendet sind. Bei einem lokalen Agenten steht dort, dass der Agent das Repository selbst mit Nur-Lese-Werkzeugen liest, welche das sind, der Arbeitsordner und ob die Agentensitzung neu oder fortgesetzt ist, und jeder Dialog endet mit *Die KI schreibt nur einen Entwurf. Nichts wird gesendet, bevor Sie es selbst senden.* Der Entwurf ersetzt den Text in einem Rückgängig-Schritt, und das Textfeld wechselt zur Vorschau, damit Sie ihn vor dem Senden lesen; `Ctrl+Z` bringt Ihren Text zurück. Eine Pull-Request-Beschreibung wird aus den Commits und dem Diff des Branches geschrieben und nimmt dieselbe Anweisung; das Repository-Kästchen ist dort ausgegraut („nicht für diesen Text“).

Was das Modell erhält: Titel, Status und Beschreibung des Elements, den Thread, auf den Sie antworten, die letzten Kommentare (die neuesten zuerst, innerhalb von `ai.limits.hosting_reply_context_bytes`, standardmäßig 49152), das Markup, das der Host annimmt (eine Jira-Antwort kommt also als Markdown zurück, das GitPoulpi umwandelt), was in dieser Antwort stehen soll und, um Ihren eigenen Text zu verbessern, Ihren Entwurf. Alles, was andere Personen geschrieben haben, ist in der Anfrage als nicht vertrauenswürdige Daten markiert, sodass eine in einem Kommentar versteckte Anweisung nicht befolgt wird. Über dem Limit wird nichts stillschweigend gekürzt: Sie werden gefragt, und die ältesten Kommentare entfallen zuerst (der Thread, auf den Sie antworten, bleibt erhalten, bis zur Hälfte des Budgets); die CLI meldet die Kürzung in `truncated`, und `--send-all` hebt sie auf.

**Ihr eigener Basis-Prompt.** Die Anweisungen, von denen beide Aktionen ausgehen, sind bearbeitbar: *Einstellungen > KI > Antworten auf einem Host > Basis-Prompt* (`ai.hosting_reply.base_prompt`). Leer verwendet den eingebauten Prompt, der nur den Text verlangt, im Markup des Hosts und in der Sprache der Unterhaltung, es sei denn, Ihr Hinweis sagt etwas anderes. Ihre globalen, Workspace- und Repository-Anweisungen folgen weiterhin, und der Text anderer Personen bleibt als nicht vertrauenswürdig markiert, was Sie dort auch schreiben. `ai prompt --action hosting_reply` gibt die Ebenen aus.

#### Repository-Zugriff

**Standardmäßig aus und pro Nutzung angekreuzt.** Das Lesen des Repositorys liefert viel bessere Antworten auf „tritt dieser Absturz in unserem Code auf?“, sendet aber mehr, daher ist es eine Entscheidung, die Sie jedes Mal treffen: *Die KI das Repository lesen lassen* beginnt in jedem Feld nicht angekreuzt. `ai.hosting_reply.read_repository` (*Die KI standardmäßig das Repository lesen lassen*, aus) lässt das Kästchen nur angekreuzt beginnen; `--read-repo` ist das Ankreuzen der CLI. Die Auswahl ist ausgegraut, mit ihrem Grund beim Aktivieren, in drei Fällen: Das Repository ist **nicht geklont** (die Unterhaltung eines anderen Repositorys im Radar: es gibt nichts zu lesen), der gewählte lokale Agent hat **keinen Nur-Lese-Modus**, dem GitPoulpi traut, oder der Text wird **nicht aus dem Repository geschrieben** (eine Pull-Request-Beschreibung). Nur Ihr eigenes Ankreuzen schaltet es ein.

Was dann geschieht, hängt vom Profil ab:

| Profil | Was das Repository liest | Was gesendet wird |
|---|---|---|
| **API** (OpenAI-kompatibel, Anthropic) | GitPoulpi, über den Daemon, nur lesend | Ein Kontext, den es baut und dem Modell gibt: die Repository-Übersicht (oberster Baum und die Pfade um passende Dateien), die Dateien, die die Unterhaltung nennt (die ganze Datei oder ein Fenster von 40 Zeilen um die Zeile oder das Symbol), Suchtreffer zu Bezeichnern und Fehlermeldungen der Unterhaltung und bei einem Pull Request sein Diff. Innerhalb von `ai.limits.hosting_reply_*` (Kontext 49152, Datei 8192, Übersicht 6144, 20 Treffer je Begriff, 8 Begriffe), gekürzt in der Reihenfolge Diff, Dateien, Treffer, Übersicht und gemeldet. Dateien, die sensibel aussehen, werden ausgelassen, und Geheimnisse im Rest werden zuerst maskiert (siehe [Datenschutz](#datenschutz)). |
| **Lokaler Agent** (Claude Code, Codex), headless | Der Agent selbst, im Repository-Ordner | Es wird nichts gebaut: Der Agent wird im Repository mit **Nur-Lese-Werkzeugen** gestartet und liest, was er braucht. Claude Code erhält `Read`, `Grep` und `Glob` und bekommt `Bash`, `Edit`, `Write`, `NotebookEdit`, `WebFetch` und `WebSearch` verweigert, ohne geladenen MCP-Server (der eigene Server eines Projekts könnte senden); Codex läuft in seiner schreibgeschützten Sandbox. Die Hosting-Zugangsdaten Ihrer Umgebung (`GH_TOKEN`, `GITHUB_TOKEN`, `GITLAB_TOKEN`, die Variablen von Jira, Linear, Azure DevOps und Bitbucket, `GIT_ASKPASS`, `SSH_AUTH_SOCK`) werden aus seiner Umgebung entfernt, sodass auch ein falsch eingerichteter Agent nichts veröffentlichen oder pushen kann. |

Ein Agent, für den GitPoulpi keinen bestätigten Nur-Lese-Modus kennt (und jeder Agent im Ausführungsmodus Interaktives Terminal), kann nicht hineingelassen werden: Die Auswahl bleibt ausgegraut mit *Dieser Agent hat keinen Nur-Lese-Modus, dem GitPoulpi traut.*

**Caching und Kontext erneuern.** Was gesammelt wurde, wird wiederverwendet, damit die nächste Antwort schnell und günstig ist:

- Der gebaute Kontext eines API-Profils wird je Repository für `ai.hosting_reply.context_ttl_minutes` behalten (*Repository-Kontext behalten für*, standardmäßig 30). Er ist durch das Repository, sein `HEAD`, die Auswahl und die Limits bestimmt, und seine Teile werden einzeln behalten, sodass das nächste Issue desselben Repositorys nur die Teile neu baut, die es nicht teilt. Im Fenster leben die Caches im Speicher (die letzten 8 Kontexte); die CLI behält sie in Dateien in ihrem privaten Statusordner, sodass ein zweites `ai hosting-reply --read-repo` wiederverwendet, was das erste gebaut hat.
- Ein neuer Commit auf dem ausgecheckten Branch (`HEAD` bewegt sich), eine Änderung der Limits oder das Zeitlimit beginnen von vorn. Ebenso **Kontext erneuern**, die Schaltfläche im Panel, wenn das Repository gelesen wird, oder `--refresh-context` in der CLI. Eine Zeile unter dem Feld sagt, was der letzte Entwurf getan hat: *Kontext erstellt: 5 Teile, 31.2 KB, HEAD a1b2c3d*, *Kontext wiederverwendet: 4/5, HEAD a1b2c3d*, *Anbieter-Cache: 12k Tokens genutzt*, *Der Agent las das Repository (HEAD a1b2c3d)*, *Agentensitzung gestartet* oder *Agentensitzung fortgesetzt*.
- **Prompt-Cache von Anthropic.** Bei einem Anthropic-Profil wird die Anfrage mit dem Basis-Prompt und den Anweisungen als einem gecachten Block und dem Repository-Kontext als zweitem gesendet, danach Ihre Unterhaltung, Ihr Hinweis und Ihr Entwurf, sodass eine zweite Antwort auf demselben Kontext aus dem Cache des Anbieters gelesen wird und einen Bruchteil kostet (Nachfragen cachen auch die Unterhaltungsnachricht und den neuesten Entwurf, siehe unten). Die Cache-Zähler der Antwort zeigt *Anbieter-Cache: N Tokens genutzt*. OpenAI-kompatible Server erhalten dieselbe stabile Reihenfolge der Anfrage, von der ihr automatisches Präfix-Caching profitiert; GitPoulpi sendet nichts Zusätzliches.
- **Der Agent behält seine Sitzung.** Mit **Agentensitzung zwischen Antworten behalten** (`ai.hosting_reply.reuse_agent_session`, standardmäßig an) und angekreuztem *Die KI das Repository lesen lassen* wird ein lokaler Agent, der das unterstützt (heute Claude Code), mit einer Sitzungs-ID gestartet und dann damit fortgesetzt, sodass das, was er vom Repository gelesen hat, in seinem eigenen Verlauf bleibt und die nächste Antwort schneller startet. Andere Agenten laufen einmalig. Die Sitzung gilt je Repository, `HEAD` und Aktion; ein neuer Commit oder **Kontext erneuern** beginnt eine neue; bei ausgeschalteter Option oder nicht angekreuztem Kästchen ist jede Antwort ein einmaliger Lauf, der nichts behält. Die Sitzung lebt im eigenen Speicher des Agenten, GitPoulpi behält nur ihre ID (im Speicher im Fenster, in einer Datei nur für den Benutzer bei der CLI).

**Nachfragen: kürzer, eine andere Anweisung.** Noch einmal im selben Textfeld zu fragen („kürzer fassen“, „ein Dankeschön ergänzen“) ist ein Austausch. Eine fortgesetzte Claude-Code-Sitzung erinnert sich selbst daran. Bei allem anderen (einem API-Profil, Codex und jedem anderen Agenten) gibt GitPoulpi ihn zurück: Die Anfrage trägt die Unterhaltungsnachricht der ersten Anfrage, dann jede frühere Anweisung und den Entwurf, der sie beantwortet hat, als richtige Chat-Runden (bei einem Agenten, der ein einziges Dokument liest, als Protokoll im Prompt), und zuletzt nur das Neue: Ihre Anweisung und, wenn *Meinen Entwurf verbessern* angekreuzt ist, der Text, der jetzt im Feld steht, mit Ihren eigenen Änderungen. Eine Nachfrage beginnt also nie bei null, und nichts wird erneut aus dem Repository gelesen. „Was gesendet wird“ zeigt alles. Der Austausch wird wie die Unterhaltung auf `ai.limits.hosting_reply_context_bytes` begrenzt: Die ältesten Nachfragen fallen zuerst weg, die Unterhaltungsnachricht und Ihr neuester Entwurf bleiben immer (das `context.history_omitted` der CLI-Antwort zählt sie). Innerhalb eines Austauschs bleibt der Repository-Kontext der seiner ersten Anfrage, er wird also wiederverwendet, und der Cache des Anbieters greift: Bei einem Anthropic-Profil sind der Basis-Prompt, der Repository-Kontext, die Unterhaltungsnachricht und der neueste Entwurf der gecachte Teil, und ein OpenAI-kompatibler Server erhält dieselbe Reihenfolge der Nachrichten für sein Präfix-Caching. Codex ließe sich mit `codex exec resume <id>` fortsetzen, aber es wählt die Sitzungs-ID selbst und gibt sie nur unter seinen JSON-Ereignissen aus (und `--last` würde Ihre eigenen Sitzungen aufgreifen), deshalb lässt GitPoulpi es einmalig laufen und liefert den Verlauf stattdessen erneut mit. Der Austausch lebt mit dem Textfeld: Er endet, wenn das Textfeld geschlossen oder der Text gesendet wird, und **Kontext erneuern** verwirft ihn ebenfalls (die nächste Anfrage ist wieder eine erste). Nur eine Anfrage, die beantwortet wurde, gehört dazu; eine fehlgeschlagene oder gestoppte nicht. Eine Pull-Request-Beschreibung wird aus dem Diff geschrieben und hat keinen Austausch.

Bei der CLI ist jeder Aufruf getrennt, deshalb nimmt `--continue` den Austausch wieder auf: Es setzt den letzten `ai hosting-reply` für dasselbe Ziel in diesem Repository fort (gleiches `--item`, `--reply-to`, `--path`/`--line` oder `--issue-title`), aus einer kleinen Datei im privaten Zustandsordner. Ohne `--continue` beginnt ein neuer Austausch und ersetzt den gespeicherten. `--continue` ohne gespeicherten Austausch für das Ziel ist ein Fehler. Das `context` der Antwort trägt `history_turns` (und `history_omitted`), wenn frühere Runden mitgegangen sind.

```
gitpoulpi ai hosting-reply --conversation conversation.json --hint "say I will look tomorrow"
gitpoulpi ai hosting-reply --conversation conversation.json --hint "shorter" --continue
gitpoulpi ai hosting-reply --conversation conversation.json --reply-to <thread id> --draft "thx will do"
gitpoulpi ai hosting-reply --conversation conversation.json --read-repo --hint "is this our bug?"
gitpoulpi ai hosting-reply --issue-title "Crash on empty config" --hint "steps to reproduce"
```

`--conversation` nimmt eine Datei, `-` für die Standardeingabe oder das JSON selbst: das JSON, das `hosting issue comments --json` oder `hosting review conversation --json` ausgibt (eine ältere Seite mit `header` und `entries` wird ebenfalls angenommen), sodass `gitpoulpi hosting issue comments 7 --json | gitpoulpi ai hosting-reply --conversation -` funktioniert; `--item <ref>` holt die Unterhaltung selbst (`7` oder `#7` ein Issue, `!7` ein Pull Request). Mit `--read-repo` legen `--pr-base REV --pr-head REV` den Diff eines Pull Requests in den Kontext; das `context` der Antwort nennt, welche Teile gelesen wurden, ob sie wiederverwendet wurden, die Cache-Tokens des Anbieters und die Agentensitzung. Das MCP-Tool `ai_hosting_reply` ist derselbe Befehl und nur lesend.

## Wohin der Schlüssel gehört

Ein API-Schlüssel (OpenAI, Anthropic, OpenRouter, ein OpenAI-kompatibler Server) ist an die Adresse gebunden, für die er gespeichert wurde: den Origin der Basis-URL des Profils, festgehalten in `trust.toml` im privaten Datenordner, der nur dem Benutzer gehört. Zeigt die Basis-URL später woandershin, wird die Anfrage abgelehnt, bevor etwas gesendet wird, und der Schlüssel bleibt, wo er ist. Führen Sie `gitpoulpi ai trust <profile> --accept-origin <url>` aus (oder akzeptieren Sie es in der App), um den Schlüssel bewusst an die neue Adresse zu verschieben.

- Einfaches `http://` ist auf diesem Rechner erlaubt (Ollama, LM Studio auf `localhost`); anderswo braucht es `gitpoulpi ai trust <profile> --allow-insecure-http on`.
- Ein selbst gehosteter Server, dessen Zertifikat nicht zu einer bekannten Zertifizierungsstelle führt, wird nur für den Schlüssel-Fingerabdruck akzeptiert, den Sie mit `--accept-certificate <sha256>` festlegen; der TLS-Handshake beweist weiterhin, dass der Server diesen Schlüssel besitzt.
- Ein Profil, das schon vor dieser Prüfung einen Schlüssel hatte, wird bei der ersten Verwendung stillschweigend aus seiner aktuellen Basis-URL erfasst.
- Ein lokaler Agent (`claude -p` und Ähnliche) hat keinen Schlüssel und kein HTTP, es gibt also nichts zu binden.

## Review-Kontext

Ein Code-Review-Entwurf (`✦` oder `hosting review suggest`) liest Diff, Titel, Beschreibung und die
vorhandenen Kommentare der Anfrage. Er kann auch die eigenen Konventionen des Projekts erhalten, in einem Block
**Project conventions** vor dem Diff, und ein lokaler Agent kann im Repository arbeiten. Die Standardwerte:

| Einstellung | Standard | Bedeutung |
|---|---|---|
| `run_in_repository` | `profile` | `profile`: der *Arbeitsordner* des KI-Profils entscheidet. `yes`: ein lokaler Agent läuft für das Review im Repository, was auch immer das Profil sagt. `no`: nicht dort (ein Profil mit *Repository* nutzt stattdessen den eigenen Ordner). HTTP-Profile haben keinen Arbeitsordner, daher ändert es für sie nichts. |
| `include_claude_md` | an | Dem Modell die `CLAUDE.md` des Repositorys geben. |
| `include_agents_md` | an | Dem Modell die `AGENTS.md` des Repositorys geben. |
| `extra_file` | keine | Eine weitere Datei: ein Pfad relativ zur Repository-Wurzel, ein absoluter Pfad oder ein Link. |

Sie sind wie die Anweisungen geschichtet, wobei das Spezifischste Feld für Feld gewinnt: die globale Tabelle `[ai.review]`,
dann die eigenen Werte des Repositorys (ein nicht gesetztes Feld erbt), dann die Wahl eines einzelnen Laufs. Ein
Repository, das nicht geklont ist, behält seine eigenen Werte unter `host/project` in `remote-reviews.toml`, neben seinen
Review-Entwürfen.

Ein Repository kann die globale zusätzliche Datei auch **abschalten**: sein `no_extra_file = true` (das Kontrollkästchen
**Keine zusätzliche Datei** in *Repository-spezifische Einstellungen > KI*, im Dialog ✦ Review-Optionen und das Flag
`--no-extra-file` von `ai review-context`) bedeutet „keine zusätzliche Datei für dieses Repository“, was auch immer
`[ai.review]` nennt. Ein **leeres** Feld **Zusätzliche Datei** ist nicht dasselbe: Es erbt die globale Datei. Eine Datei
des Repositorys und der Schalter schließen sich aus (das Setzen des einen löscht den anderen); der Schalter gewinnt, wenn
eine von Hand bearbeitete Datei beide enthält. Ein älteres `extra_file = ""` eines Repositorys bedeutet weiterhin keine.

Woher die Dateien kommen:

- `CLAUDE.md`, `AGENTS.md` und eine relative zusätzliche Datei werden aus der **Wurzel** des Repositorys auf dem
  Branch gelesen, auf den die Anfrage zielt, nie aus dem Arbeitsverzeichnis (das auf einem anderen Branch sein oder lokale
  Änderungen haben kann). Mit einem Klon aus dem Basis-Commit der Anfrage; ohne einen über die API des Hosts.
- Ein absoluter Pfad wird von diesem Rechner gelesen.
- Ein Link auf eine Datei eines Code-Hosts (GitHub `.../blob/<branch>/<path>`, GitLab `.../-/blob/<branch>/<path>`,
  Gitea und Forgejo `.../src/branch/<branch>/<path>` oder ein Raw-Link) wird über das Integrationsprofil gelesen,
  dessen Host passt, sodass ein privates Repository mit dem Token funktioniert, das Sie schon angegeben haben. Ein Link, den kein
  Profil abdeckt, oder jede andere Adresse wird anonym abgerufen. Ein Branch-Name mit Schrägstrich lässt sich nicht
  aus einem Link lesen: Nutzen Sie einen Pfad oder einen Link auf einem Branch ohne einen.
- Eine fehlende `CLAUDE.md` oder `AGENTS.md` wird stillschweigend übersprungen; eine fehlende zusätzliche Datei wird gemeldet
  (`context.problems` im JSON der CLI).
- Jede Datei wird bei **8 KB** gekürzt und alle Dateien zusammen bei **16 KB**, mit einem sichtbaren
  Hinweis `[CLAUDE.md cut: showing ... of ... bytes]` in dem, was das Modell liest.

Läuft ein lokaler Agent im Repository, sagt ihm der Prompt, dass die ausgecheckten Dateien nicht zum Head der Anfrage
passen müssen und dass der Diff maßgeblich ist.

In der Review-Ansicht öffnet `s` den Dialog **✦ Review-Optionen** mit diesen Werten gefüllt: denen des Repositorys vor
den globalen. Ändern Sie sie nur für diesen Lauf, oder haken Sie *Als Standard dieses Repositorys speichern* an, um sie zu behalten (siehe
[Review-Optionen](review.md#review-optionen)). Die Standardwerte lassen sich auch in *Einstellungen > KI* bearbeiten (der Block *Code-Review*,
vier Zeilen) und pro Repository in *Repository-spezifische Einstellungen > KI*, wo jede Zeile *Global verwenden* wählen
kann und eine Schaltfläche *Auf global zurücksetzen* die eigenen Werte des Repositorys verwirft; die Einstellungssuche findet sie (`CLAUDE.md einbeziehen`,
`AGENTS.md einbeziehen`, `Zusätzliche Datei`). Alles, was dem Modell nicht übergeben werden konnte (eine fehlende zusätzliche Datei) oder
auf das Limit gekürzt wurde, steht auf der Entwurfskarte.

Von der Kommandozeile:

```sh
gitpoulpi ai review-context                      # the global defaults and what a review gets
gitpoulpi ai review-context --agents-md off      # save a global default
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # follow the global defaults again
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` braucht kein Modell und funktioniert bei ausgeschalteter KI. `ai prompt --action code_review` gibt den
wirksamen Kontext als `review_context` aus. Dieselben vier Einstellungen sind `ai.review.*` in
`config.toml` ([Referenz](config-reference.md#ai-eingebaute-ki-standardmäßig-aus)).

## Eingabelimits

Jede Eingabe, die GitPoulpi in eine Anfrage legt, hat ein Größenlimit, eine Einstellung
unter `[ai.limits]` (*Einstellungen > KI > Limits*). Die Limits sind GitPoulpis eigene
Schranke (das Kontextfenster eines Modells ist von Modell zu Modell verschieden): Erhöhen
Sie sie, wenn Ihr Modell genug Platz hat, senken Sie sie, um Anfragen klein zu halten.

| Einstellung (`ai.limits.`) | Standard | Was sie begrenzt |
|---|---|---|
| `conflict_bytes` | 40000 | Die Blöcke Basis, aktuell und eingehend eines Merge-Konflikts |
| `diff_bytes` | 60000 | Der Diff einer Nachricht zu einem Commit, einer Erklärung, eines Stash-Namens, einer Pull-Request-Beschreibung, eines Commit-Plans oder einer Aufteilung; der gesamte Diff eines Code-Reviews |
| `review_file_bytes` | 12000 | Der Diff einer Datei in einem Code-Review |
| `review_text_bytes` | 4000 | Die Beschreibung des begutachteten Pull Requests |
| `review_threads_bytes` | 6000 | Die bereits vorhandenen Review-Kommentare |
| `review_context_file_bytes` | 8192 | Eine Projekt-Kontextdatei eines Reviews (`CLAUDE.md`, `AGENTS.md`, die zusätzliche Datei) |
| `review_context_bytes` | 16384 | Alle Kontextdateien eines Reviews zusammen |
| `hosting_reply_context_bytes` | 49152 | Das Element und die Unterhaltung für eine entworfene Antwort |
| `hosting_reply_file_bytes` | 8192 | Eine Repository-Datei für eine entworfene Antwort (wenn die KI das Repository liest) |
| `hosting_reply_map_bytes` | 6144 | Die Repository-Übersicht für eine entworfene Antwort |
| `hosting_reply_hits_per_term` | 20 | Suchtreffer je Begriff (1 bis 200) |
| `hosting_reply_terms` | 8 | Nachgeschlagene Suchbegriffe (1 bis 50) |

**Nichts wird stillschweigend gekürzt.** Liegt eine Eingabe über ihrem Limit, wartet die
Anfrage, und *Über dem KI-Limit* listet auf, was darüber liegt: welcher Teil (der Diff,
eine Datei, die Beschreibung, eine Kontextdatei), seine Größe, das Limit und die
Einstellung. Ein Review listet jeden Teil über dem Limit in einem Dialog auf. Die
Schaltflächen, jeweils mit ihrer Taste:

- *Alles senden* (`A`): nur diese Anfrage, nichts wird gekürzt. Sie durchläuft weiterhin die
  Datenschutzprüfungen (Geheimnisse ersetzt, sensible Dateien ausgelassen) und, wenn er an
  ist, den Dialog *Prüfen, was gesendet wird*, der als Nächstes kommt.
- *Gekürzt senden* (`G`): Was über seinem Limit liegt, wird gekürzt, wie es immer war; jede
  Zeile des Dialogs sagt, was ausgelassen wird (das Ende des Diffs, die letzten Hunks, die
  Dateien, die nicht passen).
- *Einstellung öffnen* (`E`): bricht die Anfrage ab und öffnet *Einstellungen > KI > Limits*
  beim ersten aufgelisteten Limit. Hier startet der Fokus: Enter sendet nie.
- *Abbrechen* (`B`, `Esc`, das Kreuz oder ein Klick außerhalb): sendet nichts.

Ein Konflikt ist die Ausnahme: Er wird nie gekürzt. *Trotzdem senden* (`T`) sendet ihn
einmal vollständig.

Nichts startet von selbst eine KI-Anfrage, also gibt es keinen Hintergrundlauf, der
fragen könnte: Jede Anfrage haben Sie selbst gestartet. Ein Lauf, der nicht fragen kann
(CLI und MCP), kürzt wie bisher und sagt es; siehe unten.

**Auf der Kommandozeile** kann nichts einen Dialog beantworten, daher wird eine Eingabe
über ihrem Limit wie bisher gekürzt, und die Antwort nennt, was gekürzt wurde:
`truncated: [{part, detail?, bytes, limit, setting}]` (nur vorhanden, wenn etwas gekürzt
wurde; `part` ist eines von `diff`, `split_hunks`, `review_diff`, `review_file`,
`review_text`, `review_threads`, `context_file`, `context_total`). `--send-all` (Alias
`--no-truncate`) sendet stattdessen alles vollständig: bei `ai` (allen Unterbefehlen,
auch `ai resolve-conflict`, wo ein Konflikt über seinem Limit sonst abgelehnt wird), bei
`commit --ai`, bei `stash push --ai` und bei `hosting review suggest`. Die MCP-Tools haben
dasselbe Argument `send_all` und dasselbe Feld `truncated`.

## Datenschutz

- Nichts wird gesendet, bis Sie eine Schaltfläche drücken oder einen Befehl ausführen.
  Keine KI im Hintergrund, keine Telemetrie, kein GitPoulpi-Konto.
- Was gesendet wird: der Prompt (eingebaut, dann Ihre Anweisungen) und der Diff, nach dem
  Sie gefragt haben; bei einem Pull Request außerdem die Commit-Betreffs des Branches. Ein
  Diff über 60 KB (`ai.limits.diff_bytes`) wird nie stillschweigend gekürzt: siehe
  [Eingabelimits](#eingabelimits).
- Eine Anfrage für ein Skript zum Umschreiben des Verlaufs sendet Ihre Anweisung und den festen
  Skriptvertrag (die API der Skript-Sandbox). Autor, Datum und Betreff von bis zu 20 Commits
  gehen **nur** mit, wenn Sie im Dialog *Eine Stichprobe senden* ankreuzen (standardmäßig aus;
  die CLI braucht `--sample N`). Dateiinhalte und geheime Werte werden nie gesendet.
- Es geht nur an das für diese Aktion gewählte Profil: diesen API-Host oder den stdin
  Ihres lokalen Agentenbefehls (der dann tut, was dieser Agent tut).
- API-Schlüssel werden vom Prozess `gitpoulpi` selbst verwendet und erreichen nie den
  Daemon, die anderen Clients des Fensters oder einen MCP-Client.
- Erzeugter Text wird nie von selbst committet, gepusht oder angewendet.

### Was aus einer Anfrage entfernt wird

Jede Anfrage, gleich welche Aktion sie sendet und welcher Anbieter sie erhält (eine gehostete API oder ein lokaler Agent), durchläuft zuerst denselben Bereinigungsschritt:

- **Geheimnisse werden geschwärzt** (`[ai.privacy] redact_secrets`, standardmäßig an). Ein Token mit bekanntem Präfix, ein privater Schlüsselblock, ein JWT, ein Passwort in einer URL und eine Zeile `password = value`, deren Wert zufällig aussieht, werden durch `[REDACTED:<rule>]` ersetzt. Das ist nicht erschöpfend: Es findet die gängigen Formen (derselbe Scanner wie bei `rewrite scan-secrets`).
- **Sensible Dateien werden ausgelassen** (`[ai.privacy] exclude_sensitive_files`, standardmäßig an). Der Diff einer Datei, die zu `[privacy] sensitive_patterns` passt (`.env` und `.env.*` außer den Dateien `.example`, `.sample` und `.template`, `*.pem`, `*.key`, private SSH-Schlüssel, `*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`), gehört nicht zur Anfrage. Ein aufgeteiltes Stück oder eine Dateiliste des Composers, die eine solche Datei nennt, wird genauso gekürzt, sodass ein Plan dafür als unvollständig abgelehnt werden kann: Stagen oder teilen Sie diese Datei selbst.

Eines von beiden auszuschalten ist eine bewusste Entscheidung, und Einstellungen > KI zeigt darunter eine rote Zeile, solange es aus ist.

**Was gesendet wird, bevor es gesendet wird.** Bevor eine Anfrage hinausgeht, zeigt die TUI den Dialog **Prüfen, was gesendet wird**: die Anweisungen und die Anfrage nach der Bereinigung (derselbe Bereich wie in den Dialogen zum Umschreiben des Verlaufs, scrollbar, `Alt+W` zeigt oder verbirgt ihn), wie viele Geheimnisse ersetzt und welche Dateien ausgelassen wurden (eine Regel und eine maskierte Vorschau wie `ghp_****abcd`, nie ein Wert), ein Kontrollkästchen **Für <Aktion> nicht mehr anzeigen** und **Senden** / **Abbrechen** (`Esc`, das Schließen-Kreuz und ein Klick außerhalb brechen ab; nichts verlässt den Rechner vor Senden). Das Kontrollkästchen schreibt `[ai.sent_preview] <action> = false`; Einstellungen > KI > *Gesendetes anzeigen vor...* hat einen Schalter je Aktion und **Alle wieder anzeigen**. Die beiden Dialoge zum Umschreiben des Verlaufs zeigen ihre Anfrage ohnehin, daher haben sie keinen Schalter.

**Auf der Kommandozeile** trägt jede Antwort von `ai ...` (und `hosting review suggest`) `privacy: {redactions: N, excluded_files: [...]}`, und `--print-sent` gibt den bereinigten System- und Benutzertext aus, als `{sent: [{system, user}], privacy, dry_run: true}`, ohne das Modell aufzurufen (es braucht `ai.enabled = true`, aber keinen Schlüssel und kein Netzwerk).

**Eine Grenze.** Ein lokaler Agent, dessen Arbeitsordner das Repository ist (`workdir = repository`, oder *Den Agenten im Repository ausführen* bei einem Review), kann die Dateien selbst lesen: Die Bereinigung deckt ab, was GitPoulpi ihm sendet, nicht, was er von sich aus öffnet. Verwenden Sie den Arbeitsordner `dedicated`, um ihn aus dem Repository herauszuhalten.

## Fehlerbehebung

| Symptom | Abhilfe |
|---|---|
| Nirgends ein `✦` | Die KI ist aus: *Einstellungen > KI > KI aktivieren*. |
| Das `✦` ist grau | Es ist nichts gestagt (Commit-Nachricht). |
| *Verbindung testen*: Befehl nicht gefunden | Der **Befehl** des lokalen Agenten liegt nicht im `PATH`, den GitPoulpi sieht: Starten Sie GitPoulpi dort, wo `claude --version` funktioniert, oder geben Sie den vollständigen Pfad an. |
| Ein lokaler Agent hängt oder zeigt einen interaktiven Bildschirm | Fügen Sie im Ausführungsmodus *Headless* seine Headless-Option (`-p` für Claude Code) zu den **Argumenten** hinzu, oder wählen Sie seinen **Agententyp** erneut, um den Headless-Aufruf wiederherzustellen. Der interaktive Bildschirm ist das, wofür der Ausführungsmodus *Interaktiv (virtuelles Terminal)* da ist. |
| "Use "Open the agent"..." | Der Agent hat eine Frage beim ersten Start gestellt (dem Ordner vertrauen, anmelden). Nutzen Sie **Agent öffnen...** am Profil, beantworten Sie sie, schließen Sie den Agenten und versuchen Sie es erneut. |
| Eine Anfrage im virtuellen Terminal läuft in das Zeitlimit | Die Meldung endet mit den letzten Zeilen des Agentenbildschirms: Eine Rückfrage zu Berechtigungen, eine Anmeldung oder eine Update-Frage erscheint dort. Beantworten Sie sie mit **Agent öffnen...**, oder erhöhen Sie das **Zeitlimit**. |
| `fehlgeschlagen:` 401 | Falscher Schlüssel oder Schlüssel eines anderen Anbieters: erneut eingeben. |
| `fehlgeschlagen:` 404 | Das **Modell** existiert dort nicht: leeren Sie es, um den Standard zu nutzen. |
| `fehlgeschlagen:` Verbindungsfehler (Ollama, LM Studio) | Der Server läuft nicht, oder die **Basis-URL** hat den falschen Port oder es fehlt `/v1`. |
| Der Schlüssel zeigt nach dem Tippen `(nicht gesetzt)` | Kein Credential-Helper: Richten Sie einen ein (`git config --global credential.helper ...`) und geben Sie ihn erneut ein. |
| *Über dem KI-Limit* | Eine Eingabe der Anfrage liegt über einem Limit aus `[ai.limits]`: wählen Sie *Alles senden*, *Gekürzt senden*, *Einstellung öffnen* oder *Abbrechen* (siehe [Eingabelimits](#eingabelimits)). |
| „Konflikt zu groß zum Senden“ | Der Konflikt liegt über `ai.limits.conflict_bytes` (standardmäßig 40000). *Trotzdem senden* sendet ihn einmal; *Einstellung öffnen* führt zum Limit unter Einstellungen > KI > Limits. |
| Falsche Sprache oder falscher Stil | Fügen Sie Anweisungen hinzu; mit den Chips *Schnell einfügen* des Editors geht es schnell. |
| Der Composer sagt, der Plan sei ungültig | Das Modell hat eine Datei übersprungen oder wiederholt: erneut versuchen oder ein stärkeres Modell nutzen. |

## Ältere Konfigurationsdateien

Eine ältere `config.toml` mit `[[ai.presets]]` und `[ai.defaults]` lässt sich weiter laden:

- Die Standard-Voreinstellung jeder Aktion wird zu ihrem `[ai.actions.<action>]`
  (`system_prompt` wird zu `instructions`).
- Jede alte Voreinstellung bleibt in `[[ai.legacy_presets]]` erhalten. Einstellungen > KI
  zeigt sie unter *Alte Voreinstellungen*, mit *Archiv löschen*.
- Das alte `[preferences.ai_presets]` eines Repositorys wird zu einer
  Repository-Überschreibung.

Der eingebaute Prompt kommt jetzt immer zuerst. Hat ein alter Prompt einen eingebauten
Prompt kopiert, entfernen Sie den kopierten Teil und behalten Sie nur, was Sie
hinzufügen wollen.
