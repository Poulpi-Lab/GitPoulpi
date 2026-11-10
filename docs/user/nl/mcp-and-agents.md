[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · **Nederlands** · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP en agents

Een AI-agent (Claude Code, Cursor, Codex, ...) kan GitPoulpi aansturen via zijn
**MCP-server** of zijn **CLI**. Beide lopen via dezelfde achtergronddaemon als het
venster, dus wat de agent doet verschijnt live in je open GitPoulpi, en beide voeren voor
elke opdracht exact dezelfde code uit.

Deze pagina gaat over *externe* agents. De eigen AI-knoppen van GitPoulpi (commit-berichten,
uitleg, de commit-composer) worden apart ingesteld: zie [AI-ondersteuning](ai.md).

## Snelle installatie

1. Zet `gitpoulpi` en `gitpoulpi-mcp` op je `PATH` ([Installatie](installation.md)).
2. Registreer de server bij je agent. Hij spreekt stdio: niets om te starten, geen poort.
3. Kies optioneel wat de agent mag doen in de tabel `[mcp]` van `config.toml` (hieronder).
   De standaardwaarden laten hem de repository lezen (wijzigingen worden geweigerd tot je
   `allow_mutations = true` instelt), je dingen tonen pas nadat je bevestigt, en nooit
   dialoogvensters vooraf invullen.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` schrijft het in plaats daarvan naar de `.mcp.json` van de repository, zodat
iedereen die het project opent het krijgt. Hetzelfde bestand met de hand, met de optionele
extra's:

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

- `--ui-control auto|ask|off` overschrijft `[mcp] ui_control` alleen voor deze agent.
- `GITPOULPI_CONFIG_DIR` geeft de agent een eigen `config.toml` (zijn eigen limieten in
  `[mcp]`, geen toegang tot jouw instellingen). Laat het weg om de jouwe te delen.
- `gitpoulpi --repo <pad> mcp serve --stdio` zet de server vast op één repository; anders
  werkt een tool op de repository rond de werkmap van de server, of het argument `repo` van
  de aanroep.

**Andere clients**: elke MCP-client die een stdio-server start.

| Client | Waar | Wat |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | het MCP-JSON-bestand van de client | het blok `mcpServers` hierboven (VS Code noemt de sleutel `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Al het andere | zijn documentatie | de opdracht `gitpoulpi-mcp` (losse binary, dezelfde server, geen argumenten nodig) |

**Via HTTP** (clients die verbinden met een URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

Stateless JSON, dezelfde tools, dezelfde `[mcp]`-afschermingen. Elk verzoek heeft het bearer-token
nodig (`[mcp] http_auth`, standaard aan): druk het af met `gitpoulpi mcp token
--show` (of `--copy`, `--rotate`; het token staat in een bestand dat alleen jij kunt lezen in de
privégegevensmap) en geef het als header aan de client:

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

Een ontbrekend of onjuist token krijgt `401` met `WWW-Authenticate: Bearer`. Een vernieuwd
token geldt meteen, zonder herstart van de server. `http_auth` uitzetten is bedoeld voor een
loopback-eindpunt waar je elk lokaal account vertrouwt; de server waarschuwt dan op
stderr.

### Het eindpunt vanaf een andere machine bereiken

De veiligste opzet heeft helemaal geen remote listener nodig: houd het loopbackadres en
stuur de poort via SSH door vanaf de machine waarop de agent draait, `ssh -L
8808:127.0.0.1:8808 host`. Dan komt `--allow-remote` er niet aan te pas.

`--allow-remote` (een adres dat geen loopback is) wordt geweigerd tenzij authenticatie aan staat
met een token op zijn plaats **en** het verkeer is versleuteld. De weigering noemt wat
ontbreekt. Er zijn twee manieren om versleuteld te zijn:

- **TLS in de server** (`[mcp] tls = "self_signed"` of `"files"`). De server
  beëindigt TLS zelf: hij heeft de sleutel, doet de handshake, en alleen een voltooide
  handshake bereikt de HTTP-laag; een client met gewone HTTP mislukt in de handshake en er bestaat
  geen gewone listener ernaast. De versleuteling is dus een feit dat de server bij elke
  verbinding vaststelt, geen aanname. Met `self_signed` maakt de
  eerste start een paar onder de privégegevensmap en drukt de SHA-256-vingerafdruk af
  (ook in de Instellingen); een client vertrouwt de server door die vingerafdruk of het
  certificaatbestand te pinnen. Met `files` stel je `tls_cert` en
  `tls_key` in op je PEM-bestanden en clients verifiëren het certificaat via zijn CA.
  Clients authenticeren met het bearer-token; clientcertificaten worden niet gebruikt.
- **Een tunnel of reverse proxy** (`[mcp] behind_proxy = true`). Een tunnelagent of
  proxy verbindt zoals elke lokale of LAN-client en zijn headers kunnen worden vervalst, dus
  de server kan van binnenuit niet zien of er een is. Daarom is het een
  expliciete bevestiging (met een rode waarschuwing in de Instellingen) in plaats van een
  detectie. Laat de tunnel op dezelfde machine draaien en houd het loopbackadres als je
  kunt. Bij een adres dat geen loopback is achter een proxy, is de schakel tussen de proxy
  en de server platte tekst op dat netwerk.

**Zonder MCP** gebruikt een agent met een shell de CLI: `gitpoulpi --json <opdracht>` (zie de
[CLI-naslag](cli-reference.md)). Geef hem de kant-en-klare skill
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), die zegt wanneer en hoe je
GitPoulpi in plaats van kale `git` gebruikt.

## Rechten

| Schakelaar | Standaard | Effect |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` weigeren te starten (Instellingen > Agents & MCP > *MCP-server*). Gelezen bij het starten van de server: een draaiende server houdt zijn instellingen. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Adres dat `--http` gebruikt als er geen is opgegeven (alleen loopback tenzij `--allow-remote`, dat authenticatie en versleuteling vereist, zie hierboven). Instellingen > Agents & MCP > *Poort van MCP-server* wijzigt de poort en behoudt de host. |
| `[mcp] http_auth` | `true` | `--http` heeft `Authorization: Bearer <token>` nodig (`gitpoulpi mcp token`). Instellingen > Agents & MCP > *HTTP-authenticatie*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` of `files` (je PEM-paden): de server beëindigt TLS zelf. Instellingen > Agents & MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Je staat ervoor in dat een tunnel of reverse proxy het verkeer versleutelt, waardoor `--allow-remote` zonder TLS kan draaien. |
| `[mcp] allow_mutations` | `false` | Uit: elke tool die iets wijzigt wordt geweigerd; lezen, `ui_*` en voorstellen werken nog. `true` laat de agent committen, pushen, resetten... |
| `[mcp] mutation_allowlist` | `[]` | Niet leeg: alleen deze tools mogen iets wijzigen, bijv. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: de agent kan je venster niet verplaatsen (`ui_state` / `ui_viewers` antwoorden nog). |
| `[mcp] ui_control` | `"ask"` | `ask`: een chip in je venster die je bevestigt; `auto`: toegepast tenzij je bezig bent; `off`: geweigerd. Per agent: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Laat de agent vooraf ingevulde dialoogvensters openen die jij bevestigt (Instellingen > Agents & MCP > *AI-agents wijzigingen laten voorstellen die ik bevestig*). |
| `[mcp] allow_sensitive_staging` | `false` | Alleen in `config.toml`. Uit biedt de tool `stage` `allow_sensitive` niet aan, en een stage die een gevoelig bestand zou toevoegen (een privésleutel, een `.env`, een herkend token) wordt geweigerd met `{error: "sensitive_files", files}`: de agent moet het jou vragen. `true` laat hem `allow_sensitive` meegeven. Zie [Stagen en committen](staging-and-committing.md#gevoelige-bestanden). |
| `[mcp] allow_history_purge` | `false` | Alleen in `config.toml`. `true` maakt de tool `rewrite_purge` zichtbaar, die de oude geschiedenis van een herschrijving (en het ongedaan maken ervan) voorgoed verwijdert. Uit: de agent ziet hem niet. Hij heeft nog steeds `allow_mutations` nodig. |
| `[ui] allow_agent_view_control` | `true` | De hoofdschakelaar van je venster (Instellingen > Agents & MCP > *AI-agents de weergave laten besturen*). Uit: geen enkele agent bereikt je venster. |
| `[ui] launch` | `"off"` | Of een agent een GitPoulpi-venster mag openen als er geen open is (zie [hieronder](#wanneer-er-geen-venster-open-is-of-het-in-een-andere-sessie-staat)). |

Wat dit in de praktijk betekent:

- **Standaard is de agent alleen-lezen**: hij kan lezen, je dingen tonen in je venster
  (`ui_*`) en, als je het toestaat, wijzigingen voorstellen die jij bevestigt. Om hem de
  repository te laten wijzigen (committen, pushen, resetten...) stel je `allow_mutations =
  true` in; voor een smalle toestemming houd je het `true` met een `mutation_allowlist`. Elke
  tool draagt MCP-annotaties (`readOnlyHint`, `destructiveHint`, `openWorldHint`), zodat een
  client als Claude Code je nog steeds vraagt vóór de riskante.
- **Conflicten**: `conflicts_check` (een voorspelling, vóór een merge), `conflicts_list`, `conflicts_show` en `conflicts_state` zijn lezen;
  `conflicts_resolve` (het verwerpt de niet gekozen zijde, `--delete` verwijdert het
  bestand) en `conflicts_abort` (laat de hele merge/rebase/cherry-pick vallen) zijn
  destructief; `conflicts_unresolve` is een gewone schrijfactie. Een geweigerde aanroep
  (`stale`, `not_conflicted`, `pick_incomplete`...) antwoordt `{"refused": {...}}` met
  afsluitcode 2 in plaats van te falen.
- **Vastgelopen staten**: `conflicts_am_continue` is een schrijfactie, `conflicts_am_skip`,
  `conflicts_am_abort` en `index_lock_remove` zijn destructief, `index_lock_show` is lezen.
- **Geschiedenis herschrijven**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (de groottenanalyse), `rewrite_scan_secrets` (treffers met een gemaskeerd voorbeeld, nooit een waarde),
  `rewrite_script_check`, `rewrite_script_test` en `rewrite_discard` zijn lezen (een
  voorbeeld wijzigt niets; het bewaart alleen een plan in de daemon en op schijf in de
  git-map, dus `rewrite_apply` werkt ook na een herstart). `rewrite_apply` en
  `rewrite_run` zijn destructief: ze verplaatsen veel refs naar herschreven commits als
  één ongedaanmaakstap (`undo` zet ze terug) en pushen nooit. Geef het recept mee als
  `recipe_json` (of `recipe`, een pad) en geheimen als `secrets_file`, nooit inline.
  Scripts in shell, Python en externe Lua (`script_language`) draaien niet in een sandbox
  en **een agent kan ze niet vertrouwen**: er is geen tool `rewrite_script_trust` en
  geen argument `trust`, dus zo'n script draait alleen nadat een persoon
  `gitpoulpi rewrite script trust FILE` op precies die tekst heeft uitgevoerd (een gewijzigd script wordt
  opnieuw geweigerd, `script_not_trusted` met de hash in `refused.script`), en
  alleen als `rewrite.external_scripts` zijn taal aanzet
  (`scripts_disabled`).
  Een weigering antwoordt `{"refused": {"reason", "message", "detail"}}` en een plan dat
  gepushte refs verplaatst antwoordt `{"needs_confirmation": ...}` (afsluitcode 3), tot
  de aanroep `yes` zet - toon het eerst aan de gebruiker.
- **Nooit bereikbaar voor een agent**: een token opslaan (`hosting login`), een SSH-sleutel
  maken en uploaden, `config set` (hij zou zijn eigen limieten kunnen herschrijven) en de
  levenscyclus van de daemon. Een token wordt de agent nooit getoond.
- **Accountbrede lijsten**: `hosting_radar` is een lezing (verzoeken die op jouw review wachten, openstaande
  verzoeken, toegewezen issues over elke repository van een profiel, uit een cache van vijf minuten
  tenzij `refresh`); `hosting_integrations_radar` wijzigt de tabel `[integrations.radar]` van een profiel, dus
  `allow_mutations` begrenst het (ook als het zonder optie wordt aangeroepen, wat de instellingen alleen afdrukt).
- **Reviewcontext**: `ai_review_context` toont wat een codereviewconcept krijgt naast het
  verschil (CLAUDE.md, AGENTS.md, een extra bestand, waar een lokale agent draait). Met een
  instelargument slaat het een standaard op (globaal, of de eigen waarde van de `repo` of het `project`),
  dus `allow_mutations` begrenst het, zoals de andere tools die een instelling wijzigen; zonder argument
  drukt het alleen af. Zoals elke `ai_*`-tool wordt het alleen getoond zolang `ai.enabled` aan staat.
  `hosting_review_suggest` neemt dezelfde argumenten, maar voor één run.
- **De afschermingen in `[mcp]` gelden voor de MCP-server.** De CLI is je eigen shell en wordt niet
  begrensd door `allow_mutations`; alleen `ui_control` en `allow_ui_propose` gelden ook voor
  `gitpoulpi ui ...`.
- Een geweigerde aanroep is een gewone toolfout (`isError: true`, de reden in
  `structuredContent.error`), nooit een stille no-op.

```toml
[mcp]
allow_mutations = false        # true: de agent mag de repository wijzigen
mutation_allowlist = []        # bijv. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" laat de agent je weergave verplaatsen zonder te vragen
allow_ui_propose = false       # true: de agent mag dialogen voor Reword / Split / commit / branch / tag vooraf invullen
allow_sensitive_staging = false  # true: de agent mag bestanden stagen die de stagebewaker markeert
allow_history_purge = false    # true: de agent ziet rewrite_purge (verwijdert oude geschiedenis voorgoed)
```

## De tools

Eén tool per CLI-opdracht, genoemd naar zijn pad met `_` (`branch create` is `branch_create`).
Elk neemt de argumenten van de opdracht plus `repo` (de repository; standaard die van de
server) en `session`. De `structuredContent` van een geslaagde aanroep is exact wat
`gitpoulpi --json <opdracht>` toont. **Soort**: *read* wijzigt nooit iets; *write* en
*destructive* zijn mutaties (begrensd); *network* praat met een remote of een host; *view*
verplaatst alleen je venster; *propose* opent een dialoogvenster dat jij bevestigt.

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
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
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
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

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
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
<!-- mcp-tools:end -->

### Branchingworkflowacties zijn plan-dan-uitvoeren

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` en `flow_set_parent`
([Branchingworkflows](branching-workflows.md#acties)) draaien nooit bij een kale
aanroep. `dry_run: true` geeft het plan (elke git-stap en -opdracht, waarschuwingen,
wat Ongedaan maken niet kan terugdraaien, blokkades) als een normaal resultaat terug; `yes: true` voert het uit;
met geen van beide wordt de aanroep geweigerd en draagt hij het plan. De `[hooks]` van het workflowbestand
zijn shellopdrachten uit een bestand dat het team deelt, dus ze draaien alleen met een
expliciete `run_hooks: true`: het plan somt ze op (`hooks`, `hooks_run`) zodat je
de gebruiker eerst de exacte opdrachten kunt tonen. Een merge die op conflicten stopt
antwoordt `status: "conflicted"` (een foutresultaat, afsluitcode 2); nadat de
conflicttools de bestanden hebben opgelost, roep je dezelfde tool aan met `continue_: true`
of `abort: true`. Een merge request openen (`finish` in de modus `merge-request`)
is niet ongedaan te maken en het plan zegt dat.

### Wat een agent met gewone `git` doet staat ook in de ongedaanmaakgeschiedenis

Een agent die met zijn eigen `git` (niet via deze tools) branches en worktrees
maakt, commit, reset of stashes dropt, blijft zichtbaar: de daemon vergelijkt
de refs van de repository met wat hij het laatst zag en legt elke wijziging
vast als een item van `undo --list` met `"source": "outside"`, de reden van git
(`outside.message`) en de committer die git vastlegde. De tools `undo`, `redo`
en `undo_restore` nemen deze items zoals elk ander, zodat een agent (of de
gebruiker, met `Ctrl+Z`) een verwijderde branch kan terugbrengen. Bewerkingen
van bestanden worden nooit vastgelegd. Zie
[Ongedaan maken en opnieuw](undo-redo.md#wijzigingen-buiten-gitpoulpi).

## Dingen aan de gebruiker tonen

Een agent kan naar iets wijzen in plaats van het te beschrijven: `ui_show_commit`,
`ui_show_diff` (een bestand op een regel, inline of split), `ui_show_file` (inhoud, blame of
geschiedenis op een regel), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`, `ui_show_conflict`
(de merge-editor op een conflicterend bestand, bij een conflict), `ui_show_wip` (`composer`
zet de cursor in de commit-samenvatting, zonder iets te typen), `ui_show_tab`, `ui_show_home`,
`ui_show_search`, `ui_show_palette` (de query getypt, nooit uitgevoerd), `ui_show_keys`,
`ui_show_settings` (een pagina en een instelling, nooit bewerkt), `ui_highlight` (regels van
de open diff of het open bestand inkleuren), `ui_notify` (een toast) en `ui_back` (de laatste
weergavewijziging van de agent ongedaan maken). `ui_state` leest wat een venster toont
(`text: true` voegt de zichtbare tekst toe); `ui_viewers` somt de vensters op.

Deze tools **wijzigen de repository nooit** en kunnen geen instelling typen, klikken of
bewerken. Revisies en paden worden gecontroleerd voordat iets het venster bereikt.

**Wat de gebruiker ziet en beslist**:

- `ask` (standaard), of wanneer je typt, een dialoogvenster of de Instellingen open zijn, je
  aan het slepen bent, of je in de laatste 1,5 s een toets hebt ingedrukt: de agent krijgt
  meteen `deferred` en je statusbalk toont een chip *"claude-code wil ... tonen"*.
  `Ctrl+N Enter` of een klik toont het; `Esc` of `Ctrl+N Backspace` wijst het af. Het verloopt
  na 60 s.
- Toegepast: het zicht verplaatst, een toast van 4 s zegt wie het verplaatste, en het
  tabblad krijgt een paarse `✦` tot je volgende toets. `Ctrl+N Backspace` (palet `Agent:
  Terug`) gaat terug; het venster onthoudt tien weergaven.
- Niets wordt ooit zichtbaar gemaakt of uitgecheckt: een commit die je solo-filter verbergt
  antwoordt `partial` (`hidden_by_filter`); een PR- of issuelijst die niet aan een provider is
  gekoppeld antwoordt `hosting_not_linked`.

**Antwoorden**: `{"outcome": "applied" | "partial" | "deferred" | "refused" | "not_found" |
"no_viewer" | "proposed", ...}`, met de `state` van het venster als het is toegepast.
CLI-afsluitcodes: 0 applied/partial/proposed, 3 deferred, 1 anders. Aanroepen zijn
rate-limited (burst van 5, daarna één per seconde).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Een wijziging voorstellen (`ui_propose_*`)

Met `allow_ui_propose = true` kan een agent een van je dialoogvensters **vooraf ingevuld**
openen, gemarkeerd met *✦ Voorgesteld door <agent>*. Er gebeurt niets tot je op de eigen
bevestigingsknop van dat dialoogvenster drukt; je kunt eerst bewerken of annuleren.

| Tool | Opent | Je bevestigt met |
|---|---|---|
| `ui_propose_reword` (rev, message) | de pop-up Reword | *Bericht bijwerken* (een gepushte commit vraagt nog naar force pushen) |
| `ui_propose_commit_message` (summary, body) | het commit-formulier, ingevuld; commit nooit | de knop Commit |
| `ui_propose_branch` (name, start point) | het inline vak Nieuwe branch | `Enter` |
| `ui_propose_tag` (name, target, message) | het inline vak Nieuwe tag (geannoteerd als er een bericht is) | `Enter` |
| `ui_propose_split` (rev, plan file) | de splitseditor over de commit, ingevuld vanuit het plan | *Splitsen* (`Ctrl+S`); `Esc` wijst af |
| `ui_propose_rewrite` (recipe file) | de werkbank voor het herschrijven van de geschiedenis met de regels van het recept erin; er wordt niets bekeken of toegepast | je eigen *Voorbeeld* en dan *Toepassen* (`Ctrl+Enter`); `Esc` wijst af. Een recept met een script wordt geweigerd, geheime waarden worden weggelaten |
| `ui_propose_resolution` (path, pick or content file) | de merge-editor op een conflicterend bestand, met de keuzes (of de hele tekst) al in de Uitvoer | *Opslaan & als opgelost markeren* (`Ctrl+S`); sluiten wijst af |

Eén voorstel tegelijk; een voorstel voor een commit-bericht wordt geweigerd zolang je een eigen
concept in de composer hebt, en een voorstel voor een oplossing zolang de merge-editor
niet-opgeslagen werk van jou bevat of als de stages van het bestand zijn gewijzigd sinds de
agent ze las. Een oplossing die je wijzigt vóór je bevestigt wordt gemeld als
`accepted_edited`. Het antwoord heeft een `proposal`-id; wat je besliste staat in de
`proposals` van `ui_state` (`offered`, `open`, `accepted`, `accepted_edited`, `rejected`,
`expired`, `failed`).

### Wanneer er geen venster open is, of het in een andere sessie staat

- Een venster hoort bij de sessie waarin het is gestart. Heeft de sessie van de agent er geen,
  dan zoekt `ui show` **één** venster in een andere sessie waarin de repository open is en
  gebruikt het (`routed_session` in het antwoord). Komen er meerdere overeen: `no_viewer` die
  ze noemt; geef `--session` mee.
- **Een venster voor de agent openen** staat standaard uit. Stel `[ui] launch` in (Instellingen
  > Agents & MCP > *Een venster voor agents openen*) op een voorinstelling voor een terminal of
  multiplexer of een `custom`-opdracht; dan opent een aanroep met `launch: true` (CLI
  `gitpoulpi ui --launch show ...`) er een en wacht tot 10 s. Met `ui_control = "auto"` wordt
  het ook zonder de vlag geopend; met `ask` krijgt de agent `no_viewer` en moet hij het je
  vertellen.

| Voorinstelling | Opdracht |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (je standaardterminal) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (nieuw venster), `zellij` (nieuw paneel), `cmux` (nieuwe workspace) | binnen een draaiende multiplexer, in de map van de repository |
| `custom` | `[ui] launch_command`, bijv. `my-term --cd {repo} {gitpoulpi} --session {session}`; in argumenten gesplitst, geen shell |

Werkt een voorinstelling niet met jouw versie van het programma, gebruik dan `custom`.

## Fouten

- Een ongeldige aanroep (onbekende tool of onbekend argument, een ontbrekend verplicht
  argument) is een JSON-RPC-fout `-32602 invalid_params`.
- Een weigering, of een fout van git of de daemon (ongeldige revisie, conflict, ...), is een
  toolfout: `isError: true` met de melding in `content` en `structuredContent.error`.
