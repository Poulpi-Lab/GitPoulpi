[English](../staging-and-committing.md) · [Čeština](../cs/staging-and-committing.md) · [Deutsch](../de/staging-and-committing.md) · [Español](../es/staging-and-committing.md) · [Français](../fr/staging-and-committing.md) · [Italiano](../it/staging-and-committing.md) · [日本語](../ja/staging-and-committing.md) · [한국어](../ko/staging-and-committing.md) · [Nederlands](../nl/staging-and-committing.md) · [Polski](../pl/staging-and-committing.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/staging-and-committing.md) · [Русский](../ru/staging-and-committing.md) · [Svenska](../sv/staging-and-committing.md) · [Türkçe](../tr/staging-and-committing.md) · [Українська](../uk/staging-and-committing.md) · [简体中文](../zh-CN/staging-and-committing.md)

# Stage e commit

Como ver o que você alterou, escolher exatamente o que entra em um commit, escrever o
commit e olhar os arquivos com mais profundidade (blame, histórico, o editor
embutido). Termos como *linha WIP* e *hunk* são explicados no
[tour pela interface](interface-tour.md).

## As alterações em andamento

O trabalho não commitado é a **primeira linha do Log** (círculo tracejado, `// WIP`,
com contadores `✎ n` modificados e `+ n` adicionados). Selecione-a e o painel da
direita mostra as alterações em andamento:

![As alterações em andamento: arquivos Unstaged e Staged à direita, com o formulário de commit embaixo](../images/hero.png)

- **Arquivos unstaged**: arquivos editados, adicionados (não rastreados) e excluídos
  que ainda não estão no próximo commit.
- **Arquivos staged**: o que o próximo commit vai conter.
- **Caminho** ou **Árvore** (`m`) lista os arquivos de forma plana ou em pastas.
  **Recolher tudo** (`z`) recolhe as pastas. **Ver todos os arquivos** (`a`) mostra a
  árvore inteira em vez de só os arquivos alterados. Clique com o botão direito no espaço
  vazio da lista para **Expandir tudo** / **Recolher tudo**, ou em uma linha de pasta para
  **Expandir todas as subpastas** / **Recolher todas as subpastas**; teclas `=`, `-`, `*`, `_`
  (veja [Mouse e gestos](mouse-and-gestures.md)).
- O título diz quantos arquivos mudaram em qual branch.

### Stage, unstage, descartar

| Para | Faça |
|---|---|
| Fazer stage de um arquivo | Passe o mouse sobre a linha dele e clique em **Stage arquivo**, ou selecione-o e pressione `Ctrl+Space`, ou `Ctrl+G s` |
| Fazer stage de uma pasta | **Stage pasta** na linha da pasta |
| Fazer stage de tudo | **Stage tudo** no título da seção, ou `Ctrl+G a` |
| Fazer unstage | Os mesmos botões e teclas na seção Staged (`Ctrl+G u`) |
| Descartar as alterações de um arquivo | Clique com o botão direito no arquivo > **Descartar alterações...** (pergunta antes; `Ctrl+Z` traz as alterações de volta) |

### Arquivos sensíveis

Fazer stage de um arquivo, de uma pasta ou de tudo primeiro confere o que seria adicionado. Um arquivo cujo
nome corresponde a `[privacy] sensitive_patterns` (`.env` e `.env.*`, exceto os
`.example`, `.sample` e `.template`, `*.pem`, `*.key`, chaves privadas SSH,
`*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`), ou
cujo texto contém um segredo que o scanner reconhece, não recebe stage. A pergunta **Arquivos
sensíveis** lista cada arquivo com o seu motivo (um padrão, ou o tipo de segredo
com uma prévia mascarada, nunca o valor) e oferece **1. Stage mesmo assim** ou **2. Cancelar,
não fazer stage de nada**. Nenhuma resposta fica destacada até você pressionar uma seta ou um dígito,
então um Enter perdido não responde nada; `Esc`, o `✕` de fechar e um clique fora
cancelam. É a mesma pergunta pelo teclado, por um botão, por um menu, pela paleta ou por
arrastar e soltar, porque todos enviam o mesmo pedido. Cancelar não faz stage de nada,
nem dos arquivos comuns do mesmo stage.

Arquivos de texto de até 1 MiB cada e 32 MiB no total são lidos; arquivos maiores e arquivos
binários são julgados apenas pelo nome. `[privacy] staging_guard = false` desativa a verificação.
Fazer stage de um hunk ou de algumas linhas não é verificado: você já está trabalhando nesse arquivo.

Clique com o botão direito em um arquivo para ver o menu dele: stage ou unstage,
descartar, copiar o caminho, histórico e blame do arquivo, abrir no seu editor
externo, ferramenta de diff ou gerenciador de arquivos (configurados em
[Configurações > Diff e editor](settings.md#diff-e-editor)), e mais.

## Ignorar arquivos e tratá-los como binários ou como texto

Clique com o botão direito em um arquivo > **Ignorar** para ignorá-lo ou ignorar sua extensão, ou em uma linha de pasta > **Ignorar todos os arquivos da pasta**. O GitPoulpi pergunta primeiro em qual arquivo vai a regra: o `.gitignore` da raiz do repositório (padrão), o `.gitignore` de cada pasta até a do arquivo, ou `.git/info/exclude` (somente este clone, nunca versionado). Arquivos que ainda não existem são marcados como *novo* e criados ao confirmar. A linha é escrita relativa ao arquivo escolhido: `src/a/b.txt` vira `/a/b.txt` em `src/.gitignore`. Mova-se com as setas, digite para filtrar a lista, `Enter` escreve, `Esc` cancela. `Ctrl+Z` desfaz uma escrita em um `.gitignore`; `.git/info/exclude` fica fora da árvore de trabalho e não pode ser desfeito.

![A lista que pergunta em qual arquivo vai uma regra de ignorar: o .gitignore do repositório, o .gitignore da pasta do arquivo ou .git/info/exclude, cada um com a linha que seria escrita](../images/ignore-picker.png)

Clique com o botão direito em um arquivo > **Tratar como binário** (ou **Tratar como texto** quando o git já o trata como binário) escreve uma regra de `.gitattributes`: `binary` ou `text diff`. Escolha o próprio arquivo ou todos os arquivos com sua extensão e depois o `.gitattributes` onde escrever, com a mesma lista acima (e `.git/info/attributes` somente para este clone). Uma regra já escrita para o mesmo padrão é substituída, nunca contradita. O diff do arquivo é atualizado na hora e a linha de status diz o que foi escrito e onde (e avisa se outra regra ainda prevalece). Uma linha de pasta tem **Tratar os arquivos da pasta como** > **Binário** ou **Texto**. O mesmo existe na paleta de comandos (`Alterações: tratar o arquivo selecionado como binário...`) e na linha de comando (`gitpoulpi treat binary|text <caminhos>`). A visualização de arquivo também segue a regra: um arquivo tratado como texto mostra suas linhas mesmo com um byte NUL.

## Menus, seleção múltipla e conversão nas listas de arquivos

As listas de arquivos do painel direito (Unstaged, Staged, os arquivos alterados de um commit e **Ver todos os arquivos**) têm os mesmos menus e gestos. Clique com o botão direito em um arquivo, em uma linha de pasta ou na parte vazia de uma lista, ou pressione `.` (`Shift+F10`, a tecla Menu) na linha com foco, ou execute **Arquivos: abrir o menu da linha ou da seleção** na paleta.

- **Um arquivo de um commit** (e um arquivo de **Ver todos os arquivos**): **Abrir diff**, **Abrir arquivo**, **Histórico do arquivo**, **Blame do arquivo**, abrir no seu editor, no programa padrão ou em **Abrir com...**, **Mostrar na pasta**, **Editar arquivo**, **Copiar caminho do arquivo**, **Copiar o caminho completo do arquivo**, **Ignorar**, **Tratar como binário** ou **Tratar como texto**, **Converter**, **Restaurar o arquivo a partir deste commit** (pergunta antes; como `git checkout <commit> -- <arquivo>`, também faz stage do arquivo, e `Ctrl+Z` traz de volta o arquivo anterior) e **Excluir arquivo...**. Uma linha sobre a cópia de trabalho fica cinza, com o motivo, quando o arquivo não está na árvore de trabalho. **Diff com o diretório de trabalho** abre o diff da versão do commit contra o arquivo em disco. **Criar patch das alterações do arquivo...** grava um patch da alteração desse arquivo no commit (uma seleção e uma linha de pasta também o têm; em **Ver todos os arquivos** nas alterações em andamento, cria o patch das alterações do arquivo).
- **Uma linha de pasta**: **Abrir pasta**, **Copiar caminho do arquivo**, **Ignorar todos os arquivos da pasta**, **Tratar os arquivos da pasta como**, **Converter**, **Criar um arquivo nesta pasta**.
- **A parte vazia de uma lista**: **Criar arquivo** e **Abrir a pasta do repositório**. A lista Unstaged acrescenta **Stage tudo** e **Descartar todas as alterações...**, a lista Staged **Unstage tudo**.

![O menu de contexto de um arquivo de um commit: abrir o diff, histórico e blame do arquivo, abrir em um editor ou no programa padrão, copiar caminhos, ignorar, converter, restaurar do commit e excluir](../images/commit-file-menu.png)

### Selecionar vários arquivos

Os gestos são os do Log. **Ctrl+clique** adiciona uma linha à seleção ou a remove. **Shift+clique** (**Alt+clique** em terminais que ficam com o Shift+clique) pega o intervalo a partir da âncora. Um clique simples recomeça e abre o diff como antes. `Esc` limpa a seleção. No teclado, `Space` ou `Insert` alterna a linha atual em todas as listas (em Unstaged e Staged `Ctrl+Space` faz stage da seleção, ou da linha atual) e `Shift+Cima` / `Shift+Baixo` ampliam o intervalo. Uma linha de pasta seleciona todos os arquivos dentro dela, recolhida ou não. As linhas selecionadas ficam destacadas e a linha de status as conta. `Ctrl+A` seleciona todas as linhas da lista em foco.

Clique com o botão direito em uma linha selecionada (ou pressione `.`) para o menu de toda a seleção, intitulado **N arquivos**: stage ou unstage, descartar, ignorar, tratar como binário ou texto, converter, copiar os caminhos, restaurar do commit, excluir. Stage, unstage e restaurar são uma ação cada; descartar e excluir perguntam antes e dizem quantos arquivos. Um clique com o botão direito em uma linha fora da seleção recomeça a seleção com essa linha.

![Quatro arquivos selecionados na lista Unstaged, com o menu intitulado 4 arquivos: stage, descartar, ignorar, tratar como binário ou texto, converter, copiar os caminhos e excluir](../images/file-pane-multiselect.png)

### Converter codificação, fins de linha e indentação

**Converter** está no menu de um arquivo, de uma pasta ou de uma seleção. `gitpoulpi convert` faz o mesmo pela linha de comando.

- **Codificação**: UTF-8, UTF-8 BOM, UTF-16 LE e BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR.
- **Finais de linha**: LF, CRLF ou CR.
- **Indentação**: espaços ou tabulações no início da linha, com a largura de uma tabulação.

Os arquivos da árvore de trabalho são reescritos em segundo plano com as mesmas funções da linha de status do editor. Um arquivo que não é UTF-8 válido e não tem marca de ordem de bytes é lido como Windows-1252, como no editor. Arquivos binários, ausentes ou que a codificação de destino não consegue representar são ignorados e relatados (*12 arquivos convertidos, 3 ignorado(s) (3 binário(s))*), nunca danificados; um arquivo que não pode ser gravado gera uma notificação de falha. O lote inteiro é uma única entrada do diário de desfazer: um `Ctrl+Z` devolve todos os arquivos. Um arquivo aberto no editor bloqueia a conversão até ser fechado.

## Lendo um diff

Clique no nome de um arquivo (ou pressione `d` nele) e o diff dele substitui o Log no
centro. `Esc` ou `q` traz o Log de volta.

![Um diff com dois hunks, cada um com os botões Descartar hunk e Stage hunk](../images/diff.png)

A barra acima dele troca a visualização:

- **Inline** mostra o diff unificado; **Dividido** coloca o antes e o depois lado a
  lado (`m` alterna os layouts).
- **Hunks** (`Shift+H`) mostra só os hunks alterados com uma linha de contexto.
  Desative para ver o arquivo inteiro.
- **EB** (`s`) oculta as alterações que diferem só em espaços em branco.
- **Quebra** (`w`) quebra as linhas longas em vez de truncá-las.
- **Mapa** (`Shift+M`) mostra um minimapa do arquivo inteiro ao lado da barra de
  rolagem. Clique nele para pular. Outras ocorrências de uma palavra selecionada são
  coloridas e marcadas na barra de rolagem.
- `n` e `p` (ou as setas na barra) pulam entre hunks.
- **Auto** (o último botão, `Shift+C`) é a codificação em que o arquivo é lido. Clique nele (ou pressione
  `Shift+C`, ou execute *Diff: escolher a codificação...* na paleta) e escolha uma da lista:
  **Detecção automática** primeiro, depois UTF-8, UTF-8 BOM, UTF-16 LE e BE, Windows-1252, ISO-8859-1,
  ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5 e EUC-KR (digite para
  buscar na lista). Os dois lados do arquivo são lidos de novo, como bytes exatos, nessa codificação, então um
  arquivo Latin-1 mostra os seus acentos e um arquivo UTF-16 (que o git chama de binário) mostra as suas linhas. A
  escolha é lembrada para o caminho desse arquivo até você sair, nesta visualização e na
  [revisão](review.md). Enquanto uma codificação está escolhida, os hunks são os próprios desta visualização, então o stage e o
  descarte ficam desativados (escolha **Detecção automática** para fazer stage de novo). Uma leitura lenta diz *lendo como ...* no
  título depois de um instante.

![O mesmo diff no layout dividido, antes e depois lado a lado](../images/diff-split.png)

Você pode selecionar texto com o mouse (arrastando, ou com clique duplo em uma
palavra) e copiá-lo com `y` ou `Ctrl+C`; só o texto do arquivo é copiado, nunca
números de linha nem marcadores `+` e `-`.

### Fazer stage de um hunk ou de uma única linha

Em um diff da árvore de trabalho, o cabeçalho de cada hunk tem **Stage hunk** e
**Descartar hunk** (no lado Staged, **Unstage hunk**).

- `Space` faz stage do hunk sob o cursor.
- Para fazer stage de algumas linhas, pressione `v` para iniciar uma seleção, mova
  com `j`/`k` e pressione `Enter`. Só as linhas *alteradas* contam.
- `x` descarta o hunk e `Shift+X` as linhas selecionadas. Ambos perguntam antes, e
  `Ctrl+Z` traz as alterações de volta.
- Clique com o botão direito (ou `.`) em um hunk para ver o menu dele: stage, unstage, descartar,
  histórico do arquivo, blame, copiar.

O diff de um commit é somente leitura. Uma imagem alterada aparece como imagem; veja
[Imagens](image-view.md).

## Escrevendo o commit

O formulário de commit fica na parte de baixo do painel da direita, e seu **Resumo**
é também o campo da linha de alterações em andamento do Log: digitar em um aparece
no outro.

![O formulário de commit preenchido com uma mensagem escrita pela IA: resumo com contador de caracteres, descrição e o botão verde de commit](../images/ai-commit-message.png)

- **Resumo** (o contador mostra quantos dos 72 caracteres restam) e **Descrição**.
  `Enter` no resumo passa para a descrição, `Enter` ali adiciona uma linha.
- **Commit das alterações** (o botão verde) ou `Ctrl+Enter` faz o commit das
  alterações em stage. Se nada estiver em stage, o botão diz isso e não faz nada.
- **Amend do commit anterior** (`Shift+A`) incorpora as alterações em stage e a nova
  mensagem ao último commit. Um commit que já foi enviado precisará então de um
  force push (com lease).
- **Ignorar hooks** (`Shift+H`, ou **Commit: alternar Ignorar hooks** na paleta) faz o
  commit sem executar os hooks `pre-commit` e `commit-msg` do repositório. Vale por
  commit; **Configurações > Configurações do repositório > Commit** faz a opção
  começar marcada. Aparece quando a linha tem espaço.
- `c` no painel Status move o cursor para o formulário. `Esc` sai dele.
- `Ctrl+G c` e `Ctrl+G Enter` (**Git: Commit...** e **Git: Fazer commit das
  alterações em stage agora** na paleta) são um só comando, o botão de commit de
  qualquer lugar. Com um resumo no formulário e algo em stage, faz o commit na
  hora, respeitando **Amend do commit anterior** e **Ignorar hooks**, sem mover o
  foco. Caso contrário, não faz commit de nada: move o cursor para o resumo do
  formulário (mostrando o painel se estava oculto ou recolhido) e a linha de
  status diz qual atalho faz o commit (`Ctrl+Enter` por padrão). Sem nada em
  stage, ele diz isso. Nada é digitado na linha de status. Enquanto uma
  solicitação de IA ainda está em andamento, ele não faz commit de nada e diz
  isso (`Esc` cancela a solicitação). `Ctrl+Z` desfaz o commit como qualquer
  outro.
- Por repositório, as [Configurações](settings.md#específico-do-repositório-commit) podem
  fazer **push após cada commit**, iniciar o formulário com um **modelo de commit**
  (e remover os comentários `#` da mensagem) e fazer um merge **squash**: o
  formulário então abre com a mensagem de squash do git.
- Com uma integração de hospedagem vinculada, `Alt+R` no formulário insere uma
  referência de issue; veja
  [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md).

Com a [IA](ai.md) ativada, o botão **`✦`** à direita do resumo escreve uma mensagem
a partir das alterações em stage (`Ctrl+G i`), e **Compor commits com IA** divide o
seu trabalho em vários commits, abaixo. O texto vai para o formulário para você
editá-lo; nada é commitado até você pressionar o botão de commit.

## O compositor de commits: divida suas alterações em commits

*(Exige a IA ativada: veja [Assistência de IA](ai.md).)* Pressione **Compor commits
com IA** (ou `Ctrl+G C`, ou clique com o botão direito na linha de alterações em
andamento). O modelo propõe vários commits, cada um com seus arquivos; cada arquivo
alterado fica em exatamente um commit, e um plano inválido é devolvido ao modelo uma
vez. A proposta abre como uma revisão em que nada foi commitado ainda:

![O compositor de commits: quatro commits propostos, cada um expansível até seus arquivos, com controles de subir, descer, editar e descartar, e os botões Redefinir, Cancelar e Criar 4 commits](../images/composer.png)

| Para | Faça |
|---|---|
| Expandir ou recolher um commit | `Left` / `Right`, ou clique na seta; `*` alterna todos |
| Mover um commit para cima ou para baixo | O `▲` `▼` na linha dele, ou `Alt+Up` / `Alt+Down` (`K` / `J`) |
| Mover um arquivo para outro commit | Arraste-o sobre o commit (da lista, ou da lista de alterações em andamento à direita); ou `m` no arquivo e escolha o commit no menu suspenso; ou `Alt+Up` / `Alt+Down` no arquivo |
| Colocar um arquivo em um commit só dele | `n` no arquivo, ou o botão `+ commit próprio` |
| Reescrever um commit | O lápis, `e`, `Enter` ou um clique duplo: edite a mensagem, `Enter` a mantém, `Esc` a descarta |
| Deixar um commit de fora | `drop` (`d` ou `Delete`); seus arquivos continuam sem commit. `restaurar` o traz de volta |
| Recomeçar | **Redefinir** (`r`) volta ao plano do modelo |
| Criá-los | **Criar N commits** (`Ctrl+S`, `Ctrl+Enter` ou `a`) |
| Sair | **Cancelar** ou `Esc`: nada é feito |

Criar os commits faz unstage de tudo e depois, para cada commit, faz stage de seus
arquivos e o commit, então sua árvore de trabalho nunca é tocada. Se uma etapa
falhar, ele para e diz quantos commits foram feitos. Arquivos inteiros entram em
stage: um arquivo que estava em stage só em parte é commitado inteiro. Escolha qual
perfil e quais instruções o compositor usa em *Configurações > IA*; veja
[Assistência de IA](ai.md#o-compositor-de-commits).

## Visualização de arquivo, Blame e Histórico do arquivo

A barra de visualização tem **Arquivo** (o conteúdo do arquivo), **Diff**, **Blame** e o
seletor `Histórico │ Todos` da [coluna de commits](interface-tour.md#a-coluna-de-commits) (a sua lista `Histórico` é o Histórico do arquivo); os mesmos estão no menu de botão direito de um arquivo.
Eles mostram o arquivo como estava no commit de onde você veio, ou como não commitado
(`@ WIP`) quando você o abriu pelas alterações em andamento.

- **Blame** e o **Histórico do arquivo** são visões completas por si: veja [Blame](log-and-history.md#blame) e [Histórico do arquivo](log-and-history.md#histórico-do-arquivo) em *Trabalhando com o Log*. `Enter` num hunk do blame ou numa linha do histórico vai a esse commit no Log.
- `y` ou `Ctrl+C` copia o texto selecionado (no Blame, só o código).
- `Shift+G` pula para a última linha. `q` fecha a visualização.

## O editor embutido

**Editar** (`e`) abre a *cópia do diretório de trabalho* do arquivo em um pequeno
editor sobre o painel central: cores de sintaxe, números de linha, desfazer e
refazer, busca e uma linha de status (posição, fim de linha, indentação,
linguagem). Ele sempre edita a cópia de trabalho, mesmo quando você o abriu pelo
diff de um commit antigo.

![O editor mostrando um arquivo Rust com realce de sintaxe e números de linha; a barra tem Salvar, Desfazer, Refazer e Buscar](../images/editor.png)

| Tecla | Ação |
|---|---|
| `Ctrl+S` | Salvar (o arquivo é gravado em um arquivo temporário e renomeado, então nunca fica meio escrito; fins de linha, BOM e a quebra de linha final são preservados) |
| `Ctrl+Z`, `Ctrl+Y` | Desfazer, refazer |
| `Ctrl+F`, `F3` | Buscar, buscar próximo |
| `Tab`, `Shift+Tab` | Indentar, desindentar (seguindo o estilo do próprio arquivo) |
| `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V` | Selecionar tudo, copiar, recortar, colar |
| `Esc` | Fechar. Com alterações não salvas, pergunta **Salvar** (`s`), **Descartar** (`d`) ou **Cancelar** (`c`) |

Arquivos binários são recusados. Tamanho da
tabulação, fins de linha, realce de sintaxe, números de linha e quebra de linha ficam
em *Configurações > Diff e editor*; veja [Configurações](settings.md#diff-e-editor).

Com **Seguir a seleção** marcada (a caixa na barra do editor, também *Editor: alternar seguir a seleção* na paleta e *Configurações > Diff e editor*), selecionar outro arquivo no painel direito enquanto o editor está aberto abre a cópia de trabalho desse arquivo no editor, em vez do diff ou da Visualização de arquivo. Havendo alterações não salvas, ele pergunta antes **Salvar**, **Descartar** ou **Cancelar**; Cancelar mantém você no arquivo atual. Um arquivo que não pode ser editado (binário, grande demais, excluído) abre na visualização habitual e a linha de status diz o motivo. Ao fechar o editor, você cai na visualização do arquivo selecionado. Marcada por padrão. Isso vale para as alterações de trabalho: escolher um arquivo de um commit ou de uma comparação durante a edição sai do editor (perguntando antes **Salvar**, **Descartar** ou **Cancelar** se houver alterações não salvas) e mostra o diff do arquivo nesse commit, e a coluna de commits, que o editor não tem, volta junto.

A linha de status, à direita, mostra a **codificação**, as **quebras de linha** e a
**indentação** do arquivo, depois a posição do cursor e a sintaxe. Os três primeiros são
botões: cada um abre uma lista suspensa (setas ou roda, digite para filtrar, `Enter`
para escolher, `Esc` para fechar). A paleta (`Ctrl+P`) abre os mesmos menus: *Editor: alterar codificação*, *Editor: reabrir com codificação*, *Editor: converter para codificação*, *Editor: converter quebras de linha*, *Editor: alterar indentação*.

![O menu de codificação do editor aberto sob o botão UTF-8 da barra: reabrir o arquivo com outra codificação ou convertê-lo](../images/editor-encoding-menu.png)

![O menu de fim de linha do editor aberto sob o botão LF: converter o arquivo para LF, CRLF ou CR](../images/editor-eol-menu.png)

- **Codificação.** Um arquivo abre na codificação que ele anuncia (um BOM: UTF-8 ou
  UTF-16), senão em UTF-8, senão em Windows-1252, e a barra diz qual é. **Reabrir com
  codificação** lê os bytes do arquivo de novo com outra codificação (pergunta antes se
  há alterações não salvas): use quando a detecção errou. **Converter para codificação**
  mantém o texto e muda a codificação em que o arquivo é salvo (UTF-8, UTF-8 BOM,
  UTF-16 LE e BE, Windows-1252, ISO-8859-1 e -15, Windows-1250, Windows-1251, KOI8-R,
  Shift_JIS, EUC-JP, GBK, Big5, EUC-KR). Nada é gravado até você salvar, e Desfazer
  reverte. Se o texto tem caracteres que o destino não comporta, a linha de status diz
  quantos e onde está o primeiro, e nada é convertido; salvar recusa da mesma forma,
  nunca grava `?`.
- **Quebras de linha.** Mostra `LF`, `CRLF`, `CR` ou `Misto` quando o arquivo mistura
  vários tipos. O menu converte todas as quebras de linha do arquivo para LF, CRLF ou CR
  em um único passo de desfazer. As linhas novas que você digita usam a quebra do próprio
  arquivo. Sua escolha vale mais que a configuração *Quebras de linha*.
- **Indentação.** Mostra `Espaços: 4` ou `Tabulações: 4`. O menu escolhe o que `Tab` e a
  indentação automática inserem (espaços ou tabulações), o tamanho (de 1 a 8: o passo da
  indentação e a largura de uma tabulação neste arquivo), e converte o espaço em branco
  inicial de cada linha em espaços ou em tabulações, em um único passo de desfazer.

## Stash, patches e Git LFS

- **Stash** e **Pop** estão na barra de ferramentas; veja
  [Trabalhando com o Log](log-and-history.md#branches-tags-e-a-barra-lateral-refs).
- **Criar patch** e **Aplicar patch** estão nos menus do commit e da ref; a CLI tem
  `gitpoulpi patch`.
- Arquivos grandes armazenados com o Git LFS aparecem como pequenos arquivos de
  ponteiro até você fazer pull do conteúdo deles; veja [Git LFS](lfs.md).

## Pela linha de comando

Tudo nesta página pode ser automatizado por script:

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi stage --all --allow-sensitive   # depois da recusa abaixo
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

`gitpoulpi stage` recusa um arquivo sensível do mesmo jeito: nada recebe stage, o código de
saída é 2 e `--json` imprime `{error: "sensitive_files", files: [{path, reason}]}`
(uma regra e uma prévia mascarada, nunca um valor). `--allow-sensitive` é a nova tentativa. Um
agente MCP recebe a mesma recusa e não pode passar `allow_sensitive`, a menos que
`[mcp] allow_sensitive_staging = true`: ele é instruído a perguntar a você.

Veja a [referência da CLI](cli-reference.md).
