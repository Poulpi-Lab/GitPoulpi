[English](../getting-started.md) · [Čeština](../cs/getting-started.md) · [Deutsch](../de/getting-started.md) · [Español](../es/getting-started.md) · [Français](../fr/getting-started.md) · [Italiano](../it/getting-started.md) · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · [Nederlands](../nl/getting-started.md) · [Polski](../pl/getting-started.md) · [Português (Brasil)](../pt-BR/getting-started.md) · **Português (Portugal)** · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · [简体中文](../zh-CN/getting-started.md)

# Primeiros passos

Esta página leva-o de uma instalação nova até ao seu primeiro commit. A
[visita guiada à interface](interface-tour.md) dá nome a cada zona da janela.
As [Definições](settings.md) listam todas as opções.

## 1. Instalar e iniciar

Siga a [Instalação](installation.md). Depois abra um terminal num repositório
Git e execute:

```
gitpoulpi-tui
```

`gitpoulpi` sem subcomando faz o mesmo. O GitPoulpi abre o repositório que
contém a pasta atual. Fora de um repositório, abre o separador **Início**, onde
pode procurar, clonar ou inicializar um repositório (veja o
[passo 7](#7-abrir-clonar-e-inicializar-repositórios)).

Os ícones precisam de uma Nerd Font ou de um tipo de letra com emoji. Se vir
quadrados, consulte a [Resolução de problemas](troubleshooting.md).

![A janela principal: a barra lateral Refs à esquerda, o grafo do Log ao centro e as alterações em curso com o formulário de commit à direita](../images/hero.png)

## 2. Três teclas para aprender tudo o resto

| Tecla | Abre |
|---|---|
| `?` | A **ajuda**: todas as teclas e gestos do rato. Escreva para filtrar. `Ctrl+D` abre esta documentação no seu navegador. |
| `Ctrl+P` | A **paleta de comandos**: escreva para encontrar um comando, `Enter` executa-o. Cada linha mostra o seu atalho. |
| `Ctrl+,` | As **Definições**. A roda dentada na barra de título, à esquerda do ×, também as abre. |

Pode fazer tudo com o teclado ou com o rato. A paleta chega a todos os
comandos. Veja os [Atalhos de teclado](keybindings.md) e o
[Rato e gestos](mouse-and-gestures.md).

![A paleta de comandos filtrada pela palavra stage, listando os comandos de stage com os seus atalhos](../images/palette.png)

## 3. Ver o histórico

O centro da janela é o **Log**: uma linha por commit, do mais recente para o
mais antigo. Mostra as branches e as tags como etiquetas coloridas, o grafo, a
mensagem, o autor, a data e o hash curto.

- Clique num commit. Os seus detalhes e os ficheiros alterados aparecem à direita.
- Clique num ficheiro alterado. O seu diff abre ao centro. `Esc` fecha-o.
- `j` / `k` ou as setas movem a seleção. `f` pesquisa nos commits.
- Clique com o botão direito num commit para ver o seu menu.

![O Log com um commit de merge selecionado: a sua mensagem, o autor e os dois ficheiros que adiciona, à direita](../images/log-graph.png)

## 4. Fazer uma alteração e fazer stage

Edite um ficheiro no seu editor. A **linha de topo do Log** mostra agora as
alterações em curso: uma linha tracejada com o número de ficheiros alterados.
Clique nela. O painel da direita mostra **Ficheiros unstaged** e **Ficheiros
staged**.

- Passe o rato sobre um ficheiro e clique no botão **Stage ficheiro**. Ou
  clique em **Stage tudo**.
- Clique no nome de um ficheiro para abrir o seu diff. Aí pode fazer stage de
  um **hunk** (**Stage hunk**) ou de uma **linha**.

Veja [Stage e commit](staging-and-committing.md).

![O diff de um ficheiro modificado, com os botões Stage hunk e Descartar hunk por cima de cada hunk](../images/diff.png)

## 5. Fazer commit

Escreva um resumo, e opcionalmente uma descrição, no formulário de commit em
baixo à direita. Depois clique no botão verde **Commit das alterações em N
ficheiros** ou prima `Ctrl+Enter` (ou `Ctrl+G Enter`, a partir de qualquer lugar). Assinale **Amend do commit anterior** para
juntar as suas alterações ao último commit em vez de criar um novo.

Se ativou a [IA](ai.md), o botão `✦` junto ao resumo escreve uma mensagem a
partir das alterações em stage. O GitPoulpi nunca faz commit até clicar em
**Commit das alterações em N ficheiros**.

## 6. Partilhar o seu trabalho

A barra de ferramentas tem os botões **Pull** e **Push**, cada um com uma lista
`▾`. Pelo teclado, prima `Ctrl+G` e depois uma letra: `Ctrl+G p` faz pull,
`Ctrl+G Shift+P` faz push. Depois de `Ctrl+G`, a barra de estado lista as letras.

O GitPoulpi usa a autenticação que o seu `git` já usa (credential helper,
ssh-agent). Não precisa de uma conta própria. Para ver e criar pull requests,
ligue uma conta de alojamento uma só vez. Veja
[Integrações de alojamento e de gestores de issues](hosting-integration.md).

## 7. Abrir, clonar e inicializar repositórios

Clique no ícone da casa na extremidade esquerda da linha de separadores, ou
prima `Ctrl+O`. O separador **Início** lista os seus repositórios abertos,
favoritos e recentes, com os botões **Procurar**, **Clonar** e **Init**. Cada
repositório abre no seu próprio separador. `Ctrl+W` fecha o separador atual.

Nos três diálogos (e no seletor de pastas que eles abrem), `Ctrl+N` mostra um selo em cada campo e botão: prima o carácter para pôr o cursor no campo ou premir o botão.

![O separador Início com a lista de repositórios abertos e recentes e os botões Procurar, Clonar e Init](../images/home.png)

**Clonar** pede um URL e uma pasta, ou deixa-o escolher um repositório de uma conta ligada (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; veja [Clonar a partir de uma conta ligada](hosting-integration.md#clonar-a-partir-de-uma-conta-ligada)). **Init** transforma uma pasta num novo
repositório.

| ![A caixa de diálogo Clonar: para onde clonar, o URL, o nome da pasta e o botão Clonar](../images/clone.png) | ![A caixa de diálogo Inicializar um repositório: a pasta, o nome e a branch inicial](../images/init.png) |
|---|---|

### Checkout parcial

Um repositório grande não tem de chegar inteiro ao disco. Na caixa de diálogo
Clonar, assinale **Checkout parcial (apenas algumas pastas)** e escreva as pastas
que quer, separadas por espaços ou vírgulas (`src docs/guide`). Só essas pastas e
os ficheiros da raiz do repositório passam por checkout; o histórico completo é
transferido na mesma, por isso o Log, o blame e as branches funcionam como
sempre. Com a caixa assinalada, é preciso pelo menos uma pasta.

![A caixa de diálogo Checkout parcial com duas pastas escritas e os botões Cancelar, Desativar e Aplicar](../images/sparse-checkout.png)

Qualquer repositório pode ser alterado mais tarde: abra a paleta de comandos
(`Ctrl+P`) e execute **Repositório: checkout parcial...**, ou use **Checkout
parcial...** no menu do separador do repositório. A caixa de diálogo lista num
único campo as pastas que estão em checkout: adicione ou retire nomes e prima
**Aplicar** (`Enter`), ou **Desativar (checkout completo)** para trazer todos os
ficheiros de volta. Enquanto a árvore de trabalho é parcial, a barra de estado
mostra um pequeno distintivo `parcial`. Os mesmos controlos são uma página de
*Definições > Checkout parcial* (uma lista de pastas, uma por linha, com os botões **Iniciar**, **Parar** e **Atualizar**).

Notas:

- É o modo cone do git (pastas, não padrões); `*`, `..` e caminhos absolutos são
  recusados.
- O Desfazer não regista a alteração do conjunto: nada se perde, e **Desativar**
  repõe todos os ficheiros. O git pode recusar-se a retirar uma pasta que tem
  alterações não commitadas (o erro aparece na barra de estado); faça primeiro
  commit ou stash.
- O mesmo a partir de scripts e agentes: `gitpoulpi clone <url> --sparse src docs`
  e `gitpoulpi sparse-checkout status | set | add | disable` (veja a
  [referência da CLI](cli-reference.md)); o servidor MCP tem as mesmas
  ferramentas.

## Para a frente

- [Visita guiada à interface](interface-tour.md): o nome de tudo o que vê.
- [Trabalhar com o Log](log-and-history.md) e
  [Stage e commit](staging-and-committing.md): o trabalho do dia a dia.
- [Temas](themes.md) e [Definições](settings.md).
- [Assistência de IA](ai.md) e [Integrações de alojamento e de gestores de issues](hosting-integration.md):
  as duas integrações opcionais.
- [Referência da CLI](cli-reference.md) e [MCP e agentes](mcp-and-agents.md):
  as mesmas operações a partir de scripts e agentes.
- [Resolução de problemas](troubleshooting.md).
