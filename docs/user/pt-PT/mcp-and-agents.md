[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · **Português (Portugal)** · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP e agentes

Um agente de IA (Claude Code, Cursor, Codex, ...) pode controlar o GitPoulpi através do seu
**servidor MCP** ou da sua **CLI**. Ambos passam pelo mesmo daemon em segundo plano que a
janela, por isso o que o agente faz aparece em direto no seu GitPoulpi aberto, e ambos executam
exatamente o mesmo código para cada comando.

Esta página trata de agentes *externos*. Os botões de IA do próprio GitPoulpi (mensagens
de commit, explicações, o compositor de commits) são configurados em separado: consulte
[Assistência de IA](ai.md).

## Configuração rápida

1. Coloque `gitpoulpi` e `gitpoulpi-mcp` no seu `PATH` ([Instalação](installation.md)).
2. Registe o servidor no seu agente. Fala por stdio: nada para iniciar, nenhuma porta.
3. Opcionalmente, escolha o que o agente pode fazer na tabela `[mcp]` do `config.toml`
   (abaixo). As predefinições deixam-no ler o repositório (as alterações são recusadas
   até definir `allow_mutations = true`), mostrar-lhe coisas apenas depois de
   confirmar, e nunca pré-preencher diálogos.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` escreve-o antes no `.mcp.json` do repositório, para que todos
os que abrirem o projeto o recebam. O mesmo ficheiro à mão, com os extras opcionais:

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

- `--ui-control auto|ask|off` substitui `[mcp] ui_control` apenas para este agente.
- `GITPOULPI_CONFIG_DIR` dá ao agente o seu próprio `config.toml` (os seus próprios limites
  `[mcp]`, sem acesso às suas definições). Omita-o para partilhar o seu.
- `gitpoulpi --repo <path> mcp serve --stdio` fixa o servidor a um repositório;
  caso contrário, uma ferramenta atua no repositório em torno da pasta de trabalho do servidor,
  ou no argumento `repo` da chamada.

**Outros clientes**: qualquer cliente MCP que inicie um servidor stdio.

| Cliente | Onde | O quê |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | o ficheiro JSON de MCP do cliente | o bloco `mcpServers` acima (o VS Code dá à chave o nome `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Qualquer outro | a sua documentação | o comando `gitpoulpi-mcp` (binário autónomo, o mesmo servidor, sem argumentos necessários) |

**Por HTTP** (clientes que se ligam a um URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, predefinição http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

JSON sem estado, as mesmas ferramentas, as mesmas barreiras `[mcp]`. Cada pedido precisa
do token bearer (`[mcp] http_auth`, ativo por predefinição): mostre-o com `gitpoulpi mcp token
--show` (ou `--copy`, `--rotate`; o token fica num ficheiro só do utilizador na pasta
de dados privada) e entregue-o ao cliente como um cabeçalho:

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

Um token em falta ou errado recebe `401` com `WWW-Authenticate: Bearer`. Um token
renovado aplica-se de imediato, sem reiniciar o servidor. Desativar `http_auth` serve para um
endpoint em loopback onde confia em todas as contas locais; o servidor avisa então no
stderr.

### Aceder ao endpoint a partir de outra máquina

A configuração mais segura não precisa de nenhum recetor remoto: mantenha o endereço de loopback e
reencaminhe a porta por SSH a partir da máquina que executa o agente, `ssh -L
8808:127.0.0.1:8808 host`. Assim `--allow-remote` não entra em jogo.

`--allow-remote` (um endereço que não é de loopback) é recusado a menos que a autenticação esteja ativa
com um token disponível **e** o tráfego esteja cifrado. A recusa diz o que falta. Há duas
formas de estar cifrado:

- **TLS no servidor** (`[mcp] tls = "self_signed"` ou `"files"`). O servidor
  termina o TLS ele próprio: tem a chave, executa o handshake, e só um handshake
  concluído chega à camada HTTP; um cliente HTTP simples falha o handshake e não existe
  nenhum recetor simples ao lado. Assim, a cifra é um facto que o servidor estabelece em cada
  ligação, não uma suposição. Com `self_signed`, o primeiro arranque gera um par na pasta de dados
  privada e mostra a sua impressão digital SHA-256 (também nas Definições); um cliente confia no servidor fixando
  essa impressão digital ou o ficheiro do certificado. Com `files`, defina `tls_cert` e
  `tls_key` para os seus ficheiros PEM e os clientes verificam o certificado pela sua CA.
  Os clientes autenticam-se com o token bearer; os certificados de cliente não são usados.
- **Um túnel ou proxy inverso** (`[mcp] behind_proxy = true`). Um agente de túnel ou
  proxy liga-se como qualquer cliente local ou da LAN e os seus cabeçalhos podem ser falsificados, por isso
  o servidor não consegue saber, a partir de dentro, se existe um. É por isso que é um
  reconhecimento explícito (mostrado com um aviso vermelho nas Definições) em vez de uma
  deteção. Execute o túnel na mesma máquina e mantenha o endereço de loopback quando
  puder. Com um endereço que não é de loopback atrás de um proxy, o troço entre o proxy
  e o servidor circula em texto simples nessa rede.

**Sem MCP**, um agente com uma shell usa a CLI: `gitpoulpi --json <command>`
(consulte a [referência da CLI](cli-reference.md)). Dê-lhe a skill pronta
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), que diz quando
e como usar o GitPoulpi em vez do `git` em bruto.

## Permissões

| Interruptor | Predefinição | Efeito |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` recusam-se a iniciar (Definições > Agentes e MCP > *Servidor MCP*). Lido quando o servidor arranca: um servidor em execução mantém as suas definições. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Endereço que `--http` usa quando nenhum é indicado (apenas loopback, a menos que `--allow-remote`, que exige autenticação e cifra, veja acima). Definições > Agentes e MCP > *Porta do servidor MCP* edita a respetiva porta e mantém o anfitrião. |
| `[mcp] http_auth` | `true` | `--http` precisa de `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Definições > Agentes e MCP > *Autenticação HTTP*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` ou `files` (os seus caminhos PEM): o servidor termina o TLS ele próprio. Definições > Agentes e MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Garante que um túnel ou proxy inverso cifra o tráfego, o que deixa `--allow-remote` funcionar sem TLS. |
| `[mcp] allow_mutations` | `false` | Desligado: toda a ferramenta que altera algo é recusada; leituras, `ui_*` e propostas continuam a funcionar. `true` deixa o agente fazer commit, push, reset... |
| `[mcp] mutation_allowlist` | `[]` | Não vazio: só estas ferramentas podem alterar coisas, por exemplo `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: o agente não pode mover a sua janela (`ui_state` / `ui_viewers` continuam a responder). |
| `[mcp] ui_control` | `"ask"` | `ask`: um chip na sua janela que confirma; `auto`: aplicado a menos que esteja ocupado; `off`: recusado. Por agente: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Deixa o agente abrir diálogos pré-preenchidos que confirma (Definições > Agentes e MCP > *Deixar agentes de IA proporem alterações para eu confirmar*). |
| `[mcp] allow_sensitive_staging` | `false` | Só no `config.toml`. Desligado, a ferramenta `stage` não oferece `allow_sensitive`, e um stage que acrescentaria um ficheiro sensível (uma chave privada, um `.env`, um token reconhecido) é recusado com `{error: "sensitive_files", files}`: o agente deve perguntar-lhe. `true` deixa-o passar `allow_sensitive`. Veja [Stage e commit](staging-and-committing.md#ficheiros-sensíveis). |
| `[mcp] allow_history_purge` | `false` | Só no `config.toml`. `true` lista a ferramenta `rewrite_purge`, que elimina de vez o histórico antigo de uma reescrita do histórico (e o respetivo desfazer). Desativado: o agente não a vê. Continua a precisar de `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | O interruptor geral da sua janela (Definições > Agentes e MCP > *Permitir que agentes de IA controlem a vista*). Desligado: nenhum agente alcança a sua janela. |
| `[ui] launch` | `"off"` | Se um agente pode abrir uma janela do GitPoulpi quando nenhuma está aberta (consulte [abaixo](#quando-não-há-nenhuma-janela-aberta-ou-está-noutra-sessão)). |

O que isto significa na prática:

- **Por predefinição, o agente é só de leitura**: pode ler, mostrar-lhe coisas na
  sua janela (`ui_*`) e, se permitir, propor alterações que confirma. Para o deixar
  alterar o repositório (commit, push, reset...) defina `allow_mutations = true`;
  para uma concessão estreita mantenha-o `true` com uma `mutation_allowlist`. Cada ferramenta
  leva anotações MCP (`readOnlyHint`, `destructiveHint`, `openWorldHint`),
  por isso um cliente como o Claude Code continua a perguntar-lhe antes das arriscadas.
- **Conflitos**: `conflicts_check` (uma previsão, antes de qualquer merge), `conflicts_list`, `conflicts_show` e `conflicts_state` são
  leituras; `conflicts_resolve` (descarta o lado que não é mantido, `--delete` remove
  o ficheiro) e `conflicts_abort` (descarta todo o merge/rebase/cherry-pick) são
  destrutivas; `conflicts_unresolve` é uma escrita simples. Uma chamada recusada (`stale`,
  `not_conflicted`, `pick_incomplete`...) responde `{"refused": {...}}` com código
  de saída 2 em vez de falhar.
- **Estados bloqueados**: `conflicts_am_continue` é uma escrita, `conflicts_am_skip`,
  `conflicts_am_abort` e `index_lock_remove` são destrutivas, `index_lock_show`
  é uma leitura.
- **Reescrita do histórico**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (a análise de tamanhos), `rewrite_scan_secrets` (resultados com uma pré-visualização ocultada, nunca um valor),
  `rewrite_script_check`, `rewrite_script_test` e
  `rewrite_discard` são leituras (uma pré-visualização não altera nada; só guarda um plano
  no daemon e em disco, na pasta git, pelo que `rewrite_apply` também funciona depois de um reinício). `rewrite_apply` e
  `rewrite_run` são destrutivas: movem muitas refs para commits reescritos como uma só
  entrada de desfazer (`undo` repõe-nas) e nunca enviam. Passe a receita como
  `recipe_json` (ou `recipe`, um caminho) e os segredos como `secrets_file`, nunca em linha.
  Os scripts de shell, Python e Lua externo (`script_language`) não correm em sandbox
  e **um agente não pode confiar neles**: não existe a ferramenta `rewrite_script_trust`
  nem o argumento `trust`, por isso um script destes só corre depois de uma pessoa executar
  `gitpoulpi rewrite script trust FICHEIRO` sobre esse texto exato (um script alterado é
  recusado de novo, `script_not_trusted` com o hash em `refused.script`), e só quando
  `rewrite.external_scripts` ativa a respetiva linguagem (`scripts_disabled`).
  Uma recusa responde `{"refused": {"reason", "message", "detail"}}` e um plano que
  move refs já enviadas responde `{"needs_confirmation": ...}` (código de saída 3) até a
  chamada definir `yes` - mostre-o primeiro ao utilizador.
- **Nunca acessível por um agente**: guardar um token (`hosting login`), criar
  e enviar uma chave SSH, `config set` (poderia reescrever os seus próprios limites), e
  o ciclo de vida do daemon. Um token nunca é mostrado ao agente.
- **Listas de toda a conta**: `hosting_radar` é uma leitura (pedidos à espera da sua
  revisão, pedidos abertos, issues atribuídas em todos os repositórios de um perfil,
  a partir de uma cache de cinco minutos salvo com `refresh`); `hosting_integrations_radar`
  altera a tabela `[integrations.radar]` de um perfil, por isso `allow_mutations` limita-a
  (mesmo quando chamada sem opções, o que apenas mostra as definições).
- **Contexto da revisão**: `ai_review_context` mostra o que um rascunho de revisão de código recebe
  além do diff (CLAUDE.md, AGENTS.md, um ficheiro extra, onde um agente local é executado). Com
  um argumento de definição guarda uma predefinição (global, ou a própria do `repo` ou do `project`),
  por isso `allow_mutations` limita-a, como as outras ferramentas que alteram uma definição; sem nenhum
  apenas mostra. Como toda a ferramenta `ai_*`, só é listada enquanto `ai.enabled` está ligado.
  `hosting_review_suggest` aceita os mesmos argumentos apenas para uma execução.
- **As barreiras `[mcp]` aplicam-se ao servidor MCP.** A CLI é a sua própria shell e
  não é limitada por `allow_mutations`; apenas `ui_control` e `allow_ui_propose`
  se aplicam também a `gitpoulpi ui ...`.
- Uma chamada recusada é um erro de ferramenta normal (`isError: true`, o motivo em
  `structuredContent.error`), nunca uma não-operação silenciosa.

```toml
[mcp]
allow_mutations = false        # true: o agente pode alterar o repositório
mutation_allowlist = []        # p. ex. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" deixa o agente mover a sua vista sem perguntar
allow_ui_propose = false       # true: o agente pode pré-preencher os diálogos de Reword / Split / commit / branch / tag
allow_sensitive_staging = false  # true: o agente pode fazer stage dos ficheiros que a proteção de stage assinala
allow_history_purge = false    # true: o agente vê rewrite_purge (elimina o histórico antigo de vez)
```

## As ferramentas

Uma ferramenta por comando da CLI, com o nome do seu caminho com `_` (`branch create` é
`branch_create`). Cada uma recebe os argumentos do comando mais `repo` (o repositório;
por omissão: o do servidor) e `session`. O `structuredContent` de uma chamada bem-sucedida é
exatamente o que `gitpoulpi --json <command>` imprime. **Kind**: *read* nunca altera
nada; *write* e *destructive* são mutações (limitadas); *network* comunica com um
remoto ou um host; *view* move apenas a sua janela; *propose* abre um diálogo que
confirma.

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
| `gitpoulpi flow trust-hooks` | trusting the shell commands a repository commits is a person's decision; an agent can only run hooks a person trusted |
<!-- mcp-tools:end -->

### As ações dos fluxos de trabalho de branches são planear e depois executar

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` e `flow_set_parent`
([Fluxos de trabalho de branches](branching-workflows.md#ações)) nunca executam numa chamada
simples. `dry_run: true` devolve o plano (cada passo e comando de git, avisos,
o que o Desfazer não consegue reverter, bloqueios) como resultado normal; `yes: true` executa-o;
sem nenhum dos dois, a chamada é recusada e traz o plano. Os `[hooks]` do ficheiro do
fluxo de trabalho são comandos de shell de um ficheiro que a equipa partilha, por isso só executam com
um `run_hooks: true` explícito: o plano lista-os (`hooks`, `hooks_run`) para que possa
mostrar primeiro ao utilizador os comandos exatos. Um merge que pára em conflitos
responde `status: "conflicted"` (um resultado de erro, código de saída 2); depois de as
ferramentas de conflito resolverem os ficheiros, chame a mesma ferramenta com `continue_: true`
ou `abort: true`. Abrir um merge request (`finish` no modo `merge-request`)
não é desfazível e o plano di-lo.

### O que um agente faz com o `git` simples também está no histórico de desfazer

Um agente que cria branches e worktrees, faz commits, resets ou descarta stashes
com o seu próprio `git` (e não através destas ferramentas) continua visível: o
daemon compara as refs do repositório com o que viu da última vez e regista cada
alteração como uma entrada de `undo --list` com `"source": "outside"`, o motivo
do git (`outside.message`) e o committer que o git registou. As ferramentas
`undo`, `redo` e `undo_restore` tratam estas entradas como quaisquer outras, pelo
que um agente (ou o utilizador, com `Ctrl+Z`) pode repor uma branch que foi
eliminada. As edições a ficheiros nunca são capturadas. Veja
[Desfazer e refazer](undo-redo.md#alterações-feitas-fora-do-gitpoulpi).

## Mostrar coisas ao utilizador

Um agente pode apontar para algo em vez de o descrever: `ui_show_commit`,
`ui_show_diff` (um ficheiro numa linha, inline ou dividido), `ui_show_file` (conteúdo,
blame ou histórico numa linha), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`,
`ui_show_conflict` (o editor de merge num ficheiro em conflito, num conflito),
`ui_show_wip` (`composer` põe o cursor no resumo do commit, sem escrever nada),
`ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette` (a consulta
escrita, nunca executada), `ui_show_keys`, `ui_show_settings` (uma página e uma definição,
nunca editada), `ui_highlight` (colore linhas do diff ou ficheiro aberto),
`ui_notify` (um toast) e `ui_back` (desfaz a última mudança de vista do agente).
`ui_state` lê o que uma janela mostra (`text: true` acrescenta o texto visível);
`ui_viewers` lista as janelas.

Estas ferramentas **nunca alteram o repositório** e não podem escrever, clicar nem editar uma
definição. As revisões e os caminhos são verificados antes de qualquer coisa chegar à janela.

**O que o utilizador vê e decide**:

- `ask` (predefinição), ou sempre que está a escrever, um diálogo ou as Definições estão abertos, está
  a arrastar, ou premiu uma tecla nos últimos 1,5 s: o agente recebe
  `deferred` de imediato e a sua barra de estado mostra um chip *"claude-code quer mostrar
  ..."*. `Ctrl+N Enter` ou um clique mostra-o; `Esc` ou `Ctrl+N Backspace`
  dispensa-o. Expira após 60 s.
- Aplicado: a vista move-se, um toast de 4 s diz quem a moveu, e o separador fica com
  um `✦` roxo até à sua próxima tecla. `Ctrl+N Backspace` (paleta `Agente: voltar`)
  volta atrás; a janela memoriza dez vistas.
- Nada é alguma vez reexibido ou feito checkout: um commit que o seu filtro solo oculta
  responde `partial` (`hidden_by_filter`); uma lista de PR ou issues que não está ligada a
  um fornecedor responde `hosting_not_linked`.

**Respostas**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, com o `state` da janela quando
aplicado. Códigos de saída da CLI: 0 applied/partial/proposed, 3 deferred, 1 nos restantes casos.
As chamadas têm limite de taxa (rajada de 5, depois uma por segundo).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Propor uma alteração (`ui_propose_*`)

Com `allow_ui_propose = true`, um agente pode abrir um dos seus diálogos
**pré-preenchido**, marcado *✦ Proposto por <agente>*. Nada acontece até premir
o próprio botão de confirmar desse diálogo; pode editar primeiro ou cancelar.

| Ferramenta | Abre | Confirma com |
|---|---|---|
| `ui_propose_split` (rev, plan file) | o editor de divisão sobre o commit, preenchido a partir do plano | *Dividir* (`Ctrl+S`); `Esc` rejeita |
| `ui_propose_reword` (rev, message) | o popup Reword | *Atualizar mensagem* (um commit já enviado ainda pergunta sobre force-push) |
| `ui_propose_commit_message` (summary, body) | o formulário de commit, preenchido; nunca faz commit | o botão de Commit |
| `ui_propose_branch` (name, start point) | a caixa Nova branch inline | `Enter` |
| `ui_propose_tag` (name, target, message) | a caixa Nova tag inline (anotada quando há mensagem) | `Enter` |
| `ui_propose_rewrite` (recipe file) | o painel de reescrita do histórico com as regras da receita; nada é pré-visualizado nem aplicado | o seu próprio *Pré-visualizar* e depois *Aplicar* (`Ctrl+Enter`); `Esc` rejeita. Uma receita com um script é recusada, os valores secretos são retirados |
| `ui_propose_resolution` (path, pick or content file) | o editor de merge num ficheiro em conflito, com as escolhas (ou o texto todo) já no Resultado | *Guardar e marcar resolvido* (`Ctrl+S`); fechá-lo rejeita |

Uma proposta de cada vez; uma proposta de mensagem de commit é recusada enquanto tiver o seu
próprio rascunho no compositor, e uma proposta de resolução enquanto o editor de merge guardar
trabalho seu por guardar ou quando as fases do ficheiro mudaram desde que o agente as leu.
Uma resolução que altere antes de confirmar é comunicada como `accepted_edited`. A resposta tem um id `proposal`; o que decidiu está em
`proposals` de `ui_state` (`offered`, `open`, `accepted`, `accepted_edited`,
`rejected`, `expired`, `failed`).

### Quando não há nenhuma janela aberta, ou está noutra sessão

- Uma janela pertence à sessão em que foi iniciada. Se a sessão do agente não tem
  nenhuma, `ui show` procura **uma** janela noutra sessão que tenha o
  repositório aberto e usa-a (`routed_session` na resposta). Se várias correspondem:
  `no_viewer` a nomeá-las; passe `--session`.
- **Abrir uma janela para o agente** está desligado por omissão. Defina
  `[ui] launch` (Definições > Agentes e MCP > *Abrir uma janela para agentes*) para uma predefinição de terminal ou
  multiplexador ou um comando `custom`; então uma chamada com `launch: true`
  (CLI `gitpoulpi ui --launch show ...`) abre uma e espera até 10 s. Com
  `ui_control = "auto"` também é aberta sem a flag; com `ask` o agente
  recebe `no_viewer` e deve avisá-lo.

| Predefinição | Comando |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (o seu terminal predefinido) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (nova janela), `zellij` (novo painel), `cmux` (novo workspace) | dentro de um multiplexador em execução, na pasta do repositório |
| `custom` | `[ui] launch_command`, por exemplo `my-term --cd {repo} {gitpoulpi} --session {session}`; dividido em argumentos, sem shell |

Se uma predefinição não funcionar com a sua versão do programa, use `custom`.

## Erros

- Uma chamada malformada (ferramenta ou argumento desconhecido, falta um obrigatório) é um
  erro JSON-RPC `-32602 invalid_params`.
- Uma recusa, ou um erro de git ou do daemon (revisão inválida, conflito, ...), é um erro
  de ferramenta: `isError: true` com a mensagem em `content` e
  `structuredContent.error`.
