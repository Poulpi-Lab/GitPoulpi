[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · [Français](../fr/lfs.md) · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

O Git LFS (“Large File Storage”) mantém arquivos grandes (arquivos de design, vídeos, conjuntos de dados) fora do histórico de um repositório: o git guarda no lugar um texto de *ponteiro* de três linhas, e o conteúdo real fica no servidor LFS e em um cache local em `.git/lfs`.

O GitPoulpi não precisa de conta para nada disso. O LFS usa as credenciais que o seu remoto já usa (o seu auxiliar de credenciais do git, ou o ssh-agent).

## O que precisa do programa `git-lfs` e o que não precisa

O GitPoulpi executa o programa padrão `git-lfs` para tudo o que converte ou move arquivos (install, track, fetch, pull, push, bloqueios).

Ler o *estado* do LFS não precisa de programa: se um repositório usa LFS, quais arquivos são ponteiros, qual o tamanho deles e se o objeto real já está guardado localmente.

| | precisa de `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (informa que o programa está ausente em vez de falhar) | não |
| `gitpoulpi lfs files` | não |
| install / track / untrack / prune | sim |
| fetch / pull / push | sim |
| bloqueios | sim |

## Instalando o `git-lfs`

- **Windows**: ele vem com o Git for Windows; ou `winget install GitHub.GitLFS`.
- **Debian / Ubuntu**: `apt install git-lfs`.
- **macOS**: `brew install git-lfs`.
- Qualquer outro: https://git-lfs.com

É necessária a versão 3.0 ou mais recente. Depois de instalar, o GitPoulpi percebe em até um minuto (ou imediatamente em `lfs status`).

## Quando o `git-lfs` está ausente

O GitPoulpi nunca trava nem sai por causa disso:

- `gitpoulpi lfs status` tem sucesso e diz `git-lfs: not installed`, com o link de instalação.
- Uma operação que precisa do programa falha com uma mensagem clara que termina com o link de instalação (`https://git-lfs.com`) e um código de saída diferente de zero; via MCP a ferramenta devolve essa mensagem.
- Um repositório que usa LFS ainda abre; seus arquivos grandes simplesmente aparecem como ponteiros até o programa ser instalado.

## Olhando os arquivos LFS

```
gitpoulpi lfs status            # program, patterns, filters, hook
gitpoulpi lfs files             # arquivos de ponteiro no índice, com tamanho e local/não baixado
gitpoulpi lfs files --at v1.0   # ... em uma revisão
gitpoulpi show big.bin --at HEAD  # informa "Git LFS object, 12.3 MB, sha256:..." em vez do texto do ponteiro
```

## Preparando um repositório: install, track, untrack

```
gitpoulpi lfs install                 # filtros + hooks para ESTE repositório (git lfs install --local)
gitpoulpi lfs install --user          # na sua configuração global do git (nunca feito em silêncio)
gitpoulpi lfs track '*.psd'           # route *.psd through LFS
gitpoulpi lfs track '*.psd' --lockable   # ... e os deixa somente leitura até serem bloqueados
gitpoulpi lfs track '*.bin' --renormalize  # ... e prepara de novo como ponteiros os *.bin já commitados
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # apaga do armazenamento local objetos antigos que já foram enviados
```

- `install` grava, por padrão, no `.git/config` e nos hooks do próprio repositório, então a sua configuração global do git nunca é tocada, a menos que você passe `--user`.
- `track` e `untrack` editam o `.gitattributes` e o deixam **unstaged**: ele aparece como uma alteração comum da árvore de trabalho que você mesmo comita. Arquivos adicionados *depois* do rastreamento são guardados como ponteiros no próximo commit deles.
- Arquivos que **já estão commitados** como conteúdo simples continuam simples no histórico. `--renormalize` faz stage deles de novo para que o seu próximo commit os substitua por ponteiros. Para movê-los também em todos os commits passados, use a regra de reescrita de histórico *Migrar arquivos para o LFS* (veja [Reescrita de histórico](history-rewrite.md#mover-arquivos-para-o-git-lfs-ou-para-fora-dele)); ela é do próprio GitPoulpi, não o `git lfs migrate`.
- Coloque o padrão entre aspas para que o seu shell não o expanda.
- Pelo MCP, estas são mutações comuns (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), sujeitas ao controle `[mcp]` do `config.toml`.

## Movendo objetos: fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # baixa objetos para o armazenamento local; árvore de trabalho intacta
gitpoulpi lfs pull  [--remote origin]           # fetch, depois troca os arquivos de ponteiro pelo conteúdo real
gitpoulpi lfs push  [--remote origin] [--all]   # envia os objetos da branch atual (ou de todas as refs com --all)
```

- O remoto padrão é o remoto padrão do repositório, como em `fetch` e `push`.
- As credenciais são as habituais do seu remoto (auxiliar de credenciais, ssh-agent). O GitPoulpi nunca pede uma conta própria.
- `pull` instala os filtros LFS deste repositório se estiverem ausentes: sem eles o git-lfs baixa os objetos mas não substitui os ponteiros, e um checkout posterior colocaria os ponteiros de volta.
- O MCP trata `lfs push` como `push`: uma ferramenta destrutiva (ela altera o remoto).
- A janela mostra o progresso de uma transferência (arquivos e bytes) na linha de status. A CLI espera pelo resultado.

### Fazendo checkout do conteúdo: `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: grava o conteúdo real sobre os arquivos de ponteiro, a partir do armazenamento local
```

Ele nunca baixa (isso é o `lfs pull`); objetos ausentes do armazenamento local continuam ponteiros. Como o `pull`, ele instala antes os filtros LFS do repositório quando não há nenhum configurado. Como toda operação LFS, não pode ser desfeito; veja [Desfazer e refazer](undo-redo.md). Pelo MCP é `lfs_checkout` (uma escrita, não destrutiva).

### O botão LFS da barra de ferramentas, e pull / push

A barra de ferramentas tem um botão **`LFS ▾`** depois de **Pop** (oculto quando `git.lfs_enabled` está desativado, e em um repositório em que o LFS nem foi inicializado nem é usado pelo `.gitattributes`: veja [Configurações > LFS](settings.md#específico-do-repositório-lfs)). Um clique abre uma lista suspensa: **Fetch de todos os arquivos LFS** (`lfs fetch --all`), **Checkout de todos os arquivos LFS** (`lfs checkout`), **Pull de todos os arquivos LFS** (`lfs pull`), **Push de todos os arquivos LFS** (`lfs push --all`, os objetos de todas as refs) e **Limpar o armazenamento LFS local** (`lfs prune`). Cada linha também é um comando da paleta (`LFS: Fetch de todos os objetos`, `LFS: checkout de todos os arquivos`, `LFS: Pull de objetos`, `LFS: Push de todos os objetos`, `LFS: limpar objetos locais`). `LFS: abrir menu` abre a lista suspensa. Cada um tem um acorde `Ctrl+G`: `l` para o menu, depois `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` e `Shift+V` para as linhas. Você pode alterá-los em *Configurações > Teclado e mouse*. Com o `git-lfs` ausente, toda linha leva ao diálogo de instalação.

![A lista suspensa LFS da barra de ferramentas: fetch, checkout, pull e push de todos os arquivos LFS, e limpeza do armazenamento local](../images/lfs-menu.png)

**Pull e push cuidam do LFS por conta própria.** Em um repositório que usa LFS, com o `git-lfs` instalado, um `pull` simples antes confere se os filtros LFS do repositório estão instalados, e um `push` simples confere o hook `pre-push` dele. Quando um está ausente, o GitPoulpi executa `git lfs install --local` (só este repositório) antes da transferência, e a linha de status diz “filtros LFS instalados”. Sem o hook, um push não enviaria nenhum objeto LFS. Desative com `git.lfs_autofix_filters = false` (*Configurações > Git e codificação > Instalar os filtros LFS antes de pull e push*). Um `push` simples continua enviando apenas os objetos da branch enviada, nunca os de todas as refs; o prune nunca é implícito.

## Bloqueios de arquivo

Alguns arquivos não podem ser mesclados (um arquivo do Photoshop, uma fase de jogo, um asset binário). Os *bloqueios* do Git LFS deixam uma pessoa dizer “estou editando isto”: ninguém mais pode bloqueá-lo nem fazer push de alterações nele até que seja desbloqueado.

```
gitpoulpi lfs locks  [--remote origin]              # quem tem o bloqueio de cada arquivo (os seus aparecem marcados)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- O bloqueio é um recurso do **servidor LFS** (GitHub, GitLab e Gitea o têm). Ele é acessado com as credenciais do próprio remoto; o GitPoulpi não tem conta própria.
- Um servidor sem bloqueio faz o `lock` falhar com uma mensagem que diz isso (o tipado `LfsLockingUnsupported`). `locks` em um servidor assim não lista nada, que é o que o próprio git-lfs informa.
- Rastrear um padrão com `--lockable` (`lfs track '*.psd' --lockable`) deixa esses arquivos somente leitura até que você os bloqueie.
- `unlock` do bloqueio de outra pessoa é recusado; `--force` o quebra, e é por isso que o MCP trata `lfs_unlock` como uma ferramenta destrutiva. `lfs_locks` é somente leitura.

## Clonando um repositório LFS

`gitpoulpi clone <url>` (e o diálogo Clonar da TUI) faz o checkout do repositório *sem* executar os filtros LFS, depois, quando o repositório usa LFS e o `git-lfs` está instalado, baixa os objetos com `git lfs pull` (instalando os filtros para esse repositório se nenhum estiver configurado) e mostra isso como uma fase própria, *Baixando objetos LFS*. `--no-lfs` pula essa etapa e deixa os ponteiros.

Um clone **nunca falha por causa do LFS**: com o `git-lfs` ausente, ou um download que falha, o clone ainda tem sucesso e os arquivos grandes continuam como ponteiros; o status LFS do repositório diz o motivo, e `gitpoulpi lfs pull` conclui o trabalho quando o problema for corrigido.

## Na interface de terminal

- **O selo `LFS`.** Um arquivo guardado com Git LFS tem um pequeno selo `LFS` depois do nome nas listas Unstaged e Staged do painel Status e na lista de arquivos alterados de um commit.
- **O cartão do diff.** Abrir o diff de um arquivo LFS mostra um cartão em vez do texto do ponteiro: `Objeto Git LFS`, `1.2 MB -> 1.4 MB`, os oids curtos antigo e novo, e se o novo objeto está guardado localmente ou `não baixado`. Pressione `r` para o diff bruto do ponteiro, `r` de novo para voltar. Só o staging em nível de arquivo se aplica a um arquivo LFS (não há hunks nem linhas para fazer stage). Sob o cartão, **Abrir no programa padrão** e **Abrir em** (`o`, `Shift+O` ou a paleta) abrem o arquivo real, nunca o ponteiro: veja [Abrir em](open-in.md#arquivos-git-lfs).
- **A Visualização de arquivo** de um arquivo LFS diz `Objeto Git LFS, 12.3 MB, sha256:ab12cd34...` e onde obter o conteúdo.
- **O aviso.** Para um repositório que usa LFS enquanto o `git-lfs` está ausente ou é anterior à 3.0, um aviso no painel Status informa. Clique nela (ou use a entrada da paleta *LFS: abrir página de instalação*) para abrir https://git-lfs.com. Nada mais muda: a janela abre, os diffs funcionam, os arquivos grandes simplesmente aparecem como ponteiros.
- **O menu do arquivo e a paleta.** Clique com o botão direito em um arquivo alterado no painel Status: *Rastrear com LFS: este arquivo* / *esta extensão* para um arquivo comum; *Bloquear arquivo (LFS)*, *Desbloquear arquivo (LFS)* ou *Forçar desbloqueio (LFS, bloqueio de outra pessoa)* para um arquivo LFS, conforme quem detém o bloqueio. A linha de um arquivo bloqueado diz `bloqueado por você` / `bloqueado por <nome>`. Tudo também é um comando da paleta (`Ctrl+P`, digite `LFS`): inicializar, rastrear ou parar de rastrear um padrão, fetch, pull ou push de objetos, prune, mostrar bloqueios, abrir a página de instalação.
- **Uma ação que falha se explica.** Se uma ação LFS falha porque o `git-lfs` está ausente, um diálogo traz a mensagem e um botão *Abrir página de instalação*.
- **Configurações.** *Configurações > LFS* (por repositório) mostra se o LFS está inicializado, lista os padrões rastreados e tem **Configurar o LFS aqui**. *Git e codificação* (por repositório) tem a chave **Git LFS**, uma linha somente leitura **Programa git-lfs** e um botão **Inicializar o LFS neste repositório**.
- **Git LFS** desativado (`git.lfs_enabled = false`, veja a [referência de configuração](config-reference.md)) oculta tudo isso. Os filtros LFS do próprio git continuam funcionando.
- O estado do LFS é lido logo depois que o repositório abre, depois de cada alteração, em uma atualização manual, quando o `.gitattributes` muda, e a cada dez segundos em um repositório que usa LFS.

## Limites conhecidos

- Um repositório cujos padrões LFS ficam apenas em um `.gitattributes` *aninhado* (não o da raiz, nem `.git/info/attributes`) é detectado como usuário de LFS quando passa a ter um armazenamento de objetos LFS (`.git/lfs`), não antes.
- O próprio `git lfs migrate` não é usado. Mover arquivos para dentro ou para fora do LFS ao longo do histórico é feito pelas regras *Migrar arquivos para o LFS* / *Trazer arquivos de volta do LFS* da reescrita de histórico: elas atuam no clone local, só o `.gitattributes` da raiz é escrito, e os objetos ainda precisam ser enviados depois (*Enviar objetos LFS* no painel de resultado, ou `lfs push --all`).
