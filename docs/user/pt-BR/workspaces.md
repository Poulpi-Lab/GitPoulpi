[English](../workspaces.md) · [Čeština](../cs/workspaces.md) · [Deutsch](../de/workspaces.md) · [Español](../es/workspaces.md) · [Français](../fr/workspaces.md) · [Italiano](../it/workspaces.md) · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · [Nederlands](../nl/workspaces.md) · [Polski](../pl/workspaces.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/workspaces.md) · [Русский](../ru/workspaces.md) · [Svenska](../sv/workspaces.md) · [Türkçe](../tr/workspaces.md) · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Espaços de trabalho

Um **espaço de trabalho** é um grupo nomeado de repositórios: aqueles em que você
trabalha em conjunto (um front end, sua API, a documentação). Ele lembra quais deles
estão abertos como abas e pode ter um **perfil de commit padrão**, de modo que os
commits nesses repositórios usem a identidade certa sem você trocar de perfil à mão.
Um repositório pode estar em vários espaços de trabalho, e não estar em nenhum também
é normal: "Sem espaço de trabalho" é a simples lista de abas de sempre.

Os espaços de trabalho ficam em `workspaces.toml`, ao lado de `config.toml`, e são
compartilhados por todas as sessões, pela janela, pela CLI e pelo MCP
(`gitpoulpi workspace ...`, veja a [referência da CLI](cli-reference.md)). Um espaço de trabalho adicionado, renomeado ou removido a partir de um shell ou por `gitpoulpi workspace ...` com a janela
aberta aparece sozinho em cerca de um segundo (a aba Espaços de trabalho, a etiqueta da barra de abas e a lista dela), sem
pressionar tecla alguma. Fechar ou
excluir um espaço de trabalho nunca toca em um repositório no disco. As edições de
espaços de trabalho são configurações, como favoritos e perfis: não fazem parte de
[Desfazer e refazer](undo-redo.md), e Excluir pergunta antes.

## A etiqueta do espaço de trabalho

Assim que você tem um espaço de trabalho, uma etiqueta aparece na barra de abas logo
depois do `⌂`: um ponto colorido e o nome do espaço de trabalho ativo, ou "Sem espaço
de trabalho" (esmaecido). Em uma barra estreita ela encolhe para o ponto, e em uma
barra muito estreita desaparece.

![A lista suspensa de espaços de trabalho sobre a barra de abas, com Sem espaço de trabalho, Orbit (marcado), Side projects, Archive e Gerenciar espaços de trabalho, com a etiqueta Orbit e seu perfil "Work - do espaço de trabalho Orbit" na barra de ferramentas](../images/workspace-switch.png)

- **Clique nela**, pressione `Alt+W` ou execute `Espaço de trabalho: trocar...` na
  paleta para abrir uma lista suspensa: seus espaços de trabalho, "Sem espaço de
  trabalho" e "Gerenciar espaços de trabalho..." (que abre a aba Espaços de trabalho).
- **Trocar troca as abas.** As abas que você tinha abertas são guardadas para o
  espaço de trabalho que você deixa, e as abas do destino voltam (na primeira vez,
  seus repositórios que existem). Volte e tudo está onde você deixou. A janela
  reinicia como em uma troca de aba; o daemon continua.
- **O espaço de trabalho é lembrado**: a próxima inicialização volta nele.
- **Fechar a última aba** de um espaço de trabalho mantém o espaço ativo e mostra a
  aba Início.
- **O `+` (nova aba) com um espaço de trabalho ativo** adiciona o repositório que
  você escolher à lista de abas e não pergunta nada. Para colocá-lo no próprio espaço
  de trabalho, use "Adicionar ao espaço de trabalho" no menu de clique com o botão
  direito da aba (ou `Espaço de trabalho: adicionar a aba ao espaço de trabalho`);
  "Remover do espaço de trabalho" o tira de novo e deixa a aba aberta.

Sem nenhum espaço de trabalho ainda, não há etiqueta: crie um primeiro pela aba Espaços de trabalho
ou pela paleta.

## A aba Espaços de trabalho

Abra-a pelo menu do `+`, com `Alt+J` ou com `Espaço de trabalho: gerenciar...` na paleta; o seu `x` (ou `Ctrl+W`) a fecha, e ela continua aberta entre as sessões até você fechá-la. Cada espaço de
trabalho é uma linha com seu ponto colorido, nome, quantos repositórios lista (e
quantos estão ausentes), seu perfil padrão e "ativo" quando a janela está nele.
`Direita`, `Espaço` ou um clique o abre até seus repositórios.

![A aba Espaços de trabalho: Orbit aberto até seus dois repositórios, Side projects e Archive marcado com "1 ausente"](../images/workspaces.png)

| Por espaço de trabalho | Tecla | |
|---|---|---|
| Abrir | `Enter` | Torna-o ativo e abre suas abas. |
| Editar... | `e` | O formulário abaixo. |
| Excluir... | `x` | Pergunta antes. Os repositórios não são tocados. |
| Fetch tudo | `Shift+F` | Um repositório depois do outro, progresso na linha de status, falhas listadas no fim. |
| Pull tudo | `Shift+P` | O mesmo, só com fast-forward: nunca para em um merge. |
| Clonar os ausentes | `m` | Clona todo repositório ausente que tenha um remoto registrado. |
| Novo espaço de trabalho... | `n` | |
| A partir das abas abertas... | `a` | Um novo espaço de trabalho com suas abas abertas marcadas. |

Um repositório **ausente** (sua pasta sumiu) continua listado, marcado, com
**Localizar...** (`l` ou `Enter`: indique onde ele está agora), **Clonar** (quando a
URL de `origin` foi registrada ao adicioná-lo) e **Remover** (`x`). Toda linha também
tem um menu `⋮` (clique com o botão direito, `.` ou `Shift+F10`) com as mesmas ações,
e toda ação está na paleta em `Espaço de trabalho:`.

## O formulário Novo / Editar

![O formulário Editar espaço de trabalho: nome, cor, descrição, perfil padrão e a lista de repositórios](../images/workspace-edit.png)

- **Nome**: obrigatório, único.
- **Cor**: uma lista suspensa de cores do tema; o ponto da etiqueta e das linhas.
- **Descrição**: texto livre.
- **Perfil**: uma lista suspensa dos seus `[[profiles]]`, ou "Nenhum". É opcional.
- **✦ Instruções para a IA**: só com a IA ativada, veja [abaixo](#instruções-de-ia-de-um-espaço-de-trabalho).
- **Repositórios**: uma lista de marcação de todo repositório que o GitPoulpi conhece
  (abas abertas, favoritos, recentes, os do próprio espaço de trabalho, inclusive os
  ausentes). `Espaço` marca uma linha; **Adicionar pasta...** (`Ctrl+B`) escolhe um
  repositório do disco. A URL de origem de cada novo repositório é lida do repositório
  e registrada (sem nenhuma credencial), e é o que Clonar usa depois.

## Instruções de IA de um espaço de trabalho

Com a IA ativada (*Configurações > IA*), o formulário tem uma linha **✦ Instruções para a IA** com um botão **Editar...** (`Enter` na
linha). Ele abre um pequeno editor do que a IA recebe em **todo repositório do espaço de trabalho**: escolha *Toda ação de
IA* ou uma ação (mensagem de commit, explicar, mensagem de stash, pull request, compositor de commits, resolução de conflitos,
revisão de código) na lista **Para**, escolha **Combinar** - *Somar ao global* (enviado depois das instruções globais) ou
*Substituir o global* (enviado no lugar delas) - e escreva o texto. **Aplicar** guarda o conjunto todo, **Salvar** no formulário
o grava em `workspaces.toml` junto com o espaço de trabalho; **Cancelar** descarta o que você digitou. Com a IA desativada a linha fica
oculta e as instruções do espaço de trabalho ficam como estão.

As palavras do espaço de trabalho ficam entre as instruções globais e as do próprio repositório (veja
[Instruções de IA](ai.md#instruções) para a ordem e para o que *substituir* faz). Um repositório que pertence a
vários espaços de trabalho usa o espaço de trabalho ativo da janela quando ele o contém, senão o primeiro que o contém. Por
script: `gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] (--text ... |
--clear)`; `workspace show` as imprime como `ai_instructions`.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # toda ação de IA neste espaço de trabalho
text = "Use the Acme glossary."
mode = "add"                       # ou "replace"

[workspace.ai_overrides.code_review]   # uma ação
text = "Flag missing tests."
mode = "add"
```

## O perfil padrão

Os commits são assinados pelo primeiro destes que se aplicar:

1. o perfil que você escolheu **à mão** para aquele repositório (o campo de perfil da
   barra de ferramentas),
2. o perfil do **espaço de trabalho ativo**, quando o repositório é um dos seus
   membros,
3. `general.active_profile`.

O campo de perfil da barra de ferramentas diz de onde ele vem: "Work - do espaço de
trabalho Acme". O campo também aparece quando você tem um único perfil, desde que o
padrão de um espaço de trabalho esteja em vigor. Um espaço de trabalho cujo perfil não
existe mais simplesmente deixa de se aplicar. Na CLI, `commit --workspace <id>` (ou
`GITPOULPI_WORKSPACE`) resolve da mesma forma.

## Teclas

| Tecla | Ação |
|---|---|
| `Alt+W` | Abre a lista suspensa de espaços de trabalho |
| `Alt+J` | Abrir a aba Espaços de trabalho |
| `n`, `a` | Novo espaço de trabalho, novo a partir das abas abertas |
| `e` | Editar o espaço de trabalho |
| `Shift+F`, `Shift+P`, `m`, `l` | Fetch tudo, Pull tudo, Clonar os ausentes, Localizar |

Todas podem ser reatribuídas em Configurações > Teclado e mouse; a lista completa está em
[Atalhos de teclado](keybindings.md).
