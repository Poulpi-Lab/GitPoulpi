[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

Um worktree é uma segunda cópia com checkout do repositório: outra pasta, em outra
branch, compartilhando o mesmo histórico. A seção WORKTREES do painel Refs os lista; o
principal é marcado com `(main)`.

## Mostrando a seção

A seção fica oculta em um repositório que ainda não tem nenhum worktree vinculado. Para
vê-la mesmo assim (ela então mostra "Nenhum worktree ainda: + para criar um"), use o
comando da paleta **Painel esquerdo: mostrar a seção Worktrees**, marque **WORKTREES** na
lista de mostrar/ocultar do painel esquerdo (clique com o botão direito em um cabeçalho de
seção) ou execute qualquer um dos comandos de criação abaixo. O GitPoulpi lembra isso para
aquele repositório (veja **Seções do painel esquerdo** em [Configurações](settings.md));
outro repositório sem worktrees continua começando com a seção oculta.

## Criando um worktree

O **+** no cabeçalho WORKTREES (teclado: `+` na seção; paleta: **Refs: criar
worktree...**) abre um menu:

- **A partir da branch atual**: a branch está em checkout na pasta principal, então o
  diálogo propõe uma nova branch sobre ela (`<branch>-wt`).
- **A partir do commit selecionado**: fica esmaecido até que um commit seja selecionado
  no Log. O worktree começa desanexado ali, a menos que você dê um nome a uma nova branch.
- **Escolher uma branch...**: o diálogo abre na lista **Ref**.

A paleta tem os mesmos três (**Refs: criar worktree a partir da branch atual**, **Git:
criar worktree no commit selecionado...**, **Refs: criar worktree escolhendo uma
branch**). As entradas mais antigas continuam funcionando: **Criar worktree a partir
de...** no menu de uma branch, tag ou commit.

O diálogo propõe tudo e deixa você mudar:

| Campo | Significado |
|---|---|
| **Nome** | A pasta recebe esse nome. Proposto a partir da branch ou do id curto. |
| **Pasta** | Uma irmã do repositório, `<repositório>-<nome>`. Ela acompanha o Nome até você digitar na Pasta; **Procurar** escolhe onde. |
| **Ref** | A branch ou o commit de onde o worktree parte (uma lista; digite para buscar). Mudá-la propõe um novo Nome. |
| **Nova branch** | Vazio faz checkout da branch da Ref ali (ou desanexa em um commit). Obrigatório quando a branch já está em checkout. |
| **Abrir o novo worktree como aba** | Ativado por padrão. |

Uma pasta que não está vazia, um nome inválido ou uma branch que já existe são
informados na linha do próprio diálogo, e **Criar** continua esmaecido.

## O menu da linha

Clique com o botão direito em um worktree (ou selecione-o e pressione a tecla de menu,
`Shift+F10` ou `.`):

- **Abrir em uma nova aba**: como um clique duplo ou `Enter`.
- **Abrir em**: as entradas de [Abrir em](open-in.md) cuja caixa **Pode abrir um
  worktree** está marcada (Configurações > Abrir em; toda entrada vem marcada por padrão).
  Elas rodam com a pasta do worktree como `${cwd}`.
- **Merge <branch> em...**: escolha a branch que vai receber o merge (por padrão, a
  branch do worktree principal). O merge comum roda no worktree em que essa branch está em
  checkout, com as suas opções de merge; um conflito é resolvido ali pelo fluxo de
  conflitos de sempre. Uma branch que não está em checkout em lugar nenhum é recusada,
  com o motivo. Não é oferecido para um worktree desanexado.
- **Remover worktree...**: nunca o principal.

A paleta tem as mesmas ações sobre a linha selecionada (**Refs: abrir o worktree em...**,
**Refs: Merge da branch do worktree em...**, **Refs: remover worktree...**, **Refs: abrir
worktree em uma aba**).

## Removendo um worktree

Um worktree limpo recebe uma confirmação simples. Se a branch dele já passou por merge
(no upstream dela, na branch do worktree principal ou em `main` / `master`), a caixa
**Excluir também a branch <b>** vem marcada por padrão; uma branch sem merge avisa que
continua. **Merge em... e remover** faz o merge primeiro e só remove quando o merge
terminou sem conflitos.

Com alterações pendentes (rastreadas ou não), o diálogo as lista (contagem, com rolagem)
e oferece:

- **Fazer commit e remover**: um campo de mensagem no próprio diálogo; toda alteração é
  preparada (respeitando o `.gitignore`) e vira commit na branch do worktree, e depois o
  worktree é removido. Um worktree desanexado dá nome a uma branch para o commit no mesmo
  diálogo.
- **Fazer stash e remover**: toda alteração, inclusive os arquivos não rastreados, vai
  para um stash com uma mensagem editável (padrão "WIP em <pasta>"), e depois o worktree
  é removido. A lista de stashes é compartilhada por todos os worktrees: o stash fica
  depois em STASHES do repositório principal, pronto para o pop.
- **Abrir para commit**: abre o worktree em uma aba com a mensagem de commit em foco;
  nada é removido.
- **Remover mesmo assim**: descarta as alterações depois de uma segunda confirmação
  explícita.

## Desfazer

Remover um worktree não pode ser desfeito: a pasta se foi. O que foi salvo antes
continua: o commit está na branch, o stash na lista. A exclusão de branch que **Excluir
também a branch** acrescenta é a comum, registrada no diário: **Desfazer** traz a branch
de volta (não a pasta). Criar um worktree também não faz parte do histórico.

## Por um script ou um agente

`gitpoulpi worktree changes <path>` lista as alterações pendentes de um worktree.
`gitpoulpi worktree remove <path> [--force] [--commit-message M [--commit-branch B]]
[--stash-message M] [--delete-branch]` o remove; as ferramentas MCP são
`worktree_changes` e `worktree_remove`. Veja a [referência da CLI](cli-reference.md).
