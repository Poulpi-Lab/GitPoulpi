[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · [Français](../fr/custom-builds.md) · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Compilações personalizadas

Para quem quer um binário menor: um script interativo pergunta quais idiomas,
integrações de hospedagem, temas, cores do logo e funções incluir, e depois
compila só isso com o perfil otimizado. A compilação normal e o release oficial
não mudam (tudo ativado).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd ou PowerShell (scripts/build-custom.ps1)
```

A saída vai para `target/custom/`, nunca por cima do oficial `target/dist/`. O
script mostra o tamanho de cada binário e a diferença em relação a
`target/dist/` quando essa compilação existe. Ele também grava
`BUILD-FEATURES.txt` ao lado dos binários.

## O que você escolhe

| Escolha | Se você desativar |
|---|---|
| Idiomas | Só inglês (sempre compilado) mais os que você marcar. O seletor das Configurações lista apenas os idiomas compilados. |
| Temas | GitPoulpi Dark, GitPoulpi Light e Terminal são sempre compilados (`system` precisa de Dark e Light). Todos os outros temas são escolhidos por família (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` as imprime). Os temas não selecionados não são compilados: faltam na lista das Configurações e em `gitpoulpi theme list`, e uma configuração que cite um deles cai para o padrão como um tema desconhecido, com o aviso de sempre. |
| Cores do logo | O polvo em pixels da janela Sobre vem em 20 combinações de cores (`--list-logo-colors`). Escolha as que serão compiladas (pelo menos uma), a padrão (o Sobre mostra essa primeiro; é também a que um `ui.logo_color` guardado que não esteja compilado usa como alternativa; deve ser uma das compiladas; padrão oficial: lavender) e a cor do ícone do executável do Windows (qualquer combinação, padrão: a padrão do logo). Com uma única combinação o logo ainda gira ao clicar mas continua daquela cor, e a lista de cores (`c`, botão direito) não é oferecida. |
| Integrações (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Sem nenhuma: só clone por URL; sem issues, sem pull/merge requests. |
| IA | Sem botões nem comandos de IA, nem ferramentas `ai` na CLI/MCP. |
| MCP | Sem o binário `gitpoulpi-mcp` e sem o comando `gitpoulpi mcp`. Os comandos de agente `ui` continuam. |
| LFS, Worktree, Submódulos | As telas de gerenciamento e os comandos correspondentes somem; é devolvida uma resposta tipada “not built into this binary”. |
| Download de avatares | Só emblemas com iniciais, sem buscar nada na rede. |
| Pré-visualização de imagens | Os arquivos de imagem mostram o aviso binário simples; **Editar em** continua funcionando. |
| Radar | Sem aba Radar, sem `Alt+R` nem comando da paleta; o Radar nunca abre. Exige uma integração (sem nenhuma, ele fica inerte de qualquer jeito). |
| Mermaid | Sem Copiar como gitGraph do Mermaid (menu do Log, `Shift+Y`, paleta) e sem `log --format mermaid` na CLI/MCP. Também não há o botão **Abrir diagrama** sob um bloco `mermaid` de uma [conversa](hosting-integration.md#5c-conversas) (o bloco continua como código) nem a linha *Conversa: abrir o diagrama da entrada* da paleta. |
| Board | Sem a visão de quadro Kanban (botão do cabeçalho ISSUES, paleta) e sem o comando `hosting board`. Exige uma integração. |
| Conversa | Sem a visão de [conversa](hosting-integration.md#5c-conversas) de uma issue ou solicitação no aplicativo: Enter em uma linha de issue, em um cartão do quadro ou em uma linha do Radar abre o navegador como antes, não há a linha *Abrir conversa* nem comandos *Conversa: ...* na paleta, e não há **Escrever com IA** em uma conversa (as caixas de texto da revisão, do formulário de issue e do formulário de pull request mantêm a delas). Os comandos de conversa da CLI e do MCP (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) e `ai hosting-reply` continuam. Exige uma integração. |
| Review | Sem a visão de revisão de código no aplicativo (menu da linha, Enter em uma linha de pull request, paleta, `Revisar` do Radar) e sem a assistência de revisão por IA. Os comandos `hosting review` da CLI/MCP continuam (o Gerrit e os agentes os usam). Exige uma integração. |
| Painel do histórico (reescrita avançada do histórico: autores, datas, mensagens, conteúdo) | Sem o painel de reescrita do histórico: sem comandos `Reescrita:` na paleta, sem atalho de acesso, teclas nem seção de ajuda do painel, sem linhas "Reescrever histórico" nos menus do Log, sem o botão "Modificar o histórico mais a fundo..." no diálogo Alterar, sem "Criar regras com IA" nem IA de scripts, sem limites de script na página Sincronização das configurações e sem blocos de IA de reescrita do histórico nas configurações de IA. Os comandos `rewrite` da CLI/MCP, `ai rewrite-script`, `ai rewrite-rules` e `ui propose rewrite` saem da árvore de comandos. As identidades do `.mailmap` (`mailmap`, `log.use_mailmap`) não fazem parte dele e continuam. |
| Busca global | Sem o botão de Busca global na barra de título, sem `Ctrl+Alt+F`, sem o comando de paleta `Buscar: espaço de trabalho...` e sem o comando `gitpoulpi search` na CLI/MCP. A barra de busca do Log (`ui show search`) é outro recurso e continua. |
| Espaços de trabalho | A janela usa a única lista de abas padrão ("Sem espaço de trabalho"): sem o chip de espaço de trabalho, sem a aba Espaços de trabalho, sem `Alt+W` / `Alt+J`, sem comandos de paleta `Espaço de trabalho:`, sem linhas "Adicionar ao espaço de trabalho" nas abas, sem os comandos `gitpoulpi workspace` na CLI/MCP e sem a opção `--workspace` em `commit` e `search`. Abas e sessões funcionam como de costume. Um `workspaces.toml` escrito por outra build não é lido nem alterado; o perfil padrão e as instruções de IA de um espaço de trabalho não se aplicam. O Radar e a Busca global cobrem as abas abertas. |
| Scripts de reescrita (Lua embutido) | As regras Script de uma reescrita do histórico são recusadas com uma resposta tipada "não incluído neste binário", e nenhum Lua é vinculado. O painel do histórico em si continua (ele tem sua própria escolha). Sem sentido, e com aviso, quando o painel do histórico está desativado. |

Seus arquivos de configuração são compartilhados entre as compilações: uma
configuração de algo que não foi compilado é mantida e ignorada.

## Interativo

O script Bash usa listas de marcação `whiptail` ou `dialog` (três telas, depois
um resumo), ou perguntas numeradas quando nenhum dos dois existe. O script
PowerShell usa `Out-GridView` quando disponível, senão perguntas. Suas escolhas
anteriores vêm pré-marcadas.

## Não interativo

Qualquer argumento pula as perguntas e vai direto para
`cargo run -p xtask -- build-custom`:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # escolhas anteriores, só imprime o comando cargo
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Opção | Significado |
|---|---|
| `--langs fr,de` / `all` / `en` | Idiomas além do inglês. Os códigos são os nomes dos arquivos de idioma (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Provedores: GitHub, GitLab (inclusive auto-hospedado), Azure DevOps (Services e Server), Bitbucket (Cloud e Data Center) (hospedagem), Gitea e Forgejo incluindo o Codeberg (`gitea`) (hospedagem), Jira, Trello, Linear, YouTrack (Cloud e Server), Redmine (issue trackers), Gerrit Code Review (`gerrit`, só revisão), issues Markdown locais (`local-issues`, uma pasta ou um cofre do Obsidian). Cada um de `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack` é uma feature própria do cargo; os tamanhos estão em [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Famílias de temas a compilar (padrão `all`). `--list-themes` imprime as famílias. |
| `--logo-colors all` / `violet,teal` | Combinações de cores do logo a compilar (padrão `all`, mantidas na ordem da tabela, que é a ordem do giro). `--list-logo-colors` as imprime (`*` = padrão oficial). |
| `--logo-default teal` | A combinação padrão; deve estar entre as compiladas (padrão: lavender, senão a primeira compilada). |
| `--icon-color sun` | Cor do ícone do executável do Windows (padrão: a padrão do logo). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Cada uma é ativada por padrão. |
| `--[no-]history-rewrite` | O painel do histórico (reescrita avançada do histórico: autores, datas, mensagens, conteúdo). Ativado por padrão. |
| `--[no-]global-search` | Busca global (botão da barra de título, `Ctrl+Alt+F`, `gitpoulpi search`). Ativada por padrão. |
| `--[no-]workspaces` | Espaços de trabalho (grupos nomeados de repositórios). Ativados por padrão; desativados, resta só a lista de abas padrão. |
| `--[no-]rewrite-scripts` | Lua embutido para as regras Script de uma reescrita. Ativado por padrão; ignorado com um aviso quando o painel do histórico está desativado. |
| `--save <name\|file>` | Salva as escolhas resolvidas como seleção salva (um nome ou um caminho de arquivo). |
| `--preset <name\|file>` | Parte de uma seleção salva (um nome ou um caminho de arquivo); as outras opções a sobrescrevem, onde quer que estejam na linha. |
| `--list-presets`, `--delete-preset <name>` | Mostra as seleções salvas e a pasta delas / apaga uma pelo nome. |
| `--last` | Reutiliza `target/custom-build/last-choices.env`; as outras opções o substituem. |
| `--full` | Nenhuma seleção (tudo ativado). |
| `--profile custom\|dist\|release` | Padrão `custom` (as configurações do `dist`, pasta de saída separada). |
| `--dry-run` | Imprime o comando cargo exato e para. |
| `-- <cargo args>` | Repassados ao cargo, por exemplo `--target x86_64-unknown-linux-musl`. |

Nomes no PowerShell: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

A seleção de temas chega à compilação pela variável de ambiente
`GITPOULPI_THEMES` (`all`, `none` ou uma lista de famílias separadas por vírgula),
lida pelo build script do `gitpoulpi-config`, que descarta as outras paletas em
tempo de compilação; defina-a você mesmo para um `cargo build` simples, se
preferir. `gitpoulpi
--version` imprime uma linha `themes:` com o conjunto que foi compilado.

As escolhas do logo são as variáveis de ambiente `GITPOULPI_LOGO_COLORS`
(`all` ou uma lista de slugs separados por vírgula), `GITPOULPI_LOGO_DEFAULT` e
`GITPOULPI_ICON_COLOR`, lidas pelos build scripts do `gitpoulpi-tui` (que filtra
`assets/logo/colourways.json` na tabela que a janela Sobre incorpora) e dos três
crates de binário (o ícone). A fonte das cores continua sendo
`assets/logo/colourways.json`.

### Ícone do executável do Windows

Quando o alvo é o Windows, os executáveis `gitpoulpi`, `gitpoulpi-tui` e
`gitpoulpi-mcp` recebem o polvo como ícone, gerado em tempo de compilação (sem
precisar de Python nem de ImageMagick) na cor de ícone escolhida, em tamanhos de
16 a 128 px. Ele é incorporado com o crate `winresource` e precisa do compilador
de recursos da plataforma (`rc.exe` do Windows SDK no MSVC, `windres` no GNU). Se
estiver ausente, a compilação só imprime um aviso e o executável mantém o ícone
padrão. Na toolchain GNU só o `gitpoulpi` leva o ícone (senão o recurso seria
linkado duas vezes). As compilações para Linux e macOS ignoram tudo isso, exceto
a tabela de cores.

## Seleções salvas

Para recompilar uma nova versão sem refazer toda a seleção, salve-a com um nome e parta dela depois. As opções informadas junto com `--preset` continuam sobrescrevendo os valores salvos.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # guarda estas escolhas como "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # parte delas mais tarde
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # um caminho de arquivo também funciona
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

As seleções salvas são arquivos `CHAVE=valor` simples (o formato de `last-choices.env`, legível por um shell). Ficam na pasta `custom-builds` do diretório de configuração do GitPoulpi, fora de `target/`, então `cargo clean` não as apaga:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (`~/.config/gitpoulpi/custom-builds/` por padrão)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` move essa pasta junto com a configuração. `--preset` e `--save` também aceitam um caminho de arquivo (um valor que contém `/` ou `\`, ou que termina em `.env`), para guardar uma seleção em um repositório de dotfiles.

Uma seleção escrita por outra versão do GitPoulpi ainda carrega. Uma chave desconhecida, ou um idioma, integração, tema ou cor que não existe mais, é ignorada com um aviso. Uma chave que falta no arquivo (um recurso adicionado depois) assume o valor padrão, e uma integração adicionada depois é incluída; ambos são informados na saída de erro.

Quando existe uma seleção salva, o modo interativo oferece primeiro "Partir de" (as escolhas anteriores, uma seleção salva ou tudo ativado) e, no final, pergunta se deve salvar a seleção atual com um nome.

## O que ela economiza

Medido no Linux em 2026-10-29 no commit `65999ef5c`, a partir da compilação completa, com a diferença em relação a ela entre parênteses
(detalhes em `docs/developer/binary-size.md`):

| Compilação (a partir de `full`, salvo indicação) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Completa (tudo ativado) | 64.6 | 50.5 | 31.7 |
| Só inglês | 52.0 (-12.6) | 40.2 (-10.3) | 31.7 (0.0) |
| MCP desativado | 61.6 (-3.0) | 49.6 (-0.8) | não compilado |
| IA desativada | 62.6 (-2.0) | 48.6 (-1.8) | 30.5 (-1.3) |
| Todas as integrações desativadas | 56.3 (-8.2) | 43.7 (-6.7) | 26.7 (-5.0) |
| IA + integrações + download de avatares desativados | 52.2 (-12.3) | 39.9 (-10.6) | 23.0 (-8.8) |
| Painel do histórico desativado (`--no-history-rewrite`) | 61.6 (-3.0) | 48.9 (-1.5) | 28.5 (-3.2) |
| Scripts de reescrita desativados | 63.1 (-1.5) | 49.0 (-1.5) | 31.1 (-0.7) |
| Busca global desativada | 63.5 (-1.1) | 49.5 (-1.0) | 31.6 (-0.1) |
| Espaços de trabalho desativados | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.2) |
| Radar desativado | 63.4 (-1.2) | 49.3 (-1.2) | 31.7 (0.0) |
| Board desativado | 63.6 (-1.0) | 49.5 (-1.0) | 31.7 (0.0) |
| Review desativado | 63.2 (-1.4) | 49.0 (-1.4) | 31.7 (0.0) |
| Mermaid desativado | 63.6 (-1.0) | 49.5 (-0.9) | 31.6 (-0.1) |
| LFS desativado | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.3) |
| Worktree desativado | 63.7 (-0.9) | 49.6 (-0.8) | 31.7 (0.0) |
| Submódulos desativados | 63.7 (-0.9) | 49.6 (-0.9) | 31.6 (-0.1) |
| Download de avatares desativado | 63.7 (-0.9) | 49.6 (-0.9) | 31.7 (0.0) |
| Pré-visualização de imagens desativada | 63.3 (-1.2) | 49.2 (-1.3) | 31.7 (0.0) |
| Tudo desativado, só inglês (LFS mantido) | 35.2 (-29.4) | 28.9 (-21.5) | não compilado |
| **Tudo desativado, só inglês** | **34.9 (-29.7)** | **28.8 (-21.6)** | não compilado |

## Notas

- Uma compilação personalizada é lenta como qualquer compilação `dist` (LTO
  total, alguns minutos).
- Ela roda sem sccache, como o `build-opti`.
- Os códigos de idioma são conferidos com os idiomas distribuídos; um código
  desconhecido é recusado com a lista dos válidos.
