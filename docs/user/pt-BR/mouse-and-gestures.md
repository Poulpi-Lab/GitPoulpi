[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · [Deutsch](../de/mouse-and-gestures.md) · [Español](../es/mouse-and-gestures.md) · [Français](../fr/mouse-and-gestures.md) · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · [Polski](../pl/mouse-and-gestures.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Mouse e gestos

Tudo o que você pode fazer com o mouse também tem um equivalente no teclado. Veja
[Atalhos de teclado](keybindings.md). Os nomes das áreas estão no
[tour pela interface](interface-tour.md).

## Regras gerais

- **Clique** em uma linha para selecioná-la. Isso vale em todo painel, lista, menu
  e diálogo.
- **Clique duplo** em uma linha para abri-la ou executá-la. Um clique duplo em uma
  branch local, uma tag ou uma pílula de ref faz checkout dela. Um clique duplo em
  uma linha de repositório abre o repositório.
- **Clique com o botão direito** em uma linha para ver o menu dela. `.`, `Shift+F10` ou a tecla
  Menu abre o mesmo menu para a linha selecionada.
- **A roda** rola a área sob o ponteiro. Em uma lista, ela só move a vista. A
  seleção continua onde está, mesmo quando sai da vista. Só um clique ou uma tecla
  move a seleção.
- **Shift+roda** rola para o lado, no diff, na Visualização de arquivo e no editor
  quando a quebra de linha está desativada.
- **Passar o mouse** sobre um botão, uma linha ou o polegar de uma barra de rolagem
  o acende. Muitos terminais não conseguem mudar o ponteiro do mouse, então esse
  brilho costuma ser o único sinal de que algo é clicável. Deixe o ponteiro parado
  sobre um botão ou uma célula do Log para ver uma dica.
- **Clique fora** de um campo de texto para sair dele. O que você digitou é mantido.
  O filtro de Refs aplica o que você digitou, como `Enter` faz. A barra de busca do
  Log se fecha, como `Esc` faz.

## Fechando janelas e diálogos

Toda janela ou diálogo com moldura tem um `✕` no canto superior direito da borda, e as
mesmas regras valem em todo lugar:

- **Clique no `✕`** para fechá-lo. O `Esc` faz o mesmo e fecha primeiro a coisa mais
  interna (um menu suspenso aberto, um campo de texto, uma confirmação) antes da janela.
- **Clique fora** de um diálogo, de um painel de ajuda ou de informações ou de um popup
  para fechá-lo. Um clique fora nunca fecha uma visão inteira (Configurações, a visão
  Diff, a Visualização de arquivo, os editores, Revisão, Quadro): elas não têm um fora.
- Um clique **dentro** de um painel nunca o fecha.
- Um formulário que guarda texto digitado por você (uma nova issue, um pull ou merge
  request) ignora um clique fora, para que um clique perdido não o apague. O `✕` e o
  `Esc` perguntam antes **Descartar alterações?**; **Voltar a editar** retorna. Um
  formulário sem nada digitado fecha na hora, e um diálogo de uma linha também.
- A paleta de comandos, os seletores (aba, tema, idioma), os pequenos diálogos de
  revisão, quadro, merge e compositor e o prompt de confirmação também têm o `✕`.
  Menus de contexto e listas suspensas não têm: eles fecham com `Esc` ou com um
  clique fora.

## Listas nas Configurações

As listas das Configurações (Perfis, perfis de IA, Fluxo de trabalho, Abrir em,
Imagens) respondem ao mouse assim. **Clique em uma subaba** (Geral ou a lista) para ir a ela; `Alt+Left` / `Alt+Right` faz o mesmo.

- **Clique** em uma linha para selecioná-la, **clique duplo** para ir aos campos dela,
  **roda** para rolar sem selecionar.
- **Pressione uma linha e arraste** para cima ou para baixo para reordenar: uma barra
  mostra onde ela vai cair, soltar a deixa ali, `Esc` cancela. O equivalente no teclado
  é `Alt+Up` / `Alt+Down`.
- **Clique em `+ Adicionar`, `Duplicar`, `Remover`, `↑`, `↓`** sob a lista. Teclas:
  `Insert`, `Ctrl+D`, `Delete`, `Alt+Up` / `Alt+Down`; veja
  [Configurações](settings.md#listas).

## Listas em popups

Estas regras valem para toda lista em um popup que tem um campo de filtro: a paleta
de comandos, seus seletores (tema, idioma, cores do grafo), o seletor de abas, os
menus suspensos pesquisáveis, a lista de referências de issues e a ajuda.

- **Clique** em uma linha para selecioná-la.
- **Clique duplo** em uma linha para executá-la ou escolhê-la, como `Enter` faz.
- **A roda** rola a lista. Ela não move a seleção.
- Quando a lista é mais longa que o popup, uma **barra de rolagem** aparece na borda
  direita. Arraste o polegar ou clique na trilha.

Um menu suspenso curto sem campo de filtro escolhe a linha no primeiro clique. Um
menu de contexto executa a linha no primeiro clique. As teclas dessas listas estão
em [Atalhos de teclado](keybindings.md#listas-de-popup-filtráveis).

## Barras de rolagem

Toda área que pode rolar mostra uma barra de rolagem fina na sua borda quando o
conteúdo não cabe: o Log, as seções de Refs, as listas de arquivos, o diff, a
Visualização de arquivo, o editor e as listas de popups.

- **Arraste o polegar** para rolar.
- **Clique na trilha** para mover uma página em direção ao clique. No Log e nas
  listas de arquivos, a vista pula para o clique.
- **Segure o botão em uma seta de ponta** para continuar rolando. Começa após um
  curto atraso. Saia de cima da seta para pausar.
- Uma barra de rolagem só move a vista. Ela nunca move a seleção.

No diff e no editor, a barra de rolagem vertical também mostra o arquivo inteiro em
pequenas marcas: verde para linhas adicionadas, vermelho para linhas removidas,
âmbar para linhas alteradas, `◆` para a linha do cursor e `▌` para cada outro lugar
em que o texto selecionado aparece. O editor de merge marca cada conflito em suas
barras de rolagem. Clique em uma marca para ir até lá.

## Falhas e o Registro de atividade

- **Clique no erro vermelho** da linha de status para abrir o Registro de atividade
  na entrada dele; **clique em Copiar / no botão sugerido** (Tentar de novo, Pull
  primeiro, Stash e tentar de novo, Abrir Integrações) da entrada.
- **Clique em Atividade** na extremidade direita da linha de status para abrir o
  Registro de atividade. Clique em uma aba para trocar, clique na caixa de um nível
  (ou em **Tudo** / **Nada**) para mostrar ou ocultar essas linhas, clique em uma
  entrada de Repositório para abrir os detalhes, clique em **Limpar registro** na aba
  Aplicativo (ele pergunta antes). Role as linhas longas para o lado
  com `Shift` + roda do mouse ou com a barra de rolagem horizontal. Veja
  [O Registro de atividade](troubleshooting.md#o-registro-de-atividade).
- **Clique em `[x]`** no indicador de tarefas à direita da linha de status para cancelar a tarefa em andamento, **clique no rótulo** para abrir o painel de tarefas. O botão **Atividade** no canto inferior direito da aba Início (ou `Ctrl+N Shift+G`) abre o Registro de atividade em Todos os repositórios (a caixa Escopo, uma lista suspensa sob as abas, escolhe um repositório ou um espaço de trabalho).

## Redimensionando

- **Arraste um divisor** entre a barra lateral Refs, o Log e o painel da direita.
  Arraste-o além da largura mínima para recolher esse painel. Os equivalentes de teclado movem o divisor um passo por vez: `Ctrl+N Left` / `Ctrl+N Right` (o divisor entre o painel Refs e o Log) e `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` (o divisor entre o Log e o painel da direita), o divisor indo para onde a seta aponta; a paleta os tem como `Navegação: alargar o painel Refs`, `Navegação: estreitar o painel Refs`, `Navegação: alargar o painel Status` e `Navegação: estreitar o painel Status`, e `Navegação: recolher/expandir o painel Refs` / `Navegação: recolher/expandir o painel Status`.
- **Arraste a borda inferior** de uma seção de Refs para mudar sua altura.
- **Arraste a borda inferior** do bloco da mensagem do commit no painel da direita.
- **Arraste a borda entre dois cabeçalhos do Log** (por padrão o espaço, a partir do cabeçalho ou de qualquer ponto da tabela; Configurações > Log e grafo > Redimensionamento das colunas) para redimensionar uma coluna (pelo teclado: `Shift+C` para escolher uma coluna, depois `]` / `[`).
- **Dê um clique duplo na borda entre dois cabeçalhos do Log** para ajustar a coluna à esquerda dela ao seu conteúdo (pelo teclado: `Shift+C` para escolher uma coluna, depois `=`; `+` ajusta todas as colunas). A última coluna tem uma alça na sua própria última célula: arraste-a para redimensionar essa coluna, com a coluna Mensagem absorvendo a diferença.
- **Role o Log para os lados** (a disposição de colunas `scroll`; Configurações > Log e grafo > Disposição das colunas, ou a caixa `Rolar para os lados` do seletor): a roda horizontal ou `Shift`+roda sobre a tabela, a barra na borda inferior (arraste o polegar, pressione a trilha vazia, clique nas setas das extremidades), ou `Shift+Left` / `Shift+Right`. Na disposição `fit` padrão, as colunas preenchem o painel e não há o que rolar.
- **Arraste um cabeçalho do Log** para mover a coluna: um fantasma do nome dele segue o ponteiro, o cabeçalho de onde ela saiu fica esmaecido e uma barra de destaque marca o lugar onde ela cairia. As colunas se movem ao soltar; `Esc` cancela, e o ponteiro mostra a mão aberta enquanto você a carrega (pelo teclado: `Shift+C` para escolher uma coluna, depois `<` / `>`).

O GitPoulpi lembra de todos os tamanhos. Veja
[Os tamanhos são lembrados](interface-tour.md#os-tamanhos-são-lembrados).

## Barra de título, linha de abas e barra de ferramentas

- **Clique em <img src="../../../assets/logo/gitpoulpi.png" alt="Ícone do GitPoulpi" height="16">** na barra de título para abrir **Sobre**. Em Sobre, as pupilas do
  polvo seguem o ponteiro; clique no logo para girá-lo até a próxima cor, clique com
  o botão direito nele para ver a lista de cores, clique em um link para abri-lo.
- **Clique em Abrir em ▾** na barra de título para ver a lista de programas. Veja
  [Abrir em](open-in.md).
- **Clique na versão** (`v1.0.0`, ou `vX.Y.Z disponível`) na barra de título para abrir a aba Notas da versão.
- **Clique na lupa e em `Busca global`** na barra de título, logo à esquerda da engrenagem, para a
  [busca global](search.md).
- **Clique na engrenagem** na barra de título, à esquerda do ×, para abrir as
  Configurações.
- **Clique em ×** na extremidade direita da barra de título para sair.
- **Clique em uma aba** para ir a ela. **Clique no `x` dela** (mostrado na aba ativa
  e na aba sob o ponteiro) para fechá-la. **Arraste uma aba** para o lado para
  reordená-la. **Clique com o botão direito em uma aba** para ver o menu da aba.
- **Clique com o botão do meio em uma aba** para fechá-la, como o `x` dela.
- **Clique em `‹` ou `›`** nas pontas das abas (mostrados quando elas não cabem todas) para rolar a linha uma aba; mantenha o botão pressionado para continuar rolando, ou use a roda sobre a linha. Arrastar uma aba sobre uma seta também rola a linha. A linha também rola sozinha para mostrar a aba atual sempre que ela muda.
- **Clique no `+`** depois da última aba para abrir um menu: Nova aba (`Alt+T`) e, enquanto as abas deles estão fechadas, Radar e Espaços de trabalho (`Alt+J`).
- **Clique no ícone da casa** para a aba Início.
- **Clique na aba Radar ou Espaços de trabalho** para abri-la; o `x` dela, um clique com o botão do meio ou `Ctrl+W` a fecha. **Clique na etiqueta do espaço de trabalho** (`Alt+W`) para trocar de espaço de trabalho.
- **Clique em Sair e cd aqui** (mostrado à direita das abas quando você iniciou o GitPoulpi pelo wrapper de shell `gp`) para sair e deixar o seu shell na pasta da aba ativa. Veja [Seguindo a pasta da aba ativa](following-the-tab-folder.md).
- **Clique em um botão da barra de ferramentas** para executá-lo. **Clique em `▾`**
  ao lado de Pull, Push ou LFS para ver a lista dele. **Clique em `⋯`** para os
  botões que não couberam.
- **Clique na caixa de combinação de perfil** na extremidade direita da barra de ferramentas
  para escolher a identidade de commit deste repositório.

## Aba Início

- **Clique** em uma linha para selecioná-la. **Clique duplo** nela para abrir o
  repositório.
- **Clique na estrela** para adicionar ou remover um favorito.
- **Clique em Abrir, Ir, Fechar ou Remover** em uma linha, ou em **`⋮`** para o menu
  da linha. **Clique com o botão direito** em uma linha para o mesmo menu.
- **Clique no título de uma seção** para recolher a seção.
- **Clique em Procurar, Clonar, Init, Recolher tudo ou Expandir tudo** no topo, e em
  **Fechar todas as abas** ou **Remover todos** no título de uma seção.

No seletor de pastas **Procurar**: clique em um local ou em uma pasta para ir até
ela, dê um clique duplo em uma pasta para abri-la, clique no campo de caminho para
digitar um caminho. Marque
**Mostrar pastas irmãs** (`Alt+S`, lembrado) e a árvore da esquerda também lista as
outras pastas de cada nível aberto: as pastas do caminho aberto mostram `▾`,
as outras `▸`, e um clique em uma delas vai até ela. Em **Clonar** e **Init**: clique em um campo para digitar nele
e clique em um botão ou caixa de seleção para usá-lo.

## Barra lateral Refs

- **Clique** em uma linha para selecioná-la. **Clique duplo** em uma branch local ou
  uma tag para fazer checkout dela. **Clique duplo** em um worktree ou submódulo
  para abri-lo em uma aba. O **+** no cabeçalho WORKTREES cria um worktree; clique com
  o botão direito em um worktree para Abrir em uma nova aba, Abrir em, Mesclar e Remover
  ([Worktrees](worktrees.md)).
- **Passe o mouse** sobre uma linha para mostrar o ícone à esquerda dela. **Clique**
  nele para ocultar a ref no Log, ou para mostrá-la de novo. Enquanto o **Solo**
  está ativado, o mesmo lugar faz solo da ref. Em uma linha de pasta, o ícone age em
  todas as refs da pasta.
- **Clique em Parar solo** ou **Mostrar tudo** no título da barra lateral para
  limpar todo solo ou todas as refs ocultas.
- **Clique com o botão direito** em uma linha para ver o menu: checkout, merge, fast-forward,
  rebase, criar um worktree, renomear, excluir, ocultar, solo e mais. A linha própria
  de um remoto tem **Fetch**, **Editar**, **Remover** e outras ações de remoto. Um
  stash tem **Pop**, **Apply** e **Drop**. Uma linha acinzentada diz por que não
  pode ser executada.
- **Clique em `+`** no título REMOTO para adicionar um remoto.
- **Clique em uma linha de pasta** para recolhê-la ou expandi-la.

### Pull requests e issues

| Gesto | Ação |
|---|---|
| Clicar em uma linha | Selecioná-la. |
| Clique duplo em uma linha | Abri-la no navegador. |
| Clique com o botão direito em uma linha | Seu menu: abrir, copiar a URL ou o número, fazer checkout da branch de origem, criar uma branch a partir de uma issue. |
| Clicar em `+` no título da seção | Abrir o formulário **Criar pull request** ou **Nova issue**. |
| Clicar na linha de filtro (`Open ▾`) | Escolher quais requisições ou issues listar. |
| Clicar no título de um grupo | Recolher ou expandir o grupo. |
| Clicar na linha do provedor ou do repositório (ISSUES) | Escolher a origem das issues. **Nenhum (desvincular este repositório)** a remove. |
| Clicar na linha de consulta (ISSUES) | Digitar uma consulta JQL ou WIQL (Jira, Azure DevOps), ou escolher listas do Trello. |
| Clicar no chip da barra de ferramentas (`GH #12`, `GL !53`, ...) | Abrir o pull request no navegador. Clique com o botão direito nele para **Abrir no navegador**, **Copiar URL**, **Atualizar** e **Trocar de perfil**. |
| Clicar no chip esmaecido `+ PR` | Iniciar um pull request para a branch enviada. |

Veja [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md).

## O Log

- **Clique** em um commit para selecioná-lo. Os detalhes dele aparecem no painel da
  direita.
- **Ctrl+clique** adiciona um commit à seleção ou o remove. **Shift+clique**
  seleciona um intervalo (**Alt+clique** nos terminais que ficam com o Shift+clique
  para a própria seleção de texto, como o kitty e o Windows Terminal). Clique com o
  botão direito em um commit selecionado para fazer cherry-pick de todos. A seleção pode incluir a
  linha WIP e os stashes; duas ou mais linhas transformam o painel direito em uma
  comparação, veja [Comparando commits](log-and-history.md#comparando-commits).
- **Clique em um cartão** do painel de comparação para levar o cursor do Log até
  aquela linha; **clique em Trocar** para inverter as extremidades De e Até. Um
  clique simples em uma linha sai da comparação.
- **Clique na linha WIP** para as alterações em andamento. **Clique no texto
  `// WIP`** dela para digitar ali o resumo do commit.
- **Passe o mouse** sobre uma célula para ver o texto completo: a mensagem, o autor
  e e-mail, a data completa, o hash completo.
- **Passe o mouse sobre um selo `+N`** para ver as outras refs do commit. Mova-se
  para dentro do popup para clicar em uma ref ali. Clique duplo em uma ref do popup
  faz checkout dela.
- **Clique duplo em uma pílula de ref** para fazer checkout dela.
- **Clique com o botão direito em um commit** para o menu do commit. Ele tem um submenu para cada
  ref do commit.
- **Clique com o botão direito em uma pílula de ref** para um único menu com as ações da ref e as
  ações do commit.
- **Clique com o botão direito no cabeçalho** para mostrar ou ocultar colunas: uma lista de verificação (`[x]` / `[ ]`) com um ✕ no título; clique em uma linha ou use as setas e `Space` / `Enter`, depois feche com o ✕, `Esc` ou um clique fora (a paleta: `Exibir: Log: escolher as colunas a mostrar`). Abaixo das caixas: `Ajustar <coluna> ao conteúdo` (a coluna clicada com o botão direito), `Ajustar todas as colunas ao conteúdo` e `Redefinir as larguras das colunas`. O equivalente no teclado do clique direito: `Shift+C` para escolher uma coluna e depois `.` (ou `Shift+F10`) abre o mesmo menu nela.
- **As colunas do Radar** funcionam do mesmo jeito: arraste o separador depois da coluna de repositório, `#`, autor ou idade para dimensioná-la, **dê um clique duplo nele** para ajustar a coluna à sua entrada mais larga (`C` escolhe uma coluna, depois `=`; `+` ajusta todas as colunas), e **clique com o botão direito no título de uma coluna** (ou `C`, depois `.`) para `Ajustar <coluna> ao conteúdo` e `Ajustar todas as colunas ao conteúdo`.
- **Clique em `↑`, `↓` ou `✕`** na barra de busca para ir à correspondência anterior
  ou seguinte, ou para fechar a barra.
- **Clique em `#12`, `ABC-12` ou `AB#12`** em uma mensagem de commit, no painel da
  direita, para abrir essa issue ou pull request no navegador.

### Seções do painel esquerdo

**As pastas** (branches e tags com nomes como `a/b/c`, os arquivos do painel direito na
visão em árvore) se dobram e desdobram de uma vez: **clique com o botão direito no cabeçalho ou no espaço
vazio de uma seção** que tem pastas para **Expandir tudo** / **Recolher tudo**
(e **Mostrar/ocultar seções...**, que leva à lista de verificação abaixo), e
**clique com o botão direito em uma linha de pasta** para **Expandir todas as subpastas** / **Recolher
todas as subpastas** (a pasta e tudo o que há abaixo dela). Teclas, no painel Refs, nas
listas Unstaged e Staged e nos arquivos alterados de um commit: `=` expande tudo,
`-` recolhe tudo, `*` (ou `Shift+Right`) expande a pasta sob o cursor
e tudo abaixo dela, `_` (ou `Shift+Left`) a recolhe. Paleta: *Refs:
expandir todas as pastas*, *Refs: recolher todas as pastas*, *Refs: expandir a pasta
selecionada e suas subpastas*, *Refs: recolher a pasta selecionada e suas
subpastas* e as quatro gêmeas *Arquivos:* para a lista de arquivos em foco. Enquanto um filtro de
nome está ativo (o filtro de Refs, *Filtrar arquivos* de um commit), toda pasta
que contém uma correspondência aparece aberta, não importa o que você tinha dobrado, e as suas
próprias dobras não são alteradas nem salvas: limpe o filtro e elas voltam. Uma dobra feita
durante o filtro age sobre o que é mostrado e é esquecida junto com o filtro.

**Clique com o botão direito no cabeçalho de uma seção** (uma seção sem pastas, ou a parte vazia do painel
esquerdo) para ver uma lista de verificação das seções: Local, Remoto, Worktrees,
Stashes, Tags, Pull requests, Issues e Submódulos. Um clique desmarca ou marca uma
seção; o menu continua aberto. Uma seção oculta não é desenhada e o teclado a pula.
A última seção visível não pode ser desmarcada. A escolha vale por repositório e fica guardada entre reinícios; um repositório sem escolha própria segue o padrão (o `ui.sidebar_sections_hidden` global). *Usar em todos os repositórios* torna a escolha atual o padrão, e *Restaurar o padrão* descarta a escolha própria deste repositório (paleta: *Painel esquerdo: usar estas seções em todos os repositórios*). Paleta: *Painel esquerdo: mostrar ou
ocultar seções...*, um comando *mostrar ou ocultar* por seção e *Painel esquerdo:
restaurar as seções padrão*.

### Arrastando refs

Pressione uma branch na barra lateral Refs, ou uma pílula de ref no Log, e arraste-a
sobre outra branch. A linha de status diz o que você está carregando. Um destino que pode
receber o item se acende. Solte para obter um menu. `Esc` cancela o arrastar.

| Arrastar | Menu |
|---|---|
| Branch local X sobre a branch local Y | **Merge de X em Y**, **Rebase de X sobre Y**, **Rebase interativo de X sobre Y...**, **Fast-forward de Y até X** |
| Branch local sobre uma branch remota | **Push de X** para esse remoto |
| Branch remota sobre a branch local Y | **Merge**, **Rebase de Y sobre ela**, **Fast-forward de Y até ela** e **Pull** quando ela é o upstream de Y |

Quando a ação altera uma branch que não está em checkout, o GitPoulpi faz checkout
dela primeiro. Se o git recusar o checkout, nada muda. Tags não podem ser arrastadas
e não podem receber um item solto.

Teclado: “marcar e soltar” faz o que um arrasto faz, para qualquer par de refs.
No painel Refs, pressione `m` em uma branch (paleta: `Refs: marcar como origem do
soltar`). A ref mostra um `◆` e fica sublinhada, também nas pílulas dela no Log, e a
linha de status diz aonde ir. Selecione outra ref, em qualquer seção, e pressione
`Shift+M` (paleta: `Refs: soltar aqui a ref marcada...`): aparece o mesmo menu que um
arrasto abre, e as linhas dele rodam exatamente como depois de um arrasto. A marca
é gasta quando o menu abre; `Esc`, ou `m` na ref marcada, a remove antes. O comando
da paleta `Refs: soltar a ref selecionada na branch em checkout...` abre o menu para
a ref selecionada e a branch atual.

## O painel da direita

### Alterações em andamento

- **Clique** em um arquivo para abrir o diff dele. **Clique no mesmo arquivo de
  novo** para voltar ao Log. Se o arquivo está aberto no editor com alterações não
  salvas, o GitPoulpi pergunta antes.
- **Passe o mouse** sobre uma linha para mostrar o botão **Stage arquivo** ou
  **Unstage arquivo** dela. Uma linha de pasta tem **Stage pasta** ou **Unstage
  pasta**.
- **Clique em Stage tudo** ou **Unstage tudo** no título de uma seção.
- **Clique no título de uma seção** (`Arquivos unstaged`, `Arquivos staged`) para
  recolhê-la.
- **Clique em Caminho ou Árvore** para escolher uma lista plana ou pastas. **Clique
  em Recolher tudo** ou **Expandir tudo** para recolher cada pasta.
- **Clique em `✗`** à esquerda do título para descartar todas as alterações
  (pergunta antes). **Clique em `✦`** à direita para explicar as alterações com a
  IA.
- **Clique com o botão direito em um arquivo** para ver o menu: stage, descartar, ignorar, LFS,
  fazer stash do arquivo, histórico, blame, abrir em um programa externo, copiar o
  caminho, criar um patch, editar, excluir. Um arquivo em conflito começa com as
  linhas **Resolver com ...**.
- **Clique com o botão direito em uma linha de pasta** para o menu da pasta: stage, descartar,
  ignorar, stash, criar um patch, abrir a pasta, criar um arquivo nela.
- **Clique com o botão direito na parte vazia** de Arquivos unstaged para **Criar arquivo**.
- **Ignorar**, **Tratar como binário** e **Tratar como texto** (menus de arquivo e pasta) abrem primeiro uma lista dos arquivos `.gitignore` ou `.gitattributes` onde a regra pode ir. Clique em uma linha e pressione `Enter` (ou use as setas); `Esc` ou um clique fora cancela sem escrever nada.
- **Ctrl+clique** em uma linha de arquivo ou de pasta a adiciona à seleção ou a remove, **Shift+clique** (**Alt+clique**) pega um intervalo; o mesmo nos arquivos alterados de um commit e em Ver todos os arquivos. Um clique simples recomeça. **Clique com o botão direito em uma linha selecionada**: o menu de todos os arquivos selecionados.
- **Clique com o botão direito na parte vazia** de uma lista também oferece **Abrir a pasta do repositório**; Unstaged acrescenta **Stage tudo** e **Descartar todas as alterações...**, Staged acrescenta **Unstage tudo**.
- **Converter** (menus de arquivo, pasta e seleção) reescreve a codificação, os fins de linha ou a indentação dos arquivos no disco; um `Ctrl+Z` desfaz o lote.

### O formulário de commit

- **Clique** no Resumo ou na Descrição para digitar neles.
- **Clique em Amend do commit anterior** para ativar ou desativar o amend.
- **Clique em Commit das alterações** para fazer o commit. Quando o botão está
  acinzentado, um clique diz por quê (por exemplo, *Faça stage das alterações para o
  commit*).
- **Clique em `✦`** à direita do Resumo para escrever a mensagem com a IA.
- **Clique em `#▾`** no fim do Resumo para ver a lista de issues. Clique em uma para
  inserir a referência dela.
- **Clique em Compor commits com IA** para abrir o compositor de commits.

### Detalhes do commit

- **Clique** em um arquivo alterado para abrir o diff dele. **Clique de novo** para
  voltar ao Log.
- **Arraste** a borda inferior da mensagem para redimensioná-la. A roda sobre a
  mensagem rola uma descrição longa.
- **Clique com o botão direito** no resumo, na descrição ou no bloco do autor para
  *Alterar o resumo...*, *Alterar a descrição...* ou *Alterar o autor...*; **clique duas vezes** em um deles (ou na data de autoria) para
  abrir o diálogo Alterar completo, com o cursor nesse campo. Os equivalentes de teclado são `Ctrl+G Shift+E` (todos os campos) e `Ctrl+G Shift+A` (o autor); o resumo e a descrição estão na paleta.
- **Clique com o botão direito** em um arquivo alterado, uma linha de pasta, a parte vazia da lista ou uma seleção para o seu menu (abrir, histórico, blame, abrir em um programa, copiar caminhos, ignorar, tratar como binário ou texto, converter, restaurar do commit, excluir). **Ctrl+clique** e **Shift+clique** selecionam vários arquivos.

Veja [Stage e commit](staging-and-committing.md).

## O diff e a Visualização de arquivo

- **Clique em um botão** na barra acima do diff: Arquivo, Diff, Blame,
  o seletor de coluna `Histórico │ Todos`, Editar, `↑` `↓`, Inline, Dividido, Hunks, EB, Quebra, Mapa, Codificação. Um botão ativado fica
  em destaque. Um clique em `Histórico` ou `Todos` mostra essa lista na coluna de
  commits; um clique no segmento já selecionado oculta a coluna (veja
  [A coluna de commits](interface-tour.md#a-coluna-de-commits)).
- **Clique em `✕`** no título para fechar o arquivo.
- **Passe o mouse sobre um hunk** para mostrar **Stage hunk** ou **Unstage hunk**, e
  **Descartar hunk**.
- **Passe o mouse sobre uma linha alterada** para mostrar os botões próprios dela
  **+ Stage**, **− Unstage** e **✗ Descartar** na extremidade direita da linha.
- **Clique com o botão direito em uma linha** para o menu do hunk: stage, unstage ou descartar o
  hunk ou as linhas selecionadas, histórico do arquivo, blame, copiar.
- **Clique no minimapa** para mover a vista até aquela linha. Mantenha o botão
  pressionado e mova para rolar.

Todo descarte pergunta antes. Quando um descarte exclui arquivos untracked, as pastas que ficam completamente vazias vão junto (nunca uma que ainda guarde um arquivo rastreado, ignorado ou oculto, nem uma que já estava vazia); Desfazer traz os arquivos de volta com as pastas.

### Blame e histórico do arquivo

- **Blame.** Deixe o ponteiro sobre uma anotação para ver o cartão do hunk; um clique seleciona um hunk, um clique com o botão direito abre seu menu. Arraste a borda da coluna de anotações para redimensioná-la (salvo). Clique ou arraste no minimapa para rolar.
- **Histórico do arquivo.** Um clique seleciona um commit; `Shift+clique` seleciona um intervalo para comparar; um clique duplo mostra o commit no Log; um clique com o botão direito abre o menu da linha; arraste a borda entre a lista e o diff para redimensioná-la; o ✕ da lista a oculta (clique na tira `›` para trazê-la de volta); clique em `Arquivo` / `Diff` / `Blame` na barra para trocar o que o lado direito mostra, e no seletor `Histórico │ Todos` para escolher a lista da coluna. A roda rola a metade sob o ponteiro. Cada um tem seu equivalente de teclado ([Blame](log-and-history.md#blame), [Histórico do arquivo](log-and-history.md#histórico-do-arquivo)).

### A coluna de commits (lista Todos)

- **Passe o mouse sobre uma linha** para a dica do Log (a mensagem inteira, o nome e o e-mail do autor, a data completa, conforme a parte sob o ponteiro).
- **Clique com o botão direito em uma linha** para o menu dela: mostrar no Log, o diff do arquivo nesse commit, o arquivo nesse commit, o blame dele e depois o menu do Log para o commit. A tecla de menu faz o mesmo pelo teclado.
- **Arraste um limite do cabeçalho** (`ui.column_resize`: por padrão a célula separadora de uma célula entre duas partes, marcada por um `│` esmaecido no cabeçalho e desenhada como uma linha de destaque ao longo da coluna enquanto o ponteiro está sobre ela, e um arrasto a partir de qualquer ponto dela redimensiona; com `header`, a primeira letra do nome seguinte, iluminada enquanto o ponteiro está sobre ela) para dimensionar a parte à esquerda dele: faixas, mensagem, autor, data, refs, alterações ou SHA. A célula logo depois da última parte é o limite da última parte.
- **Dê um clique duplo em um limite do cabeçalho** para ajustar a parte à esquerda dele ao seu conteúdo, como fazem os limites do Log: a largura que mostra a sua linha mais larga inteira, nunca maior que a coluna (pelo teclado: clique em uma linha ou `Ctrl+N` para dar as teclas à coluna, `Tab` para escolher a parte, depois `=`; `+` ajusta todas as partes; a paleta tem `Ajustar a coluna ao conteúdo` e `Ajustar todas as colunas ao conteúdo`). A mensagem fica com o que as outras partes deixam.
- **Arraste um nome do cabeçalho** sobre outro para mover a parte até lá: o mesmo fantasma, a origem esmaecida e a barra de destaque do cabeçalho do Log, e a parte se move ao soltar (`Esc` cancela; `<` / `>` do teclado, após `Tab`, fazem o mesmo, um lugar por vez).
- **Clique com o botão direito no cabeçalho** para escolher quais partes são exibidas (Ref, Grafo, Autor, Data, Alterações, SHA; a Mensagem está sempre visível); a lista tem um ✕ no título. Abaixo dela: `Ajustar <parte> ao conteúdo` (a parte clicada com o botão direito) e `Ajustar todas as colunas ao conteúdo`. A paleta tem `Escolher as colunas` (o menu pelo teclado, na parte que `Tab` escolheu) e uma alternância por parte.
- **Shift+roda** ou uma roda horizontal rola as linhas para os lados quando elas são mais largas que a coluna.
- **Arraste a borda direita** da coluna para redimensioná-la. **Clique na tira `›`** (ou no ✕ da coluna, ou **clique no segmento selecionado** do seletor `Histórico │ Todos` da barra) para trazê-la de volta ou ocultá-la.

### Selecionando e copiando texto

Você pode selecionar e copiar o texto do diff, da Visualização de arquivo, do Blame
e do Histórico do arquivo.

- **Pressione e arraste** para selecionar texto. A seleção continua no mesmo texto
  quando você rola.
- **Clique duplo** para selecionar uma palavra.
- **Um clique simples** não seleciona texto. Ele move o cursor ou seleciona a linha,
  e limpa a seleção de texto.
- **`y` ou `Ctrl+C`** copia a seleção. Sem seleção, copiam a linha atual.
- **Clique com o botão direito** no diff para **Copiar seleção**, **Copiar linha** e **Copiar
  hunk**.

O GitPoulpi copia só o texto do arquivo. Números de linha, marcas `+` e `-`,
cabeçalhos de hunk e as colunas do Blame não são copiados. No modo Dividido, uma
seleção fica no lado em que você começou o arraste.

No diff, um arraste também seleciona as linhas que ele cruza, como `v` faz. `Enter`
então faz stage dessas linhas. `Esc` primeiro limpa a seleção de texto, depois a
seleção de linhas, depois fecha o diff.

Selecionar texto também colore todos os outros lugares em que esse texto aparece no
diff e no editor.

## Campos de texto

Isto vale para todo campo de texto: filtros, a barra de busca, a paleta, o Resumo e
a Descrição, campos de diálogo e valores das Configurações.

- **Clique** para colocar o cursor ali.
- **Pressione e arraste** para selecionar texto.
- **Clique duplo** para selecionar uma palavra.
- Digite para substituir a seleção. `Backspace` ou `Delete` a remove.

As teclas dos campos de texto estão em [Atalhos de teclado](keybindings.md).

## O editor

- **Clique** para colocar o cursor ali. **Shift+clique** estende a seleção.
- **Arraste** para selecionar. Arraste além da borda superior ou inferior para rolar.
- **Clique duplo** para selecionar uma palavra.
- **A roda** rola sem mover o cursor.
- **Clique em Salvar, Desfazer, Refazer ou Buscar** na barra. No modo de busca,
  **`↑`** e **`↓`** vão para a correspondência anterior e a seguinte.
- **Clique em `✕`** para fechar. O GitPoulpi pergunta antes quando há alterações
  não salvas.

## Imagens

Veja [Imagens](image-view.md) para o mouse na visualização de imagem.

## Mover e redimensionar diálogos

Os diálogos podem ser movidos, e as janelas maiores redimensionadas:

- **Arraste a barra de título** para mover um diálogo (a cruz de fechar mantém o clique). A paleta de comandos e os seletores não têm barra de título própria: arraste a borda superior.
- **Arraste a borda esquerda, direita ou inferior**, ou um canto inferior (a alça `◢`), para redimensionar uma janela. O diálogo fica na tela e nunca encolhe abaixo do seu mínimo (60 x 16 para o Histórico de desfazer, menor para os outros). Uma janela redimensionada aproveita o espaço: áreas de texto e listas crescem, o texto quebra na nova largura.
- O arrasto continua quando o ponteiro sai do diálogo, e soltar fora nunca o fecha.
- **Arraste os divisores** dentro da visão Reflog / Trabalho perdido do Histórico de desfazer; um clique duplo em um deles o restaura.
- Os equivalentes de teclado são `Ctrl+N` e depois `m` (as setas movem, `Shift`+setas redimensionam, `0` restaura, `Enter` ou `Esc` saem) e `Alt`+setas para os divisores; a paleta tem `Diálogo: mover ou redimensionar` e `Diálogo: restaurar posição e tamanho`. Eles agem no diálogo do topo. `Ctrl+N` funciona igual em toda tela: mostra um selo nos controles da janela, do diálogo ou do popup da frente (campos, listas, abas, botões e a cruz de fechar), a tecla seguinte aciona esse controle, `m` inicia o modo de ajuste e `Esc` cancela; a linha de status indica `<selo>`, `m=ajustar` e `Esc`. Vale também para a paleta de comandos, os seletores, a busca no workspace e a sobreposição de ajuda, onde `Ctrl+N` não desce mais na lista. `Alt+G` (`palette.arrange`) é mais um jeito, válido sobre todo diálogo e popup, de ajustar o da frente sem o prefixo.
- **Uma só configuração** (`ui.dialog_geometry`, Configurações > Aparência > Layout) vale para todos os diálogos: cada um lembra a própria posição e o próprio tamanho nesta sessão, em disco para a próxima abertura, ou nunca.

| Diálogo | Mover | Redimensionar |
|---|---|---|
| Histórico de desfazer / Reflog / Trabalho perdido, Log de atividade, painel de tarefas, seletor de pasta | sim | sim |
| Paleta de comandos, seletores de aba / tema / idioma, busca no workspace | sim | sim (a lista cresce com a altura, as linhas com a largura) |
| Clone (a lista de repositórios), Pull request e Nova issue (a descrição), "O que é enviado" (o painel), Mermaid (a prévia), opções de revisão (os campos, só a largura) | sim | sim |
| Editores de texto longo das Configurações (modelos, hooks, prompts de IA), a caixa de IA do formulário de workspace | sim | sim (a área de texto) |
| Reescrita do histórico: os dois diálogos auxiliares de IA | sim | sim (a descrição, o painel enviado) |
| Sobre, painel de atualização, verificação de segurança, conflitos previstos, "Acima do limite de IA", exclusão de tag, arquivos sensíveis | sim | não (o tamanho segue o texto) |
| Init, Worktree, Sparse checkout, diálogo Flow, diálogos de fluxo de issues, formulário de mensagem de commit (reword), formulário de Workspace, seletores de rótulo / marco | sim | não |
| Confirmações (todo "Tem certeza?") | sim | não |
| Diálogos de Revisão e de Quadro: seus pequenos diálogos (comentário, concluir, escolher, colunas) | sim | não |
| Popups da página de atalhos (confirmação de redefinição, captura de tecla), o popup de resposta da IA | sim | não |
| Configurações: os resultados e a confirmação da autenticação do repositório | sim | não |
| Reescrita do histórico: diálogos de purge, de identidade e de pergunta | sim | não |
| Popup de mover arquivo do compositor de commits, avisos de merge | sim | não |

Não podem ser movidos: as telas de página inteira (Configurações, Ajuda, as páginas de Revisão e de Quadro, que já ocupam a janela), menus de contexto e listas suspensas (ficam presos ao que os abriu), dicas (tooltips) e os pequenos popups dentro de uma página (o formulário de remotos, o editor de divisão de commit).

O ponteiro mostra uma forma de mover sobre a barra de título e uma forma de redimensionar sobre as bordas e os cantos (terminais com `OSC 22`).

## Forma do ponteiro do mouse

Alguns terminais deixam o GitPoulpi mudar o ponteiro do mouse: um ponteiro de
redimensionamento sobre os divisores, sobre uma borda de coluna do Log, do Radar e da coluna de commits (também ao arrastá-la), uma mão sobre itens clicáveis. Isso funciona
no kitty, foot, Ghostty, mintty e em uma versão do WezTerm com o PR upstream #8092 (a sequência
`OSC 22`). Ela é enviada só quando a forma muda, dentro do tmux só embrulhada para a
passagem direta (`allow-passthrough on`; kitty e foot a mostram, Ghostty não), e nunca no screen,
zellij ou herdr. O ponteiro padrão é restaurado quando o GitPoulpi sai. Em outros terminais (Rio,
Alacritty, Tabby, iTerm2, Terminal.app, Windows Terminal, xterm), o ponteiro não muda; veja
[Escolhendo um terminal](terminals.md#formato-do-ponteiro-do-mouse-osc-22). O divisor ou o botão
sob o ponteiro se acende em todos os terminais.

O mesmo tipo de componente dá o mesmo ponteiro em todo o aplicativo.

- **Os botões são botões de verdade**, nas Configurações (Integrações: Editar,
  Desconectar, Remover, Procurar, Excluir, os botões de trecho do editor de instruções...)
  como nos diálogos: rótulos com margem sobre um fundo preenchido que clareia sob o
  ponteiro, esmaecidos quando desativados, nunca texto entre `[ ]`. Os únicos colchetes que
  restam são as marcas das caixas de seleção `[x]` / `[ ]`, que são caixas de seleção e
  levam a mão como qualquer outro controle.
- **A mão que aponta** fica nos controles: botões, caixas de seleção e interruptores,
  caixas combinadas (nas Configurações, nos painéis de issues e pull requests, no Radar,
  nos diálogos), abas e suas cruzes, os botões `+`, links e referências
  (`#12`, `!45`), cabeçalhos de grupo e de pasta com a seta de expandir, pílulas de ref,
  barras de rolagem, e os botões Atividade, tarefas e versão da barra de status. Uma
  caixa de seleção ou um botão dentro de uma linha mantém a mão.
- **A seta** fica nas linhas. Uma linha se acende por inteiro quando o ponteiro está sobre ela
  (a mesma luz em todas as listas) e não é um controle por si só: as linhas dos
  painéis Refs, Worktrees, Stashes e Submódulos, o Log e a coluna de commits
  (só uma pílula de ref leva a mão), as linhas de um diff, os arquivos alterados,
  os itens de pull request, issue e Radar, e as linhas das listas em
  diálogos e painéis (clone, seletor de pastas, Workspaces, os cartões do quadro, o
  painel de tarefas, seletores de responsável, rótulo e marco, passos do rebase, arquivos
  e commits de revisão, escolhas de menus e de caixas combinadas).
- **O cursor de texto (I-beam)** fica nas caixas de entrada, sobre a caixa inteira: as caixas
  de busca (busca global, Configurações, a paleta de comandos, Home), os filtros (Refs,
  arquivos, Radar, Log de atividade, a busca do painel de issues), as linhas de texto, número e
  área das Configurações (também enquanto você as edita), os campos dos diálogos, as
  caixas de comentário de revisão, o compositor de commits, o texto do editor de arquivos e o
  resultado editável do editor de merge (os painéis somente leitura mantêm a seta). Nos
  detalhes do commit, a mensagem e a descrição são editáveis, então o I-beam aparece
  nelas e o bloco inteiro se acende de leve.
- **Nos detalhes do commit**, as linhas de informação são controles: clique em `commit: <id>`
  para copiar o id curto, dê um clique duplo na linha do autor ou da data para emendá-la
  (a mão e uma marca leve as indicam). A linha de estatísticas só informa e mantém a
  seta.

Os ponteiros de redimensionamento ficam sobre divisores e bordas de coluna, e a mão aberta
aparece só enquanto você carrega uma ref.

## Copiando para a área de transferência

As ações de copiar colocam o texto na área de transferência do sistema da máquina em que o
GitPoulpi roda e, onde o terminal é conhecido por respeitá-lo, também o enviam ao terminal
(OSC 52). Por SSH, ou sem tela, só o OSC 52 é enviado: se o seu terminal não o suporta
(iTerm2 até a permissão de área de transferência dele ser ativada, Terminal.app, GNU screen),
nada é copiado e a linha de status diz que o texto só foi enviado ao terminal. Veja
[Escolhendo um terminal](terminals.md#área-de-transferência-ações-de-copiar).
