[English](../review.md) · [Čeština](../cs/review.md) · [Deutsch](../de/review.md) · [Español](../es/review.md) · [Français](../fr/review.md) · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · [Polski](../pl/review.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Revisando um pull ou merge request

Revise um pull request (GitHub, Gitea, Forgejo), um merge request (GitLab) ou uma change do Gerrit (veja [Gerrit](#gerrit))
**sem sair da janela do repositório**. A visualização de revisão mostra a descrição, a conversa, os arquivos
alterados com as threads de comentários sob as suas linhas e os commits, e é nela que você escreve a revisão:
comentários, aprovação e pedidos de alterações. Ela precisa de um perfil de um desses hosts (veja
[Integrações de hospedagem e de rastreadores de issues](hosting-integration.md)) e de um repositório vinculado a ele; os outros provedores não
oferecem revisões. Os limites de cada host são respeitados: o GitLab não tem o veredito *Solicitar alterações* e
aceita uma linha por comentário. Esta página chama todos eles de *solicitação*.

![A visualização de revisão, página Visão geral: o cabeçalho da solicitação com os botões de metadados, a descrição, uma thread de código não resolvida e uma resolvida, os comentários e a caixa de comentário](../images/review-overview.png)

![A página Arquivos: os arquivos alterados com as caixas de visualizado, o diff com uma thread de comentários sob a sua linha e uma thread resolvida fora das linhas mostradas](../images/review-files.png)

## Abrindo uma revisão

Todos os caminhos levam à mesma visualização:

- a lista **PULL REQUESTS** da barra lateral: selecione uma linha e pressione **Enter**, ou clique nela com o botão direito e
  escolha **Revisar...**;
- o **Radar**: o menu da linha, **Revisar...**, ou a tecla `e` (o repositório abre e a revisão com ele;
  uma solicitação de um repositório que você não clonou pergunta se deve clonar ou revisar pela API,
  veja [Revisando sem um clone](#revisando-sem-um-clone));
- a **busca no espaço de trabalho** (`Ctrl+Alt+F`): realce um pull ou merge request e pressione **`Alt+R`**;
- a paleta: **Hospedagem: revisar solicitação...** lista as solicitações para você escolher.

A visualização preenche a janela sobre o repositório (é um modo da aba do repositório, não uma janela separada).
**`Esc`** ou o `✕` no canto dela a fecha. Enquanto ela abre, a solicitação é lida do host e o seu código é
obtido com o git em segundo plano; um indicador de progresso aparece se isso demorar mais que um instante.

## O cabeçalho

Sempre visível: o número e o título, o estado (Aberto, Rascunho, Mesclado, Fechado), as branches
(`main ← feature/moons`), o autor, as verificações (`✓ verificações aprovadas`, `✗ verificações com falha`,
`● verificações em andamento`), se ela pode ser mesclada sem conflitos ou se há conflitos, o tamanho, e
**`3 comentários pendentes`** quando você escreveu alguns.

Os quatro botões de metadados - **Responsáveis**, **Revisores**, **Rótulos**, **Marco** - abrem os mesmos seletores que
a barra lateral (veja [Responsáveis, revisores, rótulos e marco](pull-requests-and-issues.md#responsáveis-revisores-rótulos-e-marco));
o cabeçalho mostra o resultado assim que o host o aceita. Os botões de ação: **Fazer checkout** (o checkout
habitual da branch da solicitação, que também fecha a visualização), **Abrir no navegador**, **Atualizar**
(**`F5`**) e **Concluir revisão**.

## Três páginas

Um controle segmentado na borda superior troca de página: **Visão geral**, **Arquivos**, **Commits**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`**, ou um clique).

### Visão geral

A Visão geral é **a conversa da solicitação**, a mesma linha do tempo e a mesma caixa de comentário que uma issue tem
([Conversas](conversations.md)): primeiro a descrição (*abriu isto*), depois os comentários, as
discussões no código com o seu trecho do diff, as revisões enviadas com o seu veredito (`aprovada`, `alterações
solicitadas`, `comentada`, `dispensada`; no GitLab os vereditos são as suas aprovações) e os eventos, da mais antiga
para a mais nova, na janela mais recente com **Carregar anteriores** no topo. A caixa de comentário fica acoplada sob a
linha do tempo (**Novo comentário**): **Escrever**, **Prévia** e **Lado a lado**, **Cancelar** e **Enviar**. **Um
comentário publicado aqui vai ao host na hora** (nada fica pendente: os pendentes são os do código, enviados com a
revisão), aparece na lista imediatamente e ocupa o seu lugar quando o host responde; se o host o recusa, a caixa abre
de novo com o seu texto e o motivo.

As teclas são as da visão de conversa: **`Up`** / **`Down`** (e `Ctrl+J` / `Ctrl+K`) movem entre as entradas, **`r`**
responde na discussão da entrada selecionada, **`Q`** a cita em um novo comentário, **`c`** escreve um novo comentário,
**`e`** / **`d`** editam e excluem os seus próprios comentários (uma pergunta vem antes de excluir), **`l`** ou `Enter`
abre os links de uma entrada, **`g`** o seu diagrama, **`y`** copia o seu endereço, **`o`** a abre no navegador, **`n`**
vai para a próxima entrada não lida, **`v`** mostra ou oculta os eventos, **`m`** carrega entradas mais antigas, `Tab`
alterna entre a linha do tempo e a caixa, e **Nota interna** (`Alt+I`) existe nos hosts que têm essas notas. Os
botões da entrada selecionada (**Responder**, **Citar**, **Editar**, **Excluir**, **Links**) fazem o mesmo, e a paleta
tem as linhas *Conversa: ...* ao lado das *Revisão: ...*. Um link é perguntado antes de abrir, como em toda a revisão.

Uma thread de código mostra o seu `path:line` como um link (sublinhado, um botão de verdade). Um clique nele, **`G`** na
thread selecionada ou *Conversa: abrir o código da thread* na paleta passam para a página **Arquivos**, abrem esse arquivo
e põem o cursor na linha comentada (no lado antigo ou no novo, como foi escrito) com a thread aberta embaixo dela (uma
recolhida é expandida). **Esc**, o botão **Voltar à visão geral** à esquerda das abas da página ou *Revisão: voltar ao
comentário na visão geral* voltam à mesma entrada da Visão geral. Uma thread cuja linha não está mais no diff
(desatualizada) abre o arquivo na linha mais próxima e a linha de status avisa; uma thread em um arquivo que o pedido
não altera mais diz que o arquivo não está no diff. Em uma visão de issue ou de Radar do mesmo pedido não há código a
abrir: `G` responde que a revisão o tem.

Onde uma tecla da conversa e uma da revisão se encontram, a conversa fica com a letra nesta página: `l` (rótulos), `m`
(marco), `Q` (abandonar, do Gerrit), `e` / `d`, `r`. Os rótulos, o marco e o abandono continuam nos botões do cabeçalho
e na paleta; `a` (responsáveis), `p` (revisores), `f` (**Concluir revisão**), `b` (**Fazer checkout**), `P`, `D`, `U` e
`Alt+1` / `Alt+2` / `Alt+3` mantêm o seu significado aqui. Enquanto a caixa tem o teclado, toda tecla é da caixa,
exceto `Alt+1` / `Alt+2` / `Alt+3`; `Esc` devolve o teclado à linha do tempo e um segundo `Esc` fecha a revisão.

Um comentário escrito em outra janela ou pela CLI aparece aqui sozinho: toda escrita do GitPoulpi (esta janela
inclusive) avisa a sessão, e as janelas que mostram a mesma solicitação a leem de novo.

A caixa de comentário é o editor Markdown compartilhado com [a visão de conversa](conversations.md#escrever)
(também a caixa de uma resposta a uma discussão, de um comentário em uma linha e do resumo de **Concluir revisão**):
`Ctrl+Enter` (também `Ctrl+S`, `Alt+Enter`) envia e, onde a IA está ligada, `Alt+E` abre **Escrever com IA**
([o capítulo de IA](ai.md#ajuda-de-escrita-em-um-host)). A prévia mostra o texto como o host o mostrará, referências
incluídas (`#7`, `!12`, `ABC-12`, ids de commit); um texto não enviado é guardado por solicitação.

### Arquivos

Os arquivos alterados à esquerda (letra de status, `+adicionadas -removidas`, uma contagem `●n` de threads abertas, uma
contagem `✎n` dos seus comentários pendentes) e o diff do arquivo selecionado à direita. O diff vem do git: o head da
solicitação é obtido em `refs/gitpoulpi/pr/<n>` (nunca uma branch, nada vai para checkout) e comparado com a
base do merge, de modo que os números de linha da margem são exatamente os que os comentários usam.

- **`[ ]` antes de um arquivo**: marque-o como *visualizado* (clique, ou pressione **`v`** / `Espaço` na lista). O título os conta.
  As marcas de visualizado são guardadas por repositório e sobrevivem a uma reinicialização.
- As **threads** ficam logo abaixo da sua linha: primeiro comentário, respostas e os botões **Responder** e
  **Resolver** / **Reabrir**. Uma thread resolvida se recolhe em uma linha; **`z`** (ou `Enter` nela, ou um clique no
  `▾`) recolhe e expande qualquer thread.
- As **threads desatualizadas**, e as threads em linhas que o diff não mostra, são listadas depois dos hunks sob
  *Comentários fora das linhas mostradas*, com a etiqueta `desatualizada` quando o código mudou sob elas.
- **`c`** comenta a linha em que o cursor está (o pequeno `+` no início da linha faz o mesmo com o mouse).
  **`Shift+Up` / `Shift+Down`** (ou um arrasto sobre as linhas) selecionam várias linhas antes e o
  comentário cobre o intervalo. O GitLab aceita uma linha por comentário, e a visualização avisa isso em vez de selecionar.
- **`r`** responde à thread sob o cursor, **`x`** a resolve ou a reabre. Ambos são enviados ao host **na
  hora**.
- Teclas: `Cima` / `Baixo` / `PgUp` / `PgDn` / `Home` / `End` movem; `Tab`, `Direita` e `Enter` vão da lista de arquivos
  para o diff, `Esquerda` e `Tab` voltam; `]` e `[` saltam para o próximo arquivo e o anterior.

### As opções de exibição do diff

O painel de diff de **Arquivos** e de **Commits** é o da própria [visualização de Diff](staging-and-committing.md), com a mesma barra na primeira
linha e as mesmas escolhas lembradas (é uma única configuração: o que você escolhe aqui é com o que a visualização de Diff abre,
e vice-versa):

- **Inline / Dividido**: uma coluna, ou antes e depois lado a lado. No Dividido, uma thread, um comentário pendente ou uma
  sugestão fica sob a metade a que pertence: um comentário em uma linha removida sob a metade esquerda, um comentário em uma
  linha adicionada ou inalterada sob a metade direita. Clique na metade que você quer antes de pressionar **`c`** em uma linha que
  mostra uma remoção e uma adição; o teclado comenta o lado novo.
- **Hunks**: só as alterações com o seu contexto, ou o arquivo todo com as alterações no lugar. No arquivo todo, uma
  thread em uma linha fora das alterações fica sob a sua linha em vez de *Comentários fora das linhas mostradas*.
  O texto do arquivo é lido em segundo plano; os hunks ficam na tela com uma linha avisando isso até ele chegar (na
  página Commits, até 40 arquivos de um commit são lidos assim).
- **EB**: mostrar ou ignorar alterações que só diferem em espaços em branco.
- **Quebra**: quebrar linhas longas.
- **Setas para cima / para baixo**: o hunk anterior e o próximo.
- **Mapa** (**`M`**): o minimapa ao lado da barra de rolagem, uma única configuração com a da visualização de Diff. Clique ou arraste nele para
  rolar.
- **Auto** (**`E`**, ou *Revisão: escolher a codificação...*): a codificação em que o arquivo é lido, como na visualização de Diff:
  Detecção automática ou uma das codificações da lista, escolhida para o arquivo que o painel mostra (na página Commits, o
  arquivo sob o cursor). Os dois lados do arquivo são lidos de novo nessa codificação, em segundo plano, e
  a escolha é compartilhada com a visualização de Diff para o mesmo caminho. Um arquivo que não pode ser lido assim diz o motivo acima
  do seu diff.
- **`⋯`**: quando o painel é estreito demais para todos os botões, os que não cabem ficam atrás de um `⋯` no fim da
  barra; a lista dele executa cada um como se fosse clicado. As suas teclas e linhas da paleta funcionam como sempre.

A régua da barra de rolagem mostra onde as coisas estão: as alterações (só no arquivo todo, como na visualização de Diff), um ponto `●`
para cada thread, comentário pendente e sugestão, e o cursor.

As linhas longas são cortadas na borda do painel (ative **Quebra** para vê-las inteiras) e rolam para o lado: **`Direita`**
e **`Esquerda`** no diff (`Esquerda` na borda esquerda volta à lista de arquivos), a roda lateral ou `Shift`
com a roda, a barra de rolagem ao longo da borda inferior, ou *Revisão: rolar o diff para a esquerda / direita*.

### Selecionando texto e linhas

O ponto onde um arrasto começa decide o que ele seleciona. Um arrasto que começa **no código** seleciona texto, como na visualização de Diff
(o clique duplo seleciona uma palavra); **`Ctrl+C`** (ou *Revisão: copiar o texto selecionado ou a linha*) copia exatamente esses
caracteres, sem números de linha nem marcadores, e sem uma seleção de texto copia a linha sob o cursor. Um
arrasto que começa **na calha** (os números de linha, o marcador, o `+`) seleciona linhas, para comentá-las
como antes, e um clique duplo na calha ainda inicia um comentário nessa linha.

As teclas `n`, `p`, `m`, `w`, `s` da visualização de Diff são as próprias da revisão, então na revisão os alternadores são
letras maiúsculas: **`V`** layout, **`H`** hunks ou arquivo todo, **`S`** espaços em branco, **`W`** quebra, **`J`** / **`K`** próximo e
anterior hunk, **`E`** a codificação, **`M`** o minimapa. Cada um tem uma linha na paleta (*Revisão: alternar layout inline / dividido*,
...).

### Commits

Só os commits da solicitação (não o histórico por trás dela), do mais novo para o mais antigo, e o diff do selecionado
(as suas alterações em relação ao pai), com todos os seus arquivos um sob o outro e as mesmas opções de exibição.

## Escrevendo uma revisão

Um comentário no código fica **pendente**: ele é guardado na sua máquina (no arquivo de estado do repositório, então sobrevive a uma
reinicialização) e contado no cabeçalho. Nada é enviado até você concluir a revisão. Um comentário pendente aparece
sob a sua linha com a etiqueta `Pendente - linha 12` e dois botões: **`e`** (Editar) e **`d`** (Excluir).

**Concluir revisão** (**`f`**, o botão, ou a paleta) abre um pequeno formulário: um resumo (opcional) e uma lista de
**Veredito**: *Comentar*, *Aprovar*, *Solicitar alterações*. Só é listado o que o host permite: o GitLab não tem
*Solicitar alterações*. **Enviar revisão** (`Ctrl+S`) envia o resumo, o veredito e todos os comentários pendentes de
uma vez. **Descartar revisão...** joga fora os comentários pendentes depois de uma confirmação.

O que é enviado na hora, sem esperar pela revisão: respostas, resolver e reabrir, e comentários na
conversa. A caixa de comentário diz de que tipo ele é (*Este comentário é publicado na hora* ou *Este comentário é adicionado à sua
revisão pendente até você concluí-la*). Se o host recusar uma resposta, a caixa se abre de novo com as suas palavras.

Se a solicitação recebeu novos commits depois que você escreveu comentários pendentes, a visualização avisa que as linhas deles podem ter
mudado de lugar (o GitHub e o GitLab colocam um comentário no commit que você leu; uma revisão de um head mais antigo é recusada).

## Gerrit

Um servidor Gerrit é conectado como os outros hosts (veja [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md): URL do
servidor, nome de usuário, senha HTTP) e vinculado em **PULL REQUESTS** a um *projeto*. As suas changes são listadas como pull
requests (`#7`, chip `GR`) e revisadas na mesma visualização, com as diferenças que o Gerrit traz:

- **Patch sets.** Uma change tem patch sets numerados (cada envio do commit é um). O cabeçalho tem uma
  lista suspensa **`Patch set N`** na borda superior, ao lado das abas das páginas (**`P`**, ou a linha da paleta *Revisão: escolher
  patch set...*). Ela começa no mais recente; escolher outro lê os arquivos e o código desse patch set, e as
  threads escritas em outros patch sets são listadas como *desatualizadas*. A revisão que você conclui é enviada no patch set
  que está sendo mostrado.
- **Comparar com outro patch set.** Uma segunda lista suspensa à esquerda dela, **`Comparar: base`** (**`D`**, ou *Revisão:
  comparar patch sets...*), lista a base e cada patch set mais antigo que o mostrado. Escolha o patch set 1 enquanto o
  patch set 3 é mostrado e a página Arquivos lista, e mostra o diff de, só o que o patch set 3 mudou desde o patch set 1
  (a diferença do próprio servidor, então um rebase sobre um destino mais novo não aparece como alterações); a página
  Commits mostra essa única diferença. Os dois patch sets são buscados em `refs/gitpoulpi/pr/<n>` e
  `refs/gitpoulpi/pr/<n>-ps<k>`. As threads escritas em outros patch sets que não o mostrado são *desatualizadas*.
  Escolher para ler um patch set mais antigo que o comparado volta a comparação para a base. O equivalente na CLI é
  `review show|fetch --compare-with K`.
- **Submeter e abandonar.** Quando o servidor diz que a change pode ser submetida (os votos e as verificações de que ela
  precisa estão lá), um botão **Submeter** se junta aos botões do cabeçalho; **Abandonar** está lá enquanto você puder
  abandonar a change. Os dois (teclas **`U`** e **`Q`**, paleta *Revisão: submeter a solicitação...* / *Revisão:
  abandonar a solicitação...*) perguntam antes, em um popup que diz que a ação acontece no servidor e **não pode ser
  desfeita pelo GitPoulpi** (uma change integrada continua integrada; uma abandonada pode ser restaurada na interface web
  do Gerrit). Recusar não envia nada; em caso de sucesso a visualização lê a change de novo. Os equivalentes na CLI são
  `gitpoulpi hosting review submit-change <n>` e `hosting review abandon <n> [--message M]`.
- **Votos em vez de um veredito.** **Concluir revisão** mostra uma lista suspensa por rótulo em que você pode votar - `Code-Review`
  (-2 a +2) e `Verified` onde o projeto o tem - listando só os valores que o servidor permite **para esta
  change e para você** (quem não tem o direito não pode escolher +2). Ela começa no seu voto atual; só um voto que você
  mudou é enviado. O resumo e os comentários pendentes vão na mesma chamada. Os rascunhos que você salvou na
  interface web do Gerrit são publicados com a revisão.
- As **threads** são as cadeias de comentários do Gerrit. Uma thread está resolvida quando o seu comentário mais recente diz `unresolved: false`;
  **resolver** publica uma resposta `Done` que faz isso, **reabrir** uma resposta `Reopened`. Respostas e resolver vão para o servidor
  na hora, como nos outros hosts.
- **Push para revisão.** O Gerrit cria uma change quando você faz push para `refs/for/<branch>`. Com a seção PULL REQUESTS
  vinculada a um projeto Gerrit, o botão **`+`** da seção e a linha da paleta **Hospedagem: push para
  revisão...** pedem a branch de destino e fazem push de `HEAD` para `refs/for/<branch>` (um tópico pode ser dado na linha
  de comando). O push não pode ser desfeito pelo GitPoulpi (abandone a change no Gerrit); a mensagem avisa isso. O
  equivalente na CLI é `gitpoulpi hosting review push`.
- Os botões de metadados são somente leitura para o Gerrit (os revisores são mostrados), não há veredito *Solicitar alterações*
  (um voto de -1 ou -2 é a mesma coisa) e o chip de verificações é o rótulo `Verified`.

## ✦ Rascunho de revisão da IA

![Duas sugestões ✦ sob as suas linhas na página Arquivos, cada uma com Aceitar, Editar e Dispensar, ao lado de uma thread existente](../images/review-ai.png)

Com a IA ativada (*Configurações > IA*, veja [IA](ai.md)) o cabeçalho tem um botão **`✦ Gerar rascunho...`** (também **`s`**,
e a linha da paleta *Revisão: ✦ gerar rascunho de revisão...*). Ele abre as [opções de revisão](#opções-de-revisão) e
**`Enter`** ali consulta o modelo. É a única coisa que consulta o modelo: abrir uma revisão
nunca o faz. O modelo lê o título e a descrição da solicitação, os comentários que já existem (para
não os repetir) e o diff, e responde com um **rascunho de revisão**: o texto a enviar, o veredito que recomenda (Aprovar, Solicitar alterações ou Comentar), um resumo geral e uma lista de comentários sugeridos. Um
diff longo é cortado para o modelo - por arquivo e no total - e a Visão geral diz quantos arquivos foram reduzidos
ou deixados de fora. Enquanto ele trabalha, a linha de status mostra um indicador de progresso depois de um instante e **`Esc`** (ou o mesmo botão)
o cancela. As instruções de *Revisão de código* dos [quatro níveis](ai.md#instruções) valem, então uma equipe ou um
espaço de trabalho pode dizer como é uma boa revisão.

### Opções de revisão

`s` abre o diálogo **✦ Opções de revisão**, preenchido com os padrões do repositório (os globais onde o
repositório não tem nenhum). `Enter` (ou o botão **Executar**) inicia a execução com o que o diálogo mostra; `Esc`, **Cancelar**
ou a cruz o fecha e não consulta nada.

| Controle | O que faz |
|---|---|
| **Executar o agente no repositório** | *Definição do perfil* (a pasta de trabalho do perfil de IA decide), *Sim* ou *Não*. Esmaecido, com o motivo logo abaixo, para um perfil HTTP (ele não tem pasta de trabalho) e quando a solicitação é revisada pela API, sem clone. |
| **CLAUDE.md**, **AGENTS.md** | Entregam ao modelo esse arquivo do repositório, lido na branch de destino da solicitação. Quando o agente roda no repositório, ele os lê sozinho, e o diálogo diz isso. |
| **Arquivo extra** | Mais um arquivo: um caminho no repositório, um caminho absoluto ou um link para um arquivo de um host de código. **Procurar...** (`Ctrl+B`, ou `Enter` no botão) abre um seletor de arquivos que começa na raiz do repositório (ou ao lado do arquivo já indicado); um arquivo dentro do repositório é escrito como caminho relativo à raiz, um fora dele como caminho absoluto. Digitar no campo funciona como antes. |
| **Nenhum arquivo extra** | Nenhum arquivo extra nesta execução, mesmo quando a configuração global ou o repositório indica um; digitar um arquivo ou escolher um com **Procurar...** desmarca a caixa. Com **Salvar como padrão deste repositório**, vira a chave *Nenhum arquivo extra* do repositório. Um campo vazio conta como nenhum aqui, porque o diálogo parte do que a execução herdaria. |
| **Salvar como padrão deste repositório** | Também guarda os valores para as próximas revisões: no arquivo de estado do repositório, ou em `remote-reviews.toml` para um repositório que não está clonado. Só um valor que difere do global é guardado, então o resto continua seguindo *Configurações > IA*. |

Tudo é acessível com `Tab`, `Espaço` alterna uma caixa de seleção e o campo de caminho é um campo de texto comum. As
mesmas quatro configurações ficam em *Configurações > IA* (*Revisão de código*) e em *Configurações do repositório > IA*; veja
[Contexto da revisão](ai.md#contexto-da-revisão). O que não pôde ser entregue ao modelo (um arquivo extra ausente ou
ilegível) e qualquer arquivo que foi reduzido para caber nos limites aparecem no cartão do rascunho.

As sugestões **não são comentários**. Elas aparecem em dois lugares:

- **Sob a sua linha em Arquivos**, como comentários fantasma marcados com `✦` com uma gravidade (`info`, `aviso`, `problema`) e
  três botões: **Aceitar**, **Editar** e **Dispensar**. Uma sugestão sobre uma linha que o diff não mostra é
  mantida, nunca descartada: ela é listada depois dos hunks sob *Sugestões da IA sobre o arquivo todo* e, depois de
  aceita, vai parar na primeira linha alterada do arquivo com as palavras *(perto da linha N)* na frente.
- **Na Visão geral**, em um bloco acima da conversa (só depois de uma execução): o resumo, depois a lista com os
  mesmos botões e **Aceitar todas** / **Dispensar todas**.

**Aceitar** (`y`) transforma uma sugestão em um **comentário pendente** comum (você ainda pode editá-lo ou excluí-lo);
**Editar** (`w`) abre a caixa de comentário com as palavras do modelo primeiro e aceita o que você deixar; **Dispensar** (`n`)
a descarta. **Aceitar todas** (`A`) só preenche a lista de pendentes. Na página Arquivos as teclas agem na
sugestão sob o cursor, na Visão geral na primeira da lista (`y` e `n` voltam a ser o copiar link e o próxima entrada não lida
da conversa quando a lista fica vazia). **Usar resumo da IA** (`i`, ou o botão
do formulário Concluir revisão) coloca o resumo do modelo no texto da revisão; ele nunca é inserido sozinho.
**Nada é enviado ao host até você enviar a revisão.**

A Visão geral mostra o rascunho como um cartão no topo desse bloco, acima das sugestões: o chip do veredito, o texto da revisão e três
botões.

- **✦ Usar rascunho...** (`u`) aceita cada sugestão aberta como um comentário pendente e abre *Concluir revisão* já
  preenchido com o texto e o veredito do rascunho. Você ainda o lê, o altera e o envia por conta própria; cancelar o
  formulário mantém o rascunho. Os limites do host são respeitados: o GitLab não tem *Solicitar alterações*, então o formulário diz isso e
  volta para *Comentar*; no Gerrit o veredito vira um voto de +1 ou -1 em Code-Review, só quando o servidor
  permite esse valor para você (senão nenhum voto é definido e o formulário diz isso).
- **Editar texto...** (`t`) altera o texto de revisão do rascunho antes de você usá-lo.
- **Descartar rascunho** (`N`) joga fora o texto, o veredito e as sugestões restantes.

O rascunho é mantido por solicitação, então continua ali quando você fecha e reabre a revisão. Quando o
head da solicitação mudou desde que o modelo o escreveu, o cartão diz *A solicitação mudou desde que este rascunho foi escrito.*
Enviar a revisão consome o rascunho.

## Tudo tem uma tecla e uma linha na paleta

| Ação | Tecla | Paleta |
|---|---|---|
| Visão geral / Arquivos / Commits | `Alt+1` / `Alt+2` / `Alt+3` | *Revisão: mostrar visão geral*, *mostrar arquivos*, *mostrar commits* |
| Comentar a linha ou o intervalo | `c` | *Revisão: comentar na linha...* |
| Abrir o código de uma thread (Visão geral) / voltar ao comentário | `G` / `Esc` | *Conversa: abrir o código da thread*, *Revisão: voltar ao comentário na visão geral* |
| Comentar na conversa | `c` na Visão geral, `C` | *Revisão: comentar na conversa...* (abre a caixa da Visão geral), *Conversa: novo comentário* |
| Responder à thread | `r` | *Revisão: responder à thread...* |
| Resolver ou reabrir | `x` | *Revisão: resolver ou reabrir a thread* |
| Recolher ou expandir uma thread | `z` | *Revisão: recolher ou expandir a thread* |
| Editar / excluir um comentário pendente | `e` / `d` | *Revisão: editar comentário pendente...*, *excluir comentário pendente* |
| Marcar o arquivo como visualizado | `v` | *Revisão: marcar arquivo como visualizado* |
| Inline / Dividido | `V` | *Revisão: alternar layout inline / dividido* |
| Arquivo todo / só os hunks | `H` | *Revisão: alternar arquivo todo / só hunks* |
| Alterações só de espaços em branco | `S` | *Revisão: alternar alterações só de espaços em branco* |
| Quebra de linha | `W` | *Revisão: alternar quebra de linha* |
| Próximo / anterior hunk | `J` / `K` | *Revisão: próximo hunk*, *hunk anterior* |
| Próximo arquivo / anterior | `]` / `[` | *Revisão: próximo arquivo*, *arquivo anterior* |
| Concluir / descartar a revisão | `f` | *Revisão: concluir revisão...*, *descartar revisão pendente...* |
| Atualizar | `F5`, `Ctrl+R` | *Revisão: atualizar* |
| Abrir no navegador / fazer checkout | `o` / `b` | *Revisão: abrir no navegador*, *fazer checkout da branch* |
| Responsáveis / revisores / rótulos / marco | `a` / `p` / `l` / `m` | *Revisão: editar responsáveis...* e as suas três irmãs |
| Escolher o patch set (Gerrit) | `P` | *Revisão: escolher patch set...* |
| Comparar com um patch set mais antigo (Gerrit) | `D` | *Revisão: comparar patch sets...* |
| Submeter / abandonar a change (Gerrit) | `U` / `Q` | *Revisão: submeter a solicitação...*, *Revisão: abandonar a solicitação...* |
| ✦ Gerar rascunho... (IA ativada; abre as opções de revisão) | `s` | *Revisão: ✦ gerar rascunho de revisão...* |
| Aceitar / editar e aceitar / dispensar uma sugestão | `y` / `w` / `n` | *Revisão: aceitar sugestão*, *editar e aceitar sugestão...*, *dispensar sugestão* |
| Aceitar / dispensar todas as sugestões | `A` / `N` | *Revisão: aceitar todas as sugestões*, *dispensar todas as sugestões* |
| Usar o resumo da IA no texto da revisão | `i` | *Revisão: usar o resumo da IA no texto da revisão* |
| Usar o ✦ rascunho (preenche Concluir revisão) | `u` | *Revisão: ✦ usar o rascunho...* |
| Editar o texto do rascunho | `t` | *Revisão: editar o texto do ✦ rascunho...* |
| Descartar o ✦ rascunho | `N` | *Revisão: descartar o ✦ rascunho* |
| Fechar | `Esc` | *Revisão: fechar* |

Abra a paleta (`Ctrl+P`) com a visualização aberta: as suas linhas agem sobre ela e dizem por que uma não faz nada
agora (“coloque antes o cursor em uma linha de código”).

## Revisando sem um clone

Uma solicitação de um repositório que não está na sua máquina ainda pode ser revisada. Pelo Radar, **Revisar**
(`e`) na linha dela pergunta como (veja [O Radar](radar.md#revisando-uma-solicitação)); **Revisar pela API** abre
esta mesma visualização na janela Início, sobre o Radar. Tudo nesta página funciona igual, com estas
diferenças:

- O código (os arquivos alterados, o diff de cada arquivo, os commits da solicitação, o diff de um commit, o arquivo
  inteiro) é lido pela API do host, então o primeiro olhar a uma solicitação grande leva um instante, e um arquivo para o qual o host não
  envia diff (um binário ou um muito grande) diz isso em vez de mostrar um.
- Os seus comentários pendentes ficam em `remote-reviews.toml` na pasta de configuração, arquivados sob o host
  e o projeto (`host/project`), em vez de no estado próprio de um repositório. Continuam ali quando você
  fecha e reabre a revisão, e enviar a revisão os consome.
- Não há escolha de codificação (*Indisponível sem um clone*) nem **Fazer checkout** (ele diz para clonar o
  repositório primeiro).
- `Esc` ou o `✕` no canto sai da visualização e devolve o Radar. A paleta (`Ctrl+P`) alcança as linhas da
  visualização como na aba de um repositório.

Para ficar com o clone, escolha **Clonar e depois revisar** (ou *Radar: clonar e revisar...* da paleta):
o diálogo Clonar abre na conta conectada da solicitação com o repositório dela selecionado, e a revisão
abre na aba do novo repositório assim que o clone termina.

## Pela linha de comando e para agentes

A mesma revisão está disponível como `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(e as ferramentas MCP feitas a partir deles): veja a [Referência da CLI](cli-reference.md). `threads` também lista os
vereditos já enviados (`reviews`). `suggest <n>` consulta o modelo (IA ativada) e imprime o seu `verdict`, `body`, resumo e as
sugestões em JSON sem publicar nada, de modo que um script ou um agente possa escolher os comentários a passar para
`submit --comment`.

Nada disto precisa de um clone: `--project owner/name` (com `--id` quando há várias integrações configuradas)
executa `show`, `threads`, `comment`, `reply`, `resolve`, `submit` e `suggest` contra um projeto que o diretório
atual desconhece. `hosting review diff <n>` imprime o diff com seus hunks (`--api` o lê do
host mesmo dentro de um clone), e `hosting file <path> --project P [--ref R]` lê um arquivo em uma revisão, por
exemplo o `CLAUDE.md` do projeto. Ambos leem a API do host: GitHub, GitLab, Gitea, Forgejo e Gerrit.
