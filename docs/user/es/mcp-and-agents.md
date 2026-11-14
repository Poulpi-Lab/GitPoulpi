[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · **Español** · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP y agentes

Un agente de IA (Claude Code, Cursor, Codex, ...) puede manejar GitPoulpi a
través de su **servidor MCP** o de su **CLI**. Ambos pasan por el mismo daemon en
segundo plano que la ventana, así que lo que hace el agente se ve en directo en
tu GitPoulpi abierto, y los dos ejecutan exactamente el mismo código para cada
comando.

Esta página trata de los agentes *externos*. Los botones de IA propios de
GitPoulpi (mensajes de commit, explicaciones, el compositor de commits) se
configuran aparte: consulta [Asistencia de IA](ai.md).

## Configuración rápida

1. Pon `gitpoulpi` y `gitpoulpi-mcp` en tu `PATH` ([Instalación](installation.md)).
2. Registra el servidor en tu agente. Habla por stdio: no hay nada que arrancar
   ni puerto que abrir.
3. Si quieres, elige lo que el agente puede hacer en la tabla `[mcp]` de
   `config.toml` (más abajo). Por defecto puede leer el repositorio (los cambios se rechazan hasta que
      pongas `allow_mutations = true`),
   te muestra cosas solo después de que lo confirmes y nunca rellena diálogos de
   antemano.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` lo escribe en el `.mcp.json` del repositorio, de modo que
todos los que abran el proyecto lo tengan. El mismo archivo a mano, con los
extras opcionales:

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

- `--ui-control auto|ask|off` sustituye a `[mcp] ui_control` solo para este
  agente.
- `GITPOULPI_CONFIG_DIR` da al agente su propio `config.toml` (sus propios
  límites `[mcp]`, sin acceso a tus ajustes). Omítelo para compartir el tuyo.
- `gitpoulpi --repo <path> mcp serve --stdio` fija el servidor a un único
  repositorio; si no, una herramienta actúa sobre el repositorio que rodea el
  directorio de trabajo del servidor, o sobre el argumento `repo` de la llamada.

**Otros clientes**: cualquier cliente MCP que lance un servidor stdio.

| Cliente | Dónde | Qué |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | el archivo JSON de MCP del cliente | el bloque `mcpServers` de arriba (VS Code llama a la clave `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Cualquier otro | su documentación | el comando `gitpoulpi-mcp` (binario independiente, mismo servidor, no necesita argumentos) |

**Por HTTP** (clientes que se conectan a una URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, por defecto http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

JSON sin estado, las mismas herramientas, las mismas puertas `[mcp]`. Cada
solicitud necesita el token bearer (`[mcp] http_auth`, activado por defecto):
imprímelo con `gitpoulpi mcp token --show` (o `--copy`, `--rotate`; el token vive
en un archivo que solo lee el usuario, bajo la carpeta de datos privada) y dáselo
al cliente como cabecera:

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

Un token ausente o incorrecto recibe `401` con `WWW-Authenticate: Bearer`. Un
token renovado se aplica al instante, sin reiniciar el servidor. Desactivar
`http_auth` es para un endpoint de loopback en el que confías en todas las
cuentas locales; el servidor avisa entonces por stderr.

### Acceder al endpoint desde otra máquina

La configuración más segura no necesita ningún listener remoto: mantén la
dirección de loopback y reenvía el puerto por SSH desde la máquina que ejecuta el
agente, `ssh -L 8808:127.0.0.1:8808 host`. Entonces `--allow-remote` no
interviene.

`--allow-remote` (una dirección que no es de loopback) se rechaza salvo que la
autenticación esté activada con un token en su sitio **y** el tráfico esté
cifrado. El rechazo nombra lo que falta. Hay dos formas de estar cifrado:

- **TLS en el servidor** (`[mcp] tls = "self_signed"` o `"files"`). El servidor
  termina TLS él mismo: tiene la clave, ejecuta el handshake, y solo un handshake
  terminado llega a la capa HTTP; un cliente HTTP sin cifrar falla el handshake y
  no existe ningún listener sin cifrar a su lado. Así, el cifrado es un hecho que
  el servidor establece en cada conexión, no una suposición. Con `self_signed`, el
  primer arranque genera un par bajo la carpeta de datos privada e imprime su
  huella SHA-256 (también en Ajustes); un cliente confía en el servidor fijando
  esa huella o el archivo del certificado. Con `files`, pon `tls_cert` y
  `tls_key` en tus archivos PEM y los clientes verifican el certificado por su
  CA. Los clientes se autentican con el token bearer; no se usan certificados de
  cliente.
- **Un túnel o proxy inverso** (`[mcp] behind_proxy = true`). Un agente de túnel
  o un proxy se conecta como cualquier cliente local o de la LAN y sus cabeceras
  se pueden falsificar, así que el servidor no puede saber desde dentro si hay
  uno. Por eso es un reconocimiento explícito (se muestra con un aviso rojo en
  Ajustes) en lugar de una detección. Ejecuta el túnel en la misma máquina y
  mantén la dirección de loopback cuando puedas. Con una dirección que no es de
  loopback detrás de un proxy, el tramo entre el proxy y el servidor va en texto
  plano por esa red.

**Sin MCP**, un agente con acceso a un shell usa la CLI: `gitpoulpi --json <command>`
(consulta la [referencia de la CLI](cli-reference.md)). Dale la skill ya
preparada [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), que
dice cuándo y cómo usar GitPoulpi en lugar de `git` a secas.

## Permisos

| Interruptor | Por defecto | Efecto |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` se niegan a arrancar (Ajustes > Agentes y MCP > *Servidor MCP*). Se lee al arrancar el servidor: un servidor en marcha conserva sus ajustes. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Dirección que usa `--http` cuando no se indica ninguna (solo loopback salvo `--allow-remote`, que necesita autenticación y cifrado, véase arriba). Ajustes > Agentes y MCP > *Puerto del servidor MCP* edita su puerto y conserva el host. |
| `[mcp] http_auth` | `true` | `--http` necesita `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Ajustes > Agentes y MCP > *Autenticación HTTP*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` o `files` (tus rutas PEM): el servidor termina TLS él mismo. Ajustes > Agentes y MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Respondes de que un túnel o proxy inverso cifra el tráfico, lo que permite que `--allow-remote` funcione sin TLS. |
| `[mcp] allow_mutations` | `false` | Desactivado: se rechaza toda herramienta que cambie algo; las lecturas, `ui_*` y las propuestas siguen funcionando. `true` permite al agente hacer commit, push, reset... |
| `[mcp] mutation_allowlist` | `[]` | No vacío: solo estas herramientas pueden cambiar cosas, p. ej. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_sensitive_staging` | `false` | Solo en `config.toml`. Desactivado, la herramienta `stage` no ofrece `allow_sensitive`, y un stage que añadiría un archivo sensible (una clave privada, un `.env`, un token reconocido) se rechaza con `{error: "sensitive_files", files}`: el agente debe preguntarte. `true` le deja pasar `allow_sensitive`. Consulta [Stage y commit](staging-and-committing.md#archivos-sensibles). |
| `[mcp] allow_history_purge` | `false` | Solo en `config.toml`. `true` muestra la herramienta `rewrite_purge`, que elimina para siempre el historial antiguo de una reescritura (y su deshacer). Desactivado: el agente no la ve. Sigue necesitando `allow_mutations`. |
| `[mcp] allow_ui` | `true` | `false`: el agente no puede mover tu ventana (`ui_state` / `ui_viewers` siguen respondiendo). |
| `[mcp] ui_control` | `"ask"` | `ask`: un chip en tu ventana que confirmas tú; `auto`: se aplica salvo que estés ocupado; `off`: se rechaza. Por agente: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Deja que el agente abra diálogos rellenados de antemano que tú confirmas (Ajustes > Agentes y MCP > *Dejar que los agentes de IA propongan cambios para que yo los confirme*). |
| `[ui] allow_agent_view_control` | `true` | El interruptor general de tu ventana (Ajustes > Agentes y MCP > *Permitir que los agentes de IA controlen la vista*). Desactivado: ningún agente puede llegar a tu ventana. |
| `[ui] launch` | `"off"` | Si un agente puede abrir una ventana de GitPoulpi cuando no hay ninguna abierta (consulta [más abajo](#cuando-no-hay-ninguna-ventana-abierta-o-está-en-otra-sesión)). |

Qué significa esto en la práctica:

- **De fábrica el agente es de solo lectura**: puede leer, mostrarte cosas en tu
  ventana (`ui_*`) y, si lo permites, proponer cambios que tú confirmas. Para
  dejar que modifique el repositorio (commit, push, reset...) pon
  `allow_mutations = true`; para una concesión estrecha, déjalo en `true` con una
  `mutation_allowlist`. Cada herramienta lleva anotaciones MCP (`readOnlyHint`,
  `destructiveHint`, `openWorldHint`), de modo que un cliente como Claude Code
  sigue pidiéndote confirmación antes de las arriesgadas.
- **Conflictos**: `conflicts_check` (una predicción, antes de cualquier merge), `conflicts_list`, `conflicts_show` y `conflicts_state` son
  lecturas; `conflicts_resolve` (descarta el lado que no se conserva, `--delete`
  elimina el archivo) y `conflicts_abort` (abandona todo el
  merge/rebase/cherry-pick) son destructivas; `conflicts_unresolve` es una
  escritura normal. Una llamada rechazada (`stale`, `not_conflicted`,
  `pick_incomplete`...) responde `{"refused": {...}}` con código de salida 2 en
  lugar de fallar.
- **Estados atascados**: `conflicts_am_continue` es una escritura;
  `conflicts_am_skip`, `conflicts_am_abort` e `index_lock_remove` son
  destructivas; `index_lock_show` es una lectura.
- **Reescritura del historial**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (el análisis de tamaño), `rewrite_scan_secrets` (hallazgos con una vista censurada, nunca un valor),
  `rewrite_script_check`, `rewrite_script_test` y `rewrite_discard` son
  lecturas (una vista previa no cambia nada: solo guarda un plan en el daemon y en
  disco, en el directorio git, así que `rewrite_apply` también funciona tras un
  reinicio). `rewrite_apply` y `rewrite_run` son destructivas: mueven muchas refs a
  commits reescritos como una sola entrada de deshacer (`undo` las devuelve a su
  sitio) y nunca hacen push. Pasa la receta como `recipe_json` (o `recipe`, una
  ruta) y los secretos como `secrets_file`, nunca en línea.
  Los scripts de shell, Python y Lua externo (`script_language`) no están aislados
  y **un agente no puede darles confianza**: no existe la herramienta
  `rewrite_script_trust` ni el argumento `trust`, así que un script así solo se
  ejecuta después de que una persona haya ejecutado `gitpoulpi rewrite script trust FILE`
  sobre ese texto exacto (un script modificado se vuelve a rechazar, `script_not_trusted`
  con el hash en `refused.script`), y solo cuando `rewrite.external_scripts` activa su
  lenguaje (`scripts_disabled`).
  Un rechazo responde
  `{"refused": {"reason", "message", "detail"}}` y un plan que mueve refs ya
  publicadas responde `{"needs_confirmation": ...}` (código de salida 3) hasta que
  la llamada pone `yes`: muéstraselo antes al usuario.
- **Nunca accesible para un agente**: guardar un token (`hosting login`), crear
  y subir una clave SSH, `config set` (podría reescribir sus propios límites) y
  el ciclo de vida del daemon. Un token nunca se muestra al agente.
- **Listas de toda la cuenta**: `hosting_radar` es una lectura (solicitudes que esperan tu
  revisión, solicitudes abiertas, issues asignados en todos los repositorios de un perfil,
  desde una caché de cinco minutos salvo con `refresh`); `hosting_integrations_radar`
  cambia la tabla `[integrations.radar]` de un perfil, así que `allow_mutations` la limita
  (incluso llamada sin opciones, que solo imprime los ajustes).
- **Contexto de la revisión**: `ai_review_context` muestra lo que recibe un borrador de revisión de código
  además del diff (CLAUDE.md, AGENTS.md, un archivo adicional, dónde se ejecuta un agente local). Con
  un argumento de ajuste guarda un valor por defecto (global, o el propio de `repo` o de `project`),
  así que `allow_mutations` lo limita, como las demás herramientas que cambian un ajuste; sin ninguno
  solo imprime. Como todas las herramientas `ai_*`, solo aparece en la lista mientras `ai.enabled` está activado.
  `hosting_review_suggest` toma los mismos argumentos, pero solo para una ejecución.
- **Las puertas `[mcp]` se aplican al servidor MCP.** La CLI es tu propio shell
  y `allow_mutations` no la limita; solo `ui_control` y `allow_ui_propose`
  también se aplican a `gitpoulpi ui ...`.
- Una llamada rechazada es un error de herramienta corriente (`isError: true`,
  el motivo en `structuredContent.error`), nunca una operación nula silenciosa.

```toml
[mcp]
allow_mutations = false        # true: el agente puede modificar el repositorio
mutation_allowlist = []        # p. ej. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" deja que el agente mueva tu vista sin preguntar
allow_sensitive_staging = false  # true: el agente puede hacer stage de archivos que marca la protección de stage
allow_history_purge = false    # true: el agente ve rewrite_purge (elimina el historial antiguo para siempre)
allow_ui_propose = false       # true: el agente puede rellenar de antemano diálogos de Reword / Split / commit / rama / tag
```

## Las herramientas

Una herramienta por comando de la CLI, con el nombre de su ruta unida por `_`
(`branch create` es `branch_create`). Cada una acepta los argumentos del comando
más `repo` (el repositorio; por defecto, el del servidor) y `session`. El
`structuredContent` de una llamada correcta es exactamente lo que imprime
`gitpoulpi --json <command>`. **Tipo** (*Kind*, los valores de la tabla se dejan
tal cual): *read* nunca cambia nada; *write* y *destructive* son mutaciones
(con puerta); *network* habla con un remoto o un host; *view* solo mueve tu
ventana; *propose* abre un diálogo que tú confirmas.

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

### Las acciones del flujo de trabajo de ramas son de planificar y después ejecutar

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` y `flow_set_parent`
([Flujos de trabajo de ramas](branching-workflows.md#acciones)) nunca se ejecutan con
una llamada a secas. `dry_run: true` devuelve el plan (cada paso y comando de git,
avisos, lo que Deshacer no puede revertir, bloqueos) como un resultado normal;
`yes: true` lo ejecuta; sin ninguno de los dos, la llamada se rechaza y lleva el
plan. Los `[hooks]` del archivo del flujo de trabajo son comandos de shell de un
archivo que comparte el equipo, así que solo se ejecutan con un `run_hooks: true`
explícito: el plan los lista (`hooks`, `hooks_run`) para que puedas mostrar antes al
usuario los comandos exactos. Un merge que se detiene por conflictos responde
`status: "conflicted"` (un resultado de error, código de salida 2); cuando las
herramientas de conflictos hayan resuelto los archivos, llama a la misma herramienta
con `continue_: true` o `abort: true`. Abrir un merge request (`finish` en modo
`merge-request`) no se puede deshacer y el plan lo dice.

### Lo que un agente hace con `git` a secas también está en el historial de deshacer

Un agente que crea ramas y worktrees, hace commits, resets o descarta stashes con su propio
`git` (no a través de estas herramientas) sigue siendo visible: el daemon compara las refs del
repositorio con lo que vio por última vez y registra cada cambio como una entrada de
`undo --list` con `"source": "outside"`, el motivo de git (`outside.message`) y el committer
que git registró. Las herramientas `undo`, `redo` y `undo_restore` tratan estas entradas como
cualquier otra, así que un agente (o la persona, con `Ctrl+Z`) puede recuperar una rama que se
eliminó. Las ediciones de archivos nunca se capturan. Ver
[Deshacer y rehacer](undo-redo.md#cambios-hechos-fuera-de-gitpoulpi).

## Mostrar cosas al usuario

Un agente puede señalar algo en lugar de describirlo: `ui_show_commit`,
`ui_show_diff` (un archivo en una línea, en línea o dividido), `ui_show_file`
(contenido, blame o historial en una línea), `ui_show_ref`, `ui_show_pr`,
`ui_show_issue`, `ui_show_conflict` (el editor de merge sobre un archivo en
conflicto, en un conflicto), `ui_show_wip` (`composer` coloca el cursor en el
resumen del commit, sin escribir nada), `ui_show_tab`, `ui_show_home`,
`ui_show_search`, `ui_show_palette` (la consulta escrita, nunca ejecutada),
`ui_show_keys`, `ui_show_settings` (una página y un ajuste, nunca editado),
`ui_highlight` (resalta líneas del diff o del archivo abierto), `ui_notify` (un
aviso emergente) y `ui_back` (deshace el último cambio de vista del agente).
`ui_state` lee lo que muestra una ventana (`text: true` añade el texto visible);
`ui_viewers` lista las ventanas.

Estas herramientas **nunca modifican el repositorio** y no pueden escribir,
hacer clic ni editar un ajuste. Las revisiones y las rutas se comprueban antes de
que nada llegue a la ventana.

**Lo que ve y decide el usuario**:

- `ask` (por defecto), o siempre que estés escribiendo, haya un diálogo o los
  Ajustes abiertos, estés arrastrando o hayas pulsado una tecla en el último
  1,5 s: el agente recibe `deferred` al instante y tu barra de estado muestra un chip
  *«claude-code quiere mostrar ...»*. `Ctrl+N Enter` o un clic lo muestra; `Esc`
  o `Ctrl+N Backspace` lo descarta. Caduca a los 60 s.
- Aplicado: la vista se mueve, un aviso emergente de 4 s dice quién la movió y la
  pestaña recibe un `✦` morado hasta tu siguiente tecla. `Ctrl+N Backspace`
  (paleta `Agente: Volver`) vuelve atrás; la ventana recuerda diez vistas.
- Nunca se desoculta ni se hace checkout de nada: un commit que oculta tu
  filtro solo responde `partial` (`hidden_by_filter`); una lista de PR o de
  issues que no está vinculada a un proveedor responde `hosting_not_linked`.

**Respuestas**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, con el `state` de la ventana
cuando se aplica. Códigos de salida de la CLI: 0 applied/partial/proposed, 3
deferred, 1 en cualquier otro caso. Las llamadas tienen límite de frecuencia
(ráfaga de 5, luego una por segundo).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Proponer un cambio (`ui_propose_*`)

Con `allow_ui_propose = true`, un agente puede abrir uno de tus diálogos
**rellenado de antemano**, marcado *✦ Propuesto por <agente>*. No ocurre nada hasta
que pulsas el propio botón de confirmar de ese diálogo; puedes editar antes o
cancelar.

| Herramienta | Abre | Confirmas con |
|---|---|---|
| `ui_propose_reword` (rev, message) | la ventana emergente Reword | *Actualizar mensaje* (un commit ya enviado sigue preguntando por el force push) |
| `ui_propose_commit_message` (summary, body) | el formulario de commit, relleno; nunca hace commit | el botón Commit |
| `ui_propose_branch` (name, start point) | el cuadro en línea Nueva rama | `Enter` |
| `ui_propose_tag` (name, target, message) | el cuadro en línea Nuevo tag (anotado cuando hay mensaje) | `Enter` |
| `ui_propose_split` (rev, plan file) | el editor de división sobre el commit, rellenado a partir del plan | *Dividir* (`Ctrl+S`); `Esc` lo rechaza |
| `ui_propose_rewrite` (recipe file) | el editor de reescritura del historial con las reglas de la receta ya puestas; no se previsualiza ni se aplica nada | tu propio *Vista previa* y luego *Aplicar* (`Ctrl+Enter`); `Esc` la rechaza. Una receta con un script se rechaza y los valores secretos se descartan |
| `ui_propose_resolution` (path, pick or content file) | el editor de merge sobre un archivo en conflicto, con las elecciones (o el texto entero) ya en el Resultado | *Guardar y marcar resuelto* (`Ctrl+S`); cerrarlo la rechaza |

Una propuesta a la vez; una propuesta de mensaje de commit se rechaza mientras
tengas tu propio borrador en el compositor, y una propuesta de resolución
mientras el editor de merge tenga trabajo tuyo sin guardar o cuando los stages
del archivo hayan cambiado desde que el agente los leyó. Una resolución que
cambias antes de confirmar se notifica como `accepted_edited`. La respuesta trae
un id de `proposal`; lo que decidiste está en `proposals` de `ui_state`
(`offered`, `open`, `accepted`, `accepted_edited`, `rejected`, `expired`,
`failed`).

### Cuando no hay ninguna ventana abierta, o está en otra sesión

- Una ventana pertenece a la sesión en la que se inició. Si la sesión del agente
  no tiene ninguna, `ui show` busca **una** ventana de otra sesión que tenga el
  repositorio abierto y la usa (`routed_session` en la respuesta). Si coinciden
  varias: `no_viewer` que las nombra; pasa `--session`.
- **Abrir una ventana para el agente** está desactivado por defecto. Establece
  `[ui] launch` (Ajustes > Agentes y MCP > *Abrir una ventana para agentes*) en
  un preset de terminal o multiplexor, o en un comando `custom`; entonces una
  llamada con `launch: true` (CLI `gitpoulpi ui --launch show ...`) abre una y
  espera hasta 10 s. Con `ui_control = "auto"` también se abre sin la opción; con
  `ask` el agente recibe `no_viewer` y debería avisarte.

| Preset | Comando |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (tu terminal por defecto) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (nueva ventana), `zellij` (nuevo panel), `cmux` (nuevo espacio de trabajo) | dentro de un multiplexor en ejecución, en la carpeta del repositorio |
| `custom` | `[ui] launch_command`, p. ej. `my-term --cd {repo} {gitpoulpi} --session {session}`; se divide en argumentos, sin shell |

Si un preset no funciona con tu versión del programa, usa `custom`.

## Errores

- Una llamada mal formada (herramienta o argumento desconocido, falta uno
  obligatorio) es un error JSON-RPC `-32602 invalid_params`.
- Un rechazo, o un error de git o del daemon (revisión incorrecta, conflicto,
  ...), es un error de herramienta: `isError: true` con el mensaje en `content` y
  en `structuredContent.error`.
