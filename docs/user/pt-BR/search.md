[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · [Español](../es/search.md) · [Français](../fr/search.md) · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Busca

`Ctrl+Alt+F` (ou `Ctrl+P`, depois `Buscar: espaço de trabalho...`) abre um buscador
aproximado (fuzzy) sobre **todos os repositórios em que você está trabalhando**: os
repositórios do [espaço de trabalho](workspaces.md) ativo, quando há um, senão as
abas abertas. O botão **Busca global** (a lupa e seu rótulo na barra de título, logo à
esquerda da engrenagem) também o abre. Ele encontra:

- **repositórios** (pelo nome, depois pelo caminho),
- **branches** (locais e de rastreamento remoto) e **tags**,
- **worktrees** (pela pasta, depois pelo caminho e pela branch),
- **stashes** (pela mensagem),
- **commits** (pelo assunto; o corpo de um commit só é buscado quando o assunto não
  coincide e você digitou três caracteres ou mais),
- **pull ou merge requests** e **issues** (pelo título, `#número`, autor e branch).

![O popup de busca sobre dois repositórios abertos: branches, um stash e um commit que coincidem com "main", cada um com seu repositório à direita](../images/workspace-search.png)

Os resultados são agrupados por tipo, com a melhor coincidência primeiro dentro de
cada grupo. Cada linha tem o ícone do tipo, os caracteres coincidentes realçados, um
detalhe esmaecido (o upstream de uma branch, o autor e a branch de uma solicitação, a
posição de um stash) e o nome do repositório à direita, de modo que dois repositórios
com uma branch `main` nunca se confundem. Um indicador de atividade aparece no título
quando um repositório ainda está indexando seus commits (a primeira busca em um
repositório grande pode levar um instante); os outros tipos aparecem na hora e os
commits entram assim que ficam prontos.

## Indo até lá

| Tecla | Faz |
|---|---|
| `Cima` / `Baixo`, `Ctrl+P` / `Ctrl+N`, `PageUp` / `PageDown`, `Home` / `End` | Move pelos resultados (os títulos dos grupos são pulados). |
| `Enter` | Vai até o resultado: a aba do repositório abre (ou é ativada) e então o resultado é mostrado. |
| `Esc` | Fecha a busca. |
| `Alt+1` .. `Alt+8` | Liga ou desliga um tipo (as etiquetas acima da lista, na ordem delas). `Alt+0` liga todos. |

O que "ir até lá" significa:

| Resultado | Enter |
|---|---|
| Branch ou tag | Seleciona-a no painel Refs e seu commit no Log. |
| Commit | Seleciona-o no Log, carregando antes mais do Log quando ele é mais antigo do que o carregado. |
| Stash | Seleciona sua linha na seção STASHES. |
| Worktree | Abre-o como uma aba. |
| Pull/merge request, issue | Seleciona sua linha na barra lateral (na aba daquele repositório). |
| Repositório | Troca para a sua aba (abrindo-a quando não está aberta). |

`Ctrl+N Backspace` (ou `Voltar` na paleta) retorna para onde você estava, como em
qualquer mudança de visão feita por um agente por você. Nada é feito checkout, mesclado
ou alterado por ir a algum lugar.

Com o mouse: um clique em uma linha a seleciona, um clique duplo vai até lá, um clique
em uma etiqueta liga ou desliga aquele tipo, a roda rola e um clique fora fecha o popup.

## Restringindo com um prefixo

Comece a consulta com um prefixo para buscar um só tipo, não importa o que as etiquetas
digam:

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

`fix: login` é uma consulta comum, não um prefixo: só conta uma única letra (ou `pr:` /
`mr:`) seguida de dois-pontos, ou um `#` no início.

## Como ele coincide

As mesmas três camadas da paleta de comandos: seu texto literal, depois literal em
qualquer ponto, depois seus caracteres em ordem mas separados. Uma coincidência mais
cedo e um texto mais curto vencem. Maiúsculas e minúsculas são iguais.

## Pull requests e issues

Elas vêm do que o GitPoulpi já carregou para cada repositório (as listas por trás das
seções PULL REQUESTS e ISSUES da barra lateral), guardadas em um pequeno arquivo de cache
na pasta `.gitpoulpi/` do repositório. A busca **nunca chama o GitHub nem o GitLab**: um
repositório cujas solicitações nunca foram carregadas em uma janela não tem nenhuma para
encontrar, e o que você vê é tão recente quanto a última atualização da barra lateral.
Sem uma conta vinculada não há nada a encontrar, e nada a configurar.

## Pela linha de comando

`gitpoulpi search <consulta> [--workspace ID] [--kind K]... [--limit N]` faz o mesmo e
imprime JSON (veja a [referência da CLI](cli-reference.md#espaços-de-trabalho-e-busca));
o servidor MCP tem uma ferramenta `search`, então um agente encontra o que você
encontra. `search.commit_limit` em
[Configurações](config-reference.md#search-a-busca-no-espaço-de-trabalho) limita quantos
commits de cada repositório são indexados (`0` desliga a busca de commits).

## Mudando a tecla

`Buscar: espaço de trabalho...` é um comando da paleta e a tecla é a ação
`global.workspace_search`: reatribua-a em Configurações > Teclado e mouse, ou na tabela
`[keybindings.global]` (`workspace_search = "Ctrl+Alt+F"`).
