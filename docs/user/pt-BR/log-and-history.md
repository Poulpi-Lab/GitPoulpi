[English](../log-and-history.md) · [Čeština](../cs/log-and-history.md) · [Deutsch](../de/log-and-history.md) · [Español](../es/log-and-history.md) · [Français](../fr/log-and-history.md) · [Italiano](../it/log-and-history.md) · [日本語](../ja/log-and-history.md) · [한국어](../ko/log-and-history.md) · [Nederlands](../nl/log-and-history.md) · [Polski](../pl/log-and-history.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/log-and-history.md) · [Русский](../ru/log-and-history.md) · [Svenska](../sv/log-and-history.md) · [Türkçe](../tr/log-and-history.md) · [Українська](../uk/log-and-history.md) · [简体中文](../zh-CN/log-and-history.md)

# Trabalhando com o Log

O Log é o centro do GitPoulpi: todo o histórico como um grafo, com cada
branch, tag e stash nele. Esta página cobre como lê-lo, como encontrar coisas
nele e tudo o que você pode fazer com commits e refs. As áreas são nomeadas no
[tour pela interface](interface-tour.md); as teclas exatas estão em
[Atalhos de teclado](keybindings.md) e os gestos do mouse em
[Mouse e gestos](mouse-and-gestures.md).

![O Log com um commit de merge selecionado: seus detalhes e arquivos à direita](../images/log-graph.png)

## Lendo o grafo

Cada linha é um commit, do mais novo para o mais antigo. Um ponto fica na faixa do
commit; linhas ligam um commit a seus pais; um merge mostra duas linhas se
encontrando. As pílulas à esquerda nomeiam as branches e tags desse commit, na cor
da faixa, com uma linha fina até o ponto. Um ícone de laptop marca uma branch
local, um globo uma remota, e `✓` a branch em que você fez checkout. `+N` significa
que mais refs apontam para esse commit; passe o mouse sobre ele para listá-las, ou
use o menu abaixo.

A primeira linha são as **alterações em andamento** (um círculo tracejado: seu
trabalho não commitado, ligado ao commit em que ele se apoia). Os stashes aparecem
como pequenos quadrados em uma faixa ao lado do commit em que foram criados.

### Cores, tom da linha e avatares

As cores das faixas vêm do tema. *Configurações > Aparência > Tema de cores do grafo*
combina qualquer tema com qualquer paleta de faixas, e o seletor da paleta
*App: mudar tema de cores do grafo...* (`Ctrl+N Shift+Y`) pré-visualiza cada paleta
ao vivo; veja [Temas](themes.md). *Fundo das faixas do grafo* definido como **Tom da
linha** pinta uma faixa suave na cor da faixa atrás da coluna do grafo e uma barra
fina na sua borda. *Intensidade do tom da linha* (**Forte**, **Média** (padrão),
**Sutil**) define o quanto a faixa é esmaecida.

| Paleta de faixas Serie com tom da linha | Paleta de faixas Material com tom da linha |
|---|---|
| ![O grafo do Log no tema de cores Serie com tom da linha](../images/theme-serie.png) | ![O grafo do Log no tema de cores Material com tom da linha](../images/theme-material.png) |

Os *Avatares dos autores* (uma imagem no ponto do grafo ou na coluna Autor) vêm
desativados por padrão. Ativá-los envia um hash do e-mail de cada autor ao
Gravatar e consulta seu provedor de hospedagem. *Fontes de avatar* e *Avatar
alternativo*, na mesma página, escolhem de onde vêm as imagens e o que é desenhado
quando não há nenhuma.

Se o seu terminal não consegue mostrar imagens, o grafo é desenhado com caracteres.
*Renderizador do grafo* e *Protocolo de imagem* forçam um ou outro; veja
[Solução de problemas](troubleshooting.md).

O destaque da linha selecionada ou sob o mouse começa no círculo do commit. As
faixas à esquerda dele mantêm o fundo normal.

### Colunas

Clique com o botão direito no cabeçalho para escolher as colunas: **Ref**,
**Grafo**, **Mensagem** (sempre ativa), **Autor**, **Data**, **Alterações** e
**SHA**. A coluna opcional **Alterações** mostra, para cada commit,
`n arquivos +adicionadas -removidas`. Arraste a borda entre dois cabeçalhos para
redimensionar uma coluna; a escolha é lembrada, em células do terminal. Com o `ui.column_resize = "line"` padrão, a borda é o espaço de uma célula no fim de uma coluna: uma pequena marca fica nele na linha do cabeçalho e, enquanto o ponteiro está sobre o espaço (no cabeçalho ou em qualquer ponto da tabela), uma linha fina de destaque é desenhada em toda a altura da tabela e um arrasto a partir de qualquer ponto dela redimensiona (o ponteiro vira um ponteiro de redimensionamento nos terminais que suportam). Com `"header"` não há marca nem linha: o primeiro caractere da célula do cabeçalho depois da borda se acende ao passar o ponteiro e o arrasto começa nele, só na linha do cabeçalho (Configurações > Log e grafo > Redimensionamento das colunas). As linhas têm o mesmo aspecto nos dois modos: o espaço faz parte da coluna à esquerda dele, então a seleção e as cores das linhas o atravessam. Ref, Grafo e as
outras colunas mantêm a largura quando a janela é redimensionada ou maximizada: só a
**Mensagem** cresce ou encolhe (em uma janela muito estreita as outras colunas cedem
temporariamente, o Grafo primeiro, e voltam às suas larguras quando a janela volta a
crescer). Arraste o Grafo até a largura mínima, um ponto de commit, e o grafo fica compacto: todos os commits em uma única faixa, um sob o outro, sem linhas de branch nem de merge. Uma célula a mais e a árvore volta. Com as imagens dos autores nos círculos dos commits (Configurações > Aparência > Avatares: grafo), o círculo ocupa duas células, então a coluna para uma célula antes (três células) e nunca corta uma imagem.

Arrastar um cabeçalho mostra um fantasma do nome dele seguindo o ponteiro e uma barra de destaque onde a coluna cairia; a ordem muda ao soltar, e `Esc` cancela. O seletor aberto pelo clique direito é uma lista de verificação (`[x]` / `[ ]`) com um ✕ no título; ele também se abre pela paleta (`Exibir: Log: escolher as colunas a mostrar`), percorre-se com as setas e alterna-se com `Space` / `Enter`.

Pelo teclado, `Shift+C` (ou `Exibir: coluna do Log: escolher uma coluna para mover ou redimensionar` da paleta) escolhe uma coluna, e a célula do cabeçalho dela se acende. Depois `Tab` / `Shift+Tab` escolhem outra coluna, `<` / `>` movem a coluna um lugar para a esquerda ou para a direita entre as mostradas (o equivalente de teclado de arrastar o cabeçalho), `]` / `[` alargam ou estreitam (o equivalente de arrastar a borda; a última coluna move a borda esquerda), `0` devolve todas as larguras ao automático e `Esc` ou `Enter` saem. As setas continuam percorrendo os commits nesse meio-tempo. A ordem e as larguras são lembradas como as de um arraste, e são do próprio Log: a coluna de commits ao lado de um diff tem uma lista própria.

*Ajustar ao conteúdo.* Um **clique duplo** no limite de uma coluna ajusta a coluna à esquerda dele ao seu conteúdo visível mais largo e ao seu cabeçalho (caracteres largos e emojis contados pelas células que ocupam). Na disposição ajustada, o Log sempre preenche o painel, então uma coluna para no que o painel comporta depois que a **Mensagem** mantém a sua parte e cada outra coluna a sua largura mínima: a linha de status diz então `Autor precisa de 34 células, cabem 20.` O mesmo pelo teclado: com uma coluna escolhida (`Shift+C`, `Tab`), `=` a ajusta e `+` ajusta todas as colunas; a paleta tem `Exibir: coluna do Log: ajustar a coluna ao conteúdo` e `... ajustar todas as colunas ao conteúdo`; o seletor do botão direito lista `Ajustar <coluna> ao conteúdo` (a coluna sob o ponteiro), `Ajustar todas as colunas ao conteúdo` e `Redefinir as larguras das colunas` abaixo das caixas de seleção. A **Mensagem** é a coluna flexível e não tem largura própria para ajustar: ela ocupa o espaço que as outras deixam.

*Rolar para os lados.* Por padrão, todas as colunas cabem no painel. Na disposição **scroll** (a caixa `Rolar para os lados (as colunas mantêm a largura)` do seletor, Configurações > Log e grafo > Disposição das colunas, `Exibir: Log: alternar a rolagem para os lados das colunas` na paleta, ou `s` enquanto uma coluna está escolhida) as colunas mantêm suas larguras, **Mensagem** incluída (ela tem uma largura própria ali, e ajustá-la usa o seu resumo mais largo), nada é cedido quando a janela é estreita, e a tabela pode ser mais larga que o painel. Uma barra na borda inferior então a rola: arraste o polegar, pressione a trilha vazia para trazer o polegar até o ponteiro, ou use as setas das extremidades. Pelo teclado, `Shift+Left` / `Shift+Right` rolam quatro células (a paleta tem `Exibir: Log: rolar as colunas para a esquerda` / `direita`); a roda horizontal ou `Shift`+roda sobre a tabela fazem o mesmo, e escolher uma coluna (`Shift+C`, `Tab`) a traz para a vista. Uma tabela mais estreita que o painel deixa espaço em branco à direita, e a faixa de uma linha selecionada vai até a borda. Cada disposição guarda suas próprias larguras (`ui.log_column_widths` e `ui.log_column_widths_scroll`); o primeiro arrastar, tecla ou ajuste na disposição scroll grava cada coluna na largura que ela tem, então só a que você muda se move. Na disposição ajustada, um ajuste que não cabe no painel avisa e aponta para esta disposição.

*A alça da última coluna.* A última coluna também tem uma alça, na sua própria última célula (não na borda do painel, que continua redimensionando os painéis): arrastar a move conforme a distância que o ponteiro percorreu, com a **Mensagem** absorvendo a diferença. O **SHA** tem 8 células de largura por padrão (os 7 dígitos e essa célula da alça).

![O Log com a coluna Alterações visível: cada commit lista seu número de arquivos e de linhas adicionadas e removidas](../images/log-changes-column.png)

### Autores e `.mailmap`

Quando o repositório tem um `.mailmap` (o arquivo do git que dá um só nome e um só
e-mail a uma pessoa que fez commits com vários), a coluna **Autor** do Log, os
detalhes do commit, o Blame e o Histórico do arquivo mostram a identidade
**canônica**, como o `git log` faz. Os commits em si não são tocados. A
configuração `log.use_mailmap` (Configurações > Log e grafo, ativada por padrão)
a desliga para mostrar os nomes gravados. A busca por autor casa com o nome e o
e-mail como são exibidos, então uma busca pelo nome canônico encontra todos os
commits da pessoa. A aba **Identidades** do painel de reescrita do histórico
lista os nomes e e-mails do histórico e escreve as linhas do `.mailmap` por você,
veja `history-rewrite.md`; pelo terminal: `gitpoulpi mailmap list|check|add`.

## Encontrando coisas

- **Buscar** (`f`, o botão **Buscar** da barra de ferramentas, ou
  *Navegação: buscar commits...* na paleta): procura na mensagem inteira do commit,
  no nome e e-mail do autor, no hash e nos nomes das refs. Ele esmaece o que não
  corresponde e mantém a forma do grafo. `Enter` / `Shift+Enter` pulam para a
  próxima / anterior correspondência.
- **Filtro de refs** (`/` na barra lateral): restringe as listas de branches, tags
  e remotos por nome.
- **Solo** (a tecla `o`, ou o menu da ref, sobre uma ref, pasta ou remoto): mostra
  só os commits das refs em solo; use solo em várias para ver a união delas.
  `Shift+O` para o solo.
- **Ocultar** uma ref (no menu dela) para tirá-la do grafo: a pílula some e
  também os commits que só ela alcança (o commit com checkout sempre permanece,
  com a linha da árvore de trabalho). *Ocultas n* no título de Refs as conta,
  **Mostrar tudo** (`Shift+H`) as traz de volta. O total da linha de status conta
  o histórico visível.

## Branches, tags e a barra lateral Refs

Dê um clique duplo em uma branch local ou tag para fazer checkout. Clique com o
botão direito em uma ref (na barra lateral, ou sua pílula no Log) para ver o menu
dela. As entradas mais úteis:

| Entrada | Faz |
|---|---|
| Mostrar no grafo | Rola o Log até o commit da ref (carregando antes o histórico mais antigo, se preciso) e faz a linha piscar; a seleção permanece. Um único clique em uma ref faz o mesmo, a menos que *Clicar em uma ref a mostra no grafo* esteja desativado em Configurações. Uma ref oculta do grafo (ou por Solo) avisa isso na linha de status em vez disso |
| Checkout | Muda para a branch ou desanexa na tag |
| Merge, Rebase sobre | Faz merge dela em, ou rebase da branch em checkout sobre, esta ref |
| Fast-forward de `<branch>` até `<atual>` / de `<atual>` até `<branch>` | Avança uma branch quando isso é um fast-forward; senão fica acinzentado com o motivo (`divergiram, não é um fast-forward`) |
| Criar worktree a partir de... | Abre um worktree vinculado em sua própria aba |
| Pull, Push, Fetch, Remotos de <branch>... | Operações de rede nessa branch (Remotos de <branch>... abre o diálogo de remotos: o upstream mais uma caixa Push e uma Pull também por remoto; veja [Vários remotos para uma branch](multiple-remotes.md)) |
| Ocultar / Mostrar no grafo, Solo | Só exibição; ocultam ou isolam uma ref (ocultar também tira os commits que só essa ref alcança) sem tocar no repositório |
| Renomear, Excluir | Excluir pergunta antes; também é possível excluir a gêmea remota |
| Copiar nome da branch, Copiar link da branch no remoto | Para a área de transferência |

**Onde uma tag está.** Uma tag pode estar neste computador, em um remoto, ou nos dois, e as
linhas de TAGS da barra lateral (e a pílula da tag no Log) dizem qual caso com um marcador
depois do nome. Passe o mouse sobre uma tag para ver o mesmo em palavras.

| Marcador | Significado |
|---|---|
| seta para cima | Só aqui: nenhum remoto a tem. O menu dela oferece *Push* |
| seta nos dois sentidos (esquerda-direita) | Aqui e em um remoto, no mesmo commit |
| seta para baixo (linha esmaecida) | Só em um remoto: não está neste repositório (nunca recebeu fetch, ou foi excluída aqui). O menu dela oferece *Fetch de `<tag>` de `<remoto>`* e *Excluir `<tag>` em `<remoto>`...*; `Git: Fetch da tag selecionada` na paleta |
| seta para cima e para baixo | Também em um remoto, mas em outro commit (um push seria recusado) |

O git não guarda refs de rastreamento remoto para tags, então o GitPoulpi descobre o que cada
remoto tem pelo seu último **fetch** (qualquer fetch, inclusive o automático) e pelos próprios
pushes e exclusões de tags. Nada consulta a rede enquanto você navega; se outra pessoa mudou as
tags de um remoto, faça fetch para ver. A mesma informação está em `gitpoulpi tag list` (campos
`local`, `remote`, `differs_on`, `where`).

**Excluir uma tag.** Quando um remoto tem a tag, o botão vermelho **Excluir** abre um diálogo com
três linhas numeradas (`1`-`9`, ou Cima/Baixo e `Enter`; `Esc` cancela), e a linha em destaque se
explica logo abaixo:

1. **Excluir localmente**: remove só deste repositório. Roda na hora, e a mensagem avisa que o
   remoto ainda a tem. **O próximo fetch a traz de volta** (o git acompanha as tags do remoto),
   a menos que você a exclua também no remoto.
2. **Excluir em `<remoto>`**: remove lá para todos (`git push <remoto> --delete`). Pede uma
   confirmação que cita o remoto. A sua tag local continua. Não dá para desfazer.
3. **Excluir localmente e em `<remoto>`**: as duas, depois da mesma confirmação, então um fetch
   não a traz de volta. **Desfazer restaura só a tag local** (o mesmo objeto de tag); ela volta a
   ser só local e *Push* a coloca de novo no remoto.

![A caixa Excluir tag com suas três linhas, Excluir localmente, Excluir em origin e Excluir localmente e em origin, acima da seção Tags do painel de refs, onde cada tag tem uma marca que indica se está só local ou também em origin](../images/tag-delete.png)

Com vários remotos, cada um tem as suas linhas. Uma tag só aqui mantém a confirmação simples; uma
tag só em um remoto vai direto para a confirmação desse remoto. CLI: `gitpoulpi tag delete <nome>
[--local | --remote <r> | --both [--remote <r>]]`.

**Fetch e tags.** Um fetch também atualiza o que cada remoto tem (os marcadores) e o próprio
acompanhamento de tags do git continua valendo: uma tag excluída só aqui é baixada de novo. Uma
tag que alguém **excluiu no remoto** some do lado do remoto no marcador depois de qualquer fetch,
mas continua aqui como a sua tag local (agora *só aqui*), porque o padrão do git é não fazer prune
de tags. **Configurações > Sincronização > Auto-prune de tags** (desativado por padrão;
`gitpoulpi fetch --prune-tags`) também exclui essa tag local, mas só uma que estava no remoto: uma
tag que você nunca enviou, ou que você moveu, nunca é tocada. Exige o git 2.41 ou mais novo para
saber o que foi podado.

**Nova branch ou tag.** Os botões **Branch** e **Tag** da barra de ferramentas,
`Ctrl+G b` e `Ctrl+G t`, ou *Criar branch aqui...* de um commit pedem o nome em uma
caixa embutida na coluna Ref, na linha do commit de destino. `Enter` cria, `Esc`
cancela, e um nome que o git recusaria é reportado em vermelho sem fechar a caixa.

Em um commit sem ref, passar o ponteiro na linha também mostra, na coluna Ref, uma pílula esmaecida com o nome da branch a que pertence (senão a branch remota), deduzido do histórico; tags nunca são mostradas como fantasma. Uma branch apagada depois de um merge continua aparecendo, riscada, sob o nome que consta no assunto do commit de merge (`Merge branch 'x'`, `Merge pull request #N from owner/x`...); squash merges, rebases e fast-forwards não deixam esse rastro. Passar o ponteiro na pílula mostra o tipo. Ela não cria nada: use o comando de criar branch acima. **Mostrar uma branch fantasma no commit sob o mouse** em Configurações > Log e grafo a desativa; nada é desenhado onde a coluna Ref é estreita demais.

**Remotos.** O `+` no título REMOTO adiciona um; clique com o botão direito no nome
de um remoto para fazer fetch dele, copiar seu link web, torná-lo o remoto padrão,
ocultá-lo do grafo, editar suas URLs ou removê-lo. **Ocultar um remoto** (o ⊙ da
linha dele, *Ocultar remoto do grafo* no menu, ou `Remoto: ocultar remoto no grafo...`
na paleta) oculta todas as branches que ele tem agora ou vier a ter, e os commits que
só ele alcança saem do Log: a forma de navegar por um histórico sem o histórico de um
remoto de arquivo sem relação; traga-o de volta com *Mostrar remoto no grafo*. Mostrar
um remoto também mostra as branches dele que tinham sido ocultadas uma a uma. Uma
branch de um remoto oculto avisa na linha de status se o ⊙ dela for usado: mostre
o remoto primeiro.

**Stashes.** **Stash** faz stash de tudo (arquivos não rastreados inclusos);
`Ctrl+G m` pergunta primeiro uma mensagem (com a IA ativada, `Ctrl+G` dentro desse
prompt sugere uma). Clique com o botão direito em uma linha de stash, no Log ou na
seção STASHES, para **Pop**, **Apply** (mantém o stash) ou **Drop**.

**Worktrees.** *Criar worktree a partir de...* (em uma branch, tag ou commit) abre
um diálogo que propõe uma pasta ao lado do repositório, um nome opcional para uma
nova branch e “Abrir o novo worktree como aba”. Os worktrees aparecem na seção
WORKTREES; dê um clique duplo em um para abri-lo, e sua aba se fecha sozinha quando
o worktree é removido. A funcionalidade inteira (abrir em outra ferramenta, fazer merge da
branch de um worktree, remover) está em [Worktrees](worktrees.md); a seção SUBMODULES do
mesmo painel está em [Submódulos](submodules.md).

## Arrastando refs sobre commits

Você pode pressionar uma branch (na barra lateral ou sua pílula
no Log) e arrastá-la. Um rótulo fantasma segue o ponteiro e os destinos válidos se
acendem; solte para obter um menu:

- uma branch local **sobre outra branch local**: Merge, Rebase, Rebase interativo,
  Fast-forward;
- uma branch local **sobre uma branch remota**: Push para esse remoto;
- uma branch remota **sobre uma branch local**: Merge, Rebase, Fast-forward, Pull.

`Esc` cancela um arrastar. O equivalente no teclado é o comando da paleta
*Refs: soltar a ref selecionada na branch em checkout...*.

**Merge ou fast-forward.** Quando o destino não tem commit próprio (a branch arrastada está apenas à frente dele), um merge poderia simplesmente avançar o destino. Por padrão (Configurações > Git > **Merge fast-forward** = *never*, ou a escolha do próprio repositório na página Opções de merge) a linha do menu diz *Merge de x em y (commit de merge)* e cria um commit de merge de verdade, para que o histórico guarde a branch; use a linha separada *Fast-forward de y até x* quando quiser o avanço simples (para uma linha reta, use *Rebase*). Com *allowed* a linha de Merge diz *(fast-forward)*, e com *only* *(somente fast-forward)*, esmaecida com o motivo quando as branches divergiram. As mesmas linhas e rótulos estão no menu da própria branch, e `gitpoulpi merge --ff | --no-ff | --ff-only` escolhe a cada merge.

## O menu do commit

Clique com o botão direito em um commit (ou pressione `.` sobre ele) para ver o menu:

![O menu do commit: cherry-pick, revert, reset, editar mensagem, explicar com IA, rebase interativo, criar branch, tag, checkout, worktree, patches e ações de copiar](../images/context-menu.png)

| Entrada | Faz |
|---|---|
| **Cherry-pick em `<branch>`** | Reaplica o commit na branch em checkout |
| **Revert do commit...** | Cria um commit que o desfaz. Em um commit de merge, uma lista suspensa pergunta antes qual pai é a linha principal (o lado que permanece; o pai 1 é a branch em que o merge foi feito), e depois vem a confirmação de sempre. Desfazer restaura o estado anterior |
| **Reset de `<branch>` para este commit** | Um submenu para o tipo de reset; um hard reset pede confirmação |
| **Editar mensagem do commit...** | Veja abaixo |
| **Alterar...** | Muda a mensagem, o nome, o e-mail e a data do autor do commit em um único diálogo; veja [Alterando um commit](#alterando-um-commit-mensagem-autor-datas) |
| **Dividir commit...** | Divide o commit em vários; veja [Dividindo um commit](#dividindo-um-commit) |
| **Dividir commit (IA)...** | O mesmo, com um agrupamento proposto pelo modelo; só quando a [IA](ai.md) está ativada |
| **Explicar alterações (IA)** | Um popup com uma explicação da IA; só quando a [IA](ai.md) está ativada |
| **Rebase interativo a partir daqui...** | Veja abaixo |
| **Criar branch aqui...**, **Criar tag aqui...**, **Criar tag anotada aqui...** | Caixa de nome embutida |
| **Checkout deste commit**, **Criar worktree aqui...** | HEAD desanexado, ou um worktree |
| **Criar patch a partir deste commit...**, **daqui até `<branch>`...** | Grava arquivos `.patch`; *Aplicar patch* fica no menu de uma ref |
| **Copiar link do commit no remoto**, **Copiar** > | Para a área de transferência (hash, mensagem e mais) |

## Editando a mensagem de um commit (reword)

*Editar mensagem do commit...* (ou `Ctrl+G e`) abre a mensagem completa: um resumo
de uma linha com contador, uma descrição abaixo dele, **Cancelar** e **Atualizar
mensagem**. Qualquer commit da branch em checkout pode ser reescrito (HEAD ou um
ancestral); os commits acima dele são recriados com suas próprias árvores, autores
e datas, merges continuam sendo merges, e suas alterações com e sem stage ficam
intactas. Se o commit já foi enviado, o popup avisa em âmbar que você precisará de
um force push (com lease) e pede uma segunda confirmação. `Ctrl+Z` (o Desfazer da
barra de ferramentas) desfaz o reword; veja [Desfazer e refazer](undo-redo.md). A
linha de status também nomeia a ponta anterior.

Um commit que não está na branch em checkout (ele vive em outra branch) não pode ser reescrito nem alterado onde você
está, então *Editar mensagem do commit...* e *Alterar...* abrem **Editar um commit de outra branch**, uma lista de
escolhas (`Up` / `Down` ou `Ctrl+J` / `Ctrl+K` movem, `Enter` ou um clique escolhe, `Esc`, o ✕ ou
**Cancelar (C)** fecha):

- **Checkout de `<branch>` e editar (1)**: uma linha por branch local cujo histórico contém o commit (`1` a `8`, as
  oito primeiras, as demais são contadas). O checkout da branch é feito do jeito de sempre e depois a edição abre
  nela. O desfazer é o das duas ações: `Ctrl+Z` desfaz a edição, o checkout continua sendo um checkout.
- **Editar com reescrita avançada de histórico (R)**: abre o [painel de reescrita do histórico](history-rewrite.md)
  nesse commit, nas compilações que o têm.
- **Criar uma branch aqui... (B)**: só quando nenhuma branch local contém o commit (ele está só em uma branch
  remota, por exemplo); abre a caixa de nome para uma branch nele.

## Alterando um commit (mensagem, autor, datas)

*Alterar...* no menu do commit (ou `Ctrl+G Shift+E`, ou *Commit: alterar...* na paleta) abre um único diálogo com as propriedades do commit: o resumo, a descrição e o nome, o e-mail e a data do autor, cada um preenchido com os valores do commit. Mude o que quiser e pressione **Aplicar**; só os campos alterados são enviados. Uma data se escreve `YYYY-MM-DD HH:MM:SS +HH:MM` e aparece em UTC enquanto você não a editar. O committer é você, agora (como em um reword). Qualquer commit da branch atual pode ser alterado, com as mesmas regras, o mesmo aviso de commit já enviado e a mesma segunda confirmação de *Editar mensagem do commit...*; os commits por cima são recriados com seus próprios autores e datas, e suas mudanças no stage e fora dele ficam como estão. `Ctrl+Z` (Desfazer da barra de ferramentas) desfaz a operação. Em compilações com reescrita do histórico, **Modificar o histórico mais a fundo...** (`Alt+H`), no canto inferior esquerdo do diálogo, fecha-o sem aplicar nada e abre o [painel de reescrita do histórico](history-rewrite.md) com o escopo deste commit até o mais recente; se você editou um campo, ele pergunta antes (`Esc` confirma). `Ctrl+N` mostra um selo em cada campo e botão do diálogo; pressione-o para pôr o cursor nesse campo ou acionar esse botão (veja [Selos](keybindings.md#selos)).

Todos os caminhos até o diálogo o abrem completo, com todos os campos visíveis e editáveis; só o cursor muda de lugar. No painel de detalhes do commit (à direita), clique com o botão direito no resumo, na descrição ou no bloco do autor e escolha *Alterar o resumo...*, *Alterar a descrição...* ou *Alterar o autor...*, ou dê um clique duplo no resumo, na descrição, no autor ou na data de autoria: o diálogo abre com o cursor nesse campo. Os equivalentes de teclado funcionam a partir do Log e do painel de detalhes: `Ctrl+G Shift+E` (todos os campos), `Ctrl+G Shift+A` (o autor) e, na paleta, *Commit: alterar...*, *Commit: alterar autor...*, *Commit: alterar resumo...* e *Commit: alterar descrição...*. Os dois atalhos podem ser remapeados (`git_prefix.amend`, `git_prefix.edit_author`; veja [Atalhos de teclado](keybindings.md)). Em um script ou agente: `gitpoulpi amend <commit>` com `--message` (ou `--summary` / `--description`), `--author-name`, `--author-email`, `--author-date` e as opções `--committer-*` correspondentes (a ferramenta MCP é `amend`).

## Dividindo um commit

*Dividir commit...* no menu do commit (ou na paleta: *Git: dividir o commit selecionado...*)
divide um commit em dois ou mais, do mais antigo ao mais novo. Qualquer commit que não
seja de merge da branch em checkout pode ser dividido (HEAD ou um ancestral, inclusive um
commit raiz). O editor de divisão tem dois painéis: **Novos commits** à esquerda (cada
um com sua mensagem) e as **Alterações** do commit à direita, uma linha por arquivo e por
hunk, cada uma com um campo `[N v]` que indica o novo commit que fica com ela.

| Tecla | Mouse | Faz |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | clique em uma linha | percorre as alterações (`Shift+Up` / `Shift+Down` estende a seleção) |
| `1`-`9` | o campo `[N v]` da linha, depois uma escolha | atribui a alteração selecionada a esse novo commit; `0` a deixa sem atribuição |
| `Enter` / `m` | o campo `[N v]` | abre a lista de commits para a alteração selecionada |
| `n` | `+ Novo commit` | coloca a alteração selecionada em um novo commit |
| `v` | | **modo de linhas**: divide um hunk linha por linha (arraste sobre as linhas para selecionar um intervalo) |
| `Tab` | clique em um painel | alterna entre os commits e as alterações |
| `e` / `Enter` (no painel de commits) | o lápis | edita a mensagem desse commit |
| `d` / `Delete` | a cruz | remove um novo commit (as alterações dele ficam sem atribuição) |
| `Alt+Up` / `Alt+Down` | as setas | move um novo commit para cima ou para baixo |
| `s` | a caixa de seleção | assina os novos commits |
| `Shift+R` | `Redefinir` | volta ao plano inicial |
| `Ctrl+S` | `Dividir em N` | divide |
| `Esc` | a cruz, `Cancelar` | sai; com edições pendentes, um segundo toque confirma |

Cada alteração deve ir para exatamente um commit; o editor conta o que ainda está *Sem
atribuição* e o botão continua recusado até que o número chegue a zero. Um arquivo que é
binário, só muda o modo, é novo ou excluído no commit, é um submódulo ou um arquivo LFS só
pode ser atribuído por inteiro. O primeiro commit começa com a mensagem original; dê uma
mensagem a cada um dos novos. Um novo commit mantém o autor e a data originais; o
committer é você, agora.

**Nada no seu repositório muda, exceto a branch.** O último commit novo tem exatamente o
conteúdo do commit original, então os commits acima dele são recriados com as mesmas
árvores, autores e mensagens (merges continuam sendo merges), e suas alterações com e sem
stage ficam intactas: uma árvore de trabalho suja não atrapalha e nenhum conflito é
possível. Os hooks não são executados. A divisão fica bloqueada enquanto houver um
auto-stash pendente.

**Assinatura.** A caixa *Assinar commits* começa com o valor de `commit.gpgSign` do
repositório. Desmarcada, as assinaturas dos commits reescritos (os novos e os que estão
acima deles) são descartadas, e o editor avisa isso sob a caixa. Marcada, todos são
assinados; se a assinatura falhar, nada é alterado.

**Desfazer.** Um `Ctrl+Z` (o Desfazer da barra de ferramentas) traz de volta o commit
original, também com a árvore de trabalho suja; veja [Desfazer e refazer](undo-redo.md).
Um commit que já foi enviado precisa depois de um force push (com lease).

**A partir do rebase interativo.** Cada linha da revisão do rebase tem um pequeno botão
**dividir...** (`t` na linha): ele abre o mesmo editor para esse commit e, quando a
divisão termina (ou você cancela), você volta à tela de revisão, já no novo histórico. Se
você tinha editado o plano, pressione dividir uma segunda vez para confirmar que essas
edições serão perdidas.

Para scripts e agentes: `split-source` e `split-commit` na
[referência da CLI](cli-reference.md); um agente também pode abrir este editor já
preenchido para você revisar (`ui propose split`, veja [MCP e agentes](mcp-and-agents.md)).

## Rebase interativo

Escolha **Rebase interativo a partir daqui...** em um commit que seja ancestral da
branch em checkout, ou arraste uma branch sobre outra e escolha *Rebase
interativo*. O Log é substituído por uma tela de revisão: uma linha por commit, o
mais novo no topo, cada uma com uma alça de arrastar e um menu suspenso de ação.

![A tela de rebase interativo: quatro commits, o segundo definido como Squash e o terceiro como Drop, com os botões Redefinir, Cancelar rebase e Iniciar rebase](../images/rebase.png)

| Ação | Tecla | Efeito |
|---|---|---|
| Pick | `p` | Mantém o commit |
| Reword | `r` | Mantém com uma nova mensagem |
| Squash | `s` | Funde no commit de baixo, combinando as mensagens |
| Fixup | `f` | Funde no commit de baixo, descartando sua mensagem |
| Drop | `d` | Deixa de fora |
| Edit | `e` | Para ali para que você possa fazer amend |

Arraste uma linha (ou `Ctrl+Up` / `Ctrl+Down`) para reordenar. `a` pré-visualiza o
autosquash (os commits `squash!` e `fixup!` vão para perto de seus alvos). Nada
acontece até você pressionar **Iniciar rebase** (`Ctrl+S`); **Redefinir** restaura o
plano e **Cancelar rebase** (`Esc`) sai sem alterar nada. Quando uma etapa para em
um conflito ou em um `edit`, um aviso acima do Log oferece **Continuar**, **Pular**
e **Abortar**. Um intervalo que contém um commit de merge não pode sofrer rebase
interativo; a entrada do menu diz por quê.

## Cherry-pick de vários commits

`Ctrl+clique` alterna um commit, `Shift+clique` (ou `Alt+clique`) seleciona um intervalo, `Espaço`
alterna o atual, `Esc` limpa. Clique com o botão direito em um commit selecionado
para **Cherry-pick de N commits em `<branch>`...**: a mesma tela de revisão se abre,
com as mesmas ações e reordenação, e **Iniciar cherry-pick** os reaplica em ordem.

## Copiando commits como um gitGraph do Mermaid

![O popup Copiar como gitGraph do Mermaid sobre três commits selecionados: as escolhas de escopo, detalhe e direção, a opção do bloco de código e a pré-visualização ao vivo](../images/log-mermaid.png)

Selecione commits no Log (`Ctrl+clique`, `Shift+clique`, `Espaço`; ou apenas deixe o cursor em um)
e abra **Copiar como gitGraph do Mermaid...**: `Shift+Y` no Log, o submenu **Copiar** do
menu do commit, ou `Log: copiar como gitGraph do Mermaid...` na paleta. Um popup mostra o script
[Mermaid](https://mermaid.js.org/syntax/gitgraph.html) gerado, ao vivo e rolável; `Enter`
(ou o botão **Copiar**) coloca exatamente esse texto na área de transferência. Cole-o em um README, na descrição de um pull
request ou no [mermaid.live](https://mermaid.live).

O popup tem três listas suspensas, uma caixa de seleção e a pré-visualização (`Tab` move entre elas; cada
lista abre com `Enter`, `Espaço` ou `Baixo`):

- **Commits**: a seleção ou - quando um commit está selecionado e o HEAD descende dele - **Da seleção
  até HEAD**, a sequência de commits entre eles (respondida a partir da parte carregada do Log).
- **Detalhe**: `structure` (commits, branches e merges; os ids são SHAs curtos), `refs` (acrescenta os nomes
  reais das branches como branches do Mermaid, as tags como `tag:` e o commit HEAD como `type: HIGHLIGHT`),
  `subjects` (o id do commit vira `<sha> <assunto>`, com escape e cortado em `ui.mermaid_subject_width`
  caracteres) e `full` (acrescenta o autor e a data em UTC ao id).
- **Direção**: da esquerda para a direita (`gitGraph LR:`) ou de cima para baixo (`gitGraph TB:`).
- **Incluir o bloco ```mermaid** envolve o script em um bloco de código Markdown.

O detalhe, a direção e o bloco de código são lembrados (`ui.mermaid_detail`, `ui.mermaid_orientation`,
`ui.mermaid_fence`). O mesmo gerador roda pela linha de comando:
`gitpoulpi log --format mermaid [--detail structure|refs|subjects|full] [--orientation lr|tb] [--fence]
[--range A..B | --limit N]` (veja a [referência da CLI](cli-reference.md)); o servidor MCP o expõe como
o argumento `format` da ferramenta `log`.

O `gitGraph` do Mermaid é uma linguagem de reprodução (commit, branch, checkout, merge), não uma descrição
de grafo, então o gerador faz algumas escolhas que vale conhecer:

- Um commit continua a branch do seu primeiro pai selecionado; os outros filhos abrem uma nova
  branch com o nome de uma branch real nele (`b1`, `b2`... em `structure`). A primeira branch é `main`
  (ou o nome real, por um cabeçalho `mainBranchName`) e segue a cadeia de primeiros pais do HEAD.
- Um merge junta a branch do outro pai. Quando essa branch avançou depois do pai, o
  gerador cria uma pequena branch de ponteiro (`<branch>-<sha>`) no pai e faz o merge dela, de modo que o
  merge continue apontando para o commit certo.
- Um pai fora da seleção é descartado: um commit que fica sem nenhum pai selecionado inicia uma linha
  própria, então lacunas em uma seleção aparecem como linhas desconectadas.
- Um merge octopus (três ou mais pais) vira merges sequenciais; os intermediários recebem o
  sufixo ` (octopus 1/2)`, e o último traz o commit real.
- Cherry-picks não são detectados; aspas duplas nos assuntos viram apóstrofos (o Mermaid não consegue
  escapá-las).

## Comparando commits

Selecione duas ou mais linhas do Log: `Ctrl+clique` alterna uma
linha, `Shift+clique` (ou `Alt+clique`, `Shift+Up/Down`, `Alt+Up/Down`) pega um
intervalo, `Space` alterna a linha atual. Commits, a linha **WIP** e as linhas de
**stash** podem ser selecionados. O painel direito vira um painel **Comparar**:

- o cabeçalho tem um botão **Trocar** e, embaixo dele, cada linha selecionada é
  listada na ordem do Log (a mais recente primeiro) como um **cartão** de duas linhas: o avatar do autor (conforme as
  configurações de avatar do Log: uma imagem nos terminais que mostram imagens,
  iniciais nos outros), o nome, há quanto tempo, o id curto e o resumo. A linha
  **WIP** é “Diretório de trabalho / Alterações não commitadas”, um stash mostra o
  glifo de stash, `stash@{n}` e a mensagem. A extremidade mais antiga leva um
  rótulo **De** e a mais recente um rótulo **Até** (Trocar as move); com três
  ou mais linhas, uma nota diz que as alterações são combinadas. A lista de
  cartões ocupa no máximo um terço do painel e depois rola (roda do mouse, barra
  de rolagem, ou arraste o divisor embaixo para redimensionar);
- clique em um cartão para levar o cursor do Log até aquela linha e fazê-la piscar
  no grafo; a seleção não muda, então `Space` depois tira essa linha dela;
- embaixo, a lista habitual de arquivos alterados (visão por caminho ou árvore,
  filtro, ordenação) lista os arquivos que diferem entre elas. Clique em um
  arquivo, ou pressione `Enter`, para abrir o diff dele no centro, nos mesmos
  layouts unificado ou dividido, com o Blame, o Histórico do arquivo e a visão do
  arquivo na extremidade mais recente.

![Três linhas do Log selecionadas: o painel Comparar as lista como cartões com avatares e rótulos De / Até, acima dos arquivos alterados](../images/compare-cards.png)

O que é comparado depende da seleção:

| Seleção | Comparado |
|---|---|
| dois commits (ou um stash e um commit) | o mais antigo com o mais recente, na ordem do Log, seja qual for a ordem dos cliques |
| a linha WIP e um commit ou stash | esse commit com o diretório de trabalho: alterações em stage e fora dele dos arquivos rastreados (arquivos não rastreados não fazem parte de um diff) |
| três ou mais commits | as alterações combinadas deles: de logo antes do mais antigo até o mais recente (merges seguem o primeiro pai) |

Um arquivo renomeado ou movido aparece como um só arquivo, com o nome antigo e a similaridade no topo do diff. **Trocar** (`x`) inverte as
extremidades de uma comparação de duas linhas; com a linha WIP, que é sempre a
extremidade mais recente, ou com um intervalo de três ou mais, não há o que trocar.
Uma comparação com a linha WIP acompanha o diretório de trabalho conforme você
salva arquivos.

Teclado: `v` (ou **Log: comparar a seleção** na paleta de comandos) leva o foco para
a lista de arquivos da comparação; `x` (ou **Log: Trocar as extremidades da
comparação**) troca; `n` (ou **Log: ir para a próxima linha comparada**) é o
equivalente de teclado de clicar em um cartão: o cursor do Log vai para a próxima
linha selecionada, voltando ao início no fim. `Esc` ou um clique simples em uma linha
volta para o commit único. A entrada **Cherry-pick** só conta os commits da seleção;
a linha WIP e os stashes são ignorados ali.

As mesmas listas estão disponíveis para scripts e agentes:
`gitpoulpi diff --from <rev> [--to <rev|worktree>] [--name-only]` (a CLI e a
ferramenta MCP `diff`; `--to` é o diretório de trabalho por padrão).

## Conflitos e outros avisos

Quando um merge, rebase, cherry-pick ou revert para em um conflito, um aviso
aparece acima do Log e os arquivos em conflito ganham um `!` vermelho no painel
Status. Resolva-os no editor de merge e depois use **Continuar** ou **Commit merge**
do aviso, ou **Abortar** para recuar. Veja
[Resolvendo conflitos de merge](merge-conflicts.md).

Outros avisos acima do Log:

| Faixa | Botões | Paleta |
|---|---|---|
| *Rebase de `<branch>` sobre `<commit>` · etapa/total · parou em ...* | **Continuar**, **Pular**, **Abortar** | *Rebase: continuar / pular / abortar* |
| *Aplicando patches (git am) · patch n de m* (um `git am` que parou) | **Resolver conflitos...** (quando um patch tem conflito), **Continuar**, **Pular**, **Abortar** (pergunta antes) | *git am: Continuar / Pular este patch / Abortar* |
| *Um index.lock deixado para trás (n s) bloqueia qualquer alteração neste repositório* (um comando git morreu e deixou `.git/index.lock`) | **Remover index.lock** (pergunta antes; só é oferecido quando o arquivo tem pelo menos 10 segundos) | *Repositório: remover index.lock obsoleto* |
| *Auto-stash pendente: ...* | Veja [Auto-stash](#auto-stash) | *Aviso: ...* |

## Blame

O Blame mostra o arquivo inteiro com números de linha e cores de sintaxe, e uma coluna de anotações à **direita** do código (`ui.blame_side = "left"` a move para a esquerda). Abra-o pela barra de um arquivo (`Blame`), pelo menu do arquivo, pela paleta ou a partir de uma linha do histórico (`b`). A partir da árvore de trabalho, as linhas ainda sem commit aparecem à parte (veja Faixas).

- **Hunks.** Uma sequência de linhas consecutivas cuja última alteração é de um mesmo commit é um hunk. A primeira linha mostra o avatar do autor (uma imagem onde o terminal desenha imagens, senão o selo de iniciais, nada quando `ui.avatars` está desativado), o autor, há quanto tempo e o resumo do commit; as demais linhas mostram só a barra de cor do autor. Uma linha fina separa os hunks. Quando um hunk começa acima da tela, sua anotação se repete na primeira linha visível.
- **Faixas.** Ao lado da anotação, a barra de cor do autor e uma **faixa de calor de idade**: quente para linhas recentes, fria para antigas, relativa à linha mais antiga e à mais recente do próprio arquivo (a legenda está no título). Linhas sem commit são pontilhadas e dizem “Ainda não commitado”.
- **Cartão.** Deixe o ponteiro um instante sobre uma anotação, ou pressione `i`, para ver o cartão de detalhes do hunk: ID completo do commit, autor e e-mail, data, resumo e suas ações. O hunk selecionado fica tingido.

  ![Blame com o minimapa ligado e o cartão aberto no hunk selecionado](../images/blame-card.png)
- **Ações** (linhas do cartão, menu de contexto num hunk, teclas e `Blame: ...` na paleta): mostrar o commit no Log (`Enter`), copiar seu ID (`Shift+Y`), mostrar o diff do commit para este arquivo (`d`), Blame no pai do commit (`b`, seguindo renomeações), abrir o histórico do arquivo (`Shift+H`).
- **Mover-se.** `j` / `k`, PageUp/PageDown, Home, `Shift+G` e a roda como em todo lugar; `n` / `p` hunk seguinte / anterior, `Shift+N` / `Shift+P` hunk seguinte / anterior do mesmo commit, `g` ir para um número de linha.
- **Hunks altos.** Sob o resumo, um hunk de três ou mais linhas mostra o ID curto do commit e a data completa (esmaecidos), para a coluna não ficar vazia.
- **Minimapa.** Com `ui.minimap` ligado (o botão `Mapa`, `M`), o minimapa do arquivo fica ao lado do código, colorido com o mesmo calor de idade da faixa; clique ou arraste ao longo dele para rolar.
- **Largura.** Arraste a borda da coluna de anotações ou pressione `>` / `<`; a largura é salva como `ui.blame_width`.

## Histórico do arquivo

Abra-o pelo segmento `Histórico` do seletor `Histórico │ Todos` da barra do diff, por um menu de arquivo ou pela paleta (`Histórico do arquivo: ...`). O centro se divide em dois: à esquerda os commits do arquivo, do mais recente ao mais antigo, seguindo renomeações; à direita o diff do arquivo no commit selecionado, com todos os modos da visão Diff (hunks / arquivo inteiro, em linha / dividido, diff por palavra, minimapa, quebra de linha).

- **A lista.** Duas linhas por commit. A primeira: a letra da alteração (A adicionado, M modificado, D excluído, R renomeado), o resumo e os `+adicionadas -removidas` coloridos. A segunda, esmaecida: o avatar do autor (exatamente o que a coluna Autor do Log mostra conforme *Avatares dos autores*), o autor, a data relativa, o ID curto do commit e `antigo -> novo` no commit que renomeou o arquivo. Uma lista estreita descarta primeiro a renomeação, depois o ID, depois o fim do nome do autor, mas nunca o autor nem a data. Páginas de 100 são carregadas ao chegar perto do fim; enquanto isso aparece uma linha `carregando mais...`.
- **Mover-se.** `j` / `k`, setas, PageUp/PageDown, Home / End, a roda e um clique; o diff acompanha na hora (os diffs são lidos em segundo plano, guardados por linha, e uma tecla segurada pede só um). Uma linha anterior a uma renomeação é comparada com o nome que o arquivo tinha então.
- **Diff / Arquivo / Blame.** Os botões **Arquivo**, **Diff** e **Blame** da própria barra da visualização escolhem o que o lado direito mostra para o commit selecionado: sua **visão Diff**, a **visão Arquivo** (o arquivo como era naquele commit) ou seu **Blame**, tudo dentro da divisão, com a lista à esquerda, e o segmento `Histórico` do seletor da coluna continua aceso. O modo escolhido é mantido ao trocar de linha e o lado direito acompanha a seleção. Teclas `Alt+1` / `Alt+2` / `Alt+3` (na lista também `d` / `o` / `b`), de qualquer metade, e as linhas `Histórico do arquivo: ...` da paleta fazem o mesmo; `Esc` volta ao Diff.
- **A coluna.** A lista é a *coluna de commits* mostrando a sua lista **Histórico**: ocultar, mostrar, tamanhos e o seletor `Histórico │ Todos` estão descritos uma só vez em [A coluna de commits](interface-tour.md#a-coluna-de-commits). Um clique em `Todos` sai do Histórico do arquivo e mantém a visualização no commit selecionado.

  ![Histórico do arquivo com um commit do meio selecionado: seu diff à direita](../images/file-history.png)

  ![Histórico do arquivo com o lado direito em modo Blame, o blame do arquivo no commit selecionado](../images/file-history-blame.png)
- **Comparar dois commits.** `Shift+Up` / `Shift+Down` ou `Shift+clique` selecionam um intervalo; o lado direito mostra o diff do arquivo entre as duas pontas, cada uma com seu caminho. `Esc` descarta o intervalo.
- **Foco.** `Tab` (ou um clique) dá o teclado ao diff e o devolve; o diff mantém suas próprias teclas.
- **Ações** (menu de contexto, teclas e `Histórico do arquivo: ...` na paleta): mostrar no Log (`Enter`, clique duplo; o Log carrega até onde for preciso), copiar o ID (`y`) ou o resumo (`Shift+Y`), mostrar o diff (`d`), o arquivo naquele commit (`o`) ou seu blame (`b`) à direita, comparar com a árvore de trabalho (`c`, de novo para sair), restaurar o arquivo àquela versão (`Shift+R`, com confirmação e registrada no diário, então Desfazer traz de volta os bytes antigos).
- **Desfazer.** Restaurar uma versão é um passo registrado: Desfazer (`Ctrl+Z`) devolve os bytes antigos do arquivo.
- **Divisor.** Arraste a borda entre as metades, ou `Alt+Right` / `Alt+Left`, `Alt+0` para redefinir; a largura é lembrada por sessão.

## Fetch, pull e push

- **Pull** (`Ctrl+G p`) faz pull da forma que *Configurações > Git e codificação >
  Botão Pull* indica (`git.pull_mode`: merge, somente fast-forward, rebase; uma
  configuração por repositório). Seu `▾` executa uma vez *Fetch All*, *Pull
  (fast-forward se possível)*, *Pull (só fast-forward)* ou *Pull (rebase)*, sem
  mudar o botão. O modo é sempre passado explicitamente ao git, então o
  `pull.rebase` da sua configuração do git não pode lhe pregar peças.
- **Push** (`Ctrl+G Shift+P`) envia a branch em checkout para o remoto padrão,
  definindo o upstream na primeira vez. Seu `▾` faz push uma vez para outro remoto
  de sua escolha. Se o remoto tem commits que você não tem, um diálogo oferece
  **Pull (merge) e depois push**, **Force push (com lease)...** (nunca um
  `--force` puro) ou **Cancelar**.
- `Ctrl+G f` faz fetch de todos os remotos. Em *Configurações > Sincronização*,
  *Auto-prune* decide se os fetches fazem prune, e *Intervalo do auto-fetch
  (minutos)* faz fetch em segundo plano.

Tudo isso também está disponível como `gitpoulpi fetch`, `pull` e `push` na
[CLI](cli-reference.md).

## Auto-stash

Quando alterações não commitadas bloqueiam um checkout, pull, merge ou rebase, o
GitPoulpi pode colocá-las de lado para você: ele faz stash delas, executa a
operação e depois faz pop do stash. *Configurações > Sincronização > Auto-stash
antes de checkout, pull, merge ou rebase* (`general.auto_stash`):

- **perguntar** (padrão): um diálogo oferece *Fazer stash das alterações e continuar*, *Sempre fazer stash automaticamente* (o mesmo, e define `general.auto_stash = "always"` para você não ser perguntado de novo; Configurações > Sincronização reverte) ou *Cancelar*.
  A CLI e o MCP falham em vez disso e citam `--autostash`.
- **sempre**: feito sem perguntar. **nunca**: a operação é recusada.

*Incluir arquivos não rastreados no auto-stash* também leva os arquivos não
rastreados. Uma árvore limpa nunca vê um stash. O stash, a operação e o pop podem
ser desfeitos cada um.

Se o stash não puder ser recolocado (entra em conflito com o resultado, ou um merge
ou rebase parou em seu próprio conflito), ele permanece na lista de stashes. Um
**aviso sobre o Log** diz *Auto-stash pendente: suas alterações de antes do checkout
estão em stash@{n}*. Seus botões:

| Botão | Faz |
|---|---|
| **Resolver conflitos...** | Abre o editor de merge (só após um pop com conflito) |
| **Manter resultado** | Mantém a árvore de trabalho como está e descarta a cópia do stash (só após um pop com conflito, quando nenhum arquivo está mais em conflito) |
| **Pop agora** (**Tentar pop de novo** após um pop que falhou) | Recoloca o stash |
| **Mostrar stash** | Mostra as alterações do stash |
| **Recomeçar** | Veja abaixo |
| **Drop** | Descarta o stash (pergunta antes) |

Cada botão tem um comando na paleta (*Aviso: pop do auto-stash pendente*, *Aviso:
recomeçar...*, ...). Nada é repetido ou descartado por você.

**Recomeçar** pergunta antes. Depois ele:

1. guarda uma cópia do estado atual dos arquivos do stash em
   `refs/gitpoulpi-backup/autostash-start-over-<time>`;
2. descarta as alterações apenas nesses arquivos (ele se recusa, e os nomeia, se
   outros arquivos têm alterações locais que poderiam ser perdidas);
3. desfaz a operação que fez o auto-stash, pelo diário de desfazer (portanto essa
   operação precisa ser a mais recente);
4. faz pop do stash de volta no commit original e limpa o aviso.

Se uma etapa falhar, ele para com o stash intacto e diz onde. Enquanto o aviso é
exibido e a operação mais recente é a que fez o auto-stash, **Desfazer** (`Ctrl+Z`,
`gitpoulpi undo`) faz o mesmo, após uma confirmação. Os scripts usam
`gitpoulpi autostash start-over [--dry-run] [--yes]` (código de saída 3 sem
`--yes`).

**Enquanto um auto-stash está pendente, as operações que movem o histórico são
recusadas:** checkout de uma branch ou commit, merge, rebase (o interativo também),
pull, fast-forward, cherry-pick, revert, reword, composição de commits com IA e
refazer. A janela deixa esses botões e entradas da paleta acinzentados e dá o
motivo. A CLI e o MCP recebem um erro `PendingAutoStash` (código de saída 1). Todo
o resto continua permitido: leituras, stage, descartar, commit, branches, tags,
remotos, fetch, push, continuar / pular / abortar rebase, resolução de conflitos,
`stash push`, `reset` e `stash pop` / `apply` / `drop`. Desfazer também é permitido.

Quando o auto-stash aguarda um merge ou rebase que parou em conflitos, ele faz pop
sozinho assim que essa operação termina (no GitPoulpi, ou com o git puro seguido de
uma atualização).
