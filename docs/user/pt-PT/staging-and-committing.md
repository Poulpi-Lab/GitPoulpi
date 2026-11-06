[English](../staging-and-committing.md) · [Čeština](../cs/staging-and-committing.md) · [Deutsch](../de/staging-and-committing.md) · [Español](../es/staging-and-committing.md) · [Français](../fr/staging-and-committing.md) · [Italiano](../it/staging-and-committing.md) · [日本語](../ja/staging-and-committing.md) · [한국어](../ko/staging-and-committing.md) · [Nederlands](../nl/staging-and-committing.md) · [Polski](../pl/staging-and-committing.md) · [Português (Brasil)](../pt-BR/staging-and-committing.md) · **Português (Portugal)** · [Русский](../ru/staging-and-committing.md) · [Svenska](../sv/staging-and-committing.md) · [Türkçe](../tr/staging-and-committing.md) · [Українська](../uk/staging-and-committing.md) · [简体中文](../zh-CN/staging-and-committing.md)

# Stage e commit

Como ver o que alterou, escolher exatamente o que entra num commit, escrever o
commit e olhar para os ficheiros com mais profundidade (blame, histórico, o
editor integrado). Termos como *linha WIP* e *hunk* são explicados na
[visita guiada à interface](interface-tour.md).

## As alterações em curso

O trabalho por commitar é a **primeira linha do Log** (círculo tracejado,
`// WIP`, com os contadores `✎ n` modificados e `+ n` adicionados). Selecione-a
e o painel direito mostra as alterações em curso:

![As alterações em curso: ficheiros Unstaged e Staged à direita, com o formulário de commit por baixo](../images/hero.png)

- **Ficheiros unstaged**: ficheiros editados, adicionados (não acompanhados) e
  eliminados que ainda não estão no próximo commit.
- **Ficheiros staged**: o que o próximo commit vai conter.
- **Caminho** ou **Árvore** (`m`) lista os ficheiros em plano ou em pastas.
  **Recolher tudo** (`z`) recolhe as pastas. **Ver todos os ficheiros** (`a`)
  mostra a árvore inteira em vez de apenas os ficheiros alterados. Clique com o botão direito no espaço vazio da lista para
  **Expandir tudo** / **Recolher tudo**, e numa linha de pasta para **Expandir todas as subpastas** /
  **Recolher todas as subpastas**; teclas `=`, `-`, `*`, `_` (ver
  [Rato e gestos](mouse-and-gestures.md)).
- O título diz quantos ficheiros mudaram e em que branch.

### Stage, unstage, descartar

| Para | Faça |
|---|---|
| Fazer stage de um ficheiro | Passe o ponteiro pela sua linha e clique em **Stage ficheiro**, ou selecione-o e prima `Ctrl+Space`, ou `Ctrl+G s` |
| Fazer stage de uma pasta | **Stage pasta** na linha da pasta |
| Fazer stage de tudo | **Stage de todas as alterações** no título da secção, ou `Ctrl+G a` |
| Unstage | Os mesmos botões e teclas na secção Staged (`Ctrl+G u`) |
| Descartar as alterações de um ficheiro | Clique com o botão direito no ficheiro > **Descartar alterações...** (pergunta primeiro; `Ctrl+Z` recupera as alterações) |

### Ficheiros sensíveis

Fazer stage de um ficheiro, de uma pasta ou de tudo verifica primeiro o que isso acrescentaria. Um ficheiro cujo
nome corresponda a `[privacy] sensitive_patterns` (`.env` e `.env.*` exceto os
`.example`, `.sample` e `.template`, `*.pem`, `*.key`, chaves privadas SSH,
`*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`), ou
cujo texto contenha um segredo que o analisador reconheça, não entra em stage. A pergunta
**Ficheiros sensíveis** lista cada ficheiro com o seu motivo (um padrão, ou o tipo de segredo
com uma pré-visualização mascarada, nunca o valor) e oferece **1. Stage mesmo assim** ou **2. Cancelar,
não stage nada**. Nenhuma resposta fica realçada até premir uma seta ou um dígito,
pelo que um Enter perdido não responde a nada; `Esc`, a cruz de fechar e um clique fora cancelam.
É a mesma pergunta a partir do teclado, de um botão, de um menu, da paleta ou de um
arrastar e largar, porque todos enviam o mesmo pedido. Cancelar não coloca nada em stage,
nem sequer os ficheiros normais do mesmo stage.

Os ficheiros de texto são lidos até 1 MiB cada e 32 MiB no total; os ficheiros maiores e os
binários são apenas avaliados pelo nome. `[privacy] staging_guard = false` desativa a verificação.
Fazer stage de um hunk ou de algumas linhas não é verificado: já está a trabalhar nesse ficheiro.

Clique com o botão direito num ficheiro para ver o seu menu: stage ou unstage,
descartar, copiar o caminho, histórico e blame do ficheiro, abrir no seu editor
externo, ferramenta de diff ou gestor de ficheiros (definidos em
[Definições > Diff e editor](settings.md#diff-e-editor)), e mais.

## Ignorar ficheiros e tratá-los como binários ou como texto

Clique com o botão direito num ficheiro > **Ignorar** para o ignorar ou ignorar a sua extensão, ou numa linha de pasta > **Ignorar todos os ficheiros da pasta**. O GitPoulpi pergunta primeiro em que ficheiro vai a regra: o `.gitignore` da raiz do repositório (predefinição), o `.gitignore` de cada pasta até à do ficheiro, ou `.git/info/exclude` (apenas este clone, nunca versionado). Os ficheiros que ainda não existem são marcados como *novo* e criados ao confirmar. A linha é escrita relativa ao ficheiro escolhido: `src/a/b.txt` passa a `/a/b.txt` em `src/.gitignore`. Mova-se com as setas, escreva para filtrar a lista, `Enter` escreve, `Esc` cancela. `Ctrl+Z` desfaz uma escrita num `.gitignore`; `.git/info/exclude` está fora da árvore de trabalho e não pode ser desfeito.

![A lista que pergunta em que ficheiro vai uma regra de ignorar: o .gitignore do repositório, o .gitignore da pasta do ficheiro ou .git/info/exclude, cada um com a linha que seria escrita](../images/ignore-picker.png)

Clique com o botão direito num ficheiro > **Tratar como binário** (ou **Tratar como texto** quando o git já o trata como binário) escreve uma regra de `.gitattributes`: `binary` ou `text diff`. Escolha o próprio ficheiro ou todos os ficheiros com a sua extensão e depois o `.gitattributes` onde escrever, com a mesma lista acima (e `.git/info/attributes` apenas para este clone). Uma regra já escrita para o mesmo padrão é substituída, nunca contradita. O diff do ficheiro atualiza-se de imediato e a barra de estado diz o que foi escrito e onde (e avisa se outra regra ainda prevalece). Uma linha de pasta tem **Tratar os ficheiros da pasta como** > **Binário** ou **Texto**. O mesmo existe na paleta de comandos (**Alterações: tratar o ficheiro selecionado como binário...**) e na linha de comandos (`gitpoulpi treat binary|text <caminhos>`). A vista de ficheiro também segue a regra: um ficheiro tratado como texto mostra as suas linhas mesmo com um byte NUL.

## Menus, seleção múltipla e conversão nas listas de ficheiros

As listas de ficheiros do painel direito (Unstaged, Staged, os ficheiros alterados de um commit e **Ver todos os ficheiros**) têm os mesmos menus e gestos. Clique com o botão direito num ficheiro, numa linha de pasta ou na parte vazia de uma lista, ou prima `.` (`Shift+F10`, a tecla Menu) na linha com foco, ou execute **Ficheiros: abrir o menu da linha ou da seleção** na paleta.

- **Um ficheiro de um commit** (e um ficheiro de **Ver todos os ficheiros**): **Abrir diff**, **Abrir ficheiro**, **Histórico do ficheiro**, **Blame do ficheiro**, abrir no seu editor, no programa predefinido ou em **Abrir com...**, **Mostrar na pasta**, **Editar ficheiro**, **Copiar caminho do ficheiro**, **Copiar o caminho completo do ficheiro**, **Ignorar**, **Tratar como binário** ou **Tratar como texto**, **Converter**, **Restaurar o ficheiro a partir deste commit** (pergunta antes; como `git checkout <commit> -- <ficheiro>`, também faz stage do ficheiro, e `Ctrl+Z` repõe o ficheiro anterior) e **Eliminar ficheiro...**. Uma linha sobre a cópia de trabalho fica a cinzento, com o motivo, quando o ficheiro não está na árvore de trabalho. **Diff com o diretório de trabalho** abre o diff da versão do commit contra o ficheiro em disco. **Criar patch das alterações do ficheiro...** grava um patch da alteração desse ficheiro no commit (uma seleção e uma linha de pasta também o têm; em **Ver todos os ficheiros** nas alterações em curso, cria o patch das alterações do ficheiro).
- **Uma linha de pasta**: **Abrir pasta**, **Copiar caminho do ficheiro**, **Ignorar todos os ficheiros da pasta**, **Tratar os ficheiros da pasta como**, **Converter**, **Criar um ficheiro nesta pasta**.
- **A parte vazia de uma lista**: **Criar ficheiro** e **Abrir a pasta do repositório**. A lista Unstaged acrescenta **Stage tudo** e **Descartar todas as alterações...**, a lista Staged **Unstage tudo**.

![O menu de contexto de um ficheiro de um commit: abrir o diff, o histórico e o blame do ficheiro, abrir num editor ou no programa predefinido, copiar caminhos, ignorar, converter, restaurar a partir do commit e eliminar](../images/commit-file-menu.png)

### Selecionar vários ficheiros

Os gestos são os do Log. **Ctrl+clique** adiciona uma linha à seleção ou remove-a. **Shift+clique** (**Alt+clique** nos terminais que ficam com o Shift+clique) seleciona o intervalo a partir da âncora. Um clique simples recomeça e abre o diff como antes. `Esc` limpa a seleção. No teclado, `Space` ou `Insert` alterna a linha atual em todas as listas (em Unstaged e Staged `Ctrl+Space` faz stage da seleção, ou da linha atual) e `Shift+Cima` / `Shift+Baixo` alargam o intervalo. Uma linha de pasta seleciona todos os ficheiros que contém, recolhida ou não. As linhas selecionadas ficam realçadas e a barra de estado conta-as. `Ctrl+A` seleciona todas as linhas da lista em foco.

Clique com o botão direito numa linha selecionada (ou prima `.`) para o menu de toda a seleção, intitulado **N ficheiros**: stage ou unstage, descartar, ignorar, tratar como binário ou texto, converter, copiar os caminhos, restaurar do commit, eliminar. Stage, unstage e restaurar são uma ação cada; descartar e eliminar perguntam antes e indicam quantos ficheiros. Um clique com o botão direito numa linha fora da seleção recomeça a seleção com essa linha.

![Quatro ficheiros selecionados na lista de não preparados, com o menu intitulado 4 ficheiros: preparar, descartar, ignorar, tratar como binário ou texto, converter, copiar os caminhos e eliminar](../images/file-pane-multiselect.png)

### Converter codificação, fins de linha e indentação

**Converter** está no menu de um ficheiro, de uma pasta ou de uma seleção. `gitpoulpi convert` faz o mesmo na linha de comandos.

- **Codificação**: UTF-8, UTF-8 BOM, UTF-16 LE e BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR.
- **Fins de linha**: LF, CRLF ou CR.
- **Indentação**: espaços ou tabulações no início da linha, com a largura de uma tabulação.

Os ficheiros da árvore de trabalho são reescritos em segundo plano com as mesmas funções da barra de estado do editor. Um ficheiro que não é UTF-8 válido e não tem marca de ordem de bytes é lido como Windows-1252, como no editor. Os ficheiros binários, em falta ou que a codificação de destino não consegue representar são ignorados e comunicados (*12 ficheiros convertidos, 3 ignorado(s) (3 binário(s))*), nunca danificados; um ficheiro que não pode ser gravado gera uma notificação de falha. O lote inteiro é uma única entrada do diário do desfazer: um `Ctrl+Z` repõe todos os ficheiros. Um ficheiro aberto no editor bloqueia a conversão até ser fechado.

## Ler um diff

Clique no nome de um ficheiro (ou prima `d` sobre ele) e o seu diff substitui o
Log ao centro. `Esc` ou `q` devolve o Log.

![Um diff com dois hunks, cada um com os botões Descartar hunk e Stage hunk](../images/diff.png)

A barra por cima muda a vista:

- **Inline** mostra o diff unificado; **Dividido** põe o antes e o depois lado a
  lado (`m` percorre os layouts).
- **Hunks** (`Shift+H`) mostra apenas os hunks alterados com uma linha de
  contexto. Desligue-o para ver o ficheiro todo.
- **EB** (`s`) oculta as alterações que diferem apenas em espaços em branco.
- **Quebra** (`w`) quebra as linhas compridas em vez de as truncar.
- **Mapa** (`Shift+M`) mostra um minimapa do ficheiro todo junto à barra de
  deslocação. Clique nele para saltar. As outras ocorrências de uma palavra
  selecionada são tingidas, e marcadas na barra de deslocação.
- `n` e `p` (ou as setas da barra) saltam entre hunks.
- **Auto** (o último botão, `Shift+C`) é a codificação com que o ficheiro é lido. Clique nele (ou prima
  `Shift+C`, ou execute *Diff: escolher codificação...* na paleta) e escolha uma da lista:
  **Deteção automática** primeiro, depois UTF-8, UTF-8 BOM, UTF-16 LE e BE, Windows-1252, ISO-8859-1,
  ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5 e EUC-KR (escreva para
  pesquisar a lista). Os dois lados do ficheiro são lidos de novo, como bytes exatos, nessa codificação, pelo que um
  ficheiro Latin-1 mostra os seus acentos e um ficheiro UTF-16 (que o git considera binário) mostra as suas linhas. A
  escolha é lembrada para o caminho desse ficheiro até sair, nesta vista e na
  [revisão](review.md). Enquanto há uma codificação escolhida os hunks são os desta vista, pelo que preparar
  e descartar estão desativados (escolha **Deteção automática** para voltar a preparar). Uma leitura lenta diz *a ler como ...* no
  título passado um momento.

![O mesmo diff no layout dividido, antes e depois lado a lado](../images/diff-split.png)

Pode selecionar texto com o rato (arrastar, ou clique duplo numa palavra) e
copiá-lo com `y` ou `Ctrl+C`; só o texto do ficheiro é copiado, nunca números de
linha nem marcadores `+` e `-`.

### Fazer stage de um hunk ou de uma só linha

Num diff da árvore de trabalho, o cabeçalho de cada hunk tem **Stage hunk** e
**Descartar hunk** (do lado Staged, **Unstage hunk**).

- `Space` faz stage do hunk sob o cursor.
- Para fazer stage de algumas linhas, prima `v` para iniciar uma seleção, mova-se
  com `j`/`k` e prima `Enter`. Só contam as linhas *alteradas*.
- `x` descarta o hunk e `Shift+X` as linhas selecionadas. Ambos perguntam
  primeiro, e `Ctrl+Z` recupera as alterações.
- Clique com o botão direito (ou `.`) num hunk para ver o seu menu: stage,
  unstage, descartar, histórico do ficheiro, blame, copiar.

O diff de um commit é só de leitura. Uma imagem alterada aparece como imagem;
veja [Imagens](image-view.md).

## Escrever o commit

O formulário de commit fica na parte de baixo do painel direito, e o seu
**Resumo** é também o campo da linha das alterações em curso do Log: escrever
num aparece no outro.

![O formulário de commit preenchido com uma mensagem escrita por IA: resumo com um contador de carateres, descrição, e o botão verde de commit](../images/ai-commit-message.png)

- **Resumo** (o contador mostra quantos dos 72 carateres restam) e
  **Descrição**. `Enter` no resumo passa à descrição, `Enter` aí acrescenta uma
  linha.
- **Commit das alterações em N ficheiros** (o botão verde) ou `Ctrl+Enter` faz commit das
  alterações staged. Se nada estiver staged, o botão di-lo e não faz nada.
- **Amend do commit anterior** (`Shift+A`) funde as alterações staged e a nova
  mensagem no último commit. Um commit que já foi enviado por push precisará
  então de um force push (com lease).
- **Ignorar hooks** (`Shift+H`, ou **Commit: alternar ignorar hooks** na paleta) faz o commit
  sem executar os hooks `pre-commit` e `commit-msg` do repositório. Vale por commit;
  **Definições > Definições do repositório > Commit** faz com que comece
  assinalado. Aparece quando a linha tem espaço.
- `Ctrl+G c` ou `c` no painel Status leva o cursor para o formulário. `Esc`
  sai dele.
- `Ctrl+G Enter` (**Git: fazer commit das alterações em stage agora** na paleta) é o
  botão de commit a partir de qualquer lugar: faz commit das alterações em stage com a
  mensagem do formulário, respeitando **Amend do commit anterior** e **Ignorar hooks**,
  sem mover o foco. Sem nada em stage, ou sem resumo, di-lo e põe o cursor no resumo.
  Enquanto uma mensagem de IA ainda está a ser escrita, não faz commit de nada e uma
  mensagem na barra de estado di-lo (`Esc` cancela o pedido de IA). `Ctrl+Z` desfaz o
  commit como qualquer outro.
- Por repositório, as [Definições](settings.md#específico-do-repositório-commit) podem fazer **push após
  cada commit**, iniciar o formulário com um **modelo de commit** (e remover os seus
  comentários `#` da mensagem) e fazer um merge **squash**: o formulário
  abre então com a mensagem de squash do git.
- Com uma integração de alojamento ligada, `Alt+R` no formulário insere uma
  referência de issue; veja
  [Alojamento e gestores de issues](hosting-integration.md).

Com a [IA](ai.md) ligada, o botão **`✦`** à direita do resumo escreve uma
mensagem a partir das alterações staged (`Ctrl+G i`), e **Compor commits com
IA** divide o seu trabalho em vários commits, abaixo. O texto vai para o
formulário para o editar; nada é commitado até premir o botão de commit.

## O compositor de commits: dividir as alterações em commits

*(Precisa da IA ligada: veja [Assistência de IA](ai.md).)* Prima **Compor
commits com IA** (ou `Ctrl+G C`, ou clique com o botão direito na linha das
alterações em curso). O modelo propõe vários commits, cada um com os seus
ficheiros; cada ficheiro alterado está em exatamente um commit, e um plano
inválido é devolvido uma vez ao modelo. A proposta abre-se como uma revisão em
que nada foi ainda commitado:

![O compositor de commits: quatro commits propostos, cada um expansível até aos seus ficheiros, com controlos de subir, descer, editar e drop, e os botões Repor, Cancelar e Criar 4 commits](../images/composer.png)

| Para | Faça |
|---|---|
| Expandir ou recolher um commit | `Left` / `Right`, ou clique na seta; `*` alterna todos |
| Subir ou descer um commit | Os `▲` `▼` na sua linha, ou `Alt+Up` / `Alt+Down` (`K` / `J`) |
| Mover um ficheiro para outro commit | Arraste-o para o commit (da lista, ou da lista de alterações em curso à direita); ou `m` no ficheiro e escolha o commit na lista; ou `Alt+Up` / `Alt+Down` no ficheiro |
| Pôr um ficheiro num commit só seu | `n` no ficheiro, ou o botão `+ commit próprio` |
| Reformular um commit | O lápis, `e`, `Enter` ou um clique duplo: edite a mensagem, `Enter` mantém, `Esc` descarta |
| Deixar um commit de fora | `drop` (`d` ou `Delete`); os seus ficheiros ficam por commitar. `restaurar` repõe-no |
| Recomeçar | **Repor** (`r`) volta ao plano do modelo |
| Criá-los | **Criar N commits** (`Ctrl+S`, `Ctrl+Enter` ou `a`) |
| Sair | **Cancelar** ou `Esc`: nada feito |

Criar os commits faz unstage de tudo e depois, para cada commit, faz stage dos
seus ficheiros e commit, pelo que a sua árvore de trabalho nunca é tocada. Se um
passo falhar, pára e diz quantos commits foram feitos. São feitos stage de
ficheiros inteiros: um ficheiro que estava staged só em parte é commitado
inteiro. Escolha que perfil e instruções o compositor usa em *Definições > IA*;
veja [Assistência de IA](ai.md#o-compositor-de-commits).

## Vista de ficheiro, Blame e Histórico do ficheiro

A barra de vista tem **Ficheiro** (o conteúdo do ficheiro), **Diff**, **Blame** e o
seletor `Histórico │ Todos` da [coluna de commits](interface-tour.md#a-coluna-de-commits) (a sua
lista `Histórico` é o Histórico do ficheiro); os mesmos estão no menu de clique com o botão direito
de um ficheiro. Mostram o ficheiro tal como estava no commit de onde veio, ou
como por commitar (`@ WIP`) quando o abriu a partir das alterações em curso.

- **Blame** e o **Histórico do ficheiro** são vistas completas por si: veja [Blame](log-and-history.md#blame) e [Histórico do ficheiro](log-and-history.md#histórico-do-ficheiro) em *Trabalhar com o Log*. `Enter` num hunk do blame ou numa linha do histórico vai a esse commit no Log.
- `y` ou `Ctrl+C` copia o texto selecionado (no Blame, apenas o código).
- `Shift+G` salta para a última linha. `q` fecha a vista.

## O editor integrado

**Editar** (`e`) abre a *cópia da pasta de trabalho* do ficheiro num pequeno
editor sobre o painel central: cores de sintaxe, números de linha, desfazer e
refazer, pesquisa e uma barra de estado (posição, fim de linha, indentação,
linguagem). Edita sempre a cópia de trabalho, mesmo quando o abriu a partir do
diff de um commit antigo.

![O editor a mostrar um ficheiro Rust com realce de sintaxe e números de linha; a barra tem Guardar, Desfazer, Refazer e Pesquisar](../images/editor.png)

| Tecla | Ação |
|---|---|
| `Ctrl+S` | Guardar (o ficheiro é escrito num ficheiro temporário e renomeado, pelo que nunca fica meio escrito; os fins de linha, o BOM e a nova linha final são preservados) |
| `Ctrl+Z`, `Ctrl+Y` | Desfazer, refazer |
| `Ctrl+F`, `F3` | Pesquisar, pesquisar seguinte |
| `Tab`, `Shift+Tab` | Indentar, desindentar (seguindo o estilo do próprio ficheiro) |
| `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V` | Selecionar tudo, copiar, cortar, colar |
| `Esc` | Fechar. Com alterações por guardar pergunta **Guardar** (`s`), **Descartar** (`d`) ou **Cancelar** (`c`) |

Os ficheiros binários são recusados. O
tamanho da tabulação, os fins de linha, o realce de sintaxe, os números de linha
e a quebra de linha estão em *Definições > Diff e editor*; veja
[Definições](settings.md#diff-e-editor).

**Seguir a seleção**: Com o editor aberto, selecionar outro ficheiro no painel direito também o abre no editor, depois de perguntar se as alterações por guardar devem ser guardadas. Um ficheiro que não pode ser editado abre na vista habitual. Escolher um ficheiro de um commit ou de uma comparação sai do editor para o diff desse ficheiro, perguntando antes **Guardar**, **Descartar** ou **Cancelar** se houver edições por guardar.

A barra de estado, à direita, mostra a **codificação**, as **quebras de linha** e a
**indentação** do ficheiro, depois a posição do cursor e a sintaxe. Os três primeiros são
botões: cada um abre uma lista pendente (setas ou roda, escreva para filtrar, `Enter`
para escolher, `Esc` para fechar). A paleta (`Ctrl+P`) abre os mesmos menus: *Editor: alterar codificação*, *Editor: reabrir com codificação*, *Editor: converter para codificação*, *Editor: Converter quebras de linha*, *Editor: alterar indentação*.

![O menu de codificação do editor aberto sob o botão UTF-8 da barra: reabrir o ficheiro com outra codificação ou convertê-lo](../images/editor-encoding-menu.png)

![O menu de fim de linha do editor aberto sob o botão LF: converter o ficheiro para LF, CRLF ou CR](../images/editor-eol-menu.png)

- **Codificação.** Um ficheiro abre na codificação que anuncia (um BOM: UTF-8 ou
  UTF-16), senão em UTF-8, senão em Windows-1252, e a barra diz qual é. **Reabrir com
  codificação** lê os bytes do ficheiro de novo com outra codificação (pergunta antes se
  houver alterações por guardar): use-o quando a deteção falhou. **Converter para
  codificação** mantém o texto e muda a codificação em que o ficheiro é guardado (UTF-8,
  UTF-8 BOM, UTF-16 LE e BE, Windows-1252, ISO-8859-1 e -15, Windows-1250, Windows-1251,
  KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR). Nada é escrito até guardar, e Desfazer
  reverte. Se o texto tem carateres que o destino não suporta, a barra de estado diz
  quantos e onde está o primeiro, e nada é convertido; guardar recusa da mesma forma,
  nunca escreve `?`.
- **Quebras de linha.** Mostra `LF`, `CRLF`, `CR` ou `Misto` quando o ficheiro mistura
  vários tipos. O menu converte todas as quebras de linha do ficheiro para LF, CRLF ou CR
  num único passo de desfazer. As linhas novas que escrever usam a quebra do próprio
  ficheiro. A sua escolha prevalece sobre a definição *Quebras de linha*.
- **Indentação.** Mostra `Espaços: 4` ou `Tabulações: 4`. O menu escolhe o que `Tab` e a
  indentação automática inserem (espaços ou tabulações), o tamanho (de 1 a 8: o passo da
  indentação e a largura de uma tabulação neste ficheiro), e converte o espaço em branco
  inicial de cada linha em espaços ou em tabulações, num único passo de desfazer.

## Stash, patches e Git LFS

- **Stash** e **Pop** estão na barra de ferramentas; veja
  [Trabalhar com o Log](log-and-history.md#branches-tags-e-a-barra-lateral-refs).
- **Criar patch** e **Aplicar patch** estão nos menus do commit e da ref; a CLI
  tem `gitpoulpi patch`.
- Os ficheiros grandes guardados com Git LFS aparecem como pequenos ficheiros
  ponteiro até fazer pull do seu conteúdo; veja [Git LFS](lfs.md).

## A partir da linha de comandos

Tudo nesta página pode ser automatizado:

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi stage --all --allow-sensitive   # depois da recusa abaixo
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

`gitpoulpi stage` recusa um ficheiro sensível da mesma forma: nada entra em stage, o código
de saída é 2 e `--json` mostra `{error: "sensitive_files", files: [{path, reason}]}`
(uma regra e uma pré-visualização mascarada, nunca um valor). `--allow-sensitive` é a nova tentativa. Um
agente MCP é recusado com a mesma resposta e não pode passar `allow_sensitive`, a menos que
`[mcp] allow_sensitive_staging = true`: é-lhe dito que lhe pergunte a si.

Veja a [Referência da CLI](cli-reference.md).
