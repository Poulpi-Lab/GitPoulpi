[English](../getting-started.md) · [Čeština](../cs/getting-started.md) · [Deutsch](../de/getting-started.md) · [Español](../es/getting-started.md) · [Français](../fr/getting-started.md) · [Italiano](../it/getting-started.md) · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · [Nederlands](../nl/getting-started.md) · [Polski](../pl/getting-started.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/getting-started.md) · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · [简体中文](../zh-CN/getting-started.md)

# Primeiros passos

Esta página leva você de uma instalação nova até o seu primeiro commit. O
[tour pela interface](interface-tour.md) dá nome a cada área da janela. As
[Configurações](settings.md) listam todas as opções.

## 1. Instalar e abrir

Siga a [Instalação](installation.md). Depois abra um terminal em um repositório
Git e execute:

```
gitpoulpi-tui
```

`gitpoulpi` sem subcomando faz o mesmo. O GitPoulpi abre o repositório que
contém a pasta atual. Fora de um repositório, ele abre a aba **Início**, onde
você pode procurar, clonar ou inicializar um repositório (veja o
[passo 7](#7-abrir-clonar-e-inicializar-repositórios)).

Os ícones precisam de uma Nerd Font ou de uma fonte com emojis. Se você vir
quadrados, veja a [Solução de problemas](troubleshooting.md).

![A janela principal: a barra lateral Refs à esquerda, o grafo do Log no meio e as alterações em andamento com o formulário de commit à direita](../images/hero.png)

## 2. Três teclas para aprender todo o resto

| Tecla | Abre |
|---|---|
| `?` | A **ajuda**: cada tecla e gesto do mouse. Digite para filtrá-la. `Ctrl+D` abre esta documentação no seu navegador. |
| `Ctrl+P` | A **paleta de comandos**: digite para encontrar um comando, `Enter` o executa. Cada linha mostra o seu atalho. |
| `Ctrl+,` | As **Configurações**. A engrenagem na barra de título, à esquerda do ×, também as abre. |

Você pode fazer tudo com o teclado ou com o mouse. A paleta alcança todos os
comandos. Veja os [Atalhos de teclado](keybindings.md) e
[Mouse e gestos](mouse-and-gestures.md).

![A paleta de comandos filtrada pela palavra stage, listando comandos de staging com seus atalhos](../images/palette.png)

## 3. Olhar o histórico

O meio da janela é o **Log**: uma linha por commit, do mais novo ao mais antigo.
Ele mostra as branches e as tags como pílulas coloridas, o grafo, a mensagem, o
autor, a data e o hash curto.

- Clique em um commit. Seus detalhes e os arquivos alterados aparecem à direita.
- Clique em um arquivo alterado. O diff dele abre no meio. `Esc` o fecha.
- `j` / `k` ou as setas movem a seleção. `f` busca nos commits.
- Clique com o botão direito em um commit para ver o menu dele.

![O Log com um commit de merge selecionado: sua mensagem, autor e os dois arquivos que ele adiciona, à direita](../images/log-graph.png)

## 4. Fazer uma alteração e fazer stage dela

Edite um arquivo no seu editor. A **linha superior do Log** agora mostra as
alterações em andamento: uma linha tracejada com o número de arquivos alterados.
Clique nela. O painel direito mostra **Arquivos unstaged** e **Arquivos
staged**.

- Passe o ponteiro sobre um arquivo e clique no botão **Stage arquivo**. Ou
  clique em **Stage tudo**.
- Clique no nome de um arquivo para abrir o diff dele. Ali você pode fazer stage
  de um **hunk** (**Stage hunk**) ou de uma **linha**.

Veja [Stage e commit](staging-and-committing.md).

![O diff de um arquivo modificado, com botões Stage hunk e Descartar hunk acima de cada hunk](../images/diff.png)

## 5. Fazer commit

Digite um resumo, e opcionalmente uma descrição, no formulário de commit no
canto inferior direito. Depois clique no botão verde de commit (**Commit das
alterações em N arquivos**) ou pressione `Ctrl+Enter` (ou `Ctrl+G Enter`, de qualquer lugar). Marque **Amend do commit
anterior** para adicionar suas alterações ao último commit em vez de criar um
novo.

Se você ativou a [IA](ai.md), o botão `✦` ao lado do resumo escreve uma
mensagem a partir das alterações em stage. O GitPoulpi nunca faz commit até que
você clique no botão de commit.

## 6. Compartilhar seu trabalho

A barra de ferramentas tem os botões **Pull** e **Push**, cada um com uma lista
`▾`. Pelo teclado, pressione `Ctrl+G` e depois uma letra: `Ctrl+G p` faz pull,
`Ctrl+G Shift+P` faz push. Depois de `Ctrl+G`, a linha de status lista as letras.

O GitPoulpi usa a autenticação que o seu `git` já usa (auxiliar de credenciais,
ssh-agent). Ele não precisa de conta própria. Para ver e criar pull requests,
conecte uma conta de hospedagem uma única vez. Veja
[Integrações de hospedagem e de rastreadores de issues](hosting-integration.md).

## 7. Abrir, clonar e inicializar repositórios

Clique no ícone da casa na ponta esquerda da fileira de abas, ou pressione
`Ctrl+O`. A aba **Início** lista seus repositórios abertos, favoritos e
recentes, com os botões **Procurar**, **Clonar** e **Init**. Cada repositório
abre em sua própria aba. `Ctrl+W` fecha a aba atual.

![A aba Início com a lista de repositórios abertos e recentes e os botões Procurar, Clonar e Init](../images/home.png)

**Clonar** pede uma URL e uma pasta, ou deixa você escolher um repositório de uma conta conectada (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; veja [Clonar de uma conta conectada](hosting-integration.md#clonar-de-uma-conta-conectada)). **Init** transforma uma pasta em um novo
repositório. Nos três diálogos (e no seletor de pastas que eles abrem), `Ctrl+N` mostra um selo em cada campo e botão: pressione o caractere para pôr o cursor no campo ou acionar o botão.

| ![O diálogo Clonar: onde clonar, a URL, o nome da pasta e o botão para clonar o repositório](../images/clone.png) | ![O diálogo Inicializar um repositório: a pasta, o nome e a branch inicial](../images/init.png) |
|---|---|

### Checkout parcial

Um repositório grande não precisa chegar inteiro ao disco. No diálogo Clonar,
marque **Checkout parcial (apenas alguns diretórios)** e digite os diretórios
que você quer, separados por espaços ou vírgulas (`src docs/guide`). Só esses
diretórios e os arquivos da raiz do repositório passam por checkout; o histórico
completo é baixado do mesmo jeito, então o Log, o blame e as branches funcionam
como sempre. Com a caixa marcada, é preciso pelo menos um diretório.

![O diálogo Checkout parcial com dois diretórios digitados e os botões Cancelar, Desativar e Aplicar](../images/sparse-checkout.png)

Qualquer repositório pode ser mudado depois: abra a paleta de comandos
(`Ctrl+P`) e execute **Repositório: checkout parcial...**, ou use **Checkout
parcial...** no menu da aba do repositório. O diálogo lista num único campo os
diretórios que estão em checkout: adicione ou remova nomes e pressione
**Aplicar** (`Enter`), ou **Desativar (checkout completo)** para trazer todos os
arquivos de volta. Enquanto a árvore de trabalho é parcial, a linha de status
mostra um pequeno selo `parcial`. Os mesmos controles formam uma página de
*Configurações > Checkout parcial* (uma lista de caminhos, um por linha, com os
botões **Iniciar checkout parcial**, **Encerrar checkout parcial** e
**Atualizar checkout parcial**).

Observações:

- É o modo cone do git (diretórios, não padrões); `*`, `..` e caminhos absolutos
  são recusados.
- Desfazer não registra a mudança do conjunto: nada se perde, e **Desativar**
  restaura todos os arquivos. O git pode se recusar a tirar um diretório que
  tem alterações não commitadas (o erro aparece na linha de status); faça commit
  ou stash antes.
- A mesma coisa a partir de scripts e agentes: `gitpoulpi clone <url> --sparse src
  docs` e `gitpoulpi sparse-checkout status | set | add | disable` (veja a
  [referência da CLI](cli-reference.md)); o servidor MCP tem as mesmas
  ferramentas.

## Próximos passos

- [Tour pela interface](interface-tour.md): o nome de tudo o que você vê.
- [Trabalhando com o Log](log-and-history.md) e
  [Stage e commit](staging-and-committing.md): o trabalho diário.
- [Temas](themes.md) e [Configurações](settings.md).
- [Assistência de IA](ai.md) e
  [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md):
  as duas integrações opcionais.
- [Referência da CLI](cli-reference.md) e [MCP e agentes](mcp-and-agents.md):
  as mesmas operações a partir de scripts e agentes.
- [Solução de problemas](troubleshooting.md).
