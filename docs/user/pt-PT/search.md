[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · [Español](../es/search.md) · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · **Português (Portugal)** · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Pesquisa

`Ctrl+Alt+F` (ou `Ctrl+P` e depois `Pesquisar: área de trabalho...`) abre um
localizador aproximado sobre **todos os repositórios em que está a trabalhar**: os
repositórios da [área de trabalho](workspaces.md) ativa, quando há uma, senão os
separadores abertos. O botão **Pesquisa global** (a lupa e a sua etiqueta na barra de título, logo à
esquerda da roda dentada) também o abre. Encontra:

- **repositórios** (pelo nome, depois pelo caminho),
- **branches** (locais e de seguimento remoto) e **tags**,
- **worktrees** (pela pasta, depois pelo caminho e pela branch),
- **stashes** (pela mensagem),
- **commits** (pelo assunto; o corpo de um commit só é pesquisado quando o assunto
  não coincide e escreveu três carateres ou mais),
- **pull ou merge requests** e **issues** (pelo título, `#número`, autor e branch).

![O popup de pesquisa sobre dois repositórios abertos: branches, um stash e um commit que coincidem com «main», cada um com o seu repositório à direita](../images/workspace-search.png)

Os resultados são agrupados por tipo, com a melhor correspondência primeiro dentro
de cada grupo. Cada linha tem o ícone do tipo, os carateres coincidentes realçados,
um detalhe esbatido (o upstream de uma branch, o autor e a branch de um pedido, a
posição de um stash) e o nome do repositório à direita, para que dois repositórios
com uma branch `main` nunca se confundam. Aparece um indicador de atividade no
título quando um repositório ainda está a indexar os seus commits (a primeira
pesquisa num repositório grande pode demorar um momento); os outros tipos estão
logo lá e os commits juntam-se assim que ficam prontos.

## Ir até lá

| Tecla | Faz |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `PageUp` / `PageDown`, `Home` / `End` | Move-se pelos resultados (os títulos dos grupos são saltados). |
| `Enter` | Vai ao resultado: o separador do repositório abre (ou é ativado) e depois o resultado é mostrado. |
| `Esc` | Fecha a pesquisa. |
| `Alt+1` .. `Alt+8` | Liga ou desliga um tipo (as etiquetas acima da lista, pela sua ordem). `Alt+0` liga-os todos. |

O que «ir até lá» significa:

| Resultado | Enter |
|---|---|
| Branch ou tag | Seleciona-o no painel Refs e o seu commit no Log. |
| Commit | Seleciona-o no Log, carregando antes mais Log quando é mais antigo do que o carregado. |
| Stash | Seleciona a sua linha na secção STASHES. |
| Worktree | Abre-o como separador. |
| Pull/merge request, issue | Seleciona a sua linha na barra lateral (no separador desse repositório). |
| Repositório | Muda para o seu separador (abrindo-o quando não está aberto). |

`Ctrl+N Backspace` (ou `Voltar` na paleta) regressa ao sítio onde estava, como
qualquer mudança de vista que um agente faz por si. Nada é feito checkout, integrado
ou alterado por ir a algum lado.

Com o rato: um clique numa linha seleciona-a, um clique duplo vai até lá, um clique
numa etiqueta liga ou desliga esse tipo, a roda desloca, e um clique fora fecha o
popup.

## Restringir com um prefixo

Comece a consulta com um prefixo para pesquisar apenas um tipo, digam o que
disserem as etiquetas:

| Prefixo | Tipo |
|---|---|
| `r:` | repositórios |
| `b:` | branches |
| `t:` | tags |
| `w:` | worktrees |
| `s:` | stashes |
| `c:` | commits |
| `pr:` (ou `mr:`, `p:`) | pull ou merge requests |
| `i:` | issues |
| `#` | pull/merge requests e issues (`#12` encontra o número 12) |

`fix: login` é uma consulta normal, não um prefixo: só conta uma única letra (ou
`pr:` / `mr:`) seguida de dois pontos, ou um `#` inicial.

## Como corresponde

Os mesmos três níveis da paleta de comandos: o seu texto tal e qual, depois tal e
qual em qualquer sítio, depois os seus carateres por ordem mas separados. Uma
correspondência mais cedo e um texto mais curto ganham. Maiúsculas e minúsculas são
iguais.

## Pull requests e issues

Vêm do que o GitPoulpi já carregou para cada repositório (as listas por trás das
secções PULL REQUESTS e ISSUES da barra lateral), guardado num pequeno ficheiro de
cache na pasta `.gitpoulpi/` do repositório. A pesquisa **nunca contacta o GitHub
nem o GitLab**: um repositório cujos pedidos nunca foram carregados numa janela não
tem nenhum para encontrar, e o que vê está tão atual como a última atualização da
barra lateral. Sem uma conta ligada não há nada a encontrar, e nada a configurar.

## A partir da linha de comandos

`gitpoulpi search <consulta> [--workspace ID] [--kind K]... [--limit N]` faz o mesmo
e imprime JSON (ver a [referência da CLI](cli-reference.md#áreas-de-trabalho-e-pesquisa));
o servidor MCP tem uma ferramenta `search`, para que um agente encontre o que você
encontra. `search.commit_limit` nas [Definições](config-reference.md#search-a-pesquisa-na-área-de-trabalho)
limita quantos commits de cada repositório são indexados (`0` desliga a pesquisa de
commits).

## Mudar a tecla

`Pesquisar: área de trabalho...` é um comando da paleta e a tecla é a ação
`global.workspace_search`: reatribua-a em Definições > Teclado e rato, ou na tabela
`[keybindings.global]` (`workspace_search = "Ctrl+Alt+F"`).
