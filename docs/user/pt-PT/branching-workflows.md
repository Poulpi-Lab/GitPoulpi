[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · [Français](../fr/branching-workflows.md) · [Italiano](../it/branching-workflows.md) · [日本語](../ja/branching-workflows.md) · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · **Português (Portugal)** · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# Fluxos de trabalho de branches

Um fluxo de trabalho de branches diz ao GitPoulpi como a sua equipa usa as branches: quais vivem
para sempre (`main`, `develop`, `production`), quais são branches de tópico de vida curta
(`feature/...`, `hotfix/...`), como se chamam e onde cada uma começa e
termina. É o Git Flow tornado personalizável: nomes fixos, prefixos e sufixos, ou
padrões com marcadores, de modo que as branches que um bot cria a partir de uma issue sejam
reconhecidas como as suas.

Esta página cobre o modelo, o ficheiro, os modelos prontos, a deteção e a importação, os
comandos de leitura, as ações que iniciam, concluem, sincronizam e promovem branches
(CLI, MCP e aplicação), e os ecrãs da aplicação: o editor de definições, a barra lateral
agrupada, o menu `Flow` e os seus diálogos.

## Conceitos

- Uma **branch base** é duradoura. Uma base pode ter um **pai** (`develop` entra em
  `main`); um tronco não tem nenhum. Uma base pode listar vários nomes candidatos
  (`main` ou `master`, `develop` ou `dev`): usa-se o que existir.
- Um **tipo de tópico** é uma espécie de branch de vida curta (feature, bugfix, release,
  hotfix, support...). Tem um **padrão de nome**, um **pai** (onde começa
  e para onde é concluída) e uma forma de concluir.
- Cada branch é **classificada**: uma base (pelo nome), um tópico de algum tipo (pelo
  padrão) ou não classificada.
- O **pai de uma branch** é, por ordem: o que ficou registado quando o GitPoulpi criou
  a branch (`git config branch.<name>.gitpoulpi-parent`), senão o
  `git-town-branch.<name>.parent` do Git Town quando existe, senão o pai do seu tipo.

## Onde é guardado

| O quê | Onde | Com commit |
|---|---|---|
| O fluxo de trabalho partilhado | `.gitpoulpi/workflow.toml` no repositório | Sim, por si: o GitPoulpi só o escreve quando pede |
| As suas próprias alterações | `[workflow]` em `.gitpoulpi/repo-state.toml` | Não (esse ficheiro é ignorado) |

A pasta de estado ignora-se a si própria (`.gitpoulpi/.gitignore`), exceto
`workflow.toml`, que se destina a levar commit. O fluxo de trabalho efetivo é o
ficheiro partilhado com a sua substituição por cima: uma base com o mesmo nome, ou um tópico com
o mesmo id, é substituído; os novos são acrescentados; `disable_bases` e `disable_topics`
removem os partilhados só para si.

Guardar por cima de um ficheiro existente mantém as chaves que esta versão não conhece (as de um
GitPoulpi mais recente, as de um colega), mas não os comentários nem a ordem das chaves.

## O ficheiro

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (informativo)

[[base]]
name = "main"
names = ["master"]             # outras grafias da mesma branch
protected = true               # nunca eliminada, commits diretos são suspeitos

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
# regex = "^feat-(?P<slug>.+)$"  # também corresponde; os grupos com nome tornam-se campos
parent = "develop"             # onde começa e o destino de conclusão por omissão
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # comandos de shell opcionais, sempre mostrados antes de executarem
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # fluxos guiados pelo host, ver abaixo
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode` e `hooks` comandam as [ações](#ações) abaixo;
`issue_flow` comanda os [fluxos guiados pelo host](#fluxos-guiados-pelo-host).

### Padrões de nome

Texto com `{marcadores}`:

| Marcador | Corresponde a | Gerado a partir de |
|---|---|---|
| `{slug}` | texto livre; pode conter `/` quando é o último | o texto, em minúsculas, sem acentos, com `-` entre palavras, no máximo 50 carateres |
| `{issue}` | dígitos (`123`) | `123` ou `#123` |
| `{issue_key}` | uma chave do tracker (`PROJ-123`) | tal como indicada |
| `{user}` | um nome sem `/` | convertido em slug |
| `{date}` | `2026-10-16` ou `20261016` | tal como indicada |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | tal como indicada |
| `{n}` | dígitos | tal como indicado |

Tudo o resto é literal, por isso prefixos e sufixos são apenas texto:
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. Cada
marcador pode aparecer uma só vez e dois marcadores precisam de texto entre eles. Um
padrão tem de produzir nomes de branch válidos (as regras de `git check-ref-format`):
sem espaços, sem `~ ^ : ? * [ \`, sem `..`, sem componentes que comecem por `.` ou terminem
em `.lock`.

Uma `regex` serve só para corresponder (está ancorada nas duas pontas); use-a para nomes que nenhum
padrão consiga exprimir. Os grupos com nome tornam-se os campos da branch.

### Que tópico ganha

1. Uma branch com o nome de uma base (qualquer dos seus nomes) é essa base.
2. Caso contrário, cada padrão de cada tópico que corresponda é candidato: `pattern`,
   depois `patterns`, depois `regex`, tópico a tópico pela ordem do ficheiro. Ganha o que fixa
   mais carateres literais (`release/{version}` vence `{user}/{slug}`,
   que vence um simples `{slug}`); uma regex conta o seu prefixo literal. Um empate fica com o
   primeiro do ficheiro.
3. Caso contrário, a branch fica não classificada.

Um padrão ou regex avariado é comunicado como aviso e ignorado; o resto
continua a funcionar.

## Modelos

| Modelo | Bases | Tópicos |
|---|---|---|
| `git-flow` | `main` ou `master`; `develop` ou `dev` (pai main) | `feature/` e `bugfix/` (a partir de develop), `release/` (a partir de develop, para main e develop, com tag), `hotfix/` (a partir de main, para main e develop, com tag), `support/` (a partir de main); as grafias no plural (`features/`, `releases/`...) também correspondem |
| `github-flow` | `main` | `feature/` ou qualquer nome, a partir de main, merge request para main, squash, eliminada depois |
| `gitlab-flow` | `main`; `pre-production` e `production` (branches de ambiente, encadeadas main, pre-production, production) | `feature/` ou qualquer nome a partir de main; `release/{version}` como destino de cherry-pick |
| `trunk` | `main` | `{user}/{slug}` de vida curta a partir de main, com rebase |
| `custom` | nenhuma | nenhum |

As branches de ambiente também correspondem como `preprod`, `staging` e `prod`. Um modelo é
um ponto de partida: imprima-o com `flow config init` e edite o ficheiro.

## Deteção e importação

`gitpoulpi flow detect` ordena os modelos pelas branches que o repositório tem,
com uma pontuação em 100 e os nomes concretos encontrados (`master` para `main`, `dev`
para `develop`, `features/` para `feature/`). As bases valem 60 pontos, a parte
das outras branches que os padrões específicos do modelo reconhecem vale 40, e uma
branch duradoura de outro modelo (um `develop` num candidato GitHub Flow)
tira 25.

`gitpoulpi flow import --from git-flow` lê as definições clássicas do git-flow
(`gitflow.branch.master`, `gitflow.branch.develop`,
`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`) do `.git/config`
do próprio repositório; `--from git-town` lê `git-town.main-branch` e
`git-town.perennial-branches` (o pai de cada branch,
`git-town-branch.<name>.parent`, é aproveitado como pai registado). Nada é
aplicado em silêncio: o comando imprime o TOML e `--write` cria
`.gitpoulpi/workflow.toml` só se este não existir.

## Comandos

Todos aceitam `--json`; o servidor MCP expõe-nos como ferramentas (`flow_status`,
`flow_templates`, `flow_detect`, `flow_config_show` leem; as ações são
descritas abaixo).

| Comando | O que faz |
|---|---|
| `gitpoulpi flow status` | Cada branch local e remota: tipo (base, tópico, não classificada), tipo de tópico e campos, pai e de onde veio (`recorded`, `git_town`, `type`, `base`) e, nas branches locais, quantos commits está à frente (`ahead`) e atrás (`behind`) do pai. Também os avisos encontrados na definição, a deteção ordenada e as definições de outras ferramentas que poderiam ser importadas. |
| `gitpoulpi flow templates` | Os modelos acima, com a definição completa. Não precisa de repositório. |
| `gitpoulpi flow detect` | Os modelos ordenados com as bases e os tópicos encontrados. |
| `gitpoulpi flow config [--show-effective]` | O ficheiro partilhado (o seu caminho, se existe, o fluxo de trabalho lido, chaves desconhecidas) ou, com `--show-effective`, o fluxo de trabalho com que as suas branches são classificadas. `flow config show` é o mesmo. |
| `gitpoulpi flow config init --template <t> [--write]` | Imprime o TOML de um modelo; `--write` cria `.gitpoulpi/workflow.toml` e recusa substituir um existente. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | Como descrito acima. |

Os contadores à frente/atrás são calculados para no máximo 300 branches locais por chamada.

Um ficheiro de fluxo de trabalho que não se consiga ler (TOML inválido, um valor com tipo errado) é um erro
que nomeia o ficheiro; nunca é ignorado em silêncio.

## Ações

`start`, `finish`, `sync`, `promote` e `set-parent` atuam sobre o repositório. Cada
uma começa por **planear**: lista todos os passos de git que vai fazer, com o comando git
a que cada um equivale, os avisos, o que não se pode desfazer e tudo o que a
bloqueia. Nada muda até confirmar. Cada execução é **uma entrada no histórico de
desfazer** (`gitpoulpi undo` reverte em conjunto as branches, a tag, a branch eliminada,
o pai registado e o checkout).

| Comando | O que faz |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | Dá nome à branch a partir do padrão do tipo (`flow start feature "Login form"` cria `feature/login-form`; `flow start release version=1.2.0` cria `release/1.2.0`; uma palavra solta é o `slug`; `user` e `date` assumem por omissão o utilizador e o dia de hoje), cria-a a partir do pai do tipo ou de `--from` (uma branch, uma branch remota ou uma tag), faz checkout dela e regista o seu pai. `--publish` faz push com `-u` (não desfeito por `undo`). |
| `gitpoulpi flow finish [<branch>]` | Para cada destino `finish` do tipo, por ordem: faz checkout do destino e merge da branch com a estratégia do destino (`merge`: sempre um commit de merge; `squash`: um commit com o título de `--title` ou do nome da branch; `rebase`: rebase da branch sobre o destino e depois fast-forward; `fast-forward`), cria a tag quando o destino tem uma (`tag = "v{version}"`, anotada), e depois faz checkout do pai e elimina a branch local quando o tipo o indica (`--delete` / `--keep` substituem-no; `--delete-remote` elimina-a também no remoto, não desfazível). Com `finish_mode = "merge-request"` (ou `--mode merge-request`) apenas envia a branch e abre o pull ou merge request para o pai, com `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone` e `--integration` (não desfazível; nada é integrado nem eliminado localmente). `--mode local-merge` substitui um tipo `merge-request`, e um tipo que diga `ask` precisa de um dos dois. |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | Traz os commits do pai para a branch: um merge, ou um rebase quando a estratégia da branch é `rebase` (ou `--rebase`). Com `--stack` também as branches cujo pai é esta (registado com `set-parent`), recursivamente, por ordem. Volta à branch onde estava. |
| `gitpoulpi flow promote <env-branch>` | Branches de ambiente do GitLab Flow: faz merge nela da branch de que está a jusante (`main` em `pre-production`, `pre-production` em `production`), num único passo, e volta. Faz fast-forward quando pode. |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | Escreve (ou remove) `branch.<name>.gitpoulpi-parent`. Recusa um ciclo. Desfazível. |

Uma branch iniciada noutro sítio é concluída aí: quando o pai registado
(por `--from` ou `set-parent`) difere do do tipo, o primeiro destino de conclusão
é o pai registado.

### Confirmar, planear, automatizar

| Opção | Significado |
|---|---|
| `--dry-run` | Imprime o plano em JSON (`{"plan": ...}`) e pára. Código de saída 1 quando algo o bloqueia. |
| `--yes` | Executa sem perguntar. |
| nenhuma | Num terminal: o plano é mostrado e é feita uma pergunta. Caso contrário (um script, `--json`, um agente) o plano volta como `{"needs_confirmation": {"plan": ...}}` com código de saída 3 e nada alterado. `start` sem `--publish` nem hooks, `set-parent` e `--abort` não precisam de confirmação. |
| `--run-hooks` / `--no-hooks` | Ver [hooks](#hooks). |
| `--autostash` | Guarda em stash as alterações locais que bloqueiam um checkout ou merge e repõe-nas depois, seja qual for o valor de `general.auto_stash`. |

Um plano bloqueado indica o motivo: uma branch base onde é preciso um tópico, um nome
que já existe, um campo em falta (`release/{version}` precisa de `version=...`),
uma tag que existe, uma branch de destino atrás do seu remoto (`--allow-behind`
avança mesmo assim), um merge ou rebase já em curso, um fluxo de trabalho que não está
configurado.

Com a definição de **auto-stash** (`general.auto_stash`), as alterações de trabalho que
bloqueiam um checkout ou merge são guardadas em stash e repostas em torno da ação exatamente como
num checkout simples; com `never` a ação pára antes de alterar o que quer que seja.

### Conflitos

Um merge ou rebase que pára em conflitos deixa o estado de conflito normal (o aviso
de conflito, `gitpoulpi conflicts ...`) e o fluxo fica memorizado. Código de saída
2, `status: "conflicted"`, os caminhos em `conflicts`. Resolva os ficheiros e
marque-os como resolvidos; depois:

- `gitpoulpi flow finish --continue` faz commit do merge (ou continua o
  rebase, ou faz commit do squash com a sua mensagem) e executa os passos restantes;
- `gitpoulpi flow finish --abort` aborta o merge ou rebase, volta à
  branch onde o fluxo começou e esquece o resto. Os passos já
  concluídos (o primeiro destino de uma conclusão dupla, com a sua tag) mantêm-se: o `undo`
  reverte-os.

Um fluxo interrompido por um hook que falha retoma-se da mesma forma (`--continue` executa esse
passo de novo). A execução com conflito e a sua continuação são duas entradas de desfazer.

### Merge requests

O daemon envia a branch; abrir o pedido precisa do seu token de alojamento, por isso
a CLI fá-lo com o mesmo código de `gitpoulpi hosting pr` (uma integração
que corresponda ao remoto, senão a página web pré-preenchida do host, impressa como URL).
O resultado traz `merge_request` (o que foi pedido) e `opened` (o que o host
respondeu), ou `open_error` quando a branch foi enviada mas o pedido não pôde ser
aberto.

### Hooks

`[hooks]` no ficheiro do fluxo de trabalho lista comandos de shell (`after_start`,
`before_finish`). Vêm de um ficheiro que a sua equipa partilha, por isso **nunca executam
a menos que o peça**: `--run-hooks`, ou sim à pergunta que um terminal faz depois de
os listar. O plano lista-os sempre, com `hooks_run: false` quando não
vão executar. Um hook executa numa shell com o repositório como diretoria de trabalho e
`GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH` definidos; o comando exato, o código de saída
e toda a saída estão no resultado. Um hook que termina com valor diferente de zero pára o
fluxo aí (um hook `before_finish` executa antes de qualquer alteração, por isso é um erro
simples; depois de um hook `after_start` a branch existe e o fluxo pode ser
continuado ou abortado). `--no-hooks` ignora-os sem perguntar. Um hook pode fazer
qualquer coisa, por isso o `undo` só reverte as alterações de git.

### Para agentes (MCP)

As ferramentas são `flow_start`, `flow_finish`, `flow_sync`, `flow_promote` e
`flow_set_parent` (finish, sync e promote estão anotadas como destrutivas, por isso
`allow_mutations` controla-as). Funcionam em planear e depois executar: chame com
`dry_run: true` para obter o plano (um resultado normal) e de novo com `yes: true`
para o executar. Sem nenhuma das duas a chamada é recusada e traz o plano, por isso nada
executa por acidente. Os hooks só executam com um `run_hooks: true` explícito; o plano
lista-os (`hooks`, `hooks_run`) para que o agente possa mostrar primeiro os comandos ao
utilizador. Depois de um conflito, `continue_: true` (a opção é `--continue`) ou
`abort: true`.

### Desfazer

| Ação | O que o `undo` faz |
|---|---|
| start | Elimina a branch, esquece o seu pai, volta a fazer checkout da branch anterior. |
| finish | Restaura de uma só vez todas as branches que moveu, a tag, a branch eliminada e o checkout. Uma branch enviada, uma eliminação remota e um merge request aberto mantêm-se (o plano di-lo). |
| sync | Repõe cada branch atualizada, incluindo os ficheiros na que tem checkout. |
| promote | Repõe a branch de ambiente. |
| set-parent | Restaura o pai registado anterior. |

## Fluxos guiados pelo host

Equipas em que uma issue dá início ao trabalho: a issue é escrita a partir de um modelo e o
host (GitLab, GitHub, Gitea, Forgejo) ou um bot cria a branch e o merge
request. O GitPoulpi cria a issue, espera pelo que o host cria e assume-o:
uma branch local a seguir a remota, com o pai registado (o destino do merge request),
com checkout feito. Precisa da secção ISSUES associada a um tracker
(`gitpoulpi hosting link issues`); a associação de PULL REQUESTS diz onde vivem as branches
e os pedidos (o projeto de ISSUES quando não está associada).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # um bot costuma disparar com uma etiqueta
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # um ficheiro do repositório
branch_from_issue = "{type}/{issue}-{slug}"   # o nome que o host ou o bot dá à branch
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # durante quanto tempo `wait` consulta
merge_request = true                 # host-creates-branch: também um pedido em rascunho
assignees = []
```

`{type}` em `branch_from_issue` é o id do tipo de tópico; os restantes são os tokens habituais do
[padrão de nome](#padrões-de-nome) (`{issue}`, `{slug}`, `{issue_key}`...). Uma
branch pertence à issue 12 quando corresponde a esse padrão com `{issue}` igual a 12
(`feature/12-login` pertence, `feature/120-x` não).

| Modo | Depois de a issue ser criada |
|---|---|
| `local` | Nada no host. Com `--pick-up`, um `flow start` do tipo com o número e o título da issue (desfazível como qualquer start). |
| `host-creates-branch` | O GitPoulpi cria a branch **no host** a partir do pai do tipo (GitLab: a API de branches; GitHub: `createLinkedBranch`, de modo que a issue a mostra em Development; Gitea/Forgejo: a API de branches) e um pedido **em rascunho** que diz `Closes #12`. Um host que recusa um pedido vazio (GitHub: ainda sem commits) deixa a branch e di-lo como aviso. |
| `bot` | Nada: um bot (um tratador de webhook, uma tarefa de CI, uma GitHub App) cria a branch e o pedido. `--wait` consulta até `wait_for` estar satisfeito. |

O que `wait` examina, para cada fornecedor: as ligações do próprio host (GitLab
`issues/:iid/related_merge_requests` e `closed_by`; GitHub, branches associadas e os
pull requests `connected` / `cross-referenced` da linha cronológica), as branches cujo nome
corresponde a `branch_from_issue` e os pedidos abertos cuja branch de origem corresponde. Consulta
com uma pausa crescente (1, 2, 3, 5, 8 e depois a cada 10 s) até `wait_for` estar satisfeito ou
`timeout_secs` terminar, e um host lento ou brevemente inacessível depois de uma resposta é
tentado de novo. Uma fonte que o host não oferece (um Gitea não tem ligações de issues) é ignorada; quando
todas as fontes falham, o primeiro erro é a resposta.

| Comando | O que faz |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | Cria a issue a partir do modelo (`issue_title`, `issue_labels`, o ficheiro de descrição, `--body` em vez dele) e prossegue como o modo indica. `--dry-run` imprime o que seria criado (`{"plan": ...}`) e não cria nada. `--pick-up` espera primeiro no modo `bot`. Código de saída 4: a espera esgotou-se (a issue existe; `flow issue wait` continua a esperar). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Consulta o que o host criou para uma issue, com progresso no stderr num terminal; Ctrl+C cancela. `--once` olha uma única vez. Código de saída 4: ainda nada. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | Faz fetch, cria a branch local a seguir `origin/<branch>`, regista o seu pai e faz checkout dela. `#12` procura no host a branch ou o pedido da issue, `!5` um pedido, qualquer outra coisa é um nome de branch. O pai é o destino do pedido (`--parent` substitui-o), senão o pai do tipo. Não precisa de `--yes`: é local e desfazível, como o `start`. Um `--dry-run` continua a fazer fetch (só move refs de seguimento remoto e o plano precisa de ver a branch). |

`pick-up` é uma entrada no histórico de desfazer: `gitpoulpi undo` elimina a branch,
esquece o pai e volta a fazer checkout da branch anterior (o fetch mantém-se, como
todos os fetch). É planeado como as outras ações: `--dry-run` imprime o seu plano (`create_branch`,
`set_upstream`, `checkout`, `record_parent`) e aplicam-se os bloqueios habituais (as alterações
de trabalho seguem a definição de auto-stash). O pedido é
`{"action": "pick_up", "branch": ..., "parent": ...}`.

As issues criadas por um fluxo ficam **vigiadas** (`watched_issues` em `.gitpoulpi/repo-state.toml`,
nunca com commit): depois de cada fetch a aplicação compara as branches remotas com
`branch_from_issue` e avisa quando aparece uma para uma issue vigiada. Um pick-up
resolve-o.

### Para agentes (guiados pelo host)

As ferramentas são `flow_issue_new`, `flow_issue_wait` e `flow_pick_up` (escrita, rede;
`allow_mutations` controla-as). `flow_issue_new` com `dry_run: true` devolve o
plano; `flow_issue_wait` bloqueia até `timeout` segundos, por isso dê a um agente um valor curto
e chame de novo (`once: true` responde de imediato). Ambas devolvem JSON com `found`
(`branches`, `requests`) e `timed_out`.

### Fluxos guiados pelo host na aplicação

Com um fluxo de trabalho e um tracker associado, o menu Flow ganha *Nova issue de <tipo>...* (uma por
tipo; paleta `Flow: nova issue de <tipo>...` e `Flow: nova issue...`) e *Assumir uma
branch criada pelo host...* (paleta `Flow: assumir branch do host...`).

- **Nova issue** pede o tipo (uma lista) e o título; mostra de onde vem a
  descrição e quem cria a branch. *Assumir a branch quando existir*
  está ativo por omissão. *Criar issue* cria-a e depois, no modo `bot`, mostra o **cartão
  de espera**: um indicador de progresso, o que espera, o tempo decorrido e o momento em que desiste, e
  **Deixar de esperar** (`Esc`). Parar não perde a issue: continua vigiada. Quando
  a branch existe é assumida e a barra de estado di-lo.
- **Apareceu uma branch** é um cartão que abre depois de um fetch trazer a branch de uma
  issue vigiada (quando nenhum outro diálogo está aberto): *Assumir* assume-a, *Mais tarde* fecha-o
  (não volta a ser oferecido até a aplicação reiniciar).
- **Assumir** aceita `#12`, `!5` ou um nome de branch e mostra o que encontrou antes de a
  assumir.

Tudo corre em segundo plano; uma chamada que dure mais de 150 ms mostra um
indicador de progresso.

### Experimentar numa bancada de testes

`scripts/fake-gitlab-bot.sh` é um bot de substituição para o GitLab em Docker
(`scripts/test-bench.sh`): vigia um projeto à procura de issues com uma etiqueta e cria
`<type>/<iid>-<slug>` (com um commit) e um pedido em rascunho que fecha a issue.
`cargo run -p xtask -- mock-host --flow-bare <bare repo>` serve o mesmo
offline (uma API com forma de GitLab sobre um repositório bare real, com o bot).

## Na aplicação

![A barra lateral com um fluxo de trabalho Git Flow: main e develop marcadas como branches base, as branches feature numa pasta Feature, as restantes em Outras](../images/workflow-sidebar.png)

![Iniciar Feature: o tipo, o slug, o ponto de partida e o nome de branch que vai criar](../images/workflow-start.png)

![Definições > Repositório > Fluxo de trabalho: o modelo, a deteção e o editor de branches base](../images/workflow-settings.png)

Tudo o que se segue aparece apenas num repositório que tenha um fluxo de trabalho (um
`.gitpoulpi/workflow.toml` ou uma substituição sua). Sem um, nada muda:
a barra lateral é a árvore normal e não há botão `Flow`.

### Definições > Fluxo de trabalho

*Definições > Definições do repositório > Fluxo de trabalho* edita a definição. Trabalha sobre um
rascunho e não guarda nada até premir um botão Guardar.

- **Modelo** é uma lista dos modelos prontos; escolher um substitui as bases e
  os tipos de tópico abaixo pelo modelo (os seus hooks e o fluxo de issues mantêm-se).
- **Detetar a partir das branches** lê os nomes das branches deste repositório e mostra o
  modelo mais próximo com a sua pontuação, os nomes de base que encontrou (`master` para `main`),
  quantas branches cada tipo de tópico reconhece, com exemplos, e a pontuação de
  cada modelo. **Usar o fluxo de trabalho detetado** carrega-o no rascunho.
- Quando o repositório contém a configuração de outra ferramenta (os `gitflow.*` do Git Flow,
  os `git-town.*` do Git Town), é oferecido **Importar** e carrega a definição importada
  no rascunho. Nada é aplicado até o premir.
- As **Branches base**, os **Tipos de tópico** e as suas **Regras de conclusão** formam uma
  [lista](settings.md#listas) por grupos no separador **Regras**; o separador **Geral**
  contém o resto (modelo, deteção, hooks, fluxo de issues). Selecione um item para o editar à direita (nome e outros nomes, pai,
  protegida, estratégia de merge, tag em merge, branch de ambiente; rótulo, padrão de nome,
  outros padrões, expressão regular, pai, modo de conclusão, eliminar após concluir).
  **+ Adicionar ▾** escolhe o que adicionar (uma regra de conclusão pertence ao tipo de
  tópico selecionado); adicionar, duplicar, remover e arrastar funcionam como em todas as
  listas das Definições, e os itens movem-se dentro do seu grupo.
  A linha **Padrão de nome** lista os tokens (`{slug} {issue} {issue_key} {user}
  {date} {version} {n}`); por baixo, uma pré-visualização em direto mostra as branches existentes que o
  padrão reconhece e o nome que criaria, e um padrão que não é válido
  diz porquê a vermelho, mesmo por baixo do campo (nada do que escreveu se perde).
- **Hooks** são comandos de shell, um por linha (`Depois de iniciar`, `Antes de concluir`),
  editados num editor de várias linhas (`Ctrl+S` guarda, `Esc` cancela). Executam
  comandos na sua máquina: a página di-lo e os diálogos mostram-nos antes de
  executarem.
- **Fluxo de issues** traz os campos que os [fluxos guiados pelo host](#fluxos-guiados-pelo-host) usam
  (modo, título e etiquetas da issue, padrão de branch, o que esperar), guardados no
  ficheiro do fluxo de trabalho.
- **Problemas** lista o que a definição tem de errado (um pai que não
  existe, um padrão avariado).
- Uma barra fixa no fundo da página mostra **● Por guardar** (ou **Sem alterações**).
  **Guardar no ficheiro partilhado** escreve o rascunho em `.gitpoulpi/workflow.toml`
  (faça commit dele), mantendo as chaves que esta versão não conhece, e incorpora a sua
  própria substituição. **Só para mim** mantém apenas a diferença em relação ao ficheiro
  partilhado no estado do seu repositório (`.gitpoulpi/repo-state.toml`, sem commit).
  **Descartar alterações** volta a ler o que está guardado. Fechar as Definições com um
  rascunho por guardar pergunta primeiro.

### A barra lateral

LOCAL e REMOTE são agrupados pelo fluxo de trabalho: primeiro as branches base, cada uma com um
selo `[base]` (`[base, protegida]` quando é protegida), depois uma pasta por tipo de tópico
com o nome do seu rótulo e o número de branches (`Feature (3)`), e por fim as
branches que nenhum tipo reivindica em `Outras`. As pastas recolhem-se como qualquer pasta; as árvores
simples voltam com *Flow > Agrupar as branches por tipo* (a linha do menu Flow) ou com o
comando da paleta `Flow: alternar o agrupamento de branches`. Pairar sobre uma branch mostra o seu tipo,
o seu pai e quantos commits está atrás (e à frente) do pai.

### O menu Flow e os seus diálogos

O botão **Flow** da barra de ferramentas (`Ctrl+G Shift+W`, paleta `Flow: abrir menu`) lista
*Iniciar <Tipo>...* para cada tipo de tópico, *Concluir a branch atual...*,
*Sincronizar com o pai...*, *Promover...* (branches de ambiente do GitLab Flow), *Definir o
pai...*, a alternância do agrupamento e *Definições do fluxo de trabalho...*. A paleta tem os
mesmos comandos (`Flow: iniciar feature...` é gerado para cada tipo).

- **Iniciar** é um formulário construído a partir do padrão de nome do tipo: um campo por token. `{issue}`
  e `{issue_key}` trazem uma lista das issues do tracker associado, que também
  preenche `{slug}` com o título da issue; `{version}` sugere a versão a seguir à sua
  tag de release mais alta; `{user}` é o seu perfil ativo; `{date}` é hoje. Uma linha
  mostra o nome de branch que o padrão produz (ou porque não pode: uma ref inválida, um nome
  já usado). Partir de é uma lista (o pai do tipo primeiro) e *Publicar*
  envia a nova branch.
- **Concluir**, **Sincronizar**, **Promover** e **Definir pai** pedem o que precisam e depois
  mostram a **pré-visualização** que o daemon calcula (o mesmo plano que `gitpoulpi flow ... --dry-run`
  imprime): cada passo por ordem, avisos (um destino atrás do seu remoto, branches empilhadas
  sobre esta) e **bloqueios** (um destino que não existe, um destino atrás do seu
  remoto) que desativam o botão de executar e dizem porquê. Os passos que não se podem desfazer
  (um push, a eliminação de uma branch remota, um merge request) são marcados com `!` e coloridos, com
  uma linha «O Desfazer não reverte: ...». Os hooks que iriam executar são listados sob um
  aviso, com uma caixa *Executar os hooks* (desligada por omissão). Quando o modo de conclusão
  do tipo é `ask`, Concluir pergunta primeiro *Concluir por*: merge local, ou abrir um pull ou
  merge request.
- A ação executa então com uma marca por passo (um indicador de progresso ao fim de 150 ms) e o diálogo
  mostra como correu cada passo. A ação inteira é **uma entrada de Desfazer**: `Ctrl+Z` reverte
  de uma vez o start, finish, sync ou promote (o histórico de desfazer mostra
  `Flow finish feature/login`); pushes, eliminações remotas e merge requests mantêm-se, como a
  pré-visualização disse.
- Um **conflito** interrompe a ação com **Continuar** e **Abortar**. O diálogo pode ser
  fechado para resolver o conflito com as ferramentas de conflito habituais e depois reaberto com o
  comando da paleta `Flow: continuar ou abortar a ação interrompida...` (também uma linha do menu
  Flow enquanto uma ação está interrompida); Continuar recusa enquanto houver ficheiros em conflito
  e mantém os dois botões. Abortar volta à branch onde a ação começou; os passos
  já feitos mantêm-se (o desfazer reverte-os).
- Uma conclusão por **merge request** envia a branch e depois abre o formulário habitual *Criar pull
  request* (*Criar merge request* no GitLab) preenchido: o destino é o pai da branch, o título vem da
  branch (e da issue). Os botões de metadados (responsáveis, revisores, etiquetas, marco)
  funcionam como em qualquer pull request. Quando nenhuma integração responde pelo remoto, o URL
  de comparação é aberto no navegador (e mostrado na barra de estado).
- Os textos dos passos e dos bloqueios estão em inglês, como os da CLI; o texto próprio do diálogo está
  traduzido. Uma árvore de trabalho com alterações segue a definição de `auto-stash`.

Tudo isto é alcançável pelo teclado e pela paleta, e cada
lista é uma caixa combinada (Enter abre-a, escrever filtra, Enter escolhe).

As linhas guiadas pelo host (*Nova issue de <tipo>...*, *Assumir uma branch criada pelo host...*) estão
descritas [acima](#fluxos-guiados-pelo-host-na-aplicação).
