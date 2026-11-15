[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP e agentes

Um agente de IA (Claude Code, Cursor, Codex, ...) pode controlar o GitPoulpi pelo seu
**servidor MCP** ou pela sua **CLI**. Ambos passam pelo mesmo daemon em segundo plano que a
janela, então o que o agente faz aparece ao vivo no seu GitPoulpi aberto, e ambos executam
exatamente o mesmo código para cada comando.

Esta página trata de agentes *externos*. Os botões de IA do próprio GitPoulpi (mensagens de commit,
explicações, o compositor de commits) são configurados separadamente: veja
[Assistência de IA](ai.md).

## Configuração rápida

1. Coloque `gitpoulpi` e `gitpoulpi-mcp` no seu `PATH` ([Instalação](installation.md)).
2. Registre o servidor no seu agente. Ele fala stdio: nada para iniciar, nenhuma porta.
3. Opcionalmente, escolha o que o agente pode fazer na tabela `[mcp]` do `config.toml`
   (abaixo). Os padrões deixam que ele leia o repositório (alterações são recusadas até você definir
      `allow_mutations = true`), mostre coisas a você
   somente depois que você confirmar, e nunca preencha diálogos antecipadamente.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` o grava no `.mcp.json` do repositório, para que todos
que abrirem o projeto o recebam. O mesmo arquivo manualmente, com os extras opcionais:

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

- `--ui-control auto|ask|off` substitui `[mcp] ui_control` somente para este agente.
- `GITPOULPI_CONFIG_DIR` dá ao agente seu próprio `config.toml` (seus próprios limites de
  `[mcp]`, sem acesso às suas configurações). Deixe de fora para compartilhar as suas.
- `gitpoulpi --repo <path> mcp serve --stdio` fixa o servidor em um repositório;
  caso contrário, uma ferramenta age no repositório em torno do diretório de trabalho do servidor,
  ou no argumento `repo` da chamada.

**Outros clientes**: qualquer cliente MCP que inicie um servidor stdio.

| Cliente | Onde | O quê |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | o arquivo JSON de MCP do cliente | o bloco `mcpServers` acima (o VS Code chama a chave de `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Qualquer outro | a sua documentação | o comando `gitpoulpi-mcp` (binário independente, mesmo servidor, sem argumentos necessários) |

**Por HTTP** (clientes que se conectam a uma URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, padrão http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

JSON sem estado, as mesmas ferramentas, as mesmas travas de `[mcp]`. Toda requisição precisa do token
bearer (`[mcp] http_auth`, ativado por padrão): imprima-o com `gitpoulpi mcp token
--show` (ou `--copy`, `--rotate`; o token fica em um arquivo só do usuário na pasta
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

Um token ausente ou errado recebe `401` com `WWW-Authenticate: Bearer`. Um token
substituído vale na hora, sem reiniciar o servidor. Desativar `http_auth` serve para um
endpoint loopback em que você confia em todas as contas locais; o servidor então avisa no
stderr.

### Acessando o endpoint de outra máquina

A configuração mais segura não precisa de nenhum listener remoto: mantenha o endereço loopback e
encaminhe a porta por SSH a partir da máquina que executa o agente, `ssh -L
8808:127.0.0.1:8808 host`. Assim o `--allow-remote` nem entra em cena.

`--allow-remote` (um endereço que não seja loopback) é recusado, a menos que a autenticação esteja ativada
com um token definido **e** o tráfego seja criptografado. A recusa nomeia o que
falta. Há duas formas de ter criptografia:

- **TLS no servidor** (`[mcp] tls = "self_signed"` ou `"files"`). O servidor
  encerra o TLS por conta própria: ele guarda a chave, executa o handshake, e só um
  handshake concluído chega à camada HTTP; um cliente HTTP simples falha no handshake e não existe
  listener simples ao lado. Então a criptografia é um fato que o servidor
  estabelece em cada conexão, não uma suposição. Com `self_signed`, a
  primeira inicialização gera um par na pasta de dados privada e imprime a impressão digital
  SHA-256 dele (também nas Configurações); um cliente confia no servidor fixando
  essa impressão digital ou o arquivo do certificado. Com `files`, defina `tls_cert` e
  `tls_key` para os seus arquivos PEM e os clientes verificam o certificado pela CA dele.
  Os clientes se autenticam com o token bearer; certificados de cliente não são usados.
- **Um túnel ou proxy reverso** (`[mcp] behind_proxy = true`). Um agente de túnel ou
  proxy se conecta como qualquer cliente local ou da LAN e seus cabeçalhos podem ser forjados, então
  o servidor não consegue saber de dentro se há um. Por isso isto é um
  reconhecimento explícito (mostrado com um aviso vermelho nas Configurações) em vez de uma
  detecção. Execute o túnel na mesma máquina e mantenha o endereço loopback quando
  puder. Com um endereço que não seja loopback atrás de um proxy, o trecho entre o proxy
  e o servidor é texto simples nessa rede.

**Sem MCP**, um agente com um shell usa a CLI: `gitpoulpi --json <command>`
(veja a [referência da CLI](cli-reference.md)). Dê a ele a skill pronta
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), que diz quando
e como usar o GitPoulpi em vez do `git` puro.

## Permissões

| Chave | Padrão | Efeito |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` se recusam a iniciar (Configurações > Agentes e MCP > *Servidor MCP*). Lido quando o servidor inicia: um servidor em execução mantém suas configurações. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Endereço que `--http` usa quando nenhum é informado (somente loopback, a menos que `--allow-remote`, que exige autenticação e criptografia, veja acima). Configurações > Agentes e MCP > *Porta do servidor MCP* edita a porta e mantém o host. |
| `[mcp] http_auth` | `true` | `--http` exige `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Configurações > Agentes e MCP > *Autenticação HTTP*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` ou `files` (os seus caminhos PEM): o servidor encerra o TLS por conta própria. Configurações > Agentes e MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Você declara que um túnel ou proxy reverso criptografa o tráfego, o que deixa `--allow-remote` funcionar sem TLS. |
| `[mcp] allow_mutations` | `false` | Desligado: toda ferramenta que altera algo é recusada; leituras, `ui_*` e propostas continuam funcionando. `true` deixa o agente fazer commit, push, reset... |
| `[mcp] mutation_allowlist` | `[]` | Não vazia: somente estas ferramentas podem alterar coisas, por exemplo `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: o agente não pode mexer na sua janela (`ui_state` / `ui_viewers` ainda respondem). |
| `[mcp] ui_control` | `"ask"` | `ask`: um chip na sua janela que você confirma; `auto`: aplicado, a menos que você esteja ocupado; `off`: recusado. Por agente: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Deixa o agente abrir diálogos pré-preenchidos que você confirma (Configurações > Agentes e MCP > *Deixar agentes de IA proporem alterações para eu confirmar*). |
| `[mcp] allow_sensitive_staging` | `false` | Somente no `config.toml`. Desativado, a ferramenta `stage` não oferece `allow_sensitive`, e um stage que adicionaria um arquivo sensível (uma chave privada, um `.env`, um token reconhecido) é recusado com `{error: "sensitive_files", files}`: o agente deve perguntar a você. `true` deixa-o passar `allow_sensitive`. Veja [Stage e commit](staging-and-committing.md#arquivos-sensíveis). |
| `[mcp] allow_history_purge` | `false` | Somente no `config.toml`. `true` lista a ferramenta `rewrite_purge`, que apaga de vez o histórico antigo de uma reescrita de histórico (e o desfazer dela). Desativado: o agente não a vê. Ela ainda exige `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | A chave geral da sua janela (Configurações > Agentes e MCP > *Permitir que agentes de IA controlem a visualização*). Desligada: nenhum agente alcança a sua janela. |
| `[ui] launch` | `"off"` | Se um agente pode abrir uma janela do GitPoulpi quando nenhuma está aberta (veja [abaixo](#quando-nenhuma-janela-está-aberta-ou-ela-está-em-outra-sessão)). |

O que isto significa na prática:

- **De fábrica o agente é somente leitura**: ele pode ler, mostrar coisas a você na
  sua janela (`ui_*`) e, se você permitir, propor alterações que você confirma. Para deixar
  que ele altere o repositório (commit, push, reset...) defina `allow_mutations = true`;
  para uma concessão restrita, mantenha-o `true` com uma `mutation_allowlist`. Toda ferramenta
  carrega anotações MCP (`readOnlyHint`, `destructiveHint`, `openWorldHint`),
  então um cliente como o Claude Code ainda pergunta a você antes das arriscadas.
- **Conflitos**: `conflicts_check` (uma previsão, antes de qualquer merge), `conflicts_list`, `conflicts_show` e `conflicts_state` são
  leituras; `conflicts_resolve` (ele descarta o lado não mantido, `--delete` remove
  o arquivo) e `conflicts_abort` (descarta o merge/rebase/cherry-pick inteiro) são
  destrutivas; `conflicts_unresolve` é uma escrita simples. Uma chamada recusada (`stale`,
  `not_conflicted`, `pick_incomplete`...) responde `{"refused": {...}}` com código de
  saída 2 em vez de falhar.
- **Conversas de issues e de pull requests**: `hosting_issue_comments` e
  `hosting_review_conversation` leem a linha do tempo inteira (leituras, mundo aberto:
  chamam o host); `hosting_markdown` limpa um texto e encontra as referências dele e não
  chama nada (mundo fechado). Publicar (`hosting_issue_comment`,
  `hosting_review_comment`, `hosting_review_reply`) e editar
  (`hosting_*_comment_edit`) são mutações simples marcadas como mundo aberto: vão para
  o host na hora, com a identidade que o token tem (`posted_as` na resposta; um token
  de bot publica como o bot), então `allow_mutations` e `mutation_allowlist`
  decidem se um agente pode. Excluir um comentário (`hosting_*_comment_delete`) é
  destrutivo. Depois de publicar, editar ou excluir, o daemon é avisado, e uma
  janela aberta do repositório atualiza esse item. `ui_show_pr` / `ui_show_issue`
  aceitam `--view conversation` para abrir a conversa no lugar da linha.
- **Estados travados**: `conflicts_am_continue` é uma escrita, `conflicts_am_skip`,
  `conflicts_am_abort` e `index_lock_remove` são destrutivas, `index_lock_show`
  é uma leitura.
- **Reescrita de histórico**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (a análise de tamanho), `rewrite_scan_secrets` (achados com
  uma prévia ocultada, nunca um valor),
  `rewrite_script_check`, `rewrite_script_test` e `rewrite_discard` são leituras
  (uma prévia não altera nada; só guarda um plano no daemon e em disco, no
  diretório git, então `rewrite_apply` também funciona após um reinício).
  `rewrite_apply` e `rewrite_run` são destrutivas: movem muitas refs para commits
  reescritos como uma única entrada de desfazer (`undo` as devolve) e nunca fazem
  push. Passe a receita como `recipe_json` (ou `recipe`, um caminho) e os segredos
  como `secrets_file`, nunca inline. Scripts de shell, Python e Lua externo
  (`script_language`) não são isolados e **um agente não pode confiar neles**:
  não existe a ferramenta `rewrite_script_trust` nem o argumento `trust`, então
  um script assim só roda depois que uma pessoa executou
  `gitpoulpi rewrite script trust ARQUIVO` sobre esse texto exato (um script
  alterado é recusado de novo, `script_not_trusted` com o hash em
  `refused.script`), e só quando `rewrite.external_scripts` ativa a linguagem
  dele (`scripts_disabled`). Uma recusa responde
  `{"refused": {"reason", "message", "detail"}}` e um plano que move refs já
  publicadas responde `{"needs_confirmation": ...}` (código de saída 3) até a
  chamada definir `yes` - mostre-o ao usuário antes.
- **Nunca acessíveis por um agente**: guardar um token (`hosting login`), criar
  e enviar uma chave SSH, `config set` (ele poderia reescrever os próprios limites) e
  o ciclo de vida do daemon. Um token nunca é mostrado ao agente.
- **Listas de toda a conta**: `hosting_radar` é uma leitura (solicitações que aguardam a sua
  revisão, solicitações abertas, issues atribuídas, em todos os repositórios de um perfil,
  a partir de um cache de cinco minutos, a menos que haja `refresh`); `hosting_integrations_radar`
  altera a tabela `[integrations.radar]` de um perfil, então `allow_mutations` a controla
  (mesmo chamada sem nenhuma opção, que só imprime as configurações).
- **Contexto da revisão**: `ai_review_context` mostra o que um rascunho de revisão de código recebe
  além do diff (CLAUDE.md, AGENTS.md, um arquivo extra, onde um agente local roda). Com
  um argumento de configuração, salva um padrão (global, ou o próprio do `repo` ou do `project`),
  então `allow_mutations` a controla, como as outras ferramentas que alteram uma configuração; sem
  nenhum, só imprime. Como toda ferramenta `ai_*`, só é listada enquanto `ai.enabled` está ativado.
  `hosting_review_suggest` aceita os mesmos argumentos, só para uma execução.
- **As travas de `[mcp]` valem para o servidor MCP.** A CLI é o seu próprio shell e
  não é limitada por `allow_mutations`; somente `ui_control` e `allow_ui_propose` também
  valem para `gitpoulpi ui ...`.
- Uma chamada recusada é um erro de ferramenta comum (`isError: true`, o motivo em
  `structuredContent.error`), nunca uma operação nula silenciosa.

```toml
[mcp]
allow_mutations = false        # true: o agente pode alterar o repositório
mutation_allowlist = []        # e.g. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" deixa o agente mudar a sua visualização sem perguntar
allow_ui_propose = false       # true: o agente pode preencher antes os diálogos de Reword / Split / commit / branch / tag
allow_sensitive_staging = false  # true: o agente pode fazer stage de arquivos que a proteção de stage sinaliza
allow_history_purge = false    # true: o agente vê rewrite_purge (apaga o histórico antigo de vez)
```

## As ferramentas

Uma ferramenta por comando da CLI, nomeada a partir do seu caminho com `_` (`branch create` é
`branch_create`). Cada uma recebe os argumentos do comando mais `repo` (o repositório;
padrão: o do servidor) e `session`. O `structuredContent` de uma chamada bem-sucedida é
exatamente o que `gitpoulpi --json <command>` imprime. **Kind**: *read* nunca altera
nada; *write* e *destructive* são mutações (limitadas); *network* fala com um
remoto ou um host; *view* só mexe na sua janela; *propose* abre um diálogo que você
confirma.

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

### As ações de fluxo de trabalho de branches são planejar-e-depois-executar

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` e `flow_set_parent`
([Fluxos de trabalho de branches](branching-workflows.md#ações)) nunca são executadas em uma
chamada simples. `dry_run: true` retorna o plano (cada passo e comando do git, avisos,
o que o Desfazer não consegue reverter, bloqueios) como um resultado normal; `yes: true` o executa;
sem nenhum dos dois, a chamada é recusada e traz o plano. Os `[hooks]` do arquivo do fluxo de trabalho
são comandos de shell de um arquivo que a equipe compartilha, então só são executados com
um `run_hooks: true` explícito: o plano os lista (`hooks`, `hooks_run`) para que você
possa mostrar ao usuário os comandos exatos antes. Um merge que para em conflitos
responde `status: "conflicted"` (um resultado de erro, código de saída 2); depois que as
ferramentas de conflito resolverem os arquivos, chame a mesma ferramenta com `continue_: true`
ou `abort: true`. Abrir um merge request (`finish` no modo `merge-request`)
não é desfazível e o plano avisa.

### O que um agente faz com `git` puro também está no histórico de desfazer

Um agente que cria branches e worktrees, faz commits, resets ou descarta stashes
com o seu próprio `git` (sem passar por estas ferramentas) continua visível: o
daemon compara as refs do repositório com o que viu por último e registra cada
alteração como uma entrada de `undo --list` com `"source": "outside"`, o motivo do
git (`outside.message`) e o committer que o git registrou. As ferramentas `undo`,
`redo` e `undo_restore` tratam essas entradas como quaisquer outras, então um
agente (ou o usuário, com `Ctrl+Z`) pode trazer de volta uma branch que foi
excluída. Edições em arquivos nunca são capturadas. Veja
[Desfazer e refazer](undo-redo.md#alterações-feitas-fora-do-gitpoulpi).

## Mostrar coisas ao usuário

Um agente pode apontar para algo em vez de descrevê-lo: `ui_show_commit`,
`ui_show_diff` (um arquivo em uma linha, inline ou dividido), `ui_show_file` (conteúdo,
blame ou histórico em uma linha), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`,
`ui_show_conflict` (o editor de merge em um arquivo em conflito, em um conflito),
`ui_show_wip` (`composer` coloca o cursor no resumo do commit, sem digitar nada),
`ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette` (a consulta
digitada, nunca executada), `ui_show_keys`, `ui_show_settings` (uma página e uma opção,
nunca editada), `ui_highlight` (colore linhas do diff ou do arquivo aberto),
`ui_notify` (um toast) e `ui_back` (desfaz a última mudança de visualização do agente).
`ui_state` lê o que uma janela mostra (`text: true` acrescenta o texto visível);
`ui_viewers` lista as janelas.

Estas ferramentas **nunca alteram o repositório** e não podem digitar, clicar nem editar
uma configuração. Revisões e caminhos são verificados antes de qualquer coisa chegar à janela.

**O que o usuário vê e decide**:

- `ask` (padrão), ou sempre que você estiver digitando, um diálogo ou as Configurações estiverem abertos, você
  estiver arrastando, ou tiver pressionado uma tecla no último 1,5 s: o agente recebe
  `deferred` na hora e a sua linha de status mostra um chip *“claude-code quer mostrar
  ...”*. `Ctrl+N Enter` ou um clique o mostra; `Esc` ou `Ctrl+N Backspace`
  o dispensa. Ele expira após 60 s.
- Aplicado: a visualização se move, um toast de 4 s diz quem a moveu, e a aba ganha um
  `✦` roxo até a sua próxima tecla. `Ctrl+N Backspace` (paleta `Agente: voltar`)
  volta; a janela lembra dez visualizações.
- Nada é jamais reexibido nem recebe checkout: um commit que o seu filtro solo oculta
  responde `partial` (`hidden_by_filter`); uma lista de PRs ou issues que não está vinculada
  a um provedor responde `hosting_not_linked`.

**Respostas**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
“not_found” | “no_viewer” | “proposed”, ...}`, com o `state` da janela quando
aplicado. Códigos de saída da CLI: 0 applied/partial/proposed, 3 deferred, 1 nos demais casos.
As chamadas têm limite de taxa (rajada de 5, depois uma por segundo).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Propor uma alteração (`ui_propose_*`)

Com `allow_ui_propose = true`, um agente pode abrir um dos seus diálogos
**pré-preenchido**, marcado com *✦ Proposto por <agent>*. Nada acontece até você pressionar
o próprio botão de confirmação desse diálogo; você pode editar antes ou cancelar.

| Ferramenta | Abre | Você confirma com |
|---|---|---|
| `ui_propose_reword` (rev, message) | o popup Reescrever mensagem do commit | *Atualizar mensagem* (um commit já enviado ainda pergunta sobre force push) |
| `ui_propose_commit_message` (summary, body) | o formulário de commit, preenchido; nunca faz commit | o botão Commit |
| `ui_propose_branch` (name, start point) | a caixa embutida Nova branch | `Enter` |
| `ui_propose_tag` (name, target, message) | a caixa embutida Nova tag (anotada quando há uma mensagem) | `Enter` |
| `ui_propose_split` (rev, plan file) | o editor de divisão sobre o commit, preenchido a partir do plano | *Dividir* (`Ctrl+S`); `Esc` rejeita |
| `ui_propose_resolution` (path, pick or content file) | o editor de merge em um arquivo em conflito, com as escolhas (ou o texto inteiro) já no Resultado | *Salvar e marcar resolvido* (`Ctrl+S`); fechá-lo rejeita |
| `ui_propose_rewrite` (arquivo de receita) | o painel de reescrita de histórico com as regras da receita; nada é pré-visualizado nem aplicado | o seu próprio *Prévia* e depois *Aplicar* (`Ctrl+Enter`); `Esc` rejeita. Uma receita com script é recusada e os valores secretos são removidos |

Uma proposta por vez; uma proposta de mensagem de commit é recusada enquanto você tiver o seu
próprio rascunho no compositor, e uma proposta de resolução enquanto o editor de merge guardar
trabalho seu não salvo ou quando os stages do arquivo mudaram desde que o agente os leu.
Uma resolução que você altera antes de confirmar é informada como `accepted_edited`. A resposta tem um id de `proposal`; o que você decidiu está em
`proposals` de `ui_state` (`offered`, `open`, `accepted`, `accepted_edited`,
`rejected`, `expired`, `failed`).

### Quando nenhuma janela está aberta, ou ela está em outra sessão

- Uma janela pertence à sessão em que foi iniciada. Se a sessão do agente não tem
  nenhuma, `ui show` procura **uma** janela em outra sessão que tenha o
  repositório aberto e a usa (`routed_session` na resposta). Se várias correspondem:
  `no_viewer` nomeando-as; informe `--session`.
- **Abrir uma janela para o agente** vem desligado por padrão. Defina
  `[ui] launch` (Configurações > Agentes e MCP > *Abrir uma janela para agentes*) como um preset de terminal ou
  multiplexador ou um comando `custom`; então uma chamada com `launch: true`
  (CLI `gitpoulpi ui --launch show ...`) abre uma e espera até 10 s. Com
  `ui_control = "auto"` ela também é aberta sem a flag; com `ask` o agente
  recebe `no_viewer` e deve avisar você.

| Preset | Comando |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (o seu terminal padrão) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (nova janela), `zellij` (novo painel), `cmux` (novo workspace) | dentro de um multiplexador em execução, na pasta do repositório |
| `custom` | `[ui] launch_command`, por exemplo `my-term --cd {repo} {gitpoulpi} --session {session}`; dividido em argumentos, sem shell |

Se um preset não funcionar com a sua versão do programa, use `custom`.

## Erros

- Uma chamada malformada (ferramenta ou argumento desconhecido, um obrigatório ausente) é um
  erro JSON-RPC `-32602 invalid_params`.
- Uma recusa, ou um erro do git ou do daemon (revisão inválida, conflito, ...), é um erro
  de ferramenta: `isError: true` com a mensagem em `content` e
  `structuredContent.error`.
