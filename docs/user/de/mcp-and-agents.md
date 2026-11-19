[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · **Deutsch** · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP und Agenten

Ein KI-Agent (Claude Code, Cursor, Codex, ...) kann GitPoulpi über seinen
**MCP-Server** oder seine **CLI** steuern. Beide laufen über denselben
Hintergrund-Daemon wie das Fenster, sodass das, was der Agent tut, live in Ihrem
geöffneten GitPoulpi erscheint, und beide führen für jeden Befehl genau denselben Code
aus.

Diese Seite handelt von *externen* Agenten. Die KI-Schaltflächen von GitPoulpi selbst
(Commit-Nachrichten, Erklärungen, der Commit-Composer) werden getrennt konfiguriert:
siehe [KI-Unterstützung](ai.md).

## Schnelle Einrichtung

1. Legen Sie `gitpoulpi` in Ihren `PATH` ([Installation](installation.md)). Der Server ist `gitpoulpi mcp serve`; das Relay `gitpoulpi-mcp` im Archiv funktioniert weiterhin als Alias, eine bestehende Agent-Konfiguration, die es nennt, läuft also weiter.
2. Registrieren Sie den Server bei Ihrem Agenten. Er spricht stdio: nichts zu starten,
   kein Port.
3. Legen Sie optional in der Tabelle `[mcp]` der `config.toml` fest, was der Agent
   darf (unten). Die Standardwerte lassen ihn das Repository lesen (Änderungen werden abgelehnt, bis
   Sie `allow_mutations = true` setzen), Ihnen
   Dinge erst nach Ihrer Bestätigung zeigen und nie Dialoge vorausfüllen.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` schreibt es stattdessen in die `.mcp.json` des Repositorys, sodass
alle, die das Projekt öffnen, es bekommen. Dieselbe Datei von Hand, mit den optionalen
Extras:

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "command": "gitpoulpi",
      "args": ["mcp", "serve", "--stdio", "--ui-control", "auto"],
      "env": { "GITPOULPI_CONFIG_DIR": "/home/me/.config/gitpoulpi-agent" }
    }
  }
}
```

- `--ui-control auto|ask|off` überschreibt `[mcp] ui_control` nur für diesen Agenten.
- `GITPOULPI_CONFIG_DIR` gibt dem Agenten seine eigene `config.toml` (eigene
  `[mcp]`-Grenzen, kein Zugriff auf Ihre Einstellungen). Lassen Sie es weg, um Ihre zu
  teilen.
- `gitpoulpi --repo <path> mcp serve --stdio` bindet den Server an ein Repository;
  sonst wirkt ein Tool auf das Repository um das Arbeitsverzeichnis des Servers oder
  auf das Argument `repo` des Aufrufs.

**Andere Clients**: jeder MCP-Client, der einen stdio-Server startet.

| Client | Wo | Was |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | die MCP-JSON-Datei des Clients | der Block `mcpServers` oben (VS Code nennt den Schlüssel `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Alles andere | seine Dokumentation | der Befehl `gitpoulpi mcp serve` (standardmäßig stdio; `gitpoulpi-mcp` ist derselbe Server für Konfigurationen, die es schon nutzen) |

**Über HTTP** (Clients, die sich mit einer URL verbinden):

```
gitpoulpi mcp serve --http                   # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000    # or: --bind 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

Zustandsloses JSON, dieselben Tools, dieselben `[mcp]`-Sperren. Jede Anfrage braucht das
Bearer-Token (`[mcp] http_auth`, standardmäßig an): Geben Sie es mit `gitpoulpi mcp token
--show` aus (oder `--copy`, `--rotate`; das Token liegt in einer Datei nur für den Benutzer im
privaten Datenordner) und übergeben Sie es dem Client als Header:

```
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp \
  --header "Authorization: Bearer <token>"
```

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "type": "http",
      "url": "http://127.0.0.1:8808/mcp",
      "headers": { "Authorization": "Bearer <token>" }
    }
  }
}
```

Ein fehlendes oder falsches Token erhält `401` mit `WWW-Authenticate: Bearer`. Ein erneuertes
Token gilt sofort, ohne Neustart des Servers. `http_auth` auszuschalten ist für einen
Loopback-Endpunkt gedacht, bei dem Sie jedem lokalen Konto vertrauen; der Server warnt dann auf
stderr.

### Den Endpunkt von einem anderen Rechner erreichen

Die sicherste Einrichtung braucht gar keinen entfernten Listener: Behalten Sie die Loopback-Adresse
und leiten Sie den Port per SSH vom Rechner weiter, auf dem der Agent läuft, `ssh -L
8808:127.0.0.1:8808 host`. Dann kommt `--allow-remote` nicht ins Spiel.

`--allow-remote` (eine Nicht-Loopback-Adresse) wird abgelehnt, außer die Authentifizierung ist an
und ein Token ist vorhanden **und** der Datenverkehr ist verschlüsselt. Die Ablehnung nennt, was
fehlt. Es gibt zwei Wege zur Verschlüsselung:

- **TLS im Server** (`[mcp] tls = "self_signed"` oder `"files"`). Der Server beendet TLS selbst:
  Er hält den Schlüssel, führt den Handshake aus, und nur ein abgeschlossener Handshake erreicht
  die HTTP-Schicht; ein Klartext-HTTP-Client scheitert am Handshake, und daneben gibt es keinen
  unverschlüsselten Listener. Die Verschlüsselung ist also eine Tatsache, die der Server bei jeder
  Verbindung herstellt, keine Annahme. Bei `self_signed` erzeugt der erste Start ein Paar im
  privaten Datenordner und gibt dessen SHA-256-Fingerabdruck aus (auch in den Einstellungen); ein
  Client vertraut dem Server, indem er diesen Fingerabdruck oder die Zertifikatsdatei anheftet. Bei
  `files` setzen Sie `tls_cert` und `tls_key` auf Ihre PEM-Dateien, und Clients prüfen das
  Zertifikat anhand seiner CA. Clients authentifizieren sich mit dem Bearer-Token;
  Client-Zertifikate werden nicht verwendet.
- **Ein Tunnel oder Reverse-Proxy** (`[mcp] behind_proxy = true`). Ein Tunnel-Agent oder Proxy
  verbindet sich wie jeder lokale oder LAN-Client, und seine Header lassen sich fälschen, daher
  kann der Server von innen nicht erkennen, ob einer da ist. Deshalb ist es eine ausdrückliche
  Bestätigung (in den Einstellungen mit roter Warnung angezeigt) statt einer Erkennung. Betreiben
  Sie den Tunnel möglichst auf demselben Rechner und behalten Sie die Loopback-Adresse. Bei einer
  Nicht-Loopback-Adresse hinter einem Proxy ist der Abschnitt zwischen Proxy und Server in diesem
  Netzwerk Klartext.

**Ohne MCP** nutzt ein Agent mit einer Shell die CLI: `gitpoulpi --json <Befehl>` (siehe
die [CLI-Referenz](cli-reference.md)). Geben Sie ihm den fertigen Skill
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), der sagt, wann und wie
GitPoulpi statt rohem `git` zu verwenden ist.

## Berechtigungen

| Schalter | Standard | Wirkung |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` (und sein Alias `gitpoulpi-mcp`) verweigern den Start (Einstellungen > Agenten & MCP > *MCP-Server*). Wird beim Start des Servers gelesen: Ein laufender Server behält seine Einstellungen. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Adresse, die `--http` nutzt, wenn keine angegeben ist (nur Loopback, außer `--allow-remote`, das Authentifizierung und Verschlüsselung braucht, siehe oben). Einstellungen > Agenten & MCP > *Port des MCP-Servers* ändert den Port und behält den Host. |
| `[mcp] http_auth` | `true` | `--http` braucht `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Einstellungen > Agenten & MCP > *HTTP-Authentifizierung*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` oder `files` (Ihre PEM-Pfade): Der Server beendet TLS selbst. Einstellungen > Agenten & MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Sie bestätigen, dass ein Tunnel oder Reverse-Proxy den Datenverkehr verschlüsselt, was `--allow-remote` ohne TLS ermöglicht. |
| `[mcp] allow_mutations` | `false` | Aus: Jedes Tool, das etwas ändert, wird abgelehnt; Lesezugriffe, `ui_*` und Vorschläge funktionieren weiter. `true` lässt den Agenten committen, pushen, resetten... |
| `[mcp] mutation_allowlist` | `[]` | Nicht leer: Nur diese Tools dürfen etwas ändern, z. B. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: Der Agent kann Ihr Fenster nicht bewegen (`ui_state` / `ui_viewers` antworten weiterhin). |
| `[mcp] ui_control` | `"ask"` | `ask`: ein Chip in Ihrem Fenster, den Sie bestätigen; `auto`: angewandt, außer Sie sind beschäftigt; `off`: abgelehnt. Pro Agent: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Lässt den Agenten vorausgefüllte Dialoge öffnen, die Sie bestätigen (Einstellungen > Agenten & MCP > *KI-Agenten Änderungen zur Bestätigung vorschlagen lassen*). |
| `[mcp] allow_sensitive_staging` | `false` | Nur in der `config.toml`. Aus: Das Werkzeug `stage` bietet `allow_sensitive` nicht an, und ein Staging, das eine sensible Datei hinzufügen würde (ein privater Schlüssel, eine `.env`, ein erkanntes Token), wird mit `{error: "sensitive_files", files}` abgelehnt: Der Agent soll Sie fragen. `true` lässt ihn `allow_sensitive` übergeben. Siehe [Staging und Committen](staging-and-committing.md#sensible-dateien). |
| `[mcp] allow_history_purge` | `false` | Nur in der `config.toml`. `true` führt das Werkzeug `rewrite_purge` auf, das den alten Verlauf eines Umschreibens (und dessen Rückgängig) endgültig löscht. Aus: Der Agent sieht es nicht. Es braucht weiterhin `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | Der Hauptschalter Ihres Fensters (Einstellungen > Agenten & MCP > *KI-Agenten die Ansicht steuern lassen*). Aus: Kein Agent erreicht Ihr Fenster. |
| `[ui] launch` | `"off"` | Ob ein Agent ein GitPoulpi-Fenster öffnen darf, wenn keines offen ist (siehe [unten](#wenn-kein-fenster-offen-ist-oder-es-in-einer-anderen-sitzung-ist)). |

Was das in der Praxis heißt:

- **Ab Werk ist der Agent nur lesend**: Er kann lesen, Ihnen Dinge in Ihrem Fenster
  zeigen (`ui_*`) und, wenn Sie es erlauben, Änderungen vorschlagen, die Sie bestätigen.
  Damit er das Repository ändern darf (Commit, Push, Reset...), setzen Sie
  `allow_mutations = true`; für eine enge Freigabe lassen Sie es `true` mit einer
  `mutation_allowlist`. Jedes Tool trägt MCP-Annotationen (`readOnlyHint`,
  `destructiveHint`, `openWorldHint`), sodass ein Client wie Claude Code Sie vor den
  riskanten weiterhin fragt.
- **Unterhaltungen von Issues und Pull Requests**: `hosting_issue_comments` und
  `hosting_review_conversation` lesen die ganze Zeitleiste (Lesezugriffe, offene Welt: Sie
  rufen den Host auf); `hosting_markdown` bereinigt einen Text und findet seine Verweise und
  ruft nichts auf (geschlossene Welt). Senden (`hosting_issue_comment`,
  `hosting_review_comment`, `hosting_review_reply`) und Bearbeiten
  (`hosting_*_comment_edit`) sind gewöhnliche Mutationen, als offene Welt markiert: Sie gehen
  sofort an den Host, unter der Identität des Tokens (`posted_as` in der Antwort; ein
  Bot-Token sendet als der Bot), sodass `allow_mutations` und `mutation_allowlist`
  entscheiden, ob ein Agent es darf. Das Löschen eines Kommentars (`hosting_*_comment_delete`) ist
  destruktiv. Nach dem Senden, Bearbeiten oder Löschen wird der Daemon informiert, und ein offenes
  Fenster des Repositorys aktualisiert dieses Element. `ui_show_pr` / `ui_show_issue`
  nehmen `--view conversation`, um die Unterhaltung statt der Zeile zu öffnen.
- **Konflikte**: `conflicts_check` (eine Vorhersage, vor jedem Merge), `conflicts_list`, `conflicts_show` und `conflicts_state` sind
  Lesezugriffe; `conflicts_resolve` (es verwirft die nicht behaltene Seite, `--delete`
  entfernt die Datei) und `conflicts_abort` (verwirft den ganzen
  Merge/Rebase/Cherry-Pick) sind destruktiv; `conflicts_unresolve` ist ein einfacher
  Schreibzugriff. Ein abgelehnter Aufruf (`stale`, `not_conflicted`, `pick_incomplete`...)
  antwortet mit `{"refused": {...}}` und Exit-Code 2, statt zu scheitern.
- **Hängende Zustände**: `conflicts_am_continue` ist ein Schreibzugriff,
  `conflicts_am_skip`, `conflicts_am_abort` und `index_lock_remove` sind destruktiv,
  `index_lock_show` ist ein Lesezugriff.
- **Verlauf umschreiben**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (die Größenanalyse), `rewrite_scan_secrets` (Treffer mit geschwärzter Vorschau, nie ein Wert),
  `rewrite_script_check`, `rewrite_script_test` und `rewrite_discard` sind Lesezugriffe
  (eine Vorschau ändert nichts; sie hält nur einen Plan im Daemon und auf der Platte im
  Git-Verzeichnis, sodass `rewrite_apply` auch nach einem Neustart funktioniert).
  `rewrite_apply` und `rewrite_run` sind destruktiv: Sie verschieben viele Refs auf
  umgeschriebene Commits als einen Undo-Eintrag (`undo` stellt sie wieder her) und
  pushen nie. Das Rezept als `recipe_json` (oder `recipe`, ein Pfad) übergeben, Geheimnisse
  als `secrets_file`, nie inline.
  Shell-, Python- und externe Lua-Skripte (`script_language`) haben keine Sandbox, und
  **ein Agent kann ihnen nicht vertrauen**: Es gibt kein Werkzeug `rewrite_script_trust` und
  kein Argument `trust`, ein solches Skript läuft also nur, nachdem ein Mensch
  `gitpoulpi rewrite script trust FILE` für genau diesen Text ausgeführt hat (ein geändertes
  Skript wird erneut abgelehnt, `script_not_trusted` mit dem Hash in `refused.script`), und nur,
  wenn `rewrite.external_scripts` seine Sprache einschaltet (`scripts_disabled`).
  Eine Ablehnung antwortet mit
  `{"refused": {"reason", "message", "detail"}}`, und ein Plan, der gepushte Refs
  verschiebt, antwortet mit `{"needs_confirmation": ...}` (Exit-Code 3), bis der Aufruf
  `yes` setzt - zeigen Sie ihn vorher dem Nutzer.
- **Für einen Agenten nie erreichbar**: ein Token speichern (`hosting login`), einen
  SSH-Schlüssel erzeugen und hochladen, `config set` (er könnte seine eigenen Grenzen
  umschreiben) und der Lebenszyklus des Daemons. Ein Token wird dem Agenten nie gezeigt.
- **Kontoweite Listen**: `hosting_radar` ist ein Lesen (Requests, die auf Ihr
  Review warten, offene Requests, zugewiesene Issues über jedes Repository eines Profils,
  aus einem Fünf-Minuten-Cache, außer bei `refresh`); `hosting_integrations_radar`
  ändert die Tabelle `[integrations.radar]` eines Profils, also sperrt `allow_mutations` es
  (auch bei einem Aufruf ohne Option, der nur die Einstellungen ausgibt).
- **Review-Kontext**: `ai_review_context` zeigt, was ein Code-Review-Entwurf außer dem Diff
  erhält (CLAUDE.md, AGENTS.md, eine zusätzliche Datei, wo ein lokaler Agent läuft). Mit
  einem Einstellungsargument speichert es einen Standard (global oder den eigenen des `repo` bzw. des `project`),
  also sperrt `allow_mutations` es, wie die anderen Tools, die eine Einstellung ändern; ohne eines
  gibt es nur aus. Wie jedes `ai_*`-Tool wird es nur gelistet, solange `ai.enabled` an ist.
  `hosting_review_suggest` nimmt dieselben Argumente, aber nur für einen Lauf.
- **Die `[mcp]`-Sperren gelten für den MCP-Server.** Die CLI ist Ihre eigene Shell und
  wird nicht durch `allow_mutations` gesperrt; nur `ui_control` und `allow_ui_propose`
  gelten auch für `gitpoulpi ui ...`.
- Ein abgelehnter Aufruf ist ein gewöhnlicher Tool-Fehler (`isError: true`, der Grund in
  `structuredContent.error`), nie ein stilles Nichts.

```toml
[mcp]
allow_mutations = false        # true: the agent may change the repository
mutation_allowlist = []        # e.g. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" lets the agent move your view without asking
allow_ui_propose = false       # true: the agent may pre-fill Reword / Split / commit / branch / tag dialogs
allow_sensitive_staging = false  # true: the agent may stage files the staging guard flags
allow_history_purge = false    # true: the agent sees rewrite_purge (deletes old history for good)
```

## Die Tools

Ein Tool pro CLI-Befehl, benannt nach seinem Pfad mit `_` (`branch create` heißt
`branch_create`). Jedes nimmt die Argumente des Befehls plus `repo` (das Repository;
Standard: das des Servers) und `session`. Das `structuredContent` eines erfolgreichen
Aufrufs ist genau das, was `gitpoulpi --json <Befehl>` ausgibt. **Kind** (die Spalte
„Kind“, englisch wie im Server): *read* ändert nie etwas; *write* und *destructive* sind
Mutationen (gesperrt); *network* spricht mit einem Remote oder Host; *view* bewegt nur
Ihr Fenster; *propose* öffnet einen Dialog, den Sie bestätigen.

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_hosting_reply` | `gitpoulpi ai hosting-reply` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `ai_worktree_name` | `gitpoulpi ai worktree-name` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
| `apply_commit` | `gitpoulpi apply-commit` | write |
| `autostash_start_over` | `gitpoulpi autostash start-over` | destructive |
| `blame` | `gitpoulpi blame` | read |
| `branch_create` | `gitpoulpi branch create` | write |
| `branch_delete` | `gitpoulpi branch delete` | destructive |
| `branch_delete_remote` | `gitpoulpi branch delete-remote` | destructive, network |
| `branch_fast_forward` | `gitpoulpi branch fast-forward` | write |
| `branch_list` | `gitpoulpi branch list` | read |
| `branch_rename` | `gitpoulpi branch rename` | write |
| `branch_set_upstream` | `gitpoulpi branch set-upstream` | write |
| `branch_track` | `gitpoulpi branch track` | write |
| `branch_unset_upstream` | `gitpoulpi branch unset-upstream` | write |
| `branch_upstream_add` | `gitpoulpi branch upstream add` | write |
| `branch_upstream_list` | `gitpoulpi branch upstream list` | read |
| `branch_upstream_primary` | `gitpoulpi branch upstream primary` | write |
| `branch_upstream_pull` | `gitpoulpi branch upstream pull` | write |
| `branch_upstream_remove` | `gitpoulpi branch upstream remove` | write |
| `checkout` | `gitpoulpi checkout` | write |
| `cherry_pick` | `gitpoulpi cherry-pick` | write |
| `clone` | `gitpoulpi clone` | write, network |
| `commit` | `gitpoulpi commit` | write |
| `config_get` | `gitpoulpi config get` | read |
| `config_path` | `gitpoulpi config path` | read |
| `conflicts_abort` | `gitpoulpi conflicts abort` | destructive |
| `conflicts_am_abort` | `gitpoulpi conflicts am-abort` | destructive |
| `conflicts_am_continue` | `gitpoulpi conflicts am-continue` | write |
| `conflicts_am_skip` | `gitpoulpi conflicts am-skip` | destructive |
| `conflicts_check` | `gitpoulpi conflicts check` | read |
| `conflicts_list` | `gitpoulpi conflicts list` | read |
| `conflicts_resolve` | `gitpoulpi conflicts resolve` | destructive |
| `conflicts_show` | `gitpoulpi conflicts show` | read |
| `conflicts_skip` | `gitpoulpi conflicts skip` | destructive |
| `conflicts_state` | `gitpoulpi conflicts state` | read |
| `conflicts_unresolve` | `gitpoulpi conflicts unresolve` | write |
| `convert` | `gitpoulpi convert` | write |
| `diff` | `gitpoulpi diff` | read |
| `discard` | `gitpoulpi discard` | destructive |
| `fetch` | `gitpoulpi fetch` | write, network |
| `file_history` | `gitpoulpi file-history` | read |
| `flow_config_init` | `gitpoulpi flow config init` | write |
| `flow_config_show` | `gitpoulpi flow config show` | read |
| `flow_detect` | `gitpoulpi flow detect` | read |
| `flow_finish` | `gitpoulpi flow finish` | destructive, network |
| `flow_import` | `gitpoulpi flow import` | write |
| `flow_issue_new` | `gitpoulpi flow issue new` | write, network |
| `flow_issue_wait` | `gitpoulpi flow issue wait` | write, network |
| `flow_pick_up` | `gitpoulpi flow pick-up` | write, network |
| `flow_promote` | `gitpoulpi flow promote` | destructive |
| `flow_set_parent` | `gitpoulpi flow set-parent` | write |
| `flow_start` | `gitpoulpi flow start` | write, network |
| `flow_status` | `gitpoulpi flow status` | read |
| `flow_sync` | `gitpoulpi flow sync` | destructive |
| `flow_templates` | `gitpoulpi flow templates` | read |
| `hosting_board_move` | `gitpoulpi hosting board move` | write, network |
| `hosting_board_show` | `gitpoulpi hosting board show` | read, network |
| `hosting_checkout_pr` | `gitpoulpi hosting checkout-pr` | write, network |
| `hosting_clone` | `gitpoulpi hosting clone` | write, network |
| `hosting_edit_url` | `gitpoulpi hosting edit-url` | read, network |
| `hosting_file` | `gitpoulpi hosting file` | read, network |
| `hosting_integrations_add` | `gitpoulpi hosting integrations add` | write |
| `hosting_integrations_list` | `gitpoulpi hosting integrations list` | read |
| `hosting_integrations_radar` | `gitpoulpi hosting integrations radar` | write |
| `hosting_integrations_remove` | `gitpoulpi hosting integrations remove` | destructive |
| `hosting_integrations_test` | `gitpoulpi hosting integrations test` | read, network |
| `hosting_assignment` | `gitpoulpi hosting assignment` | write |
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
| `hosting_issue_comment` | `gitpoulpi hosting issue comment` | write, network |
| `hosting_issue_comment_delete` | `gitpoulpi hosting issue comment-delete` | destructive, network |
| `hosting_issue_comment_edit` | `gitpoulpi hosting issue comment-edit` | write, network |
| `hosting_issue_comments` | `gitpoulpi hosting issue comments` | read, network |
| `hosting_issue_create` | `gitpoulpi hosting issue create` | write, network |
| `hosting_issue_edit` | `gitpoulpi hosting issue edit` | write, network |
| `hosting_issue_move` | `gitpoulpi hosting issue move` | write |
| `hosting_issue_options` | `gitpoulpi hosting issue options` | read, network |
| `hosting_issue_reopen` | `gitpoulpi hosting issue reopen` | write |
| `hosting_issues` | `gitpoulpi hosting issues` | read, network |
| `hosting_labels` | `gitpoulpi hosting labels` | read, network |
| `hosting_link` | `gitpoulpi hosting link` | write |
| `hosting_list` | `gitpoulpi hosting list` | read |
| `hosting_logout` | `gitpoulpi hosting logout` | destructive |
| `hosting_markdown` | `gitpoulpi hosting markdown` | read |
| `hosting_members` | `gitpoulpi hosting members` | read, network |
| `hosting_milestones` | `gitpoulpi hosting milestones` | read, network |
| `hosting_open` | `gitpoulpi hosting open` | read |
| `hosting_pr` | `gitpoulpi hosting pr` | write, network |
| `hosting_pr_edit` | `gitpoulpi hosting pr-edit` | write, network |
| `hosting_pr_for_branch` | `gitpoulpi hosting pr-for-branch` | read, network |
| `hosting_projects` | `gitpoulpi hosting projects` | read, network |
| `hosting_prs` | `gitpoulpi hosting prs` | read, network |
| `hosting_radar` | `gitpoulpi hosting radar` | read, network |
| `hosting_refs` | `gitpoulpi hosting refs` | read |
| `hosting_review_abandon` | `gitpoulpi hosting review abandon` | write, network |
| `hosting_review_comment` | `gitpoulpi hosting review comment` | write, network |
| `hosting_review_comment_delete` | `gitpoulpi hosting review comment-delete` | destructive, network |
| `hosting_review_comment_edit` | `gitpoulpi hosting review comment-edit` | write, network |
| `hosting_review_conversation` | `gitpoulpi hosting review conversation` | read, network |
| `hosting_review_diff` | `gitpoulpi hosting review diff` | read, network |
| `hosting_review_fetch` | `gitpoulpi hosting review fetch` | write, network |
| `hosting_review_push` | `gitpoulpi hosting review push` | write, network |
| `hosting_review_reply` | `gitpoulpi hosting review reply` | write, network |
| `hosting_review_resolve` | `gitpoulpi hosting review resolve` | write, network |
| `hosting_review_show` | `gitpoulpi hosting review show` | read, network |
| `hosting_review_submit` | `gitpoulpi hosting review submit` | write, network |
| `hosting_review_submit_change` | `gitpoulpi hosting review submit-change` | write, network |
| `hosting_review_suggest` | `gitpoulpi hosting review suggest` | read, network |
| `hosting_review_threads` | `gitpoulpi hosting review threads` | read, network |
| `hosting_ssh_key_delete` | `gitpoulpi hosting ssh-key delete` | destructive, network |
| `hosting_ssh_key_list` | `gitpoulpi hosting ssh-key list` | read, network |
| `hosting_use` | `gitpoulpi hosting use` | write |
| `ignore` | `gitpoulpi ignore` | write |
| `index_lock_remove` | `gitpoulpi index-lock remove` | destructive |
| `index_lock_show` | `gitpoulpi index-lock show` | read |
| `job_cancel` | `gitpoulpi job cancel` | write |
| `job_list` | `gitpoulpi job list` | read |
| `job_wait` | `gitpoulpi job wait` | read |
| `lfs_checkout` | `gitpoulpi lfs checkout` | write |
| `lfs_fetch` | `gitpoulpi lfs fetch` | write, network |
| `lfs_files` | `gitpoulpi lfs files` | read |
| `lfs_install` | `gitpoulpi lfs install` | write |
| `lfs_lock` | `gitpoulpi lfs lock` | write, network |
| `lfs_locks` | `gitpoulpi lfs locks` | read, network |
| `lfs_prune` | `gitpoulpi lfs prune` | destructive |
| `lfs_pull` | `gitpoulpi lfs pull` | write, network |
| `lfs_push` | `gitpoulpi lfs push` | destructive, network |
| `lfs_status` | `gitpoulpi lfs status` | read |
| `lfs_track` | `gitpoulpi lfs track` | write |
| `lfs_unlock` | `gitpoulpi lfs unlock` | destructive, network |
| `lfs_untrack` | `gitpoulpi lfs untrack` | write |
| `licenses` | `gitpoulpi licenses` | read |
| `log` | `gitpoulpi log` | read |
| `lost_commits` | `gitpoulpi lost-commits` | read |
| `mailmap_add` | `gitpoulpi mailmap add` | write |
| `mailmap_check` | `gitpoulpi mailmap check` | read |
| `mailmap_list` | `gitpoulpi mailmap list` | read |
| `merge` | `gitpoulpi merge` | write |
| `patch_apply` | `gitpoulpi patch apply` | write |
| `patch_create` | `gitpoulpi patch create` | write |
| `path_status` | `gitpoulpi path status` | read |
| `pull` | `gitpoulpi pull` | write, network |
| `push` | `gitpoulpi push` | destructive, network |
| `rebase` | `gitpoulpi rebase` | destructive |
| `rebase_plan` | `gitpoulpi rebase-plan` | read |
| `redo` | `gitpoulpi redo` | destructive |
| `reflog` | `gitpoulpi reflog` | read |
| `reflog_restore` | `gitpoulpi reflog-restore` | destructive |
| `refs_hide` | `gitpoulpi refs hide` | write |
| `refs_show` | `gitpoulpi refs show` | write |
| `refs_solo` | `gitpoulpi refs solo` | write |
| `remote_add` | `gitpoulpi remote add` | write |
| `remote_clear_push_url` | `gitpoulpi remote clear-push-url` | write |
| `remote_default` | `gitpoulpi remote default` | write |
| `remote_list` | `gitpoulpi remote list` | read |
| `remote_remove` | `gitpoulpi remote remove` | destructive |
| `remote_rename` | `gitpoulpi remote rename` | write |
| `remote_set_url` | `gitpoulpi remote set-url` | write |
| `remote_group_abort` | `gitpoulpi remote-group abort` | destructive |
| `remote_group_continue` | `gitpoulpi remote-group continue` | destructive, network |
| `remote_group_plan` | `gitpoulpi remote-group plan` | read, network |
| `remote_group_retry` | `gitpoulpi remote-group retry` | write, network |
| `remote_group_status` | `gitpoulpi remote-group status` | read |
| `repo_close` | `gitpoulpi repo close` | write |
| `repo_discover` | `gitpoulpi repo discover` | read |
| `repo_init` | `gitpoulpi repo init` | write |
| `repo_list` | `gitpoulpi repo list` | read |
| `repo_open` | `gitpoulpi repo open` | write |
| `repo_recent` | `gitpoulpi repo recent` | read |
| `repo_auth_show` | `gitpoulpi repo-auth show` | read |
| `repo_auth_test` | `gitpoulpi repo-auth test` | write, network |
| `repo_config_set` | `gitpoulpi repo-config set` | write |
| `repo_config_show` | `gitpoulpi repo-config show` | read |
| `repo_config_unset` | `gitpoulpi repo-config unset` | write |
| `reset` | `gitpoulpi reset` | destructive |
| `revert` | `gitpoulpi revert` | write |
| `reword` | `gitpoulpi reword` | destructive |
| `rewrite_analyze` | `gitpoulpi rewrite analyze` | read |
| `rewrite_apply` | `gitpoulpi rewrite apply` | destructive |
| `rewrite_check_signing` | `gitpoulpi rewrite check-signing` | read |
| `rewrite_diff` | `gitpoulpi rewrite diff` | read |
| `rewrite_discard` | `gitpoulpi rewrite discard` | read |
| `rewrite_maps` | `gitpoulpi rewrite maps` | write |
| `rewrite_note` | `gitpoulpi rewrite note` | read |
| `rewrite_preview` | `gitpoulpi rewrite preview` | read |
| `rewrite_purge` | `gitpoulpi rewrite purge` | destructive |
| `rewrite_push` | `gitpoulpi rewrite push` | destructive, network |
| `rewrite_rows` | `gitpoulpi rewrite rows` | read |
| `rewrite_run` | `gitpoulpi rewrite run` | destructive |
| `rewrite_scan_secrets` | `gitpoulpi rewrite scan-secrets` | read |
| `rewrite_script_check` | `gitpoulpi rewrite script check` | read |
| `rewrite_script_test` | `gitpoulpi rewrite script test` | read |
| `rewrite_tools` | `gitpoulpi rewrite tools` | read |
| `rm_file` | `gitpoulpi rm-file` | destructive |
| `search` | `gitpoulpi search` | read |
| `session_delete` | `gitpoulpi session delete` | destructive |
| `session_list` | `gitpoulpi session list` | read |
| `session_new` | `gitpoulpi session new` | write |
| `session_rename` | `gitpoulpi session rename` | write |
| `show` | `gitpoulpi show` | read |
| `sparse_checkout_add` | `gitpoulpi sparse-checkout add` | write |
| `sparse_checkout_disable` | `gitpoulpi sparse-checkout disable` | write |
| `sparse_checkout_set` | `gitpoulpi sparse-checkout set` | write |
| `sparse_checkout_status` | `gitpoulpi sparse-checkout status` | read |
| `split_commit` | `gitpoulpi split-commit` | destructive |
| `split_source` | `gitpoulpi split-source` | read |
| `ssh_command_clear` | `gitpoulpi ssh-command clear` | write |
| `ssh_command_set` | `gitpoulpi ssh-command set` | write |
| `ssh_command_show` | `gitpoulpi ssh-command show` | read |
| `stage` | `gitpoulpi stage` | write |
| `stash_apply` | `gitpoulpi stash apply` | write |
| `stash_drop` | `gitpoulpi stash drop` | destructive |
| `stash_from_commit` | `gitpoulpi stash from-commit` | write |
| `stash_list` | `gitpoulpi stash list` | read |
| `stash_pop` | `gitpoulpi stash pop` | write |
| `stash_push` | `gitpoulpi stash push` | write |
| `status` | `gitpoulpi status` | read |
| `submodule_init` | `gitpoulpi submodule init` | write |
| `submodule_list` | `gitpoulpi submodule list` | read |
| `submodule_update` | `gitpoulpi submodule update` | write, network |
| `tag_annotate` | `gitpoulpi tag annotate` | write |
| `tag_create` | `gitpoulpi tag create` | write |
| `tag_delete` | `gitpoulpi tag delete` | destructive, network |
| `tag_delete_remote` | `gitpoulpi tag delete-remote` | destructive, network |
| `tag_fetch` | `gitpoulpi tag fetch` | write, network |
| `tag_list` | `gitpoulpi tag list` | read |
| `tag_push` | `gitpoulpi tag push` | write, network |
| `theme_export` | `gitpoulpi theme export` | read |
| `theme_list` | `gitpoulpi theme list` | read |
| `treat` | `gitpoulpi treat` | write |
| `ui_back` | `gitpoulpi ui back` | view |
| `ui_highlight` | `gitpoulpi ui highlight` | view |
| `ui_notify` | `gitpoulpi ui notify` | view |
| `ui_propose_branch` | `gitpoulpi ui propose branch` | propose |
| `ui_propose_commit_message` | `gitpoulpi ui propose commit-message` | propose |
| `ui_propose_resolution` | `gitpoulpi ui propose resolution` | propose |
| `ui_propose_reword` | `gitpoulpi ui propose reword` | propose |
| `ui_propose_rewrite` | `gitpoulpi ui propose rewrite` | propose |
| `ui_propose_split` | `gitpoulpi ui propose split` | propose |
| `ui_propose_tag` | `gitpoulpi ui propose tag` | propose |
| `ui_show_commit` | `gitpoulpi ui show commit` | view |
| `ui_show_conflict` | `gitpoulpi ui show conflict` | view |
| `ui_show_diff` | `gitpoulpi ui show diff` | view |
| `ui_show_file` | `gitpoulpi ui show file` | view |
| `ui_show_home` | `gitpoulpi ui show home` | view |
| `ui_show_issue` | `gitpoulpi ui show issue` | view |
| `ui_show_keys` | `gitpoulpi ui show keys` | view |
| `ui_show_palette` | `gitpoulpi ui show palette` | view |
| `ui_show_pr` | `gitpoulpi ui show pr` | view |
| `ui_show_ref` | `gitpoulpi ui show ref` | view |
| `ui_show_search` | `gitpoulpi ui show search` | view |
| `ui_show_settings` | `gitpoulpi ui show settings` | view |
| `ui_show_tab` | `gitpoulpi ui show tab` | view |
| `ui_show_wip` | `gitpoulpi ui show wip` | view |
| `ui_state` | `gitpoulpi ui state` | view |
| `ui_viewers` | `gitpoulpi ui viewers` | view |
| `undo` | `gitpoulpi undo` | destructive |
| `undo_forget` | `gitpoulpi undo-forget` | write |
| `undo_restore` | `gitpoulpi undo-restore` | write |
| `unstage` | `gitpoulpi unstage` | write |
| `workspace_add` | `gitpoulpi workspace add` | write |
| `workspace_create` | `gitpoulpi workspace create` | write |
| `workspace_delete` | `gitpoulpi workspace delete` | destructive |
| `workspace_list` | `gitpoulpi workspace list` | read |
| `workspace_remove` | `gitpoulpi workspace remove` | write |
| `workspace_rename` | `gitpoulpi workspace rename` | write |
| `workspace_set_ai_instructions` | `gitpoulpi workspace set-ai-instructions` | write |
| `workspace_set_profile` | `gitpoulpi workspace set-profile` | write |
| `workspace_show` | `gitpoulpi workspace show` | read |
| `worktree_add` | `gitpoulpi worktree add` | write |
| `worktree_list` | `gitpoulpi worktree list` | read |
| `worktree_changes` | `gitpoulpi worktree changes` | read |
| `worktree_remove` | `gitpoulpi worktree remove` | destructive |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi hosting integrations trust` | accepting a changed address, account or certificate is the user's decision; an agent is told the security check needs the user |
| `gitpoulpi ai trust` | binding an API key to another address is the user's decision |
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
| `gitpoulpi mcp token` | prints, copies and rotates the key that guards this very server: an agent must not read or replace it |
| `gitpoulpi completions` | shell completion generation, not a GitBackend operation |
| `gitpoulpi shell-init` | prints a wrapper function for the user's own shell, not a GitBackend operation |
| `gitpoulpi desktop shortcut status` | launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer |
| `gitpoulpi desktop shortcut install` | writes files into the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop shortcut remove` | deletes files from the user's launcher, Start menu and Desktop |
| `gitpoulpi alias create` | writes a command into the user's PATH; an agent never changes the user's shell |
| `gitpoulpi alias remove` | deletes a command from the user's PATH; an agent never changes the user's shell |
| `gitpoulpi path add` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi path remove` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi uninstall` | removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine |
| `gitpoulpi launch` | starts a terminal window on the user's desktop; the launcher shortcuts' own entry point |
| `gitpoulpi daemon run` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon start` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon stop` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon restart` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon status` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi rewrite script trust` | trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted |
| `gitpoulpi config set` | an agent must not be able to rewrite its own gating config |
| `gitpoulpi hosting ssh-key create` | writes a private key and uploads it; too dangerous to let an agent automate |
| `gitpoulpi hosting login` | takes a secret token on stdin; a token must not pass through an agent's tool-call transcript |
| `gitpoulpi repo-auth set` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi repo-auth clear` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi flow trust-hooks` | trusting the shell commands a repository commits is a person's decision; an agent can only run hooks a person trusted |
<!-- mcp-tools:end -->

### Aktionen der Branching-Workflows: erst planen, dann ausführen

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` und `flow_set_parent`
([Branching-Workflows](branching-workflows.md#aktionen)) laufen nie bei einem
bloßen Aufruf. `dry_run: true` gibt den Plan (jeden Git-Schritt und -Befehl,
Warnungen, was Rückgängig nicht zurücknehmen kann, Blocker) als normales Ergebnis
zurück; `yes: true` führt ihn aus; mit keinem von beiden wird der Aufruf abgelehnt
und trägt den Plan. Die `[hooks]` der Workflow-Datei sind Shell-Befehle aus einer
Datei, die das Team teilt, daher laufen sie nur mit einem ausdrücklichen
`run_hooks: true`: Der Plan listet sie auf (`hooks`, `hooks_run`), damit Sie dem
Benutzer zuerst die genauen Befehle zeigen können. Ein Merge, der bei Konflikten
anhält, antwortet mit `status: "conflicted"` (ein Fehlerergebnis, Exit-Code 2);
nachdem die Konflikt-Werkzeuge die Dateien gelöst haben, rufen Sie dasselbe Tool
mit `continue_: true` oder `abort: true` auf. Das Öffnen eines Merge Requests
(`finish` im Modus `merge-request`) ist nicht rückgängig zu machen und sagt es im
Plan.


### Was ein Agent mit reinem `git` tut, steht ebenfalls im Rückgängig-Verlauf

Ein Agent, der mit seinem eigenen `git` (nicht über diese Tools) Branches und
Worktrees anlegt, committet, zurücksetzt oder Stashes verwirft, bleibt sichtbar:
Der Daemon vergleicht die Refs des Repositorys mit dem, was er zuletzt gesehen
hat, und zeichnet jede Änderung als Eintrag von `undo --list` mit
`"source": "outside"` auf, samt dem Grund von Git (`outside.message`) und dem
Committer, den Git festgehalten hat. Die Tools `undo`, `redo` und
`undo_restore` behandeln diese Einträge wie jeden anderen, sodass ein Agent
(oder der Benutzer mit `Ctrl+Z`) einen gelöschten Branch zurückholen kann.
Änderungen an Dateien werden nie erfasst. Siehe
[Rückgängig und Wiederholen](undo-redo.md#änderungen-außerhalb-von-gitpoulpi).

## Dem Benutzer Dinge zeigen

Ein Agent kann auf etwas zeigen, statt es zu beschreiben: `ui_show_commit`,
`ui_show_diff` (eine Datei an einer Zeile, inline oder geteilt), `ui_show_file` (Inhalt,
Blame oder Verlauf an einer Zeile), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`,
`ui_show_conflict` (der Merge-Editor auf einer Datei im Konflikt, bei einem Konflikt),
`ui_show_wip` (`composer` setzt den Cursor in die Commit-Zusammenfassung, ohne etwas zu
tippen), `ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette` (die Abfrage
getippt, nie ausgeführt), `ui_show_keys`, `ui_show_settings` (eine Seite und eine
Einstellung, nie bearbeitet), `ui_highlight` (Zeilen des offenen Diffs oder der Datei
hervorheben), `ui_notify` (eine Benachrichtigung) und `ui_back` (die letzte Ansichtsänderung
des Agenten rückgängig machen). `ui_state` liest, was ein Fenster zeigt (`text: true`
fügt den sichtbaren Text hinzu); `ui_viewers` listet die Fenster auf.

Diese Tools **ändern nie das Repository** und können weder tippen, klicken noch eine
Einstellung bearbeiten. Revisionen und Pfade werden geprüft, bevor etwas das Fenster
erreicht.

**Was der Benutzer sieht und entscheidet**:

- `ask` (Standard) oder immer dann, wenn Sie tippen, ein Dialog oder die Einstellungen
  offen sind, Sie ziehen oder in den letzten 1,5 s eine Taste gedrückt haben: Der Agent
  erhält sofort `deferred`, und Ihre Statuszeile zeigt einen Chip *„claude-code möchte ...
  zeigen“*. `Ctrl+N Enter` oder ein Klick zeigt es; `Esc` oder `Ctrl+N Backspace` weist
  es ab. Er läuft nach 60 s ab.
- Angewandt: Die Ansicht bewegt sich, ein Hinweis von 4 s sagt, wer sie bewegt hat, und
  der Tab erhält ein lila `✦` bis zu Ihrer nächsten Taste. `Ctrl+N Backspace` (Palette
  `Agent: Zurück`) geht zurück; das Fenster merkt sich zehn Ansichten.
- Nie wird etwas eingeblendet oder ausgecheckt: Ein Commit, den Ihr Solo-Filter
  ausblendet, antwortet `partial` (`hidden_by_filter`); eine Liste von PRs oder Issues,
  die nicht mit einem Anbieter verknüpft ist, antwortet `hosting_not_linked`.

**Antworten**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, mit dem `state` des Fensters, wenn
angewandt. CLI-Exit-Codes: 0 applied/partial/proposed, 3 deferred, sonst 1. Aufrufe
sind ratenbegrenzt (Burst von 5, dann einer pro Sekunde).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Eine Änderung vorschlagen (`ui_propose_*`)

Mit `allow_ui_propose = true` kann ein Agent einen Ihrer Dialoge **vorausgefüllt**
öffnen, markiert mit *✦ Vorgeschlagen von <Agent>*. Nichts geschieht, bis Sie die
eigene Bestätigung dieses Dialogs drücken; Sie können vorher bearbeiten oder abbrechen.

| Tool | Öffnet | Bestätigen Sie mit |
|---|---|---|
| `ui_propose_reword` (rev, message) | das Reword-Popup | *Nachricht aktualisieren* (ein gepushter Commit fragt weiterhin wegen des Force-Pushs) |
| `ui_propose_commit_message` (summary, body) | das Commit-Formular, ausgefüllt; committet nie | die Commit-Schaltfläche |
| `ui_propose_branch` (name, start point) | das Inline-Feld Neuer Branch | `Enter` |
| `ui_propose_tag` (name, target, message) | das Inline-Feld Neuer Tag (annotiert, wenn es eine Nachricht gibt) | `Enter` |
| `ui_propose_split` (rev, plan file) | der Aufteilungseditor über dem Commit, aus dem Plan gefüllt | *In N aufteilen* (`Ctrl+S`); `Esc` lehnt ab |
| `ui_propose_rewrite` (recipe file) | die Werkbank zum Umschreiben mit den Regeln des Rezepts darin; nichts wird in der Vorschau gezeigt oder angewendet | Ihre eigene *Vorschau*, dann *Anwenden* (`Ctrl+Enter`); `Esc` lehnt ab. Ein Rezept mit einem Skript wird abgelehnt, geheime Werte werden entfernt |
| `ui_propose_resolution` (path, pick oder content file) | der Merge-Editor auf einer Datei im Konflikt, mit den Auswahlen (oder dem ganzen Text) bereits im Ergebnis | *Speichern & als gelöst markieren* (`Ctrl+S`); Schließen lehnt ab |

Ein Vorschlag nach dem anderen; ein Vorschlag für eine Commit-Nachricht wird abgelehnt,
solange Sie einen eigenen Entwurf im Composer haben, und ein Lösungsvorschlag, solange der
Merge-Editor ungespeicherte Arbeit von Ihnen enthält oder wenn sich die Stages der Datei
geändert haben, seit der Agent sie gelesen hat. Eine Lösung, die Sie vor dem Bestätigen
ändern, wird als `accepted_edited` gemeldet. Die Antwort trägt eine `proposal`-ID; was Sie
entschieden haben, steht in den `proposals` von `ui_state` (`offered`, `open`, `accepted`,
`accepted_edited`, `rejected`, `expired`, `failed`).

### Wenn kein Fenster offen ist oder es in einer anderen Sitzung ist

- Ein Fenster gehört zu der Sitzung, in der es gestartet wurde. Hat die Sitzung des
  Agenten keines, sucht `ui show` **ein** Fenster in einer anderen Sitzung, die das
  Repository geöffnet hat, und nutzt es (`routed_session` in der Antwort). Passen
  mehrere: `no_viewer` mit ihren Namen; geben Sie `--session` an.
- **Ein Fenster für den Agenten zu öffnen** ist standardmäßig aus. Setzen Sie `[ui] launch`
  (Einstellungen > Agenten & MCP > *Fenster für Agenten öffnen*) auf eine Terminal- oder
  Multiplexer-Voreinstellung oder einen Befehl `custom`; dann öffnet ein Aufruf mit
  `launch: true` (CLI `gitpoulpi ui --launch show ...`) eines und wartet bis zu 10 s. Mit
  `ui_control = "auto"` öffnet es sich auch ohne die Option; bei `ask` erhält der Agent
  `no_viewer` und sollte es Ihnen sagen.

| Voreinstellung | Befehl |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (Ihr Standardterminal) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (neues Fenster), `zellij` (neuer Bereich), `cmux` (neuer Workspace) | innerhalb eines laufenden Multiplexers, im Repository-Ordner |
| `custom` | `[ui] launch_command`, z. B. `my-term --cd {repo} {gitpoulpi} --session {session}`; in Argumente zerlegt, keine Shell |

Funktioniert eine Voreinstellung nicht mit Ihrer Programmversion, nutzen Sie `custom`.

## Fehler

- Ein fehlerhafter Aufruf (unbekanntes Tool oder Argument, ein fehlendes erforderliches)
  ist ein JSON-RPC-Fehler `-32602 invalid_params`.
- Eine Ablehnung oder ein Git- oder Daemon-Fehler (falsche Revision, Konflikt, ...) ist
  ein Tool-Fehler: `isError: true` mit der Meldung in `content` und
  `structuredContent.error`.
