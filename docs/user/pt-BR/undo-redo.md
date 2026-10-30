[English](../undo-redo.md) · [Čeština](../cs/undo-redo.md) · [Deutsch](../de/undo-redo.md) · [Español](../es/undo-redo.md) · [Français](../fr/undo-redo.md) · [Italiano](../it/undo-redo.md) · [日本語](../ja/undo-redo.md) · [한국어](../ko/undo-redo.md) · [Nederlands](../nl/undo-redo.md) · [Polski](../pl/undo-redo.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/undo-redo.md) · [Русский](../ru/undo-redo.md) · [Svenska](../sv/undo-redo.md) · [Türkçe](../tr/undo-redo.md) · [Українська](../uk/undo-redo.md) · [简体中文](../zh-CN/undo-redo.md)

# Desfazer e refazer

**Você sempre pode voltar atrás.** Toda alteração que o GitPoulpi faz no seu
repositório é registrada (até 100 ações por padrão, `general.undo_depth`), e toda
posição que uma branch já teve também, inclusive os movimentos feitos por outros
programas. Se você cometer um erro, uma destas opções traz o trabalho de volta:

| Você quer | Use |
|---|---|
| desfazer a última coisa | `Ctrl+Z` (refazer: `Ctrl+Y` ou `Ctrl+Shift+Z`), ou os botões **Desfazer** e **Refazer** da barra de ferramentas |
| desfazer várias coisas de uma vez, ou uma coisa de muito tempo atrás | o **Histórico de desfazer** (`Alt+Z`, a seta ao lado de Desfazer, `Editar: histórico de desfazer...`) |
| recuperar uma branch, tag, stash, arquivo ou hunk que você removeu várias ações atrás, **sem desfazer o que você fez depois** | **Restaurar só esta** no Histórico de desfazer |
| mover uma branch de volta para onde ela estava, mesmo que outro programa a tenha movido | o **Reflog** da branch (menu da ref, `Reflog...`) |
| encontrar commits que nenhuma branch alcança mais | **Trabalho perdido** (`Editar: recuperar commits perdidos...`) |
| desfazer o que um shell ou um **agente de IA** fez em branches, tags, stashes ou HEAD | o mesmo `Ctrl+Z`: essas alterações também estão no histórico, marcadas como **externas** (veja [Alterações feitas fora do GitPoulpi](#alterações-feitas-fora-do-gitpoulpi)) |

Depois de toda ação que destrói algo e pode ser desfeita (descartar, excluir um
arquivo, excluir uma branch ou tag, reset hard, descartar um stash, remover um
remoto), a linha de status avisa: “descartar concluído - Ctrl+Z para desfazer”, com
a tecla como você a configurou. A dica de Desfazer diz quantas ações você pode
voltar.

`Ctrl+Z` desfaz a última coisa que o GitPoulpi fez no seu repositório. `Ctrl+Y`
(ou `Ctrl+Shift+Z`) a restaura. A barra de ferramentas tem os botões **Desfazer** e
**Refazer**, a paleta tem `Editar: desfazer` e `Editar: refazer`, e a dica de cada
botão diz o que ele desfaria (“Desfazer Commit “fix typo” (a81b9e4)”).

![A barra de ferramentas após Stage tudo: Desfazer está aceso e sua dica nomeia a operação que ele desfaria](../images/undo-toolbar.png)

Refazer nunca executa a operação de novo. Ele restaura o estado que foi
registrado, então um commit refeito é exatamente o mesmo commit, com o mesmo id e o
mesmo timestamp.

`Ctrl+Z` não faz nada enquanto um campo de texto está com o teclado (a mensagem do
commit, a caixa de filtro, o editor, que mantém seu próprio desfazer de texto).

## O Histórico de desfazer

`Alt+Z` (reatribuível), a seta ao lado do botão Desfazer ou `Editar: histórico de
desfazer...` abre um painel com toda ação registrada, da mais recente para a mais
antiga: um ícone do tipo de ação (commit, branch, tag, stash, reset, merge e
afins, descarte, preparação, configuração, alteração externa), o que ela fez no seu
idioma, há quanto tempo, e seu estado (**restaurável**, **bloqueada** com o motivo,
ou **desfeita** e pronta para refazer). Quando as entradas abrangem vários dias,
um título fino (Hoje, Ontem, depois a data) abre cada dia. O cabeçalho as conta
(“12 ações podem ser desfeitas (até 100)”).

As ações que você desfez ficam **acima** da ação mais recente, sob um divisor
(“desfeitas - Refazer as traz de volta”), esmaecidas, com a que Refazer traz de
volta primeiro mais perto do divisor. A seleção começa na ação mais recente, não
nelas. As setas movem (os títulos e o divisor são pulados), `Tab` alterna entre as
três vistas, `Esc` fecha. Para a linha selecionada:

- **Desfazer até aqui** (`u`): desfaz essa ação e todas as mais recentes, da mais
  recente para a mais antiga, depois de uma única confirmação que lista todas. Um
  passo que o daemon considera arriscado pergunta de novo antes de rodar.
- **Restaurar só esta** (`r`): traz de volta o que essa ação removeu e mantém tudo
  o que foi feito depois dela. Uma branch ou tag excluída volta no commit que tinha
  (se o nome dela estiver em uso agora, você é convidado a escolher outro, com um
  nome livre já preenchido); um stash descartado volta ao topo da lista de stashes;
  um arquivo descartado ou excluído é gravado de volta no mesmo lugar quando não
  mudou desde então, senão ao lado dele como `nome.restored` (nada é sobrescrito
  jamais); um remoto, upstream, linha de ignore ou linha de attributes removido é
  adicionado de novo. Um reset, amend, rebase, merge ou commit ganha uma branch
  `restore/<branch>-<sha>` na ponta antiga - sua branch atual e seus arquivos nunca
  se movem. A confirmação mostra exatamente o que será feito, e a restauração é uma
  nova ação: `Ctrl+Z` a desfaz.
- **Refazer até aqui** (`y`) em uma linha desfeita.
- **Restaurar main para onde Commit a deixou** (`o`, ou `Alt+Shift+Z` /
  `Editar: restaurar a branch para onde a ação bloqueada a deixou` fora do painel):
  quando outro programa moveu a branch e bloqueou o Desfazer, o reflog ainda sabe
  onde a ação a deixou.

Clique com o botão direito em uma linha (ou pressione `.`) para as mesmas ações em
um menu.

![O Histórico de desfazer depois de excluir uma branch e fazer dois commits: a linha da branch excluída está selecionada e pode ser restaurada sozinha](../images/undo-history.png)

## O Reflog

O menu da ref de uma branch local tem **Reflog...**; `Editar: Reflog...` mostra o
de HEAD, e o Histórico de desfazer tem uma aba para ele. Ele lista as posições que
a branch já teve, da mais recente para a mais antiga: SHA curto, o motivo
registrado, há quanto tempo, quem - inclusive os movimentos feitos em um shell. A
mensagem e os arquivos alterados da posição selecionada aparecem ao lado da lista.

O reflog é lido de 200 posições em 200. Um mais longo continua por conta própria:
mover o cursor (ou rolar) perto do fim da lista busca a página seguinte, e a última
linha mostra **... Carregar mais** enquanto houver uma. **Carregar mais** (`m`, ou
`Reflog: carregar mais` na paleta) a busca na hora e informa "posições carregadas: N"
na linha de status. Repositórios que usam o formato reftable do git são lidos como
qualquer outro.

![O Reflog de main: todas as posições que ela já teve, da mais recente para a mais antiga, com a selecionada descrita à direita](../images/reflog.png)

**Mostrar no Log** (`s`), **Criar branch aqui** (`b`) e **Restaurar `<branch>` para
esta posição** (`r`). A restauração pode ser desfeita. Quando a branch está em
checkout e você tem alterações locais que ela não consegue mover sem sobrescrever,
uma lista suspensa pergunta o que fazer com elas: manter minhas alterações, soft,
mixed ou hard. Hard descarta as alterações nos seus arquivos mas as salva antes, de
modo que `Ctrl+Z` ainda as traz de volta (ele avisa isso em vermelho).

## Trabalho perdido

`Editar: recuperar commits perdidos...` (também no menu de commit do Log) encontra
commits que nenhuma branch, tag ou stash alcança: os que um reflog ainda nomeia (os
commits de uma branch excluída), os que só um pin de desfazer mantém, e os soltos.
A lista aparece na hora e a varredura mais lenta por commits soltos vai se
preenchendo enquanto um indicador de progresso gira; ela nunca trava a interface.

![Trabalho perdido: os dois commits de uma branch excluída (encontrados pelo reflog) e commits soltos](../images/lost-work.png)

**Criar branch aqui** (`b`) recupera um commit com o nome que você escolher,
**Cherry-pick** (`c`) o aplica na branch atual, **Mostrar no Log** (`s`) funciona
para os que ainda são alcançáveis.

## Pela paleta de comandos

Cada ação de linha das três vistas também é uma linha da paleta de comandos, então o
painel não precisa de mouse nem de uma letra para lembrar. Pressione `Ctrl+P` com o
painel aberto: a paleta abre sobre ele e as suas linhas **Histórico de desfazer:**,
**Reflog:** e **Trabalho perdido:** agem na linha selecionada da vista que está na tela.

| Vista | Linhas da paleta |
|---|---|
| Histórico de desfazer | `Histórico de desfazer: desfazer até aqui...`, `refazer até aqui...`, `restaurar só esta...`, `restaurar a branch para onde estava...`, `atualizar`, `fechar` |
| Reflog | `Reflog: mostrar no Log`, `criar branch aqui...`, `restaurar para esta posição...`, `carregar mais`, `atualizar` |
| Trabalho perdido | `Trabalho perdido: mostrar no Log`, `criar branch aqui...`, `cherry-pick`, `buscar de novo` |

As linhas ficam esmaecidas (com o motivo) enquanto a vista delas não está aberta; os
três abridores, `Editar: histórico de desfazer...`, `Editar: Reflog...` e `Editar:
recuperar commits perdidos...`, também trocam um painel aberto para a vista deles.
Uma linha que a seleção não pode executar diz o motivo na linha de status em vez de
não fazer nada: "esta ação já foi desfeita: use Refazer até aqui", o motivo pelo qual
uma entrada não pode ser restaurada isoladamente, "só uma branch ou HEAD pode ser movida
de volta", e assim por diante. As linhas com `...` abrem a mesma confirmação ou o mesmo
pedido de nome que o botão.

## O que pode ser desfeito

| Área | Operações |
|---|---|
| Commits | commit, amend, reword, o compositor de commits com IA (todos os seus commits como um único passo) |
| Branches e tags | criar, excluir, renomear, definir ou remover o upstream, criar uma branch de rastreamento, criar / excluir / anotar uma tag, fast-forward, fazer checkout de um pull request como branch |
| Navegação | checkout de uma branch ou de um commit (inclusive um HEAD desanexado) |
| Histórico | merge (fast-forward ou commit de merge), rebase (simples e interativo, depois de concluído), cherry-pick, revert, reset (soft, mixed e hard), a parte local de um pull |
| Alterações em andamento | stage e unstage (arquivo, pasta, hunk, linhas, tudo), descartar (arquivo, pasta, hunk, linhas, um checkout de arquivo), aplicar um patch, **excluir um arquivo do disco**, **ignorar** (arquivo, extensão ou pasta: a linha do `.gitignore` escolhido), **tratar como binário ou texto** (a linha do `.gitattributes`) |
| Stashes | push, apply, pop, drop |
| Remotos e configuração | adicionar, remover, renomear, alterar a URL ou a URL de push de um remoto, track e untrack do Git LFS |

Enquanto um merge, cherry-pick ou revert está parado por conflitos, **Salvar e
marcar resolvido**, o stage e o unstage também podem ser desfeitos (cada um é
desfeito antes do que vem abaixo dele), e **Abortar merge** (ou cherry-pick, revert)
pode ser desfeito: desfazê-lo traz de volta toda a operação interrompida, com seus
conflitos (as três versões de cada arquivo não resolvido), os arquivos já
resolvidos e em stage, os marcadores de conflito na árvore de trabalho e a mensagem
do merge; refazer aborta de novo. O stage e as resoluções deixam de poder ser
desfeitos quando a operação termina (commit feito ou abortada). *Unstage* e *Unstage
All* nunca tocam em um arquivo que ainda está em conflito: ele não tem alteração em
stage para desfazer.

Tudo o que um descarte ou um hard reset joga fora é guardado antes, então desfazê-lo
traz de volta os bytes exatos, inclusive arquivos não rastreados, bits de execução e
links simbólicos. Desfazer um stash descartado devolve o mesmo stash, com o mesmo id
de commit (ele volta ao topo da lista de stashes, como `stash@{0}`, e não à posição
antiga). Desfazer a remoção de um remoto devolve suas branches de rastreamento
remoto, seu alias `origin/HEAD` e as configurações de upstream das branches que o
rastreavam.

## O que não pode ser desfeito, e por quê

- **Qualquer coisa em um servidor**: push, excluir uma branch ou tag remota, enviar
  uma tag, push do LFS, lock e unlock. Esse estado é compartilhado com outras
  pessoas; desfazê-lo seria outro push, e o GitPoulpi não fará isso por você.
  Desfazer também nunca fica bloqueado por uma operação dessas.
- **Fetch**: ele só move refs de rastreamento remoto, que espelham o servidor. O
  próximo fetch faria tudo de novo.
- **Abortar um rebase**: desfazer não é oferecido para isso (um rebase move branches
  e mantém seu próprio estado; o abort já é o caminho de volta). Abortar um
  **merge, cherry-pick ou revert** *é* registrado, veja acima.
- **Criar um worktree**, **criar um arquivo de patch**, clone, init, ações de
  hospedagem (pull requests, issues) e chamadas de IA: não fazem parte do histórico
  do repositório.
- **Criar um novo arquivo vazio** pelo menu de arquivo, a remoção automática de
  arquivos `*.orig` após um merge e salvar um arquivo no editor embutido não são
  registrados. Alterar o **comando SSH** (`core.sshCommand`) também não, nem alterar o **checkout parcial** (nada se perde; *Desativar* traz todos os arquivos de volta).
- **Escrever em `.git/info/exclude` ou `.git/info/attributes`** (a opção “somente este clone” das listas de ignorar e de tratar como binário/texto): esses arquivos ficam fora da árvore de trabalho, então não são registrados.
- **Edições em arquivos da árvore de trabalho feitas fora do GitPoulpi** (um
  editor, um script, um agente de IA escrevendo código) nunca são capturadas:
  não existe snapshot delas, então um `git reset --hard` ou `git checkout -- arquivo`
  feito em um shell perde para sempre as edições não commitadas. O que *é*
  capturado do trabalho de um shell é o que vive nas refs: veja a próxima seção.

## Alterações feitas fora do GitPoulpi

Quando você trabalha com um agente de IA (ou com um shell ao lado do GitPoulpi), a
maior parte do trabalho com git acontece fora da janela: commits, novas branches e
worktrees, branches excluídas, resets, rebases, stashes. O GitPoulpi vê tudo isso
também. O daemon guarda as últimas refs que viu (branches locais, tags, HEAD, a
lista de stashes, a lista de worktrees) e, sempre que o repositório mudou e não foi
obra do próprio GitPoulpi, registra **uma entrada por alteração coerente** no
Histórico de desfazer, marcada como **externa**, com o motivo que o git registrou
(`commit: fix login`, `rebase (finish)`, `reset: moving to HEAD~1`, `checkout:
moving from a to b`) e o nome que o git registrou para quem a fez. O GitPoulpi não
distingue um shell de um agente de IA e nunca adivinha; a entrada diz apenas que foi
feita fora.

- **Uma rajada é uma entrada.** Vinte commits feitos entre duas verificações são
  uma entrada, do primeiro valor ao último; um rebase é uma entrada, registrada
  quando termina (nada é registrado enquanto um rebase ou merge está parado).
- **Refs excluídas ficam presas.** Uma branch, tag ou stash que sumiu é mantida
  viva em `refs/gitpoulpi/undo/`, no commit antigo, antes que algo possa podá-la;
  assim **Restaurar só esta** (ou `Ctrl+Z`) traz de volta a branch que o agente
  excluiu, mesmo depois do reflog e de uma coleta de lixo.
- **Enquanto o GitPoulpi estava fechado.** O snapshot é salvo ao lado do diário
  (`.git/gitpoulpi/refs-snapshot.json`), então o que aconteceu enquanto nenhum
  GitPoulpi rodava é encontrado na próxima inicialização e marcado como “enquanto o
  GitPoulpi estava fechado”.
- **`Ctrl+Z` funciona nelas como em qualquer entrada**: as refs voltam, uma branch
  excluída é recriada, um stash descartado é guardado de novo, um checkout externo
  faz checkout da branch antiga outra vez. Uma branch em checkout volta como `git
  reset --keep`: os arquivos acompanham, e quando uma alteração local sua está no
  caminho ele **recusa e não altera nada**. Refazer a aplica de novo. **Restaurar
  só esta** também funciona: uma branch excluída volta e as ações posteriores
  ficam; uma branch movida volta como `restore/<branch>-<sha>`.
- **A linha de status avisa uma vez**: “alteração externa registrada - Ctrl+Z para
  desfazer”, quando chega uma nova entrada externa e a janela está em primeiro
  plano.
- Como a alteração externa passa a ser a entrada mais recente, a antiga recusa
  “bloqueada por uma alteração externa” quase desaparece: você desfaz primeiro a
  alteração externa.

O que não é registrado: movimentos só de fetch das branches de rastreamento remoto
(elas espelham o servidor), as refs ocultas do próprio GitPoulpi, uma worktree
adicionada ou removida sem nenhuma mudança de branch (a entrada da branch que ela
carrega diz “worktree adicionada <caminho>”; desfazer só move refs, não remove a
worktree) e qualquer coisa dentro de arquivos. `general.journal_outside = false`
(Configurações > Geral, “Registrar alterações feitas fora do GitPoulpi”) desativa
isso: nada externo é registrado e uma alteração feita fora bloqueia a entrada mais
recente, como antes.

## Quando o desfazer se recusa

Antes de agir, o GitPoulpi verifica se tudo o que a operação tocou ainda está
exatamente como a operação deixou. Se não estiver, ele **não altera nada** e diz o
que mudou:

- `main` está em 3f2a1c0, e não em a81b9e4, onde o Commit a deixou (um commit feito
  em um shell, por exemplo);
- um arquivo, o índice, um stash ou uma seção de configuração foi alterado depois;
- um merge, rebase ou cherry-pick está em andamento;
- a operação nunca terminou porque o GitPoulpi foi encerrado enquanto ela rodava (a
  entrada fica marcada como *interrupted*, nunca é desfeita automaticamente, e
  `gitpoulpi undo --list` mostra o que ela estava prestes a alterar para que você
  possa corrigir à mão).

Desfazer é estritamente o último a entrar, o primeiro a sair: uma entrada mais
recente bloqueada bloqueia também as mais antigas, porque desfazer uma operação
antiga por baixo de uma mais nova que ainda vale seria adivinhar. Se você mesmo
recolocar as coisas no lugar, a entrada volta a ser válida. Alterações que não
tocam em nada do que uma entrada tocou (um commit em outra branch feito no shell,
um fetch) não bloqueiam nada.

### Uma entrada bloqueada que você não consegue liberar: Esquecer

Quando a entrada mais recente está presa de vez (o repositório seguiu em frente
fora do GitPoulpi, ou a entrada nunca terminou), nada abaixo dela pode ser desfeito
também. A dica de Desfazer e a recusa então avisam isso e indicam a saída:
**Editar: esquecer a entrada de desfazer bloqueada** na paleta (ele pergunta
antes), ou `gitpoulpi undo-forget` na linha de comando. Esquecer apenas remove essa
entrada do diário; o repositório nunca é tocado, e a entrada não pode mais ser
desfeita nem refeita. A entrada abaixo dela passa a ser a mais recente.

Dois casos se resolvem sozinhos:

- um merge ou rebase em conflito que você conclui com o git puro: a entrada é
  fechada na próxima vez que o GitPoulpi lê o estado do repositório;
- os arquivos de um stash pop ou apply em conflito que você resolve com **Salvar e
  marcar resolvido**: Desfazer recoloca o conflito enquanto o arquivo resolvido
  ainda está em stage e sem commit.

## Confirmações

Desfazer só pergunta quando é arriscado:

- o commit (ou merge, rebase, cherry-pick, revert) que você está desfazendo já está
  no upstream da branch: desfazê-lo reescreve um histórico que você publicou, e
  você teria de fazer force-push do resultado;
- você tem alterações não commitadas e o desfazer move a árvore de trabalho: suas
  alterações são mantidas, mas o desfazer para, sem alterar nada, se uma delas
  estiver em um arquivo que ele precisa reescrever.

Refazer, e todo desfazer de um simples movimento de ref, nunca pergunta: a direção
contrária está a uma tecla de distância.

## Até onde voltar

`general.undo_depth` (Configurações > Geral) é o número de operações mantidas. O
padrão é **100**; `0` significa ilimitado. Reduzi-lo esquece as entradas mais
antigas na hora. Iniciar uma nova operação depois de um desfazer descarta o que
você poderia ter refeito, como em qualquer editor.

O histórico é mantido por worktree (um worktree vinculado tem o seu), no diretório
git (`.git/gitpoulpi/journal.json`), então sobrevive ao fechamento do programa e
nunca faz parte do `git status` nem de um commit.

## Desativando

`general.undo_enabled = false` (Configurações > Geral, “Desfazer ativado”)
desativa o recurso inteiro (e com ele o registro das alterações externas). Nada é
registrado nem guardado, então stage, descartar e commit ficam um pouco mais rápidos. Os botões Desfazer e Refazer desaparecem e
`Ctrl+Z` diz “Desfazer está desativado”.

**Desativar apaga o histórico**: o arquivo do diário e as refs ocultas descritas
abaixo são excluídos na primeira vez que qualquer requisição chega ao daemon
enquanto ele está desativado. Ativar de novo começa com um histórico vazio.

## O que é armazenado

Para garantir que um desfazer nunca perca dados, todo commit, objeto de tag, stash
ou snapshot de que uma entrada possa precisar é mantido vivo por uma ref oculta em
`refs/gitpoulpi/undo/` enquanto a entrada estiver no histórico. Elas ficam fora das
listagens do próprio GitPoulpi, mas um `git log --all` ou `git for-each-ref` no
shell as mostra, e `git push --mirror` as enviaria (use `git push --all` no lugar).
Elas são excluídas quando sua entrada sai do histórico ou quando o desfazer é
desativado. Nada mais é excluído.

## Pela linha de comando e pelo MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` descreve os passos e não altera nada.
- `--list` imprime o diário, com a entrada mais recente por último (`--all` inclui as
  entradas que não podem mais ser refeitas mas ainda são mantidas), com o id e o
  estado de cada entrada.
- `--expect <id>` faz a chamada recusar como *stale* a menos que essa entrada ainda
  seja a mais recente: um script (ou uma segunda janela) nunca desfaz algo diferente
  do que viu.
- `undo-forget` remove do diário a entrada mais recente bloqueada (`--redo`: a
  entrada de refazer mais recente). Ele se recusa quando essa entrada não está
  bloqueada, e termina com **3** sem `--yes`. É uma mutação para o MCP, mas não
  destrutiva: o repositório não é tocado.
- Código de saída **0** feito, **2** recusado (o JSON nomeia o motivo), **3**
  arriscado e exige `--yes`.

Voltar atrás sem desfazer tudo o que veio depois:

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      (também: undo restore <id>)
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   (também: reflog restore ...)
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` mostra, para cada entrada, o que **restaurar só esta** poderia
  trazer de volta (`restore.items`) ou por que nada (`restore.not_restorable`), e
  quem a fez: `"source": "gitpoulpi"`, ou `"source": "outside"` com `outside.message`
  (o motivo do git) e `outside.committer` para uma alteração feita por um shell ou
  por um agente de IA. `undo`, `redo` e `undo-restore` tratam as entradas externas
  como quaisquer outras.
- `undo-restore <id>` traz de volta o que essa entrada removeu (uma branch ou tag
  excluída, um stash descartado, um arquivo descartado ou excluído, um remoto
  removido com suas branches de rastreamento e upstreams, uma linha de ignore) e
  mantém todas as operações posteriores. Nunca sobrescreve: um arquivo que mudou
  desde então recebe a versão guardada ao lado, como `<name>.restored`; um nome de
  branch em uso é uma recusa (código de saída 2) que sugere um livre, informado com
  `--name`; depois de um reset, amend ou rebase ela cria uma nova branch na ponta
  antiga e nunca move a sua branch. A restauração é uma entrada do diário, então
  `gitpoulpi undo` a desfaz.
- `reflog` lista onde `HEAD` ou uma branch esteve, com o motivo do git, inclusive o
  que um shell fez. `reflog-restore` move uma branch (ou `HEAD`) para uma dessas
  posições; uma branch em checkout com alterações não commitadas precisa de
  `--mode`. Também é registrado no diário.
- `lost-commits` lista commits que nenhuma branch, tag, remoto ou stash alcança.
- Quando um movimento feito em um shell bloqueia a entrada mais recente, a recusa
  traz a posição do reflog que guarda o que a entrada espera
  (`reason.RefMoved.restore`), para ser passada a `reflog-restore`.

Existem dois comandos para que as ações acima possam ser desfeitas em todas as
superfícies: `gitpoulpi rm-file <paths>` (excluir arquivos da árvore de trabalho) e
`gitpoulpi ignore <pattern> [--file <path/.gitignore>]` (acrescentar uma linha a um
`.gitignore`; nada acontece quando a linha já está lá; desfazer devolve o arquivo
como era, ou o remove se o comando o criou, e se recusa se o arquivo foi editado
depois).

O servidor MCP tem uma ferramenta por comando, então `undo`, `redo`, `rm-file` e
`ignore` também são ferramentas. `undo`, `redo` e `rm-file` são classificados como
destrutivos (controlados como `reset`, veja [MCP e agentes](mcp-and-agents.md));
`ignore` é uma mutação comum. O diário é um arquivo no repositório, então
`gitpoulpi undo` em um shell desfaz o que a janela fez, e a janela pode desfazer o
que um script fez.
