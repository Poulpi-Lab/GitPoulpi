[English](../mcp-and-agents.md) · **Čeština** · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP a agenti

AI agent (Claude Code, Cursor, Codex, ...) může GitPoulpi ovládat přes jeho
**server MCP** nebo přes jeho **CLI**. Obojí prochází stejným daemonem na pozadí
jako okno, takže to, co agent dělá, se živě projevuje ve vašem otevřeném
GitPoulpi, a obojí spouští u každého příkazu přesně stejný kód.

Tato stránka je o *externích* agentech. Vlastní AI tlačítka GitPoulpi (zprávy
commitů, vysvětlení, commit composer) se konfigurují zvlášť: viz
[Asistence AI](ai.md).

## Rychlé nastavení

1. Dejte `gitpoulpi` a `gitpoulpi-mcp` na `PATH` ([Instalace](installation.md)).
2. Zaregistrujte server u svého agenta. Mluví přes stdio: není co spouštět, žádný port.
3. Volitelně zvolte, co agent smí, v tabulce `[mcp]` v `config.toml`
   (níže). Výchozí hodnoty mu dovolí číst repozitář (změny se odmítnou, dokud
   nenastavíte `allow_mutations = true`), ukazovat vám věci až po vašem
   potvrzení a nikdy nepředvyplňovat dialogy.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` ho místo toho zapíše do `.mcp.json` repozitáře, takže ho dostane
každý, kdo projekt otevře. Stejný soubor ručně, s volitelnými doplňky:

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

- `--ui-control auto|ask|off` přepíše `[mcp] ui_control` jen pro tohoto agenta.
- `GITPOULPI_CONFIG_DIR` dá agentovi vlastní `config.toml` (vlastní limity
  `[mcp]`, bez přístupu k vašemu nastavení). Vynechejte ho pro sdílení vašeho.
- `gitpoulpi --repo <path> mcp serve --stdio` připne server k jednomu repozitáři;
  jinak nástroj působí na repozitář kolem pracovního adresáře serveru, nebo na
  argument `repo` volání.

**Další klienti**: jakýkoli klient MCP, který spouští server přes stdio.

| Klient | Kde | Co |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | soubor JSON MCP klienta | blok `mcpServers` výše (VS Code pojmenovává klíč `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Cokoli jiného | jeho dokumentace | příkaz `gitpoulpi-mcp` (samostatný binární soubor, stejný server, žádné argumenty nejsou třeba) |

**Přes HTTP** (klienti, kteří se připojují k URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

Bezstavový JSON, stejné nástroje, stejné brány `[mcp]`. Každý požadavek potřebuje
bearer token (`[mcp] http_auth`, ve výchozím stavu zapnuto): vypište ho pomocí `gitpoulpi mcp token
--show` (nebo `--copy`, `--rotate`; token žije v souboru jen pro uživatele v
soukromé datové složce) a předejte ho klientovi jako hlavičku:

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

Chybějící nebo špatný token dostane `401` s `WWW-Authenticate: Bearer`. Obnovený
token platí okamžitě, bez restartu serveru. Vypnutí `http_auth` je určeno pro
endpoint na loopbacku, kde důvěřujete každému místnímu účtu; server pak varuje na
stderr.

### Přístup k endpointu z jiného počítače

Nejbezpečnější uspořádání nepotřebuje žádný vzdálený posluchač: ponechte adresu loopback a
přesměrujte port přes SSH z počítače, na kterém běží agent, `ssh -L
8808:127.0.0.1:8808 host`. Pak se `--allow-remote` vůbec nepoužije.

`--allow-remote` (adresa mimo loopback) se odmítne, pokud není zapnuto ověřování
s existujícím tokenem **a** provoz není šifrovaný. Odmítnutí pojmenuje, co
chybí. Šifrovat lze dvěma způsoby:

- **TLS v serveru** (`[mcp] tls = "self_signed"` nebo `"files"`). Server
  ukončuje TLS sám: drží klíč, provádí handshake a k vrstvě HTTP se dostane jen dokončený
  handshake; klient s prostým HTTP handshake nedokončí a vedle něj neexistuje
  žádný prostý posluchač. Šifrování je tedy skutečnost, kterou server
  zajistí při každém spojení, ne předpoklad. U `self_signed` první
  spuštění vygeneruje pár v soukromé datové složce a vypíše jeho otisk
  SHA-256 (také v Nastavení); klient serveru důvěřuje připnutím
  tohoto otisku nebo souboru certifikátu. U `files` nastavte `tls_cert` a
  `tls_key` na své soubory PEM a klienti ověřují certifikát podle jeho CA.
  Klienti se ověřují bearer tokenem; klientské certifikáty se nepoužívají.
- **Tunel nebo reverzní proxy** (`[mcp] behind_proxy = true`). Agent tunelu nebo
  proxy se připojuje jako jakýkoli místní klient nebo klient z LAN a jeho hlavičky lze zfalšovat, takže
  server zevnitř nepozná, zda tam jeden je. Proto jde o
  výslovné potvrzení (v Nastavení s červeným varováním) místo
  detekce. Tunel provozujte na stejném počítači a pokud můžete, ponechte adresu loopback.
  U adresy mimo loopback za proxy je úsek mezi proxy
  a serverem v té síti prostý text.

**Bez MCP** agent s shellem používá CLI: `gitpoulpi --json <command>`
(viz [Referenční příručka CLI](cli-reference.md)). Dejte mu hotový skill
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), který říká, kdy
a jak používat GitPoulpi místo holého `git`.

## Oprávnění

| Přepínač | Výchozí | Účinek |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` odmítnou startovat (Nastavení > Agenti a MCP > *Server MCP*). Čte se při startu serveru: běžící server si ponechá svá nastavení. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Adresa, kterou `--http` použije, když žádná není zadána (jen loopback, pokud není `--allow-remote`, které vyžaduje ověřování a šifrování, viz výše). Nastavení > Agenti a MCP > *Port serveru MCP* upraví jeho port a ponechá hostitele. |
| `[mcp] http_auth` | `true` | `--http` potřebuje `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Nastavení > Agenti a MCP > *Ověřování HTTP*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` nebo `files` (vaše cesty PEM): server ukončuje TLS sám. Nastavení > Agenti a MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Ručíte za to, že provoz šifruje tunel nebo reverzní proxy, což umožní běh `--allow-remote` bez TLS. |
| `[mcp] allow_mutations` | `false` | Vypnuto: každý nástroj, který něco mění, se odmítne; čtení, `ui_*` a návrhy dál fungují. `true` agentovi dovolí commitovat, pushovat, resetovat... |
| `[mcp] mutation_allowlist` | `[]` | Neprázdné: jen tyto nástroje smějí něco měnit, např. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: agent nemůže hýbat vaším oknem (`ui_state` / `ui_viewers` dál odpovídají). |
| `[mcp] ui_control` | `"ask"` | `ask`: čip ve vašem okně, který potvrdíte; `auto`: použije se, pokud nejste zaneprázdněni; `off`: odmítnuto. Pro jednoho agenta: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Dovolí agentovi otevřít předvyplněné dialogy, které potvrdíte (Nastavení > Agenti a MCP > *Nechat AI agenty navrhovat změny k mému potvrzení*). |
| `[mcp] allow_sensitive_staging` | `false` | Jen v `config.toml`. Vypnuto, nástroj `stage` nenabízí `allow_sensitive` a přidání do indexu, které by zahrnulo citlivý soubor (soukromý klíč, `.env`, rozpoznaný token), se odmítne s `{error: "sensitive_files", files}`: agent se má zeptat vás. `true` mu dovolí předat `allow_sensitive`. Viz [Stagování a commitování](staging-and-committing.md#citlivé-soubory). |
| `[mcp] allow_history_purge` | `false` | Jen v `config.toml`. `true` zpřístupní nástroj `rewrite_purge`, který smaže nadobro starou historii přepisu (i její vrácení zpět). Vypnuto: agent ho nevidí. Stále vyžaduje `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | Hlavní vypínač vašeho okna (Nastavení > Agenti a MCP > *Povolit AI agentům ovládat zobrazení*). Vypnuto: žádný agent se k vašemu oknu nedostane. |
| `[ui] launch` | `"off"` | Zda smí agent otevřít okno GitPoulpi, když žádné není otevřené (viz [níže](#když-není-otevřeno-žádné-okno-nebo-je-v-jiné-relaci)). |

Co to znamená v praxi:

- **Hned po instalaci je agent jen pro čtení**: může číst, ukazovat vám věci
  ve vašem okně (`ui_*`) a, pokud to dovolíte, navrhovat změny, které potvrdíte.
  Aby mohl měnit repozitář (commit, push, reset...), nastavte
  `allow_mutations = true`; pro úzké oprávnění ho nechte `true` s
  `mutation_allowlist`. Každý nástroj nese anotace MCP (`readOnlyHint`,
  `destructiveHint`, `openWorldHint`), takže klient jako Claude Code se vás před
  rizikovými stále zeptá.
- **Konflikty**: `conflicts_check` (předpověď před jakýmkoli merge), `conflicts_list`, `conflicts_show` a `conflicts_state` jsou
  čtení; `conflicts_resolve` (zahodí stranu, kterou neponecháte, `--delete`
  odstraní soubor) a `conflicts_abort` (zahodí celý merge/rebase/cherry-pick) jsou
  destruktivní; `conflicts_unresolve` je běžný zápis. Odmítnuté volání (`stale`,
  `not_conflicted`, `pick_incomplete`...) odpoví `{"refused": {...}}` s kódem
  ukončení 2 místo selhání.
- **Zaseknuté stavy**: `conflicts_am_continue` je zápis, `conflicts_am_skip`,
  `conflicts_am_abort` a `index_lock_remove` jsou destruktivní, `index_lock_show`
  je čtení.
- **Přepis historie**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (analýza velikosti), `rewrite_scan_secrets` (nálezy se zakrytým náhledem, nikdy hodnota),
  `rewrite_script_check`, `rewrite_script_test` a
  `rewrite_discard` jsou čtení (náhled nic nemění; jen uchová plán
  v daemonu a na disku v git adresáři, takže `rewrite_apply` funguje i po restartu).
  `rewrite_apply` a `rewrite_run` jsou destruktivní: přesunou mnoho refů na
  přepsané commity jako jeden záznam pro vrácení (`undo` je vrátí) a nikdy
  neodesílají. Sadu pravidel předejte jako `recipe_json` (nebo `recipe`, cesta)
  a tajné hodnoty jako `secrets_file`, nikdy přímo v textu.
  Skripty v shellu, Pythonu a externí Lua (`script_language`) nejsou v izolaci
  a **agent jim nemůže důvěřovat**: nástroj `rewrite_script_trust` ani argument
  `trust` neexistuje, takže takový skript běží, až když člověk nad přesně tímto
  textem spustil `gitpoulpi rewrite script trust FILE` (změněný skript se opět
  odmítne, `script_not_trusted` s hashem v `refused.script`), a jen když jeho
  jazyk zapíná `rewrite.external_scripts` (`scripts_disabled`).
  Odmítnutí odpoví
  `{"refused": {"reason", "message", "detail"}}` a plán, který přesouvá odeslané
  refy, odpoví `{"needs_confirmation": ...}` (kód ukončení 3), dokud volání
  nenastaví `yes` - nejdřív ho ukažte uživateli.
- **Pro agenta nikdy nedostupné**: uložení tokenu (`hosting login`), vytvoření
  a nahrání klíče SSH, `config set` (mohl by přepsat vlastní limity) a životní
  cyklus daemona. Token se agentovi nikdy neukáže.
- **Seznamy za celý účet**: `hosting_radar` je čtení (žádosti čekající na vaši
  kontrolu, otevřené žádosti, přiřazené issues napříč všemi repozitáři profilu,
  z pětiminutové mezipaměti, pokud není `refresh`); `hosting_integrations_radar`
  mění tabulku `[integrations.radar]` profilu, takže ho brání `allow_mutations`
  (i při volání bez voleb, které jen vypíše nastavení).
- **Kontext kontroly**: `ai_review_context` ukazuje, co koncept kontroly kódu dostane
  kromě diffu (CLAUDE.md, AGENTS.md, další soubor, kde běží místní agent). S argumentem
  nastavení uloží výchozí hodnotu (globální, nebo vlastní hodnotu `repo` či `project`),
  takže ho brání `allow_mutations`, jako ostatní nástroje, které mění nastavení; bez něj
  jen vypíše. Jako každý nástroj `ai_*` je uveden, jen dokud je zapnuto `ai.enabled`.
  `hosting_review_suggest` bere stejné argumenty jen pro jeden běh.
- **Brány `[mcp]` platí pro server MCP.** CLI je váš vlastní shell a
  `allow_mutations` ho neomezuje; na `gitpoulpi ui ...` se vztahují jen
  `ui_control` a `allow_ui_propose`.
- Odmítnuté volání je běžná chyba nástroje (`isError: true`, důvod v
  `structuredContent.error`), nikdy tichá prázdná operace.

```toml
[mcp]
allow_mutations = false        # true: the agent may change the repository
mutation_allowlist = []        # e.g. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" lets the agent move your view without asking
allow_ui_propose = false       # true: agent smí předvyplnit dialogy Reword / Split / commit / větev / tag
allow_sensitive_staging = false  # true: agent smí přidat do indexu soubory, které ochrana označí
allow_history_purge = false    # true: agent vidí rewrite_purge (smaže nadobro starou historii)
```

## Nástroje

Jeden nástroj na příkaz CLI, pojmenovaný podle jeho cesty s `_` (`branch create`
je `branch_create`). Každý bere argumenty příkazu plus `repo` (repozitář;
výchozí: ten serveru) a `session`. `structuredContent` úspěšného volání je
přesně to, co vytiskne `gitpoulpi --json <command>`. **Druh**: *čtení* nikdy nic
nemění; *zápis* a *destruktivní* jsou mutace (s bránou); *síť* komunikuje s
remote nebo hostitelem; *zobrazení* hýbe jen vaším oknem; *návrh* otevře dialog,
který potvrdíte.

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

### Akce workflow větvení jsou plán, a teprve pak provedení

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` a `flow_set_parent`
([Workflow větvení](branching-workflows.md#akce)) se nikdy nespustí holým
voláním. `dry_run: true` vrátí plán (každý krok a příkaz git, varování,
co Undo nevezme zpět, blokátory) jako běžný výsledek; `yes: true` ho provede;
bez obojího se volání odmítne a nese plán. `[hooks]` ze souboru workflow
jsou příkazy shellu ze souboru, který tým sdílí, takže se spustí jen s
výslovným `run_hooks: true`: plán je vypisuje (`hooks`, `hooks_run`), abyste
uživateli mohli nejdřív ukázat přesné příkazy. Merge, který se zastaví na konfliktech,
odpoví `status: "conflicted"` (výsledek chyby, kód ukončení 2); až nástroje pro konflikty
soubory vyřeší, zavolejte stejný nástroj s `continue_: true`
nebo `abort: true`. Otevření merge requestu (`finish` v režimu `merge-request`)
nelze vrátit a plán to říká.

### To, co agent udělá obyčejným `git`, je také v historii vrácení

Agent, který vytváří větve a worktrees, commituje, resetuje nebo zahazuje stashe
vlastním `git` (ne přes tyto nástroje), je přesto vidět: daemon porovná refy
repozitáře s tím, co viděl naposledy, a každou změnu zaznamená jako položku
`undo --list` s `"source": "outside"`, důvodem od gitu (`outside.message`) a
committerem, kterého git zapsal. Nástroje `undo`, `redo` a `undo_restore` berou
tyto položky jako kteroukoli jinou, takže agent (nebo uživatel pomocí `Ctrl+Z`)
může vrátit smazanou větev. Úpravy souborů se nikdy nezachycují. Viz
[Zpět a znovu](undo-redo.md#změny-provedené-mimo-gitpoulpi).

## Zobrazování věcí uživateli

Agent může na něco ukázat, místo aby to popisoval: `ui_show_commit`,
`ui_show_diff` (soubor na řádku, inline nebo split), `ui_show_file` (obsah,
blame nebo historie na řádku), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`,
`ui_show_conflict` (editor merge na souboru v konfliktu, u konfliktu),
`ui_show_wip` (`composer` umístí kurzor do souhrnu commitu, nic nenapíše),
`ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette` (napsaný
dotaz, nikdy nespuštěný), `ui_show_keys`, `ui_show_settings` (stránka a
nastavení, nikdy neupravené), `ui_highlight` (podbarví řádky otevřeného diffu
nebo souboru), `ui_notify` (toast) a `ui_back` (vrátí poslední změnu zobrazení
od agenta). `ui_state` čte, co okno ukazuje (`text: true` přidá viditelný text);
`ui_viewers` vypíše okna.

Tyto nástroje **nikdy nemění repozitář** a neumí psát, klikat ani upravovat
nastavení. Revize a cesty se zkontrolují, než cokoli dorazí do okna.

**Co uživatel vidí a rozhoduje**:

- `ask` (výchozí), nebo kdykoli píšete, je otevřený dialog či Nastavení, táhnete
  nebo jste za posledních 1,5 s stiskli klávesu: agent okamžitě dostane
  `deferred` a váš stavový řádek ukáže čip *„claude-code chce zobrazit ...“*. Zobrazí
  se pomocí `Ctrl+N Enter` nebo kliknutím; `Esc` nebo `Ctrl+N Backspace` ho
  zavře. Vyprší po 60 s.
- Použito: zobrazení se přesune, 4 s toast řekne, kdo ho přesunul, a karta
  dostane fialové `✦` do vaší další klávesy. `Ctrl+N Backspace` (paleta
  `Agent: Zpět`) se vrátí; okno si pamatuje deset zobrazení.
- Nic se nikdy neodkryje ani nepřepne (checkout): commit, který skrývá váš solo
  filtr, odpoví `partial` (`hidden_by_filter`); seznam PR nebo issues, který
  není propojen s poskytovatelem, odpoví `hosting_not_linked`.

**Odpovědi**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, s `state` okna, když bylo
použito. Kódy ukončení CLI: 0 applied/partial/proposed, 3 deferred, jinak 1.
Volání jsou omezena četností (dávka 5, pak jedno za sekundu).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Navrhování změny (`ui_propose_*`)

S `allow_ui_propose = true` může agent otevřít jeden z vašich dialogů
**předvyplněný** a označený *✦ Navrhl <agent>*. Nic se nestane, dokud nestisknete
vlastní potvrzení daného dialogu; můžete nejdřív upravit nebo zrušit.

| Nástroj | Otevře | Potvrdíte pomocí |
|---|---|---|
| `ui_propose_reword` (rev, message) | vyskakovací okno Reword | *Aktualizovat zprávu* (u odeslaného commitu se stále zeptá na force push) |
| `ui_propose_commit_message` (summary, body) | formulář commitu, vyplněný; nikdy necommituje | tlačítko Commit |
| `ui_propose_branch` (name, start point) | inline pole Nová větev | `Enter` |
| `ui_propose_tag` (name, target, message) | inline pole Nový tag (anotovaný, když je zpráva) | `Enter` |
| `ui_propose_resolution` (path, pick or content file) | editor merge na souboru v konfliktu, s volbami (nebo celým textem) již ve Výstupu | *Uložit a označit jako vyřešené* (`Ctrl+S`); zavřením se odmítne |
| `ui_propose_rewrite` (recipe file) | okno přepisu historie s pravidly sady v seznamu; nic se nezobrazuje v náhledu ani neprovádí | váš vlastní *Náhled* a pak *Provést* (`Ctrl+Enter`); `Esc` odmítne. Sada se skriptem se odmítne, tajné hodnoty se vyjmou |
| `ui_propose_split` (rev, plan file) | editor rozdělení nad commitem, vyplněný podle plánu | *Rozdělit* (`Ctrl+S`); `Esc` odmítne |

Vždy jen jeden návrh; návrh zprávy commitu se odmítne, když máte v composeru
vlastní koncept, a návrh řešení, když editor merge drží vaši neuloženou práci
nebo když se stavy souboru od jeho přečtení agentem změnily. Řešení, které před
potvrzením změníte, je hlášeno jako `accepted_edited`. Odpověď obsahuje id
`proposal`; to, jak jste rozhodli, je v `proposals` ve výstupu `ui_state`
(`offered`, `open`, `accepted`, `accepted_edited`, `rejected`, `expired`,
`failed`).

### Když není otevřeno žádné okno nebo je v jiné relaci

- Okno patří relaci, ve které bylo spuštěno. Pokud relace agenta žádné nemá,
  `ui show` hledá **jedno** okno v jiné relaci, které má repozitář otevřený, a
  použije ho (`routed_session` v odpovědi). Pokud vyhovuje několik: `no_viewer`
  s jejich výpisem; zadejte `--session`.
- **Otevření okna pro agenta** je ve výchozím stavu vypnuto. Nastavte
  `[ui] launch` (Nastavení > Agenti a MCP > *Otevřít okno pro agenty*) na předvolbu
  terminálu nebo multiplexeru či na příkaz `custom`; pak volání s `launch: true`
  (CLI `gitpoulpi ui --launch show ...`) jedno otevře a čeká až 10 s. S
  `ui_control = "auto"` se otevře i bez příznaku; s `ask` agent dostane
  `no_viewer` a měl by vám to říct.

| Předvolba | Příkaz |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (váš výchozí terminál) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (nové okno), `zellij` (nový panel), `cmux` (nový workspace) | uvnitř běžícího multiplexeru, ve složce repozitáře |
| `custom` | `[ui] launch_command`, např. `my-term --cd {repo} {gitpoulpi} --session {session}`; rozdělí se na argumenty, bez shellu |

Pokud předvolba s vaší verzí programu nefunguje, použijte `custom`.

## Chyby

- Poškozené volání (neznámý nástroj nebo argument, chybějící povinný) je chyba
  JSON-RPC `-32602 invalid_params`.
- Odmítnutí, nebo chyba gitu či daemona (špatná revize, konflikt, ...), je chyba
  nástroje: `isError: true` se zprávou v `content` a `structuredContent.error`.
