[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · **Português (Portugal)** · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

Um worktree é uma segunda cópia em checkout do repositório: outra pasta, noutra
branch, com o mesmo histórico. A secção WORKTREES do painel Refs lista-os; o
principal aparece marcado com `(principal)`.

## Mostrar a secção

A secção fica oculta num repositório que ainda não tem nenhum worktree ligado.
Para a ver na mesma (passa a dizer "Ainda sem worktrees: + para criar um"), use
o comando da paleta **Painel esquerdo: mostrar a secção Worktrees**, marque
**WORKTREES** na lista de mostrar/ocultar da barra lateral (clique direito no
cabeçalho de uma secção) ou execute qualquer um dos comandos de criação abaixo.
O GitPoulpi lembra-se disso para esse repositório (veja **Secções do painel
esquerdo** em [Definições](settings.md)); outro repositório sem worktrees
continua a começar com a secção oculta.

## Criar um worktree

O **+** no cabeçalho de WORKTREES (teclado: `+` na secção; paleta: **Refs:
criar worktree...**) abre um menu:

- **A partir da branch atual**: a branch está em checkout na pasta principal,
  por isso o diálogo propõe uma nova branch sobre ela (`<branch>-wt`).
- **A partir do commit selecionado**: fica a cinzento até haver um commit
  selecionado no Log. O worktree começa aí com HEAD desanexado, a menos que dê
  nome a uma nova branch.
- **Escolher uma branch...**: o diálogo abre na sua lista **Ref**.

A paleta tem os mesmos três (**Refs: criar worktree a partir da branch atual**,
**Git: criar worktree no commit selecionado...**, **Refs: criar worktree
escolhendo uma branch**). As entradas mais antigas continuam a funcionar:
**Criar worktree a partir de...** no menu de uma branch, tag ou commit.

O diálogo propõe tudo e deixa-o alterar:

| Campo | Significado |
|---|---|
| **Nome** | A pasta é construída a partir dele. Proposto a partir da branch ou do id curto. |
| **Pasta** | Vizinha do repositório, `<repositório>-<nome>`. Acompanha o Nome até escrever na Pasta; **Procurar** escolhe onde. |
| **Ref** | A branch ou o commit de onde o worktree parte (uma lista; escreva para pesquisar). Mudá-la propõe um novo Nome. |
| **Nova branch** | Vazio faz checkout da branch da Ref lá (ou desanexa num commit). Obrigatório quando a branch já está em checkout. |
| **Abrir o novo worktree como separador** | Ligado por predefinição. |

Uma pasta que não está vazia, um nome inválido ou uma branch que já existe são
indicados na linha do próprio diálogo, e **Criar** fica a cinzento.

## O menu da linha

Clique com o botão direito num worktree (ou selecione-o e prima a tecla de menu,
`Shift+F10` ou `.`):

- **Abrir em novo separador**: como um clique duplo ou `Enter`.
- **Abrir em**: as entradas de [Abrir em](open-in.md) que têm marcada a caixa
  **Pode abrir um worktree** (Definições > Abrir em; todas as entradas vêm
  marcadas por predefinição). Correm com a pasta do worktree como `${cwd}`.
- **Merge de <branch> em...**: escolha a branch para onde fazer o merge (por
  predefinição, a do worktree principal). O merge normal corre no worktree onde
  essa branch está em checkout, com as suas opções de merge; um conflito
  resolve-se aí pelo fluxo habitual de conflitos. Uma branch que não está em
  checkout em lado nenhum é recusada com o motivo. Não é oferecido para um
  worktree com HEAD desanexado.
- **Remover worktree...**: nunca o principal.

A paleta tem as mesmas ações sobre a linha selecionada (**Refs: abrir worktree
em...**, **Refs: merge da branch do worktree em...**, **Refs: remover
worktree...**, **Refs: abrir worktree num separador**).

## Remover um worktree

Um worktree limpo recebe uma confirmação simples. Se a sua branch já tem o merge
feito (no seu upstream, na branch do worktree principal ou em `main` /
`master`), a caixa **Eliminar também a branch <b> (o merge em <destino> já foi
feito)** vem marcada por predefinição; uma branch sem merge indica que se mantém.
**Merge em... e depois remover** faz primeiro o merge e só remove quando o merge
terminou sem conflitos.

Com alterações por registar (com ou sem seguimento), o diálogo lista-as
(contagem, com deslocamento) e oferece:

- **Commit e remover**: um campo **Mensagem do commit** no diálogo; todas as
  alterações vão para stage (respeitando o `.gitignore`) e fazem commit na
  branch do worktree, e depois o worktree é removido. Um worktree com HEAD
  desanexado dá nome a uma branch para o commit no mesmo diálogo.
- **Stash e remover**: todas as alterações, incluindo os ficheiros sem
  seguimento, vão para um stash com uma mensagem editável (por predefinição "WIP
  em <pasta>"), e depois o worktree é removido. A lista de stashes é partilhada
  por todos os worktrees: o stash fica nos STASHES do repositório principal,
  pronto a restaurar.
- **Abrir para commit**: abre o worktree num separador com a mensagem de commit
  em foco; nada é removido.
- **Remover mesmo assim**: descarta as alterações depois de uma segunda
  confirmação explícita.

## Desfazer

Remover um worktree não se pode desfazer: a pasta desaparece. O que foi guardado
antes fica: o commit está na sua branch, o stash na lista. A eliminação da
branch que **Eliminar também a branch** acrescenta é a normal, registada no
diário: **Desfazer** repõe a branch (não a pasta). Criar um worktree também não
faz parte do histórico.

## A partir de um script ou de um agente

`gitpoulpi worktree changes <path>` lista as alterações por registar de um
worktree. `gitpoulpi worktree remove <path> [--force] [--commit-message M
[--commit-branch B]] [--stash-message M] [--delete-branch]` remove-o; as
ferramentas MCP são `worktree_changes` e `worktree_remove`. Veja a
[referência da CLI](cli-reference.md).
