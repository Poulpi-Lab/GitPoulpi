[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · **Português (Portugal)** · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull requests e issues

Ler, criar e gerir pull / merge requests e issues a partir da barra lateral, do quadro e da barra de ferramentas. Ligue primeiro uma conta e associe um repositório: veja [Integrações de alojamento e de gestores de issues](hosting-integration.md). Tudo o que se diz sobre uma issue ou um pedido (comentários, revisões, tópicos) está em [Conversas](conversations.md); ler e rever um pedido está em [Rever um pull ou merge request](review.md).

## Pull requests e issues na barra lateral

### PULL REQUESTS

A secção tem um cabeçalho com a contagem e um `+` (criar um pull request),
as listas pendentes de fornecedor e projeto, uma caixa de pesquisa e uma lista pendente de estado (*Abertos*,
*Merged / fechados*, *Todos*). Seguem-se quatro grupos, cada um com a sua contagem:

- **Meus pull requests**: é o autor.
- **Atribuídos a mim**.
- **A aguardar minha revisão**: é um revisor pedido.
- **Todos os pull requests**: recolhido no início.

A caixa de pesquisa filtra todos os grupos (título, número, autor, branch). Cada
alteração da lista pendente de estado volta a perguntar ao host.

Uma linha começa por um glifo de estado: `●` aberto (verde), `○` rascunho, `◆` merged
(roxo), `✕` fechado (vermelho). O pedido da branch em checkout está a negrito.
As linhas lêem-se `#12  título` (GitHub, Bitbucket) ou `!53  título` (GitLab,
Azure DevOps).

### ISSUES

O título da secção nomeia o fornecedor: `ISSUES DO GITHUB`, `ISSUES DO JIRA`, e assim
por diante. Lê-se `ITENS DE TRABALHO DO ...` no Azure DevOps e `CARTÕES DO ...` no Trello.
Por baixo das listas pendentes: uma caixa de pesquisa, a lista pendente de estado (*Abertas*,
*Atribuídas a mim*, *Fechadas*, *Todas*) e as issues. Uma linha do Jira lê-se
`ABC-12  título`, com o tipo e o estado esbatidos quando a barra lateral é
suficientemente larga.

![A secção ISSUES da barra lateral com o menu de clique direito da issue #7 aberto, Abrir conversa em primeiro lugar](../images/issues-sidebar.png)

### Ações sobre uma linha

- `Enter` ou um clique duplo abre a linha: um pull request na [revisão](review.md) (GitHub, GitLab, Gitea,
  Forgejo, Gerrit), uma issue, e um pedido no Azure DevOps ou no Bitbucket, como uma [conversa](conversations.md)
  na janela, e tudo o resto no navegador. **Abrir no navegador** do menu da linha abre sempre o navegador.
- O botão direito, ou `.`, abre o menu da linha:
  - **Abrir conversa** (issues, e pedidos que não têm vista de revisão), **Abrir no navegador**, **Copiar URL**,
    **Copiar número**.
  - Para um pull request: **Editar no <fornecedor>**, **Checkout da branch de origem**,
    **Ver commits da branch de origem**. Para um pedido vindo de um fork, o checkout lê-se
    **Checkout de pull request (fork)**, e **Ver** fica a cinzento.
  - Para uma issue: **Criar branch a partir da issue**. Abre o pedido de nova branch
    com um nome como `12-fix-the-thing` (ou `ABC-123-fix-the-thing` para
    o Jira).
  - No GitHub, GitLab, Gitea e Forgejo, uma linha de pull request tem também **Rever...**
    (também **Enter** na linha): a revisão na própria aplicação, ver
    [Rever um pull ou merge request](review.md).
  - No GitHub, GitLab, Gitea e Forgejo, ambas as linhas têm também **Editar responsáveis...**,
    **Editar etiquetas...** e **Definir marco...** (e **Editar revisores...** num
    pull request): ver
    [Responsáveis, revisores, etiquetas e marco](#responsáveis-revisores-etiquetas-e-marco).
- **Editar no <fornecedor>** tem outras três formas de acesso: o lápis `✎` na extremidade direita
  de uma linha de pull request com o rato por cima ou selecionada, `e` na linha selecionada, e
  o comando da paleta *Integrações: editar pull request associado no host*
  (para o pedido da branch em checkout).
- Se a branch de origem ainda não estiver no repositório, o GitPoulpi di-lo. Faça
  primeiro fetch (`Ctrl+G f`).

**Checkout de um pull request.** O GitPoulpi faz fetch do pedido para uma branch
local `pr/<n>` (`mr/<n>` no GitLab) que o acompanha, e muda para ela. No
GitHub, GitLab, Gitea e Forgejo isto funciona também com forks (o GitPoulpi faz fetch da ref de head
especial do host). O Azure DevOps e o Bitbucket fazem fetch da branch de origem do
pedido. Um pedido de um fork é aí recusado: adicione o fork como remoto e
faça manualmente o checkout da respetiva branch. Desfazer volta à branch anterior. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**Rever um pull request na linha de comandos (e um agente).** No GitHub
(incluindo Enterprise), GitLab (incluindo self-managed), Gitea e Forgejo um pedido pode ser
lido e revisto sem sair do GitPoulpi: `hosting review show <n>` (ficheiros,
estado de integração, verificações), `hosting review threads <n>` (discussões de revisão e
conversa), `hosting review comment`, `hosting review reply`, `hosting review resolve`
e `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. O diff vem do seu repositório, não do
host: `hosting review fetch <n>` obtém o head do pedido para
`refs/gitpoulpi/pr/<n>` e a sua branch de destino para `refs/gitpoulpi/pr/<n>-base`
(nenhum é uma branch, nada fica com checkout). Os números de linha são os desse
diff: `new` é a linha no head, `old` a linha na base de integração. O GitLab
não tem o estado de revisão «pedir alterações»: é recusado aí, sem enviar nada.
O Gitea e o Forgejo aceitam um comentário numa só linha (sem intervalos); uma discussão são os
comentários de uma linha de um ficheiro. O Gitea pode resolver uma discussão, o Forgejo 11 não
(o host não tem endpoint para isso: a chamada di-lo). Uma resposta num servidor
sem o endpoint de resposta é publicada como uma revisão de um comentário na mesma linha.
O Gitea recusa aprovar ou rejeitar o seu próprio pedido.
Sem um clone: acrescente `--project owner/name` (GitLab: `group/project`) a `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` e `suggest` para rever um projeto que não é o repositório atual,
`hosting review diff <n> --project P` lê o diff (hunks por ficheiro) através da API do host, e
`hosting file <path> --project P [--ref R]` lê um ficheiro numa revisão (GitHub, GitLab, Gitea, Forgejo e Gerrit; no Gerrit o diff de um
patch set é lido ficheiro a ficheiro e o ficheiro vem da branch ou, para um id de commit, do commit). Um ficheiro que o host
não mostra no diff (demasiado grande) é marcado `omitted`.
As ferramentas MCP são os mesmos comandos (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Responsáveis, revisores, etiquetas e marco

No GitHub (incluindo Enterprise), GitLab (incluindo self-managed), Gitea e
Forgejo, pode mudar a quem um pull request ou uma issue está atribuído, quem o revê, as suas etiquetas
e o seu marco sem sair do GitPoulpi. Os outros fornecedores não mostram nada disto.

- **Menu da linha**: **Editar responsáveis...**, **Editar revisores...** (só pull requests),
  **Editar etiquetas...**, **Definir marco...**.
- **Paleta**: *Alojamento: editar responsáveis...*, *Alojamento: editar revisores...*,
  *Alojamento: editar etiquetas...*, *Alojamento: definir marco...* listam primeiro os
  pedidos e as issues da barra lateral a escolher. No
  [Radar](radar.md) os mesmos quatro atuam sobre a linha selecionada.
- O seletor abre de imediato e carrega do host as pessoas ou as etiquetas do projeto (e o que
  o item tem agora). Uma linha com indicador de atividade aparece passados cerca de 150 ms.
  Escreva para filtrar (correspondência aproximada), `Up` / `Down` movem, `Space` marca ou
  desmarca, `Enter` aplica, `Esc` cancela, `Tab` chega aos botões. Um clique
  numa linha marca-a e a roda desloca. As pessoas mostram um selo com iniciais,
  as etiquetas a sua cor.
- O marco é uma lista pendente (com **Sem marco** para o limpar).
- Depois de **Aplicar** o host é atualizado, e as listas e o Radar mostram a
  alteração depois da sua atualização seguinte, que começa logo.

Os formulários de criação (um pull request, uma issue) têm as mesmas escolhas como uma fila de
botões: `Responsáveis: nenhum`, `Revisores: nenhum`, `Etiquetas: nenhum`,
`Marco: nenhum`. No GitHub, GitLab, Gitea e Forgejo o botão Etiquetas substitui o campo
de etiquetas escritas do formulário de issue. Se um host aceitar o novo pull request mas
recusar algumas das suas etiquetas ou pessoas, a barra de estado di-lo.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, e `--assignee`, `--reviewer`, `--label`,
`--milestone` em `hosting pr` e `hosting issue create`; ver a
[referência da CLI](cli-reference.md).

### Teclas nas secções

| Tecla | Ação |
|---|---|
| `Enter` / clique duplo | abrir a conversa no GitPoulpi onde o host tem uma (um pull request no GitHub, GitLab, Gitea ou Gerrit abre a revisão), senão o navegador |
| `o` | abrir a linha no navegador |
| `.` / botão direito | menu da linha |
| `e` | editar o pull request selecionado no host |
| `n` | novo pull request (PULL REQUESTS) ou nova issue (ISSUES) |
| `v` | abrir a lista pendente de estado |
| `r` | atualizar |
| `Enter` / `Space` num cabeçalho | recolher ou expandir a secção |

### Religar e offline

Quando o token falta, é rejeitado ou não tem um âmbito, a primeira linha da
secção lê-se `Religar (token rejeitado)`. Clique nela, ou prima `Enter`, para abrir
o perfil em Definições > Integrações.

Quando o host não pode ser alcançado, as últimas listas ficam no ecrã e o cabeçalho
mostra a sua idade (`· 5m de idade`). A última resposta fica também em cache no disco
(`.gitpoulpi/hosting-cache.json`), por isso as listas aparecem antes de a rede
responder.

## O quadro de issues

O botão **Quadro** do cabeçalho ISSUES (paleta: **Issues: abrir quadro**) mostra as issues da origem associada como um quadro
Kanban sobre a janela. As colunas dependem da origem:

| Origem | Colunas | Mover um cartão |
|---|---|---|
| Notas Markdown locais | os estados (`statuses` do perfil, depois os outros em uso) | reescreve o `status` da nota |
| GitHub, GitLab, Gitea, Forgejo | **Abertas**, depois as etiquetas que escolheu, depois **Fechadas** | troca a etiqueta de estado; para **Fechadas** fecha a issue, para uma coluna outra vez reabre-a |
| Trello | as listas abertas do quadro, da esquerda para a direita | move o cartão para a lista |
| Redmine | os estados das issues | muda o estado da issue (aplicam-se as regras de fluxo do Redmine) |
| Jira (Cloud e Data Center) | os estados do projeto, por fazer primeiro, concluído por último | executa a transição de fluxo que leva à coluna (`GET` e depois `POST .../transitions`); sem essa transição o cartão volta atrás e a barra de estado diz `no transition from X to Y` |
| Azure DevOps (Services e Server) | os estados dos tipos de work item do projeto, por categoria de estado | define `System.State`; um work item cujo tipo não tem o estado da coluna volta atrás, com o motivo (`a Bug has no state Done`) |

Num projeto do GitHub ou do GitLab, **Colunas** (ou `c`, ou **Quadro: definir etiquetas das colunas...**) pede as etiquetas que são as
colunas, por ordem, por exemplo `status::todo, status::doing, status::review`. Propõe as etiquetas do projeto que
parecem estados. A escolha é guardada para o repositório. Sem etiquetas o quadro é Abertas / Fechadas. Mover um cartão
só mexe nas etiquetas de estado que listou: as suas outras etiquetas ficam. Um quadro de host lista as issues abertas e as 60 mais
recentemente fechadas.

Uma transição do Jira que pede campos obrigatórios (uma resolução, por exemplo) é recusada pelo Jira e o cartão volta atrás com
a mensagem do próprio Jira: faça essa no site.

O mesmo quadro está disponível para scripts e agentes: `gitpoulpi hosting board` imprime as colunas com os seus cartões e
`gitpoulpi hosting board move <card> <column>` move um (ver [a referência da CLI](cli-reference.md)).

Um cartão mostra o número, o título, os responsáveis como iniciais, as etiquetas e o marco. Arraste-o para outra coluna,
ou `Shift+Left` / `Shift+Right`, ou **Quadro: mover cartão para...** (`m`). Move-se de imediato; passados cerca de 150 ms um indicador de atividade
diz que o host ainda está a trabalhar, e se o host recusar, o cartão volta atrás e a barra de estado diz porquê. `Enter` abre
a [conversa](conversations.md) da issue no GitPoulpi, e o quadro volta quando a fecha; `e` edita responsáveis, etiquetas e marco; `n` inicia uma nova issue na
coluna selecionada; `a`, `l`, `M` e `/` filtram por responsável, etiqueta, marco e texto, `x` limpa os filtros; `r`
lê o host de novo. O quadro é lido de novo quando as listas da barra lateral se atualizam. O Radar não tem quadro.

## Criar um pull request

Abra o formulário com o `+` do cabeçalho de PULL REQUESTS, `n` na secção, ou
o comando da paleta *Pull Requests: criar uma*. Um menu de branch tem também
**Iniciar pull request a partir de <branch>...**.

O formulário tem:

- Um separador por perfil ligado que tenha pull requests (`Left` / `Right` na
  linha de separadores).
- Caixas combinadas **Repo de origem / Branch** e **Repo de destino / Branch**. As listas longas
  permitem pesquisa. Pode escrever uma branch ou um repositório que as listas não mostrem.
- **Título**, preenchido com o assunto do commit de topo.
- **Descrição**, em várias linhas (`Enter` adiciona uma nova linha, `Tab` sai do
  campo).
- Botões **Responsáveis**, **Revisores**, **Etiquetas** e **Marco** (GitHub,
  GitLab, Gitea e Forgejo), que abrem o seletor.
- **Enviar como rascunho**.
- Com a [IA](ai.md) ativa: **Gerar título e descrição**.

O formulário abre no perfil, repositório e branches da branch em checkout
(`feature -> main`). Um repositório de origem diferente cria um pedido
a partir de um fork.

**Criar pull request** (ou `Ctrl+S`) cria o pedido no host. O
resultado abre no navegador e a barra de estado nomeia-o.

## Criar uma issue

Quando a secção ISSUES tem um fornecedor **e** um projeto, o seu cabeçalho tem um
`+` (*Nova issue*). Também: `n` na secção, ou o comando da paleta
*Alojamento: nova issue...*.

O formulário tem **Título** (obrigatório), **Descrição** (várias linhas) e
**Labels** (separadas por vírgula, por exemplo `bug, ui`; no GitHub, GitLab, Gitea e Forgejo os
botões Etiquetas, Responsáveis e Marco abrem antes o seletor). O Jira e o Azure DevOps
acrescentam uma lista pendente **Tipo**. O Trello acrescenta uma lista pendente **Lista**. O Linear e o YouTrack
acrescentam uma lista pendente **Estado** (o estado do fluxo, ou o valor de `State`, em que a issue
começa). O Redmine acrescenta uma lista pendente **Tipo** com os trackers do projeto (Bug, Feature,
Support...); não tem etiquetas, por isso não há botão Etiquetas para ele e uma etiqueta
dada na linha de comandos é recusada. O formulário preenche estas
listas pendentes a partir do host quando abre.

**Criar issue** (ou `Ctrl+S`) cria-a no host. `Esc` ou **Cancelar**
fecha o formulário. Se o host recusar (token rejeitado, âmbito em falta, projeto
desconhecido, label desconhecida), o formulário mostra o motivo e mantém o que escreveu.
Em caso de sucesso, a barra de estado nomeia a nova issue e a secção ISSUES é
atualizada.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Criar na própria página web do host

O formulário Nova issue e o formulário Criar pull request têm um botão
**Criar no <fornecedor>**, em cima à direita. A tecla é `Alt+W`. Abre o
formulário web do host no navegador, preenchido com o que escreveu até ao momento. A
janela de diálogo fica aberta. Não precisa de token e não faz nenhuma chamada à API, por isso também funciona
depois de um erro da API.

- O GitHub não tem parâmetro de URL para *rascunho*, por isso a caixa de rascunho não é
  transportada. O GitLab recebe um prefixo `Draft: ` no título.
- O GitLab aceita labels apenas por id, por isso são acrescentadas como uma
  ação rápida `/label ~"x"` no fim da descrição.
- Uma descrição muito longa é cortada para que o URL fique abaixo de 6000 carateres,
  com uma nota `[...truncated: paste the rest]`.
- O Bitbucket Data Center não tem endereço de formulário web, por isso não tem
  botão **Criar no <fornecedor>**.

CLI: `gitpoulpi hosting issue create ... --web` e `gitpoulpi hosting pr --web`
imprimem o mesmo URL.

## O chip da barra de ferramentas

No extremo da barra de ferramentas (à direita por predefinição), um chip mostra o pedido da branch
em checkout: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) ou
`BB #12` (Bitbucket). A cor mostra o estado: verde aberto, cinzento rascunho,
roxo merged, vermelho fechado.

- Passe o rato por cima para ver o título, o estado e `origem → destino`.
- Clique nele, ou prima `Ctrl+G g`, para abrir o pedido no navegador.
- Clique com o botão direito para **Abrir no navegador**, **Copiar URL**, **Atualizar** e
  **Mudar de perfil**.

Uma branch enviada sem pedido mostra um chip `+ PR` esbatido. Clique nele para iniciar um
pull request. Não há chip quando nenhum perfil corresponde, quando a branch não
foi enviada, ou quando o host não pôde ser alcançado. Numa janela estreita o chip fica
oculto para deixar espaço aos botões da barra de ferramentas.

## Quando as listas são atualizadas

As listas são atualizadas:

- quando abre um repositório;
- quando a branch em checkout muda;
- após um push ou um fetch;
- a cada *Atualização de Pull Requests e Issues (minutos)*
  (`general.hosting_refresh_minutes`, predefinição 5, `0` desliga o temporizador;
  Definições > Integrações);
- quando fecha as Definições;
- quando prime `r`, executa *Integrações: atualizar pull requests e issues*, ou
  altera um filtro.

Os pedidos nunca bloqueiam a janela.

Os hosts limitam quantas chamadas pode fazer. O GitPoulpi reenvia o `ETag` da
última resposta, por isso uma lista inalterada custa pouco (nada no GitHub). Quando um
host indica que restam poucas chamadas, a barra de estado di-lo. Quando já não
restam nenhumas, o GitPoulpi deixa de perguntar a esse host até o limite repor, e as listas
ficam como estão.

## Terminologia

O GitHub, o Azure DevOps e o Bitbucket dizem "pull request". O GitLab diz "merge
request". *Terminologia de pedido de revisão* (`ui.pr_terminology`, em
Definições > Integrações) escolhe a palavra para todas as etiquetas, menus, comandos
da paleta e mensagens:

- **Auto (por host)**: segue o fornecedor ligado do repositório ("Pull
  Request" enquanto nada estiver ligado).
- **Pull Request** ou **Merge Request**: sempre essa palavra.

A alteração aplica-se de imediato. A CLI e o MCP dizem sempre "pull/merge request".
Esta página diz "pull request".

## Referências em mensagens de commit e nomes de branch

Quando as issues ou os pull requests do repositório estão ligados, as referências na
mensagem de commit dos Detalhes do commit aparecem sublinhadas. Um clique abre a
página ligada. Teclado: o comando da paleta
*Integrações: abrir referência do commit (#issue / !pedido)...* (um menu quando a
mensagem tem várias referências). Um repositório sem nada ligado não tem
ligações.

O que conta como referência depende do fornecedor da secção ISSUES:

| ISSUES ligadas a | Numa mensagem | Num nome de branch |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` e `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, apenas quando `ABC` é a chave do projeto ligado. Texto como `UTF-8`, `SHA-256` ou `ISO-8601` nunca é uma ligação. | `ABC-123` em qualquer ponto do nome (`feature/ABC-123-x`), em maiúsculas |
| Linear | `ENG-123`, apenas quando `ENG` é a chave da equipa ligada (ou de uma equipa na lista de equipas do perfil). `UTF-8` nunca é uma ligação. | O `branchName` próprio do Linear quando cria a branch a partir de uma issue; `ENG-123` em qualquer ponto do nome é lido de volta |
| YouTrack | `ABC-123`, apenas quando `ABC` é o nome curto do projeto ligado (ou de um da lista de projetos do perfil) | `ABC-123` em qualquer ponto do nome |
| Redmine | `#123`, apenas quando o Redmine é a origem ISSUES ligada (uma origem de pull requests mantém o seu próprio sigilo) | `123-fix-login` |
| Trello | um endereço `https://trello.com/c/<shortLink>`; `#12` apenas quando um cartão em cache tem o número 12 | `12-fix-login` |

| PULL REQUESTS ligados a | Numa mensagem |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Quando ambas as listas correspondem a `#N`, a issue ganha. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**Em conversas.** O texto de uma issue ou de um pedido é lido com a gramática do seu host, que é mais ampla do que a
tabela acima: `owner/repo#7`, `!7`, `GH-7`, `@login`, ids de commit e as outras formas listadas em
[Referências cruzadas](conversations.md#referências-cruzadas). As fontes associadas ao repositório são comparadas primeiro, por isso uma
chave do Jira num comentário do GitHub continua a abrir a sua issue na aplicação. `gitpoulpi hosting markdown --body
TEXTO --id I` mostra o que a aplicação encontra num texto.

### Inserir uma referência de issue

No compositor de commits, o botão `#▾` na extremidade direita do Resumo abre
uma lista filtrável das issues ligadas. A issue indicada pela branch atual
vem primeiro, marcada com *(da branch)*. `Enter` ou um clique insere a sua referência
(`ABC-123`, `#12`, `AB#12`) no cursor. Teclado: `Alt+R` no compositor.
Paleta: *Commit: Inserir referência de issue...*. Sem uma lista de issues ligada, o
botão fica a cinzento e diz *ligue primeiro uma lista de issues na barra lateral*.
O GitPoulpi nunca acrescenta uma referência por si.
