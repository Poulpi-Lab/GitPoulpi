[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · **Svenska** · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP och agenter

En AI-agent (Claude Code, Cursor, Codex, ...) kan styra GitPoulpi via sin
**MCP-server** eller sin **CLI**. Båda går via samma bakgrundsdaemon som
fönstret, så det agenten gör syns direkt i ditt öppna GitPoulpi, och båda kör
exakt samma kod för varje kommando.

Den här sidan handlar om *externa* agenter. GitPoulpis egna AI-knappar
(commit-meddelanden, förklaringar, commit-composern) ställs in separat: se
[AI-hjälp](ai.md).

## Snabb installation

1. Lägg `gitpoulpi` och `gitpoulpi-mcp` i din `PATH` ([Installation](installation.md)).
2. Registrera servern hos din agent. Den talar stdio: inget att starta, ingen port.
3. Välj vid behov vad agenten får göra i tabellen `[mcp]` i `config.toml`
   (nedan). Standardvärdena låter den läsa repositoryt (ändringar avvisas tills
   du sätter `allow_mutations = true`), visa saker för dig först när du
   bekräftar, och aldrig förifylla dialoger.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` skriver den i stället till repositoryts `.mcp.json`, så att
alla som öppnar projektet får den. Samma fil för hand, med de valfria
tilläggen:

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

- `--ui-control auto|ask|off` åsidosätter `[mcp] ui_control` bara för den här agenten.
- `GITPOULPI_CONFIG_DIR` ger agenten sin egen `config.toml` (egna gränser i
  `[mcp]`, ingen åtkomst till dina inställningar). Utelämna den för att dela
  din.
- `gitpoulpi --repo <path> mcp serve --stdio` låser servern till ett enda
  repository; annars verkar ett verktyg på repositoryt kring serverns
  arbetskatalog, eller på anropets argument `repo`.

**Andra klienter**: alla MCP-klienter som startar en stdio-server.

| Klient | Var | Vad |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | klientens MCP-JSON-fil | blocket `mcpServers` ovan (VS Code kallar nyckeln `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Allt annat | dess dokumentation | kommandot `gitpoulpi-mcp` (fristående binär, samma server, inga argument behövs) |

**Över HTTP** (klienter som ansluter till en URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

Tillståndslös JSON, samma verktyg, samma `[mcp]`-spärrar. Varje begäran kräver
bearer-token (`[mcp] http_auth`, på som standard): skriv ut den med `gitpoulpi mcp token
--show` (eller `--copy`, `--rotate`; token ligger i en fil som bara användaren kan läsa under den
privata datamappen) och ge den till klienten som en header:

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

En saknad eller felaktig token ger `401` med `WWW-Authenticate: Bearer`. En roterad
token gäller direkt, utan omstart av servern. Att stänga av `http_auth` är till för en
loopback-endpoint där du litar på varje lokalt konto; servern varnar då på
stderr.

### Att nå endpointen från en annan dator

Den säkraste uppsättningen behöver ingen fjärrlyssnare alls: behåll loopback-adressen och
vidarebefordra porten över SSH från datorn som kör agenten, `ssh -L
8808:127.0.0.1:8808 host`. Då är `--allow-remote` inte inblandat.

`--allow-remote` (en adress som inte är loopback) nekas om inte autentisering är på
med en token på plats **och** trafiken är krypterad. Nekandet namnger det som
saknas. Det finns två sätt att vara krypterad:

- **TLS i servern** (`[mcp] tls = "self_signed"` eller `"files"`). Servern
  avslutar TLS själv: den håller nyckeln, kör handskakningen, och bara en slutförd
  handskakning når HTTP-lagret; en klient med vanlig HTTP misslyckas i handskakningen och ingen
  vanlig lyssnare finns bredvid. Så krypteringen är ett faktum som servern
  fastställer vid varje anslutning, inte ett antagande. Med `self_signed` skapar
  den första starten ett par under den privata datamappen och skriver ut dess
  SHA-256-fingeravtryck (finns också i Inställningar); en klient litar på servern genom att fästa
  det fingeravtrycket eller certifikatfilen. Med `files` anger du `tls_cert` och
  `tls_key` till dina PEM-filer och klienter verifierar certifikatet via dess CA.
  Klienter autentiserar sig med bearer-token; klientcertifikat används inte.
- **En tunnel eller omvänd proxy** (`[mcp] behind_proxy = true`). En tunnelagent eller
  proxy ansluter som vilken lokal klient eller LAN-klient som helst och dess headers kan förfalskas, så
  servern kan inte inifrån avgöra om en finns. Därför är det ett
  uttryckligt medgivande (visas med en röd varning i Inställningar) i stället för en
  upptäckt. Kör tunneln på samma dator och behåll loopback-adressen när
  du kan. Med en adress som inte är loopback bakom en proxy är sträckan mellan proxyn
  och servern klartext i det nätverket.

**Utan MCP** använder en agent med ett skal CLI:t: `gitpoulpi --json <command>`
(se [CLI-referensen](cli-reference.md)). Ge den den färdiga skillen
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), som säger
när och hur man använder GitPoulpi i stället för rå `git`.

## Behörigheter

| Strömbrytare | Standard | Effekt |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` vägrar starta (Inställningar > Agenter & MCP > *MCP-server*). Läses när servern startar: en server som körs behåller sina inställningar. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Adress som `--http` använder när ingen anges (bara loopback om inte `--allow-remote`, som kräver autentisering och kryptering, se ovan). Inställningar > Agenter & MCP > *MCP-serverns port* ändrar porten och behåller värden. |
| `[mcp] http_auth` | `true` | `--http` kräver `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Inställningar > Agenter & MCP > *HTTP-autentisering*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` eller `files` (dina PEM-sökvägar): servern avslutar TLS själv. Inställningar > Agenter & MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Du går i god för att en tunnel eller omvänd proxy krypterar trafiken, vilket låter `--allow-remote` köra utan TLS. |
| `[mcp] allow_mutations` | `false` | Av: varje verktyg som ändrar något avvisas; läsning, `ui_*` och förslag fungerar fortfarande. `true` låter agenten committa, pusha, reseta... |
| `[mcp] mutation_allowlist` | `[]` | Inte tom: bara dessa verktyg får ändra något, t.ex. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: agenten kan inte flytta ditt fönster (`ui_state` / `ui_viewers` svarar fortfarande). |
| `[mcp] ui_control` | `"ask"` | `ask`: en chip i ditt fönster som du bekräftar; `auto`: tillämpas om du inte är upptagen; `off`: avvisas. Per agent: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Låter agenten öppna förifyllda dialoger som du bekräftar (Inställningar > Agenter & MCP > *Låt AI-agenter föreslå ändringar som jag bekräftar*). |
| `[mcp] allow_sensitive_staging` | `false` | Bara i `config.toml`. Av: verktyget `stage` erbjuder inte `allow_sensitive`, och en stage som skulle lägga till en känslig fil (en privat nyckel, en `.env`, en igenkänd token) nekas med `{error: "sensitive_files", files}`: agenten ska fråga dig. `true` låter den skicka `allow_sensitive`. Se [Stage och commit](staging-and-committing.md#känsliga-filer). |
| `[mcp] allow_history_purge` | `false` | Bara i `config.toml`. `true` listar verktyget `rewrite_purge`, som raderar den gamla historiken från en historikomskrivning (och ångra-posten för den) för gott. Av: agenten ser det inte. Det kräver fortfarande `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | Ditt fönsters huvudbrytare (Inställningar > Agenter & MCP > *Låt AI-agenter styra vyn*). Av: ingen agent når ditt fönster. |
| `[ui] launch` | `"off"` | Om en agent får öppna ett GitPoulpi-fönster när inget är öppet (se [nedan](#när-inget-fönster-är-öppet-eller-det-finns-i-en-annan-session)). |

Vad det här betyder i praktiken:

- **Som standard är agenten skrivskyddad**: den kan läsa, visa saker för dig i
  ditt fönster (`ui_*`) och, om du tillåter det, föreslå ändringar som du
  bekräftar. För att låta den ändra repositoryt (committa, pusha, reseta...)
  sätt `allow_mutations = true`; för ett smalt tillstånd behåll `true` med en
  `mutation_allowlist`. Varje verktyg bär MCP-annotationer (`readOnlyHint`,
  `destructiveHint`, `openWorldHint`), så en klient som Claude Code frågar
  fortfarande dig före de riskabla.
- **Konflikter**: `conflicts_check` (en förutsägelse, före någon merge), `conflicts_list`, `conflicts_show` och `conflicts_state` är
  läsningar; `conflicts_resolve` (den kastar den sida som inte behålls,
  `--delete` tar bort filen) och `conflicts_abort` (släpper hela
  merge/rebase/cherry-pick) är destruktiva; `conflicts_unresolve` är en vanlig
  skrivning. Ett avvisat anrop (`stale`, `not_conflicted`, `pick_incomplete`...)
  svarar `{"refused": {...}}` med slutkod 2 i stället för att misslyckas.
- **Konversationer för ärenden och pull requests**: `hosting_issue_comments` och
  `hosting_review_conversation` läser hela tidslinjen (läsningar, öppen värld: de
  anropar värden); `hosting_markdown` rensar en text och hittar dess referenser och
  anropar inget (stängd värld). Att publicera (`hosting_issue_comment`,
  `hosting_review_comment`, `hosting_review_reply`) och redigera
  (`hosting_*_comment_edit`) är vanliga mutationer markerade som öppen värld: de går
  direkt till värden, som den identitet token har (`posted_as` i svaret; en
  bot-token publicerar som boten), så `allow_mutations` och `mutation_allowlist`
  avgör om en agent får. Att ta bort en kommentar (`hosting_*_comment_delete`) är
  destruktivt. Efter en publicering, redigering eller borttagning underrättas daemonen,
  och ett öppet fönster för repositoryt uppdaterar objektet. `ui_show_pr` / `ui_show_issue`
  tar `--view conversation` för att öppna konversationen i stället för raden.
- **Fastnade lägen**: `conflicts_am_continue` är en skrivning,
  `conflicts_am_skip`, `conflicts_am_abort` och `index_lock_remove` är
  destruktiva, `index_lock_show` är en läsning.
- **Historikomskrivning**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (storleksanalysen), `rewrite_scan_secrets` (fynd med en maskerad förhandsvisning, aldrig ett värde),
  `rewrite_script_check`, `rewrite_script_test` och `rewrite_discard` är
  läsningar (en förhandsgranskning ändrar ingenting; den behåller bara en plan i
  daemonen och på disk i git-katalogen, så `rewrite_apply` fungerar också efter
  en omstart). `rewrite_apply` och `rewrite_run` är destruktiva: de flyttar många
  refs till omskrivna commits som en enda ångra-post (`undo` sätter tillbaka dem)
  och pushar aldrig. Ange receptet som `recipe_json` (eller `recipe`, en
  sökväg) och hemligheter som `secrets_file`, aldrig direkt i anropet.
  Shell-, Python- och externa Lua-skript (`script_language`) körs utan sandlåda
  och **en agent kan inte lita på dem**: det finns inget verktyg
  `rewrite_script_trust` och inget argument `trust`, så ett sådant skript körs
  först efter att en människa har kört `gitpoulpi rewrite script trust FIL` på
  just den texten (ett ändrat skript vägras igen, `script_not_trusted` med
  hashen i `refused.script`), och bara när `rewrite.external_scripts` slår på
  dess språk (`scripts_disabled`). Ett
  avslag svarar `{"refused": {"reason", "message", "detail"}}` och en plan som
  flyttar pushade refs svarar `{"needs_confirmation": ...}` (slutkod 3) tills
  anropet sätter `yes` - visa den för användaren först.
- **Aldrig nåbart för en agent**: att lagra en token (`hosting login`), att
  skapa och ladda upp en SSH-nyckel, `config set` (den kunde skriva om sina egna
  gränser) och daemonens livscykel. En token visas aldrig för agenten.
- **Kontoövergripande listor**: `hosting_radar` är en läsning (begäranden som väntar på din
  granskning, öppna begäranden, tilldelade ärenden i alla repositories för en profil,
  från en femminuterscache om inte `refresh` anges); `hosting_integrations_radar`
  ändrar en profils tabell `[integrations.radar]`, så `allow_mutations` spärrar det
  (även när det anropas utan alternativ, vilket bara skriver ut inställningarna).
- **Granskningskontext**: `ai_review_context` visar vad ett kodgranskningsutkast får
  förutom diffen (CLAUDE.md, AGENTS.md, en extra fil, var en lokal agent körs). Med
  ett inställningsargument sparar det ett standardvärde (globalt, eller `repo`:ts eller `project`:s egna),
  så `allow_mutations` spärrar det, som de andra verktygen som ändrar en inställning; utan något
  skrivs bara värdena ut. Som varje `ai_*`-verktyg listas det bara medan `ai.enabled` är på.
  `hosting_review_suggest` tar samma argument, men bara för en körning.
- **Spärrarna i `[mcp]` gäller MCP-servern.** CLI:t är ditt eget skal och
  spärras inte av `allow_mutations`; bara `ui_control` och `allow_ui_propose`
  gäller också för `gitpoulpi ui ...`.
- Ett avvisat anrop är ett vanligt verktygsfel (`isError: true`, orsaken i
  `structuredContent.error`), aldrig en tyst no-op.

```toml
[mcp]
allow_mutations = false        # true: agenten får ändra repositoryt
mutation_allowlist = []        # t.ex. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" låter agenten flytta din vy utan att fråga
allow_ui_propose = false       # true: agenten får förifylla dialoger för reword / split / commit / gren / tag
allow_sensitive_staging = false  # true: agenten får stagea filer som stagingskyddet flaggar
allow_history_purge = false    # true: agenten ser rewrite_purge (raderar gammal historik för gott)
```

## Verktygen

Ett verktyg per CLI-kommando, uppkallat efter dess sökväg med `_` (`branch create`
är `branch_create`). Varje tar kommandots argument plus `repo` (repositoryt;
standard: serverns) och `session`. Ett lyckat anrops `structuredContent` är
exakt det `gitpoulpi --json <command>` skriver ut. **Slag**: *read* ändrar aldrig
något; *write* och *destructive* ändrar något (spärrade av `allow_mutations`); *network* pratar med
en remote eller en värd; *view* flyttar bara ditt fönster; *propose* öppnar en
dialog som du bekräftar.

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
| `hosting_assignment` | `gitpoulpi hosting assignment` | write |
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
| `worktree_changes` | `gitpoulpi worktree changes` | read |
| `worktree_list` | `gitpoulpi worktree list` | read |
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

### Grenarbetsflödets åtgärder är plan-och-kör

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` och `flow_set_parent`
([Grenarbetsflöden](branching-workflows.md#åtgärder)) körs aldrig på ett nakent
anrop. `dry_run: true` returnerar planen (varje git-steg och kommando, varningar,
vad Ångra inte kan ta tillbaka, hinder) som ett vanligt resultat; `yes: true` kör den;
med ingen av dem vägras anropet och bär planen. Arbetsflödesfilens
`[hooks]` är skalkommandon från en fil som teamet delar, så de körs bara med
ett uttryckligt `run_hooks: true`: planen listar dem (`hooks`, `hooks_run`) så att du
kan visa användaren de exakta kommandona först. En merge som stannar på konflikter
svarar `status: "conflicted"` (ett felresultat, slutkod 2); när
konfliktverktygen löst filerna, anropa samma verktyg med `continue_: true`
eller `abort: true`. Att öppna en merge request (`finish` i läget `merge-request`)
kan inte ångras och planen säger det.

### Det en agent gör med vanlig `git` finns också i ångrahistoriken

En agent som skapar grenar och worktrees, committar, gör reset eller droppar
stashar med sin egen `git` (inte via de här verktygen) syns ändå: daemonen
jämför repositoryts refs med det den senast såg och registrerar varje ändring
som en post i `undo --list` med `"source": "outside"`, gits orsak
(`outside.message`) och den committer git registrerade. Verktygen `undo`, `redo`
och `undo_restore` tar dessa poster som vilka andra som helst, så en agent (eller
användaren, med `Ctrl+Z`) kan få tillbaka en gren som togs bort. Ändringar av
filer fångas aldrig. Se [Ångra och gör om](undo-redo.md#ändringar-gjorda-utanför-gitpoulpi).

## Att visa saker för användaren

En agent kan peka på något i stället för att beskriva det: `ui_show_commit`,
`ui_show_diff` (en fil på en rad, inline eller split), `ui_show_file` (innehåll,
blame eller historik på en rad), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`,
`ui_show_conflict` (merge-redigeraren på en fil med konflikt, vid en konflikt),
`ui_show_wip` (`composer` sätter markören i commit-sammanfattningen, utan att
skriva något), `ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette`
(frågan skriven, aldrig körd), `ui_show_keys`, `ui_show_settings` (en sida och
en inställning, aldrig ändrad), `ui_highlight` (färga rader i den öppna diffen
eller filen), `ui_notify` (en toast) och `ui_back` (ångra agentens senaste
vyändring). `ui_state` läser vad ett fönster visar (`text: true` lägger till den
synliga texten); `ui_viewers` listar fönstren.

De här verktygen **ändrar aldrig repositoryt** och kan inte skriva, klicka eller
ändra en inställning. Revisioner och sökvägar kontrolleras innan något når
fönstret.

**Vad användaren ser och bestämmer**:

- `ask` (standard), eller så fort du skriver, en dialog eller Inställningar är
  öppna, du drar, eller du tryckte på en tangent under de senaste 1,5 s: agenten
  får `deferred` direkt och din statusrad visar en chip ”claude-code vill visa
  ...”. `Ctrl+N Enter` eller ett klick visar det; `Esc` eller `Ctrl+N
  Backspace` avfärdar det. Det löper ut efter 60 s.
- Tillämpat: vyn flyttas, en toast på 4 s säger vem som flyttade den, och fliken
  får en lila `✦` till din nästa tangent. `Ctrl+N Backspace` (paletten `Agent:
  Tillbaka`) går tillbaka; fönstret minns tio vyer.
- Inget blir någonsin synligt igen eller utcheckat: en commit som ditt solofilter
  döljer svarar `partial` (`hidden_by_filter`); en PR- eller ärendelista som inte
  är länkad till en leverantör svarar `hosting_not_linked`.

**Svar**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, med fönstrets `state` när det
tillämpats. CLI-slutkoder: 0 applied/partial/proposed, 3 deferred, 1 annars.
Anropen är hastighetsbegränsade (burst på 5, sedan ett per sekund).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Att föreslå en ändring (`ui_propose_*`)

Med `allow_ui_propose = true` kan en agent öppna en av dina dialoger
**förifylld**, märkt *✦ Föreslaget av <agent>*. Inget händer förrän du trycker
på den dialogens egen bekräftelseknapp; du kan ändra först eller avbryta.

| Verktyg | Öppnar | Du bekräftar med |
|---|---|---|
| `ui_propose_reword` (rev, message) | popupen Reword | *Uppdatera meddelande* (en pushad commit frågar fortfarande om force-push) |
| `ui_propose_commit_message` (summary, body) | commit-formuläret, ifyllt; committar aldrig | knappen Commit |
| `ui_propose_branch` (name, start point) | den inbäddade rutan Ny gren | `Enter` |
| `ui_propose_tag` (name, target, message) | den inbäddade rutan Ny tag (annoterad när det finns ett meddelande) | `Enter` |
| `ui_propose_rewrite` (receptfil) | omskrivningsvyn med receptets regler i; ingenting förhandsgranskas eller tillämpas | din egen *Förhandsgranska* och sedan *Tillämpa* (`Ctrl+Enter`); `Esc` avvisar. Ett recept med ett skript avvisas, hemliga värden tas bort |
| `ui_propose_split` (rev, plan file) | uppdelningsredigeraren över commiten, ifylld från planen | *Dela upp* (`Ctrl+S`); `Esc` avvisar |
| `ui_propose_resolution` (path, pick or content file) | merge-redigeraren på en fil med konflikt, med valen (eller hela texten) redan i Resultatet | *Spara och markera löst* (`Ctrl+S`); att stänga den avvisar |

Ett förslag i taget; ett förslag på commit-meddelande avvisas medan du har ett
eget utkast i composern, och ett lösningsförslag medan merge-redigeraren håller
osparat arbete av dig eller när filens staging ändrats sedan agenten läste dem.
En lösning du ändrar före bekräftelsen rapporteras som `accepted_edited`.
Svaret har ett `proposal`-id; vad du bestämde finns i `ui_state`s `proposals`
(`offered`, `open`, `accepted`, `accepted_edited`, `rejected`, `expired`,
`failed`).

### När inget fönster är öppet, eller det finns i en annan session

- Ett fönster tillhör sessionen det startades i. Om agentens session inte har
  något letar `ui show` efter **ett** fönster i en annan session som har
  repositoryt öppet och använder det (`routed_session` i svaret). Matchar flera:
  `no_viewer` som namnger dem; ange `--session`.
- **Att öppna ett fönster åt agenten** är av som standard. Sätt `[ui] launch`
  (Inställningar > Agenter & MCP > *Öppna ett fönster för agenter*) till en
  förinställning för terminal eller multiplexer eller ett `custom`-kommando; då
  öppnar ett anrop med `launch: true` (CLI `gitpoulpi ui --launch show ...`) ett
  och väntar upp till 10 s. Med `ui_control = "auto"` öppnas det också utan
  flaggan; med `ask` får agenten `no_viewer` och bör berätta det för dig.

| Förinställning | Kommando |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (din standardterminal) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (nytt fönster), `zellij` (ny ruta), `cmux` (ny arbetsyta) | inuti en körande multiplexer, i repositoryts mapp |
| `custom` | `[ui] launch_command`, t.ex. `my-term --cd {repo} {gitpoulpi} --session {session}`; delas upp i argument, inget skal |

Om en förinställning inte fungerar med din version av programmet, använd `custom`.

## Fel

- Ett felformat anrop (okänt verktyg eller argument, ett obligatoriskt som saknas)
  är ett JSON-RPC-fel `-32602 invalid_params`.
- Ett avvisande, eller ett fel från git eller daemonen (ogiltig revision,
  konflikt, ...), är ett verktygsfel: `isError: true` med meddelandet i `content`
  och `structuredContent.error`.
