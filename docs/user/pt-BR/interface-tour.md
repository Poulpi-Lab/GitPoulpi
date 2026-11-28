[English](../interface-tour.md) · [Čeština](../cs/interface-tour.md) · [Deutsch](../de/interface-tour.md) · [Español](../es/interface-tour.md) · [Français](../fr/interface-tour.md) · [Italiano](../it/interface-tour.md) · [日本語](../ja/interface-tour.md) · [한국어](../ko/interface-tour.md) · [Nederlands](../nl/interface-tour.md) · [Polski](../pl/interface-tour.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/interface-tour.md) · [Русский](../ru/interface-tour.md) · [Svenska](../sv/interface-tour.md) · [Türkçe](../tr/interface-tour.md) · [Українська](../uk/interface-tour.md) · [简体中文](../zh-CN/interface-tour.md)

# Tour da interface

Esta página dá nome a cada área da janela do GitPoulpi, de cima para baixo. As
outras páginas usam os mesmos nomes. As imagens usam um pequeno repositório de
demonstração com autores inventados.

![A janela principal com suas áreas: barra de abas, barra de ferramentas, barra lateral Refs, Log, painel de alterações em andamento, formulário de commit e linha de status](../images/hero.png)

## A moldura

### Barra de título

A borda superior da janela mostra
<img src="../../../assets/logo/gitpoulpi.png" alt="Ícone do GitPoulpi" height="16"> `GitPoulpi vX.Y.Z -- <repository path>` (o nome da sessão fica na linha de status, embaixo por padrão).

- **<img src="../../../assets/logo/gitpoulpi.png" alt="Ícone do GitPoulpi" height="16">** abre a janela **Sobre o GitPoulpi**: versão, recursos da compilação, licença,
  links, um breve aviso de que o GitPoulpi é um projeto independente, os crates de
  terceiros e as licenças e os créditos. Em um terminal que desenha imagens, o polvo é o
  logotipo em pixels, na cor do seu logotipo (nos outros fica o emoji). Paleta:
  `Ajuda: sobre o GitPoulpi`; teclas: `Ctrl+N` e depois `A`. Em Sobre, as pupilas
  do polvo seguem o ponteiro do mouse. Clique no logotipo (ou pressione `Space`)
  para girá-lo até a próxima cor. Clique com o botão direito no logotipo (ou
  pressione `c`) para ver a lista de cores. O GitPoulpi guarda sua escolha em
  `ui.logo_color`. Os links abrem com um clique ou com a tecla de acesso deles
  (`s` código-fonte, `d` documentação, `i` licença), e `l` lista os crates de
  terceiros com versões e licenças (uma lista rolável; `Esc` ou `l` volta, assim como um clique no título dela).
  Cada crate da lista tem um link para a sua página (o repositório ou, na falta dele, a página no crates.io): clique na linha, ou escolha-a com `↑` `↓` e pressione `Enter`, para abri-la no navegador. `t` (ou o comando da paleta `Help: Licenses and Credits`) mostra os avisos de licença e os créditos de tudo o que o programa traz embutido, em um texto rolável (`Up` / `Down`, `PgUp` / `PgDn`, `Home` / `End`; `Esc` ou `t` para voltar). Em um script, `gitpoulpi licenses` imprime o mesmo texto.
- A **versão** (`v1.0.0`; `vX.Y.Z disponível` quando existe uma versão mais nova)
  fica entre o nome e o caminho: um clique abre o painel de atualização quando
  existe uma versão mais nova, senão a aba Notas da versão. Veja
  [Atualizações](updates.md).
- **Abrir em ▾**, logo após o caminho, abre a pasta da aba em um terminal,
  um editor, um multiplexador ou no gerenciador de arquivos (`Alt+O`). Veja
  [Abrir em](open-in.md).
- **⌕ Busca global** (uma lupa e seu rótulo), logo à esquerda da engrenagem, abre a [busca global](search.md) (`Ctrl+Alt+F`). É a primeira coisa a ceder quando a barra de título fica estreita demais: primeiro some o rótulo (a lupa fica), depois o botão.
- A **engrenagem** (`⚙`), logo à esquerda do **×**, abre as **Configurações** (`Ctrl+,`).
- **×** na extremidade direita sai, como `q`.

![A janela Sobre sobre a janela principal: o polvo em pixels, a versão, a licença, os recursos da compilação, os links e o número de crates](../images/about.png)

![A lista de crates de terceiros da janela Sobre: em cada linha o nome, a versão, um link para a página do crate e a licença](../images/about-crates.png)

### Linha de status

A linha de status é, por padrão, a última linha da janela (**Configurações > Aparência > Posição da linha de status** a leva para o topo, acima das abas de repositório). Ela mostra a branch atual, o número de commits carregados,
`dirty` ou `clean`, o nome da sessão e as teclas mais úteis: `?` ajuda,
`Ctrl+G` ações do git, `Ctrl+N` navegação, `Ctrl+P` comandos, `Shift+F10`
menu, `q` sair. Em uma janela estreita, as teclas menos úteis ficam de fora. A
linha de status também mostra mensagens de status e progresso. As teclas que ela mostra são sempre
as teclas em vigor, inclusive as que você mudou (veja
[Configurações](settings.md#teclado-e-mouse)).

A contagem de commits aparece como `500 / 48213 commits` enquanto ainda há histórico mais antigo a carregar (`500 / … commits` enquanto o total está sendo contado) e apenas como `2500 commits` quando tudo foi carregado. Em um checkout parcial (sparse), uma etiqueta `parcial` vem depois do nome da sessão. Como o total é contado é uma configuração: **Contar o total de commits com** (veja [Configurações](settings.md#log-e-grafo)).

Na extremidade direita, o botão **Atividade** abre o Registro de atividade: uma aba
Operações do Git (as operações com o tempo que levaram, mensagens e erros), uma aba
Aplicativo (o registro do aplicativo, com uma caixa de nível e um botão **Limpar registro**)
e uma caixa Escopo que restringe as duas a este repositório, a um espaço de trabalho ou a
todos os repositórios. Um número
no botão conta os erros que você ainda não viu. Quando uma operação falha, a linha de status
diz isso em vermelho; um clique nela abre a entrada do Registro de atividade, com o
motivo, um próximo passo, os detalhes e um Copiar para uma IA. Os dois estão descritos em
[Quando algo falha](troubleshooting.md#quando-algo-falha).

À esquerda do botão **Atividade**, o indicador de tarefas mostra uma tarefa em segundo plano com um `[x]` para cancelá-la (a lista de tarefas tem **Cancelar tudo**, e a paleta tem *Tarefas: cancelar tudo o que for possível*; uma tarefa que não pode ser interrompida com segurança diz o motivo). A aba Início tem a mesma linha de status, com o botão **Atividade** no canto inferior direito (ele abre em Todos os repositórios).

### Sobreposições

A paleta de comandos, a ajuda, os menus de contexto, os diálogos, os popups e as
Configurações abrem sobre a janela. Cada um tem um `✕` no canto superior direito da
borda; `Esc` ou um clique no `✕` o fecha, e um clique fora fecha um diálogo, um popup ou
um painel (veja [Fechando janelas e diálogos](mouse-and-gestures.md#fechando-janelas-e-diálogos)).

### Quando a janela é pequena

Nada some em silêncio quando a janela estreita. Um botão primeiro perde o rótulo longo (cortado com `...`), depois
passa para uma segunda linha ou vai para o menu `⋯` da sua linha, e a sua tecla e a sua entrada na paleta continuam
funcionando o tempo todo. Um diálogo grande demais para a janela desenha uma moldura pequena com o título, o `✕` e a
linha **Janela pequena demais: aumente-a** em vez de nada; `Esc` e `✕` o fecham, e ele volta inteiro quando você
aumenta a janela.

## Linha de abas

A aba **Início** (o ícone de casa) fica na extremidade esquerda, seguida da
[etiqueta do espaço de trabalho](workspaces.md) (quando você tem um espaço de trabalho), da aba fixa
**Radar** (ela fecha como qualquer aba; veja [o Radar](radar.md)) e da aba
**Espaços de trabalho**, depois que você a abriu (`Alt+J`; ela fecha como qualquer aba). Depois vem
uma aba por repositório aberto. O `x` que fecha uma aba só aparece na aba ativa e na
aba sob o ponteiro. A linha de abas só tem abas (o botão da
[busca global](search.md) fica na barra de título).

Quando as abas dos repositórios não cabem, a linha rola em vez de encolhê-las: as abas ficam
entre dois botões de seta, `‹` à esquerda e `›` à direita. Uma seta fica esmaecida quando
nenhuma aba está oculta daquele lado. Clique em uma seta para mostrar mais uma aba daquele
lado (mantenha pressionado para continuar rolando), ou use a roda sobre a linha (para cima ou
para a esquerda mostra as abas anteriores). Sempre que a aba atual muda (você troca de aba pelo
teclado, pelo seletor de abas ou pela paleta, abre ou fecha uma aba, ou redimensiona a janela),
a linha rola o suficiente para mostrá-la inteira. Havendo espaço, um botão `…` depois da seta
direita abre o seletor de abas. Arrastar uma aba sobre uma seta rola a linha enquanto você a
mantém ali, para soltá-la depois das abas visíveis. `Ctrl+N` coloca um distintivo em cada seta ativa.

**Configurações > Aparência > Posição da barra de abas** move esta linha para o fim da janela (abaixo da linha de status) ou a transforma em uma coluna na borda esquerda: uma aba por linha, primeiro Início, depois a etiqueta do espaço de trabalho, o Radar, a aba Espaços de trabalho, as abas (um nome longo termina em `…`) e o `+`. Quando há mais abas que linhas, uma linha `…` mostra quantas estão ocultas acima (`▲`) e abaixo (`▼`) e abre a lista de abas; a roda rola a coluna. Ao lado de uma barra de ferramentas em coluna, as abas ficam por fora. Uma janela com menos de 80 colunas mantém as abas no topo. Cliques, a cruz de fechar, clique do meio, o menu do clique direito, arrastar para reordenar e o teclado funcionam igual em qualquer posição.

- O **+** logo depois da última aba abre um pequeno menu: **Nova aba** (`Alt+T`, paleta
  **Aba: Nova aba**) e, enquanto as abas deles estão fechadas, **Radar** e
  **Espaços de trabalho** (`Alt+J`), que abrem essas abas; **Aba: nova...** na paleta abre o mesmo
  menu. Uma **Nova aba** é a visão Início com o campo de busca pronto, então digitar filtra seus
  repositórios na hora. Escolha um (ou Procurar, Clonar, Init) e a aba dele aparece no
  fim da barra; um repositório que já está aberto apenas é ativado. `Esc` na busca
  vazia, `Ctrl+W`, o `x` dela ou um clique com o botão do meio fecha a Nova aba e volta
  para a aba em que você estava. Ela nunca é salva com a sua sessão.

  ![Uma Nova aba: a visão Início com o campo de busca pronto, sobre as abas abertas](../images/new-tab.png)
- Clique com o botão direito em uma aba, ou pressione `Ctrl+T`, para o menu da aba: **Nova aba**, **Fechar aba**,
  **Fechar outras abas...**, **Fechar abas à direita...**, **Adicionar aos favoritos** (ou
  **Remover dos favoritos**), **Adicionar ao espaço de trabalho** (ou **Remover do
  espaço de trabalho**, quando você tem espaços de trabalho), **Copiar caminho**, **Abrir em ›**,
  **SSH e credenciais...** e **Checkout parcial...**.
- `Ctrl+PageUp` e `Ctrl+PageDown` alternam entre as abas, assim como `Ctrl+Shift+Tab` e
  `Ctrl+Tab` onde o seu terminal os repassa (protocolo de teclado kitty; alguns
  terminais e multiplexadores os guardam para si). `Ctrl+W` fecha a aba
  atual. Um clique com o botão do meio em uma aba também a fecha (com a mesma pergunta).
- `t` (ou `Ctrl+N Tab`) abre o seletor de abas, uma lista pesquisável das abas
  abertas.
- **Reordenar abas:** pressione uma aba de repositório e arraste para o lado (um fantasma
  da aba acompanha o ponteiro e uma barra mostra onde ela vai cair; `Esc` cancela), ou
  arraste uma linha do seletor de abas, pressione `Alt+Cima` / `Alt+Baixo` na linha
  destacada do seletor, ou execute **Aba: mover para a esquerda** / **Aba: mover para a
  direita** pela paleta. A ordem é salva com a sessão. Um clique simples, sem
  movimento, ainda seleciona a aba. A casa, a etiqueta do espaço de trabalho e o
  `+` nunca se movem.
- **Sair e cd aqui** aparece à direita das abas quando você iniciou o
  GitPoulpi pelo wrapper de shell `gp` e a aba não é a pasta em que você
  iniciou. Veja [Seguindo a pasta da aba ativa](following-the-tab-folder.md).
- A **etiqueta do espaço de trabalho** logo depois da casa (um ponto colorido e um nome,
  ou "Sem espaço de trabalho") troca de [espaço de trabalho](workspaces.md): clique nela
  ou pressione `Alt+W`. Ela aparece quando você tem um espaço de trabalho.

![O seletor de abas listando os repositórios abertos](../images/tab-picker.png)

Com **Lembrar abas** ligado (o padrão), suas abas voltam na próxima
inicialização. Um worktree abre em sua própria aba. Essa aba fecha quando o worktree é
removido.

A aba de um worktree fica logo depois da aba do seu repositório, e uma aba de worktree arrastada para longe volta para lá (**Agrupar as abas de worktrees** em Configurações > Geral, ativado por padrão).

### Início

A aba Início é o gerenciador de repositórios. Ela tem quatro seções: **REPOSITÓRIOS
ABERTOS**, **FAVORITOS**, **REPOSITÓRIOS RECENTES** e **TODOS OS
REPOSITÓRIOS**. Cada linha mostra a branch, as contagens de commits à frente e atrás e o
número de alterações e, havendo espaço, o caminho da pasta: abreviado com `~` e cortado no meio quando estreito, ele
nunca empurra o nome, o proprietário nem a branch para fora. Cada linha tem uma estrela (favorito), **Abrir** ou **Ir**,
**Fechar** ou **Remover**, e um menu `⋮`.

No topo: **Procurar**, **Clonar**, **Init**, uma caixa de busca, **Recolher tudo**
e **Expandir tudo**. Uma pasta que não existe mais sai de *Recentes* e *Todos*
sozinha. Um favorito ausente permanece, com um botão **Remover**. **Remover**
nunca exclui nada no disco.

Teclas: `Enter` abrir, `f` favorito, `x` fechar ou remover, `y` copiar o caminho, `o` abrir na ferramenta padrão (o **Abrir em** do menu `⋮` escolhe outra), `.` menu, `/` busca,
`Space` recolher uma seção, `Esc` voltar à sua aba.
`Ctrl+N` mostra um selo no campo de busca, em cada seção e em cada botão: pressione o caractere de um selo para focá-lo ou acioná-lo (veja [Selos](keybindings.md#selos)).

Um único campo de busca filtra tudo o que está abaixo dele, inclusive os favoritos: um repositório fica se o nome, a branch ou o caminho contiver o texto. Os espaços de trabalho têm uma aba própria e não são uma página da aba Início: abra-a pelo menu do `+`, com `Alt+J` ou pela paleta, e feche-a com o seu `x`: veja [Espaços de trabalho](workspaces.md).

**Clonar** lista, para cada perfil conectado (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit), os repositórios
a que você tem acesso, uma seção por proprietário: os seus primeiro, depois uma por
organização ou grupo (em ordem alfabética), cada uma sob um cabeçalho. Um cadeado
marca um repositório privado. A caixa de busca filtra todas as seções; uma seção
sem correspondência desaparece. `Up` e `Down` pulam os cabeçalhos, e `Home`, `End`,
`PgUp` e `PgDn` funcionam na lista.

![A aba Início: repositórios abertos, favoritos e recentes, e os botões Procurar, Clonar e Init](../images/home.png)

## Barra de ferramentas

Por padrão, uma linha abaixo dos painéis e acima da linha de status. **Configurações > Aparência > Posição da barra de ferramentas** a coloca no topo (sob as abas), na base, ou em uma coluna à esquerda ou à direita dos painéis (uma janela com menos de 80 colunas desenha a coluna como uma linha na base). Os botões ficam no início da linha ou da coluna; **Configurações > Aparência > Alinhamento dos botões da barra de ferramentas** os centraliza ou os leva ao fim. Cada botão tem sua própria cor de ícone, definida pelo tema (veja [Temas](themes.md)). Quando não há espaço para todos, os botões menos importantes vão para trás de um `⋯`. Em uma coluna, cada botão ocupa uma linha, com seu `▾` no fim da linha, e seu menu abre ao lado da coluna; o menu de uma barra de ferramentas na base abre para cima.

| Botão | O que faz |
|---|---|
| **Desfazer**, **Refazer** | Desfazem ou restauram a última operação (`Ctrl+Z`, `Ctrl+Y`). Ocultos quando Desfazer está desligado. Veja [Desfazer e refazer](undo-redo.md). |
| **Pull** `▾` | Faz pull do jeito que **Configurações > Git e codificação > Botão Pull** define. A lista `▾` tem **Fetch All**, **Pull (fast-forward se possível)**, **Pull (só fast-forward)** e **Pull (rebase)**, depois **Pull de todas as branches** (`Ctrl+G Ctrl+P`) e o mesmo com cada um dos outros modos para a branch em checkout. Um item da lista roda uma vez e não muda o botão. |
| **Push** `▾` | Faz push da branch em checkout. A lista `▾` faz push uma vez para o remoto que você escolher. |
| **Branch** | Cria uma branch no commit em checkout. |
| **Tag** | Cria uma tag no commit em checkout. |
| **Stash**, **Pop** | Faz stash de todas as alterações. Faz pop do último stash. |
| **LFS** `▾` | Ações do Git LFS: fetch, checkout, pull, push, prune (`Ctrl+G l`). Oculto quando o Git LFS está desligado. Veja [Git LFS](lfs.md). |
| **Flow** `▾` | O menu do fluxo de trabalho de branches (iniciar, concluir, sincronizar, promover; `Ctrl+G Shift+W`). Só em um repositório que tem um fluxo de trabalho. Veja [Fluxos de trabalho de branches](branching-workflows.md). |
| **Buscar** | Abre a barra de busca de commits sobre o Log (`f`). |

Na extremidade (a direita de uma linha, a base de uma coluna), quando o repositório está vinculado a um host, um **chip** mostra o pull request da branch em checkout (por exemplo `GH #12` ou `GL !53`), ou um `+ PR` esmaecido. **Configurações > Aparência > Fixar o botão de pull request no fim** (ativado por padrão) o mantém ali; desativado, ele flui com os outros botões e segue o alinhamento deles. Em uma linha estreita, o chip encolhe primeiro (`● #12`, depois só o glifo de estado) e só então passa para o menu `⋯`. Veja [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md).

Ao lado dele (em uma coluna, na última linha), quando você tem dois ou mais [perfis](settings.md#perfis),
a **caixa de combinação de perfil** escolhe a identidade de commit deste repositório. A
primeira linha, *Padrão*, segue **Configurações > Perfis**. Depois que você escolhe um
perfil, o GitPoulpi pergunta se também deve gravar o nome e o e-mail dele na config do git do
repositório (`user.name`, `user.email`). **Gravar na config do git**
os grava. `Esc` mantém a escolha somente no GitPoulpi. Paleta:
`Repositório: alterar perfil...`.

Todo botão tem um comando da paleta e um atalho.

![A lista Pull aberta sob a barra de ferramentas: Fetch All e as três formas de fazer pull](../images/pull-push-tooltip.png)

## Banners

Um aviso acima do Log avisa quando o repositório está em um estado especial. Ele
tem botões para o caminho a seguir. Cada botão também é um comando da paleta.

| Banner | Botões |
|---|---|
| Um merge, cherry-pick ou revert parou em conflitos | **Resolver conflitos...**, **Commit** ou **Continuar**, **Abortar** |
| Um rebase em andamento | **Resolver conflitos...**, **Continuar**, **Pular**, **Abortar** |
| **Aplicando patches (git am)** | **Resolver conflitos**, **Continuar**, **Pular**, **Abortar** |
| Um `index.lock` deixado para trás bloqueia qualquer alteração | **Remover index.lock** (pergunta antes) |
| Um auto-stash não foi recolocado | **Pop agora**, **Mostrar stash**, **Recomeçar**, **Drop** (as opções dependem do estado) |

Veja [Resolver conflitos de merge](merge-conflicts.md) e
[Solução de problemas](troubleshooting.md).

## Barra lateral Refs (esquerda)

Uma árvore de tudo que dá nome a um commit. Cada seção tem uma contagem no
título, um chevron para recolhê-la e sua própria posição de rolagem. Arraste a borda
entre duas seções para redimensioná-las.

- Caixa **Filtro** no topo (`/`): filtra todas as seções por nome.
- **LOCAL**: suas branches, em pastas divididas por `/` (`feature/`, `fix/`). Uma
  marca de seleção mostra a branch em checkout. Você pode desligar as pastas nas
  Configurações.
- **REMOTO**: cada remoto, com suas branches abaixo.
- **WORKTREES**: worktrees vinculados, quando há algum (ou quando você mostrou a
  seção). O **+** no cabeçalho cria um; dê um clique duplo em um worktree para abri-lo
  em sua própria aba e clique com o botão direito nele para Abrir em, Mesclar e Remover.
  Veja [Worktrees](worktrees.md).
- **STASHES**: seus stashes. Clique com o botão direito em um para **Pop**, **Apply** e
  **Drop**.
- **TAGS**: suas tags, em pastas separadas por `/` como as branches (`releases/`, `backups/`). Uma seta significa que a tag também está no remoto.
- **PULL REQUESTS** e **ISSUES**: depois que você conecta uma conta de hospedagem e
  vincula o repositório.
- **SUBMÓDULOS**: os submódulos e seu estado, quando há algum. Veja
  [Submódulos](submodules.md).

**Mostrar ou ocultar uma seção:** clique com o botão direito no cabeçalho de uma seção, ou na parte vazia do painel, para uma lista de marcação das seções (a última visível permanece). A escolha é mantida por repositório, entre reinicializações; veja [Seções do painel esquerdo](mouse-and-gestures.md#seções-do-painel-esquerdo).

Passe o mouse sobre uma linha para mostrar seus botões: ocultar ou mostrar a ref no Log, e **Solo**
(mostrar só os commits dessa ref). Dê um clique duplo em uma branch local ou em uma tag para fazer
checkout. Clique com o botão direito em uma linha para ver seu menu. O `‹` no título da barra lateral recolhe
a barra lateral (`Ctrl+N Shift+H`).

Os dois painéis laterais (a barra lateral Refs, `Ctrl+N Shift+H`, e o painel da direita,
`Ctrl+N Shift+L`) pertencem ao Log: mostrar ou recolher um deles enquanto um diff, uma
visualização de arquivo ou o editor está aberto vale só para essa visualização. Um diff ou uma
visualização de arquivo também recolhe sozinho a barra lateral Refs, para dar espaço ao conteúdo. De volta
ao Log, os dois painéis estão como você os deixou lá, e esse é o estado salvo para a próxima inicialização.

## O Log (centro)

![O Log: uma linha de cabeçalho de colunas, a linha de alterações em andamento no topo, uma linha de stash, pílulas de ref à esquerda do grafo](../images/log-graph.png)

Uma linha por commit, do mais novo para o mais antigo. Clique com o botão direito no cabeçalho para mostrar ou ocultar
colunas. Arraste a borda entre dois cabeçalhos para redimensionar uma coluna. Arraste um cabeçalho
para mover a coluna. As larguras pertencem ao repositório: são salvas com ele e voltam quando você o
reabre, e um repositório aberto pela primeira vez tem cada coluna ajustada ao que o Log mostra (`Ajustar todas as
colunas ao conteúdo` faz isso de novo, `Redefinir as larguras das colunas` volta às larguras automáticas, que se mantêm).

O painel não tem título: os cabeçalhos das colunas dizem o que ele é. Em um commit sem ref, a linha sob o ponteiro mostra uma pílula esmaecida com o nome da branch a que pertence; se essa branch foi excluída, uma pílula riscada mostra o nome lido no commit de merge que a mesclou (desative com **Mostrar uma branch fantasma no commit sob o mouse** em Configurações > Log e grafo). As tags são desenhadas onde estão, nunca como fantasma.

| Coluna | Mostra |
|---|---|
| **Ref** | Pílulas coloridas para as branches e tags do commit: um ícone de laptop para local, um globo para remoto, um ícone de tag para tags, `+N` quando mais refs apontam para lá. |
| **Grafo** | As faixas das branches, merges e pontos de ramificação. Desenhado como imagens no kitty, Ghostty, WezTerm, Rio e iTerm2 (e também Windows Terminal, foot e Tabby, via sixel), e como caracteres em outros terminais. As cores das faixas vêm do tema ou de uma paleta que você escolhe (veja [Temas](themes.md)). Na largura mínima (um ponto de commit) a coluna fica compacta: todos os commits em uma só faixa, sem linhas de branch nem de merge. |
| **Mensagem** | O resumo do commit. |
| **Autor** | O autor, com um avatar opcional. |
| **Data** | A data do autor, no formato que você escolher. |
| **Alterações** | Opcional: arquivos alterados e linhas `+adicionadas -removidas`. |
| **SHA** | O hash curto. |

![O menu de colunas do cabeçalho do Log com as caixas de seleção Ref, Grafo, Mensagem, Autor, Data, Alterações e SHA](../images/log-columns-menu.png)

Linhas especiais:

- **A linha de alterações em andamento** (linha WIP) no topo: um círculo tracejado,
  `// WIP` ou a mensagem do seu rascunho, e as contagens de arquivos modificados e adicionados.
  Clique nela para fazer stage e commit no painel da direita.
- **Linhas de stash**: um quadradinho em uma faixa, ao lado do commit em que o stash
  foi feito.
- Durante uma busca, as linhas que não correspondem ficam esmaecidas.

Passe o mouse sobre uma célula para ver uma dica com o texto completo. Clique com o botão direito em uma linha para ver o
[menu de commit](log-and-history.md#o-menu-do-commit).

![O menu de botão direito de um commit: ações de cherry-pick, revert, reset, editar mensagem, explicar, rebase, branch, tag, checkout, worktree, patch e copiar](../images/context-menu.png)

## O painel da direita

O painel da direita acompanha a seleção no Log.

**Alterações em andamento** (a linha WIP está selecionada):

- Um título como `4 file changes on main`. Durante um merge com conflito ele acrescenta
  o número de arquivos em conflito. Um `✗` vermelho à esquerda descarta todas as
  alterações (pergunta antes). Quando a IA está ligada, um `✦` à direita explica as
  alterações.
- **Caminho** ou **Árvore** (uma lista plana ou pastas, `m`), **Recolher tudo** e
  **Ver todos os arquivos** (`a`).
- **Arquivos unstaged** e **Arquivos staged**, cada um com uma contagem e um botão
  **Stage tudo** ou **Unstage tudo**. Passe o mouse sobre uma linha para ver seu botão
  **Stage arquivo** ou **Unstage arquivo**.
- O **formulário de commit** embaixo: **Amend do commit anterior**, **Compor
  commits com IA** (quando a IA está ligada), o **Resumo** com um contador de caracteres
  e o botão `✦`, a **Descrição** e o botão **Commit das alterações**.

**Detalhes do commit** (um commit está selecionado), de cima para baixo: o hash completo, o
autor com avatar e data, o hash do pai e as contagens de alterações, depois a
mensagem (resumo e descrição; arraste a borda inferior dela para redimensioná-la). O
painel **Arquivos alterados** abaixo começa com os seus próprios controles: o botão de ordem,
**Caminho** / **Árvore**, **Ver todos os arquivos** e uma caixa **Filtrar arquivos** (com uma
caixa **Só LFS** na ponta direita em um repositório que tem arquivos LFS), fixos acima
da lista, que é a única parte que rola. Clique em um arquivo para abrir seu diff.
Em uma janela baixa, a mensagem cede espaço primeiro (até uma linha), depois a lista.
A visão das alterações em andamento mantém a sua linha **Caminho** / **Árvore** / **Ver todos os arquivos**
no topo das listas de arquivos, acima das listas Unstaged e Staged. No modo Árvore,
uma pasta recolhida termina com o número de arquivos dentro dela, subpastas
incluídas, como um `(12)` esmaecido.

**Links em uma mensagem.** Um endereço da web digitado na mensagem (`https://...`, ou o destino
de um `[texto](https://...)` em markdown) aparece sublinhado, mostra o ponteiro de mão e, ao
clicar, pergunta **Abrir este link?** com o endereço inteiro antes de abrir qualquer coisa: o
texto de uma mensagem não é confiável. **Abrir** (`A`) inicia o navegador, **Cancelar** (`C`,
`Esc`, a cruz) não faz nada. Só endereços `http` e `https` são oferecidos. O equivalente no
teclado é o comando da paleta `Detalhes do commit: abrir link na mensagem...`, que pergunta
sobre o primeiro endereço da mensagem e passa pelos outros com **Próximo** (`P`). Alguns
terminais sublinham por conta própria um endereço sob o ponteiro e o abrem sozinhos com um
clique com modificador (geralmente Shift+clique); isso é um recurso do terminal, não do GitPoulpi.

Cada lista de arquivos tem menus em arquivos, pastas e na parte vazia, e vários arquivos podem ser selecionados de uma vez (**Ctrl+clique**, **Shift+clique**); veja [Stage e commit](staging-and-committing.md).

## O painel central quando um arquivo está aberto

Clique em um arquivo alterado e a visualização dele substitui o Log. `Esc` traz o Log
de volta. Uma barra de botões corre pelo topo:

![Um arquivo modificado aberto como diff, com a barra de visualização acima dele: Arquivo, Blame, o seletor Histórico │ Todos, Editar, navegação de hunks, Inline, Dividido, Hunks, espaços em branco, Quebra e Mapa](../images/diff.png)

| Botão | O que faz |
|---|---|
| **Arquivo**, **Diff**, **Blame** | Escolhem o que o painel mostra para o arquivo. |
| **Histórico │ Todos** | Escolhe a lista da [coluna de commits](#a-coluna-de-commits) ao lado da visualização e mostra a coluna. |
| **Editar** | Abre o editor integrado na cópia de trabalho. |
| `↑` `↓` | Hunk anterior ou seguinte. |
| **Inline**, **Dividido** | Uma coluna, ou lado a lado. |
| **Hunks** | Só os hunks alterados, ou o arquivo inteiro. |
| **EB** | Mostra ou oculta alterações só de espaços em branco. |
| **Quebra** | Quebra linhas longas. |
| **Mapa** | Um minimapa do arquivo inteiro ao lado da barra de rolagem. |
| **Codificação** | A codificação em que o arquivo é lido (**Auto** primeiro). |

#### Palavras alteradas em um diff

Quando uma linha muda só em parte, as palavras que diferem são pintadas com um
fundo mais forte da cor da própria linha (um vermelho mais forte na linha removida,
um verde mais forte na adicionada), e o resto da linha mantém o tom
chapado. Dentro de um hunk, cada sequência de linhas removidas é pareada com a sequência de linhas
adicionadas que vem depois, a primeira com a primeira, a segunda com a segunda
e assim por diante; um par só é marcado quando pelo menos 40% dos caracteres não
em branco dele são iguais, então uma linha reescrita do zero continua chapada em vez de
virar um bloco único de ênfase. Palavras, sequências de espaços e caracteres de
pontuação isolados são comparados, um espaço solitário entre duas palavras alteradas
as une em uma só marca, e linhas com mais de 1000 caracteres não são
marcadas. Funciona em Inline e Dividido, com quebra de linha e rolagem para o lado,
e no painel de diff da revisão. Desligue em **Configurações > Diff e editor >
Destacar palavras alteradas** (`ui.diff_intraline`) ou pela paleta de comandos
(**Diff: alternar destaque das palavras alteradas**, sem tecla própria). Em um terminal
limitado a 16 cores, ou com um tema sem fundo de diff, as palavras alteradas
aparecem em negrito e sublinhadas.

Em um diff da árvore de trabalho, cada hunk tem os botões **Stage hunk** e **Descartar
hunk**. Pressione `v` para selecionar linhas e então faça stage ou descarte só essas
linhas.

![Blame de um arquivo: o código com cores de sintaxe e, à direita, uma coluna de anotações por hunk com avatar, autor, idade e resumo, mais as faixas de autor e de idade](../images/blame.png)

![Histórico do arquivo: os commits do arquivo à esquerda, em duas linhas, e o diff do commit selecionado à direita](../images/file-history.png)

As duas visões estão descritas em [Blame](log-and-history.md#blame) e [Histórico do arquivo](log-and-history.md#histórico-do-arquivo).

O **editor** integrado tem cores de sintaxe, números de linha, desfazer, busca e uma
linha de status com a linha, a coluna, o fim de linha, a indentação e a linguagem.

![O editor integrado mostrando um arquivo Rust com realce de sintaxe e números de linha](../images/editor.png)

Um arquivo de imagem é mostrado como imagem: antes e depois no diff, ou o próprio
arquivo na visualização de arquivo. Veja [Imagens](image-view.md).

Um arquivo binário que não é uma imagem (um PDF, um documento, um arquivo compactado) mostra um botão **Abrir em** quando o sistema conhece aplicativos para o seu tipo: veja [Abrir em](open-in.md).

## A coluna de commits

Uma coluna pode ficar à esquerda do Diff de um arquivo de commit, da visualização de arquivo ou do Blame (nunca ao lado do editor integrado, que edita a cópia de trabalho: a coluna some quando o editor abre e volta quando ele fecha sobre o diff). Ela tem três estados:

- **Histórico**: os commits desse único arquivo, duas linhas cada um (o [Histórico do arquivo](log-and-history.md#histórico-do-arquivo)). O lado direito mostra a versão do arquivo no commit selecionado.
- **Todos**: uma lista compacta dos commits do Log com o seu grafo, descrita abaixo.
- **Oculta**: uma tira fina com um `›` à esquerda da visualização traz a coluna de volta (a dica dela indica a tecla).

A barra de botões da visualização sempre ocupa toda a largura acima da coluna, então nenhum botão se move quando a coluna abre, fecha ou troca de lista. Nela, o seletor **`Histórico │ Todos`** (dois segmentos de um só controle, sempre com a mesma largura) escolhe a lista: um clique em um segmento mostra essa lista e abre a coluna se ela estava oculta; um clique no segmento já selecionado, com a coluna visível, oculta a coluna. O segmento da lista na tela fica aceso; enquanto a coluna está oculta, o segmento com que ela vai voltar fica sublinhado e a lista é mantida. A dica dele indica o clique, o segundo clique no segmento selecionado e `Alt+C`. O Histórico do arquivo também abre pelos lugares de sempre (menus de arquivo, `Histórico do arquivo: ...` na paleta).

![A coluna de commits no estado Todos ao lado do diff de um arquivo de um commit, com um cabeçalho de Ref, Grafo, Mensagem, Autor e Data e rótulos de branch e de tag na parte Ref](../images/commit-column-all.png)

- **Mostrar ou ocultar**: `Alt+C` (paleta `Exibir: alternar a coluna de commits`), um clique no segmento selecionado do seletor, o ✕ da coluna ou a tira `›`. Na paleta, `Exibir: coluna de commits: ocultar`, `... : histórico do arquivo` e `... : todos os commits` escolhem um estado. A escolha, com a lista que ela tinha, é lembrada com as outras configurações da coluna. Sem escolha, a largura do terminal decide para a lista Todos: ela aparece a partir de 140 colunas. A coluna não é desenhada quando a visualização ao lado ficaria com menos de 30 colunas, e o grafo de Todos não é desenhado para um diff da árvore de trabalho nem para uma comparação (o histórico do arquivo aparece, pois não precisa de commit).
- **Largura**: arraste a borda direita (a borda iluminada acompanha o ponteiro), ou use `Exibir: alargar a coluna de commits` e `Exibir: estreitar a coluna de commits`. Ela pode ocupar até 70% da coluna central (o resto continua da visualização, que nunca fica com menos de 30 colunas). Ela é lembrada com os outros tamanhos.
- **Escolher um commit**: clique nele, ou dê o teclado à coluna (`Ctrl+N` e o seu selo, ou `Exibir: focar a coluna de commits`) e use `Cima` / `Baixo`, `PageUp` / `PageDown`, `Home` / `End`. `Enter` ou `Direita` vai para a lista de arquivos do commit, `Esc` devolve o teclado à visualização. O painel direito acompanha a escolha: detalhes do commit e seus arquivos alterados, para abrir outro arquivo dele.
- **O que a visualização faz**: o arquivo aberto continua aberto. Se o commit escolhido o alterou, a visualização mostra o diff dele nesse commit. Se não, mostra o arquivo como era então, com "sem alterações neste commit" no título. Se o commit não tem esse arquivo, a visualização diz isso e continua utilizável. Um Blame continua sendo Blame, até o commit escolhido.

A lista Todos fica sob um cabeçalho (os próprios nomes de colunas do Log). Um commit por linha: as faixas do grafo, o título (a mensagem), o primeiro nome do autor e uma data curta e, se você ativar, os rótulos de branch e de tag. É a própria lista do Log, com os mesmos commits na mesma ordem, e ela carrega os mais antigos ao rolar. As faixas são desenhadas como na coluna Grafo do Log: em um terminal que mostra imagens (veja [Escolhendo um terminal](terminals.md)), como faixas conectadas com marcadores redondos e, com a configuração de avatares em Grafo, as imagens dos autores dentro dos círculos; em outros terminais, como caracteres. Por padrão, as faixas ocupam o espaço de que precisam, e o título sempre mantém algumas palavras: em uma coluna estreita, o autor e depois a data cedem lugar antes que as faixas sejam cortadas. Reduzida à largura mínima, um ponto de commit, a parte Grafo fica compacta, como no Log: todos os commits em uma só faixa, sem linhas de branch nem de merge. Uma parte cortada pelo painel (a linha rolada para os lados, ou uma parte Grafo mais larga que o espaço) mantém as imagens e os avatares, recortados na borda. A roda rola a lista sob o ponteiro.

- **Dicas e o menu**: deixar o ponteiro sobre uma linha mostra a dica do Log para a parte sob ele (a mensagem inteira no título, o nome e o e-mail do autor, a data completa). Um clique direito em uma linha (ou a tecla de menu, `Shift+F10`, quando a coluna tem o teclado; `Exibir: coluna de commits: menu da linha selecionada` na paleta) abre o menu dela: *Mostrar no Log*, o diff do arquivo nesse commit, o arquivo como era nesse commit, o blame do arquivo nesse commit e depois o próprio menu do Log para esse commit (copiar, cherry-pick, branch e, abaixo de um separador, as linhas que reescrevem coisas, como revert). Cada uma das quatro primeiras linhas também tem um comando na paleta (`Exibir: coluna de commits: mostrar ...`). Na lista Histórico, as linhas têm o próprio menu do Histórico do arquivo e as mesmas dicas.
- **Dimensionar as partes e rolar para os lados**: arraste um limite do cabeçalho para dimensionar a parte à esquerda dele: as faixas, a mensagem, o autor, a data, as refs, as alterações ou o SHA. A última parte também tem um limite: a célula logo depois dela (a própria borda da coluna quando a linha preenche o espaço), iluminada do mesmo modo, que dimensiona essa última parte. Como no Log (`ui.column_resize`), o limite é por padrão a célula separadora entre duas partes, onde termina a cor de fundo da linha do Grafo (a cor a atravessa, então nenhuma célula simples fica entre ela e a parte seguinte): uma marca fica nela no cabeçalho e uma linha fina é desenhada ao longo da coluna enquanto o ponteiro está sobre ela ou a arrasta, e um arrasto a partir de qualquer ponto da linha redimensiona; com `header`, é a primeira letra do nome seguinte, que se ilumina enquanto o ponteiro está sobre ela (nenhuma linha é desenhada, o cabeçalho é o único lugar de onde arrastar). Enquanto você não dimensiona nada, a mensagem fica com o que os outros deixam; o primeiro arrasto fixa cada parte na largura que ela tem, a mensagem inclusive, de modo que dimensionar uma parte move as partes seguintes e não altera nenhuma outra largura. Uma parte que você dimensionou é mantida e nunca cede espaço: quando a linha fica mais larga que a coluna, ela rola para os lados com Shift+roda, uma roda horizontal ou `Shift+Left` / `Shift+Right` quando a coluna tem o teclado, e uma barra de rolagem na borda inferior mostra onde você está. Pelo teclado: `Tab` escolhe a parte (entre as exibidas), `]` a alarga, `[` a estreita, `0` devolve todas as partes à largura automática (a paleta tem `Exibir: coluna de commits: dimensionar a próxima parte`, `... alargar a parte`, `... estreitar a parte`, `... redefinir as larguras das partes`, `... rolar para a esquerda` e `... rolar para a direita`). As larguras são lembradas. Enquanto as faixas estão cortadas por uma rolagem, elas são desenhadas como caracteres, mesmo em um terminal que mostra imagens.
- **Escolher as partes**: um clique direito no cabeçalho abre um menu com uma caixa de seleção por parte, todas as colunas que o Log oferece: Ref, Grafo, Mensagem, Autor, Data, Alterações e SHA. A Mensagem está sempre visível. **Ref**, **Alterações** e **SHA** vêm desativadas por padrão; exibidas, Ref traz os rótulos de branch e de tag do commit, Alterações seus arquivos e as linhas `+adicionadas -removidas`, e SHA o hash curto, tudo como as colunas do próprio Log. Como Ref, Alterações e SHA nunca cedem espaço: a linha rola para os lados. Pelo teclado, `Exibir: coluna de commits: escolher as colunas` abre o mesmo menu (setas e `Enter` escolhem uma linha) e `Exibir: coluna de commits: alternar Ref`, `... alternar Grafo`, `... alternar Autor`, `... alternar Data`, `... alternar Alterações` e `... alternar SHA` alternam uma parte na hora. As partes exibidas, sua ordem e suas larguras pertencem só à coluna: são gravadas no `tui.toml` (`ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths`), à parte das colunas do próprio Log, que nunca mudam com elas, nem o contrário.

  ![O menu de clique direito do cabeçalho da coluna de commits: uma caixa de seleção para Ref, Grafo, Mensagem (sempre visível), Autor, Data, Alterações e SHA](../images/commit-column-menu.png)
- **Mover as partes**: arraste um nome do cabeçalho sobre outro para mover essa parte até lá, como no Log (a linha acompanha o ponteiro; com `ui.column_resize = "header"` a primeira letra de um nome é o limite de redimensionamento, então pressione no resto do nome). Qualquer parte pode ir para qualquer lugar, as faixas e os rótulos de Ref inclusive. Pelo teclado, `Tab` escolhe a parte e `<` / `>` a movem um lugar para a esquerda ou para a direita entre as exibidas (a paleta tem `Exibir: coluna de commits: mover a parte para a esquerda` e `... mover a parte para a direita`). A ordem é lembrada.

Os gestos do mouse também estão listados em [Mouse e gestos](mouse-and-gestures.md#a-coluna-de-commits-lista-todos).

## Os tamanhos são lembrados

O GitPoulpi guarda todo tamanho que você arrasta, entre trocas de aba e reinicializações. Os
tamanhos são os mesmos para todos os repositórios. Isso vale para os divisores entre a
barra lateral Refs, o Log e o painel da direita, a altura de cada seção de Refs,
o bloco da mensagem de commit, as larguras das colunas do Log, os painéis do editor de merge e
o divisor do seletor de pastas. Os tamanhos são guardados como partes da janela, então um
layout feito em um terminal largo também cabe em um estreito.

Para redefinir todos os tamanhos, execute o comando da paleta `Navegação: redefinir o layout`. Alguns
tamanhos também têm sua própria redefinição: `Navegação: redefinir os tamanhos das seções Refs`,
`Detalhes do commit: redefinir o tamanho da mensagem` e, no editor de merge, um clique duplo
em um divisor ou `Alt+0`.

## Sobreposições e telas especiais

- **Paleta de comandos** (`Ctrl+P`): uma caixa de filtro sobre todos os comandos. Cada linha
  mostra seu atalho. Um comando que não pode rodar agora fica acinzentado, e o motivo
  aparece embaixo. Os comandos têm um prefixo de grupo como `Git:`,
  `Refs:`, `Navegação:`, `Diff:`, `Alterações:`, `Commit:`, `Exibir:`,
  `Repositório:`, `Aba:`, `LFS:`, `Pull requests:`, `Issues:` ou `IA:`. Digite
  um prefixo para ver o grupo dele.
- **Seletores**: alguns comandos da paleta abrem uma segunda lista, por exemplo
  `App: mudar tema...`, `App: mudar idioma...` e
  `App: mudar tema de cores do grafo...`. A janela mostra cada escolha ao vivo
  enquanto você percorre a lista. `Enter` a mantém. `Esc` devolve a escolha
  antiga. A lista de temas tem os filtros **Todos**, **Escuros**, **Claros** e
  **Coloridos**. Veja [Temas](themes.md).
- **Ajuda** (`?`): abas. **Teclas** lista cada tecla e gesto; as outras abas (Início, Log, Alterações, Refs, Hospedagem, Config., IA e CLI) são guias curtos das ações, com a forma de chegar a cada uma. Troque de aba com `Tab` / `Shift+Tab`, `Left` / `Right`, `Alt+1`..`Alt+8` ou um clique. Digite para filtrar a aba atual. `Ctrl+D`, ou um clique no link embaixo, abre a página de documentação da aba em que você está.

  ![A aba Hospedagem da ajuda: um guia curto de como chegar a cada ação](../images/help-hosting.png)

  ![A sobreposição de ajuda: a seção Global listando Ctrl+P, Ctrl+N, Ctrl+G, ? e as teclas de movimento, depois a seção Noções básicas do mouse](../images/help.png)
- **Barra de busca** (`f`): no canto superior direito do Log. Mostra `n de N`
  correspondências.
- **Menus de contexto**: botão direito, ou pressione `.` na linha selecionada. Uma linha
  com `›` abre um submenu.
- **Diálogos**: confirmações, um nome de branch ou tag, Clonar, Init, Criar pull
  request, as opções depois de um push rejeitado e outros. `Enter` aceita,
  `Esc` cancela.
- O **rebase interativo** e o **compositor de commits** substituem o Log por uma tela de
  revisão. Veja
  [Trabalhando com o Log](log-and-history.md#rebase-interativo) e
  [Stage e commit](staging-and-committing.md#o-compositor-de-commits-divida-suas-alterações-em-commits).

  ![O compositor de commits revisando quatro commits propostos, cada um com seus arquivos, e os botões Redefinir, Cancelar e Criar 4 commits](../images/composer.png)

- **Configurações** (`Ctrl+,` ou a engrenagem na barra de título): uma lista de páginas à esquerda e as
  opções da página à direita. Veja [Configurações](settings.md).

  ![As Configurações: a lista de páginas à esquerda, com uma caixa de busca acima, e a página Geral à direita](../images/settings.png)

- **Popups** para respostas longas, como o popup **Explicar alterações** da IA
  (`↑` `↓` rolam, `c` copia, `Esc` fecha).

  ![O popup Explicar as alterações em andamento com uma explicação da IA e uma lista de pontos a conferir](../images/ai-explain.png)

Todas as listas nos popups funcionam do mesmo jeito. Veja
[Atalhos de teclado](keybindings.md) e [Mouse e gestos](mouse-and-gestures.md).

## Palavras usadas nesta documentação

| Palavra | Significado |
|---|---|
| Painel | Uma das três colunas: barra lateral Refs, Log (ou o arquivo aberto), painel da direita. |
| Pílula | Um rótulo de branch ou tag na coluna Ref. |
| Linha WIP | A linha de alterações em andamento no topo do Log. |
| Hunk | Um bloco de linhas alteradas em um diff. |
| Compositor | O formulário de commit, e a revisão da IA que divide as alterações em commits. |
| Acorde | Uma tecla seguida de outra, como `Ctrl+G` e depois `s`. |
| Prefixo | `Ctrl+G` (ações do git) ou `Ctrl+N` (navegação). |
| Selo | A letra que `Ctrl+N` mostra ao lado de um controle. Pressione-a para focar esse controle. |
