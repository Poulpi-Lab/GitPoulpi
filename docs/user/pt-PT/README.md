[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · **Português (Portugal)** · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# Documentação de utilizador do GitPoulpi

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="O polvo do GitPoulpi" width="128"></p>

O GitPoulpi é um cliente Git para o terminal, com um grafo de commits clicável,
uma paleta de comandos, integração com plataformas de alojamento e gestores de
issues, ajuda opcional de IA, e uma CLI e um servidor MCP que fazem tudo o que a
interface faz. Estas páginas estão por ordem de leitura: comece pelo topo e salte
para as páginas de referência quando precisar delas.

![A janela principal do GitPoulpi](../images/hero.png)

## Comece aqui

1. **[Instalação](installation.md)**: o que precisa, arquivos de lançamento,
   gestores de pacotes, compilar a partir do código-fonte. Para um binário mais pequeno, veja
   **[Compilações personalizadas](custom-builds.md)**. Como o GitPoulpi se mantém atualizado
   (o indicador de versão, o painel de atualizações, reverter): **[Atualizações](updates.md)**.
   Para o executar num contentor: **[Executar no Docker](docker.md)**. Para
   administradores, feeds internos, espelhos e a política de atualização:
   **[Distribuição empresarial](enterprise-distribution.md)**.
2. **[Primeiros passos](getting-started.md)**: os seus primeiros cinco minutos,
   desde o arranque até ao primeiro commit.
3. **[Visita guiada à interface](interface-tour.md)**: o nome de cada zona da
   janela, com imagens.

## Uso diário

4. **[Trabalhar com o Log](log-and-history.md)**: ler o grafo, pesquisa,
   branches, tags, stashes, worktrees, arrastar refs, rebase, cherry-pick,
   reword e amend, avisos, pull e push, auto-stash.
5. **[Stage e commit](staging-and-committing.md)**: as alterações em curso,
   diffs, fazer stage de hunks e de linhas individuais, escrever commits, o
   compositor de commits, blame, histórico de ficheiro e o editor integrado.
6. **[Imagens](image-view.md)**: imagens na vista Diff e na vista de
   ficheiro, antes e depois, zoom, Editar em.
7. **[Resolver conflitos de merge](merge-conflicts.md)**: o editor de merge (A e
   B lado a lado sobre o Output), escolher linhas e blocos, fichas para
   conflitos binários, LFS, eliminados, de symlink e de submódulo, e como um
   agente propõe uma resolução.
8. **[Desfazer e refazer](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, o que pode e o
   que não pode ser desfeito, quando recusa ou pergunta, até onde recua, como o
   desativar.
9. **[Git LFS](lfs.md)**: ficheiros grandes, acompanhamento, bloqueios, e o que
   acontece sem o `git-lfs` instalado.
10. **[Submódulos](submodules.md)**: a secção SUBMÓDULOS, atualizá-los, e
    mantê-los em sintonia com os checkouts e os pulls.
11. **[Temas](themes.md)**: os temas incluídos, qualquer tema com qualquer paleta
    de faixas, pré-visualização em direto, os seus próprios ficheiros de tema,
    `gitpoulpi theme`.
12. **[Áreas de trabalho](workspaces.md)**: grupos nomeados de repositórios com os
    seus próprios separadores e um perfil de commit predefinido opcional, a
    etiqueta da barra de separadores, o separador Áreas de trabalho, Fetch tudo, Pull tudo, Clonar os que faltam.
13. **[Pesquisa](search.md)**: um único localizador aproximado (`Ctrl+Alt+F`) sobre
    as branches, tags, worktrees, stashes, commits, pull/merge requests e issues de
    todos os repositórios da área de trabalho ou dos separadores abertos.
14. **[O Radar](radar.md)**: os pull/merge requests, issues e trabalho em
    curso de uma área de trabalho (ou dos seus separadores abertos) numa só lista:
    filtros, afixados, adiamentos, e o que cada linha pode fazer.
    Ler e rever um desses pedidos na aplicação, com discussões de comentários,
    uma revisão pendente e Aprovar / Pedir alterações:
    **[Rever um pull request](review.md)**.
15. **[Abrir em](open-in.md)**: a lista da barra de título que abre a pasta do
    separador ativo num terminal, multiplexador, editor ou gestor de ficheiros.
16. **[Seguir a pasta do separador ativo](following-the-tab-folder.md)**:
    "Sair e cd aqui", e o wrapper `gitpoulpi shell-init` (`gp`) para bash, zsh,
    fish e PowerShell.
    Que terminal mostra melhor o GitPoulpi: **[Escolher um terminal](terminals.md)**.
    O que passa pelo tmux, zellij, screen, herdr e painéis de terminal:
    **[Multiplexadores](multiplexers.md)**.

Também: **[Fluxos de trabalho de branches](branching-workflows.md)**, os modelos Git Flow, GitHub
Flow, GitLab Flow e Trunk-based ou as suas próprias bases, padrões de nome de branch
e tipos de tópico (`gitpoulpi flow ...`).

**[Reescrita do histórico](history-rewrite.md)**: o painel de reescrita
(`Ctrl+G Shift+H`) que reescreve autores, datas, mensagens e conteúdo de ficheiros em
muitos commits, com pré-visualização, scripts Lua e um único desfazer.

## Teclado e rato

17. **[Atalhos de teclado](keybindings.md)**: todas as teclas, por zona, e todos
    os comandos da paleta. Na aplicação, `?` mostra a mesma lista e `Ctrl+P`
    encontra qualquer comando.
18. **[Rato e gestos](mouse-and-gestures.md)**: clicar, arrastar, selecionar e
    copiar, barras de deslocação e feedback ao passar o rato.

## Definições e integrações

19. **[Definições](settings.md)**: todas as páginas e opções do ecrã de
    Definições, a caixa de pesquisa, perfis, e como reatribuir atalhos.
20. **[Assistência de IA](ai.md)**: a IA do próprio GitPoulpi (desativada por
    predefinição): um agente local como `claude -p`, Anthropic, OpenAI,
    OpenRouter, Ollama ou LM Studio; instruções; mensagens de commit,
    explicações, descrições de pull requests e o compositor de commits;
    privacidade e resolução de problemas.
21. **[Integrações de alojamento e de gestores de issues](hosting-integration.md)**:
    GitHub, GitLab, Gitea e Forgejo, Azure DevOps, Bitbucket, Gerrit e os
    gestores de issues: tokens e os seus âmbitos, ligar, associar um repositório,
    pull requests e issues, referências nas mensagens de commit, chaves SSH,
    limites conhecidos. Também
    **[Issues locais e Obsidian](local-issues.md)**: uma pasta de notas Markdown
    como origem das issues, sem token, com uma Base do Obsidian.

## Automação

22. **[Referência da CLI](cli-reference.md)**: todos os comandos `gitpoulpi`, com
    `--json` para scripts.
23. **[MCP e agentes](mcp-and-agents.md)**: deixe um agente externo (Claude Code,
    Cursor, Codex, ...) conduzir o GitPoulpi: registar o servidor, `.mcp.json`,
    os interruptores de permissões e as suas predefinições, todas as ferramentas,
    e como um agente lhe mostra coisas na sua janela. A skill de agente pronta a
    usar é [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Referência e ajuda

24. **[Referência de configuração](config-reference.md)**: todas as chaves de
    `config.toml`, a sua predefinição e efeito, as definições por repositório,
    variáveis de ambiente.
25. **[Resolução de problemas e FAQ](troubleshooting.md)**: ícones, desenho do
    grafo, teclas que não chegam, daemons bloqueados, velocidade, ponteiros LFS.

## Capturas de ecrã

Todas as imagens destas páginas são capturas reais de uma compilação de
lançamento, num pequeno repositório de demonstração com autores inventados. As
imagens estão em [`images/`](../images/). O `scripts/docs-screenshots.sh`
compila os repositórios de demonstração e volta a tirá-las. São tiradas no
terminal `kitty`, que desenha o grafo de commits e os avatares dos autores como
imagens. Os terminais sem protocolo gráfico mostram em vez disso um grafo de
caracteres.

## Para contribuidores

O [CLAUDE.md](../../../CLAUDE.md), a [visão geral da arquitetura](../../architecture/overview.md)
e o [roteiro](../../roadmap.md) descrevem o desenho e o que está planeado;
o [CHANGELOG.md](../../../CHANGELOG.md) lista o que mudou em cada lançamento.

---

GitKraken é uma marca do respetivo titular. O GitPoulpi é um projeto independente, sem afiliação, aprovação ou patrocínio por parte deste; o nome é mencionado apenas para descrever compatibilidade e inspiração.
