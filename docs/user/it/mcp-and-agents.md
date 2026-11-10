[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · **Italiano** · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP e agenti

Un agente IA (Claude Code, Cursor, Codex, ...) può pilotare GitPoulpi tramite il
suo **server MCP** o la sua **CLI**. Entrambi passano dallo stesso daemon in
background della finestra, quindi ciò che fa l'agente compare subito nel tuo
GitPoulpi aperto, ed entrambi eseguono esattamente lo stesso codice per ogni
comando.

Questa pagina riguarda gli agenti *esterni*. I pulsanti IA di GitPoulpi (messaggi
di commit, spiegazioni, compositore di commit) si configurano a parte: vedi
[Assistenza IA](ai.md).

## Configurazione rapida

1. Metti `gitpoulpi` e `gitpoulpi-mcp` nel tuo `PATH` ([Installazione](installation.md)).
2. Registra il server nel tuo agente. Parla in stdio: non c'è nulla da avviare e nessuna porta.
3. Se vuoi, scegli cosa può fare l'agente nella tabella `[mcp]` di `config.toml`
   (sotto). I valori predefiniti gli permettono di leggere il
   repository (le modifiche sono rifiutate finché non imposti `allow_mutations = true`), di mostrarti cose solo dopo la tua conferma e di non precompilare
   mai i dialoghi.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` lo scrive invece nel `.mcp.json` del repository, così lo
ottiene chiunque apra il progetto. Lo stesso file a mano, con gli extra
facoltativi:

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

- `--ui-control auto|ask|off` sostituisce `[mcp] ui_control` solo per questo agente.
- `GITPOULPI_CONFIG_DIR` dà all'agente un suo `config.toml` (suoi limiti `[mcp]`,
  nessun accesso alle tue impostazioni). Omettilo per condividere le tue.
- `gitpoulpi --repo <path> mcp serve --stdio` vincola il server a un solo
  repository; altrimenti uno strumento agisce sul repository che contiene la
  cartella di lavoro del server, oppure su quello dell'argomento `repo` della chiamata.

**Altri client**: qualsiasi client MCP che avvia un server stdio.

| Client | Dove | Cosa |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | il file JSON MCP del client | il blocco `mcpServers` qui sopra (VS Code chiama la chiave `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Qualsiasi altro | la sua documentazione | il comando `gitpoulpi-mcp` (binario autonomo, stesso server, nessun argomento necessario) |

**Via HTTP** (client che si collegano a un URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

JSON senza stato, stessi strumenti, stessi controlli `[mcp]`. Ogni richiesta richiede il
token bearer (`[mcp] http_auth`, attivo per impostazione predefinita): stampalo con
`gitpoulpi mcp token --show` (oppure `--copy`, `--rotate`; il token vive in un file
riservato all'utente nella cartella dati privata) e daglielo al client come
intestazione:

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

Un token mancante o sbagliato riceve `401` con `WWW-Authenticate: Bearer`. Un token
ruotato si applica subito, senza riavviare il server. Disattivare `http_auth` è per un
endpoint loopback in cui ti fidi di ogni account locale; il server avvisa allora su
stderr.

### Raggiungere l'endpoint da un'altra macchina

La configurazione più sicura non richiede alcun listener remoto: mantieni l'indirizzo
loopback e inoltra la porta tramite SSH dalla macchina che esegue l'agente, `ssh -L
8808:127.0.0.1:8808 host`. Allora `--allow-remote` non entra in gioco.

`--allow-remote` (un indirizzo non loopback) viene rifiutato a meno che l'autenticazione
sia attiva con un token presente **e** il traffico sia cifrato. Il rifiuto nomina ciò che
manca. Ci sono due modi per essere cifrati:

- **TLS nel server** (`[mcp] tls = "self_signed"` o `"files"`). Il server termina TLS
  da sé: possiede la chiave, esegue l'handshake e solo un handshake completato arriva
  al livello HTTP; un client HTTP in chiaro fallisce l'handshake e accanto non esiste
  alcun listener in chiaro. Quindi la cifratura è un fatto che il server stabilisce a
  ogni connessione, non un'ipotesi. Con `self_signed`, il primo avvio genera una coppia
  nella cartella dati privata e stampa la sua impronta SHA-256 (anche nelle
  Impostazioni); un client si fida del server fissando quell'impronta o il file del
  certificato. Con `files`, imposta `tls_cert` e `tls_key` ai tuoi file PEM e i client
  verificano il certificato tramite la sua CA. I client si autenticano con il token
  bearer; i certificati client non vengono usati.
- **Un tunnel o un reverse proxy** (`[mcp] behind_proxy = true`). Un agente di tunnel o
  un proxy si connette come qualsiasi client locale o di LAN e le sue intestazioni
  possono essere falsificate, quindi il server non può capire dall'interno se ce n'è
  uno. Per questo è una dichiarazione esplicita (mostrata con un avviso rosso nelle
  Impostazioni) invece di un rilevamento. Esegui il tunnel sulla stessa macchina e
  mantieni l'indirizzo loopback quando puoi. Con un indirizzo non loopback dietro un
  proxy, il tratto tra il proxy e il server viaggia in chiaro su quella rete.

**Senza MCP**, un agente con una shell usa la CLI: `gitpoulpi --json <command>`
(vedi il [riferimento della CLI](cli-reference.md)). Dagli la skill pronta
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), che spiega quando
e come usare GitPoulpi al posto di `git` puro.

## Permessi

| Interruttore | Predefinito | Effetto |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` si rifiutano di avviarsi (Impostazioni > Agenti e MCP > *Server MCP*). Letto all'avvio del server: un server in esecuzione mantiene le sue impostazioni. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Indirizzo che `--http` usa quando non ne viene dato uno (solo loopback salvo `--allow-remote`, che richiede autenticazione e cifratura, vedi sopra). Impostazioni > Agenti e MCP > *Porta del server MCP* ne modifica la porta e mantiene l'host. |
| `[mcp] http_auth` | `true` | `--http` richiede `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Impostazioni > Agenti e MCP > *Autenticazione HTTP*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` o `files` (i tuoi percorsi PEM): il server termina TLS da sé. Impostazioni > Agenti e MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Dichiari che un tunnel o un reverse proxy cifra il traffico, il che permette a `--allow-remote` di funzionare senza TLS. |
| `[mcp] allow_mutations` | `false` | Disattivato: ogni strumento che modifica qualcosa viene rifiutato; letture, `ui_*` e proposte funzionano comunque. `true` lascia all'agente commit, push, reset... |
| `[mcp] mutation_allowlist` | `[]` | Non vuota: solo questi strumenti possono modificare, per esempio `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: l'agente non può muovere la tua finestra (`ui_state` / `ui_viewers` rispondono comunque). |
| `[mcp] ui_control` | `"ask"` | `ask`: un chip nella tua finestra che confermi tu; `auto`: applicato a meno che tu non sia occupato; `off`: rifiutato. Per singolo agente: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Lascia che l'agente apra dialoghi precompilati che confermi tu (Impostazioni > Agenti e MCP > *Lascia che gli agenti IA propongano modifiche da confermare*). |
| `[mcp] allow_sensitive_staging` | `false` | Solo in `config.toml`. Disattivato, lo strumento `stage` non offre `allow_sensitive`, e uno stage che aggiungerebbe un file sensibile (una chiave privata, un `.env`, un token riconosciuto) viene rifiutato con `{error: "sensitive_files", files}`: l'agente deve chiedere a te. `true` gli lascia passare `allow_sensitive`. Vedi [Staging e commit](staging-and-committing.md#file-sensibili). |
| `[mcp] allow_history_purge` | `false` | Solo in `config.toml`. `true` elenca lo strumento `rewrite_purge`, che elimina definitivamente la vecchia cronologia di una riscrittura (e il suo annullamento). Disattivato: l'agente non lo vede. Richiede comunque `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | L'interruttore generale della tua finestra (Impostazioni > Agenti e MCP > *Consenti agli agenti IA di pilotare la vista*). Disattivato: nessun agente raggiunge la tua finestra. |
| `[ui] launch` | `"off"` | Se un agente può aprire una finestra di GitPoulpi quando non ce n'è nessuna aperta (vedi [sotto](#quando-nessuna-finestra-è-aperta-o-è-in-unaltra-sessione)). |

Cosa significa in pratica:

- **Di base l'agente è in sola lettura**: può leggere, mostrarti cose nella tua
  finestra (`ui_*`) e, se lo permetti, proporti modifiche che confermi tu. Per
  lasciargli modificare il repository (commit, push, reset...) imposta
  `allow_mutations = true`; per una concessione ristretta lascialo `true` con una
  `mutation_allowlist`. Ogni strumento porta le annotazioni MCP (`readOnlyHint`,
  `destructiveHint`, `openWorldHint`), quindi un client come Claude Code ti
  chiede comunque conferma prima di quelli rischiosi.
- **Conflitti**: `conflicts_check` (una previsione, prima di qualsiasi merge), `conflicts_list`, `conflicts_show` e `conflicts_state` sono
  letture; `conflicts_resolve` (scarta il lato non tenuto, `--delete` rimuove il
  file) e `conflicts_abort` (annulla l'intero merge/rebase/cherry-pick) sono
  distruttivi; `conflicts_unresolve` è una semplice scrittura. Una chiamata
  rifiutata (`stale`, `not_conflicted`, `pick_incomplete`...) risponde
  `{"refused": {...}}` con codice di uscita 2 invece di fallire.
- **Stati bloccati**: `conflicts_am_continue` è una scrittura, `conflicts_am_skip`,
  `conflicts_am_abort` e `index_lock_remove` sono distruttivi, `index_lock_show`
  è una lettura.
- **Riscrittura della cronologia**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (l'analisi delle dimensioni), `rewrite_scan_secrets` (risultati con un'anteprima oscurata, mai un valore),
  `rewrite_script_check`, `rewrite_script_test` e `rewrite_discard` sono letture
  (un'anteprima non cambia nulla: conserva solo un piano nel daemon e su disco,
  nella directory git, quindi `rewrite_apply` funziona anche dopo un riavvio).
  `rewrite_apply` e `rewrite_run` sono distruttivi: spostano molti ref su commit
  riscritti come una sola voce di annullamento (`undo` li rimette a posto) e non
  inviano mai nulla. Passa la ricetta come `recipe_json` (o `recipe`, un percorso)
  e i segreti come `secrets_file`, mai in chiaro nella chiamata.
  Gli script di shell, Python e Lua esterno (`script_language`) non sono isolati
  e **un agente non può considerarli attendibili**: non esiste uno strumento
  `rewrite_script_trust` né un argomento `trust`, quindi uno script simile viene
  eseguito solo dopo che una persona ha eseguito `gitpoulpi rewrite script trust FILE`
  su quel testo esatto (uno script modificato viene di nuovo rifiutato,
  `script_not_trusted` con l'hash in `refused.script`), e solo se
  `rewrite.external_scripts` attiva il suo linguaggio (`scripts_disabled`).
  Un rifiuto
  risponde `{"refused": {"reason", "message", "detail"}}` e un piano che sposta ref
  già inviati risponde `{"needs_confirmation": ...}` (codice di uscita 3) finché la
  chiamata non imposta `yes`: mostralo prima all'utente.
- **Mai raggiungibili da un agente**: salvare un token (`hosting login`), creare
  e caricare una chiave SSH, `config set` (potrebbe riscrivere i propri limiti) e
  il ciclo di vita del daemon. Un token non viene mai mostrato all'agente.
- **Elenchi a livello di account**: `hosting_radar` è una lettura (richieste in attesa della tua
  revisione, richieste aperte, issue assegnate in ogni repository di un profilo,
  da una cache di cinque minuti salvo `refresh`); `hosting_integrations_radar`
  modifica la tabella `[integrations.radar]` di un profilo, quindi è controllato da `allow_mutations`
  (anche quando viene chiamato senza opzioni, nel qual caso stampa soltanto le impostazioni).
- **Contesto della revisione**: `ai_review_context` mostra ciò che una bozza di revisione del codice riceve
  oltre al diff (CLAUDE.md, AGENTS.md, un file in più, dove viene eseguito un agente locale). Con
  un argomento di impostazione salva un valore predefinito (globale, o quello proprio del `repo` o del `project`),
  quindi è controllato da `allow_mutations`, come gli altri strumenti che modificano un'impostazione; senza argomenti
  stampa soltanto. Come ogni strumento `ai_*`, è elencato solo mentre `ai.enabled` è attivo.
  `hosting_review_suggest` accetta gli stessi argomenti, ma per una sola esecuzione.
- **I controlli `[mcp]` valgono per il server MCP.** La CLI è la tua shell e non
  è soggetta a `allow_mutations`; solo `ui_control` e `allow_ui_propose`
  valgono anche per `gitpoulpi ui ...`.
- Una chiamata rifiutata è un normale errore di strumento (`isError: true`, il
  motivo in `structuredContent.error`), mai un'operazione silenziosamente
  ignorata.

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

## Gli strumenti

Uno strumento per ogni comando della CLI, chiamato come il suo percorso con `_`
(`branch create` diventa `branch_create`). Ognuno accetta gli argomenti del
comando più `repo` (il repository; predefinito: quello del server) e `session`.
Lo `structuredContent` di una chiamata riuscita è esattamente ciò che stampa
`gitpoulpi --json <command>`. **Tipo**: *read* (lettura) non modifica mai nulla;
*write* (scrittura) e *destructive* (distruttivo) sono modifiche (soggette ai
permessi); *network* (rete) parla con un remoto o un host; *view* (vista) muove
solo la tua finestra; *propose* (proposta) apre un dialogo che confermi tu.

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

### Le azioni del workflow di branching sono piano-poi-esecuzione

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` e `flow_set_parent`
([Workflow di branching](branching-workflows.md#azioni)) non vengono mai eseguiti con una
chiamata nuda. `dry_run: true` restituisce il piano (ogni passaggio e comando git, avvisi,
ciò che l'annullamento non può riprendere, blocchi) come un normale risultato; `yes: true` lo esegue;
con nessuno dei due, la chiamata viene rifiutata e porta con sé il piano. Gli `[hooks]` del
file del workflow sono comandi di shell di un file che il team condivide, quindi vengono eseguiti solo con
un esplicito `run_hooks: true`: il piano li elenca (`hooks`, `hooks_run`) così puoi
mostrare all'utente i comandi esatti prima. Un merge che si ferma sui conflitti
risponde `status: "conflicted"` (un risultato di errore, codice di uscita 2); dopo che gli strumenti
sui conflitti hanno risolto i file, richiama lo stesso strumento con `continue_: true`
o `abort: true`. Aprire una merge request (`finish` in modalità `merge-request`)
non è annullabile e il piano lo dice.

### Ciò che un agente fa con `git` semplice è anche nella cronologia annullamenti

Un agente che crea branch e worktree, fa commit, reset o elimina stash con il proprio
`git` (non tramite questi strumenti) resta comunque visibile: il daemon confronta i
ref del repository con ciò che ha visto l'ultima volta e registra ogni modifica
come voce di `undo --list` con `"source": "outside"`, il motivo di git
(`outside.message`) e il committer che git ha registrato. Gli strumenti `undo`, `redo`
e `undo_restore` trattano queste voci come qualsiasi altra, quindi un agente (o
l'utente, con `Ctrl+Z`) può riportare indietro un branch che era stato eliminato.
Le modifiche ai file non vengono mai catturate. Vedi
[Annulla e ripeti](undo-redo.md#modifiche-fatte-fuori-da-gitpoulpi).

## Mostrare le cose all'utente

Un agente può indicare qualcosa invece di descriverlo: `ui_show_commit`,
`ui_show_diff` (un file a una riga, inline o split), `ui_show_file` (contenuto,
blame o cronologia a una riga), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`,
`ui_show_conflict` (l'editor di merge su un file in conflitto, a un conflitto),
`ui_show_wip` (`composer` porta il cursore nel riepilogo del commit, senza
scrivere nulla), `ui_show_tab`, `ui_show_home`, `ui_show_search`,
`ui_show_palette` (la query digitata, mai eseguita), `ui_show_keys`,
`ui_show_settings` (una pagina e un'impostazione, mai modificata),
`ui_highlight` (colora righe del diff o del file aperto), `ui_notify` (un toast)
e `ui_back` (annulla l'ultimo cambio di vista dell'agente). `ui_state` legge ciò
che mostra una finestra (`text: true` aggiunge il testo visibile); `ui_viewers`
elenca le finestre.

Questi strumenti **non modificano mai il repository** e non possono digitare,
cliccare o modificare un'impostazione. Revisioni e percorsi vengono verificati
prima che qualcosa raggiunga la finestra.

**Cosa vede e decide l'utente**:

- Con `ask` (predefinito), o ogni volta che stai scrivendo, c'è un dialogo o
  le Impostazioni aperti, stai trascinando, oppure hai premuto un tasto nell'ultimo
  secondo e mezzo: l'agente riceve subito `deferred` e la tua barra di stato mostra un
  chip *"claude-code wants to show ..."*. `Ctrl+N Enter` o un clic lo mostra;
  `Esc` o `Ctrl+N Backspace` lo scarta. Scade dopo 60 s.
- Applicato: la vista si sposta, un toast di 4 s dice chi l'ha mossa e la scheda
  riceve un `✦` viola fino al tuo tasto successivo. `Ctrl+N Backspace` (palette
  `Agente: Indietro`) torna indietro; la finestra ricorda dieci viste.
- Nulla viene mai reso visibile se nascosto né sottoposto a checkout: un commit che il
  tuo filtro solo nasconde risponde `partial` (`hidden_by_filter`); un elenco di
  PR o issue non collegato a un provider risponde `hosting_not_linked`.

**Risposte**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, con lo `state` della finestra
quando applicato. Codici di uscita della CLI: 0 applied/partial/proposed, 3
deferred, 1 negli altri casi. Le chiamate hanno un limite di frequenza (raffica
di 5, poi una al secondo).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Proporre una modifica (`ui_propose_*`)

Con `allow_ui_propose = true`, un agente può aprire uno dei tuoi dialoghi
**precompilato**, contrassegnato *✦ Proposto da <agente>*. Non succede nulla
finché non premi la conferma del dialogo stesso; puoi prima modificare o annullare.

| Strumento | Apre | Confermi con |
|---|---|---|
| `ui_propose_reword` (rev, message) | il popup Reword | *Aggiorna il messaggio* (un commit già inviato chiede comunque del force push) |
| `ui_propose_commit_message` (summary, body) | il modulo di commit, compilato; non fa mai il commit | il pulsante Commit |
| `ui_propose_branch` (name, start point) | la casella inline Nuovo branch | `Enter` |
| `ui_propose_tag` (name, target, message) | la casella inline Nuovo tag (annotato quando c'è un messaggio) | `Enter` |
| `ui_propose_split` (rev, plan file) | l'editor di divisione sul commit, compilato dal piano | *Dividi* (`Ctrl+S`); `Esc` rifiuta |
| `ui_propose_rewrite` (recipe file) | il pannello di riscrittura della cronologia con le regole della ricetta; nulla viene mostrato in anteprima né applicato | la tua *Anteprima* e poi *Applica* (`Ctrl+Enter`); `Esc` rifiuta. Una ricetta con uno script viene rifiutata, i valori segreti vengono tolti |
| `ui_propose_resolution` (path, pick or content file) | l'editor di merge su un file in conflitto, con le scelte (o l'intero testo) già nel Risultato | *Salva e segna come risolto* (`Ctrl+S`); chiuderlo rifiuta |

Una proposta alla volta; una proposta di messaggio di commit viene rifiutata se
hai già una tua bozza nel compositore, e una proposta di risoluzione se l'editor
di merge contiene un tuo lavoro non salvato o se gli stage del file sono cambiati
da quando l'agente li ha letti. Una risoluzione che modifichi prima di
confermare viene segnalata come `accepted_edited`. La risposta ha un id
`proposal`; ciò che hai deciso è in `proposals` di `ui_state` (`offered`, `open`,
`accepted`, `accepted_edited`, `rejected`, `expired`, `failed`).

### Quando nessuna finestra è aperta, o è in un'altra sessione

- Una finestra appartiene alla sessione in cui è stata avviata. Se la sessione
  dell'agente non ne ha, `ui show` cerca **una** finestra in un'altra sessione che
  abbia il repository aperto e la usa (`routed_session` nella risposta). Se ne
  corrispondono diverse: `no_viewer` che le nomina; passa `--session`.
- **Aprire una finestra per l'agente** è disattivato di base. Imposta
  `[ui] launch` (Impostazioni > Agenti e MCP > *Apri una finestra per gli agenti*) su un preset di
  terminale o multiplexer oppure su un comando `custom`; a quel punto una
  chiamata con `launch: true` (CLI `gitpoulpi ui --launch show ...`) ne apre una
  e attende fino a 10 s. Con `ui_control = "auto"` viene aperta anche senza il
  flag; con `ask` l'agente riceve `no_viewer` e dovrebbe avvisarti.

| Preset | Comando |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (il tuo terminale predefinito) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (nuova finestra), `zellij` (nuovo pannello), `cmux` (nuovo workspace) | dentro un multiplexer in esecuzione, nella cartella del repository |
| `custom` | `[ui] launch_command`, per esempio `my-term --cd {repo} {gitpoulpi} --session {session}`; diviso in argomenti, senza shell |

Se un preset non funziona con la tua versione del programma, usa `custom`.

## Errori

- Una chiamata malformata (strumento o argomento sconosciuto, un argomento
  obbligatorio mancante) è un errore JSON-RPC `-32602 invalid_params`.
- Un rifiuto, o un errore di git o del daemon (revisione non valida, conflitto,
  ...), è un errore di strumento: `isError: true` con il messaggio in `content` e
  in `structuredContent.error`.
