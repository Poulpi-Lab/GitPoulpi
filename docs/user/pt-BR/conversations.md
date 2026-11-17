[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · [Français](../fr/conversations.md) · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Conversas

A visão de conversa de uma issue, item de trabalho, cartão ou pull / merge request. Ela exige uma conta e um repositório vinculado: veja [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md); as listas que a abrem estão em [Pull requests e issues](pull-requests-and-issues.md); revisar código está em [Revisando um pull ou merge request](review.md).

Uma **conversa** é tudo o que foi dito sobre uma issue, item de trabalho, cartão, change ou pull / merge request: a
descrição, os comentários, as revisões, as threads no código e os eventos (etiquetas, atribuições, mudanças de estado,
pushes). O GitPoulpi a lê do host, a mostra formatada e deixa você responder sem sair da janela.

### Abrindo uma

| De onde | Como |
|---|---|
| Uma linha de ISSUES | `Enter` ou um clique duplo; o **Abrir conversa** do menu da linha; o comando da paleta *Hospedagem: abrir a conversa de uma issue...* (uma lista das issues na tela). O **Abrir no navegador** do menu da linha mantém o navegador. |
| Uma linha de PULL REQUESTS no Azure DevOps ou no Bitbucket | `Enter` ou **Abrir conversa**. No GitHub, GitLab, Gitea, Forgejo e Gerrit a linha abre a [revisão](review.md), cuja página Visão geral é esta mesma conversa (a linha do tempo, as teclas e a caixa de comentário abaixo). |
| Um cartão do [quadro de issues](pull-requests-and-issues.md#o-quadro-de-issues) | `Enter` ou um clique duplo. |
| O [Radar](radar.md) | `i` em uma linha de issue ou de solicitação, o **Abrir conversa** do menu da linha, ou *Radar: abrir a conversa da issue*. |
| Uma referência em outra conversa | Clique nela (ou `l`, e depois escolha-a): `#7`, `!12`, `ABC-12` de uma origem vinculada abrem no app e entram em uma pilha de Voltar. |
| Um agente | `gitpoulpi ui show issue <n> --view conversation` (também `ui show pr`), veja [a referência da CLI](cli-reference.md). |

**Repositórios que não estão clonados.** Uma linha do Radar de um repositório do qual você não tem clone também abre a
sua conversa: a visão precisa só do perfil de integração e do projeto, então o Radar a hospeda sobre a própria janela,
e `Esc` devolve o Radar. O que precisa de um clone fica de fora: uma referência de commit abre no host em vez de no
Log, e *Deixar a IA ler o repositório* fica em cinza com o seu motivo (veja [o capítulo de IA](ai.md#ajuda-de-escrita-em-um-host)).
Uma solicitação de um repositório clonado tem a sua revisão no lugar (`e`), e o **Abrir conversa** do Radar diz que não
há conversa para ler aqui para ela; uma issue de um repositório clonado a abre na aba desse repositório.

Uma issue de um host que não tem nenhuma para ler (o Bitbucket e o Gerrit não têm issues) abre o navegador como antes, e
um build sem o recurso `conversation` faz o mesmo em toda parte ([builds personalizados](custom-builds.md)).

### A visão

Uma visão de janela inteira: o cabeçalho (estado, quem abriu e quando, as etiquetas, o número de comentários, **Mostrar
eventos**, **Voltar**, **Atualizar**, **Abrir no navegador**), a linha **Publicando como**, a linha do tempo e, sob ela, a
caixa de comentário.

![A conversa da issue #7: a descrição com um link para #9, dois comentários, um evento de etiqueta exibido por Mostrar eventos e a caixa Novo comentário vazia com Escrever, Prévia e Lado a lado](../images/issue-conversation.png)

![A página Visão geral do merge request !58: a descrição, uma thread de código não resolvida e uma resolvida com os links de arquivo e linha, os comentários, Responder e Citar na thread selecionada e a caixa de comentário](../images/pr-conversation.png)

- A linha do tempo se lê da mais antiga para a mais nova e começa na janela mais nova (no máximo 300 entradas).
  **Carregar anteriores** no topo dela (`m`, comando da paleta *Conversa: carregar entradas anteriores*) lê a janela
  anterior, com o número ainda por vir quando o host o informa.
- Um comentário mostra o autor, a data, um chip `editado`, um chip `bot` para bots, `interno` para notas privadas e os
  botões **Responder**, **Citar**, **Editar**, **Excluir** e **Links** que a entrada selecionada permite. Uma thread
  no código mostra `caminho:linha`, `resolvida` ou `não resolvida`, `desatualizada` e os seus comentários. Uma revisão
  mostra o seu veredito.
- Os eventos são uma linha discreta cada (*carol adicionou a etiqueta lfs*). **Mostrar eventos** (`v`) os oculta; a
  escolha é guardada por repositório. Os hosts sem histórico de eventos (YouTrack, itens de trabalho do Azure DevOps,
  notas locais) não têm essa caixa.
- O texto é Markdown renderizado: títulos, listas aninhadas e de tarefas, citações, alertas no estilo do GitHub, código
  destacado, tabelas, links, imagens como `[image: alt]`. Um texto que chegou na marcação própria do host é convertido
  antes (veja [Conversão de marcação](#conversão-de-marcação)). Um bloco de código `mermaid` ganha um botão **Abrir
  diagrama** que mostra o código-fonte do diagrama em um visualizador com rolagem (`Enter` o copia); a paleta tem
  *Conversa: abrir o diagrama da entrada*. Isso exige um build com `conversation` e `mermaid`.
- Na Visão geral da revisão, o `path:line` de uma thread de código é um link para o código (`G`, *Conversa: abrir o código da
  thread*; veja [Revisão](review.md)); em uma conversa aberta sozinha (uma issue, uma linha do Radar) não há página de código
  para onde ir.
- A visão lê de novo a cada 60 segundos e com `F5` (**Atualizar**).

**Teclas.** Cada uma tem um botão, uma linha de menu ou uma linha da paleta (*Conversa: ...*), e `Ctrl+N` marca os botões com selos.

| Tecla | Ação |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Entrada anterior / seguinte |
| `PageUp`, `PageDown`, `Home`, `End`, a roda, a barra de rolagem | Rolar |
| `Tab`, `Shift+Tab` | Linha do tempo, caixa de comentário e depois os botões do cabeçalho |
| `r` | Responder: na thread da entrada selecionada, ou com o texto dela citado onde o host não tem threads |
| `Q` | Cita a entrada selecionada em um novo comentário |
| `c` | Novo comentário |
| `e`, `d` | Editam, excluem um dos seus próprios comentários (uma pergunta vem antes de excluir) |
| `l`, `Enter` | Os links da entrada: `Enter` segue o único, ou abre o menu deles |
| `y`, `o` | Copiam o endereço da entrada, abrem-na no navegador |
| `n` | Próxima entrada não lida |
| `v`, `m`, `F5` | Mostrar ou ocultar eventos, Carregar anteriores, Atualizar |
| `Alt+Left`, `Backspace` | Voltar ao longo da pilha de Voltar |
| `q`, `Esc` | Fecham (perguntam antes quando a caixa tem texto) |

### Escrever

**Novo comentário** (`c`, ou `Tab` até a caixa), **Responder** (`r`), **Citar** (`Q`) e **Editar** (`e`) usam o mesmo
editor Markdown: **Escrever**, **Prévia** e **Lado a lado** (`Alt+P` alterna Escrever e Prévia, `Alt+L` liga o Lado a
lado e precisa de uma janela de 100 colunas). `Ctrl+Enter` (também `Ctrl+S` e `Alt+Enter`) ou **Enviar** publica,
**Cancelar** descarta a caixa, e `Alt+E` abre **Escrever com IA** onde a IA está ligada ([o capítulo de IA](ai.md#ajuda-de-escrita-em-um-host)).
A prévia é montada pelo mesmo renderizador da linha do tempo, com as mesmas referências. Quando o host não aceita
Markdown, o rodapé nomeia a marcação em que o texto é enviado (*Enviado como Jira wiki markup*, *Enviado como Textile*).

![Uma resposta a uma issue sendo escrita na visão Lado a lado: o rascunho em Markdown à esquerda, a prévia renderizada à direita e Enviar](../images/issue-reply-composer.png)

- O comentário é adicionado à linha do tempo na hora e assume o seu lugar quando o host responde. Se o host recusa, a
  entrada some e a caixa abre de novo com o seu texto e o motivo.
- Um texto que você não enviou é guardado por item (`remote-conversations.toml` na pasta de configuração) e está lá da
  próxima vez que você abrir aquela caixa; enviá-lo ou descartá-lo o remove. Fechar uma caixa que tem texto pergunta antes.
- **Responder** em uma thread (uma thread de revisão, uma discussão do GitLab, um comentário do Linear, uma thread do
  Bitbucket ou do Azure DevOps) publica nessa thread. Onde o host só tem uma lista plana, **Responder** inicia um novo
  comentário com a entrada selecionada citada (`> **autor**`), igual a **Citar**.
- **Nota interna** (caixa de seleção, `Alt+I`): no GitLab e no Redmine o comentário só é visível para os membros do
  projeto. Ela só aparece nos hosts que têm essas notas, e só em um novo comentário.
- Publicar é uma ação externa: nada é registrado no diário e **nada pode ser desfeito pelo GitPoulpi**. Sem conexão, a
  caixa continua, **Enviar** fica em cinza com o motivo, e o cabeçalho diz que a cópia salva está sendo mostrada.

**Quem pode editar ou excluir.** Só os seus próprios comentários têm **Editar** e **Excluir**. A palavra do próprio
host vem primeiro quando ele dá uma (`viewerCanUpdate` e `viewerCanDelete` do GitHub, `can_edit` nos outros), depois o seu
login contra o autor do comentário, comparados sem distinguir maiúsculas; os comentários de um bot de App também têm
de vir de uma conta de bot. Enquanto o login de um bot de GitHub App não é conhecido, só contam os comentários que esta
instalação publicou (o repositório guarda os ids deles): nada é adivinhado a partir de um nome. Em uma thread,
**Editar** e **Excluir** agem sobre o último comentário dela que é seu. Um host que não pode editar nem excluir (veja
[a tabela](#o-que-cada-host-não-pode-fazer)) mostra os botões em cinza com o motivo.

### Publicar como

A linha sob o cabeçalho diz como as suas mensagens são publicadas, e a mesma linha está em toda caixa:

| Linha | Significado |
|---|---|
| *Publicando como ann (token pessoal)* | O token do perfil: o seu próprio login (`GET /user`). |
| *Publicando como ci-bot (comando de token do repositório)* | O [comando de token](hosting-integration.md#8-atuar-como-um-bot-de-github-app) do repositório é usado para a API do host. |
| *Publicando como my-app[bot] (bot de GitHub App)* | Um token de instalação. O nome vem de **Publicar como**, senão do host. |
| *Publicando como o bot do GitHub App (seu nome ainda não é conhecido: defina Publicar como nas Configurações)* | Nenhum nome configurado, nenhum aprendido ainda. |
| *Publicando como ... (bot do projeto)* | Um token de acesso de projeto ou de grupo do GitLab. |
| *Escrevendo como kai (nota local)* | Uma pasta de notas Markdown: o `local.me` do perfil, senão o usuário conectado. |
| *Ainda não se sabe como você publicaria. Defina Publicar como nas Configurações.* | Nada pôde ser aprendido. |

**Publicar como** é um nome: para um GitHub App é `my-app[bot]`. Defina-o para todos os repositórios em **Configurações >
Integrações > Publicar como** (`general.hosting_post_as`), ou para um repositório em **Configurações > Configurações do repositório > SSH e
credenciais > Host**, que o sobrescreve. Se ficar vazio, o GitPoulpi pergunta ao GitHub (GraphQL `viewer { login }`, que
um token de instalação pode ou não responder) e, se isso falhar, aprende o nome pelo autor do primeiro comentário que
publica e o guarda junto ao repositório. Um token pessoal nunca precisa disso. Veja
[Atuar como um bot de GitHub App](hosting-integration.md#8-atuar-como-um-bot-de-github-app).

### Referências cruzadas

As referências em uma conversa são sublinhadas e clicáveis. As origens vinculadas de ISSUES e PULL REQUESTS do
repositório são verificadas primeiro, então `ABC-12` de um projeto Jira vinculado abre no app mesmo dentro de um
comentário do GitHub; o que a gramática do próprio host acrescenta abre a sua página depois da confirmação do link. Um
id de commit abre no Log quando o repositório está clonado e o id tem pelo menos 7 caracteres, e no host nos outros
casos. Nada dentro de um trecho de código, de um bloco delimitado, de um link Markdown, de um endereço de e-mail ou de
uma URL é uma referência, e `C#1`, `UTF-8`, `SHA-256` e `ISO-8601` também não são.

| Host | Reconhecido |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, ids de commit |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (um projeto irmão), `@user`, `~label` e `%milestone` como chips, ids de commit |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, ids de commit |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, menções `{uuid}`, ids de commit |
| Gerrit | valores de `Change-Id` (`I` e 40 dígitos hexadecimais), ids de commit |
| Redmine | `#7`, revisões `r123`, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, só para um projeto ou equipe de uma origem vinculada |
| Trello | endereços de cartão (`trello.com/c/...`) |
| Notas locais | `[[note]]`, `[[note\|alias]]` e links `.md` relativos para outra nota da pasta |

`gitpoulpi hosting markdown --body T --id I` imprime o que o app faz de um texto, referências incluídas.

### Conversão de marcação

Os hosts que não falam Markdown são convertidos nos dois sentidos, então você lê e escreve Markdown em toda parte. O que
um conversor não entende continua sendo texto legível.

- O **Jira** (Cloud e Data Center) guarda os comentários como wiki markup pela API v2. O que você lê é convertido para
  Markdown; o que você escreve é convertido para wiki markup, e o rodapé diz *Enviado como Jira wiki markup*.
- O **Redmine** formata o texto como Textile ou Markdown, o que a API não diz: a linha **Formato de texto do servidor**
  do perfil (Configurações > Integrações, `markup`, Textile por padrão) informa o GitPoulpi. O Textile é convertido para
  Markdown na leitura e de volta para Textile na escrita.
- O **Azure DevOps Server** devolve os comentários de itens de trabalho como HTML e os aceita como HTML: eles são
  mostrados como o Markdown convertido a partir dele e escritos como HTML. O Azure DevOps Services e as threads de pull
  request aceitam Markdown.
- O **Gerrit** é texto simples: não é renderizado como Markdown e o que você escreve é enviado como digitado.
- Todo importador remove antes as sequências de controle e de escape, para que um comentário não possa mexer no terminal.

### A cópia salva, Carregar anteriores, não lidas

- Cada conversa que você abre é salva (`<pasta de estado>/conversations/<integração>/<projeto>/<tipo>-<n>.json`, só do
  usuário, sem token, no máximo 2 MB por arquivo, os 50 arquivos mais recentes são mantidos). Ela aparece na hora quando
  o item abre e é atualizada por trás. Quando o host não pode ser alcançado, a cópia salva continua na tela sob
  *Offline: mostrando a cópia salva há 3 min (motivo)* e Enviar fica em cinza.
- **Não lidas:** o que você viu por último é lembrado por conversa (as 200 mais recentes são mantidas). As entradas de
  outra pessoa que são mais novas ganham uma barra de destaque e um chip `novo`, e `n` salta para a seguinte. Uma
  conversa que você abre pela primeira vez não tem nada de novo.

### O que cada host não pode fazer

Todo `não` abaixo é um limite real: o controle fica oculto ou em cinza com o motivo, e a CLI responde com um erro que
nomeia o host antes de qualquer coisa ser enviada.

| Host | Itens | Responder em uma thread | Editar e excluir os próprios | Resolver uma thread | Eventos | Notas internas | Escrito como |
|---|---|---|---|---|---|---|---|
| GitHub | issues, pull requests | pull requests | ambos | pull requests | sim | não | Markdown do GitHub |
| GitLab | issues, merge requests | sim (discussões) | ambos | merge requests | sim | sim | Markdown do GitLab |
| Gitea | issues, pull requests | pull requests | ambos | pull requests | sim | não | Markdown |
| Forgejo | issues, pull requests | pull requests | ambos | não (o host não tem endpoint) | sim | não | Markdown |
| Azure DevOps | itens de trabalho, pull requests | pull requests | ambos | não (o status é mostrado, não alterado) | só pull requests | não | Markdown (HTML nos itens de trabalho do Server) |
| Bitbucket (Cloud, Data Center) | só pull requests | sim | ambos | não (mostrado, não alterado) | sim | não | Markdown |
| Gerrit | só changes | sim | não (nenhum dos dois pode ser editado ou excluído) | sim | sim | não | texto simples |
| Jira | issues | não (lista plana) | ambos | não | sim (changelog) | não (exige um papel no projeto) | wiki markup |
| Linear | issues | sim (respostas a um comentário) | ambos | não | sim | não | Markdown |
| YouTrack | issues | não | ambos | não | não | não | Markdown |
| Redmine | issues | não | só editar (Redmine 5 e posteriores), nunca excluir | não | sim (journals) | sim (notas privadas) | Markdown ou Textile |
| Trello | cartões | não | ambos | não | sim (movimentos) | não | Markdown |
| Notas locais | notas | não | os do próprio autor local | não | não | não | Markdown |

Além disso: nenhum host permite iniciar uma nova thread em uma linha a partir da conversa (faça isso na revisão), e as
reações são mostradas só no GitHub. Um texto maior do que o host aceita é recusado (GitHub 65.536 caracteres, GitLab
1.000.000, Azure DevOps 150.000, Jira 32.767, Trello 16.384): a CLI verifica antes, no app a própria mensagem do host é
mostrada e o seu texto é mantido. Uma nota local guarda os seus comentários em uma seção final `## Comments` do próprio
arquivo da nota, uma entrada `### autor - hora` cada, então o Obsidian e um shell também os veem.

Na linha de comando, as mesmas conversas são `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit`
e `comment-delete`, e `hosting review conversation` (veja [a referência da CLI](cli-reference.md)); para agentes, são
ferramentas MCP de mesmo nome (veja [MCP e agentes](mcp-and-agents.md)).
