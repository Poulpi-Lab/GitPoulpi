[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# Documentação do usuário do GitPoulpi

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="O polvo do GitPoulpi" width="128"></p>

O GitPoulpi é um cliente Git para o terminal com um grafo de commits em que se
pode clicar, uma paleta de comandos, integração com serviços de hospedagem e
issue trackers, ajuda de IA opcional, e uma CLI e um servidor MCP que fazem tudo
o que a interface faz. Estas páginas estão em ordem de leitura: comece pelo topo
e pule para as páginas de referência quando precisar delas.

![A janela principal do GitPoulpi](../images/hero.png)

## Comece aqui

1. **[Instalação](installation.md)**: o que você precisa, arquivos de release,
   gerenciadores de pacotes, compilar a partir do código-fonte. Para um binário
   menor, veja **[Compilações personalizadas](custom-builds.md)**. Como o
   GitPoulpi se mantém atualizado (o chip de versão, o painel de atualização,
   voltar à versão anterior): **[Atualizações](updates.md)**. Para executá-lo em
   um contêiner: **[Executando no Docker](docker.md)**. Para administradores,
   feeds internos, espelhos e a política de atualização:
   **[Distribuição corporativa](enterprise-distribution.md)**.
2. **[Primeiros passos](getting-started.md)**: seus primeiros cinco minutos, da
   abertura até o primeiro commit.
3. **[Tour pela interface](interface-tour.md)**: o nome de cada área da janela,
   com imagens.

## Uso diário

4. **[Trabalhando com o Log](log-and-history.md)**: ler o grafo, busca,
   branches, tags, stashes, worktrees, arrastar refs, rebase, cherry-pick,
   reword e alteração de commits, banners, pull e push, auto-stash.
5. **[Stage e commit](staging-and-committing.md)**: as alterações em andamento,
   diffs, stage de hunks e de linhas avulsas, escrever commits, o compositor de
   commits, blame, histórico de arquivos e o editor integrado.
6. **[Imagens](image-view.md)**: imagens na visualização Diff e na
   Visualização de arquivo, antes e depois, zoom, Editar em.
7. **[Resolvendo conflitos de merge](merge-conflicts.md)**: o editor de merge (A
   e B lado a lado sobre o resultado), escolher linhas e blocos, cartões para
   conflitos binários, de LFS, de arquivos excluídos, de links simbólicos e de
   submódulos, e como um agente propõe uma resolução.
8. **[Desfazer e refazer](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, o que pode e o
   que não pode ser desfeito, quando recusa ou pergunta, até onde chega, como
   desativar.
9. **[Git LFS](lfs.md)**: arquivos grandes, rastreamento, bloqueios, e o que
   acontece sem o `git-lfs` instalado.
10. **[Submódulos](submodules.md)**: a seção SUBMÓDULOS, como atualizá-los e
    como mantê-los no ritmo dos checkouts e dos pulls.
11. **[Temas](themes.md)**: os temas integrados, qualquer tema com qualquer
    paleta de faixas, pré-visualização ao vivo, seus próprios arquivos de tema,
    `gitpoulpi theme`.
12. **[Espaços de trabalho](workspaces.md)**: grupos nomeados de repositórios com
    suas próprias abas e um perfil de commit padrão opcional, a etiqueta da barra
    de abas, a aba Espaços de trabalho, Fetch tudo, Pull tudo,
    Clonar os ausentes.
13. **[Busca](search.md)**: um só buscador aproximado (`Ctrl+Alt+F`) sobre as
    branches, tags, worktrees, stashes, commits, pull/merge requests e issues de
    todos os repositórios do espaço de trabalho ou das abas abertas.
14. **[O Radar](radar.md)**: as pull/merge requests, issues e o trabalho
    em andamento de um espaço de trabalho (ou das suas abas abertas) em uma só
    lista: filtros, itens fixados, adiamentos e o que cada linha pode fazer.
    Ler e revisar uma dessas solicitações dentro do aplicativo, com threads de
    comentários, uma revisão pendente e Aprovar / Solicitar alterações:
    **[Revisando um pull request](review.md)**.
15. **[Abrir em](open-in.md)**: a lista da barra de título que abre a pasta da
    aba ativa em um terminal, um multiplexador, um editor ou um gerenciador de
    arquivos.
16. **[Seguindo a pasta da aba ativa](following-the-tab-folder.md)**:
    “Sair e cd aqui”, e a função `gitpoulpi shell-init` (`gp`) para bash, zsh,
    fish e PowerShell.
    Qual terminal mostra melhor o GitPoulpi: **[Escolhendo um terminal](terminals.md)**.
    O que passa pelo tmux, zellij, screen, herdr e painéis de terminal:
    **[Multiplexadores](multiplexers.md)**.

Também: **[Worktrees](worktrees.md)**: criar, abrir, mesclar e remover worktrees vinculados.

Também: **[Vários remotos para uma branch](multiple-remotes.md)**: destinos de push e fontes de
pull, o diálogo de remotos, Push para todos ou para remotos selecionados, resolver um remoto que
mudou, Continuar depois de um conflito, a CLI e um alias com git puro.

**[Fluxos de trabalho de branches](branching-workflows.md)**, os modelos
Git Flow, GitHub Flow, GitLab Flow e Trunk-based ou as suas próprias bases,
padrões de nome de branch e tipos de tópico (`gitpoulpi flow ...`).

**[Reescrita do histórico](history-rewrite.md)**: o painel de reescrita
(`Ctrl+G Shift+H`) que reescreve autores, datas, mensagens e conteúdo de arquivos em
muitos commits, com prévia, scripts Lua e um único desfazer.

## Teclado e mouse

17. **[Atalhos de teclado](keybindings.md)**: cada tecla, por área, e cada
    comando da paleta. No aplicativo, `?` mostra a mesma lista e `Ctrl+P`
    encontra qualquer comando.
18. **[Mouse e gestos](mouse-and-gestures.md)**: clicar, arrastar, selecionar e
    copiar, barras de rolagem e resposta ao passar o ponteiro.

## Configurações e integrações

19. **[Configurações](settings.md)**: cada página e opção da tela de
    Configurações, a caixa de busca, os perfis e como remapear atalhos.
20. **[Assistência de IA](ai.md)**: a IA própria do GitPoulpi (desativada por
    padrão): um agente local como `claude -p`, Anthropic, OpenAI, OpenRouter,
    Ollama ou LM Studio; instruções; mensagens de commit, explicações,
    descrições de pull requests e o compositor de commits; privacidade e solução
    de problemas.
21. **[Integrações de hospedagem e de rastreadores de issues](hosting-integration.md)**: GitHub, GitLab, Gitea e Forgejo,
    Azure DevOps, Bitbucket, Gerrit e os rastreadores de issues:
    tokens e suas permissões, conexão,
    vincular um repositório, pull requests e issues, referências nas mensagens
    de commit, chaves SSH, limites conhecidos. Também
    **[Issues locais e Obsidian](local-issues.md)**: uma pasta de notas
    Markdown como fonte de issues, sem token, com uma Base do Obsidian.

## Automação

22. **[Referência da CLI](cli-reference.md)**: cada comando do `gitpoulpi`, com
    `--json` para scripts.
23. **[MCP e agentes](mcp-and-agents.md)**: deixe um agente externo (Claude
    Code, Cursor, Codex, ...) usar o GitPoulpi: registrar o servidor,
    `.mcp.json`, as chaves de permissão e seus valores padrão, cada ferramenta e
    como um agente mostra coisas a você na sua janela. A skill de agente pronta
    é [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Referência e ajuda

24. **[Referência de configuração](config-reference.md)**: cada chave do
    `config.toml`, seu valor padrão e seu efeito, as configurações por
    repositório, as variáveis de ambiente.
25. **[Solução de problemas e perguntas frequentes](troubleshooting.md)**: ícones, desenho do
    grafo, teclas que não chegam, daemons travados, velocidade, ponteiros do LFS.

## Capturas de tela

Cada imagem nestas páginas é uma captura real de uma build de release, sobre um
pequeno repositório de demonstração com autores inventados. As imagens estão em
[`images/`](../images/). `scripts/docs-screenshots.sh` cria os repositórios de
demonstração e as captura de novo. Elas são feitas no terminal `kitty`, que
desenha o grafo de commits e os avatares dos autores como imagens. Terminais sem
protocolo gráfico mostram, no lugar, um grafo de caracteres.

## Para quem contribui

[CLAUDE.md](../../../CLAUDE.md), a [visão geral da arquitetura](../../architecture/overview.md)
e o [roadmap](../../roadmap.md) descrevem o design e o que está planejado;
[CHANGELOG.md](../../../CHANGELOG.md) lista o que mudou em cada release.

---

GitKraken é uma marca de seu respectivo proprietário. O GitPoulpi é um projeto independente, sem afiliação, endosso ou patrocínio por parte dele; o nome é citado apenas para descrever compatibilidade e inspiração.
