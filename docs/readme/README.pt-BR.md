[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · **Português (Brasil)** · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="O polvo do GitPoulpi" width="128"></p>

**Um cliente Git para o terminal com o conforto de um aplicativo de desktop.**
O GitPoulpi oferece um grafo de commits completo em que você pode clicar,
arrastar e usar o botão direito, uma paleta de comandos pensada para o teclado
que dá acesso a todas as funções, integração com serviços de hospedagem de código (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, na nuvem ou auto-hospedados) e com issue trackers (Jira, Trello, Linear, YouTrack, Redmine ou uma pasta de notas em Markdown), e as mesmas
operações como CLI automatizável e como servidor MCP, para que um agente de IA o
use do mesmo jeito que você. O GitPoulpi é inspirado no GitKraken e foi reescrito do zero
em Rust: binários nativos, leves em disco e na memória, sem Electron, sem conta e sem telemetria. Roda no seu terminal favorito ([qual?](../../docs/user/pt-BR/terminals.md)), porque alguns de nós vivem lá dentro.

![O GitPoulpi mostrando o grafo de commits com branches, tags e um stash, a barra lateral Refs à esquerda e as alterações em andamento com o formulário de commit à direita](../../docs/user/images/hero.png)

- **Mouse e teclado, na mesma medida.** Clique, dê duplo clique, use o botão
  direito e arraste refs sobre os commits; ou pressione `Ctrl+P` e digite o que
  você quer. Cada gesto do mouse tem seu equivalente no teclado, e todos os
  comandos estão na paleta.
- **Um grafo de commits de verdade.** Branches, merges, tags, stashes e a cópia
  de trabalho em um único Log com rolagem, desenhado como pixels nos terminais
  que o suportam e como caracteres nos demais. Muitos temas de cores escuros e
  claros, avatares de autores, busca.
- **Stage com precisão.** Faça stage, unstage ou descarte um arquivo, uma pasta,
  um hunk ou uma única linha direto do diff. Em linha ou lado a lado, com
  minimapa.
- **Edite o histórico com segurança.** Reescreva ou altere qualquer commit
  (mensagem, autor, datas), faça rebase interativo com uma tela de revisão,
  reescreva muitos commits de uma vez com prévia e um único desfazer, faça
  cherry-pick de vários commits, fast-forward, crie worktrees e patches.
- **Hospedagem e issue trackers integrados.** Explore, faça checkout e crie pull e merge requests (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) e revise as changes do Gerrit. Explore e crie issues, work items e cartões (também Jira, Trello, Linear, YouTrack, Redmine e notas Markdown locais), se quiser em um quadro Kanban. Clone a partir da sua conta e gerencie as chaves SSH.
- **CLI e MCP para agentes.** `gitpoulpi <comando> --json` cobre tudo o que a
  interface faz, e `gitpoulpi-mcp` o oferece como ferramentas MCP. Um daemon em
  segundo plano por repositório mantém todos os frontends ao vivo e
  sincronizados.
- **IA opcional, nunca obrigatória.** Desativada por padrão e sem conta: use sua
  própria chave de API ou uma CLI de agente local como `claude -p`. Gere
  mensagens de commit, explique alterações, redija descrições de pull requests,
  divida suas alterações em andamento em vários commits e esboce regras de
  reescrita do histórico.
- **Git LFS, submódulos, worktrees, patches, blame e histórico de arquivos,
  diffs de imagens, um editor integrado.**

## Capturas de tela

| | |
|---|---|
| ![O grafo do Log com o tema de cores Material e o tom de linha ativado](../../docs/user/images/theme-material.png) **O grafo de commits**, aqui com o tema Material e o tom de linha. | ![O diff de um arquivo modificado com botões Stage hunk e Descartar hunk em cada hunk](../../docs/user/images/diff.png) **O diff**, com stage e descarte por hunk ou por linha. |
| ![A paleta de comandos filtrada pela palavra stash](../../docs/user/images/palette.png) **A paleta de comandos** (`Ctrl+P`) dá acesso a todos os comandos. | ![O compositor de commits propondo quatro commits para as alterações em andamento, prontos para revisão](../../docs/user/images/composer.png) **O compositor de commits** revisa os commits propostos pela IA antes de commitar qualquer coisa. |
| ![O menu de contexto de um commit com ações de cherry-pick, revert, reset, rebase, branch e tag](../../docs/user/images/context-menu.png) **Botão direito** em qualquer item para ver suas ações. | ![A página IA das Configurações com perfis, instruções por ação e a chave de ativação](../../docs/user/images/settings-ai.png) **As Configurações**, com busca, incluindo a página opcional de IA. |

Há mais imagens, com explicações, na [documentação do usuário](../../docs/user/pt-BR/README.md).

## Instalação

Você precisa de um `git` no sistema; `git-lfs` é opcional.

### Linux e macOS (`install.sh`)

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew (macOS, Linux)

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt (Debian, Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop (Windows)

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Instalador do Windows

Baixe `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` da
[versão mais recente](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest) e
execute-o. Para administradores, o MSI:

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

Veja [Executando no Docker](../user/pt-BR/docker.md).

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### Arquivo de release

Baixe-o da
[versão mais recente](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest),
descompacte-o e siga o `INSTALL.txt` que está dentro dele.

### A partir do código-fonte

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Seu próprio arquivo de release, uma compilação personalizada menor e notas por sistema operacional:
[docs/user/pt-BR/installation.md](../user/pt-BR/installation.md) e
[docs/user/pt-BR/custom-builds.md](../user/pt-BR/custom-builds.md).

## Início rápido

```
cd path/to/your/repository
gitpoulpi
```

- `?` abre a ajuda com todas as teclas e gestos, `Ctrl+P` a paleta de comandos,
  `Ctrl+,` as Configurações e `q` sai.
- Clique em um commit para ver suas alterações; clique na linha superior do Log
  (as alterações em andamento) para fazer stage de arquivos e commit.
- Em um terminal que o suporte (kitty, Ghostty, WezTerm, iTerm2 (e também Windows Terminal e foot, via sixel)) o grafo é
  desenhado como pixels. Nos demais (um console do Windows simples, tmux)
  ele volta automaticamente para caracteres.

A versão passo a passo está em [Primeiros passos](../../docs/user/pt-BR/getting-started.md),
e [o tour pela interface](../../docs/user/pt-BR/interface-tour.md) dá nome a cada área
da tela.

```
gitpoulpi --json status            # a CLI, automatizável
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # dá a um agente de IA as mesmas ferramentas
```

A IA é opcional e vem desativada por padrão: ative a IA própria do GitPoulpi em
*Configurações > IA* ([Assistência de IA](../../docs/user/pt-BR/ai.md)); deixe que um
agente externo use o GitPoulpi com
[MCP e agentes](../../docs/user/pt-BR/mcp-and-agents.md) (permissões, todas as
ferramentas) e com a skill de agente
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md).

## Documentação

A documentação completa do usuário começa em **[docs/user/pt-BR/README.md](../../docs/user/pt-BR/README.md)**:

| | |
|---|---|
| [Primeiros passos](../../docs/user/pt-BR/getting-started.md) | Instalar, abrir um repositório, fazer o primeiro commit |
| [Tour pela interface](../../docs/user/pt-BR/interface-tour.md) | Cada área da janela, com seu nome |
| [Trabalhando com o Log](../../docs/user/pt-BR/log-and-history.md) | Grafo, refs, branches, tags, stashes, rebase, cherry-pick |
| [Stage e commit](../../docs/user/pt-BR/staging-and-committing.md) | Status, diff, hunks e linhas, o compositor, o editor |
| [Configurações](../../docs/user/pt-BR/settings.md) | Cada página e opção das Configurações |
| [Assistência de IA](../../docs/user/pt-BR/ai.md) | Configurar a IA integrada, todas as ações de IA, privacidade |
| [Integrações de hospedagem e de gerenciadores de issues](../../docs/user/pt-BR/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: tokens, vincular um repositório, pull e merge requests |
| [CLI](../../docs/user/pt-BR/cli-reference.md) e [MCP](../../docs/user/pt-BR/mcp-and-agents.md) | Automatize, ou deixe um agente usar (permissões, todas as ferramentas) |
| [Atalhos de teclado](../../docs/user/pt-BR/keybindings.md) (e todos os comandos da paleta), [mouse](../../docs/user/pt-BR/mouse-and-gestures.md), [configuração](../../docs/user/pt-BR/config-reference.md), [LFS](../../docs/user/pt-BR/lfs.md), [desfazer](../../docs/user/pt-BR/undo-redo.md), [solução de problemas](../../docs/user/pt-BR/troubleshooting.md) | Referência e ajuda |

Para desenvolvedores: [CLAUDE.md](../../CLAUDE.md) e `docs/architecture/overview.md`
descrevem os crates e o design; [CHANGELOG.md](../../CHANGELOG.md) lista o que cada
release traz.

## Licença

[MIT](../../LICENSE).

## Créditos e licenças de terceiros

O GitPoulpi é construído sobre bibliotecas de código aberto e usa valores de cor de temas publicados. Suas licenças e créditos, incluindo os autores dos temas, estão em [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt). O arquivo acompanha cada versão; a janela Sobre e o comando `gitpoulpi licenses` também o exibem.

## Marcas

GitKraken é uma marca de seu respectivo proprietário. O GitPoulpi é um projeto independente, sem afiliação, endosso ou patrocínio por parte dele; o nome é citado apenas para descrever compatibilidade e inspiração.
