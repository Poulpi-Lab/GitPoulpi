[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · **Português (Portugal)** · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="O polvo do GitPoulpi" width="128"></p>

**Um cliente Git para o terminal com a sensação de uma aplicação de
ambiente de trabalho.** O GitPoulpi dá-lhe um grafo de commits completo em que
pode clicar, arrastar e usar o botão direito, uma paleta de comandos centrada no
teclado que chega a todas as funcionalidades, integração com plataformas de alojamento de código (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, na cloud ou self-managed) e com gestores de issues (Jira, Trello, Linear, YouTrack, Redmine ou uma pasta de notas Markdown), e as mesmas operações como CLI automatizável e como servidor MCP, para
que um agente de IA o possa conduzir exatamente como uma pessoa. O GitPoulpi
inspira-se no GitKraken e foi reescrito de raiz em Rust: binários nativos, leves em disco e em memória, sem Electron, sem conta, sem telemetria. Corre no seu terminal favorito ([qual?](../../docs/user/pt-PT/terminals.md)), porque alguns de nós vivemos lá dentro.

![O GitPoulpi a mostrar o grafo de commits com branches, tags e um stash, a barra lateral Refs à esquerda, e as alterações em curso com o formulário de commit à direita](../../docs/user/images/hero.png)

- **Rato e teclado, por igual.** Clique, duplo clique, botão direito e arraste e
  largue refs sobre commits; ou prima `Ctrl+P` e escreva o que quer. Cada gesto
  do rato tem um equivalente no teclado, e todos os comandos estão na paleta.
- **Um verdadeiro grafo de commits.** Branches, merges, tags, stashes e a cópia
  de trabalho num só Log deslizante, desenhado em píxeis nos terminais que o
  suportam e em caracteres em todos os outros. Muitos temas de cores escuros e
  claros, avatares de autores, pesquisa.
- **Stage com precisão.** Faça stage, unstage ou descarte um ficheiro, uma
  pasta, um hunk ou uma única linha diretamente a partir do diff. Em linha ou
  lado a lado, com um minimapa.
- **Edite o histórico em segurança.** Reformule ou altere qualquer commit
  (mensagem, autor, datas), faça rebase interativo com um ecrã de revisão,
  reescreva muitos commits de uma vez com uma pré-visualização e um só desfazer,
  faça cherry-pick de vários commits e fast-forward, crie worktrees e patches.
- **Plataformas de alojamento e gestores de issues integrados.** Navegue, faça checkout e crie pull e merge requests (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) e reveja as changes do Gerrit. Navegue e crie issues, itens de trabalho e cartões (também Jira, Trello, Linear, YouTrack, Redmine e notas Markdown locais), se quiser num quadro Kanban. Clone a partir da sua conta e gira chaves SSH.
- **CLI e MCP para agentes.** `gitpoulpi <command> --json` cobre tudo o que a
  interface faz, e `gitpoulpi-mcp` serve-o como ferramentas MCP. Um daemon em
  segundo plano por repositório mantém todos os frontends em direto e
  sincronizados.
- **IA opcional, nunca obrigatória.** Desativada por predefinição e sem
  precisar de conta: traga a sua própria chave de API ou uma CLI de agente local
  como `claude -p`. Gere mensagens de commit, explique alterações, escreva
  descrições de pull requests, divida as suas alterações em curso em vários
  commits e prepare regras de reescrita do histórico.
- **Git LFS, submódulos, worktrees, patches, blame e histórico de ficheiro,
  diffs de imagens, um editor integrado.**

## Capturas de ecrã

| | |
|---|---|
| ![O grafo do Log com o tema de cores Material e o tom de linha ativado](../../docs/user/images/theme-material.png) **O grafo de commits**, aqui no tema Material com tom de linha. | ![O diff de um ficheiro modificado com os botões Stage hunk e Descartar hunk em cada hunk](../../docs/user/images/diff.png) **O diff**, com stage e descarte por hunk ou por linha. |
| ![A paleta de comandos filtrada pela palavra stash](../../docs/user/images/palette.png) **A paleta de comandos** (`Ctrl+P`) chega a todos os comandos. | ![O compositor de commits a propor quatro commits para as alterações em curso, prontos para revisão](../../docs/user/images/composer.png) **O compositor de commits** revê os commits propostos pela IA antes de se fazer qualquer commit. |
| ![O menu de contexto de um commit com ações de cherry-pick, revert, reset, rebase, branch e tag](../../docs/user/images/context-menu.png) **Clique com o botão direito** em qualquer coisa para ver as suas ações. | ![A página IA das Definições com perfis, instruções por ação e o interruptor de ativação](../../docs/user/images/settings-ai.png) **As Definições**, com pesquisa, incluindo a página opcional de IA. |

Mais imagens, com explicações, estão na [documentação de utilizador](../../docs/user/pt-PT/README.md).

## Instalar

Precisa de um `git` do sistema; o `git-lfs` é opcional.

- **Com um instalador:** um `...-setup.exe` para Windows ou `install.sh` para Linux e macOS (por utilizador, sem root), anexado a cada versão. Uma compilação instalada a partir de um arquivo de versão também se pode atualizar sozinha a partir do chip de versão da barra de estado.
- **A partir de um arquivo de lançamento:** descompacte-o e siga o `INSTALL.txt`
  que está lá dentro.
- **A partir do código-fonte:**

  ```
  git clone https://github.com/Poulpi-Lab/GitPoulpi.git
  cd GitPoulpi
  cargo build --release
  ```

  Os binários ficam em `target/release/`: `gitpoulpi` (CLI, e a interface
  quando executado sem subcomando), `gitpoulpi-tui` e `gitpoulpi-mcp`.
- **Criar o seu próprio arquivo de lançamento:** `cargo run -p xtask -- dist`
  produz `target/dist/gitpoulpi-<version>-<target>.zip` (Windows) ou `.tar.gz`,
  com um ficheiro `.sha256`.
- **Binário mais pequeno só com o que precisa:** `./build-custom.sh` (Windows:
  `build-custom`) pergunta que idiomas, integrações e funcionalidades (IA, MCP,
  LFS, worktrees, submódulos, transferência de avatares) compilar; veja
  [docs/user/pt-PT/custom-builds.md](../../docs/user/pt-PT/custom-builds.md).

Todos os detalhes, ferramentas opcionais e notas por SO:
[docs/user/pt-PT/installation.md](../../docs/user/pt-PT/installation.md).

## Início rápido

```
cd path/to/your/repository
gitpoulpi-tui          # or just: gitpoulpi
```

- `?` abre a ajuda com todas as teclas e gestos, `Ctrl+P` a paleta de comandos,
  `Ctrl+,` as Definições, `q` sai.
- Clique num commit para ver as suas alterações; clique na linha de topo do Log
  (as alterações em curso) para fazer stage de ficheiros e commit.
- Num terminal que o suporte (kitty, Ghostty, WezTerm, iTerm2 (e também Windows Terminal e foot, através de sixel)) o grafo é
  desenhado em píxeis. Nos outros (uma consola simples do Windows, tmux)
  recorre automaticamente a caracteres.

A versão passo a passo está nos [Primeiros passos](../../docs/user/pt-PT/getting-started.md),
e a [visita guiada à interface](../../docs/user/pt-PT/interface-tour.md) dá nome a cada
zona do ecrã.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

A IA é opcional e está desativada por predefinição: ative a IA do próprio
GitPoulpi em *Definições > IA* ([Assistência de IA](../../docs/user/pt-PT/ai.md)); deixe
um agente externo conduzir o GitPoulpi com [MCP e agentes](../../docs/user/pt-PT/mcp-and-agents.md)
(permissões, todas as ferramentas) e a skill de agente
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md).

## Documentação

A documentação de utilizador completa começa em **[docs/user/pt-PT/README.md](../../docs/user/pt-PT/README.md)**:

| | |
|---|---|
| [Primeiros passos](../../docs/user/pt-PT/getting-started.md) | Instalar, abrir um repositório, fazer um primeiro commit |
| [Visita guiada à interface](../../docs/user/pt-PT/interface-tour.md) | Cada zona da janela, com o seu nome |
| [Trabalhar com o Log](../../docs/user/pt-PT/log-and-history.md) | Grafo, refs, branches, tags, stashes, rebase, cherry-pick |
| [Stage e commit](../../docs/user/pt-PT/staging-and-committing.md) | Estado, diff, hunks e linhas, o compositor, o editor |
| [Definições](../../docs/user/pt-PT/settings.md) | Cada página e opção das Definições |
| [Assistência de IA](../../docs/user/pt-PT/ai.md) | Configurar a IA integrada, todas as ações de IA, privacidade |
| [Integrações de alojamento e de gestores de issues](../../docs/user/pt-PT/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: tokens, associar um repositório, pull e merge requests |
| [CLI](../../docs/user/pt-PT/cli-reference.md) e [MCP](../../docs/user/pt-PT/mcp-and-agents.md) | Automatize-o, ou deixe um agente conduzi-lo (permissões, todas as ferramentas) |
| [Atalhos de teclado](../../docs/user/pt-PT/keybindings.md) (e todos os comandos da paleta), [rato](../../docs/user/pt-PT/mouse-and-gestures.md), [configuração](../../docs/user/pt-PT/config-reference.md), [LFS](../../docs/user/pt-PT/lfs.md), [desfazer](../../docs/user/pt-PT/undo-redo.md), [resolução de problemas](../../docs/user/pt-PT/troubleshooting.md) | Referência e ajuda |

Para programadores: o [CLAUDE.md](../../CLAUDE.md) e `docs/architecture/overview.md`
descrevem os crates e o desenho; o [CHANGELOG.md](../../CHANGELOG.md) lista o que há em
cada lançamento.

## Licença

[MIT](../../LICENSE).

## Créditos e licenças de terceiros

O GitPoulpi assenta em bibliotecas de código aberto e usa valores de cor de temas publicados. As suas licenças e créditos, incluindo os autores dos temas, estão em [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt). O ficheiro acompanha cada versão; a janela Acerca de e o comando `gitpoulpi licenses` também o apresentam.

## Marcas

GitKraken é uma marca do respetivo titular. O GitPoulpi é um projeto independente, sem afiliação, aprovação ou patrocínio por parte deste; o nome é mencionado apenas para descrever compatibilidade e inspiração.
