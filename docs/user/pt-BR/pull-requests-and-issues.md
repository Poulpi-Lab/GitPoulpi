[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · [Français](../fr/pull-requests-and-issues.md) · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull requests e issues

Ler, criar e gerenciar pull / merge requests e issues a partir da barra lateral, do quadro e da barra de ferramentas. Conecte uma conta e vincule um repositório primeiro: veja [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md). Tudo o que se diz sobre uma issue ou solicitação (comentários, revisões, threads) está em [Conversas](conversations.md); ler e revisar uma solicitação está em [Revisando um pull ou merge request](review.md).

## Pull requests e issues na barra lateral

### PULL REQUESTS

A seção tem um cabeçalho com a contagem e um `+` (criar um pull request),
os menus de provedor e de projeto, uma caixa de busca e um menu de estado (*Abertos*,
*Merged / fechados*, *Todos*). Depois vêm quatro grupos, cada um com sua contagem:

- **Meus pull requests**: você é o autor.
- **Atribuídos a mim**.
- **Aguardando minha revisão**: você é um revisor solicitado.
- **Todos os pull requests**: recolhido no início.

A caixa de busca filtra todos os grupos (título, número, autor, branch). Cada
mudança no menu de estado consulta o host de novo.

Uma linha começa com um glifo de estado: `●` aberto (verde), `○` rascunho, `◆` merged
(roxo), `✕` fechado (vermelho). O pull request da branch em checkout fica em negrito.
As linhas aparecem como `#12  title` (GitHub, Bitbucket) ou `!53  title` (GitLab,
Azure DevOps).

### ISSUES

O título da seção nomeia o provedor: `ISSUES DO GITHUB`, `ISSUES DO JIRA`, e assim
por diante. Fica `ITENS DE TRABALHO DO ...` para o Azure DevOps e `CARTÕES DO ...` para o Trello.
Abaixo dos menus: uma caixa de busca, o menu de estado (*Abertas*,
*Atribuídas a mim*, *Fechadas*, *Todas*) e as issues. Uma linha do Jira aparece como
`ABC-12  title`, com o tipo e o status esmaecidos quando a barra lateral é larga
o bastante.

![A seção ISSUES da barra lateral com o menu de clique direito da issue #7 aberto, Abrir conversa em primeiro lugar](../images/issues-sidebar.png)

### Ações em uma linha

- `Enter` ou um clique duplo abre a linha: um pull request na [revisão](review.md) (GitHub, GitLab, Gitea,
  Forgejo, Gerrit), uma issue, e uma solicitação no Azure DevOps ou no Bitbucket, como [conversa](conversations.md)
  na janela, e qualquer outra coisa no seu navegador. O **Abrir no navegador** do menu da linha sempre abre o navegador.
- Clique com o botão direito, ou `.`, abre o menu da linha:
  - **Abrir conversa** (issues, e solicitações que não têm visão de revisão), **Abrir no navegador**, **Copiar URL**,
    **Copiar número**.
  - Para um pull request: **Editar no <provedor>**, **Checkout da branch de origem**,
    **Ver commits da branch de origem**. Para um pull request vindo de um fork, o checkout aparece como
    **Checkout de pull request (fork)**, e **Ver** fica acinzentado.
  - Para uma issue: **Criar branch a partir da issue**. Ele abre o prompt de nova
    branch com um nome como `12-fix-the-thing` (ou `ABC-123-fix-the-thing` para
    o Jira).
  - No GitHub, GitLab, Gitea e Forgejo, uma linha de pull request também tem **Revisar...**
    (também **Enter** na linha): a revisão no aplicativo, veja
    [Revisando um pull request](review.md).
  - No GitHub, GitLab, Gitea e Forgejo, as duas linhas também têm **Editar responsáveis...**,
    **Editar rótulos...** e **Definir marco...** (e **Editar revisores...** em
    um pull request): veja
    [Responsáveis, revisores, rótulos e marco](#responsáveis-revisores-rótulos-e-marco).
- **Editar no <provedor>** tem outras três formas de acesso: o lápis `✎` na
  extremidade direita de uma linha de pull request com o mouse em cima ou selecionada, `e` na linha selecionada
  e o comando da paleta *Integrações: editar pull request vinculado no host*
  (para o pull request da branch em checkout).
- Se a branch de origem ainda não estiver no repositório, o GitPoulpi avisa. Faça
  fetch primeiro (`Ctrl+G f`).

**Checkout de um pull request.** O GitPoulpi busca o pull request em uma branch local
`pr/<n>` (`mr/<n>` no GitLab) que o acompanha, e muda para ela. No GitHub, no
GitLab, no Gitea e no Forgejo isto funciona também para forks (o GitPoulpi busca a ref head especial do
host). O Azure DevOps e o Bitbucket buscam a branch de origem do
pull request. Um pull request vindo de um fork é recusado ali: adicione o fork como remoto e
faça checkout da branch dele você mesmo. Desfazer volta para a branch anterior. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**Revisando um pull request pela linha de comando (e um agente).** No GitHub
(incluindo o Enterprise), no GitLab (incluindo o self-managed), no Gitea e no Forgejo um pull request pode ser
lido e revisado sem sair do GitPoulpi: `hosting review show <n>` (arquivos,
estado do merge, verificações), `hosting review threads <n>` (threads de revisão e
conversa), `hosting review comment`, `hosting review reply`, `hosting review resolve`
e `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. O diff vem do seu repositório, não do
host: `hosting review fetch <n>` faz fetch do head do pull request em
`refs/gitpoulpi/pr/<n>` e da sua branch de destino em `refs/gitpoulpi/pr/<n>-base`
(nenhuma das duas é uma branch, nada vai para checkout). Os números de linha são os desse
diff: `new` é a linha no head, `old` a linha na base do merge. O GitLab
não tem o estado de revisão "request changes": ele é recusado lá, sem enviar nada.
O Gitea e o Forgejo aceitam um comentário em uma linha só (sem intervalos); uma thread são os
comentários de uma linha de um arquivo. O Gitea pode resolver uma thread, o Forgejo 11 não
(o host não tem um endpoint para isso: a chamada diz isso). Uma resposta em um servidor
sem o endpoint de resposta é publicada como uma revisão de um comentário na mesma linha.
O Gitea recusa aprovar ou rejeitar o seu próprio pull request.
Sem um clone: acrescente `--project owner/name` (GitLab: `group/project`) a `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` e `suggest` para revisar um projeto que não é o repositório atual,
`hosting review diff <n> --project P` lê o diff (hunks por arquivo) pela API do host, e
`hosting file <path> --project P [--ref R]` lê um arquivo em uma revisão (GitHub, GitLab, Gitea, Forgejo e Gerrit; no Gerrit o diff de um
patch set é lido arquivo por arquivo e o arquivo vem da branch ou, para um id de commit, do commit). Um arquivo que o host não
mostra no diff (grande demais) é marcado como `omitted`.
As ferramentas MCP são os mesmos comandos (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Responsáveis, revisores, rótulos e marco

No GitHub (incluindo o Enterprise), no GitLab (incluindo o self-managed), no Gitea e no
Forgejo, você pode mudar a quem um pull request ou issue é atribuído, quem o revisa, os seus rótulos
e o seu marco sem sair do GitPoulpi. Os outros provedores não mostram nada disso.

- **Menu da linha**: **Editar responsáveis...**, **Editar revisores...** (só pull
  requests), **Editar rótulos...**, **Definir marco...**.
- **Paleta**: *Hospedagem: editar responsáveis...*, *Hospedagem: editar revisores...*,
  *Hospedagem: editar rótulos...*, *Hospedagem: definir marco...* listam cada uma primeiro os
  pull requests e as issues da barra lateral para você escolher. No
  [Radar](radar.md) as mesmas quatro agem sobre a linha selecionada.
- O seletor abre na hora e carrega do host as pessoas ou os rótulos do projeto (e o que
  o item tem agora). Uma linha com indicador de progresso aparece após cerca de 150 ms.
  Digite para filtrar (correspondência aproximada), `Cima` / `Baixo` movem, `Espaço` marca ou
  desmarca, `Enter` aplica, `Esc` cancela, `Tab` chega aos botões. Um clique
  em uma linha a marca, e a roda rola. As pessoas mostram um selo com iniciais,
  os rótulos a sua cor.
- O marco é uma lista suspensa (com **Sem marco** para limpá-lo).
- Depois de **Aplicar** o host é atualizado, e as listas e o Radar mostram a
  alteração após a próxima atualização, que começa logo em seguida.

Os formulários de criação (um pull request, uma issue) têm as mesmas escolhas como uma fileira de
botões: `Responsáveis: nenhum`, `Revisores: nenhum`, `Rótulos: nenhum`,
`Marco: nenhum`. No GitHub, GitLab, Gitea e Forgejo o botão Rótulos substitui o campo de
rótulos digitados do formulário de issue. Se um host aceitar o novo pull request mas
recusar alguns dos seus rótulos ou pessoas, a linha de status avisa.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, e `--assignee`, `--reviewer`, `--label`,
`--milestone` em `hosting pr` e `hosting issue create`; veja a
[referência da CLI](cli-reference.md).

### Teclas nas seções

| Tecla | Ação |
|---|---|
| `Enter` / clique duplo | abre a conversa no GitPoulpi onde o host tem uma (um pull request no GitHub, GitLab, Gitea ou Gerrit abre a revisão), senão o navegador |
| `o` | abre a linha no navegador |
| `.` / botão direito | menu da linha |
| `e` | edita o pull request selecionado no host |
| `n` | novo pull request (PULL REQUESTS) ou nova issue (ISSUES) |
| `v` | abre o menu de estado |
| `r` | atualizar |
| `Enter` / `Space` em um cabeçalho | recolhe ou expande a seção |

### Reconectar e offline

Quando o token está ausente, foi rejeitado ou não tem um escopo, a primeira linha da
seção diz `Reconectar (token rejeitado)`. Clique nela, ou pressione `Enter`, para abrir
o perfil em Configurações > Integrações.

Quando o host não pode ser alcançado, as últimas listas permanecem na tela e o cabeçalho
mostra a idade delas (`· 5m de idade`). A última resposta também fica em cache no disco
(`.gitpoulpi/hosting-cache.json`), então as listas aparecem antes de a rede
responder.

## O quadro de issues

O botão **Quadro** do cabeçalho de ISSUES (paleta: **Issues: abrir quadro**) mostra as issues da origem vinculada como um quadro
Kanban sobre a janela. As colunas dependem da origem:

| Origem | Colunas | Mover um cartão |
|---|---|---|
| Notas Markdown locais | os status (`statuses` do perfil, depois qualquer outro em uso) | reescreve o `status` da nota |
| GitHub, GitLab, Gitea, Forgejo | **Aberto**, depois os rótulos que você escolheu, depois **Fechado** | troca o rótulo de status; para **Fechado** ele fecha a issue, para uma coluna de novo ele a reabre |
| Trello | as listas abertas do quadro, da esquerda para a direita | move o cartão para a lista |
| Redmine | os status da issue | muda o status da issue (valem as regras de fluxo de trabalho do Redmine) |
| Jira (Cloud e Data Center) | os status do projeto, o de a fazer primeiro, o de concluído por último | executa a transição de fluxo de trabalho que leva à coluna (`GET` e depois `POST .../transitions`); sem essa transição o cartão volta e a linha de status diz `no transition from X to Y` |
| Azure DevOps (Services e Server) | os estados dos tipos de work item do projeto, por categoria de estado | define `System.State`; um work item cujo tipo não tem o estado da coluna volta, com o motivo (`a Bug has no state Done`) |

Em um projeto do GitHub ou do GitLab, **Colunas** (ou `c`, ou **Quadro: definir rótulos das colunas...**) pede os rótulos que são as
colunas, em ordem, por exemplo `status::todo, status::doing, status::review`. Ele propõe os rótulos do projeto que
parecem status. A escolha é salva para o repositório. Sem rótulos o quadro é Aberto / Fechado. Mover um cartão
só toca nos rótulos de status que você listou: os outros rótulos dele ficam. Um quadro de host lista as issues abertas e as 60
fechadas mais recentemente.

Uma transição do Jira que pede campos obrigatórios (uma resolução, por exemplo) é recusada pelo Jira e o cartão volta com
a própria mensagem do Jira: faça essa no site.

O mesmo quadro está disponível para scripts e agentes: `gitpoulpi hosting board` imprime as colunas com seus cartões e
`gitpoulpi hosting board move <cartão> <coluna>` move um (veja [a referência da CLI](cli-reference.md)).

Um cartão mostra o número, o título, os responsáveis como iniciais, os rótulos e o marco. Arraste-o para outra coluna,
ou `Shift+Left` / `Shift+Right`, ou **Quadro: mover cartão para...** (`m`). Ele se move na hora; após cerca de 150 ms um indicador de progresso
diz que o host ainda está trabalhando, e se o host recusar, o cartão volta e a linha de status diz por quê. `Enter` abre
a [conversa](conversations.md) da issue no GitPoulpi, e o quadro volta quando você a fecha; `e` edita responsáveis, rótulos e marco; `n` inicia uma nova issue na
coluna selecionada; `a`, `l`, `M` e `/` filtram por responsável, rótulo, marco e texto, `x` limpa os filtros; `r`
lê o host de novo. O quadro lê de novo quando as listas da barra lateral são atualizadas. O Radar não tem quadro.

## Criar um pull request

Abra o formulário com o `+` no cabeçalho de PULL REQUESTS, `n` na seção, ou
o comando da paleta *Pull requests: criar pull request...*. Um menu de branch
também tem **Iniciar pull request a partir de <branch>...**.

O formulário tem:

- Uma aba por perfil conectado que tenha pull requests (`Left` / `Right` na
  faixa de abas).
- Caixas de combinação **Repo de origem** e **Repo de destino**, cada uma com a sua **Branch**. Listas longas
  permitem busca. Você pode digitar uma branch ou um repositório que as listas não mostram.
- **Título**, preenchido com o assunto do commit da ponta.
- **Descrição**, em várias linhas (`Enter` adiciona uma nova linha, `Tab` sai do
  campo).
- Botões **Responsáveis**, **Revisores**, **Rótulos** e **Marco** (GitHub,
  GitLab, Gitea e Forgejo), que abrem o seletor.
- **Enviar como rascunho**.
- Com a [IA](ai.md) ligada: **Gerar título e descrição**.

O formulário abre no perfil, repositório e branches da branch em checkout
(`feature -> main`). Um repositório de origem diferente cria um pull request
a partir de um fork.

**Criar pull request** (ou `Ctrl+S`) cria o pull request no host. O
resultado abre no seu navegador e a linha de status o nomeia.

## Criar uma issue

Quando a seção ISSUES tem um provedor **e** um projeto, o cabeçalho dela tem um
`+` (*Nova issue*). Também: `n` na seção, ou o comando da paleta
*Hospedagem: nova issue...*.

O formulário tem **Título** (obrigatório), **Descrição** (várias linhas) e
**Labels** (separadas por vírgula, por exemplo `bug, ui`; no GitHub, GitLab, Gitea e Forgejo os
botões Rótulos, Responsáveis e Marco abrem o seletor no lugar). Jira e Azure DevOps
adicionam um menu **Tipo**. O Trello adiciona um menu **Lista**. O Linear e o YouTrack
adicionam um menu **Status** (o estado do fluxo de trabalho, ou o valor `State`, em que a issue
começa). O Redmine adiciona um menu **Tipo** com os trackers do projeto (Bug, Feature,
Support...); ele não tem rótulos, então não há botão Rótulos para ele e um rótulo
dado na linha de comando é recusado. O formulário preenche esses
menus a partir do host quando abre.

**Criar issue** (ou `Ctrl+S`) a cria no host. `Esc` ou **Cancelar**
fecha o formulário. Se o host recusar (token rejeitado, escopo ausente, projeto
desconhecido, label desconhecida), o formulário mostra o motivo e mantém o que você digitou.
Em caso de sucesso, a linha de status nomeia a nova issue e a seção ISSUES é
atualizada.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Criar na própria página web do host

O formulário Nova issue e o formulário Criar pull request têm um botão
**Criar no <provedor>**, no canto superior direito. A tecla é `Alt+W`. Ele abre o
formulário web do host no seu navegador, preenchido com o que você digitou até agora. O
diálogo continua aberto. Ele não precisa de token e não faz chamada de API, então também funciona
depois de um erro de API.

- O GitHub não tem parâmetro de URL para *rascunho*, então a caixa de rascunho não é
  levada. O GitLab recebe o prefixo `Draft: ` no título.
- O GitLab aceita labels apenas por id, então elas são adicionadas como uma ação rápida
  `/label ~"x"` no fim da descrição.
- Uma descrição muito longa é cortada para que a URL fique abaixo de 6000 caracteres,
  com uma nota `[...truncated: paste the rest]`.
- O Bitbucket Data Center não tem endereço de formulário web, então não tem o botão
  **Criar no <provedor>**.

CLI: `gitpoulpi hosting issue create ... --web` e `gitpoulpi hosting pr --web`
imprimem a mesma URL.

## O chip da barra de ferramentas

Na extremidade da barra de ferramentas (à direita por padrão), um chip mostra o pull request da branch
em checkout: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) ou
`BB #12` (Bitbucket). A cor mostra o estado: verde aberto, cinza rascunho,
roxo merged, vermelho fechado.

- Passe o mouse para ver o título, o estado e `source → target`.
- Clique nele, ou pressione `Ctrl+G g`, para abrir o pull request no navegador.
- Clique com o botão direito para **Abrir no navegador**, **Copiar URL**, **Atualizar** e
  **Trocar de perfil**.

Uma branch enviada sem pull request mostra um chip discreto `+ PR`. Clique nele para iniciar um
pull request. Não há chip quando nenhum perfil corresponde, quando a branch não
foi enviada, ou quando o host não pôde ser alcançado. Em uma janela estreita o chip fica
oculto para dar espaço aos botões da barra de ferramentas.

## Quando as listas são atualizadas

As listas são atualizadas:

- quando você abre um repositório;
- quando a branch em checkout muda;
- após um push ou um fetch;
- a cada *Atualização de Pull Requests e Issues (minutos)*
  (`general.hosting_refresh_minutes`, padrão 5, `0` desliga o temporizador;
  Configurações > Integrações);
- quando você fecha as Configurações;
- quando você pressiona `r`, executa *Integrações: atualizar pull requests e issues* ou
  muda um filtro.

As requisições nunca bloqueiam a janela.

Os hosts limitam quantas chamadas você pode fazer. O GitPoulpi reenvia o `ETag` da
última resposta, então uma lista sem mudanças custa pouco (nada no GitHub). Quando um
host informa que restam poucas chamadas, a linha de status avisa. Quando não resta nenhuma,
o GitPoulpi deixa de consultar esse host até o limite ser redefinido, e as listas
ficam como estão.

## Terminologia

GitHub, Azure DevOps e Bitbucket dizem “pull request”. O GitLab diz “merge
request”. *Terminologia de solicitação de revisão* (`ui.pr_terminology`, em
Configurações > Integrações) escolhe a palavra para cada rótulo, menu, comando
da paleta e mensagem:

- **Auto (por host)**: segue o provedor vinculado do repositório (“Pull
  Request” enquanto nada estiver vinculado).
- **Pull Request** ou **Merge Request**: sempre essa palavra.

A mudança vale na hora. A CLI e o MCP sempre dizem “pull/merge request”.
Esta página diz “pull request”.

## Referências em mensagens de commit e nomes de branch

Quando as issues ou os pull requests do repositório estão vinculados, as referências na
mensagem de commit dos Detalhes do commit ficam sublinhadas. Um clique abre a
página vinculada. Teclado: o comando da paleta
*Integrações: abrir referência do commit (#issue / !solicitação)...* (um menu quando a
mensagem tem várias referências). Um repositório sem nada vinculado não tem
links.

O que conta como referência depende do provedor da seção ISSUES:

| ISSUES vinculadas a | Em uma mensagem | Em um nome de branch |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` e `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, somente quando `ABC` é a chave do projeto vinculado. Textos como `UTF-8`, `SHA-256` ou `ISO-8601` nunca são um link. | `ABC-123` em qualquer ponto do nome (`feature/ABC-123-x`), em maiúsculas |
| Linear | `ENG-123`, somente quando `ENG` é a chave da equipe vinculada (ou de uma equipe da lista de equipes do perfil). `UTF-8` nunca é um link. | O próprio `branchName` do Linear quando você cria a branch a partir de uma issue; `ENG-123` em qualquer ponto do nome é lido de volta |
| YouTrack | `ABC-123`, somente quando `ABC` é o nome curto do projeto vinculado (ou de um da lista de projetos do perfil) | `ABC-123` em qualquer ponto do nome |
| Redmine | `#123`, somente quando o Redmine é a origem de ISSUES vinculada (uma origem de pull requests mantém o seu próprio sigilo) | `123-fix-login` |
| Trello | um endereço `https://trello.com/c/<shortLink>`; `#12` somente quando um cartão em cache tem o número 12 | `12-fix-login` |

| PULL REQUESTS vinculados a | Em uma mensagem |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Quando as duas listas correspondem a `#N`, a issue vence. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**Nas conversas.** O texto de uma issue ou solicitação é lido com a gramática do seu host, que é mais ampla que a
tabela acima: `owner/repo#7`, `!7`, `GH-7`, `@login`, ids de commit e as outras formas listadas em
[Referências cruzadas](conversations.md#referências-cruzadas). As origens vinculadas do repositório são verificadas primeiro, então uma chave do Jira
em um comentário do GitHub ainda abre a sua issue no app. `gitpoulpi hosting markdown --body TEXT --id I` mostra o que o app encontra em
um texto.

### Inserir uma referência de issue

No compositor de commit, o botão `#▾` na extremidade direita do Resumo abre
uma lista filtrável das issues vinculadas. A issue indicada pela branch atual
vem primeiro, marcada com *(da branch)*. `Enter` ou um clique insere a referência dela
(`ABC-123`, `#12`, `AB#12`) no cursor. Teclado: `Alt+R` no compositor.
Paleta: *Commit: Inserir referência de issue...*. Sem uma lista de issues vinculada, o
botão fica acinzentado e diz *vincule primeiro uma lista de issues na barra lateral*.
O GitPoulpi nunca adiciona uma referência por conta própria.
