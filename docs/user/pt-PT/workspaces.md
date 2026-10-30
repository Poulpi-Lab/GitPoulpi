[English](../workspaces.md) · [Čeština](../cs/workspaces.md) · [Deutsch](../de/workspaces.md) · [Español](../es/workspaces.md) · [Français](../fr/workspaces.md) · [Italiano](../it/workspaces.md) · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · [Nederlands](../nl/workspaces.md) · [Polski](../pl/workspaces.md) · [Português (Brasil)](../pt-BR/workspaces.md) · **Português (Portugal)** · [Русский](../ru/workspaces.md) · [Svenska](../sv/workspaces.md) · [Türkçe](../tr/workspaces.md) · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Áreas de trabalho

Uma **área de trabalho** é um grupo nomeado de repositórios: os que trabalha em
conjunto (um front end, a sua API, a documentação). Lembra-se de quais estão
abertos como separadores e pode ter um **perfil de commit predefinido**, para que
os commits nesses repositórios usem a identidade certa sem a mudar à mão. Um
repositório pode estar em várias áreas de trabalho, e não estar em nenhuma também
está bem: «Sem área de trabalho» é a simples lista de separadores de sempre.

As áreas de trabalho ficam guardadas em `workspaces.toml`, junto a `config.toml`,
e são partilhadas por todas as sessões, pela janela, pela CLI e pelo MCP
(`gitpoulpi workspace ...`, ver a [referência da CLI](cli-reference.md)). Uma área de trabalho adicionada, renomeada ou removida a partir de uma shell ou de `gitpoulpi workspace ...` com a janela
aberta aparece por si só em cerca de um segundo (o separador Áreas de trabalho, a etiqueta da barra de separadores e a sua lista), sem
premir nenhuma tecla. Fechar
ou eliminar uma área de trabalho nunca toca num repositório no disco. As edições
das áreas de trabalho são definições, como os favoritos e os perfis: não fazem
parte de [Desfazer e refazer](undo-redo.md), e Eliminar pergunta primeiro.

## A etiqueta da área de trabalho

Assim que tiver uma área de trabalho, aparece uma etiqueta na barra de separadores,
logo a seguir ao `⌂`: um ponto colorido e o nome da área de trabalho ativa, ou
«Sem área de trabalho» (esbatido). Numa barra estreita reduz-se ao ponto, e numa
barra muito estreita desaparece.

![A lista de áreas de trabalho sobre a barra de separadores, com Sem área de trabalho, Orbit (marcada), Side projects, Archive e Gerir áreas de trabalho, com a etiqueta Orbit e o seu perfil «Work - da área de trabalho Orbit» na barra de ferramentas](../images/workspace-switch.png)

- **Clique nela**, prima `Alt+W` ou execute `Área de trabalho: mudar...` na paleta
  para abrir uma lista: as suas áreas de trabalho, «Sem área de trabalho» e
  «Gerir áreas de trabalho...» (que abre o separador Áreas de trabalho).
- **Mudar troca os separadores.** Os separadores que tinha abertos ficam guardados
  para a área de trabalho que deixa, e os da área de destino voltam (da primeira
  vez, os seus repositórios que existem). Volte atrás e tudo está onde o deixou.
  Reinicia a janela como uma mudança de separador; o daemon continua.
- **A área de trabalho é lembrada**: o próximo arranque volta a ela.
- **Fechar o último separador** de uma área de trabalho mantém-na ativa e mostra o
  separador Início.
- **O `+` (novo separador) com uma área de trabalho ativa** adiciona o repositório
  que escolher à lista de separadores e não pergunta nada. Para o pôr na própria
  área de trabalho, use «Adicionar à área de trabalho» no menu de clique com o
  botão direito do separador (ou `Área de trabalho: adicionar o separador à área
  de trabalho`); «Remover da área de trabalho» tira-o de novo e deixa o separador
  aberto.

Sem nenhuma área de trabalho ainda, não há etiqueta: crie primeiro uma a partir do
separador Áreas de trabalho ou da paleta.

## O separador Áreas de trabalho

Abra-o pelo menu do `+`, com `Alt+J` ou com `Área de trabalho: gerir...` na paleta; o seu `x` (ou `Ctrl+W`) fecha-o, e continua aberto entre arranques até o fechar. Cada área de
trabalho é uma linha com o seu ponto colorido, o nome, quantos repositórios lista
(e quantos estão em falta), o perfil predefinido e «ativa» quando a janela está
nela. `Right`, `Space` ou um clique desdobra-a até aos seus repositórios.

![O separador Áreas de trabalho: Orbit desdobrada nos seus dois repositórios, Side projects e Archive marcada com «1 em falta»](../images/workspaces.png)

| Por área de trabalho | Tecla | |
|---|---|---|
| Abrir | `Enter` | Torna-a ativa e abre os seus separadores. |
| Editar... | `e` | O formulário abaixo. |
| Eliminar... | `x` | Pergunta primeiro. Os repositórios não são alterados. |
| Fetch tudo | `Shift+F` | Um repositório após o outro, progresso na barra de estado, falhas listadas no fim. |
| Pull tudo | `Shift+P` | O mesmo, apenas fast-forward: nunca pára num merge. |
| Clonar os que faltam | `m` | Clona todos os repositórios em falta que têm um remoto registado. |
| Nova área de trabalho... | `n` | |
| A partir dos separadores abertos... | `a` | Uma nova área de trabalho com os seus separadores abertos marcados. |

Um repositório **em falta** (a sua pasta desapareceu) continua listado, marcado, com
**Localizar...** (`l` ou `Enter`: indique onde está agora), **Clonar** (quando o
URL de `origin` foi registado ao adicioná-lo) e **Remover** (`x`). Cada linha tem
também um menu `⋮` (clique com o botão direito, `.` ou `Shift+F10`) com as mesmas
ações, e todas as ações estão na paleta em `Área de trabalho:`.

## O formulário Nova / Editar

![O formulário Editar área de trabalho: nome, cor, descrição, perfil predefinido e a lista de repositórios](../images/workspace-edit.png)

- **Nome**: obrigatório, único.
- **Cor**: uma lista de cores do tema; o ponto da etiqueta e das linhas.
- **Descrição**: texto livre.
- **Perfil**: uma lista dos seus `[[profiles]]`, ou «Nenhum». É opcional.
- **✦ Instruções da IA**: só com a IA ligada, ver [abaixo](#instruções-de-ia-de-uma-área-de-trabalho).
- **Repositórios**: uma lista de verificação de todos os repositórios que o
  GitPoulpi conhece (separadores abertos, favoritos, recentes, os da própria área
  de trabalho, incluindo os em falta). `Space` marca uma linha; **Adicionar
  pasta...** (`Ctrl+B`) escolhe um repositório do disco. O URL de origem de cada
  novo repositório é lido do repositório e registado (sem qualquer credencial), e é
  isso que Clonar usa mais tarde.

## Instruções de IA de uma área de trabalho

Com a IA ligada (*Definições > IA*), o formulário tem uma linha **✦ Instruções da IA** com um botão **Editar...** (`Enter` na
linha). Abre um pequeno editor do que é dito à IA em **todos os repositórios da área de trabalho**: escolha *Todas as ações da
IA* ou uma ação (mensagem de commit, explicar, mensagem de stash, pull request, compositor de commits, resolução de conflitos,
revisão de código) na lista **Para**, escolha **Combinar** - *Juntar às globais* (enviadas depois das instruções globais) ou
*Substituir as globais* (enviadas em vez delas) - e escreva o texto. **Aplicar** mantém o conjunto todo, **Guardar** no formulário
escreve-o em `workspaces.toml` com a área de trabalho; **Cancelar** descarta o que escreveu. Com a IA desligada a linha fica
oculta e as instruções da área de trabalho ficam como estão.

As palavras da área de trabalho ficam entre as instruções globais e as do próprio repositório (ver
[Instruções](ai.md#instruções) para a ordem e para o que *substituir* faz). Um repositório que pertence a
várias áreas de trabalho usa a área de trabalho ativa da janela quando a contém, senão a primeira que a contenha. A partir de um
script: `gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] (--text ... |
--clear)`; `workspace show` imprime-as como `ai_instructions`.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # todas as ações de IA nesta área de trabalho
text = "Use the Acme glossary."
mode = "add"                       # ou "replace"

[workspace.ai_overrides.code_review]   # uma ação
text = "Flag missing tests."
mode = "add"
```

## O perfil predefinido

Os commits são assinados pelo primeiro destes que se aplicar:

1. o perfil que escolheu **à mão** para esse repositório (o campo de perfil da
   barra de ferramentas),
2. o perfil da **área de trabalho ativa**, quando o repositório é um dos seus
   membros,
3. `general.active_profile`.

O campo de perfil da barra de ferramentas diz de onde vem: «Work - da área de
trabalho Acme». O campo aparece também quando tem um único perfil, desde que o
predefinido de uma área de trabalho esteja em vigor. Uma área de trabalho cujo
perfil já não existe deixa simplesmente de se aplicar. Na CLI,
`commit --workspace <id>` (ou `GITPOULPI_WORKSPACE`) resolve da mesma forma.

## Teclas

| Tecla | Ação |
|---|---|
| `Alt+W` | Abrir a lista de áreas de trabalho |
| `Alt+J` | Abrir o separador Áreas de trabalho |
| `n`, `a` | Nova área de trabalho, nova a partir dos separadores abertos |
| `e` | Editar a área de trabalho |
| `Shift+F`, `Shift+P`, `m`, `l` | Fetch tudo, Pull tudo, Clonar os que faltam, Localizar |

Todas podem ser reatribuídas em Definições > Teclado e rato; a lista completa
está em [Atalhos de teclado](keybindings.md).
