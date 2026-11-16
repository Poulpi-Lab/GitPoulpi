[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · **Português (Portugal)** · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Compilações personalizadas

Para quem quer um binário mais pequeno: um script interativo pergunta que
idiomas, integrações de alojamento, temas, cores do logótipo e funcionalidades
incluir, e depois compila apenas isso com o perfil otimizado. A compilação normal
e o lançamento oficial não mudam (tudo ativado).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd ou PowerShell (scripts/build-custom.ps1)
```

O resultado fica em `target/custom/`, nunca por cima do `target/dist/` oficial. O
script mostra o tamanho de cada binário e a diferença em relação a
`target/dist/` quando essa compilação existe. Escreve também um
`BUILD-FEATURES.txt` ao lado dos binários.

## O que escolhe

| Escolha | Se a desativar |
|---|---|
| Idiomas | Apenas inglês (sempre compilado) mais os que assinalar. O seletor das Definições só lista os idiomas compilados. |
| Temas | GitPoulpi Dark, GitPoulpi Light e Terminal são sempre compilados (`system` precisa de Dark e Light). Cada um dos outros temas é escolhido por família (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` imprime-as). Os temas não selecionados não são compilados: faltam na lista das Definições e em `gitpoulpi theme list`, e uma configuração que nomeie um recorre à alternativa, como um tema desconhecido, com o aviso habitual. |
| Cores do logótipo | O polvo de píxeis da janela Acerca do GitPoulpi vem em 20 combinações de cores (`--list-logo-colors`). Escolha as que compilar (pelo menos uma), a predefinida (o Acerca mostra-a primeiro; é também aquela para que recorre um `ui.logo_color` guardado que não esteja compilado; tem de ser uma das compiladas; predefinição oficial: lavender) e a cor do ícone do executável do Windows (qualquer combinação, predefinição: a predefinida do logótipo). Com uma única combinação o logótipo continua a rodar ao clique mas mantém essa cor, e a lista de cores (`c`, clique com o botão direito) não é oferecida. |
| Integrações (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Sem nenhuma: só clonar por URL; sem issues, sem pull/merge requests. |
| IA | Sem botões, comandos nem ferramentas `ai` de CLI/MCP. |
| MCP | Sem binário `gitpoulpi-mcp` e sem comando `gitpoulpi mcp`. Os comandos de agente `ui` mantêm-se. |
| LFS, Worktree, Submódulos | Os ecrãs de gestão e os comandos correspondentes desaparecem; é devolvida uma resposta tipada "not built into this binary". |
| Transferência de avatares | Só emblemas com iniciais, sem obtenção pela rede. |
| Pré-visualização de imagens | Os ficheiros de imagem mostram o aviso binário habitual; **Editar em** continua a funcionar. |
| Radar | Sem separador Radar, sem `Alt+R` nem comando da paleta; o Radar nunca abre. Precisa de uma integração (sem nenhuma, fica inerte de qualquer forma). |
| Mermaid | Sem Copiar como Mermaid gitGraph (menu do Log, `Shift+Y`, paleta) e sem `log --format mermaid` na CLI/MCP. Também sem o botão **Abrir diagrama** sob um bloco `mermaid` de uma [conversa](hosting-integration.md#5c-conversas) (o bloco fica como código) e sem a linha da paleta *Conversa: abrir o diagrama da entrada*. |
| Board | Sem vista de quadro Kanban (botão do cabeçalho ISSUES, paleta) e sem o comando `hosting board`. Precisa de uma integração. |
| Conversa | Sem vista de [conversa](hosting-integration.md#5c-conversas) de uma issue ou pedido na aplicação: Enter numa linha de issue, num cartão do quadro ou numa linha do Radar abre o navegador como antes, não há linha *Abrir conversa* nem comando da paleta *Conversa: ...*, e não há **Escrever com IA** numa conversa (as caixas de texto da revisão, do formulário de issue e do formulário de pull request mantêm a sua). Os comandos de conversa da CLI e do MCP (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) e `ai hosting-reply` mantêm-se. Precisa de uma integração. |
| Review | A página Resumo da revisão é a vista de conversa, pelo que uma compilação com Review contém também a vista de conversa, diga o que disser *Conversa*. Sem vista de revisão de código na aplicação (menu da linha, Enter numa linha de pull request, paleta, `Rever` do Radar) e sem assistência de revisão por IA. Os comandos `hosting review` da CLI/MCP mantêm-se (o Gerrit e os agentes usam-nos). Precisa de uma integração. |
| Painel do histórico (reescrita avançada do histórico: autores, datas, mensagens, conteúdo) | Sem painel de reescrita do histórico: sem comandos `Reescrever:` na paleta, sem acorde, teclas nem secção de ajuda do painel, sem linhas «Reescrever histórico» nos menus do Log, sem botão «Modificar o histórico mais a fundo...» no diálogo Emendar, sem «Criar regras com IA» nem IA para scripts, sem limites de scripts na página de definições Sincronização e sem blocos de IA de reescrita do histórico nas definições de IA. Os comandos `rewrite` da CLI/MCP, `ai rewrite-script`, `ai rewrite-rules` e `ui propose rewrite` deixam de existir na árvore de comandos. As identidades do `.mailmap` (`mailmap`, `log.use_mailmap`) não fazem parte dele e mantêm-se. |
| Pesquisa global | Sem botão de pesquisa global na barra de título, sem `Ctrl+Alt+F`, sem o comando `Pesquisar na área de trabalho` na paleta e sem o comando `gitpoulpi search` da CLI/MCP. A barra de pesquisa do Log (`ui show search`) é outra funcionalidade e mantém-se. |
| Áreas de trabalho | A janela trabalha com a única lista de separadores predefinida («Sem área de trabalho»): sem chip de área de trabalho, sem separador Áreas de trabalho, sem `Alt+W` / `Alt+J`, sem comandos `Área de trabalho:` na paleta, sem linhas «Adicionar à área de trabalho» nos separadores, sem comandos `gitpoulpi workspace` da CLI/MCP e sem a opção `--workspace` em `commit` e `search`. Separadores e sessões funcionam como de costume. Um `workspaces.toml` escrito por outro build não é lido nem alterado; o perfil predefinido e as instruções de IA de uma área de trabalho não se aplicam. O Radar e a pesquisa global cobrem os separadores abertos. |
| Scripts de reescrita (Lua incorporado) | As regras Script de uma reescrita do histórico são recusadas com uma resposta tipada «não incluído neste binário», e nenhum Lua é ligado. O painel do histórico em si mantém-se (tem a sua própria escolha). Sem sentido, e com aviso, quando o painel do histórico está desativado. |

Os seus ficheiros de configuração são partilhados entre compilações: uma
definição de algo que não está compilado é mantida e ignorada.

## Interativo

O script Bash usa listas de verificação do `whiptail` ou do `dialog` (três
ecrãs, depois um resumo), ou perguntas numeradas quando nenhum existe. O script
PowerShell usa o `Out-GridView` quando disponível, senão perguntas. As suas
escolhas anteriores vêm pré-assinaladas.

## Não interativo

Qualquer argumento salta as perguntas e vai direto a
`cargo run -p xtask -- build-custom`:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # escolhas anteriores, só imprime o comando cargo
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Opção | Significado |
|---|---|
| `--langs fr,de` / `all` / `en` | Idiomas além do inglês. Os códigos são os nomes dos ficheiros de locale (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Fornecedores: GitHub, GitLab (incl. self-managed), Azure DevOps (Services e Server), Bitbucket (Cloud e Data Center) (alojamento), Gitea e Forgejo incl. Codeberg (`gitea`) (alojamento), Jira, Trello, Linear, YouTrack (Cloud e Server), Redmine (gestores de issues), Gerrit Code Review (`gerrit`, só revisão), issues Markdown locais (`local-issues`, uma pasta ou um cofre do Obsidian). Cada um de `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack` é a sua própria feature do cargo; tamanhos em [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Famílias de temas a compilar (predefinição `all`). `--list-themes` imprime as famílias. |
| `--logo-colors all` / `violet,teal` | Combinações de cores do logótipo a compilar (predefinição `all`, mantidas pela ordem da tabela, que é a ordem de rotação). `--list-logo-colors` imprime-as (`*` = predefinição oficial). |
| `--logo-default teal` | A combinação predefinida; tem de estar entre as compiladas (predefinição: lavender, senão a primeira compilada). |
| `--icon-color sun` | Cor do ícone do executável do Windows (predefinição: a predefinida do logótipo). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Cada uma está ativada por predefinição. |
| `--[no-]history-rewrite` | O painel do histórico (reescrita avançada do histórico: autores, datas, mensagens, conteúdo). Ativado por predefinição. |
| `--[no-]global-search` | Pesquisa global (botão da barra de título, `Ctrl+Alt+F`, `gitpoulpi search`). Ativada por predefinição. |
| `--[no-]workspaces` | Áreas de trabalho (grupos nomeados de repositórios). Ativadas por predefinição; desativadas, fica só a lista de separadores predefinida. |
| `--[no-]rewrite-scripts` | Lua incorporado para as regras Script de uma reescrita. Ativado por predefinição; ignorado com um aviso quando o painel do histórico está desativado. |
| `--save <name\|file>` | Guarda as escolhas resolvidas como seleção guardada (um nome ou um caminho de ficheiro). |
| `--preset <name\|file>` | Parte de uma seleção guardada (um nome ou um caminho de ficheiro); as outras opções sobrepõem-se-lhe, onde quer que estejam na linha. |
| `--list-presets`, `--delete-preset <name>` | Mostra as seleções guardadas e a sua pasta / apaga uma pelo nome. |
| `--last` | Reutiliza `target/custom-build/last-choices.env`; as outras opções sobrepõem-se-lhe. |
| `--full` | Nenhuma seleção (tudo ativado). |
| `--profile custom\|dist\|release` | Predefinição `custom` (as definições de `dist`, diretório de saída separado). |
| `--dry-run` | Imprime o comando cargo exato e pára. |
| `-- <cargo args>` | Passados ao cargo, por exemplo `--target x86_64-unknown-linux-musl`. |

Nomes no PowerShell: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

A seleção de temas chega à compilação como a variável de ambiente
`GITPOULPI_THEMES` (`all`, `none` ou uma lista de famílias separadas por
vírgulas), lida pelo build script do `gitpoulpi-config`, que remove as outras
paletas em tempo de compilação; defina-a manualmente para um `cargo build` simples
se preferir. `gitpoulpi --version` imprime uma linha `themes:` com o conjunto que
foi compilado.

As escolhas do logótipo são as variáveis de ambiente `GITPOULPI_LOGO_COLORS`
(`all` ou uma lista de slugs separados por vírgulas), `GITPOULPI_LOGO_DEFAULT` e
`GITPOULPI_ICON_COLOR`, lidas pelos build scripts do `gitpoulpi-tui` (que filtra
`assets/logo/colorways.json` para a tabela que a janela Acerca do GitPoulpi
incorpora) e dos três crates de binários (o ícone). A fonte das cores continua a
ser `assets/logo/colorways.json`.

### Ícone do executável do Windows

Quando o alvo é o Windows, os executáveis `gitpoulpi`, `gitpoulpi-tui` e
`gitpoulpi-mcp` recebem o polvo como ícone, gerado em tempo de compilação (sem
precisar de Python nem de ImageMagick) na cor de ícone escolhida, em tamanhos de
16 a 128 px. É incorporado com o crate `winresource` e precisa do compilador de
recursos da plataforma (`rc.exe` do Windows SDK no MSVC, `windres` no GNU). Se
faltar, a compilação apenas imprime um aviso e o executável fica com o ícone
predefinido. Na toolchain GNU só o `gitpoulpi` leva o ícone (o recurso seria de
outra forma ligado duas vezes). As compilações para Linux e macOS ignoram tudo
isto, exceto a tabela de cores.

## Seleções guardadas

Para recompilar uma nova versão sem refazer toda a seleção, guarde-a com um nome e parta dela mais tarde. As opções indicadas em conjunto com `--preset` continuam a sobrepor-se aos valores guardados.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # guarda estas escolhas como "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # partir delas mais tarde
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # um caminho de ficheiro também serve
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

As seleções guardadas são ficheiros `CHAVE=valor` simples (o formato de `last-choices.env`, legível por uma shell). Ficam na pasta `custom-builds` do diretório de configuração do GitPoulpi, fora de `target/`, pelo que o `cargo clean` não as apaga:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (`~/.config/gitpoulpi/custom-builds/` por predefinição)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` move essa pasta juntamente com a configuração. `--preset` e `--save` também aceitam um caminho de ficheiro (um valor que contém `/` ou `\`, ou que termina em `.env`), para guardar uma seleção num repositório de dotfiles.

Uma seleção escrita por outra versão do GitPoulpi continua a carregar. Uma chave desconhecida, ou um idioma, integração, tema ou cor que já não existe, é ignorada com um aviso. Uma chave em falta no ficheiro (uma funcionalidade acrescentada depois) assume o valor predefinido, e uma integração acrescentada depois é incluída; ambos são indicados no erro padrão.

Quando existe uma seleção guardada, o modo interativo oferece primeiro «Partir de» (as escolhas anteriores, uma seleção guardada ou tudo ativado) e, no fim, pergunta se deve guardar a seleção atual com um nome.

## O que poupa

Medido em Linux a 2026-10-29 no commit `65999ef5c`, a partir da compilação completa, com a diferença em relação a ela entre parênteses
(detalhes em `docs/developer/binary-size.md`):

| Compilação (a partir de `full`, salvo indicação) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Completa (tudo ativado) | 64.6 | 50.5 | 31.7 |
| Só inglês | 52.0 (-12.6) | 40.2 (-10.3) | 31.7 (0.0) |
| MCP desativado | 61.6 (-3.0) | 49.6 (-0.8) | não compilado |
| IA desativada | 62.6 (-2.0) | 48.6 (-1.8) | 30.5 (-1.3) |
| Todas as integrações desativadas | 56.3 (-8.2) | 43.7 (-6.7) | 26.7 (-5.0) |
| IA + integrações + transferência de avatares desativadas | 52.2 (-12.3) | 39.9 (-10.6) | 23.0 (-8.8) |
| Painel do histórico desativado (`--no-history-rewrite`) | 61.6 (-3.0) | 48.9 (-1.5) | 28.5 (-3.2) |
| Scripts de reescrita desativados | 63.1 (-1.5) | 49.0 (-1.5) | 31.1 (-0.7) |
| Pesquisa global desativada | 63.5 (-1.1) | 49.5 (-1.0) | 31.6 (-0.1) |
| Áreas de trabalho desativadas | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.2) |
| Radar desativado | 63.4 (-1.2) | 49.3 (-1.2) | 31.7 (0.0) |
| Board desativado | 63.6 (-1.0) | 49.5 (-1.0) | 31.7 (0.0) |
| Review desativado | 63.2 (-1.4) | 49.0 (-1.4) | 31.7 (0.0) |
| Mermaid desativado | 63.6 (-1.0) | 49.5 (-0.9) | 31.6 (-0.1) |
| LFS desativado | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.3) |
| Worktree desativado | 63.7 (-0.9) | 49.6 (-0.8) | 31.7 (0.0) |
| Submódulos desativados | 63.7 (-0.9) | 49.6 (-0.9) | 31.6 (-0.1) |
| Transferência de avatares desativada | 63.7 (-0.9) | 49.6 (-0.9) | 31.7 (0.0) |
| Vista de imagens desativada | 63.3 (-1.2) | 49.2 (-1.3) | 31.7 (0.0) |
| Tudo desativado, só inglês (LFS mantido) | 35.2 (-29.4) | 28.9 (-21.5) | não compilado |
| **Tudo desativado, só inglês** | **34.9 (-29.7)** | **28.8 (-21.6)** | não compilado |

## Notas

- Uma compilação personalizada é lenta como qualquer compilação `dist` (LTO
  completo, alguns minutos).
- Corre sem sccache, como o `build-opti`.
- Os códigos de idioma são verificados contra os locales distribuídos; um código
  desconhecido é recusado com a lista dos válidos.
