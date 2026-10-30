[English](../radar.md) · [Čeština](../cs/radar.md) · [Deutsch](../de/radar.md) · [Español](../es/radar.md) · [Français](../fr/radar.md) · [Italiano](../it/radar.md) · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · [Nederlands](../nl/radar.md) · [Polski](../pl/radar.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/radar.md) · [Русский](../ru/radar.md) · [Svenska](../sv/radar.md) · [Türkçe](../tr/radar.md) · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# O Radar

O Radar é uma aba própria, logo depois de Início: uma só lista do que está esperando por você em **vários repositórios**: as pull (ou merge)
requests que você escreveu, as que pedem a sua revisão, as issues atribuídas a você ou criadas por você e
os repositórios em que você deixou trabalho em andamento. É a janela Início com a
visão Radar por cima, para que nada por baixo possa roubar um clique ou uma tecla.

Ele precisa de um perfil de host de código (veja [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md))
e de repositórios vinculados a ele, ou de um perfil com as listas de toda a conta ativadas (veja "Além do escopo: linhas de toda a conta" abaixo).
Um repositório que não está vinculado nunca é consultado: ele mostra uma
linha **Vincular...** (veja "Repositórios com algo a dizer" abaixo).

![O Radar sobre dois repositórios: um item fixado, depois Aguardando minha revisão, Meus pull requests, Atribuídos a mim, issues e trabalho em andamento](../images/radar.png)

**Revisar...** em uma linha de pull ou merge request (ou `e`) abre o repositório com a revisão da solicitação no aplicativo
([Revisando um pull request](review.md)).

## Abrindo

- **`Alt+R`** (reatribuível: `global.radar`), de qualquer lugar.
- A paleta: `Radar: abrir`.
- A aba **Radar**, logo depois de Início na barra de abas.
- A linha **Radar** do menu **`+`**, enquanto a aba está fechada.

De uma aba de repositório isso leva você à janela Início com o Radar aberto; `Esc` ou **Voltar** retorna
à lista de repositórios, e o `⌂` faz o mesmo.

## Fechando a aba, ou desativando o Radar

A aba fecha como uma aba de repositório: pelo seu **x**, com um clique do botão do meio, com `Ctrl+W` enquanto ela
aparece, ou pelo comando da paleta `Radar: fechar aba`. Início é a única aba que não fecha. Uma aba fechada
continua fechada nessa sessão e entre reinícios, até você abrir o Radar de novo (`Alt+R`, `Radar: abrir`, ou
o menu `+`); ela então volta logo depois de Início. Fechar a aba não muda nenhuma configuração.

Se você nunca o usa, desative-o em **Configurações > Geral > Ativar o Radar**
(`general.radar_enabled`, veja a [referência de configuração](config-reference.md)). Desativado, a aba, os
comandos da paleta e o `Alt+R` somem e o GitPoulpi não busca nada para ele. Vale na hora, sem reiniciar.

## O que ele cobre: o escopo

A lista suspensa **Escopo** escolhe quais repositórios são consultados:

- um **espaço de trabalho**, pelo nome (seus repositórios, mesmo os sem aba aberta);
- **Abas abertas**: os repositórios abertos como abas desta sessão;
- **Pasta de clone padrão** e **Pasta de navegação padrão**: todo repositório encontrado na pasta definida nas
  [Configurações](config-reference.md) (`repos.clone_root`, `repos.browse_root`), tenha ou não uma aba.

Ele abre no último escopo que você escolheu aqui (guardado em `radar.toml`), desde que ainda exista; antes
da sua primeira escolha, ou quando esse espaço de trabalho foi excluído, no espaço de trabalho ativo da sessão, senão nas
abas abertas. A escolha vale só para esta visão: ela não muda o espaço de trabalho ativo da janela. Quando dois repositórios têm o mesmo nome de pasta, a pasta pai é acrescentada à etiqueta para
que você os distinga.

### Os dois escopos de pasta

Um escopo de pasta procura repositórios **em dois níveis**: as pastas diretamente nela e as pastas dentro
daquelas que não são repositórios (`~/src/work/app`). Pastas ocultas são ignoradas, um repositório nunca é
entrado e no máximo 200 repositórios são listados. A busca roda em segundo plano, nunca na thread que
desenha, e o resultado é mantido até você atualizar: o botão Atualizar, `r` ou `Radar: atualizar`
procura na pasta de novo (um repositório clonado desde então aparece).

A hospedagem continua opcional por repositório: um repositório da pasta que **não está vinculado** a um host
nunca é consultado. Ele só contribui com a linha de trabalho em andamento, tem uma linha `não vinculado` com
**Vincular...** em Repositórios (ela abre a seção de pull requests desse repositório, onde você escolhe o
provedor e o projeto), e a linha de status os conta (`3 não vinculados (só trabalho em andamento)`).

Uma pasta que não está definida aparece em cinza na lista (`Pasta de navegação padrão (defina em Configurações)`);
escolhê-la abre a linha dela nas Configurações (Configurações > Geral) e deixa o escopo como estava. A pasta de
clone conta como definida pelo padrão do sistema (a pasta em que o diálogo Clonar começa) desde que essa pasta
exista; a pasta de navegação não tem padrão. A lista de escopos é lida das configurações a cada vez que abre,
então uma mudança feita lá vale na hora.

## A lista

| Grupo | O que contém |
|---|---|
| Fixados | Tudo o que você fixou, de qualquer tipo, na ordem em que fixou |
| Aguardando minha revisão | Solicitações que pedem a sua revisão (e que você não escreveu) |
| Meus pull requests | As solicitações que você escreveu; a etiqueta diz `rascunho` para um rascunho |
| Atribuídos a mim | Solicitações atribuídas a você |
| Menções a mim | Solicitações e issues abertas que mencionam você e que não estão já em um grupo acima (uma issue que não é atribuída a você nem criada por você só aparece aqui quando menciona você) |
| Outros pull requests | As demais solicitações abertas |
| Issues atribuídas a mim | Issues abertas atribuídas a você |
| Issues criadas por mim | Issues abertas que você criou |
| Trabalho em andamento | Repositórios do escopo com alterações não commitadas ou commits não enviados |
| Adiados | Itens que você ocultou por um tempo; recolhidos até você abri-los |
| Repositórios | Repositórios com algo a dizer (offline, token a renovar, não vinculado) |

As palavras "pull request" e "merge request" seguem a configuração usual de terminologia e o host.

Cada linha mostra o tipo do item (`⇄` solicitação, `●` issue, `✎` trabalho em andamento), a etiqueta do
repositório (na cor do espaço de trabalho), o número, o título, o autor, a idade e, onde houver espaço, os
rótulos. Uma linha de trabalho em andamento mostra a branch e o que aguarda nela: `✎` arquivos alterados,
`+` arquivos novos, `↑` commits a enviar, `↓` commits a puxar.

As contagens ficam em cada título. Dentro de um grupo, o item atualizado mais recentemente vem primeiro.

A linha de status sob o filtro mostra um indicador de atividade quando os hosts demoram mais que um instante
para responder. As listas em cache (o mesmo cache que a barra lateral usa) aparecem na hora e são
substituídas conforme as respostas chegam.

### Colunas

O chip do repositório, o `#` (número e marcas de verificação), o autor e a idade são **colunas redimensionáveis**. Deixada em automático, a coluna do repositório tem a largura
do maior nome na tela (até 40 células, e no máximo um terço da janela), então `owner/name` e o
`group/sub/project` da lista de uma conta não são cortados em um terminal normal. O título fica com o que sobra. Um nome que
não cabe termina em reticências (`group/sub...`), e:

- **Pare o ponteiro** sobre um repositório, título, autor ou lista de rótulos cortado e uma dica mostra o texto inteiro. Selecione
  a linha e a linha de status sob o filtro também nomeia o projeto e o título completos, então o teclado vê
  o mesmo.
- **Arraste o separador** (a célula logo após o nome do repositório, após o `#`, ou logo antes do autor ou da idade) para dimensionar a
  coluna; **clique duas vezes** nele para ajustar a coluna de novo. A lista tem uma **linha de cabeçalho** (repositório, `#`,
  título, autor, idade) e os separadores seguem `ui.column_resize` como os do Log: com `line` (o
  padrão) um `│` esmaecido fica no separador na linha do cabeçalho e, enquanto o ponteiro está sobre o separador
  (no cabeçalho ou em qualquer linha da lista), enquanto a coluna está escolhida pelas teclas, ou enquanto você arrasta, uma linha fina
  de destaque é desenhada em toda a lista; um arrasto a partir de qualquer ponto dela redimensiona. Ao soltar, a linha some, a menos que o
  ponteiro ainda esteja sobre o separador (então ela some com o ponteiro): um arrasto do mouse nunca deixa uma coluna escolhida. Com `header` não há marca nem linha: a primeira
  célula depois do separador, na linha do cabeçalho, se acende e é o único lugar de onde arrastar. O ponteiro vira
  um ponteiro de redimensionamento sobre um separador nos terminais que suportam. O chip de estado tem uma
  largura fixa, e a idade também (a maior idade mostrada, até você dimensioná-la; uma idade cortada por uma coluna estreita mostra a
  data completa em uma dica), então o autor começa na mesma célula em todas as linhas.
- Ou use as teclas: **`C`** escolhe uma coluna (de novo, ou `Tab`, para a próxima), **`]`** a alarga em duas
  células, **`[`** a estreita, **`0`** devolve todas as colunas ao automático, `Esc` sai. A paleta tem o
  mesmo: `Radar: escolher coluna`, `Radar: alargar coluna`, `Radar: estreitar coluna`, `Radar: redefinir larguras das colunas`.

As larguras ficam no `radar.toml` (`[columns]`, `repo`, `number`, `author` e `age`, em células) e sobrevivem a uma reinicialização. São
os mesmos gestos de dimensionamento das colunas do Log.

### Verificações e conflitos

Uma linha de solicitação leva duas pequenas marcas depois do seu número quando o host disse alguma coisa:

| Marca | Significado |
|---|---|
| `✓` | as verificações do commit head foram aprovadas |
| `✗` | uma verificação falhou |
| `●` | as verificações estão na fila ou em andamento |
| `!` (ao lado dela) | a solicitação tem conflito com a sua branch de destino |

Pare o ponteiro sobre as marcas e uma dica as diz em palavras (“Verificações com falha”, “Conflita com main”);
selecione a linha e a linha de status sob o filtro diz o mesmo, de modo que o teclado também vê. Uma solicitação
sem marca é uma sobre a qual o host não disse nada (sem CI configurado, ou não consultado).

- O **GitLab** envia o status do pipeline e o sinalizador de conflito com a lista de merge requests: toda linha tem
  as suas marcas.
- O **GitHub** não envia: a lista não tem nenhum dos dois. O GitPoulpi pergunta por solicitação (a própria solicitação
  pelo estado do merge, o status combinado do commit head e as suas execuções de verificação: três chamadas), para no máximo
  **15 solicitações abertas por repositório**, as suas primeiro. Se o GitHub recusar uma dessas chamadas (um limite de taxa),
  as demais ficam sem marca até a próxima atualização. As respostas são salvas com o cache de hospedagem.

- O **Gitea** e o **Forgejo** são lidos como o GitHub: a lista traz o sinalizador de conflito (`mergeable`), o status
  do commit é consultado por solicitação (no máximo 15 por repositório, as suas primeiro) e as issues são os mesmos
  grupos. As menções usam a busca do servidor (`mentioned_by`).

### Menções

**Menções a mim** e o filtro **Mostrar: Menções a mim** usam o que o host sabe sobre menções. O GitHub é
consultado pela sua busca (`mentions:`), o GitLab pela sua lista de tarefas: só contam as menções ainda **pendentes**
lá, então uma tarefa que você marcou como concluída sai do grupo.

Solicitações fechadas e mescladas e issues fechadas não são listadas.

## Visões e filtros

Três listas suspensas e um campo de texto restringem a lista. Toda escolha é uma lista suspensa: clique nela
ou use a sua tecla.

- **Visão** (`v`): Tudo, solicitações, Issues, WIP (trabalho em andamento), Adiados.
- **Mostrar** (`f`): Todos, Meus (solicitações que você escreveu, issues que você criou), Atribuídos a mim,
  Revisão solicitada, Menções a mim.
- **Escopo** (`w`): veja acima.
- **Texto do filtro** (`/` ou `Ctrl+F`): coincidência aproximada com o repositório, número, título, autor,
  branch e rótulos. `Enter` devolve as teclas à lista e mantém o texto, `Esc` o limpa.

`Ctrl+N` mostra um selo no filtro, em cada lista suspensa, nos botões Atualizar e Voltar e em cada título de seção na tela. Pressione o caractere de um selo: o filtro recebe o teclado, uma lista suspensa se abre, um botão é executado e um título recebe a seleção (depois `Enter` ou `Space` o recolhe). `Esc` cancela. Veja [Selos](keybindings.md#selos).

## Ações

Selecione uma linha e use o menu dela (clique com o botão direito, o `⋮` no fim da linha, `.` ou `Menu`), uma
tecla ou a paleta (`Radar: ...`). Uma ação do mouse sempre tem uma tecla e um gêmeo na paleta.

| Ação | Tecla | Paleta |
|---|---|---|
| Abrir repositório (a aba dele abre, ou é ativada) | `Enter` / clique duplo | `Radar: abrir repositório` |
| Revisar... (uma solicitação; uma que não está clonada pergunta como) | `e` | `Radar: revisar solicitação...` |
| Revisar uma solicitação pela API do host (sem clone) | `a` na pergunta | `Radar: revisar pela API` |
| Clonar o repositório e depois revisar a solicitação | `c` na pergunta | `Radar: clonar e revisar...` |
| Abrir no navegador | `o` | `Radar: abrir no navegador` |
| Fazer checkout da branch (uma solicitação) | `c` | `Radar: fazer checkout da branch` |
| Criar worktree... (uma solicitação) | `t` | `Radar: criar worktree...` |
| Mostrar no repositório (a lista dele) | `s` | `Radar: mostrar no repositório` |
| Começar a trabalhar na issue... (uma issue) | `n` | `Radar: começar a trabalhar numa issue...` |
| Fixar no topo, ou Desafixar | `p` / clique na `★` | `Radar: fixar ou desafixar` |
| Editar responsáveis... / revisores... / rótulos..., Definir marco... (GitHub, GitLab) | menu da linha | `Radar: editar responsáveis...`, `Radar: editar revisores...`, `Radar: editar rótulos...`, `Radar: definir marco...` |
| Adiar... | `z` | `Radar: adiar...` |
| Parar de adiar | `u` | |
| Mostrar ou ocultar as solicitações de repositórios não clonados | `a` | `Radar: exibir solicitações não clonadas` |
| Mostrar ou ocultar os rascunhos das linhas de toda a conta | `d` | `Radar: exibir rascunhos` |
| Ativar as solicitações de revisão de toda a conta de um host | `T` / `Enter` na linha de aviso dele / **Ativar** | `Radar: ativar as solicitações de revisão da conta...` |
| Atualizar | `r` | `Radar: atualizar` |
| O menu da linha | `.` | `Radar: menu da linha...` |
| Voltar ao Início | `Esc` | `Radar: voltar ao início` |

As ações que precisam da janela do próprio repositório (checkout, worktree, mostrar, começar a trabalhar,
vincular, os seletores de metadados) trocam primeiro para a aba dele e rodam lá quando ela termina de carregar:

- **Fazer checkout** usa o checkout da própria barra lateral: a branch quando ela é conhecida localmente, senão
  o head da solicitação buscado em uma branch local (o de um fork, por exemplo).
- **Criar worktree...** abre o diálogo Criar worktree na branch da solicitação.
- **Mostrar no repositório** seleciona o item na lista PULL REQUESTS ou ISSUES.
- **Começar a trabalhar na issue** abre o prompt Nova branch com um nome feito a partir da issue
  (`12-fix-the-thing`).

### Fixando e adiando

Um **item fixado** fica no topo. **Adiar** o oculta por **1 hora**, **4 horas**, **Até amanhã** (08:00 da manhã seguinte, na sua hora local, mesmo quando agora ainda não são 08:00) ou
**Até mudar** (o host informa uma atualização do item ou, em uma linha de trabalho em andamento, os
contadores dela se movem). Ambos ficam em `radar.toml` na sua pasta de configuração, indexados por host,
repositório, tipo e número, e são compartilhados por todas as sessões da máquina.

### Repositórios com algo a dizer

Um repositório com um problema ganha uma linha própria no fim da lista, e seus itens em cache continuam na
tela:

- **offline, há 5m**: o host não pôde ser alcançado;
- **Reconectar**: o token está ausente ou foi recusado; o botão abre Configurações > Integrações;
- **pausado nesta rodada**: o host pediu para ir mais devagar (ou recusou o token), então o GitPoulpi não o
  consultou de novo para os outros repositórios desta rodada;
- **não vinculado, GitHub sugerido**: o remoto do repositório combina com um perfil, mas a hospedagem é opcional
  por repositório, então nada é consultado até você vinculá-lo. **Vincular...** leva você à seção PULL REQUESTS
  do repositório, onde você escolhe o provedor e o projeto.

No máximo quatro repositórios são consultados ao mesmo tempo, e nunca na thread que desenha a tela.

## Além do escopo: linhas de toda a conta

O escopo cobre os repositórios que você clonou. Uma solicitação que aguarda a sua revisão em um repositório que
você nunca clonou fica invisível ali. As **linhas de toda a conta** consultam o próprio host, uma vez por perfil, e
entram na lista seja qual for o escopo: elas são sobre você, não sobre uma pasta.

Elas vêm **desativadas por padrão** (a busca de um host custa cota de API). Enquanto um host de código conectado não tem
nenhuma das suas listas ativada, o topo da lista mostra uma linha para ele:

`GitHub: listar as solicitações que aguardam sua revisão, inclusive em repositórios que você não clonou  [Ativar]`

Clique em **Ativar**, pressione `Enter` nessa linha, pressione `T`, ou execute `Radar: ativar as solicitações de revisão da conta...`:
isso grava `integrations.radar.review_requests = true` para esse perfil no `config.toml` e
consulta o host na hora. As outras listas (todas as solicitações abertas dos seus namespaces, issues atribuídas a você,
rascunhos, namespaces) são definidas com `gitpoulpi hosting integrations radar` ou nas configurações do perfil; veja
a [referência de configuração](config-reference.md) e a [referência da CLI](cli-reference.md). GitHub, GitLab,
Gitea, Forgejo, Azure DevOps, Bitbucket Data Center e Gerrit respondem; o Bitbucket Cloud não tem endpoint de
toda a conta, então suas solicitações aparecem só para os repositórios vinculados do escopo.

Como as linhas são colocadas:

- Uma linha que o escopo já lista (mesmo host, projeto, tipo e número) aparece uma vez: **a linha do escopo
  vence**.
- Uma linha cujo projeto você clonou em algum lugar (repositórios recentes, espaços de trabalho, as pastas que os
  escopos de pasta encontraram; combinados pela URL do remoto) carrega esse clone: **Enter**, checkout, worktree e as
  outras ações a abrem como qualquer linha.
- Qualquer outra linha é **não clonada**: o chip do repositório fica esmaecido e mostra o projeto (`acme/app`), e
  a linha de status diz `Não clonados` quando ela está selecionada. **Abrir no navegador** funciona, **Revisar** pergunta como
  revisá-la (abaixo), e checkout, worktree, mostrar no repositório e começar a trabalhar dizem
  `clone o repositório primeiro`.

### Revisando uma solicitação

**Revisar** (`e`) em uma solicitação dos repositórios do próprio escopo abre a aba desse repositório com a revisão
nela. Em uma solicitação de uma linha de toda a conta, o caminho depende de haver um clone conhecido e da
configuração **Solicitação de um repositório não clonado** do perfil (Configurações > Integrações, `review_mode`
na [referência de configuração](config-reference.md)):

| Configuração | Há um clone conhecido | Nenhum clone conhecido |
|---|---|---|
| Perguntar (padrão) | pergunta: **Abrir o clone local** (`l`) ou **Revisar pela API** (`a`) | pergunta: **Clonar e depois revisar** (`c`) ou **Revisar pela API** (`a`) |
| Clonar | abre o clone | abre o diálogo Clonar |
| Revisar pela API | revisa pela API | revisa pela API |

A pergunta é um pequeno diálogo: as letras escolhem, **Enter** escolhe a linha realçada, `Esc`, **Cancelar**,
o `✕` e um clique fora o fecham, e **Lembrar para GitHub** (`r`) grava a configuração do perfil para que a
pergunta não seja feita de novo (a linha das Configurações a altera de volta). `Ctrl+N` marca os seus controles com números.

- **Revisar pela API** abre a revisão na janela Início, sobre o Radar, com o código vindo do
  host (veja [Revisando sem um clone](review.md#revisando-sem-um-clone)). `Esc` ou o `✕` devolve
  o Radar; as linhas *Revisão: ...* da paleta funcionam nela.
- **Clonar e depois revisar** abre o diálogo Clonar na conta conectada de onde veio a linha (a mesma lista de repositórios,
  agrupada por dono), com o repositório da solicitação selecionado e a pasta de clone padrão preenchida; altere
  qualquer um e clone, com o mesmo login de qualquer clone dessa conta. Se o repositório não está na lista
  da conta (você não é membro dele), a lista fica aberta com a busca pelo nome dele e uma nota diz
  isso; para um host que não lista repositórios, o diálogo abre em **Clonar por URL** com o endereço preenchido. Quando o clone termina, o novo repositório é vinculado ao host
  (a sua fonte de PULL REQUESTS, e a de ISSUES quando o host tem issues e nenhuma está definida), a aba dele abre e
  a revisão dessa solicitação fica aberta nela. Cancelar o diálogo, ou um clone que falha, não vincula nada.
- As solicitações do Azure DevOps e do Bitbucket não têm revisão no aplicativo: **Revisar** abre a página da solicitação no
  navegador e diz isso na linha de status.

As linhas da paleta *Radar: revisar pela API* e *Radar: clonar e revisar...* pulam a pergunta.

Duas caixas de seleção ao lado do filtro (aparecem quando uma lista de toda a conta está sendo consultada), guardadas
em `radar.toml`:

| Caixa | Tecla | Padrão | Efeito |
|---|---|---|---|
| `[x] Não clonados` | `a` | ativada | Lista as linhas de repositórios sem clone conhecido. Desativada, ficam só as linhas de clones conhecidos. |
| `[ ] Rascunhos` | `d` | desativada | Lista as solicitações em rascunho das linhas de toda a conta. Os rascunhos dos repositórios do próprio escopo são sempre listados. |

Ambas estão nas marcas de `Ctrl+N` e na paleta (`Radar: exibir solicitações não clonadas`, `Radar: exibir rascunhos`).

A resposta de um perfil é guardada em `radar-account.json` por **5 minutos**: abrir o Radar de novo nesse
intervalo mostra as linhas sem consultar o host. **Atualizar** (`r`) sempre consulta. Um host que pediu para ir mais
devagar ou recusou o token não é consultado de novo na mesma rodada. No máximo 300 linhas são mantidas por perfil e tipo,
e uma linha sob **Repositórios** diz quantas mais existiam, ou por que um perfil não pôde ser lido
(**Reconectar** abre Configurações > Integrações).

## Limites

- As linhas de toda a conta vêm de perfis do GitHub, do GitLab (inclusive self-managed), do Gitea, do Forgejo, do Azure DevOps,
  do Bitbucket Data Center e do Gerrit (não do Bitbucket Cloud). As issues atribuídas a você vêm dos cinco primeiros;
  o Bitbucket e o Gerrit não têm issues.
- As listas por repositório usam perfis do GitHub, do GitLab (inclusive self-managed), do Gitea, do Forgejo, do Azure DevOps,
  do Bitbucket e do Gerrit; verificações, conflitos e menções são lidos só para os quatro primeiros.
- As menções do GitLab vêm só das tarefas pendentes; as verificações do GitHub, do Gitea e do Forgejo são consultadas para 15 solicitações por repositório.
