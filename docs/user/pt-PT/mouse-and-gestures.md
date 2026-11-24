[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · [Deutsch](../de/mouse-and-gestures.md) · [Español](../es/mouse-and-gestures.md) · [Français](../fr/mouse-and-gestures.md) · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · [Polski](../pl/mouse-and-gestures.md) · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · **Português (Portugal)** · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Rato e gestos

Tudo o que pode fazer com o rato tem também um equivalente de teclado. Veja
[Atalhos de teclado](keybindings.md). Os nomes das áreas estão na
[visita guiada à interface](interface-tour.md).

## Regras gerais

- **Clique** numa linha para a selecionar. Funciona em todos os painéis, listas,
  menus e diálogos.
- **Clique duplo** numa linha para a abrir ou executar. Um clique duplo numa
  branch local, numa tag ou numa pílula de ref faz checkout dela. Um clique duplo
  numa linha de repositório abre o repositório.
- **Clique com o botão direito** numa linha para ver o seu menu. `.`,
  `Shift+F10` ou a tecla Menu abre o mesmo menu para a linha selecionada.
- **A roda** desloca a área sob o ponteiro. Numa lista, move apenas a vista. A
  seleção fica onde está, mesmo quando sai da vista. Só um clique ou uma tecla
  move a seleção.
- **Shift+roda** desloca para o lado, no diff, na vista de ficheiro e no
  editor quando a quebra de linha está desligada.
- **Passar o ponteiro** sobre um botão, uma linha ou o indicador de uma barra de
  deslocação acende-o. Muitos terminais não conseguem mudar o ponteiro do
  rato, por isso este realce é muitas vezes o único sinal de que algo é
  clicável. Pare o ponteiro sobre um botão ou uma célula do Log para ver uma
  dica.
- **Clicar fora** de um campo de texto sai dele. O que escreveu é mantido. O
  filtro de Refs aplica o que escreveu, tal como `Enter`. A barra de pesquisa do
  Log fecha-se, tal como `Esc`.

## Fechar janelas e diálogos

Cada janela com moldura e cada diálogo tem um `✕` no canto superior direito da
borda, e as mesmas regras valem em todo o lado:

- **Clique no `✕`** para fechar. `Esc` faz o mesmo e fecha primeiro o que estiver
  mais por dentro (um menu pendente aberto, um campo de texto, uma confirmação) e só
  depois a janela.
- **Clique fora** de um diálogo, de um painel de ajuda ou de informação ou de um
  popup para o fechar. Um clique fora nunca fecha uma vista inteira (Definições, a
  vista de diff, a vista de ficheiro, os editores, a Revisão, o Quadro): essas não
  têm "fora".
- Um clique **dentro** de um painel nunca o fecha.
- Premir o rato (botão esquerdo ou direito) enquanto um acorde `Ctrl+G` ou `Ctrl+N` espera pela segunda tecla cancela o acorde como `Esc` e depois age como de costume (um clique continua a selecionar, um clique direito continua a abrir o seu menu).
- Um formulário com texto que escreveu (uma nova issue, um pull ou merge request)
  ignora um clique fora, para que um clique perdido não o faça perder. `✕` e `Esc`
  perguntam primeiro **Descartar alterações?**; **Continuar a editar** volta atrás.
  Um formulário sem nada escrito fecha logo, tal como um diálogo de uma só linha.
- A paleta de comandos, os seletores (separador, tema, idioma), os pequenos
  diálogos de revisão, quadro, merge e compositor e o pedido de confirmação
  também têm o `✕`. Os menus de contexto e os menus pendentes não têm: fecham
  com `Esc` ou com um clique fora.

## Listas nas Definições

As listas das Definições (Perfis, perfis de IA, Fluxo de trabalho, Abrir em,
Imagens) respondem ao rato assim. **Clique num subseparador** (Geral ou a lista) para mudar para ele; `Alt+Left` / `Alt+Right` faz o mesmo.

- **Clique** numa linha para a selecionar, **duplo clique** para ir aos seus
  campos, **roda** para deslocar sem selecionar.
- **Prima uma linha e arraste-a** para cima ou para baixo para a reordenar: uma
  barra mostra onde vai ficar, soltar larga-a, `Esc` cancela. O equivalente no
  teclado é `Alt+Up` / `Alt+Down`.
- **Clique em `+ Adicionar`, `Duplicar`, `Remover`, `↑`, `↓`** por baixo da lista.
  Teclas: `Insert`, `Ctrl+D`, `Delete`, `Alt+Up` / `Alt+Down`; veja
  [Definições](settings.md#listas).

## Listas em popups

Estas regras aplicam-se a todas as listas de um popup que tenha um campo de
filtro: a paleta de comandos, os seus seletores (tema, idioma, cores do grafo),
o seletor de separadores, as listas pendentes com pesquisa, a lista de
referências de issues e a ajuda.

- **Clique** numa linha para a selecionar.
- **Clique duplo** numa linha para a executar ou escolher, como `Enter`.
- **A roda** desloca a lista. Não move a seleção.
- Quando a lista é mais comprida do que o popup, aparece uma **barra de
  deslocação** na sua margem direita. Arraste o indicador ou clique na pista.

Uma lista pendente curta sem campo de filtro escolhe a linha no primeiro clique.
Um menu de contexto executa a linha no primeiro clique. As teclas destas listas
estão em [Atalhos de teclado](keybindings.md#listas-de-popup-filtráveis).

## Barras de deslocação

Cada área que pode deslocar-se mostra uma barra de deslocação fina na sua
margem quando o conteúdo não cabe: o Log, as secções de Refs, as listas de
ficheiros, o diff, a vista de ficheiro, o editor e as listas dos popups.

- **Arraste o indicador** para deslocar.
- **Clique na pista** para andar uma página em direção ao clique. No Log e nas
  listas de ficheiros, a vista salta para o clique.
- **Mantenha o botão premido numa seta das pontas** para continuar a deslocar.
  Começa após um breve atraso. Afaste-se da seta para fazer pausa.
- Uma barra de deslocação move apenas a vista. Nunca move a seleção.

No diff e no editor, a barra de deslocação vertical mostra também o ficheiro
todo em pequenas marcas: verde para linhas adicionadas, vermelho para linhas
removidas, âmbar para linhas alteradas, `◆` para a linha do cursor e `▌` para
cada outro sítio onde o texto selecionado aparece. O editor de merge marca cada
conflito nas suas barras de deslocação. Clique numa marca para ir para lá.

## Falhas e o Registo de atividade

- **Clique no erro vermelho** da barra de estado para abrir o Registo de atividade na
  entrada dele; **clique em Copiar / no botão sugerido** (Tentar de novo, Pull primeiro,
  Stash e tentar de novo, Abrir Integrações) da entrada.
- **Clique em Atividade** na extremidade direita da barra de estado para abrir o Registo
  de atividade. Clique num separador para mudar, clique na caixa de um nível (ou em
  **Tudo** / **Nada**) para mostrar ou ocultar essas linhas, clique numa entrada de
  Operações Git para abrir os detalhes, clique em **Limpar registo** no separador Aplicação (pede confirmação primeiro). Desloque as linhas longas para o lado com `Shift` +
  roda do rato ou com a barra de deslocação horizontal. Veja
  [O Registo de atividade](troubleshooting.md#o-registo-de-atividade).
- **Clique em `[x]`** no indicador de tarefas à direita da barra de estado para cancelar a tarefa em curso, **clique na etiqueta** para abrir o painel de tarefas. O botão **Atividade** no canto inferior direito do separador Início (ou `Ctrl+N Shift+G`) abre o Registo de atividade em Todos os repositórios (a sua caixa Âmbito, uma lista suspensa sob os separadores, escolhe um repositório ou uma área de trabalho).

## Redimensionar

- **Arraste um divisor** entre a barra lateral Refs, o Log e o painel direito. O teclado move-os um passo de cada vez: `Ctrl+N Left` / `Ctrl+N Right` movem o divisor Refs | Log, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` o divisor Log | painel direito (o divisor vai para o lado que a seta indica).
  Arraste-o para além da largura mínima para recolher esse painel. Os equivalentes pelo teclado são `Ctrl+N Left` / `Ctrl+N Right` (o divisor entre o painel Refs e o Log) e `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` (o divisor entre o Log e o painel direito), um passo cada, com o divisor a ir para o lado que a seta indica; na paleta são `Navegação: Alargar o painel Refs`, `... Estreitar o painel Refs`, `... Alargar o painel Status` e `... Estreitar o painel Status`, e `Navegação: recolher/expandir o painel Refs` / `... recolher/expandir o painel Status`.
- **Arraste a margem inferior** de uma secção de Refs para mudar a sua altura.
- **Arraste a margem inferior** do bloco da mensagem do commit no painel
  direito.
- **Arraste a margem entre dois cabeçalhos do Log** (por predefinição o espaço, a partir do cabeçalho ou de qualquer ponto da tabela; Definições > Log e grafo > Redimensionamento das colunas) para redimensionar uma
  coluna (pelo teclado: `Shift+C` para escolher uma coluna, depois `]` / `[`).
- **Duplo clique na margem entre dois cabeçalhos do Log** para ajustar a coluna à sua esquerda ao conteúdo (pelo teclado: `Shift+C` para escolher uma coluna, depois `=`; `+` ajusta todas). A última coluna tem uma pega na sua última célula: arraste-a para redimensionar essa coluna, com a coluna Mensagem a absorver a diferença.
- **Deslocar o Log lateralmente** (a disposição de colunas `scroll`; Definições > Log e grafo > Disposição das colunas, ou a caixa `Deslocamento lateral` do seletor): a roda horizontal ou `Shift`+roda sobre a tabela, a barra na margem inferior (arraste o cursor, prima a pista livre, clique nas setas das pontas) ou `Shift+Esquerda` / `Shift+Direita`. Na disposição `fit` predefinida as colunas preenchem o painel e não há nada para deslocar.
- **Desloque um painel de lista para o lado** quando uma linha é mais larga do que o painel: uma secção de Refs (um nome longo de branch, tag ou stash), as listas Unstaged e Staged, os ficheiros alterados do commit (ou de uma comparação) e as linhas de informação dos detalhes do commit (id, autor, data de autoria, contagens). A roda horizontal ou `Shift`+roda sobre o painel desloca-o; uma barra aparece na margem inferior do painel só enquanto algo transborda (arraste o cursor, prima a pista livre, clique nas setas das pontas), por isso nunca ocupa uma linha. Pelo teclado, `Alt+Esquerda` / `Alt+Direita` deslocam o painel que o tem (o Log desloca os detalhes do commit; dentro dos ficheiros alterados, os ficheiros); a paleta tem `Ver: deslocar o painel para a esquerda/direita`. A linha inteira desloca-se (indentação, marcador de estado, seta e nome juntos); o título do painel, a sua margem, as barras de deslocação e o botão Stage/Unstage de uma linha Status sob o ponteiro ficam no sítio. Um nome já não é cortado com reticências: a sua cauda está a um deslocamento de distância (uma branch com destinos de push extra mantém o seu emblema `⇉` à vista, e é o nome que cede). O deslocamento é por painel, pára na linha mais larga à vista e é esquecido ao mudar de repositório. O bloco da mensagem do commit quebra o texto, por isso não tem nada para deslocar. Enquanto uma secção está deslocada para o lado, a sua margem inferior leva a barra, por isso redimensione essa secção pelos seus comandos de teclado ou a partir de uma secção que caiba.
- **Arraste um cabeçalho do Log** para mover a coluna: uma sombra do seu nome segue o ponteiro, o cabeçalho que deixou fica esbatido e uma barra de destaque marca o lugar onde ela ficaria. As colunas movem-se ao largar; `Esc` cancela e o ponteiro mostra a mão aberta enquanto a leva (pelo teclado: `Shift+C` para escolher uma coluna, depois `<` / `>`).

O GitPoulpi memoriza todos os tamanhos. Veja
[Os tamanhos são memorizados](interface-tour.md#os-tamanhos-são-memorizados).

## Barra de título, linha de separadores e barra de ferramentas

- **Clique em <img src="../../../assets/logo/gitpoulpi.png" alt="Ícone do GitPoulpi" height="16">** na barra de título para abrir **Acerca do GitPoulpi**. Em Acerca de,
  as pupilas do polvo seguem o ponteiro; clique no logótipo para o fazer rodar para a cor
  seguinte, clique com o botão direito nele para ver a lista de cores, clique numa ligação
  para a abrir.
- **Clique em Abrir em ▾** na barra de título para ver a lista de programas.
  Veja [Abrir em](open-in.md).
- **Clique na versão** (`v1.0.0`, ou `vX.Y.Z disponível`) na barra de título para abrir o separador Notas de versão.
- **Clique na lupa ou em `Pesquisa global`** na barra de título, logo à esquerda da roda dentada, para a
  [pesquisa global](search.md).
- **Clique na roda dentada** na barra de título, logo à esquerda do ×, para as Definições.
- **Clique em ×** na extremidade direita da barra de título para sair.
- **Clique num separador** para mudar para ele. **Clique no seu `x`** (mostrado no separador
  ativo e no que está sob o ponteiro) para o fechar. **Arraste um separador** para o lado para
  o reordenar. **Clique com o botão direito num separador** para ver o menu do separador.
- **Clique com o botão do meio num separador** para o fechar, como o seu `x`.
- **Clique em `‹` ou `›`** nas pontas dos separadores (aparecem quando não cabem todos) para deslocar a linha de um separador; mantenha o botão premido para continuar a deslocar, ou use a roda sobre a linha. Arrastar um separador para uma seta também desloca a linha. A linha desloca-se também sozinha para mostrar o separador atual sempre que ele muda.
- **Clique no `+`** a seguir ao último separador para um menu: Novo separador (`Alt+T`) e, enquanto os respetivos separadores estão fechados, Radar e Áreas de trabalho (`Alt+J`).
- **Clique no ícone da casa** para o separador Início.
- **Clique no separador Radar ou Áreas de trabalho** para o abrir; o seu `x`, um clique com o botão do meio ou `Ctrl+W` fecham-no. **Clique na etiqueta da área de trabalho** (`Alt+W`) para mudar de área de trabalho.
- **Clique em Sair e cd aqui** (mostrado à direita dos separadores quando iniciou o GitPoulpi através do wrapper de shell `gp`) para sair e deixar a shell na pasta do separador ativo. Veja [Seguir a pasta do separador ativo](following-the-tab-folder.md).
- **Clique num botão da barra de ferramentas** para o executar. **Clique em
  `▾`** junto a Pull, Push ou LFS para ver a sua lista. **Clique em `⋯`** para
  os botões que não couberam.
- **Clique na caixa combinada de perfil** na extremidade direita da barra de
  ferramentas para escolher a identidade de commit deste repositório.

## Separador Início

- **Clique** numa linha para a selecionar. **Clique duplo** para abrir o
  repositório.
- **Clique na estrela** para adicionar ou remover um favorito.
- **Clique em Abrir, Ir, Fechar ou Remover** numa linha, ou em **`⋮`** para o
  menu da linha. **Clique com o botão direito** numa linha para o mesmo menu.
- **Clique no título de uma secção** para a recolher.
- **Clique em Procurar, Clonar, Init, Recolher tudo ou Expandir tudo** no topo,
  e em **Fechar todos os separadores** ou **Remover todos** no título de uma
  secção.

No seletor de pastas **Procurar**: clique num local ou numa pasta para ir para
lá, faça clique duplo numa pasta para a abrir, clique no campo do caminho para
escrever um caminho. Marque
**Mostrar pastas vizinhas** (`Alt+S`, memorizado) e a árvore da esquerda lista também as
outras pastas de cada nível aberto: as pastas do caminho aberto mostram `▾`,
as outras `▸`, e um clique numa delas vai para lá. Em **Clonar** e **Init**: clique num campo para escrever
nele, e clique num botão ou numa caixa de verificação para a usar.

## Barra lateral Refs

- **Clique** numa linha para a selecionar. **Clique duplo** numa branch local
  ou numa tag para fazer checkout. **Clique duplo** num worktree ou num
  submódulo para o abrir num separador. O **+** no cabeçalho de WORKTREES cria
  um worktree; clique direito num worktree para Abrir em novo separador, Abrir
  em, Merge e Remover ([Worktrees](worktrees.md)).
- **Passe o ponteiro** por uma linha para mostrar o ícone à sua esquerda.
  **Clique** nele para ocultar a ref no Log, ou para a mostrar de novo. Enquanto
  o **Solo** está ligado, o mesmo sítio põe a ref em solo. Numa linha de pasta,
  o ícone atua sobre todas as refs da pasta.
- **Clique em Parar solo** ou **Mostrar tudo** no título da barra lateral para
  limpar todos os solos ou todas as refs ocultas.
- **Clique com o botão direito** numa linha para ver o seu menu: checkout,
  merge, fast-forward, rebase, criar um worktree, renomear, eliminar, ocultar,
  solo e mais. A linha própria de um remoto tem **Fetch**, **Editar**,
  **Remover** e outras ações de remoto. Um stash tem **Pop**, **Apply** e
  **Drop**. Uma linha a cinzento diz porque não pode ser executada.
- **Clique em `+`** no título REMOTO para adicionar um remoto.
- **Clique numa linha de pasta** para a recolher ou expandir.

### Pull requests e issues

| Gesto | Ação |
|---|---|
| Clicar numa linha | Selecioná-la. |
| Clique duplo numa linha | Abri-la no navegador. |
| Clique com o botão direito numa linha | O seu menu: abrir, copiar o URL ou o número, fazer checkout da branch de origem, criar uma branch a partir de uma issue. |
| Clicar em `+` no título da secção | Abrir o formulário **Criar pull request** ou o formulário **Nova issue**. |
| Clicar na linha de filtro (`Abertos ▾`) | Escolher que pedidos ou issues listar. |
| Clicar no título de um grupo | Recolher ou expandir o grupo. |
| Clicar na linha do fornecedor ou do repositório (ISSUES) | Escolher a origem das issues. **Nenhum (desvincular este repositório)** remove-a. |
| Clicar na linha da consulta (ISSUES) | Escrever uma consulta JQL ou WIQL (Jira, Azure DevOps), ou escolher listas do Trello. |
| Clicar no chip da barra de ferramentas (`GH #12`, `GL !53`, ...) | Abrir o pull request no navegador. Clique com o botão direito nele para **Abrir no navegador**, **Copiar URL**, **Atualizar** e **Mudar de perfil**. |
| Clicar no chip esbatido `+ PR` | Iniciar um pull request para a branch enviada. |

Veja [Alojamento e gestores de issues](hosting-integration.md).

## O Log

- **Clique** num commit para o selecionar. Os seus detalhes aparecem no painel
  direito.
- **Ctrl+clique** acrescenta um commit à seleção ou retira-o. **Shift+clique**
  seleciona um intervalo (**Alt+clique** nos terminais que ficam com o Shift+clique para a
  sua própria seleção de texto, como o kitty e o Windows Terminal). Clique com o botão
  direito num commit selecionado para fazer cherry-pick de todos. A seleção pode incluir a
  linha WIP e os stashes; duas ou mais linhas transformam o painel direito numa comparação,
  veja [Comparar commits](log-and-history.md#comparar-commits).
- **Clique num cartão** do painel de comparação para levar o cursor do Log até essa linha;
  **clique em Trocar** para inverter as extremidades De e Até. Um clique simples numa linha
  sai da comparação.
- **Clique na linha WIP** para as alterações em curso. **Clique no seu texto
  `// WIP`** para escrever aí o resumo do commit.
- **Passe o ponteiro** por uma célula para ver o texto completo: a mensagem, o
  autor e o e-mail, a data completa, o hash completo.
- **Passe o ponteiro por um emblema `+N`** para ver as outras refs do commit.
  Entre no popup para clicar numa ref aí. Clique duplo numa ref do popup para
  fazer checkout dela.
- **Clique duplo numa pílula de ref** para fazer checkout dela.
- **Clique com o botão direito num commit** para ver o menu do commit. Tem um
  submenu por cada ref do commit.
- **Clique com o botão direito numa pílula de ref** para ver um menu com as
  ações da ref e as ações do commit.
- **Clique no funil do cabeçalho Autor** (ou `Shift+A` no Log, ou `Log: filtrar por autor...` na paleta) para escolher os commits de que autores ficam acesos; os restantes esbatem como numa pesquisa. `Enter` ou um clique numa linha alterna um autor, a linha **Limpar o filtro** (✕) retira o filtro, `Esc` esvazia o texto de pesquisa e depois fecha o popup, mantendo o filtro. `Esc` no Log larga um filtro aceso. **Clique com o botão direito no funil aceso** para o limpar (a paleta: `Log: limpar o filtro de autor`).
- **Clique com o botão direito no cabeçalho** para mostrar ou ocultar colunas: uma lista de verificação (`[x]` / `[ ]`) com um ✕ no título; clique numa linha ou use as setas e `Espaço` / `Enter` e feche-a com o ✕, `Esc` ou um clique fora (a paleta: `Ver: Log: escolher as colunas a mostrar`). Abaixo das caixas: `Ajustar <coluna> ao conteúdo` (a coluna clicada), `Ajustar todas as colunas ao conteúdo` e `Repor as larguras das colunas`. O equivalente de teclado do clique direito: `Shift+C` para escolher uma coluna, depois `.` (ou `Shift+F10`) abre o mesmo menu nela.
- **As colunas do Radar** funcionam da mesma forma: arraste o separador a seguir à coluna do repositório, `#`, autor ou idade para a dimensionar, **faça duplo clique nele** para ajustar a coluna à sua entrada mais larga (`C` escolhe uma coluna, depois `=`; `+` ajusta todas as colunas), e **clique com o botão direito no título de uma coluna** (ou `C`, depois `.`) para `Ajustar <coluna> ao conteúdo` e `Ajustar todas as colunas ao conteúdo`.
- **Clique em `↑`, `↓` ou `✕`** na barra de pesquisa para ir para a
  correspondência anterior ou seguinte, ou para fechar a barra.
- **Clique em `#12`, `ABC-12` ou `AB#12`** numa mensagem de commit, no painel
  direito, para abrir essa issue ou esse pull request no navegador.

### Secções do painel esquerdo

**As pastas** (branches e tags com nomes `a/b/c`, os ficheiros do painel direito em
vista de árvore) dobram-se e desdobram-se de uma só vez: **clique com o botão direito no cabeçalho ou no
espaço vazio de uma secção** que tenha pastas para **Expandir tudo** / **Recolher tudo**
(e **Mostrar/ocultar secções...**, que leva à lista de verificação abaixo), e
**clique com o botão direito numa linha de pasta** para **Expandir todas as subpastas** / **Recolher todas as
subpastas** (a pasta e tudo o que está debaixo dela). Teclas, no painel Refs, nas listas
Não preparados e Preparados e nos ficheiros alterados de um commit: `=` expande tudo,
`-` recolhe tudo, `*` (ou `Shift+Right`) expande a pasta sob o cursor
e tudo o que está abaixo, `_` (ou `Shift+Left`) recolhe-a. Paleta: *Refs: expandir todas as pastas*, *Refs: recolher todas as pastas*,
*Refs: expandir a pasta selecionada e as suas subpastas*, *Refs: recolher a pasta selecionada e as suas
subpastas* e as quatro gémeas *Ficheiros:* para a lista de ficheiros com o foco. Enquanto um filtro de nomes
está ativo (o filtro de Refs, o *Filtrar ficheiros* de um commit), todas as pastas
que contêm uma correspondência aparecem abertas, seja o que for que tivesse recolhido, e as suas próprias dobras
não são alteradas nem guardadas: limpe o filtro e elas voltam. Uma dobra que faça
durante o filtro atua sobre o que está à vista e é esquecida com o filtro.

**Clique com o botão direito no cabeçalho de uma secção** (uma secção sem pastas, ou a parte vazia do painel esquerdo) para uma
lista de verificação das suas secções: Local, Remoto, Worktrees, Stashes, Tags, Pull
requests, Issues e Submódulos. Um clique desmarca ou marca uma secção; o menu
fica aberto. Uma secção oculta não é desenhada e o teclado salta-a. A última
secção visível não pode ser desmarcada. A escolha vale por repositório e mantém-se entre reinícios; um repositório sem escolha própria segue a predefinição (o `ui.sidebar_sections_hidden` global). *Usar em todos os repositórios* torna a escolha atual a predefinição, e *Repor predefinição* descarta a escolha própria deste repositório (paleta: *Painel esquerdo: usar estas secções em todos os repositórios*). Paleta: *Painel esquerdo: mostrar/ocultar
secções...*, um comando *mostrar/ocultar* por secção, *repor as secções predefinidas*.

### Arrastar refs

Prima uma branch, uma branch remota ou uma tag na barra lateral Refs, ou uma pílula de ref
no Log, e arraste-a para outra ref. A barra de estado diz o que está a transportar. Um alvo
que pode receber o que larga acende-se. Largue para obter um menu. `Esc` cancela o
arrastamento. O Log fica onde está enquanto transporta: premir uma ref só desloca o Log para
o seu commit quando se revela um clique (o salto acontece quando o botão sobe), por isso pode
deslocar-se primeiro até ao alvo e só depois pegar na ref.

| Arrastar | Menu |
|---|---|
| Branch local X para a branch local Y | **Merge de X em Y**, **Rebase de X sobre Y**, **Rebase interativo de X sobre Y...**, **Fast-forward de Y até X** |
| Branch local para uma branch remota | **Push de X** para esse remoto |
| Branch remota para a branch local Y | **Merge**, **Rebase de Y sobre ela**, **Fast-forward de Y até ela**, e **Pull** quando é o upstream de Y |
| Tag T para qualquer branch, branch remota ou tag | **Mover a tag T para aqui**; marcado *(não é um fast-forward: T deixaria de ser um antecessor)* quando o destino não descende do commit da tag (a escolha é sua); a cinzento *já está lá* quando já está nesse commit |
| Tag T para a branch local Y | também **Merge da tag T em Y**, e abaixo de uma linha o mais raro **Merge de Y na tag T e mover a tag** (um commit de merge do commit da tag e de Y, destacado, e depois a tag move-se para ele) |

Quando a ação altera uma branch que não está em checkout, o GitPoulpi faz
primeiro checkout dela. Se o git recusar o checkout, nada muda. Só as tags podem
ser largadas numa tag, e uma branch não pode ser largada numa.

Mover uma tag mantém o seu tipo: uma tag leve continua leve, uma tag anotada é recriada
anotada com a mensagem e a linha do autor da tag transportadas sem alteração (uma tag
assinada é recusada, a sua assinatura deixaria de corresponder). Só a sua tag local se
move: um remoto que tem a tag mantém a posição antiga, a tag passa a *differs* e a barra
de estado di-lo; faça o push você mesmo quando o quiser. O movimento e o merge-e-mover
são cada um um passo de desfazer (desfazer repõe o objeto de tag antigo). Um merge numa
tag que dê conflito não altera nada. Paleta: `Refs: mover a tag selecionada para o commit
selecionado...` move a tag do painel Refs para o commit selecionado no Log (pergunta
primeiro quando não é um fast-forward); todos os outros destinos passam por marcar e
largar, abaixo.

Teclado: «marcar e depois largar» faz o que um arrasto faz, para qualquer par de
refs. No painel Refs, prima `m` numa branch (paleta: `Refs: Marcar como origem do largar`).
A ref mostra um `◆` e fica sublinhada, também nas suas etiquetas no Log, e a barra de
estado diz para onde ir. Selecione outra ref, em qualquer secção, e prima `Shift+M`
(paleta: `Refs: Largar aqui a ref marcada...`): aparece o mesmo menu que um arrasto
abre, e as suas linhas correm exatamente como depois de um arrasto. A marca
gasta-se quando o menu abre; `Esc`, ou `m` na ref marcada, limpa-a antes. O comando
da paleta `Refs: largar a ref selecionada na branch em checkout...` abre o menu para
a ref selecionada e a branch atual.

## O painel direito

### Alterações em curso

- **Clique** num ficheiro para abrir o seu diff. **Clique outra vez no mesmo
  ficheiro** para voltar ao Log. Se o ficheiro estiver aberto no editor com
  alterações por guardar, o GitPoulpi pergunta primeiro.
- **Passe o ponteiro** por uma linha para mostrar o seu botão **Stage
  ficheiro** ou **Unstage ficheiro**. Uma linha de pasta tem **Stage pasta** ou
  **Unstage pasta**.
- **Clique em Stage de todas as alterações** ou **Unstage tudo** no título de
  uma secção.
- **Clique no título de uma secção** (`Ficheiros unstaged`, `Ficheiros staged`)
  para a recolher.
- **Clique em Caminho ou Árvore** para escolher uma lista plana ou pastas.
  **Clique em Recolher tudo** ou **Expandir tudo** para recolher todas as
  pastas.
- **Clique em `✗`** à esquerda do título para descartar todas as alterações
  (pergunta primeiro). **Clique em `✦`** à direita para explicar as alterações
  com IA.
- **Clique com o botão direito num ficheiro** para ver o seu menu: stage,
  descartar, ignorar, LFS, stash do ficheiro, histórico, blame, abrir num
  programa externo, copiar o caminho, criar um patch, editar, eliminar. Um
  ficheiro em conflito começa pelas linhas **Resolver com ...**.
- **Clique com o botão direito numa linha de pasta** para ver o menu da pasta:
  stage, descartar, ignorar, stash, criar um patch, abrir a pasta, criar um
  ficheiro nela.
- **Clique com o botão direito na parte vazia** de Ficheiros unstaged para
  **Criar ficheiro**.
- **Ignorar**, **Tratar como binário** e **Tratar como texto** (menus de ficheiro e pasta) abrem primeiro uma lista dos ficheiros `.gitignore` ou `.gitattributes` onde a regra pode ir. Clique numa linha e prima `Enter` (ou use as setas); `Esc` ou um clique fora cancela sem escrever nada.
- **Ctrl+clique** numa linha de ficheiro ou de pasta adiciona-a à seleção ou remove-a, **Shift+clique** (**Alt+clique**) seleciona um intervalo; o mesmo nos ficheiros alterados de um commit e em Ver todos os ficheiros. Um clique simples recomeça. **Clique com o botão direito numa linha selecionada**: o menu de todos os ficheiros selecionados.
- **Clique com o botão direito na parte vazia** de uma lista oferece também **Abrir a pasta do repositório**; Unstaged acrescenta **Stage tudo** e **Descartar todas as alterações...**, Staged acrescenta **Unstage tudo**.
- **Converter** (menus de ficheiro, pasta e seleção) reescreve a codificação, os fins de linha ou a indentação dos ficheiros no disco; um `Ctrl+Z` desfaz o lote.

### O formulário de commit

- **Clique** no Resumo ou na Descrição para escrever nele.
- **Clique em Amend do commit anterior** para ligar ou desligar o amend.
- **Clique em Commit das alterações em N ficheiros** para fazer commit. Quando o botão está a
  cinzento, um clique diz porquê (por exemplo *Faça stage das alterações para o
  commit*).
- **Clique em `✦`** à direita do Resumo para escrever a mensagem com IA.
- **Clique em `#▾`** no fim do Resumo para ver a lista de issues. Clique numa
  para inserir a sua referência.
- **Clique em Compor commits com IA** para abrir o compositor de commits.

### Detalhes do commit

- **Clique** num ficheiro alterado para abrir o seu diff. **Clique outra vez**
  para voltar ao Log.
- **Arraste** a margem inferior da mensagem para a redimensionar. A roda sobre a
  mensagem desloca uma descrição comprida.
- **Clique com o botão direito** no resumo, na descrição ou no bloco do autor para *Alterar o
  resumo...*, *Alterar a descrição...* ou *Alterar o autor...*; **clique duplo** num deles (ou na
  data de autoria) para abrir o diálogo Alterar completo com o cursor nesse campo. Os
  equivalentes de teclado são `Ctrl+G Shift+E` (todos os campos) e `Ctrl+G Shift+A` (o autor); o
  resumo e a descrição estão na paleta.
- **Clique com o botão direito** num ficheiro alterado, numa linha de pasta, na parte vazia da lista ou numa seleção para o seu menu (abrir, histórico, blame, abrir num programa, copiar caminhos, ignorar, tratar como binário ou texto, converter, restaurar do commit, eliminar). **Ctrl+clique** e **Shift+clique** selecionam vários ficheiros.

Veja [Stage e commit](staging-and-committing.md).

## O diff e a vista de ficheiro

- **Clique num botão** da barra por cima do diff: Ficheiro, Diff, Blame,
  o seletor de coluna `Histórico │ Todos`, Editar, `↑` `↓`, Inline, Dividido, Hunks, EB, Quebra,
  Mapa, Codificação. Um botão que está ligado fica realçado. Um clique em `Histórico` ou `Todos`
  mostra essa lista na coluna de commits; um clique no segmento já selecionado oculta-a (veja
  [A coluna de commits](interface-tour.md#a-coluna-de-commits)).
- **Clique em `✕`** no título para fechar o ficheiro.
- **Passe o ponteiro por um hunk** para mostrar **Stage hunk** ou **Unstage
  hunk**, e **Descartar hunk**.
- **Passe o ponteiro por uma linha alterada** para mostrar os seus próprios
  botões **+ Stage**, **− Unstage** e **✗ Descartar** na extremidade direita da
  linha.
- **Clique com o botão direito numa linha** para ver o menu do hunk: stage,
  unstage ou descartar o hunk ou as linhas selecionadas, histórico do ficheiro,
  blame, copiar.
- **Clique no minimapa** para mover a vista para essa linha. Mantenha o botão
  premido e mova para deslocar.

Todo o descartar pergunta primeiro. Quando um descartar elimina ficheiros não acompanhados, as pastas que ficam completamente vazias saem com eles (nunca uma que ainda tenha um ficheiro acompanhado, ignorado ou oculto, nem uma que já estava vazia); Anular devolve os ficheiros com as suas pastas.

### Blame e histórico do ficheiro

- **Blame.** Deixe o ponteiro sobre uma anotação para ver o cartão do hunk; um clique seleciona um hunk, um clique com o botão direito abre o seu menu. Arraste a margem da coluna de anotações para a redimensionar (guardado). Clique ou arraste no minimapa para deslocar.
- **Histórico do ficheiro.** Um clique seleciona um commit; `Shift+clique` seleciona um intervalo para comparar; um clique duplo mostra o commit no Log; um clique com o botão direito abre o menu da linha; arraste a margem entre a lista e o diff para a redimensionar; o ✕ da lista oculta-a (clique na barra `›` para a recuperar); clique em `Ficheiro` / `Diff` / `Blame` na barra para trocar o que o lado direito mostra, e no seletor `Histórico │ Todos` para escolher a lista da coluna. A roda desloca a metade sob o ponteiro. Cada um tem o seu equivalente de teclado ([Blame](log-and-history.md#blame), [Histórico do ficheiro](log-and-history.md#histórico-do-ficheiro)).

### A coluna de commits (lista Todos)

- **Passe o ponteiro por uma linha** para a dica do Log (a mensagem inteira, o nome e o e-mail do autor, a data completa, conforme a parte sob o ponteiro).
- **Clique com o botão direito numa linha** para o seu menu: mostrá-la no Log, o diff do ficheiro nesse commit, o ficheiro nesse commit, o seu blame e depois o menu do Log para o commit. A tecla de menu faz o mesmo pelo teclado.
- **Arraste um limite do cabeçalho** (`ui.column_resize`: por predefinição o separador de uma célula entre duas partes, marcado por um `│` esbatido no cabeçalho e desenhado como uma linha de destaque ao longo da coluna enquanto o ponteiro está nele, e um arrastamento a partir de qualquer ponto dela redimensiona; com `header`, a primeira letra do nome seguinte, acesa enquanto o ponteiro está nela) para dimensionar a parte à sua esquerda: faixas, mensagem, autor, data, refs, alterações ou SHA. A célula logo a seguir à última parte é o limite dessa última parte.
- **Duplo clique num limite do cabeçalho** para ajustar a parte à sua esquerda ao conteúdo, como fazem os limites do Log: a largura que mostra a sua linha mais larga por inteiro, nunca maior do que a coluna (o teclado: clique numa linha ou `Ctrl+N` para dar as teclas à coluna, `Tab` para escolher a parte, depois `=`; `+` ajusta todas as partes; a paleta tem `Ajustar a coluna ao conteúdo` e `Ajustar todas as colunas ao conteúdo`). A mensagem fica com o que as outras partes deixam.
- **Arraste um nome do cabeçalho** para cima de outro para mover a parte para lá: a mesma sombra, origem esbatida e barra de destaque do cabeçalho do Log, e a parte move-se ao largar (`Esc` cancela; as teclas `<` / `>`, depois de `Tab`, fazem o mesmo, um lugar de cada vez).
- **Clique com o botão direito no cabeçalho** para escolher as partes mostradas (Ref, Grafo, Autor, Data, Alterações, SHA; a Mensagem está sempre visível). A lista de verificação tem um ✕ no título. Abaixo dela: `Ajustar <parte> ao conteúdo` (a parte clicada) e `Ajustar todas as colunas ao conteúdo`. A paleta tem `Escolher as colunas` (o menu a partir do teclado, na parte que `Tab` escolheu) e uma alternância por parte.
- **Shift+roda** ou uma roda horizontal desloca as linhas para os lados quando são mais largas do que a coluna.
- **Arraste a borda direita** da coluna para a redimensionar. **Clique na barra `›`** (ou no ✕ da coluna, ou **clique no segmento selecionado** do seletor `Histórico │ Todos` da barra) para a recuperar ou ocultar.

### Selecionar e copiar texto

Pode selecionar e copiar o texto do diff, da vista de ficheiro, do Blame
e do Histórico do ficheiro.

- **Prima e arraste** para selecionar texto. A seleção fica no mesmo texto
  quando desloca.
- **Clique duplo** para selecionar uma palavra.
- **Um clique simples** não seleciona texto. Move o cursor ou seleciona a
  linha, e limpa a seleção de texto.
- **`y` ou `Ctrl+C`** copia a seleção. Sem seleção, copiam a linha atual.
- **Clique com o botão direito** no diff para **Copiar seleção**, **Copiar
  linha** e **Copiar hunk**.

O GitPoulpi copia apenas o texto do ficheiro. Os números de linha, as marcas `+`
e `-`, os cabeçalhos de hunk e as colunas do Blame não são copiados. Em
Dividido, uma seleção fica no lado onde começou o arrastamento.

No diff, um arrastamento seleciona também as linhas que atravessa, como `v`.
`Enter` faz então stage dessas linhas. `Esc` limpa primeiro a seleção de texto,
depois a seleção de linhas, e depois fecha o diff.

Selecionar texto também tinge todos os outros sítios onde esse texto aparece no
diff e no editor.

## Campos de texto

Aplica-se a todos os campos de texto: filtros, a barra de pesquisa, a paleta, o
Resumo e a Descrição, campos de diálogos e valores das Definições.

- **Clique** para pôr o cursor aí.
- **Prima e arraste** para selecionar texto.
- **Clique duplo** para selecionar uma palavra.
- Escreva para substituir a seleção. `Backspace` ou `Delete` remove-a.

As teclas dos campos de texto estão em [Atalhos de teclado](keybindings.md).

## O editor

- **Clique** para pôr o cursor aí. **Shift+clique** alarga a seleção.
- **Arraste** para selecionar. Arraste para além da margem de cima ou de baixo
  para deslocar.
- **Clique duplo** para selecionar uma palavra.
- **A roda** desloca sem mover o cursor.
- **Clique em Guardar, Desfazer, Refazer ou Pesquisar** na barra. No modo de
  pesquisa, **`↑`** e **`↓`** vão para a correspondência anterior e seguinte.
- **Clique em `✕`** para fechar. O GitPoulpi pergunta primeiro quando há
  alterações por guardar.

## Imagens

Veja [Imagens](image-view.md) para o rato na vista de imagem.

## Mover e redimensionar janelas

As janelas podem ser movidas, e as janelas maiores redimensionadas:

- **Arraste a barra de título** para mover uma janela (a cruz de fechar mantém o seu clique). A paleta de comandos e os seletores não têm barra de título própria: arraste a margem superior.
- **Arraste a margem esquerda, direita ou inferior**, ou um canto inferior (a pega `◢`), para redimensionar uma janela. A janela fica no ecrã e nunca fica menor do que o seu mínimo (60 x 16 no Histórico de desfazer, menor nas outras). Uma janela redimensionada aproveita o espaço: as áreas de texto e as listas crescem, o texto quebra na nova largura.
- O arrasto continua quando o ponteiro sai da janela, e largar fora nunca a fecha.
- **Arraste os divisores** dentro da vista Reflog / Trabalho perdido do Histórico de desfazer; faça duplo clique num para o repor.
- Os equivalentes de teclado são `Ctrl+N` e depois `m` (as setas movem, `Shift`+setas redimensionam, `0` repõe, `Enter` ou `Esc` saem) e `Alt`+setas para os divisores; a paleta tem `Janela: mover ou redimensionar` e `Janela: repor posição e tamanho`. Atuam sobre a janela de cima. `Ctrl+N` funciona da mesma forma sobre qualquer superfície: mostra um selo nos controlos da janela, do diálogo ou do popup da frente (campos, listas, separadores, botões e a cruz de fechar), a tecla seguinte prime esse controlo, `m` inicia o modo de arranjo e `Esc` cancela; a linha de estado indica `<selo>`, `m=arranjar` e `Esc`. Isto vale também para a paleta de comandos, os seletores, a pesquisa de workspaces e a sobreposição de ajuda, onde `Ctrl+N` já não desce na lista. `Alt+G` (`palette.arrange`) é mais uma via, válida sobre qualquer janela e popup, para arranjar a que está à frente sem o prefixo.
- **Uma única definição** (`ui.dialog_geometry`, Definições > Aspeto > Disposição) cobre todas as janelas: cada uma memoriza o seu próprio lugar e tamanho nesta sessão, em disco para o próximo arranque, ou nunca.

| Janela | Mover | Redimensionar |
|---|---|---|
| Histórico de desfazer / Reflog / Trabalho perdido, Registo de atividade, painel de tarefas, seletor de pastas | sim | sim |
| Paleta de comandos, seletores de separador / tema / idioma, pesquisa de workspaces | sim | sim (a lista cresce com a altura, as linhas com a largura) |
| Clone (a lista de repositórios), Pull request e Nova issue (a descrição), "O que é enviado" (o painel), Mermaid (a pré-visualização), opções de revisão (os campos, só a largura) | sim | sim |
| Editores de texto longo das Definições (modelos, hooks, prompts de IA), a caixa de IA do formulário de workspace | sim | sim (a área de texto) |
| Reescrita do histórico: os dois diálogos auxiliares de IA | sim | sim (a descrição, o painel enviado) |
| Acerca de, painel de atualização, verificação de segurança, conflitos previstos, "Acima do limite de IA", eliminar tag, ficheiros sensíveis | sim | não (o tamanho segue o texto) |
| Init, Worktree, Sparse checkout, diálogo de Flow, diálogos do fluxo de issues, formulário da mensagem de commit (reword), formulário de Workspace, seletores de etiquetas / marcos | sim | não |
| Confirmações (todos os "Tem a certeza?") | sim | não |
| Diálogos de Revisão e de Quadro: os seus pequenos diálogos (comentário, terminar, escolher, colunas) | sim | não |
| Popups da página de atalhos (confirmação de reposição, captura de tecla), o popup de resposta da IA | sim | não |
| Definições: os resultados e a confirmação da autenticação do repositório | sim | não |
| Reescrita do histórico: diálogos de purga, identidade e perguntas | sim | não |
| Popup de mover um ficheiro do compositor de commits, avisos de merge | sim | não |

Não se movem: os ecrãs de página inteira (Definições, Ajuda, as páginas de Revisão e de Quadro, que já enchem a janela), os menus de contexto e os menus pendentes (pendem do que os abriu), as dicas e os pequenos popups dentro de uma página (o formulário de remotes, o editor de divisão de commit).

O ponteiro mostra uma forma de mover sobre a barra de título e uma forma de redimensionar sobre as margens e os cantos (terminais com `OSC 22`).

## Forma do ponteiro do rato

Alguns terminais deixam o GitPoulpi mudar o ponteiro do rato: um ponteiro de
redimensionamento sobre os divisores, sobre uma fronteira de coluna do Log, do Radar e da coluna de commits (também ao arrastá-la), uma mão sobre os itens clicáveis. Isto
funciona no kitty, no foot, no Ghostty, no mintty e numa compilação do WezTerm com o PR #8092 do upstream (a sequência
`OSC 22`). É enviada só quando a forma muda, dentro do tmux só embrulhada para a passagem
(`allow-passthrough on`; o kitty e o foot mostram-na, o Ghostty não), e nunca no screen, zellij ou
herdr. O ponteiro predefinido é reposto quando o GitPoulpi termina. Noutros terminais (Rio, Alacritty, Tabby, iTerm2,
Terminal.app, Windows Terminal, xterm), o ponteiro não muda; veja
[Escolher um terminal](terminals.md#forma-do-ponteiro-do-rato-osc-22). O divisor ou o botão sob o ponteiro acende-se em todos os terminais.

O mesmo tipo de componente dá o mesmo ponteiro em toda a aplicação: os botões de uma revisão mostrada sobre o Radar (separadores, a cruz de fechar, os botões de popups) e os próprios botões de Ações do Radar mostram a mão como qualquer outro botão.

- **Os botões são botões a sério**, nas Definições (Integrações: Editar, Desligar,
  Remover, Procurar, Eliminar, os botões de excertos do editor de instruções...) como nas
  caixas de diálogo: rótulos com margem sobre um fundo preenchido que se ilumina sob o
  ponteiro, esbatidos quando desativados, nunca texto entre `[ ]`. Os únicos parênteses retos
  que restam são as marcas das caixas de verificação `[x]` / `[ ]`, que são caixas de
  verificação e têm a mão como qualquer outro controlo.
- **A mão a apontar** está nos controlos: botões, caixas de verificação e interruptores,
  caixas de combinação (nas Definições, nos painéis de issues e de pull requests, no Radar,
  nas caixas de diálogo), separadores e as suas cruzes, os botões `+`, ligações e referências
  (`#12`, `!45`), cabeçalhos de grupo e de pasta com a sua seta de expansão, pílulas de ref,
  barras de deslocação e os botões Atividade, tarefas e versão da barra de estado. Uma
  caixa de verificação ou um botão dentro de uma linha mantém a mão.
- **A seta** está nas linhas. Uma linha acende-se por inteiro quando o ponteiro está nela
  (a mesma luz em todas as listas) e não é um controlo por si só: as linhas dos
  painéis Refs, Worktrees, Stashes e Submódulos, o Log e a coluna de commits
  (só uma pílula de ref leva a mão), as margens de um diff, os ficheiros alterados,
  os itens de pull requests, de issues e do Radar, e as linhas das listas das
  caixas de diálogo e dos painéis (clonar, seletor de pastas, Workspaces, os cartões do quadro, o
  painel de tarefas, os seletores de responsável, etiqueta e milestone, passos do rebase, ficheiros
  e commits da revisão, escolhas de menus e de caixas de combinação).
- **O cursor de texto (I-beam)** está nas caixas de entrada, sobre a caixa toda: as caixas de
  pesquisa (pesquisa global, Definições, paleta de comandos, Início), os filtros (Refs,
  ficheiros, Radar, Log de atividade, a pesquisa do painel de issues), as linhas de texto, número e
  área das Definições (também enquanto as edita), os campos das caixas de diálogo, as
  caixas de comentário de revisão, o compositor de commits, o texto do editor de ficheiros e o
  resultado editável do editor de merge (os seus painéis só de leitura mantêm a seta, porque
  um arrasto aí não seleciona nada). Uma regra cobre também o texto só de leitura: onde quer
  que um arrasto do rato possa selecionar texto, o ponteiro é o I-beam. É o caso do código de
  um diff (a vista Diff, Inline e Dividido, e o painel de diff da revisão), da Vista de
  ficheiro e do código do Blame. Os seus números de linha, marcadores e cabeçalhos de hunk
  mantêm a seta, e um botão numa linha mantém a mão. O mesmo vale para a conversa de uma issue
  ou de um pedido (também a Visão geral da revisão e o seu bloco de rascunho), a mensagem dos
  detalhes do commit, as Notas de versão, o Log de atividade (os dois separadores), o popup de
  ajuda, a pré-visualização Markdown do editor de comentários e os comentários, notas
  pendentes e sugestões dentro do diff da revisão: um arrasto seleciona o texto (um clique
  simples mantém o seu significado: uma ligação abre, um botão atua, uma entrada do Log de
  atividade ou uma discussão abre ou dobra), `Ctrl+C` copia-o, `Esc` ou um clique larga-o. Uma
  seleção acompanha o seu texto quando se desloca. A mensagem do commit também é editável
  (um clique duplo emenda-a), e o bloco todo acende-se ligeiramente.
- **Nos detalhes do commit**, as linhas de informação são controlos: clique em `commit: <id>`
  para copiar o id curto, faça clique duplo na linha do autor ou da data para a emendar
  (a mão e uma marca leve assinalam-nas). A linha de estatísticas só informa e mantém a
  seta.

Os ponteiros de redimensionamento ficam sobre os divisores e as fronteiras de coluna, e a mão aberta
só aparece enquanto leva uma ref.

## Copiar para a área de transferência

As ações de cópia põem o texto na área de transferência do sistema da máquina onde o GitPoulpi
corre e, onde se sabe que o terminal a respeita, enviam-no também ao terminal (OSC 52). Por SSH,
ou sem ecrã, só se envia OSC 52: se o seu terminal não o suportar (o iTerm2 até a sua permissão
de área de transferência estar ligada, o Terminal.app, o GNU screen), nada é copiado e a linha de
estado diz que o texto só foi enviado ao terminal. Veja
[Escolher um terminal](terminals.md#área-de-transferência-ações-de-copiar).
