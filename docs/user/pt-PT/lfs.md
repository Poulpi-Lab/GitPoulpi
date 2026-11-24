[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · **Português (Portugal)** · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

O Git LFS ("Large File Storage") mantém os ficheiros grandes (ficheiros de design, vídeos, conjuntos de dados) fora do histórico de um repositório: o git guarda em seu lugar um *ponteiro* de texto com três linhas, e o conteúdo real fica no servidor LFS e numa cache local em `.git/lfs`.

O GitPoulpi não precisa de nenhuma conta para isto. O LFS usa as credenciais que o seu remoto já usa (o seu credential helper do git, ou o ssh-agent).

## O que precisa do programa `git-lfs` e o que não precisa

O GitPoulpi executa o programa `git-lfs` normal para tudo o que converte ou move ficheiros (install, track, fetch, pull, push, locks).

Ler o *estado* do LFS não precisa de programa nenhum: se um repositório usa LFS, que ficheiros são ponteiros, qual o seu tamanho e se o objeto real já está guardado localmente.

| | precisa de `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (indica que o programa está em falta em vez de falhar) | não |
| `gitpoulpi lfs files` | não |
| install / track / untrack / prune | sim |
| fetch / pull / push | sim |
| locks | sim |

## Instalar o `git-lfs`

- **Windows**: vem com o Git for Windows; ou `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Qualquer outro: https://git-lfs.com

É exigida a versão 3.0 ou posterior. Depois de instalar, o GitPoulpi nota-o em menos de um minuto (ou de imediato com `lfs status`).

## Quando o `git-lfs` está em falta

O GitPoulpi nunca falha nem sai por causa disso:

- `gitpoulpi lfs status` tem êxito e diz `git-lfs: not installed`, com a ligação de instalação.
- Uma operação que precisa do programa falha com uma mensagem clara que termina com a ligação de instalação (`https://git-lfs.com`) e um código de saída diferente de zero; por MCP, a ferramenta devolve essa mensagem.
- Um repositório que usa LFS abre na mesma; os seus ficheiros grandes aparecem simplesmente como ponteiros até o programa ser instalado.

## Ver os ficheiros LFS

```
gitpoulpi lfs status            # programa, padrões, filtros, hook
gitpoulpi lfs files             # ficheiros de ponteiro no índice, com o tamanho e se estão locais ou não transferidos
gitpoulpi lfs files --at v1.0   # ... tal como numa revisão
gitpoulpi show big.bin --at HEAD  # indica "Git LFS object, 12.3 MB, sha256:..." em vez do texto do ponteiro
```

## Preparar um repositório: install, track, untrack

```
gitpoulpi lfs install                 # filtros + hooks DESTE repositório (git lfs install --local)
gitpoulpi lfs install --user          # na sua configuração git global (nunca é feito em silêncio)
gitpoulpi lfs track '*.psd'           # encaminha *.psd através do LFS
gitpoulpi lfs track '*.psd' --lockable   # ... e torna-os só de leitura até serem bloqueados
gitpoulpi lfs track '*.bin' --renormalize  # ... e volta a fazer stage dos *.bin já commitados, como ponteiros
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # elimina do armazém local os objetos antigos já enviados
```

- `install` escreve por predefinição no `.git/config` e nos hooks do próprio repositório, por isso a sua configuração global do git nunca é tocada a menos que passe `--user`.
- `track` e `untrack` editam o `.gitattributes` e deixam-no **unstaged**: aparece como uma alteração normal da árvore de trabalho, de que o utilizador faz commit. Os ficheiros adicionados *depois* de acompanhados são guardados como ponteiros pelo seu commit seguinte.
- Os ficheiros que **já foram commitados** como conteúdo simples continuam simples no histórico. `--renormalize` volta a fazer stage deles para que o seu próximo commit os substitua por ponteiros. Para os mover também em todos os commits passados, use a regra de reescrita do histórico *Migrar ficheiros para o LFS* (ver [Reescrita do histórico](history-rewrite.md#mover-ficheiros-para-dentro-ou-para-fora-do-git-lfs)); é do próprio GitPoulpi, não o `git lfs migrate`.
- Ponha o padrão entre aspas para que o seu shell não o expanda.
- Por MCP, estas são mutações normais (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), sujeitas ao controlo de `[mcp]` em `config.toml`.

## Mover objetos: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # transfere os objetos para o armazém local; a árvore de trabalho fica intacta
gitpoulpi lfs pull  [--remote origin]           # fetch e depois substitui os ficheiros de ponteiro pelo conteúdo real
gitpoulpi lfs push  [--remote origin] [--all]   # envia os objetos da branch atual (ou de todas as refs com --all)
```

- O remoto é por predefinição o remoto predefinido do repositório, como em `fetch` e `push`.
- As credenciais são as habituais do seu remoto (credential helper, ssh-agent). O GitPoulpi nunca pede uma conta própria.
- `pull` instala os filtros LFS deste repositório se estiverem em falta: sem eles, o git-lfs transfere os objetos mas não substitui os ponteiros, e um checkout posterior voltaria a pôr ponteiros.
- O MCP trata `lfs push` como `push`: uma ferramenta destrutiva (altera o remoto).
- A janela mostra o progresso de uma transferência (ficheiros e bytes) na barra de estado. A CLI espera pelo resultado.

### Fazer checkout do conteúdo: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: escreve o conteúdo real sobre os ficheiros de ponteiro, a partir do armazém local
```

Nunca transfere nada (isso é `lfs pull`); os objetos em falta no armazém local continuam ponteiros. Tal como `pull`, instala primeiro os filtros LFS do repositório quando não há nenhuns configurados. Como qualquer operação LFS, não é desfazível; veja [Desfazer e refazer](undo-redo.md). Por MCP é `lfs_checkout` (uma escrita, não destrutiva).

### O botão LFS da barra de ferramentas, e pull / push

A barra de ferramentas tem um botão **`LFS ▾`** depois de **Pop** (oculto quando `git.lfs_enabled` está desativado, e num repositório onde o LFS não está inicializado nem é usado pelo seu `.gitattributes`: veja [Definições > LFS](settings.md#específico-do-repositório-lfs)). Um clique abre uma lista: **Fetch de todos os ficheiros LFS** (`lfs fetch --all`), **Checkout de todos os ficheiros LFS** (`lfs checkout`), **Pull de todos os ficheiros LFS** (`lfs pull`), **Push de todos os ficheiros LFS** (`lfs push --all`, os objetos de todas as refs) e **Limpar o repositório LFS local** (`lfs prune`). Cada linha é também um comando da paleta (`LFS: Fetch de todos os objetos`, `LFS: checkout de todos os ficheiros`, `LFS: pull de objetos`, `LFS: Push de todos os objetos`, `LFS: limpar objetos locais`). `LFS: abrir menu` abre a lista. Cada um tem um acorde `Ctrl+G`: `l` para o menu, depois `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` e `Shift+V` para as linhas. Pode alterá-los em *Definições > Teclado e rato*. Com o `git-lfs` em falta, todas as linhas levam à caixa de diálogo de instalação.

![A lista LFS da barra de ferramentas: fetch, checkout, pull e push de todos os ficheiros LFS, e limpeza do armazém local](../images/lfs-menu.png)

**O pull e o push tratam do LFS por si.** Num repositório que usa LFS, com o `git-lfs` instalado, um `pull` simples verifica primeiro que os filtros LFS do repositório estão instalados, e um `push` simples verifica o seu hook `pre-push`. Quando um deles falta, o GitPoulpi executa `git lfs install --local` (só este repositório) antes da transferência, e a barra de estado diz "filtros LFS instalados". Sem o hook, um push não enviaria nenhum objeto LFS. Desative-o com `git.lfs_autofix_filters = false` (*Definições > Git e codificação > Instalar os filtros LFS antes de pull e push*). Um `push` simples continua a enviar apenas os objetos da branch enviada, nunca os de todas as refs; o prune nunca é implícito.

## Bloqueios de ficheiros

Alguns ficheiros não se podem integrar por merge (um ficheiro do Photoshop, um nível, um recurso binário). Os *locks* do Git LFS permitem que uma pessoa diga "estou a editar isto": mais ninguém o pode bloquear nem enviar alterações para ele até ser desbloqueado.

```
gitpoulpi lfs locks  [--remote origin]              # quem tem cada ficheiro (os seus estão marcados)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- O bloqueio é uma funcionalidade do **servidor LFS** (o GitHub, o GitLab e o Gitea têm-no). É acedido com as credenciais do próprio remoto; o GitPoulpi não tem conta própria.
- Um servidor sem bloqueio faz `lock` falhar com uma mensagem que o diz (o `LfsLockingUnsupported` tipado). `locks` num servidor assim não lista nada, que é o que o próprio git-lfs devolve.
- Acompanhar um padrão com `--lockable` (`lfs track '*.psd' --lockable`) torna esses ficheiros só de leitura até os bloquear.
- `unlock` do bloqueio de outra pessoa é recusado; `--force` quebra-o, razão pela qual o MCP trata `lfs_unlock` como uma ferramenta destrutiva. `lfs_locks` é só de leitura.

## Clonar um repositório LFS

`gitpoulpi clone <url>` (e a caixa de diálogo Clonar da TUI) faz o checkout do repositório *sem* executar os filtros LFS, depois, quando o repositório usa LFS e o `git-lfs` está instalado, transfere os objetos com `git lfs pull` (instalando os filtros desse repositório se não houver nenhuns configurados) e mostra-o como uma fase própria, *A transferir objetos LFS*. `--no-lfs` salta esse passo e deixa os ponteiros.

Um clone **nunca falha por causa do LFS**: com o `git-lfs` em falta, ou uma transferência que falha, o clone tem êxito na mesma e os ficheiros grandes ficam como ponteiros; o estado LFS do repositório diz porquê, e `gitpoulpi lfs pull` conclui o trabalho quando o problema estiver resolvido.

## Na interface de terminal

- **A etiqueta `LFS`.** Um ficheiro guardado com Git LFS tem uma pequena etiqueta `LFS` a seguir ao nome nas listas Unstaged e Staged do painel Status e na lista de ficheiros alterados de um commit. **Só LFS** (`o` com o painel Status ou o Log em foco, a caixa na barra de ferramentas das listas de ficheiros, ou **Ficheiros: alternar só LFS** na paleta) restringe essas listas aos ficheiros LFS. A caixa fica na extremidade direita da linha de filtro (painel Status: na barra de ferramentas, ou no seu menu de overflow quando estreito); na lista de ficheiros de um commit encolhe para `[ ] LFS` num painel estreito em vez de desaparecer. Só aparece quando os ficheiros listados têm alguns. No painel Status e nos ficheiros alterados de um commit, um ficheiro conta como LFS quando o seu caminho é um ponteiro no índice (uma chamada `git` em lote por atualização, nada para um repositório sem atributos `filter=lfs`). Com **Ver todos os ficheiros** ligado no Log, é listada a árvore inteira na revisão selecionada e os ficheiros LFS são os dessa revisão, lidos dos seus próprios blobs (em cache por revisão, fora da thread de renderização, sem precisar do programa `git-lfs`), pelo que um ficheiro que é LFS aí mas não no índice fica com a etiqueta e é mantido pelo filtro.
- **A ficha do diff.** Abrir o diff de um ficheiro LFS mostra uma ficha em vez do texto do ponteiro: `Objeto Git LFS`, `1.2 MB -> 1.4 MB`, os oids curtos antigo e novo, e se o novo objeto está guardado localmente ou `não transferido`. Prima `r` para o diff bruto do ponteiro, `r` outra vez para voltar. Só se aplica o stage ao nível do ficheiro num ficheiro LFS (não há hunks nem linhas para fazer stage). Sob a ficha, **Abrir no programa predefinido** e **Abrir em** (`o`, `Shift+O`, ou a paleta) abrem o ficheiro real, nunca o ponteiro: consulte [Abrir em](open-in.md#ficheiros-git-lfs).
- **A vista de ficheiro** de um ficheiro LFS diz `Objeto Git LFS, 12.3 MB, sha256:ab12cd34...` e onde obter o conteúdo.
- **O aviso.** Para um repositório que usa LFS enquanto o `git-lfs` está em falta ou é anterior à 3.0, um aviso no painel Status di-lo. Clique nele (ou use a entrada da paleta *LFS: abrir página de instalação*) para abrir https://git-lfs.com. Nada mais muda: a janela abre, os diffs funcionam, os ficheiros grandes aparecem simplesmente como ponteiros.
- **O menu do ficheiro e a paleta.** Clique com o botão direito num ficheiro alterado no painel Status: *Acompanhar com LFS: este ficheiro* / *esta extensão* para um ficheiro simples; *Bloquear ficheiro (LFS)*, *Desbloquear ficheiro (LFS)* ou *Forçar desbloqueio (LFS, bloqueio de outra pessoa)* para um ficheiro LFS, consoante quem detém o seu bloqueio. A linha de um ficheiro bloqueado diz `bloqueado por si` / `bloqueado por <nome>`. Tudo é também um comando da paleta (`Ctrl+P`, escreva `LFS`): inicializar, acompanhar ou deixar de acompanhar um padrão, fetch, pull ou push de objetos, limpar, mostrar bloqueios, abrir a página de instalação.
- **Uma ação falhada explica-se a si própria.** Se uma ação LFS falhar porque o `git-lfs` está em falta, uma caixa de diálogo traz a mensagem e um botão *Abrir página de instalação*.
- **Definições.** *Definições > LFS* (por repositório) mostra se o LFS está inicializado, lista os padrões acompanhados e tem **Configurar o LFS aqui**. *Git e codificação* (por repositório) tem o interruptor **Git LFS**, uma linha só de leitura **Programa git-lfs** e um botão **Inicializar o LFS neste repositório**.
- Com o **Git LFS** desativado (`git.lfs_enabled = false`, veja a [referência de configuração](config-reference.md)) tudo isto fica oculto. Os filtros LFS do próprio git continuam a correr.
- O estado LFS é lido pouco depois de o repositório abrir, depois de cada alteração, numa atualização manual, quando o `.gitattributes` muda e a cada dez segundos num repositório que usa LFS.

## Limites conhecidos

- Um repositório cujos padrões LFS estão apenas num `.gitattributes` *aninhado* (não o da raiz, nem `.git/info/attributes`) é detetado como a usar LFS quando tem um armazém de objetos LFS (`.git/lfs`), e não antes.
- O próprio `git lfs migrate` não é usado. Mover ficheiros para dentro ou para fora do LFS ao longo do histórico cabe às regras *Migrar ficheiros para o LFS* / *Trazer ficheiros de volta do LFS* da reescrita do histórico: trabalham no clone local, só o `.gitattributes` da raiz é escrito e os objetos ainda têm de ser enviados depois (*Enviar objetos LFS* no painel de resultado, ou `lfs push --all`).
