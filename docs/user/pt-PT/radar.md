[English](../radar.md) · [Čeština](../cs/radar.md) · [Deutsch](../de/radar.md) · [Español](../es/radar.md) · [Français](../fr/radar.md) · [Italiano](../it/radar.md) · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · [Nederlands](../nl/radar.md) · [Polski](../pl/radar.md) · [Português (Brasil)](../pt-BR/radar.md) · **Português (Portugal)** · [Русский](../ru/radar.md) · [Svenska](../sv/radar.md) · [Türkçe](../tr/radar.md) · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# O Radar

O Radar é um separador próprio, logo a seguir a Início: uma única lista do que está à sua espera em **vários repositórios**:
os pull (ou merge) requests que escreveu, os que pedem a sua revisão, as issues
que lhe estão atribuídas ou que criou, e os repositórios onde deixou trabalho em
curso. É a janela Início com a vista Radar por cima, para que nada por baixo
possa roubar um clique ou uma tecla.

Precisa de um perfil de code host (ver [Integrações de alojamento e de
gestores de issues](hosting-integration.md)) e de repositórios ligados a ele, ou de um perfil com as suas
listas de toda a conta ligadas (ver «Para além do âmbito» abaixo).
Um repositório que não está ligado nunca é consultado: mostra antes uma linha
**Ligar...** (ver «Repositórios com algo a dizer» abaixo).

![O Radar sobre dois repositórios: um pedido afixado, depois Aguardam a minha revisão, Os meus pull requests, Atribuídos a mim, issues e trabalho em curso](../images/radar.png)

**Rever...** numa linha de pull ou merge request (ou `e`) abre o repositório com a revisão do pedido na própria aplicação
([Rever um pull request](review.md)).

## Abri-lo

- **`Alt+R`** (reatribuível: `global.radar`), de qualquer sítio.
- A paleta: `Radar: abrir`.
- O separador **Radar**, logo a seguir a Início na barra de separadores.
- A linha **Radar** do menu **`+`**, enquanto o separador está fechado.

A partir do separador de um repositório isto leva-o à janela Início com o Radar
aberto; `Esc` ou **Voltar** regressa à lista de repositórios, e o `⌂` faz o mesmo.

## Fechar o separador ou desligar o Radar

O separador fecha-se como um separador de repositório: o seu **x**, um clique do meio, `Ctrl+W` enquanto é mostrado, ou o comando da paleta `Radar: fechar separador`. Início é o único separador que não se pode fechar. Um separador fechado continua fechado nessa sessão, mesmo após reiniciar, até abrir o Radar de novo (`Alt+R`, `Radar: abrir` ou o menu `+`); volta então logo a seguir a Início. Fechá-lo não altera nenhuma definição.

Se nunca o usa, desligue-o em **Definições > Geral > Ativar o Radar** (`general.radar_enabled`, ver a [referência de configuração](config-reference.md)). Desligado, o separador, os seus comandos da paleta e `Alt+R` desaparecem e o GitPoulpi não obtém nada para ele. Aplica-se de imediato, sem reiniciar.

## O que cobre: o âmbito

A lista **Âmbito** escolhe que repositórios são consultados:

- uma **área de trabalho**, pelo nome (os seus repositórios, mesmo os que não têm
  separador aberto);
- **Separadores abertos**: os repositórios abertos como separadores desta sessão;
- **Pasta de clonagem predefinida** e **Pasta de navegação predefinida**: todos os repositórios que existem na pasta definida
  nas [Definições](config-reference.md) (`repos.clone_root`, `repos.browse_root`), tenham ou não um separador.

Abre no último âmbito que escolheu aqui (guardado em `radar.toml`), desde que ainda exista; antes
da sua primeira escolha, ou quando essa área de trabalho foi eliminada, na área de trabalho ativa da sessão, senão nos separadores
abertos. A escolha vale apenas para esta vista: não muda a área de trabalho ativa da janela. Quando dois repositórios têm o
mesmo nome de pasta, a pasta de origem é acrescentada à etiqueta para os poder
distinguir.

### Os dois âmbitos de pasta

Um âmbito de pasta procura repositórios **a dois níveis de profundidade**: as pastas que estão diretamente nele e as pastas dentro
daquelas que não são, elas próprias, repositórios (`~/src/work/app`). As pastas ocultas são ignoradas, nunca se entra num
repositório e são listados no máximo 200. A pesquisa corre em segundo plano, nunca na thread que desenha, e o resultado
mantém-se até atualizar: o botão Atualizar, `r`, `F5` (a tecla de atualização da janela, reatribuível) ou `Radar: atualizar` voltam a pesquisar a pasta (um repositório clonado
entretanto aparece).

O alojamento continua a ser opcional por repositório: um repositório da pasta que **não está ligado** a um alojamento nunca é
consultado. Só contribui com a sua linha de trabalho em curso, tem uma linha `não ligado` com **Ligar...** em
Repositórios (abre a secção de pedidos desse repositório, onde escolhe o fornecedor e o
projeto), e a barra de estado conta-os (`3 não ligados (só trabalho em curso)`).

Uma pasta que não está definida aparece esbatida na lista (`Pasta de navegação predefinida (defina nas Definições)`); escolhê-la
abre a sua linha nas Definições (Definições > Geral) e deixa o âmbito como estava. A pasta de clonagem conta como definida
pela sua predefinição de cada SO (a pasta em que o diálogo Clonar começa) enquanto essa pasta existir; a pasta de
navegação não tem predefinição. A lista de âmbitos é lida das definições sempre que abre, por isso uma alteração feita
lá aplica-se de imediato.

## A lista

| Grupo | O que contém |
|---|---|
| Afixados | Tudo o que afixou, seja de que tipo for, pela ordem em que afixou |
| Aguardam a minha revisão | Pedidos que pedem a sua revisão (e que não escreveu) |
| Os meus pedidos | Os pedidos que escreveu; a etiqueta diz `rascunho` para um rascunho |
| Atribuídos a mim | Pedidos que lhe estão atribuídos |
| Mencionam-me | Pedidos abertos e issues que o mencionam e que ainda não estão num grupo acima (uma issue que não lhe está atribuída nem foi criada por si só aparece aqui quando o menciona) |
| Outros pedidos | Os restantes pedidos abertos |
| Issues atribuídas a mim | Issues abertas que lhe estão atribuídas |
| Issues criadas por mim | Issues abertas que criou |
| Trabalho em curso | Repositórios do âmbito com alterações não commitadas ou commits por enviar |
| Adiados | Itens que escondeu por algum tempo; recolhido até o abrir |
| Repositórios | Repositórios com algo a dizer (offline, token a renovar, não ligado) |

As palavras «pull request» e «merge request» seguem a definição habitual de
terminologia e o host.

Cada linha mostra o tipo de item (`⇄` pedido, `●` issue, `✎` trabalho em curso), a
etiqueta do repositório (na cor da área de trabalho), o número, o título, o autor,
a idade e, onde houver espaço, as etiquetas. Uma linha de trabalho em curso mostra
a branch e o que está à espera nela: `✎` ficheiros alterados, `+` ficheiros novos,
`↑` commits por enviar, `↓` commits por puxar.

As contagens estão em cada título. Dentro de um grupo, o item atualizado mais
recentemente vem primeiro.

A barra de estado por baixo do filtro mostra um indicador de atividade quando os
hosts demoram mais de um momento a responder. As listas em cache (a mesma cache que
a barra lateral usa) aparecem logo e são substituídas à medida que as respostas
chegam.

### Colunas

A etiqueta do repositório, o `#` (número e marcas de verificação), o autor e a idade são **colunas redimensionáveis**. Em automático, a coluna do repositório tem a
largura do nome mais comprido no ecrã (até 40 células, e no máximo um terço da janela), pelo que `owner/name` e o
`group/sub/project` de uma lista de conta não são cortados num terminal normal. O título fica com o que sobra. Um
nome que não cabe termina em reticências (`group/sub...`), e:

- **Pare o ponteiro** sobre um repositório, título, autor ou lista de etiquetas cortado e uma dica mostra o texto
  inteiro. Selecione a linha e a linha de estado por baixo do filtro nomeia também o projeto e o título completos,
  pelo que o teclado vê o mesmo.
- **Arraste o separador** (a célula logo a seguir ao nome do repositório, ao `#`, ou mesmo antes do autor ou da idade) para dimensionar
  a coluna; **duplo clique** nele para a ajustar de novo. A lista tem uma **linha de cabeçalho** (repositório, `#`,
  título, autor, idade) e os separadores seguem `ui.column_resize` como os do Log: com `line` (a
  predefinição) um `│` esbatido fica no separador na linha do cabeçalho e, enquanto o ponteiro está no separador
  (no cabeçalho ou em qualquer linha da lista), enquanto a coluna está escolhida com as teclas, ou enquanto arrasta, é desenhada uma linha fina
  de destaque ao longo de toda a lista; um arrastamento a partir de qualquer ponto dela redimensiona. Ao largar, a linha desaparece a menos que o
  ponteiro ainda esteja no separador (então some com o ponteiro): um arrastamento com o rato nunca deixa uma coluna escolhida. Com `header` não há marca nem linha: a primeira
  célula a seguir ao separador, na linha do cabeçalho, acende-se e é o único ponto de onde se arrasta. O ponteiro passa
  a ser um ponteiro de redimensionamento sobre um separador nos terminais que o suportam. A etiqueta de estado tem
  largura fixa, e a idade também (a maior idade mostrada, até a dimensionar; uma idade cortada por uma coluna estreita mostra a
  data completa numa dica), pelo que o autor começa na mesma célula em todas as linhas.
- Ou use as teclas: **`C`** escolhe uma coluna (outra vez, ou `Tab`, para a seguinte), **`]`** alarga-a duas
  células, **`[`** estreita-a, **`0`** repõe todas as colunas em automático, `Esc` sai. A paleta tem o
  mesmo: `Radar: escolher coluna`, `Radar: alargar coluna`, `Radar: estreitar coluna`, `Radar: repor larguras das colunas`.

As larguras ficam guardadas em `radar.toml` (`[columns]`, `repo`, `number`, `author` e `age`, em células) e sobrevivem a um reinício.
São os mesmos gestos de dimensionamento das colunas do Log.

### Verificações e conflitos

Uma linha de pedido leva duas pequenas marcas a seguir ao número quando o host disse alguma coisa:

| Marca | Significado |
|---|---|
| `✓` | as verificações do commit de head passaram |
| `✗` | uma verificação falhou |
| `●` | as verificações estão em fila ou em curso |
| `!` (a seguir) | o pedido está em conflito com a sua branch de destino |

Pouse o ponteiro nas marcas e uma dica di-lo por palavras («Verificações falharam», «Em conflito com main»);
selecione a linha e a barra de estado sob o filtro diz o mesmo, para que o teclado também o veja. Um pedido
sem marca é um sobre o qual o host nada disse (sem CI configurado, ou não perguntado).

- O **GitLab** envia o estado do pipeline e o indicador de conflito com a lista de merge requests: todas as linhas têm
  as suas marcas.
- O **GitHub** não: a lista não tem nenhum dos dois. O GitPoulpi pergunta-os por pedido (o próprio pedido para
  o seu estado de integração, o estado combinado do commit de head e as suas execuções de verificações: três chamadas), para no máximo
  **15 pedidos abertos por repositório**, os seus primeiro. Se o GitHub recusar uma dessas chamadas (um limite de pedidos),
  os restantes ficam sem marca até à atualização seguinte. As respostas são guardadas com a cache de alojamento.

- O **Gitea** e o **Forgejo** são lidos como o GitHub: a lista traz o indicador de conflito (`mergeable`), o estado
  dos commits é pedido por pedido (no máximo 15 por repositório, os seus primeiro), e as issues são os mesmos grupos.
  As menções usam a pesquisa do servidor (`mentioned_by`).

### Menções

**Mencionam-me** e o filtro **Mostrar: Mencionam-me** usam o que o host sabe sobre menções. O GitHub é
consultado através da sua pesquisa (`mentions:`), o GitLab através da sua lista de tarefas: só contam as menções ainda **pendentes**
aí, pelo que uma tarefa que marcou como feita sai do grupo.

Os pedidos fechados e integrados e as issues fechadas não são listados.

## Vistas e filtros

Três listas e um campo de texto restringem a lista. Cada escolha é uma lista
pendente: clique nela ou use a sua tecla.

- **Vista** (`v`): Tudo, pedidos, Issues, WIP (trabalho em curso), Adiados.
- **Mostrar** (`f`): Todos, Meus (pedidos que escreveu, issues que criou),
  Atribuídos a mim, Revisão pedida, Mencionam-me.
- **Âmbito** (`w`): ver acima.
- **Texto do filtro** (`/` ou `Ctrl+F`): correspondência aproximada no repositório,
  número, título, autor, branch e etiquetas. `Enter` devolve as teclas à lista e
  mantém o texto, `Esc` limpa-o.

`Ctrl+N` mostra um selo no filtro, em cada lista pendente, nos botões Atualizar e Voltar e em cada título de secção no ecrã. Prima o carácter de um selo: o filtro recebe o teclado, uma lista pendente abre-se, um botão é executado e um título recebe a seleção (`Enter` ou `Space` recolhe-o depois). `Esc` cancela. Veja [Selos](keybindings.md#selos).

## Ações

Selecione uma linha e use o seu menu (clique com o botão direito, o `⋮` no fim da
linha, `.` ou `Menu`), uma tecla, ou a paleta (`Radar: ...`). Uma ação do rato
tem sempre uma tecla e um equivalente na paleta.

| Ação | Tecla | Paleta |
|---|---|---|
| Abrir repositório (o seu separador abre, ou é ativado) | `Enter` / clique duplo | `Radar: abrir repositório` |
| Rever... (um pedido; um pedido não clonado pergunta como) | `e` | `Radar: rever pedido...` |
| Rever um pedido através da API do host (sem clone) | `a` na pergunta | `Radar: rever através da API` |
| Clonar o repositório e depois rever o pedido | `c` na pergunta | `Radar: clonar e rever...` |
| Abrir no navegador | `o` | `Radar: abrir no navegador` |
| Fazer checkout da branch (um pedido) | `c` | `Radar: fazer checkout da branch` |
| Criar worktree... (um pedido) | `t` | `Radar: criar worktree...` |
| Mostrar no repositório (a sua lista) | `s` | `Radar: mostrar no repositório` |
| Iniciar trabalho na issue... (uma issue) | `n` | `Radar: iniciar trabalho na issue...` |
| Afixar no topo, ou Desafixar | `p` / clique no `★` | `Radar: afixar ou desafixar` |
| Editar responsáveis... / revisores... / etiquetas..., Definir marco... (GitHub, GitLab) | menu da linha | `Radar: editar responsáveis...`, `Radar: editar revisores...`, `Radar: editar etiquetas...`, `Radar: definir marco...` |
| Adiar... | `z` | `Radar: adiar...` |
| Deixar de adiar | `u` | |
| Mostrar ou ocultar os pedidos de repositórios não clonados | `a` | `Radar: mostrar pedidos não clonados` |
| Mostrar ou ocultar os rascunhos das linhas de toda a conta | `d` | `Radar: mostrar rascunhos` |
| Ligar os pedidos de revisão de toda a conta de um host | `T` / `Enter` na sua linha de sugestão / **Ativar** | `Radar: ativar os pedidos de revisão da conta...` |
| Atualizar | `r`, `F5` | `Radar: atualizar` |
| O menu da linha | `.` | `Radar: menu da linha...` |
| Voltar ao início | `Esc` | `Radar: voltar ao início` |

As ações que precisam da janela do próprio repositório (checkout, worktree, mostrar,
iniciar trabalho, ligar, os seletores de metadados) mudam primeiro para o seu separador e executam-se aí
quando ele tiver carregado:

- **Fazer checkout** usa o checkout da própria barra lateral: a branch quando é
  conhecida localmente, o head do pedido obtido para uma branch local caso contrário
  (o de um fork, por exemplo).
- **Criar worktree...** abre o diálogo Criar worktree sobre a branch do pedido.
- **Mostrar no repositório** seleciona o item na lista PULL REQUESTS ou ISSUES.
- **Iniciar trabalho na issue** abre o pedido de Nova branch com um nome feito a
  partir da issue (`12-fix-the-thing`).

### Afixar e adiar

Um **afixado** mantém um item no topo. Um **adiamento** esconde-o por **1 hora**,
**4 horas**, **Até amanhã** (08:00 da manhã seguinte, na sua hora local, mesmo que ainda não sejam as 08:00 agora) ou **Até mudar** (o host comunica uma
atualização do item ou, numa linha de trabalho em curso, os seus contadores mexem-se).
Ambos ficam guardados em `radar.toml` na sua pasta de configuração, indexados
por host, repositório, tipo e número, e são partilhados por todas as sessões da
máquina.

### Repositórios com algo a dizer

Um repositório com um problema ganha uma linha própria no fim da lista, e os seus
itens em cache continuam no ecrã:

- **offline, há 5m**: não foi possível contactar o host;
- **Religar**: o token falta ou foi recusado; o botão abre Definições > Integrações;
- **em pausa nesta ronda**: o host pediu para abrandar (ou recusou o token), pelo
  que o GitPoulpi não o voltou a consultar para os outros repositórios desta ronda;
- **não ligado, GitHub sugerido**: o remoto do repositório corresponde a um perfil,
  mas o alojamento é opcional por repositório, por isso nada é consultado até o
  ligar. **Ligar...** leva-o à secção PULL REQUESTS do repositório, onde escolhe o
  fornecedor e o projeto.

No máximo quatro repositórios são consultados ao mesmo tempo, e nunca na thread que
desenha o ecrã.

## Para além do âmbito: linhas de toda a conta

O âmbito cobre os repositórios que clonou. Um pedido que espera pela sua revisão num repositório que
nunca clonou é invisível aí. As **linhas de toda a conta** perguntam ao próprio host, uma vez por perfil, e
juntam-se à lista seja qual for o âmbito: dizem respeito a si, não a uma pasta.

Estão **desligadas por predefinição** (a pesquisa de um host custa quota de API). Enquanto um code host ligado não tem
nenhuma das suas listas ligada, o topo da lista mostra uma linha para ele:

`GitHub: listar os pedidos à espera da sua revisão, também em repositórios que não clonou  [Ativar]`

Clique em **Ativar**, prima `Enter` nessa linha, prima `T`, ou execute `Radar: ativar os pedidos de revisão da conta...`:
escreve `integrations.radar.review_requests = true` para esse perfil em `config.toml` e
pergunta logo ao host. As outras listas (todos os pedidos abertos dos seus namespaces, issues atribuídas a si,
rascunhos, namespaces) definem-se com `gitpoulpi hosting integrations radar` ou nas definições do perfil; ver
a [referência de configuração](config-reference.md) e a [referência da CLI](cli-reference.md). Respondem o GitHub, o GitLab,
o Gitea, o Forgejo, o Azure DevOps, o Bitbucket Data Center e o Gerrit; o Bitbucket Cloud não tem endpoint de toda a conta,
por isso os seus pedidos só aparecem para os repositórios ligados do âmbito.

Os **Namespaces** do perfil limitam todas as suas listas a alguns repositórios (Definições > Integrações > Radar: uma área de texto com uma entrada por linha e uma caixa **Expressões regulares**). Uma linha corresponde ao caminho completo de um repositório quando é igual a ele ou é um prefixo de caminho que termina numa `/`: `ptm` mantém `ptm/produits/x` e descarta `ptmx/y` e `formation/y`. Com a caixa ligada, cada linha é uma expressão regular aplicada ao caminho completo tal como a escreve (`^ptm/(produits|infra)/`); uma linha que não seja válida aparece sob o campo e não é guardada. Vazio significa sem limite.

Como as linhas são colocadas:

- Uma linha que o âmbito já lista (mesmo host, projeto, tipo e número) aparece uma só vez: **ganha a linha do
  âmbito**.
- Uma linha cujo projeto clonou nalgum sítio (repositórios recentes, áreas de trabalho, as pastas que os âmbitos
  de pasta encontraram; reconhecido pelo URL remoto) traz esse clone: **Enter**, checkout, worktree e as
  outras abrem-no como qualquer linha.
- Qualquer outra linha é **não clonada**: a etiqueta do repositório fica esmaecida e mostra o projeto (`acme/app`), e
  a barra de estado diz `Não clonados` quando está selecionada. **Abrir no navegador** funciona, **Rever** pergunta como
  a rever (abaixo), e checkout, worktree, mostrar no repositório e iniciar trabalho dizem
  `clone primeiro o repositório`.

### Rever um pedido

**Rever** (`e`) num pedido dos repositórios do próprio âmbito abre o separador desse repositório com a revisão
nele. Num pedido de uma linha de toda a conta, o caminho depende de haver um clone conhecido e da definição
**Pedido de um repositório não clonado** do perfil (Definições > Integrações, `review_mode`
na [referência de configuração](config-reference.md)):

| Definição | Há um clone conhecido | Nenhum clone conhecido |
|---|---|---|
| Perguntar (predefinição) | pergunta: **Abrir o clone local** (`l`) ou **Rever através da API** (`a`) | pergunta: **Clonar e depois rever** (`c`) ou **Rever através da API** (`a`) |
| Cloná-lo | abre o clone | abre o diálogo Clonar |
| Rever através da API | revê através da API | revê através da API |

A pergunta é um pequeno diálogo: as letras escolhem, **Enter** escolhe a linha realçada, `Esc`, **Cancelar**,
o `✕` e um clique fora fecham-no, e **Lembrar para GitHub** (`r`) escreve a definição do perfil para que a
pergunta não volte a ser feita (a linha das Definições volta a alterá-la). `Ctrl+N` marca os seus controlos.

- **Rever através da API** abre a revisão na janela Início, por cima do Radar, com o código obtido do
  host (ver [Rever sem um clone](review.md#rever-sem-um-clone)). `Esc` ou o `✕` devolve
  o Radar; as linhas *Revisão: ...* da paleta funcionam sobre ela.
- **Clonar e depois rever** abre o diálogo Clonar na conta ligada de onde veio a linha (a mesma lista de repositórios,
  agrupada por proprietário), com o repositório do pedido selecionado e a pasta de clone predefinida preenchida;
  altere qualquer um, depois clone, com o mesmo início de sessão de qualquer clone dessa conta. Se o repositório não
  estiver na lista da conta (não é membro dele), a lista fica aberta com a pesquisa pelo seu nome e uma nota di-lo;
  num host que não consegue listar repositórios, o diálogo abre em **Clonar com URL** com o endereço preenchido.
  Quando o clone termina, o novo repositório fica ligado ao host
  (a sua origem de PULL REQUESTS, e a de ISSUES quando o host tem issues e nenhuma está definida), o seu separador abre e
  a revisão desse pedido fica aberta nele. Cancelar o diálogo, ou um clone que falha, não liga nada.
- Os pedidos do Azure DevOps e do Bitbucket não têm revisão na aplicação: **Rever** abre a página do pedido no
  navegador e di-lo na barra de estado.

As linhas da paleta *Radar: rever através da API* e *Radar: clonar e rever...* dispensam a pergunta.

Duas caixas ao lado do filtro (aparecem quando uma lista de toda a conta está a ser consultada), guardadas em
`radar.toml`:

| Caixa | Tecla | Predefinição | Efeito |
|---|---|---|---|
| `[x] Não clonados` | `a` | ligada | Lista as linhas de repositórios sem clone conhecido. Desligada, ficam só as linhas de clones conhecidos. |
| `[ ] Rascunhos` | `d` | desligada | Lista os pedidos em rascunho das linhas de toda a conta. Os rascunhos dos repositórios do próprio âmbito são sempre listados. |

Ambas estão nos emblemas do `Ctrl+N` e na paleta (`Radar: mostrar pedidos não clonados`, `Radar: mostrar rascunhos`).

A resposta de um perfil é guardada em `radar-account.json` durante **5 minutos**: voltar a abrir o Radar dentro desse
tempo mostra as linhas sem perguntar ao host. **Atualizar** (`r` ou `F5`) pergunta sempre: todos os repositórios e todas as listas de toda a conta, seja qual for a idade da última resposta. Um host que pediu para abrandar
ou recusou o token não volta a ser consultado na mesma ronda. No máximo 300 linhas são mantidas por perfil e tipo,
e uma linha sob **Repositórios** diz quantas mais existiam, ou porque é que um perfil não pôde ser lido
(**Religar** abre Definições > Integrações).

## Limites

- As linhas de toda a conta vêm de perfis do GitHub, do GitLab (incluindo self-managed), do Gitea, do Forgejo, do Azure DevOps,
  do Bitbucket Data Center e do Gerrit (não do Bitbucket Cloud). As issues atribuídas a si vêm dos primeiros
  cinco; o Bitbucket e o Gerrit não têm issues.
- As listas por repositório usam perfis do GitHub, do GitLab (incluindo self-managed), do Gitea, do Forgejo, do Azure DevOps,
  do Bitbucket e do Gerrit; as verificações, os conflitos e as menções só são lidos para os primeiros quatro.
- As menções do GitLab vêm apenas das tarefas pendentes; as verificações do GitHub são pedidas para 15 pedidos por repositório.
