[English](../review.md) · [Čeština](../cs/review.md) · [Deutsch](../de/review.md) · [Español](../es/review.md) · [Français](../fr/review.md) · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · [Polski](../pl/review.md) · [Português (Brasil)](../pt-BR/review.md) · **Português (Portugal)** · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Rever um pull ou merge request

Reveja um pull request (GitHub, Gitea, Forgejo), um merge request (GitLab) ou uma alteração do Gerrit (ver [Gerrit](#gerrit))
**sem sair da janela do repositório**. A vista de revisão mostra a descrição, a conversa, os ficheiros alterados
com as discussões de comentários sob as respetivas linhas e os commits, e é onde escreve a revisão: comentários,
aprovação e pedidos de alterações. Precisa de um perfil de um desses hosts (ver
[Integrações de alojamento e de gestores de issues](hosting-integration.md)) e de um repositório ligado a ele; os outros fornecedores não oferecem
revisões. Os limites de cada host são respeitados: o GitLab não tem o parecer *Pedir alterações* e aceita uma linha por comentário.
Esta página chama a todos eles um *pedido*.

![A vista de revisão, página Resumo: o cabeçalho do pedido com os botões de metadados, a descrição e a conversa com uma aprovação submetida](../images/review-overview.png)

![A página Ficheiros: os ficheiros alterados com as caixas de visto, o diff com uma discussão de comentários sob a sua linha e uma discussão resolvida fora das linhas mostradas](../images/review-files.png)

## Abrir uma revisão

Todos os caminhos vão dar à mesma vista:

- a lista **PULL REQUESTS** da barra lateral: selecione uma linha e prima **Enter**, ou clique nela com o botão direito e escolha
  **Rever...**;
- o **Radar**: o menu da linha, **Rever...**, ou a tecla `e` (o repositório abre-se e a revisão com ele;
  um pedido de um repositório que não clonou pergunta se o quer clonar ou rever através da API,
  ver [Rever sem um clone](#rever-sem-um-clone));
- a **pesquisa da área de trabalho** (`Ctrl+Alt+F`): realce um pull ou merge request e prima **`Alt+R`**;
- a paleta: **Alojamento: rever pedido...** lista os pedidos a escolher.

A vista ocupa a janela por cima do repositório (é um modo do separador do repositório, não uma janela à parte).
**`Esc`** ou o `✕` no canto sai dela. Enquanto abre, o pedido é lido do host e o seu código é
obtido com o git em segundo plano; aparece um indicador de atividade se demorar mais do que um momento.

## O cabeçalho

Sempre visível: o número e o título, o estado (Aberto, Rascunho, Integrado, Fechado), as branches
(`main ← feature/moons`), o autor, as verificações (`✓ verificações a passar`, `✗ verificações a falhar`, `● verificações em curso`),
se integra sem problemas ou tem conflitos, o tamanho e **`3 comentários pendentes`** quando já escreveu alguns.

Os quatro botões de metadados - **Responsáveis**, **Revisores**, **Etiquetas**, **Marco** - abrem os mesmos seletores que
a barra lateral (ver [Responsáveis, revisores, etiquetas e marco](hosting-integration.md#responsáveis-revisores-etiquetas-e-marco));
o cabeçalho mostra o resultado assim que o host o aceitar. Os botões de ação: **Fazer checkout** (o checkout
habitual da branch do pedido, que também fecha a vista), **Abrir no navegador**, **Atualizar**
(**`F5`**) e **Concluir revisão**.

## Três páginas

Um controlo segmentado na margem superior muda de página: **Resumo**, **Ficheiros**, **Commits**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`**, ou um clique).

### Resumo

O Resumo é **a conversa do pedido**, a mesma cronologia e a mesma caixa de comentário que uma issue tem
([Conversas](hosting-integration.md#5c-conversas)): primeiro a descrição (*abriu isto*), depois os comentários, as
discussões no código com o seu excerto do diff, as revisões submetidas com o seu parecer (`aprovou`, `pediu
alterações`, `comentou`, `dispensada`; no GitLab os pareceres são as suas aprovações) e os eventos, das mais antigas
para as mais recentes, na janela mais recente, com **Carregar mais antigos** no topo. A caixa de comentário está
fixa sob a cronologia (**Novo comentário**): **Escrever**, **Prévia** e **Lado a lado**, **Cancelar** e **Enviar**.
**Um comentário publicado aqui vai de imediato para o host** (nada fica pendente: os pendentes são os do código,
enviados com a revisão), aparece logo na lista e ocupa o seu lugar quando o host responde; se o host o recusar, a
caixa volta a abrir-se com o seu texto e o motivo.

As teclas são as da vista de conversa: **`Up`** / **`Down`** (e `Ctrl+J` / `Ctrl+K`) movem entre entradas,
**`r`** responde no tópico da entrada selecionada, **`Q`** cita-a num novo comentário, **`c`** escreve um novo
comentário, **`e`** / **`d`** editam e eliminam os seus próprios comentários (antes de eliminar há uma pergunta), **`l`**
ou `Enter` abre as ligações de uma entrada, **`g`** o seu diagrama, **`y`** copia o seu endereço, **`o`** abre-a no
navegador, **`n`** vai para a próxima entrada por ler, **`v`** mostra ou oculta eventos, **`m`** carrega entradas mais
antigas, `Tab` alterna entre a cronologia e a caixa, e **Nota interna** (`Alt+I`) existe nos hosts que têm tais notas.
Os botões da entrada selecionada (**Responder**, **Citar**, **Editar**, **Eliminar**, **Ligações**) fazem o mesmo, e a
paleta tem as linhas *Conversa: ...* junto às *Revisão: ...*. Uma ligação pergunta antes de abrir, como em toda a
revisão.

Onde uma tecla da conversa e uma da revisão se encontram, a conversa fica com a letra nesta página: `l` (etiquetas),
`m` (marco), `Q` (abandonar, do Gerrit), `e` / `d`, `r`. As etiquetas, o marco e o abandono ficam nos botões do
cabeçalho e na paleta; `a` (responsáveis), `p` (revisores), `f` (**Concluir revisão**), `b` (**Fazer checkout**), `P`,
`D`, `U` e `Alt+1` / `Alt+2` / `Alt+3` mantêm aqui o seu significado. Enquanto a caixa tem o teclado, todas as teclas
são da caixa, exceto `Alt+1` / `Alt+2` / `Alt+3`; `Esc` devolve o teclado à cronologia e um segundo `Esc` fecha a
revisão.

Um comentário escrito noutra janela ou pela CLI aparece aqui por si: cada escrita do GitPoulpi (esta janela incluída)
avisa a sessão, e as janelas que mostram o mesmo pedido leem-no de novo.

A caixa de comentário é a caixa de texto Markdown partilhada com a [vista de conversa](hosting-integration.md#escrever)
(também a caixa de uma resposta a uma discussão, de um comentário numa linha e do resumo de **Concluir revisão**):
`Ctrl+Enter` (também `Ctrl+S`, `Alt+Enter`) envia, e onde a IA está ativa `Alt+E` abre **Escrever com IA**
([o capítulo da IA](ai.md#ajuda-de-escrita-num-host)). A prévia mostra o texto como o host o fará, referências
incluídas (`#7`, `!12`, `ABC-12`, ids de commit); um texto não enviado é guardado por pedido.

### Ficheiros

Os ficheiros alterados à esquerda (letra de estado, `+adicionadas -removidas`, uma contagem `●n` de discussões abertas, uma contagem `✎n` dos
seus comentários pendentes) e o diff do ficheiro selecionado à direita. O diff vem do git: o head do pedido
é obtido para `refs/gitpoulpi/pr/<n>` (nunca uma branch, nada fica com checkout) e comparado com a
base de integração, pelo que os números de linha na margem são exatamente os que os comentários usam.

- **`[ ]` antes de um ficheiro**: marque-o como *visto* (clique, ou prima **`v`** / `Space` na lista). O título conta-os.
  As marcas de visto são guardadas por repositório e sobrevivem a um reinício.
- As **discussões** ficam logo sob a sua linha: o primeiro comentário, as respostas e os botões **Responder** e
  **Resolver** / **Reabrir**. Uma discussão resolvida recolhe-se numa só linha; **`z`** (ou `Enter` nela, ou um clique no
  `▾`) recolhe e expande qualquer discussão.
- As **discussões desatualizadas**, e as de linhas que o diff não mostra, ficam listadas depois dos hunks, em
  *Comentários fora das linhas mostradas*, com um selo `desatualizada` quando o código mudou por baixo delas.
- **`c`** comenta a linha onde está o cursor (o pequeno `+` no início da linha faz o mesmo com o
  rato). **`Shift+Up` / `Shift+Down`** (ou um arrastar sobre as linhas) selecionam primeiro várias linhas e o
  comentário abrange o intervalo. O GitLab aceita uma linha por comentário, e a vista di-lo em vez de selecionar.
- **`r`** responde à discussão sob o cursor, **`x`** resolve-a ou reabre-a. Ambos são enviados ao host **de
  imediato**.
- Teclas: `Up` / `Down` / `PgUp` / `PgDn` / `Home` / `End` movem; `Tab`, `Right` e `Enter` vão da lista de ficheiros
  para o diff, `Left` e `Tab` voltam; `]` e `[` saltam para o ficheiro seguinte e o anterior.

### As opções de apresentação do diff

O painel do diff de **Ficheiros** e de **Commits** é o da própria [vista de Diff](staging-and-committing.md), com a mesma barra na primeira
linha e as mesmas escolhas lembradas (são uma só definição: o que escolher aqui é com o que a vista de Diff abre,
e vice-versa):

- **Inline / Dividido**: uma coluna, ou antes e depois lado a lado. No modo Dividido uma discussão, um comentário pendente ou uma
  sugestão fica sob a metade a que pertence: um comentário numa linha removida sob a metade esquerda, um comentário numa linha
  adicionada ou inalterada sob a metade direita. Clique na metade que pretende antes de premir **`c`** numa linha que
  mostra uma remoção e uma adição; o teclado comenta o lado novo.
- **Hunks**: só as alterações com o seu contexto, ou o ficheiro todo com as alterações no sítio. No ficheiro todo uma
  discussão numa linha fora das alterações fica sob a sua linha em vez de em *Comentários fora das linhas mostradas*.
  O texto do ficheiro é lido em segundo plano; os hunks ficam no ecrã com uma linha a dizê-lo até chegar (na
  página Commits, até 40 ficheiros de um commit são lidos assim).
- **WS**: mostrar ou ignorar alterações que só diferem em espaços em branco.
- **Quebra**: quebrar as linhas longas.
- **Setas para cima / para baixo**: o hunk anterior e o seguinte.
- **Mapa** (**`M`**): o minimapa junto à barra de deslocação, uma só definição com a da vista de Diff. Clique ou arraste nele para
  deslocar.
- **Auto** (**`E`**, ou *Revisão: escolher codificação...*): a codificação com que o ficheiro é lido, como na vista de Diff:
  Deteção automática ou uma das codificações da lista, escolhida para o ficheiro que o painel mostra (na página Commits, o
  ficheiro sob o cursor). Os dois lados do ficheiro são lidos de novo nessa codificação, em segundo plano, e
  a escolha é partilhada com a vista de Diff para o mesmo caminho. Um ficheiro que não pode ser lido assim diz porquê acima
  do seu diff.
- **`⋯`**: quando o painel é demasiado estreito para todos os botões, os que não cabem ficam atrás de um `⋯` no fim da
  barra; a sua lista executa cada um como se fosse clicado. As suas teclas e linhas da paleta funcionam como sempre.

A régua da barra de deslocação mostra onde estão as coisas: as alterações (só no ficheiro todo, como na vista de Diff), um ponto `●`
por cada discussão, comentário pendente e sugestão, e o cursor.

As linhas longas são cortadas na margem do painel (ative **Quebra** para as ver inteiras) e deslocam-se para o lado: **`Right`**
e **`Left`** no diff (`Left` na margem esquerda volta à lista de ficheiros), a roda lateral ou `Shift`
com a roda, a barra de deslocação ao longo da margem inferior, ou *Revisão: deslocar o diff para a esquerda / direita*.

### Selecionar texto e linhas

O sítio onde um arrastar começa decide o que seleciona. Um arrastar que começa **no código** seleciona texto, como na vista de Diff
(o clique duplo seleciona uma palavra); **`Ctrl+C`** (ou *Revisão: copiar o texto selecionado ou a linha*) copia exatamente esses
carateres, sem números de linha nem marcadores, e sem seleção de texto copia a linha sob o cursor. Um
arrastar que começa **na goteira** (os números de linha, o marcador, o `+`) seleciona linhas, para as comentar
como antes, e um clique duplo na goteira continua a iniciar um comentário nessa linha.

As teclas `n`, `p`, `m`, `w`, `s` da vista de Diff são as da própria revisão, pelo que na revisão os alternadores são letras
maiúsculas: **`V`** layout, **`H`** hunks ou ficheiro todo, **`S`** espaços em branco, **`W`** quebra, **`J`** / **`K`** hunk seguinte e
anterior, **`E`** a codificação, **`M`** o minimapa. Cada um tem uma linha na paleta (*Revisão: alternar layout inline / dividido*,
...).

### Commits

Só os commits do pedido (não o histórico por trás dele), dos mais recentes para os mais antigos, e o diff do selecionado
(as suas alterações face ao pai), com todos os seus ficheiros uns sob os outros e as mesmas opções de apresentação.

## Escrever uma revisão

Um comentário no código fica **pendente**: é guardado na sua máquina (no ficheiro de estado do repositório, pelo que sobrevive a um
reinício) e contado no cabeçalho. Nada é enviado até concluir a revisão. Um comentário pendente aparece
sob a sua linha com o selo `Pendente - linha 12` e dois botões: **`e`** (Editar) e **`d`** (Eliminar).

**Concluir revisão** (**`f`**, o botão ou a paleta) abre um pequeno formulário: um resumo (opcional) e uma lista de
**Parecer**: *Comentar*, *Aprovar*, *Pedir alterações*. Só é listado o que o host permite: o GitLab não tem
*Pedir alterações*. **Submeter revisão** (`Ctrl+S`) envia o resumo, o parecer e todos os comentários pendentes de
uma só vez. **Descartar revisão...** deita fora os comentários pendentes depois de uma confirmação.

O que é enviado de imediato, sem esperar pela revisão: as respostas, resolver e reabrir, e os comentários na
conversa. A caixa de comentário diz de que tipo é (*Este comentário é publicado de imediato* ou *Este comentário é adicionado à sua
revisão pendente*). Se o host recusar uma resposta, a caixa volta a abrir-se com as suas palavras.

Se o pedido recebeu novos commits depois de ter escrito comentários pendentes, a vista avisa que as respetivas linhas podem ter
mudado (o GitHub e o GitLab colocam um comentário no commit que leu; uma revisão de um head mais antigo é recusada).

## Gerrit

Um servidor Gerrit liga-se como os outros hosts (ver [Integrações de alojamento e de gestores de issues](hosting-integration.md): URL do
servidor, nome de utilizador, palavra-passe HTTP) e associa-se, em **PULL REQUESTS**, a um *projeto*. As suas alterações são listadas como pull
requests (`#7`, chip `GR`) e revistas na mesma vista, com as diferenças que o Gerrit traz:

- **Patch sets.** Uma alteração tem patch sets numerados (cada envio do commit é um). O cabeçalho tem uma
  caixa de seleção **`Patch set N`** na margem superior, junto aos separadores de página (**`P`**, ou a linha da paleta *Revisão: escolher
  patch set...*). Começa no mais recente; escolher outro lê os ficheiros e o código desse patch set, e as
  discussões escritas noutros patch sets ficam listadas como *desatualizadas*. A revisão que concluir é enviada no patch set
  que está a ser mostrado.
- **Comparar com outro patch set.** Uma segunda caixa de seleção à esquerda dessa, **`Comparar: base`** (**`D`**, ou *Revisão: comparar
  patch sets...*), lista a base e cada patch set mais antigo do que o mostrado. Escolha o patch set 1 enquanto o patch set 3
  está a ser mostrado e a página Ficheiros lista, e mostra o diff de, apenas o que o patch set 3 alterou desde o patch set 1
  (a diferença do próprio servidor, por isso um rebase sobre um destino mais recente não aparece como alterações); a página
  Commits mostra essa diferença. Os dois patch sets são obtidos para `refs/gitpoulpi/pr/<n>` e `refs/gitpoulpi/pr/<n>-ps<k>`. As discussões
  escritas noutros patch sets que não o mostrado ficam *desatualizadas*. Escolher para ler um patch set mais antigo do que o
  comparado repõe a comparação na base. O equivalente na CLI é `review show|fetch --compare-with K`.
- **Submeter e abandonar.** Quando o servidor diz que a alteração pode ser submetida (os votos e as verificações de que precisa
  existem), um botão **Submeter** junta-se aos botões do cabeçalho; **Abandonar** está lá enquanto puder abandonar a alteração.
  Ambos (teclas **`U`** e **`Q`**, paleta *Revisão: submeter alteração...* / *Revisão: abandonar alteração...*) perguntam primeiro, num
  popup que diz que a ação acontece no servidor e **não pode ser desfeita a partir do GitPoulpi** (uma alteração integrada
  fica integrada; uma abandonada pode ser restaurada na interface web do Gerrit). Recusar não envia nada; em caso de sucesso
  a vista volta a ler a alteração. Os equivalentes na CLI são `gitpoulpi hosting review submit-change <n>` e
  `hosting review abandon <n> [--message M]`.
- **Votos em vez de um parecer.** **Concluir revisão** mostra uma caixa de seleção por etiqueta em que pode votar - `Code-Review`
  (-2 a +2) e `Verified` onde o projeto o tiver - listando apenas os valores que o servidor permite **para esta
  alteração e para si** (quem não tem o direito não pode escolher +2). Começa no seu voto atual; só um voto que
  tenha mudado é enviado. O resumo e os comentários pendentes vão na mesma chamada. Os rascunhos que guardou na interface web do
  Gerrit são publicados com a revisão.
- As **discussões** são as cadeias de comentários do Gerrit. Uma discussão está resolvida quando o seu comentário mais recente diz `unresolved: false`;
  **resolver** publica uma resposta `Done` que o faz, **reabrir** uma resposta `Reopened`. As respostas e o resolver vão para o servidor
  de imediato, como nos outros hosts.
- **Push para revisão.** O Gerrit cria uma alteração quando se faz push para `refs/for/<branch>`. Com a secção PULL REQUESTS
  associada a um projeto Gerrit, o botão **`+`** da secção e a linha da paleta **Alojamento: push para revisão...**
  pedem a branch de destino e enviam `HEAD` para `refs/for/<branch>` (um tópico pode ser indicado na linha de
  comandos). O push não pode ser desfeito a partir do GitPoulpi (abandone a alteração no Gerrit); a mensagem di-lo. O equivalente
  na CLI é `gitpoulpi hosting review push`.
- Os botões de metadados são só de leitura no Gerrit (os revisores são mostrados), não há parecer *Pedir alterações*
  (um voto de -1 ou -2 é a mesma coisa) e o chip de verificações é a etiqueta `Verified`.

## ✦ Rascunho de revisão da IA

![Duas sugestões ✦ sob as suas linhas na página Ficheiros, cada uma com Aceitar, Editar e Dispensar, junto a uma discussão existente](../images/review-ai.png)

Com a IA ligada (*Definições > IA*, ver [IA](ai.md)) o cabeçalho tem um botão **`✦ Redigir revisão...`** (também **`s`**,
e a linha da paleta *Revisão: ✦ redigir revisão...*). Abre as [opções da revisão](#opções-da-revisão) e
**`Enter`** aí consulta o modelo. É a única coisa que consulta o modelo: abrir uma revisão
nunca o faz. O modelo lê o título e a descrição do pedido, os comentários que já existem (para
não os repetir) e o diff, e responde com um **rascunho de revisão**: o texto a submeter, o parecer que recomenda (Aprovar, Pedir alterações ou Comentar), um resumo geral e uma lista de comentários sugeridos. Um
diff longo é cortado para o modelo - por ficheiro e no total - e o Resumo diz quantos ficheiros foram encurtados
ou deixados de fora. Enquanto trabalha, a barra de estado mostra um indicador de atividade passado um momento e **`Esc`** (ou o mesmo botão)
cancela. As instruções de *Revisão de código* dos [quatro níveis](ai.md#instruções) aplicam-se, pelo que uma equipa ou uma
área de trabalho pode dizer o que é uma boa revisão.

### Opções da revisão

`s` abre o diálogo **✦ Opções da revisão**, preenchido com as predefinições do repositório (as globais quando o
repositório não tem nenhuma). `Enter` (ou o botão **Executar**) inicia a execução com o que o diálogo mostra; `Esc`, **Cancelar**
ou a cruz fecha-o e não pergunta nada.

| Controlo | O que faz |
|---|---|
| **Executar o agente no repositório** | *Definição do perfil* (a pasta de trabalho do perfil de IA decide), *Sim* ou *Não*. Esmaecido, com o motivo por baixo, para um perfil HTTP (não tem pasta de trabalho) e quando o pedido é revisto através da API sem um clone. |
| **CLAUDE.md**, **AGENTS.md** | Dão ao modelo esse ficheiro do repositório, lido na branch a que o pedido se destina. Quando o agente é executado no repositório lê-os por si, e o diálogo di-lo. |
| **Ficheiro extra** | Mais um ficheiro: um caminho no repositório, um caminho absoluto ou uma ligação para um ficheiro de um code host. **Procurar...** (`Ctrl+B`, ou `Enter` no botão) abre um seletor de ficheiros que começa na raiz do repositório (ou ao lado do ficheiro já indicado); um ficheiro dentro do repositório é escrito como caminho relativo à raiz, um de fora como caminho absoluto. Escrever no campo funciona como antes. |
| **Nenhum ficheiro extra** | Nenhum ficheiro extra nesta execução, mesmo que a definição global ou o repositório indique um; escrever um ficheiro ou escolher um com **Procurar...** desmarca-o. Com **Guardar como predefinições deste repositório** passa a ser o interruptor *Nenhum ficheiro extra* do repositório. Um campo vazio conta aqui como nenhum, porque o diálogo parte do que a execução herdaria. |
| **Guardar como predefinições deste repositório** | Guarda também os valores para as próximas revisões: no ficheiro de estado do repositório, ou em `remote-reviews.toml` para um repositório não clonado. Só se guarda um valor que difira do global, pelo que o resto continua a seguir *Definições > IA*. |

Tudo se alcança com `Tab`, `Space` marca uma caixa e o campo do caminho é um campo de texto normal. As
mesmas quatro definições estão em *Definições > IA* (*Revisão de código*) e em *Definições do repositório > IA*; ver
[Contexto da revisão](ai.md#contexto-da-revisão). O que não pôde ser dado ao modelo (um ficheiro extra em falta ou
ilegível) e qualquer ficheiro que foi encurtado para caber nos limites ficam listados no cartão do rascunho.

As sugestões **não são comentários**. Aparecem em dois sítios:

- **Sob a sua linha em Ficheiros**, como comentários fantasma marcados com `✦` e com uma gravidade (`info`, `aviso`, `problema`) e
  três botões: **Aceitar**, **Editar** e **Dispensar**. Uma sugestão sobre uma linha que o diff não mostra é
  guardada, nunca descartada: fica listada depois dos hunks em *Sugestões da IA sobre o ficheiro todo* e, depois de
  aceite, vai parar à primeira linha alterada do ficheiro, com as palavras *(sobre a linha N)* à frente.
- **No Resumo**, num bloco acima da conversa (só depois de uma execução): o resumo e depois a lista com os mesmos
  botões e **Aceitar tudo** / **Dispensar tudo**.

**Aceitar** (`y`) transforma uma sugestão num **comentário pendente** normal (ainda o pode editar ou eliminar);
**Editar** (`w`) abre a caixa de comentário com as palavras do modelo primeiro e aceita o que deixar; **Dispensar** (`n`)
descarta-a. **Aceitar tudo** (`A`) apenas preenche a lista de pendentes. Na página Ficheiros as teclas atuam na
sugestão sob o cursor, no Resumo na primeira da lista (`y` e `n` voltam a ser *Copiar ligação* e *Próximo novo* da conversa quando a lista fica
vazia). **Usar resumo da IA** (`i`, ou o botão
do formulário Concluir revisão) põe o resumo do modelo no texto da revisão; nunca é inserido por si só.
**Nada é enviado ao host até submeter a revisão.**

O Resumo mostra o rascunho como um cartão no topo desse bloco, acima das sugestões: a etiqueta do parecer, o texto da revisão e três
botões.

- **✦ Usar rascunho...** (`u`) aceita cada sugestão aberta como comentário pendente e abre *Concluir revisão* já
  preenchido com o texto e o parecer do rascunho. Continua a ser você a lê-lo, alterá-lo e submetê-lo; cancelar o
  formulário mantém o rascunho. Os limites do host mantêm-se: o GitLab não tem *Pedir alterações*, por isso o formulário di-lo e
  recua para *Comentar*; no Gerrit o parecer passa a um voto Code-Review de +1 ou -1, apenas quando o servidor
  permite esse valor para si (caso contrário não se define nenhum voto e o formulário di-lo).
- **Editar texto...** (`t`) altera o texto de revisão do rascunho antes de o usar.
- **Descartar rascunho** (`N`) deita fora o texto, o parecer e as sugestões que restam.

O rascunho é guardado por pedido, por isso continua lá quando fecha e volta a abrir a revisão. Quando o
head do pedido mudou desde que o modelo o escreveu, o cartão diz *O pedido mudou desde que este rascunho foi escrito.*
Submeter a revisão gasta o rascunho.

## Tudo tem uma tecla e uma linha na paleta

| Ação | Tecla | Paleta |
|---|---|---|
| Resumo / Ficheiros / Commits | `Alt+1` / `Alt+2` / `Alt+3` | *Revisão: mostrar resumo*, *mostrar ficheiros*, *mostrar commits* |
| Comentar a linha ou o intervalo | `c` | *Revisão: comentar a linha...* |
| Comentar a conversa | `c` no Resumo, `C` | *Revisão: comentar a conversa...* (abre a caixa do Resumo), *Conversa: novo comentário* |
| Responder à discussão | `r` | *Revisão: responder à discussão...* |
| Resolver ou reabrir | `x` | *Revisão: resolver ou reabrir a discussão* |
| Recolher ou expandir uma discussão | `z` | *Revisão: recolher ou expandir a discussão* |
| Editar / eliminar um comentário pendente | `e` / `d` | *Revisão: editar comentário pendente...*, *eliminar comentário pendente* |
| Marcar o ficheiro como visto | `v` | *Revisão: marcar ficheiro como visto* |
| Inline / Dividido | `V` | *Revisão: alternar layout inline / dividido* |
| Ficheiro todo / só os hunks | `H` | *Revisão: alternar ficheiro todo / só os hunks* |
| Alterações só de espaços em branco | `S` | *Revisão: alternar alterações só de espaços em branco* |
| Quebra de linha | `W` | *Revisão: alternar quebra de linha* |
| Hunk seguinte / anterior | `J` / `K` | *Revisão: próximo hunk*, *hunk anterior* |
| Ficheiro seguinte / anterior | `]` / `[` | *Revisão: próximo ficheiro*, *ficheiro anterior* |
| Concluir / descartar a revisão | `f` | *Revisão: concluir revisão...*, *descartar revisão pendente...* |
| Atualizar | `F5`, `Ctrl+R` | *Revisão: atualizar* |
| Abrir no navegador / fazer checkout | `o` / `b` | *Revisão: abrir no navegador*, *fazer checkout da branch* |
| Responsáveis / revisores / etiquetas / marco | `a` / `p` / `l` / `m` | *Revisão: editar responsáveis...* e as suas três irmãs |
| Escolher o patch set (Gerrit) | `P` | *Revisão: escolher patch set...* |
| Comparar com um patch set mais antigo (Gerrit) | `D` | *Revisão: comparar patch sets...* |
| Submeter / abandonar a alteração (Gerrit) | `U` / `Q` | *Revisão: submeter alteração...*, *Revisão: abandonar alteração...* |
| ✦ Redigir revisão... (IA ligada; abre as opções da revisão) | `s` | *Revisão: ✦ redigir revisão...* |
| Aceitar / editar e aceitar / dispensar uma sugestão | `y` / `w` / `n` | *Revisão: aceitar sugestão*, *editar e aceitar sugestão...*, *dispensar sugestão* |
| Aceitar / dispensar todas as sugestões | `A` / `N` | *Revisão: aceitar todas as sugestões*, *dispensar todas as sugestões* |
| Usar o resumo da IA no texto da revisão | `i` | *Revisão: usar o resumo da IA no texto da revisão* |
| Usar o rascunho ✦ (preenche Concluir revisão) | `u` | *Revisão: ✦ usar o rascunho...* |
| Editar o texto do rascunho | `t` | *Revisão: editar o texto do rascunho ✦...* |
| Descartar o rascunho ✦ | `N` | *Revisão: descartar o rascunho ✦* |
| Fechar | `Esc` | *Revisão: fechar* |

Abra a paleta (`Ctrl+P`) com a vista aberta: as suas linhas atuam sobre ela e dizem porque é que uma não faz nada
neste momento («ponha primeiro o cursor numa linha de código»).

## Rever sem um clone

Um pedido de um repositório que não está na sua máquina também pode ser revisto. A partir do Radar, **Rever**
(`e`) na sua linha pergunta como (ver [O Radar](radar.md#rever-um-pedido)); **Rever através da API** abre
esta mesma vista na janela Home, por cima do Radar. Tudo nesta página funciona da mesma maneira, com estas
diferenças:

- O código (os ficheiros alterados, o diff de cada ficheiro, os commits do pedido, o diff de um commit, o ficheiro inteiro) é
  lido da API do host, por isso o primeiro olhar a um pedido grande demora um momento, e um ficheiro para o qual o host não
  envia diff (binário ou muito grande) di-lo em vez de mostrar um.
- Os seus comentários pendentes ficam em `remote-reviews.toml` na pasta de configuração, arquivados sob o host
  e o projeto (`host/project`), em vez de no estado próprio de um repositório. Continuam lá quando
  fecha e volta a abrir a revisão, e enviar a revisão gasta-os.
- Não há escolha de codificação (*Indisponível sem um clone*) nem **Fazer checkout** (diz para clonar primeiro o
  repositório).
- `Esc` ou o `✕` no canto sai da vista e devolve o Radar. A paleta (`Ctrl+P`) chega às linhas
  da vista como no separador de um repositório.

Para ficar com o clone, escolha **Clonar e depois rever** (ou *Radar: clonar e rever...* da paleta):
o diálogo Clonar abre na conta ligada do pedido com o seu repositório selecionado, e a revisão abre no
separador do novo repositório depois de o clone terminar.

## Na linha de comandos e para agentes

A mesma revisão está disponível como `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(e as ferramentas MCP feitas a partir deles): ver a [referência da CLI](cli-reference.md). `threads` também lista os
pareceres já submetidos (`reviews`). `suggest <n>` consulta o modelo (IA ligada) e imprime o seu `verdict`, `body`, resumo e as suas
sugestões em JSON sem publicar nada, para que um script ou um agente possa escolher os comentários a passar a
`submit --comment`.

Nada disto precisa de um clone: `--project owner/name` (com `--id` quando há várias integrações configuradas)
executa `show`, `threads`, `comment`, `reply`, `resolve`, `submit` e `suggest` contra um projeto de que a
pasta atual não sabe nada. `hosting review diff <n>` imprime o diff com os seus hunks (`--api` lê-o da API
do host mesmo dentro de um clone), e `hosting file <path> --project P [--ref R]` lê um ficheiro numa revisão, por
exemplo o `CLAUDE.md` do projeto. Ambos leem a API do host: GitHub, GitLab, Gitea, Forgejo e Gerrit.
