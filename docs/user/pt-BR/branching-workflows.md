[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · [Italiano](../it/branching-workflows.md) · [日本語](../ja/branching-workflows.md) · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# Fluxos de trabalho de branches

Um fluxo de trabalho de branches diz ao GitPoulpi como a sua equipe usa as branches: quais
vivem para sempre (`main`, `develop`, `production`), quais são branches de tópico de vida
curta (`feature/...`, `hotfix/...`), como elas são nomeadas e onde cada uma começa e
termina. É o Git Flow, só que personalizável: nomes fixos, prefixos e sufixos, ou
padrões com marcadores, de modo que as branches que um bot cria a partir de uma issue
sejam reconhecidas como as suas.

Esta página cobre o modelo, o arquivo, os modelos prontos, a detecção e a importação, os
comandos somente leitura, as ações que iniciam, concluem, sincronizam e promovem branches
(CLI, MCP e aplicativo) e as telas do aplicativo: o editor de configurações, a barra
lateral agrupada, o menu `Flow` e os diálogos dele.

## Conceitos

- Uma **branch base** tem vida longa. Uma base pode ter um **pai** (`develop` entra em
  `main`); um tronco não tem. Uma base pode listar vários nomes candidatos
  (`main` ou `master`, `develop` ou `dev`): vale o que existir.
- Um **tipo de tópico** é um tipo de branch de vida curta (feature, bugfix, release,
  hotfix, support...). Ele tem um **padrão de nome**, um **pai** (de onde ele começa
  e para onde é concluído) e uma forma de ser concluído.
- Toda branch é **classificada**: uma base (pelo nome), um tópico de algum tipo (pelo
  padrão) ou sem classificação.
- O **pai de uma branch** é, em ordem: o registrado quando o GitPoulpi criou a
  branch (`git config branch.<name>.gitpoulpi-parent`), senão o
  `git-town-branch.<name>.parent` do Git Town quando existir, senão o pai do tipo dela.

## Onde ele é armazenado

| O quê | Onde | Com commit |
|---|---|---|
| O fluxo de trabalho compartilhado | `.gitpoulpi/workflow.toml` no repositório | Sim, por você: o GitPoulpi só o grava quando você pede |
| Os seus próprios ajustes | `[workflow]` em `.gitpoulpi/repo-state.toml` | Não (esse arquivo é ignorado) |

O diretório de estado se ignora sozinho (`.gitpoulpi/.gitignore`), exceto
`workflow.toml`, que é feito para receber commit. O fluxo de trabalho efetivo é o
arquivo compartilhado com a sua substituição por cima: uma base com o mesmo nome, ou um
tópico com o mesmo id, é substituído; os novos são adicionados; `disable_bases` e
`disable_topics` removem os compartilhados só para você.

Salvar por cima de um arquivo existente mantém as chaves que esta versão não conhece (as
de um GitPoulpi mais novo, as de um colega), mas não os comentários nem a ordem das chaves.

## O arquivo

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (informativo)

[[base]]
name = "main"
names = ["master"]             # outras grafias da mesma branch
protected = true               # nunca é excluída, commits diretos são suspeitos

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # uma branch de ambiente (GitLab Flow): promovida a jusante

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # mais nomes que também correspondem; nunca usados para criar
# regex = "^feat-(?P<slug>.+)$"  # também corresponde; grupos nomeados viram campos
parent = "develop"             # de onde ele começa, e o destino de conclusão padrão
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # comandos de shell opcionais, sempre mostrados antes de executar
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # fluxos conduzidos pelo host, veja abaixo
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode` e `hooks` conduzem as [ações](#ações) abaixo;
`issue_flow` conduz os [fluxos conduzidos pelo host](#fluxos-conduzidos-pelo-host).

### Padrões de nome

Texto com `{marcadores}`:

| Marcador | Corresponde a | Gerado a partir de |
|---|---|---|
| `{slug}` | texto livre; pode conter `/` quando é o último | o texto, em minúsculas, sem acentos, com `-` entre as palavras, no máximo 50 caracteres |
| `{issue}` | dígitos (`123`) | `123` ou `#123` |
| `{issue_key}` | uma chave de rastreador (`PROJ-123`) | como informada |
| `{user}` | um nome sem `/` | transformado em slug |
| `{date}` | `2026-10-16` ou `20261016` | como informada |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | como informada |
| `{n}` | dígitos | como informado |

Todo o resto é literal, então prefixos e sufixos são só texto:
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. Cada
marcador pode aparecer uma vez, e dois marcadores precisam de texto entre eles. Um
padrão precisa produzir nomes de branch válidos (as regras de `git check-ref-format`):
sem espaços, sem `~ ^ : ? * [ \`, sem `..`, sem componente que comece com `.` ou termine
com `.lock`.

Um `regex` serve só para corresponder (ele é ancorado nas duas pontas); use-o para nomes que
nenhum padrão consegue expressar. Os grupos nomeados viram os campos da branch.

### Qual tópico vence

1. Uma branch com o nome de uma base (qualquer um dos nomes dela) é essa base.
2. Senão, todo padrão de todo tópico que corresponde é um candidato: `pattern`,
   depois `patterns`, depois `regex`, tópico por tópico na ordem do arquivo. Vence o que
   fixa mais caracteres literais (`release/{version}` vence `{user}/{slug}`,
   que vence um `{slug}` sozinho); um regex conta o prefixo literal dele. Um empate vai para
   o que vem antes no arquivo.
3. Senão, a branch fica sem classificação.

Um padrão ou regex quebrado é informado como aviso e ignorado; o resto continua
funcionando.

## Modelos

| Modelo | Bases | Tópicos |
|---|---|---|
| `git-flow` | `main` ou `master`; `develop` ou `dev` (pai main) | `feature/` e `bugfix/` (a partir de develop), `release/` (a partir de develop, para main e develop, com tag), `hotfix/` (a partir de main, para main e develop, com tag), `support/` (a partir de main); as grafias no plural (`features/`, `releases/`...) também correspondem |
| `github-flow` | `main` | `feature/` ou qualquer nome, a partir de main, merge request para main, squash, excluída depois |
| `gitlab-flow` | `main`; `pre-production` e `production` (branches de ambiente, encadeadas main, pre-production, production) | `feature/` ou qualquer nome a partir de main; `release/{version}` como destino de cherry-pick |
| `trunk` | `main` | `{user}/{slug}` de vida curta a partir de main, com rebase |
| `custom` | nenhuma | nenhum |

As branches de ambiente também correspondem como `preprod`, `staging` e `prod`. Um modelo é
um ponto de partida: imprima-o com `flow config init` e edite o arquivo.

## Detecção e importação

`gitpoulpi flow detect` ordena os modelos pelas branches que o repositório tem,
com uma pontuação de 100 e os nomes concretos encontrados (`master` para `main`, `dev`
para `develop`, `features/` para `feature/`). As bases valem 60 pontos, a parcela das
outras branches que os padrões específicos do modelo reconhecem vale 40, e uma
branch de longa vida de outro modelo (um `develop` em um candidato GitHub Flow)
tira 25.

`gitpoulpi flow import --from git-flow` lê as configurações clássicas do git-flow
(`gitflow.branch.master`, `gitflow.branch.develop`,
`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`) do `.git/config` do
próprio repositório; `--from git-town` lê `git-town.main-branch` e
`git-town.perennial-branches` (o pai de cada branch,
`git-town-branch.<name>.parent`, é assumido como pai registrado). Nada é
aplicado em silêncio: o comando imprime o TOML, e `--write` cria
`.gitpoulpi/workflow.toml` só se ele não existir.

## Comandos

Todos aceitam `--json`; o servidor MCP os expõe como ferramentas (`flow_status`,
`flow_templates`, `flow_detect`, `flow_config_show` leem; as ações são
descritas abaixo).

| Comando | O que faz |
|---|---|
| `gitpoulpi flow status` | Cada branch local e remota: o tipo (base, tópico, sem classificação), o tipo de tópico e os campos, o pai e de onde ele veio (`recorded`, `git_town`, `type`, `base`) e, para as branches locais, quantos commits ela tem à frente (`ahead`) e atrás (`behind`) do pai. Também os avisos encontrados na definição, a detecção ordenada e as configurações de outra ferramenta que poderiam ser importadas. |
| `gitpoulpi flow templates` | Os modelos acima, com a definição completa. Não precisa de repositório. |
| `gitpoulpi flow detect` | Os modelos ordenados com as bases e os tópicos encontrados. |
| `gitpoulpi flow config [--show-effective]` | O arquivo compartilhado (o caminho, se existe, o fluxo de trabalho lido, as chaves desconhecidas) ou, com `--show-effective`, o fluxo de trabalho com que as suas branches são classificadas. `flow config show` é o mesmo. |
| `gitpoulpi flow config init --template <t> [--write]` | Imprime o TOML de um modelo; `--write` cria `.gitpoulpi/workflow.toml` e se recusa a sobrescrever um. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | Como descrito acima. |

As contagens de à frente / atrás são calculadas para no máximo 300 branches locais por chamada.

Um arquivo de fluxo de trabalho que não pode ser lido (TOML inválido, um valor com o tipo errado) é um erro
que cita o arquivo; nunca é ignorado em silêncio.

## Ações

`start`, `finish`, `sync`, `promote` e `set-parent` agem sobre o repositório. Cada
um primeiro **planeja**: lista cada passo de git que vai fazer, com o comando do git
a que cada um equivale, os avisos, o que não pode ser desfeito e qualquer coisa que o
bloqueie. Nada muda até você confirmar. Cada execução é **uma entrada no histórico de
desfazer** (`gitpoulpi undo` desfaz de uma vez as branches, a tag, a branch excluída,
o pai registrado e o checkout).

| Comando | O que faz |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | Dá nome à branch pelo padrão do tipo (`flow start feature "Login form"` cria `feature/login-form`; `flow start release version=1.2.0` cria `release/1.2.0`; uma palavra solta é o `slug`; `user` e `date` assumem por padrão você e hoje), a cria a partir do pai do tipo ou de `--from` (uma branch, uma branch remota ou uma tag), faz checkout dela e registra o pai. `--publish` faz push dela com `-u` (o `undo` não desfaz isso). |
| `gitpoulpi flow finish [<branch>]` | Para cada destino de `finish` do tipo, em ordem: faz checkout do destino e merge da branch com a estratégia do destino (`merge`: sempre um merge commit; `squash`: um commit com o título de `--title` ou tirado do nome da branch; `rebase`: rebase da branch sobre o destino e depois fast-forward; `fast-forward`), cria a tag quando o destino tem uma (`tag = "v{version}"`, anotada), depois faz checkout do pai e exclui a branch local quando o tipo manda (`--delete` / `--keep` o substituem; `--delete-remote` também a exclui no remoto, não desfazível). Com `finish_mode = "merge-request"` (ou `--mode merge-request`) ele só faz push da branch e abre o pull ou merge request para o pai, com `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone` e `--integration` (não desfazível; nada é mesclado nem excluído localmente). `--mode local-merge` substitui um tipo `merge-request`, e um tipo que diz `ask` precisa de um dos dois. |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | Traz os commits do pai para a branch: um merge, ou um rebase quando a estratégia da branch é `rebase` (ou `--rebase`). Com `--stack` também as branches cujo pai é esta (registrado com `set-parent`), recursivamente, em ordem. Volta para a branch em que você estava. |
| `gitpoulpi flow promote <env-branch>` | Branches de ambiente do GitLab Flow: faz merge, nela, da branch da qual ela está a jusante (`main` em `pre-production`, `pre-production` em `production`), em um passo, e volta. Faz fast-forward quando pode. |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | Grava (ou remove) `branch.<name>.gitpoulpi-parent`. Recusa um ciclo. Desfazível. |

Uma branch iniciada de outro lugar é concluída lá: quando o pai registrado
(por `--from` ou `set-parent`) difere do do tipo, o primeiro destino de conclusão
é o pai registrado.

### Confirmando, planejando, automatizando

| Opção | Significado |
|---|---|
| `--dry-run` | Imprime o plano como JSON (`{"plan": ...}`) e para. Código de saída 1 quando algo o bloqueia. |
| `--yes` | Executa sem perguntar. |
| nenhuma | Em um terminal: o plano é mostrado e você é consultado. Em outro contexto (um script, `--json`, um agente) o plano volta como `{"needs_confirmation": {"plan": ...}}` com código de saída 3 e nada alterado. `start` sem `--publish` nem hooks, `set-parent` e `--abort` não precisam de confirmação. |
| `--run-hooks` / `--no-hooks` | Veja [hooks](#hooks). |
| `--autostash` | Faz stash das alterações locais que bloqueiam um checkout ou merge e as devolve depois, seja qual for o valor de `general.auto_stash`. |

Um plano bloqueado cita o motivo: uma branch base onde se precisa de um tópico, um nome
que já existe, um campo ausente (`release/{version}` precisa de `version=...`),
uma tag que existe, um destino que está atrás do remoto (`--allow-behind`
continua mesmo assim), um merge ou rebase já em andamento, um fluxo de trabalho que não está
configurado.

Com a configuração de **auto-stash** (`general.auto_stash`), as alterações em andamento que
bloqueiam um checkout ou merge sofrem stash e são restauradas em volta da ação exatamente como
em um checkout comum; com `never` a ação para antes de alterar qualquer coisa.

### Conflitos

Um merge ou rebase que para em conflitos deixa o estado normal de conflito (o
aviso de conflito, `gitpoulpi conflicts ...`) e o fluxo é lembrado. Código de saída
2, `status: "conflicted"`, os caminhos em `conflicts`. Resolva os arquivos e
marque-os como resolvidos, depois:

- `gitpoulpi flow finish --continue` faz o commit do merge (ou continua o
  rebase, ou faz o commit do squash com a mensagem dele) e executa os passos restantes;
- `gitpoulpi flow finish --abort` aborta o merge ou rebase, volta para a
  branch em que o fluxo começou e esquece o resto. Os passos já
  concluídos (o primeiro destino de uma conclusão dupla, com a tag) permanecem: o `undo`
  os desfaz.

Um fluxo interrompido por um hook com falha é retomado do mesmo jeito (`--continue` executa esse
passo de novo). A execução com conflito e a continuação dela são duas entradas de desfazer.

### Merge requests

O daemon faz push da branch; abrir a solicitação precisa do seu token de hospedagem, então
a CLI faz isso com o mesmo código de `gitpoulpi hosting pr` (uma integração
que corresponda ao remoto, senão a página web pré-preenchida do host, impressa como URL).
O resultado traz `merge_request` (o que foi pedido) e `opened` (o que o host
respondeu), ou `open_error` quando a branch foi enviada mas a solicitação não pôde ser
aberta.

### Hooks

`[hooks]` no arquivo do fluxo de trabalho lista comandos de shell (`after_start`,
`before_finish`). Eles vêm de um arquivo que a sua equipe compartilha, então **nunca são
executados a menos que você peça**: `--run-hooks`, ou sim à pergunta que um terminal faz depois de
listá-los. O plano sempre os lista, com `hooks_run: false` quando não
serão executados. Um hook é executado em um shell com o repositório como diretório de trabalho e
`GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH` definidos; o comando exato dele, o código
de saída e a saída inteira estão no resultado. Um hook que sai com valor diferente de zero interrompe o
fluxo ali (um hook `before_finish` é executado antes de qualquer alteração, então é um erro
simples; depois de um hook `after_start` a branch existe e o fluxo pode ser
continuado ou abortado). `--no-hooks` os pula sem perguntar. Um hook pode fazer
qualquer coisa, então o `undo` só desfaz as alterações do git.

### Para agentes (MCP)

As ferramentas são `flow_start`, `flow_finish`, `flow_sync`, `flow_promote` e
`flow_set_parent` (finish, sync e promote são anotadas como destrutivas, então
`allow_mutations` as controla). Elas são planejar-e-depois-executar: chame com
`dry_run: true` para obter o plano (um resultado normal) e depois de novo com `yes: true`
para executá-lo. Sem nenhum dos dois a chamada é recusada e traz o plano, então nada
é executado por acidente. Os hooks só são executados com um `run_hooks: true` explícito; o plano
os lista (`hooks`, `hooks_run`) para que o agente possa mostrar os comandos ao usuário
antes. Depois de um conflito, `continue_: true` (a opção é `--continue`) ou
`abort: true`.

### Desfazer

| Ação | O que o `undo` faz |
|---|---|
| start | Exclui a branch, esquece o pai dela, faz checkout da branch anterior de novo. |
| finish | Restaura cada branch que ele moveu, a tag, a branch excluída e o checkout, em um passo. Uma branch enviada, uma exclusão remota e um merge request aberto permanecem (o plano avisa). |
| sync | Devolve cada branch atualizada, inclusive os arquivos da que está em checkout. |
| promote | Devolve a branch de ambiente. |
| set-parent | Restaura o pai registrado anterior. |

## Fluxos conduzidos pelo host

Equipes em que uma issue inicia o trabalho: a issue é escrita a partir de um modelo, e o
host (GitLab, GitHub, Gitea, Forgejo) ou um bot cria a branch e o merge
request. O GitPoulpi cria a issue, espera o que o host cria e a assume: uma branch local
acompanhando a remota, com o pai registrado (o destino do merge request) e com checkout feito.
Isso precisa da seção ISSUES vinculada a um rastreador
(`gitpoulpi hosting link issues`); o vínculo de PULL REQUESTS diz onde as branches
e as solicitações ficam (o projeto de ISSUES quando ele não está vinculado).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # um bot costuma disparar por um rótulo
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # um arquivo do repositório
branch_from_issue = "{type}/{issue}-{slug}"   # como o host ou o bot nomeia a branch
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # por quanto tempo o `wait` consulta
merge_request = true                 # host-creates-branch: também uma solicitação em rascunho
assignees = []
```

`{type}` em `branch_from_issue` é o id do tipo de tópico; o resto são os tokens
usuais do [padrão de nome](#padrões-de-nome) (`{issue}`, `{slug}`, `{issue_key}`...). Uma
branch pertence à issue 12 quando corresponde a esse padrão com `{issue}` igual a 12
(`feature/12-login` pertence, `feature/120-x` não).

| Modo | Depois que a issue é criada |
|---|---|
| `local` | Nada no host. Com `--pick-up`, um `flow start` do tipo com o número e o título da issue (desfazível como qualquer start). |
| `host-creates-branch` | O GitPoulpi cria a branch **no host** a partir do pai do tipo (GitLab: a API de branches; GitHub: `createLinkedBranch`, então a issue a mostra em Development; Gitea/Forgejo: a API de branches) e uma solicitação **em rascunho** que diz `Closes #12`. Um host que recusa uma solicitação vazia (GitHub: ainda sem commits) deixa a branch e avisa com um aviso. |
| `bot` | Nada: um bot (um tratador de webhook, um job de CI, um GitHub App) cria a branch e a solicitação. `--wait` consulta até `wait_for` ser atendido. |

O que o `wait` olha, para cada provedor: os links do próprio host (GitLab
`issues/:iid/related_merge_requests` e `closed_by`; GitHub, as branches vinculadas e os
pull requests `connected` / `cross-referenced` da linha do tempo), as branches cujo nome
corresponde a `branch_from_issue` e as solicitações abertas cuja branch de origem corresponde. Ele consulta
com uma pausa crescente (1, 2, 3, 5, 8 e depois a cada 10 s) até `wait_for` ser atendido ou
`timeout_secs` terminar, e um host que está lento ou brevemente inacessível depois de uma resposta é
consultado de novo. Uma fonte que o host não oferece (um Gitea não tem links de issue) é pulada; quando
todas as fontes falham, o primeiro erro é a resposta.

| Comando | O que faz |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | Cria a issue a partir do modelo (`issue_title`, `issue_labels`, o arquivo de descrição, `--body` no lugar dele) e segue como o modo manda. `--dry-run` imprime o que seria criado (`{"plan": ...}`) e não cria nada. `--pick-up` espera primeiro no modo `bot`. Código de saída 4: a espera esgotou (a issue existe; `flow issue wait` continua esperando). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Consulta o que o host criou para uma issue, com o progresso no stderr em um terminal; Ctrl+C cancela. `--once` olha uma única vez. Código de saída 4: ainda nada. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | Faz fetch, cria a branch local acompanhando `origin/<branch>`, registra o pai dela e faz checkout. `#12` procura no host a branch ou a solicitação da issue, `!5` uma solicitação, qualquer outra coisa é um nome de branch. O pai é o destino da solicitação (`--parent` o substitui), senão o pai do tipo. Não precisa de `--yes`: é local e desfazível, como o `start`. Um `--dry-run` ainda faz fetch (ele só move refs de acompanhamento remoto, e o plano precisa ver a branch). |

`pick-up` é uma entrada no histórico de desfazer: `gitpoulpi undo` exclui a branch,
esquece o pai e faz checkout da branch anterior de novo (o fetch permanece, como
todo fetch). Ele é planejado como as outras ações: `--dry-run` imprime o plano dele (`create_branch`,
`set_upstream`, `checkout`, `record_parent`) e os bloqueios de sempre se aplicam (as alterações em
andamento seguem a configuração de auto-stash). A requisição é
`{"action": "pick_up", "branch": ..., "parent": ...}`.

As issues criadas por um fluxo são **vigiadas** (`watched_issues` em `.gitpoulpi/repo-state.toml`,
nunca com commit): depois de cada fetch o aplicativo compara as branches remotas com
`branch_from_issue` e avisa quando uma apareceu para uma issue vigiada. Assumir a branch
resolve isso.

### Para agentes (conduzidos pelo host)

As ferramentas são `flow_issue_new`, `flow_issue_wait` e `flow_pick_up` (write, network;
`allow_mutations` as controla). `flow_issue_new` com `dry_run: true` retorna o
plano; `flow_issue_wait` bloqueia por até `timeout` segundos, então dê a um agente um valor curto
e chame de novo (`once: true` responde na hora). As duas retornam JSON com `found`
(`branches`, `requests`) e `timed_out`.

### Fluxos conduzidos pelo host no aplicativo

Com um fluxo de trabalho e um rastreador vinculado, o menu Flow ganha *Nova issue de <Tipo>...* (uma por
tipo; paleta `Flow: nova issue de <Tipo>...` e `Flow: nova issue...`) e *Assumir uma branch criada
pelo host...* (paleta `Flow: assumir branch do host...`).

- **Nova issue** pede o tipo (uma lista suspensa) e o título; mostra de onde vem a
  descrição e quem cria a branch. *Assumir a branch quando ela existir*
  vem ligada por padrão. *Criar issue* a cria e, no modo `bot`, mostra o **cartão de
  espera**: um indicador de progresso, o que ele espera, o tempo decorrido e o tempo em que desiste, e
  **Parar de esperar** (`Esc`). Parar não perde a issue: ela continua vigiada. Quando
  a branch aparece, ela é assumida e a linha de status avisa.
- **Uma branch apareceu** é um cartão que se abre depois que um fetch trouxe a branch de uma
  issue vigiada (quando nenhum outro diálogo está aberto): *Assumir* a assume, *Depois* o fecha
  (ele não é oferecido de novo até o aplicativo reiniciar).
- **Assumir** aceita `#12`, `!5` ou um nome de branch e mostra o que encontrou antes de
  assumi-la.

Tudo é executado em segundo plano; uma chamada que dura mais de 150 ms mostra um
indicador de progresso.

### Testando em uma bancada

`scripts/fake-gitlab-bot.sh` é um bot de mentira para o GitLab em Docker
(`scripts/test-bench.sh`): ele vigia um projeto atrás de issues com um rótulo e cria
`<type>/<iid>-<slug>` (com um commit) e uma solicitação em rascunho que fecha a issue.
`cargo run -p xtask -- mock-host --flow-bare <bare repo>` serve a mesma
coisa offline (uma API no formato do GitLab sobre um repositório bare de verdade, com o bot).

## No aplicativo

![A barra lateral com um fluxo de trabalho Git Flow: main e develop marcadas como branches base, as branches feature em uma pasta Feature, o resto em Outras](../images/workflow-sidebar.png)

![Iniciar Feature: o tipo, o slug, o ponto de partida e o nome da branch que será criada](../images/workflow-start.png)

![Configurações > Repositório > Fluxo de trabalho: o modelo, a detecção e o editor de branches base](../images/workflow-settings.png)

Tudo abaixo aparece só em um repositório que tem um fluxo de trabalho (um
`.gitpoulpi/workflow.toml` ou uma substituição sua). Sem um, nada muda:
a barra lateral é a árvore comum e não há botão `Flow`.

### Configurações > Fluxo de trabalho

*Configurações > Configurações do repositório > Fluxo de trabalho* edita a definição. Ela trabalha em um
rascunho e não salva nada até você pressionar um botão Salvar.

- **Modelo** é uma lista suspensa dos modelos prontos; escolher um substitui as bases e os
  tipos de tópico abaixo pelo modelo (os seus hooks e o fluxo de issues permanecem).
- **Detectar a partir das branches** lê os nomes de branch deste repositório e mostra o
  modelo mais próximo com a pontuação, os nomes de base que encontrou (`master` para `main`),
  quantas branches cada tipo de tópico reconhece, com exemplos, e a pontuação de
  cada modelo. **Usar o fluxo de trabalho detectado** o carrega no rascunho.
- Quando o repositório guarda a configuração de outra ferramenta (o `gitflow.*` do Git Flow,
  o `git-town.*` do Git Town), **Importar** é oferecido e carrega a definição
  importada no rascunho. Nada é aplicado até você pressioná-lo.
- **Branches base**, **Tipos de tópico** e suas **Regras de conclusão** formam uma única
  [lista](settings.md#listas) em grupos na aba **Regras**; a aba **Geral** contém o
  resto (modelo, detecção, hooks, fluxo de issues). Selecione um item para editá-lo à direita (nome e outros nomes,
  pai, protegida, estratégia de merge, tag no merge, branch de ambiente; rótulo, padrão de
  nome, outros padrões, expressão regular, pai, modo de conclusão, excluir após concluir).
  **+ Adicionar ▾** escolhe o que adicionar (uma regra de conclusão pertence ao tipo de
  tópico selecionado); adicionar, duplicar, remover e arrastar funcionam como em toda
  lista das Configurações, e os itens se movem dentro do próprio grupo.
  A linha do **padrão de nome** lista os tokens (`{slug} {issue} {issue_key} {user}
  {date} {version} {n}`); abaixo dela uma pré-visualização ao vivo mostra as branches existentes
  que o padrão reconhece e o nome que ele criaria, e um padrão que não é válido
  diz por quê em vermelho, logo abaixo do campo (nada do que foi digitado se perde).
- **Hooks** são comandos de shell, um por linha (`Após iniciar`, `Antes de concluir`),
  editados em um editor de várias linhas (`Ctrl+S` salva, `Esc` cancela). Eles executam
  comandos na sua máquina: a página avisa, e os diálogos os mostram antes de
  executá-los.
- **Fluxo de issues** traz os campos que os [fluxos conduzidos pelo host](#fluxos-conduzidos-pelo-host) usam
  (modo, título e rótulos da issue, padrão de branch, o que esperar), guardados no
  arquivo do fluxo de trabalho.
- **Problemas** lista o que a definição tem de errado (um pai que não
  existe, um padrão quebrado).
- Uma barra fixa no fim da página mostra **● Não salvo** (ou **Sem alterações**).
  **Salvar no arquivo compartilhado** grava o rascunho em `.gitpoulpi/workflow.toml`
  (faça o commit dele), mantendo as chaves que esta versão não conhece, e incorpora a
  sua própria substituição nele. **Só para mim** guarda só a diferença para o arquivo
  compartilhado no estado do seu repositório (`.gitpoulpi/repo-state.toml`, sem commit).
  **Descartar alterações** lê de novo o que está salvo. Fechar as Configurações com um
  rascunho não salvo pergunta antes.

### A barra lateral

LOCAL e REMOTE são agrupadas pelo fluxo de trabalho: as branches base primeiro, cada uma com um
selo `[base]` (`[base, protegida]` quando protegida) e depois uma pasta por tipo de tópico
com o nome do rótulo dele e o número de branches (`Feature (3)`), e depois as
branches que nenhum tipo reivindica em `Outras`. As pastas se recolhem como qualquer pasta; as
árvores simples voltam com *Flow > Agrupar branches por tipo* (a linha do menu Flow) ou o
comando da paleta `Flow: alternar branches agrupadas`. Passar o mouse sobre uma branch mostra o tipo dela,
o pai e quantos commits ela está atrás (e à frente) do pai.

### O menu Flow e seus diálogos

O botão **Flow** da barra de ferramentas (`Ctrl+G Shift+W`, paleta `Flow: abrir o menu`) lista
*Iniciar <Tipo>...* para cada tipo de tópico, *Concluir a branch atual...*,
*Sincronizar com o pai...*, *Promover...* (branches de ambiente do GitLab Flow), *Definir
o pai...*, o alternador de agrupamento e *Configurações do fluxo de trabalho...*. A paleta tem os
mesmos comandos (`Flow: iniciar Feature...` é gerado para cada tipo).

- **Iniciar** é um formulário montado a partir do padrão de nome do tipo: um campo por token. `{issue}`
  e `{issue_key}` vêm com uma lista suspensa das issues do rastreador vinculado, que também
  preenche `{slug}` com o título da issue; `{version}` sugere a versão seguinte à da sua
  maior tag de release; `{user}` é o seu perfil ativo; `{date}` é hoje. Uma linha
  mostra o nome de branch que o padrão cria (ou por que ele não pode: uma ref inválida, um nome
  já em uso). *Começar de* é uma lista suspensa (o pai do tipo primeiro), e *Publicar*
  faz push da nova branch.
- **Concluir**, **Sincronizar**, **Promover** e **Definir pai** pedem o que precisam e depois
  mostram a **pré-visualização** que o daemon calcula (o mesmo plano que `gitpoulpi flow ... --dry-run`
  imprime): cada passo em ordem, avisos (um destino atrás do remoto, branches empilhadas
  sobre esta) e **bloqueios** (um destino que não existe, um destino atrás do
  remoto) que desligam o botão de executar e dizem por quê. Os passos que não podem ser desfeitos
  (um push, a exclusão de uma branch remota, um merge request) são marcados com `!` e coloridos, com
  uma linha "O Desfazer não vai reverter: ...". Os hooks que seriam executados são listados sob um
  aviso, com uma caixa *Executar os hooks* (desligada por padrão). Quando o modo de conclusão
  do tipo é `ask`, Concluir primeiro pergunta *Concluir por*: merge local, ou abrir um pull ou
  merge request.
- A ação então é executada com uma marca por passo (um indicador de progresso após 150 ms) e o diálogo
  mostra como cada passo foi. A ação inteira é **uma entrada de Desfazer**: `Ctrl+Z` desfaz
  de uma vez o start, o finish, o sync ou o promote (o Histórico de desfazer mostra
  `Flow finish feature/login`); pushes, exclusões remotas e merge requests permanecem, como a
  pré-visualização avisou.
- Um **conflito** interrompe a ação com **Continuar** e **Abortar**. O diálogo pode ser
  fechado para resolver o conflito com as ferramentas de conflito de sempre e depois reaberto com o
  comando da paleta `Flow: continuar ou abortar a ação interrompida...` (também uma linha do menu
  Flow enquanto uma ação está interrompida); Continuar se recusa enquanto ainda há arquivos em conflito
  e mantém os dois botões. Abortar volta para a branch em que a ação começou; os passos
  já feitos permanecem (o undo os desfaz).
- Uma conclusão por **merge request** faz push da branch e depois abre o formulário usual *Criar Pull
  Request* (*Criar Merge Request* no GitLab) preenchido: o destino é o pai da branch, o título vem da
  branch (e da issue). Os botões de metadados (responsáveis, revisores, rótulos, marco)
  funcionam como em qualquer pull request. Quando nenhuma integração responde pelo remoto, a URL de
  comparação é aberta no navegador (e mostrada na linha de status).
- Os textos dos passos e dos bloqueios são em inglês, como os da CLI; o texto próprio do diálogo é
  traduzido. Uma árvore de trabalho com alterações segue a configuração de `auto-stash`.

Cada um destes é alcançável pelo teclado e pela paleta, e toda
lista suspensa é uma caixa de combinação (Enter a abre, digitar filtra, Enter escolhe).

As linhas conduzidas pelo host (*Nova issue de <Tipo>...*, *Assumir uma branch criada pelo host...*) são
descritas [acima](#fluxos-conduzidos-pelo-host-no-aplicativo).
