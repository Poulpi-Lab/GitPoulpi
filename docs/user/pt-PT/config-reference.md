[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · **Português (Portugal)** · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Referência de configuração

Todas as chaves de `config.toml`, a sua predefinição e o que fazem. Raramente
precisa de editar o ficheiro à mão: o [ecrã de Definições](settings.md)
(`Ctrl+,`) escreve-o à medida que altera as coisas, e `gitpoulpi config set`
também.

## Onde está, e como se escolhe um valor

| | |
|---|---|
| Ficheiro | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` imprime-o. |
| Outra pasta | `GITPOULPI_CONFIG_DIR=<dir>` move toda a pasta de configuração (o ficheiro, `sessions/`, os ficheiros de porta do daemon): uma instalação portátil, um teste, ou um agente mantido longe das suas próprias definições. |
| Precedência | predefinição integrada, depois `config.toml`, depois a variável `GITPOULPI_*`, depois uma opção da linha de comandos. Ganha a mais alta. Um valor errado (uma escolha mal escrita no ficheiro, uma variável incorreta) é um erro, nunca é ignorado em silêncio. |
| Variáveis | Ponha a chave em maiúsculas e transforme os pontos em sublinhados: `general.auto_prune` é `GITPOULPI_GENERAL_AUTO_PRUNE`. Só para chaves que o GitPoulpi realmente lê (não as marcadas *não usada* abaixo, nem listas como `ui.log_columns_hidden`). |
| Opções | `--backend git2\|system` sobrepõe-se a `general.default_backend` num daemon que esse comando inicia. |
| Tokens e chaves | Nunca neste ficheiro. Os tokens de alojamento e as chaves de API de IA vão para o credential helper do seu sistema (Git Credential Manager, libsecret / `pass`, Keychain); o ficheiro só guarda uma referência (`token_ref`). Sem um helper, guardar um token falha com uma mensagem e o token dura apenas a sessão. |

```
gitpoulpi config path                          # o ficheiro, e se já existe
gitpoulpi config get general.auto_stash        # o valor, a camada de onde vem (default/file/env/flag) e se alguma coisa o lê
gitpoulpi --json config get ui                 # uma secção inteira
gitpoulpi config set general.auto_stash always # verificado contra o esquema e depois escrito
```

`config set` mantém o ficheiro esparso (só as chaves que alguém definiu) e mantém
as chaves sobre as quais não foi questionado, mas descarta os comentários. Recusa
listas e tabelas inteiras, exceto `keybindings.*` (vários acordes separados por
espaços; vazio retira a atribuição). Edite `[[profiles]]`, `[[integrations]]` e
`[[ai.profiles]]` nas Definições ou à mão.

Nas tabelas, **Reiniciar** significa que o valor é lido no arranque; tudo o resto
aplica-se de imediato (ou no próxima atualização, fetch ou fecho das
Definições).

## `[general]`

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, recorrendo ao programa `git` para stage, merge, rebase, stash e rede) ou `system` (apenas o `git` instalado). Um daemon em execução mantém o seu backend: **reinicie-o** (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Quando alterações locais bloqueiam um checkout, pull, merge ou rebase: `ask` (uma caixa de diálogo na TUI; a CLI e o MCP falham e indicam `--autostash`), `always` (stash, executar, pop) ou `never` (recusar). Veja [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | O auto-stash leva também os ficheiros não acompanhados. |
| `general.initial_commits_in_graph` | `5000` | Commits que o Log carrega primeiro (pelo menos 500). Mais custa memória e tempo de desenho em repositórios enormes. |
| `general.lazy_load_commits_in_graph` | `true` | Carregar commits mais antigos à medida que se desloca perto do fim. Desativado: o Log pára na contagem inicial. |
| `general.commits_per_lazy_page` | `1000` | Commits por página em segundo plano (pelo menos 50). |
| `general.max_commits_in_graph` | `100000` | Onde o carregamento progressivo pára (cerca de 300 bytes por commit); `0` = sem limite. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch All em segundo plano a cada N minutos, de 0 a 60; `0` = desativado. **Reiniciar.** |
| `general.auto_fetch_interval_unit` | `"auto"` | A unidade em que as Definições mostram o intervalo: `minutes`, `hours` ou `auto`. Só de apresentação: o intervalo é sempre `auto_fetch_interval_minutes` minutos. `auto` (e uma configuração sem a chave) escolhe a maior unidade que divide os minutos exatamente (60 lê-se "1 hora", 45 "45 minutos"); uma unidade que não serve é ignorada. As Definições escrevem as duas chaves; um GitPoulpi mais antigo ignora a unidade e continua a ler os minutos. |
| `general.fetch_on_tab_open` | `true` | Fetch All quando um separador abre (ignorado se o GitPoulpi fez fetch desse repositório há menos de 60 s). |
| `general.auto_prune` | `false` | Os fetches também fazem prune das branches de acompanhamento remoto que já não existem. |
| `general.auto_prune_tags` | `false` | Um fetch também elimina as tags locais que o remoto eliminou (só as que estavam nele; uma tag que nunca enviou é mantida). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Atualização das listas de pull requests / issues e do chip da barra de ferramentas, de 0 a 120; `0` = só a pedido e por eventos. |
| `general.hosting_refresh_unit` | `"auto"` | A unidade em que as Definições mostram a atualização: `minutes`, `hours` ou `auto`, lida como `general.auto_fetch_interval_unit`. Só de apresentação. |
| `general.keep_submodules_up_to_date` | `false` | Depois de um checkout ou pull, `git submodule update --init --recursive` quando um submódulo está fora de sintonia. Veja [Submódulos](submodules.md). |
| `general.default_branch_name` | `"main"` | Primeira branch de um repositório criado com Init; vazio deixa o git decidir. Não renomeia nada. |
| `general.delete_orig_files_after_merge` | `false` | Depois de um merge concluído, apagar os ficheiros de cópia `*.orig`. |
| `general.remember_tabs` | `true` | Reabrir os separadores da última sessão. Desativado, um arranque dentro de um repositório abre apenas esse repositório, e um arranque fora de um repositório abre apenas o Início. **Reiniciar.** |
| `general.radar_enabled` | `true` | O [Radar](radar.md): o seu separador, os comandos da paleta, `Alt+R` e a sua obtenção em segundo plano. Desativado, oculta tudo isto e nada é obtido para ele. Aplica-se de imediato; não aparece numa compilação sem a funcionalidade `radar`. |
| `general.startup_tab_in_repo` | `"current-repo"` | O separador em que um arranque abre quando a pasta de onde parte está dentro de um repositório: `"current-repo"` (o separador desse repositório, aberto se necessário), `"last"` (o separador ativo no último encerramento), `"home"` ou `"radar"`. Um repositório indicado com `--repo` abre sempre o próprio separador. `"last"` passa a Início com `remember_tabs` desativado ou se não houver nada lembrado; `"radar"` passa a Início se o Radar estiver desativado ou fora da compilação. Lido uma vez no arranque; exige reinício. |
| `general.startup_tab_outside_repo` | `"home"` | O separador em que um arranque abre quando a pasta de onde parte não é um repositório (uma pasta de utilizador, por exemplo): `"home"`, `"last"` ou `"radar"`. `"last"` passa a Início com `remember_tabs` desativado ou se não houver nada lembrado; `"radar"` passa a Início se o Radar estiver desativado ou fora da compilação. Lido uma vez no arranque; exige reinício. |
| `general.startup_tab_launcher` | `"last"` | O separador em que um arranque abre quando o GitPoulpi é iniciado pelo atalho do ambiente de trabalho ou do menu Iniciar que ele criou (passam `--launcher`), seja qual for a pasta: `"home"`, `"last"` ou `"radar"`. Tem prioridade sobre as duas definições acima. `"last"` passa a Início com `remember_tabs` desativado ou se não houver nada lembrado; `"radar"` passa a Início se o Radar estiver desativado ou fora da compilação. Lido uma vez no arranque; exige reinício. |
| `general.group_worktree_tabs` | `true` | Mantém os separadores dos worktrees associados a um repositório logo a seguir ao separador desse repositório: quando um worktree abre, quando um separador de worktree é arrastado para longe (volta a colar-se ao seu grupo; o separador do repositório é a cabeça do grupo e os seus worktrees seguem-no) e quando a opção é ativada. Desativado, os separadores ficam onde os pôr. |
| `general.undo_depth` | `100` | Quantas operações o Desfazer consegue recuar; `0` = ilimitado. |
| `general.conflict_detection` | `true` | Deteção proativa de conflitos: um selo no Log quando a integração da branch atual na branch de que partiu daria conflito, calculado em segundo plano (um merge em memória: nada muda nos seus ficheiros, no índice nem nas refs). Um repositório pode desativá-la para si (`[preferences] conflict_detection`). As branches base com que compara têm listas próprias: [`[conflict_prevention]`](#conflict_prevention-listas-de-branches-base). Veja [Resolver conflitos de merge](merge-conflicts.md#prevenção-de-conflitos). |
| `general.dev_mode` | ausente (automático) | Para quem compila o GitPoulpi a partir de um clone do código; nunca aparece nas Definições. Ausente: ativado quando o executável em uso está em `<checkout>/target/<profile>/` (`Cargo.toml` e `crates/gitpoulpi-tui` ao lado de `target`). `true` / `false` força-o. O modo de programador só desbloqueia `ui.launcher_build`. |
| `general.undo_enabled` | `true` | `false`: sem qualquer registo de desfazer (o mais rápido), o histórico de desfazer é apagado, os botões Desfazer/Refazer desaparecem. Veja [Desfazer e refazer](undo-redo.md). |
| `general.journal_outside` | `true` | Regista o que uma shell ou um agente de IA faz a branches, tags, stashes e HEAD como entradas do histórico de desfazer, marcadas como externas, para que `Ctrl+Z` as possa desfazer. As edições a ficheiros nunca são capturadas. Requer `undo_enabled`. Veja [Desfazer e refazer](undo-redo.md#alterações-feitas-fora-do-gitpoulpi). |
| `general.active_profile` | `""` | Id da identidade `[[profiles]]` com que os commits são feitos. Vazio, ou um perfil com nome ou e-mail em branco: `user.name` / `user.email` do git. |
| `general.external_editor` | `""` | Comando para *Abrir no editor externo* de um ficheiro, por ex. `code -r {file}`. Vazio: o VS Code quando instalado, senão o predefinido do sistema. |
| `general.external_diff_tool` | `""` | Comando para *Abrir na ferramenta de diff externa*: `{local}` (versão commitada / HEAD), `{remote}` (versão da árvore de trabalho / índice). Vazio: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Comando para *Abrir na ferramenta de merge externa* numa ficha de conflito (um ficheiro binário, LFS ou demasiado grande): `{base}`, `{local}` (A, atual), `{remote}` (B, recebido) e `{merged}` (o ficheiro onde a ferramenta escreve o resultado; o GitPoulpi faz stage dele quando a ferramenta termina com êxito). Vazio: `git mergetool --no-prompt`. |
| `general.file_explorer` | por SO | Comando para *Mostrar na pasta*: `{path}` (o ficheiro), `{dir}` (a sua pasta). Predefinições: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

Os três modelos de comando são primeiro divididos em argumentos (`"..."` agrupa
uma palavra), depois os marcadores são preenchidos, de modo que caminhos com
espaços continuam a ser um só argumento. Nenhuma shell está envolvida.

```toml
[[profiles]]                # identidades de commit (Definições > Perfis)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log e grafo**

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (uma imagem onde o terminal a consiga desenhar, caracteres nos outros), `image`, `text`. **Reiniciar.** |
| `ui.image_protocol` | `"auto"` | Como as imagens (grafo, avatares) chegam ao terminal: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = protocolo Kitty no kitty, no Ghostty e num herdr com `kitty_graphics = true`; iTerm2 no WezTerm, iTerm2, Rio, mintty; sixel (só avatares) no Windows Terminal, foot, Konsole, mlterm; nenhum no tmux, zellij, screen e terminais desconhecidos. **Reiniciar.** |
| `ui.theme` | `"system"` | O tema de cores: um slug integrado (`gitpoulpi theme list` lista-os todos), o slug de um ficheiro em `<config>/themes/`, ou `system` (acompanha o fundo claro ou escuro do terminal, detetado no arranque; o valor antigo `default` significa o mesmo). `config set` rejeita nomes desconhecidos. Veja [Temas](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Paleta de faixas: `theme` (acompanha o tema) ou qualquer nome de paleta de faixas: `gitpoulpi`, `light`, `material`, `serie`, e as paletas dos outros temas. |
| `ui.graph_lane_background` | `"off"` | `off` ou `row_tint` (uma faixa ténue da cor da faixa atrás do grafo, mais uma barra na sua margem). **Aplicação: alternar tom das linhas do grafo** (`Ctrl+N Shift+T`) alterna-o; **Aplicação: mudar tema de cores do grafo...** (`Ctrl+N Shift+Y`) escolhe `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (só com `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` e `right` desenham uma coluna; uma janela com menos de 80 colunas volta a `bottom`. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never`: se uma janela que moveu ou redimensionou (arraste a barra de título ou as margens, ou `Ctrl+N m`) mantém o seu lugar: até o GitPoulpi fechar (predefinição), em disco para o próximo arranque (`dialog_geometry.txt` na pasta de dados) ou de todo. |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: onde ficam os botões ao longo da barra (esquerda, meio, direita de uma linha; topo, meio, fundo de uma coluna). |
| `ui.toolbar_pr_pinned` | `true` | O botão de pull request fica fixo na extremidade da barra de ferramentas; `false` deixa-o seguir os outros botões. |
| `ui.statusbar_position` | `"bottom"` | `top` (acima dos separadores de repositório) ou `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (a última linha da janela, abaixo da barra de estado) ou `left` (uma coluna na margem esquerda; com menos de 80 colunas fica no topo). |
| `ui.graph_show_stashes` | `true` | Uma linha por stash no Log. |
| `ui.ref_connector` | `true` | Uma linha fina da etiqueta de uma ref até ao seu commit. |
| `ui.column_resize` | `"line"` | Como se agarra a fronteira de uma coluna no Log, no Radar e na coluna de commits das vistas de diff, de ficheiro e de blame. `line`: cada fronteira é um espaço de uma célula com uma pequena marca no cabeçalho; enquanto o ponteiro está na fronteira, no cabeçalho ou em qualquer ponto da tabela, é desenhada uma linha fina de destaque em toda a altura e um arrastamento a partir de qualquer ponto dela redimensiona. `header`: sem espaço nem marca; o primeiro carácter da célula de cabeçalho a seguir à fronteira acende-se e um arrastamento a partir daí redimensiona (só no cabeçalho). As teclas de dimensionamento e as linhas da paleta são as mesmas nos dois modos. |
| `ui.ref_tree` | `true` | Agrupar as branches e as tags em `/` em pastas na barra lateral Refs (`y` alterna). |
| `ui.graph_ghost_ref` | `true` | A coluna Ref do Log mostra uma pílula esbatida com o nome da branch (senão a branch remota) a que pertence um commit sob o rato, quando o commit não tem ref própria e a coluna tem espaço; nunca uma tag. Uma branch eliminada é deduzida do assunto do commit de merge que a integrou e desenhada riscada. Passar o rato na pílula mostra o tipo. |
| `ui.reveal_on_ref_click` | `true` | Um único clique numa ref (branch, tag, branch remota, stash) no painel Refs desloca o Log até ao commit dela e faz a linha piscar; a seleção de commits não se move. |
| `ui.log_wip_counts_in_message` | `true` | Contagens das alterações WIP na mensagem WIP enquanto a coluna Alterações está oculta. |
| `ui.log_columns_layout` | `"fit"` | Como o Log dispõe as suas colunas. `fit`: cada coluna visível cabe exatamente no painel e só `message` se adapta; quando o painel é estreito, colunas inteiras cedem. `scroll`: as colunas mantêm a largura (`message` também), nenhuma cede, e a tabela desloca-se lateralmente quando é mais larga do que o painel. |
| `ui.log_column_widths_scroll` | vazio | As larguras das colunas da disposição `scroll`, nome -> células (`message` permitido, ao contrário de `ui.log_column_widths`, que continua a ser o da disposição `fit` e nunca contém `message`). Escrito quando arrasta, dimensiona ou ajusta uma coluna com a disposição `scroll`. |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | vazio | As colunas do Log (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`), escritas quando oculta, arrasta ou redimensiona uma coluna. `message` não pode ser ocultada. |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | vazio | As partes da [coluna de commits](interface-tour.md#a-coluna-de-commits) ao lado de um diff, só dela e à parte das do Log: as partes mostradas pelo nome (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; vazio é faixas, mensagem, autor e data), a sua ordem da esquerda para a direita pelos mesmos nomes (vazio é a ordem que acabou de ler; um nome omitido segue nessa ordem) e as larguras arrastadas, em células por parte (uma parte omitida é automática), gravadas quando escolhe, move ou dimensiona as partes. `message` não pode ser ocultada. Um ficheiro de sessão escrito antes de estas chaves existirem é lido uma vez para elas. |
| `ui.sidebar_sections_hidden` | vazio | As secções do painel esquerdo que ocultou (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), escritas a partir do menu de clique com o botão direito do painel ou da paleta. Pelo menos uma secção existente fica sempre visível. Esta é a predefinição: cada repositório pode ocultar as suas próprias secções (guardadas no seu ficheiro de estado) e esta chave vale para um repositório sem escolha própria; *Usar em todos os repositórios* grava-a. |
| `ui.sidebar_sections_scope` | `"repository"` | A quem pertence a escolha das secções do painel esquerdo: `repository` (cada repositório guarda a sua, sendo `ui.sidebar_sections_hidden` a predefinição) ou `global` (todos os repositórios mostram `ui.sidebar_sections_hidden`, e uma alteração é escrita aí). Mudar de uma para a outra nunca perde nenhum dos conjuntos: as escolhas por repositório ficam nos respetivos ficheiros de estado enquanto `global` estiver ativo. Definições: *Secções do painel esquerdo*. |
| `ui.language` | `"auto"` | O idioma da interface: `auto` (o locale do sistema, senão inglês), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Em direto.** `GITPOULPI_LANG` sobrepõe-se-lhe. Um idioma que não está nesta compilação mostra inglês. **Aplicação: mudar idioma...** (`Ctrl+N g`) também o define, com pré-visualização em direto. A saída da CLI e do MCP continua em inglês. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... continuam em inglês em todos os idiomas. `true`: o vocabulário git do idioma escolhido (em português "enviar", "puxar", "integrar", "etiqueta"...). **Em direto.** |
| `ui.git_terms_english` | `false` | Só ja, ko e zh-CN: mostrar as palavras git em inglês (stage, unstage, staged, unstaged...) em vez da sua transcrição. Sem efeito enquanto `ui.translate_git_terms` for `true`. **Em direto.** |
| `ui.mouse_enabled` | `true` | `false` deixa o rato (e a seleção de texto) ao terminal. **Reiniciar.** |
| `ui.mouse_scroll_lines` | `3` | Linhas do Log por passo da roda, de 1 a 20. **Reiniciar.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` pede ao terminal o protocolo de teclado kitty (para que `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` cheguem); `off` nunca pede. **Reiniciar.** |
| `ui.logo_color` | `""` | Combinação de cores do logótipo na janela Acerca do GitPoulpi, um slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); vazio ou desconhecido significa a predefinição da compilação (`lavender`). Escrita quando clica no logótipo ou escolhe uma cor no Acerca. |

**Avatares** (desativados por predefinição; ativá-los envia um hash SHA-256 dos
e-mails dos autores ao Gravatar, e consulta a sua conta GitHub/GitLab ligada)

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (no círculo do commit do grafo em imagem), `author` (na coluna Autor), `off`. O antigo `ui.show_avatars` (`true`) continua a ser lido como `author` enquanto `ui.avatars` estiver em falta. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Ordem de pesquisa: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. Um anfitrião GitLab só é consultado através de um perfil ligado. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (gerado localmente) ou `initials`. Num terminal sem protocolo de imagens usa-se sempre o emblema com as iniciais. |

**Apresentação do diff e do editor**

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` ou `split` (`m` num diff). |
| `ui.diff_hunks_only` | `true` | `false` mostra o ficheiro inteiro com as alterações no sítio (`H`). |
| `ui.diff_show_whitespace` | `true` | Mostrar alterações só de espaços em branco (`s`). |
| `ui.diff_word_wrap` | `false` | Quebrar as linhas longas do diff (`w`; Definições > Diff e editor). |
| `ui.minimap` | `false` | Um minimapa ao lado da barra de deslocação do diff e do editor (`M`). |
| `ui.mermaid_detail` | `refs` | Detalhe do Copiar como Mermaid gitGraph do Log: `structure`, `refs`, `subjects`, `full` (memorizado a partir do popup; o valor por omissão de `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Direção desse grafo: `lr` (da esquerda para a direita) ou `tb` (de cima para baixo). |
| `ui.mermaid_fence` | `false` | Envolve o script Mermaid copiado num bloco de código `mermaid`. |
| `ui.mermaid_subject_width` | `40` | Os assuntos dos commits são cortados neste número de carateres nos ids do Mermaid (`subjects`, `full`). |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` ou `16`: com quantas cores desenhar. `auto` é truecolor exceto no Terminal da Apple (256), na consola Linux (16) e em terminais `*-256color` sem `COLORTERM` (256). Em direto. Veja [Temas](themes.md). |
| `ui.input_color` | `"theme"` | Fundo dos campos de texto: `theme` (o do próprio tema), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | A cor `custom`, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | O compositor de commits de IA abre com os ficheiros de cada proposta `expanded` ou `collapsed`. |
| `ui.blame_side` | `"right"` | Que lado da vista Blame tem a coluna de anotações (avatar, autor, idade, resumo): `right` (por omissão) ou `left`. |
| `ui.blame_width` | `46` | Largura da coluna de anotações do Blame em células (24 a 90). Arraste a sua margem ou prima `<` / `>` na vista; a nova largura é guardada aqui. |
| `ui.pr_terminology` | `"auto"` | `auto` (Pull Request no GitHub, Merge Request no GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Oculta o chip da barra de ferramentas com o último pull request ou merge request da branch em checkout quando este é fechado ou o merge é feito; aparece em seu lugar o chip esbatido "+ PR" (iniciar um novo pedido), como se a branch não tivesse nenhum. `false`: o chip mostra sempre o último pedido, seja qual for o estado. |

As opções do diff que altera dentro de um diff são guardadas aqui de imediato e
usadas por todos os diffs abertos depois.

**Datas**: `ui.date_format` (`"YYYY-MM-DD"`, a coluna de data do Blame / Histórico do ficheiro),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, a coluna Data do Log) e
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, os detalhes do commit, a dica da
coluna Data e *Copiar > Data*). Marcadores: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (dia da semana, completo / abreviado) `HH` `mm` `ss`; qualquer outra
coisa é copiada tal como está. Os nomes dos meses e dos dias da semana seguem
`ui.language`, e um mês escrito depois de um número de dia está no genitivo
quando o idioma o tem (russo, ucraniano, polaco, checo). Uma sequência
`D MMMM YYYY` é escrita com o ano primeiro em japonês, chinês e coreano
(`2024年3月5日`). `ui.date_locale` (`"system"`, a predefinição, ou `"english"`)
mantém os nomes em inglês seja qual for o idioma.

**Agentes, terminais e a pasta do separador**

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | A janela aceita sequer `ui show` / `ui propose` de agentes (Definições > Agentes e MCP > *Permitir que agentes de IA controlem a vista*). Desativado, nenhum agente a alcança. Veja [MCP e agentes](mcp-and-agents.md#mostrar-coisas-ao-utilizador). |
| `ui.launch` | `"off"` | Se um agente pode abrir uma janela do GitPoulpi quando não há nenhuma aberta: `off`, uma predefinição (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) ou `custom`. |
| `ui.launch_command` | `""` | O modelo `custom`: `{repo}`, `{session}`, `{gitpoulpi}`. Dividido em argumentos, sem shell. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (a linha de separadores oferece *Sair e cd aqui*) ou `auto` (cada saída muda a pasta da shell, e um novo separador executa a entrada predefinida de *Abrir em*). Veja [Seguir a pasta do separador ativo](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Antigo, veja abaixo e [Abrir em](open-in.md). Como o antigo *Terminal aqui* abria um terminal: `auto` (a predefinição `ui.launch`, senão `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), uma predefinição, ou `custom`. |
| `ui.terminal_launch_command` | `""` | O modelo `custom`; `{repo}` é a pasta. |
| `ui.open_in` | *(em falta)* | A lista [Abrir em](open-in.md), um array de tabelas `[[ui.open_in]]` com `label` e `command`. Em falta significa "ainda não configurada": a lista é calculada a partir das chaves antigas `ui.terminal_launch*` (um valor diferente de `auto` passa a ser a primeira entrada), do explorador de ficheiros do SO, dos multiplexadores e terminais detetados no ambiente (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) e dos editores encontrados no `PATH`. A primeira gravação a partir das Definições escreve a lista. `open_in = []` é uma lista vazia. Não se define com `config set` (edite o ficheiro). |
| `ui.open_in_default` | `""` | Etiqueta da entrada que "Abrir no predefinido" executa; vazio ou desconhecido significa a primeira entrada. |
| `ui.launch_terminal` | `"auto"` | O terminal em que os atalhos do ambiente de trabalho (Definições > Lançador) e o relançamento no Windows iniciam o GitPoulpi. `auto` escolhe entre os instalados (Windows: Windows Terminal, senão WezTerm, Alacritty, a consola simples; Linux: `xdg-terminal-exec`, senão o primeiro terminal encontrado; macOS: iTerm2, Ghostty, WezTerm ou kitty se instalados, senão Terminal), `default` é o do sistema operativo, ou um id de terminal como `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Só no modo de programador (veja `general.dev_mode`; só então aparece em Definições > Lançador, ignorado nos outros casos). `"current"`: os atalhos e o alias de comando iniciam a compilação que os criou. `"newest"`: iniciam a compilação mais recente entre `target/dist` e `target/release` do clone, decidida a cada clique, e iniciar uma ou outra nunca os reescreve. |
| `ui.command_alias` | `""` | O nome do comando que inicia o GitPoulpi a partir de um terminal, por exemplo `gp` (Definições > Lançador, `gitpoulpi alias create`). Vazio: nenhum. Apenas letras, dígitos, `-`, `_` e `.`. É só o nome que o botão **Criar: alias de comando** usa; nada existe até o criar. |
| `ui.command_alias_add` | `""` | O nome do segundo comando, por exemplo `gpa`, que executa `gitpoulpi --add`: abre a pasta atual num novo separador do GitPoulpi que já está aberto (Definições > Lançador, `gitpoulpi alias create gpa --add`). Vazio: nenhum. Mesmas regras de nome; criado em conjunto com `ui.command_alias` pelo botão **Criar**. |
| `ui.relaunch_in_terminal` | `true` | Apenas Windows: quando o exe é iniciado com clique duplo numa consola própria, o GitPoulpi reabre-se em `ui.launch_terminal` e termina. Desativado: fica nessa consola. |
| `ui.reuse_window` | `true` | Um clique num atalho do GitPoulpi (`gitpoulpi launch`) mostra a janela de um GitPoulpi que já está aberto, em vez de iniciar outro. Desativado: cada clique inicia uma nova janela. |

Os comandos de **`ui.open_in`** são primeiro divididos em argumentos (espaços em branco, `'...'`, `"..."`), depois as variáveis são substituídas dentro de cada argumento, de modo que uma pasta com espaços continua a ser um só argumento; nenhuma shell está envolvida a menos que o comando inicie uma (`sh -c '... "$GITPOULPI_CWD"'`). Variáveis: `${cwd}` (a pasta do separador), `${repo}` (raiz do repositório), `${name}` (nome base da pasta), `${branch}` (vazio em detached), `${web_url}` (a página web do remoto `origin` quando é um projeto GitHub ou GitLab, senão vazio; uma entrada que o use recusa-se a executar quando está vazio), `${env:NAME}` (vazio quando não definida). O antigo `{repo}` continua a significar `${cwd}`. Qualquer outro `${...}` é um erro que o nomeia. Exemplo:

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Antigo: `ui.terminal_launch` e `ui.terminal_launch_command` continuam a ser lidos (semeiam a lista enquanto `ui.open_in` estiver em falta: `custom` passa a "Terminal (custom)" com `{repo}` reescrito para `${cwd}`, o id de uma predefinição passa a essa predefinição) mas foram substituídos por `ui.open_in`.

### Imagens

| Chave | Predefinição | Significado |
|---|---|---|
| `ui.image_view` | `true` | Mostrar um ficheiro binário que é uma imagem (PNG, JPEG, GIF, WebP, BMP, ...) como imagem na vista Diff e na vista de ficheiro. Desativado: o aviso binário habitual. |
| `ui.image_view_max_mib` | `32` | As imagens maiores que isto (MiB) não são descodificadas; aparece uma ficha de informações. |
| `ui.image_view_max_unit` | `"auto"` | A unidade em que as Definições mostram o limite: `mib`, `gib` (1024 MiB) ou `auto`. Só de apresentação: o limite é sempre `image_view_max_mib` MiB, de 1 a 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | Num terminal sem protocolo de imagem: `halfblock` (caracteres de meio bloco coloridos) ou `card` (só a ficha de informações). |
| `ui.image_view_pixelated_from` | `"2x"` | Zoom a partir do qual ampliar mostra píxeis quadrados em vez de suavizar: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | Os painéis antes e depois partilham o mesmo zoom e posição de deslocação. |
| `ui.image_editors` | *(em falta)* | Os editores de imagem de "Editar em", um array de tabelas `[[ui.image_editors]]` com `label` e `command`. Em falta significa "ainda não configurada": a lista são os editores detetados nesta máquina (programas no `PATH`, instalações Flatpak, pastas de instalação conhecidas no Windows, `/Applications` no macOS), depois a aplicação predefinida do SO. A primeira gravação a partir das Definições escreve a lista. `image_editors = []` é uma lista vazia. Não se define com `config set` (edite o ficheiro). |
| `ui.image_editor_default` | `""` | Etiqueta do editor que "Editar em predefinido" executa; vazio ou desconhecido significa a primeira entrada. |

Os comandos de **`ui.image_editors`** são primeiro divididos em argumentos (espaços em branco, `'...'`, `"..."`), depois as variáveis são substituídas dentro de cada argumento, de modo que um caminho com espaços continua a ser um só argumento; nenhuma shell está envolvida. Variáveis: `${file}` (o caminho absoluto da imagem), `${dir}` (a sua pasta), `${name}` (o seu nome de ficheiro), `${repo}` (a raiz do repositório, vazio numa cópia temporária), `${env:NAME}` (vazio quando não definida). Qualquer outro `${...}` é um erro que o nomeia. Uma imagem que não é o ficheiro da árvore de trabalho (a versão de um commit, o índice, um ficheiro eliminado) é aberta como uma **cópia temporária só de leitura**, para que um editor não possa tocar no repositório. Os editores sem linha de comandos (Affinity Photo da Microsoft Store, algumas instalações do Photoshop) não estão nas predefinições: escreva uma entrada personalizada. Exemplo:

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**Ainda não usadas** (aceites, guardadas, mas nada as lê):
`ui.graph_colorization` (removidas das Definições;
ainda aceites ao carregar), `ui.date_word_format` (removida das Definições; ainda
aceite ao carregar). O antigo
`ui.diff_view_mode` é lido uma vez como `diff_layout` / `diff_hunks_only`.

## `[editor]`

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `editor.tab_size` | `4` | Colunas por paragem de tabulação, de 1 a 8 (editor e diffs). |
| `editor.eol` | `"auto"` | Fim de linha que o editor integrado escreve: `auto` (mantém o do ficheiro), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Cores da linguagem no editor. |
| `editor.line_numbers` | `true` | Números de linha no editor e nos diffs. |
| `editor.follow_selection` | `true` | Com o editor aberto, selecionar outro ficheiro no painel direito também o abre no editor, depois de perguntar se as alterações por guardar devem ser guardadas. Um ficheiro que não pode ser editado abre na vista habitual; um ficheiro de um commit sai do editor para o seu diff. |

## `[merge]`

O editor de conflitos de merge ([Resolver conflitos de merge](merge-conflicts.md)). As Definições > Diff e editor mostram-no.

| Chave | Predefinição | Significado |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | A largura, em colunas, da área do editor a partir da qual A e B são mostrados lado a lado (as linhas longas deslocam-se na horizontal: Shift+roda, Shift+setas). Mais estreito, mostra um lado de cada vez atrás de separadores. `40` é o valor mais baixo, `0` significa sempre lado a lado (o aviso abaixo de 40 colunas mantém-se). O editor oculta os painéis Refs e Status enquanto está aberto, por isso a sua área é normalmente a janela inteira. |

## `[git]`, `[credentials]`, `[ssh]`

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `git.pull_mode` | `"merge"` | O que o Pull faz (botão, `Ctrl+G p`, `Git: pull`): `merge` (fast-forward quando possível), `ff_only`, `rebase`. Sempre passado explicitamente ao git. O antigo `fetch_all` é lido como `merge`. |
| `git.push_behavior` | `"direct"` | `direct` faz push para o remoto predefinido; `ask` mostra primeiro o menu de remotos. |
| `git.merge_fast_forward` | `"never"` | O que um merge que inicia (menu da branch, arrastar e largar, paleta, `gitpoulpi merge`) faz quando a branch podia avançar por fast-forward: `never` faz sempre um commit de merge (`--no-ff`), `allowed` faz fast-forward quando possível (a predefinição do git, aplica-se `merge.ff`), `only` faz fast-forward ou recusa. A escolha própria de um repositório (Definições > Opções de merge, `merge_fast_forward` no seu `repo-state.toml`) prevalece; as opções `--ff`, `--no-ff` e `--ff-only` da CLI prevalecem sobre ambas. Em direto. |
| `git.default_encoding` | `"utf-8"` | Codificação para texto que **não** é UTF-8 válido (conteúdo de ficheiros, blame, diffs, mensagens de commit sem cabeçalho `encoding`): qualquer etiqueta WHATWG (`windows-1252`, `shift_jis`, ...). Apenas visualização; o stage nunca recodifica. Um repositório pode sobrepor-se-lhe. |
| `git.lfs_enabled` | `true` | Mostrar e controlar o Git LFS. Um repositório pode sobrepor-se-lhe. Veja [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Antes de um pull ou push de um repositório que usa LFS, instalar os seus filtros LFS / hook pre-push (`git lfs install --local`, só este repositório) quando em falta. Veja [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Não usadas*: o `core.autocrlf` / `core.eol` de um repositório são definições do próprio git, editadas em Definições > Git e codificação (ou `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | Autenticação HTTPS para fetch, pull, push, clone: `credential_helper` (o do git), `prompt` (ignora os helpers; o git pergunta num terminal, por isso falha no daemon em segundo plano), `none` (apenas remotos públicos). Os antigos valores `encrypted_file` são lidos como `credential_helper`. |
| `ssh.key_type` | `"ed25519"` | Tipo criado por *Gerar par de chaves*: `ed25519`, `rsa` (4096 bits), `ecdsa` (NIST P-256); o ficheiro é `gitpoulpi_<type>`. O próprio SSH usa sempre o seu ssh-agent e a sua configuração ssh. |
| `ssh.private_key_path`, `ssh.public_key_path` | não definidas | Registadas por *Gerar par de chaves*. Quando `ssh.private_key_path` está definida, fetch, pull, push e clone executam o git com `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`; `ssh.public_key_path` faz o mesmo (o ssh escolhe a chave correspondente do agente) quando o caminho privado está vazio. Um `GIT_SSH_COMMAND`/`GIT_SSH` no ambiente ou um `core.sshCommand` na configuração do git (a fixação por repositório, `gitpoulpi ssh-command set --key`) tem prioridade. Lidas antes de cada transferência, por isso as alterações aplicam-se à seguinte. |

## `[[integrations]]` (alojamento e gestores de issues)

Escrito pelas Definições > Integrações ou por `gitpoulpi hosting integrations add`.
Cada tipo tem um cliente (apenas tokens pessoais, sem OAuth).

```toml
[[integrations]]
id = "corp"                         # o que --id aceita; o token fica guardado em integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # a raiz da API deriva dele
token_ref = "integration:corp"      # uma referência, nunca o token
name = "Corp GitLab"                # nome a mostrar, opcional
account = "me@example.com"          # opcional, veja abaixo
enabled = true                      # false: ignorado na correspondência de hosts
```

Um servidor autoalojado pode ser http simples: `url = "http://localhost:3000"` é
mantido tal como foi escrito (o esquema e a porta chegam a cada pedido, e um remoto
nesse host corresponde ao perfil). Um token enviado por http simples viaja sem
cifragem, por isso use http apenas numa rede local ou de confiança.

`kind` aceita também `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` e `gerrit`
(o Gitea e o Forgejo partilham um só cliente; um perfil Forgejo assume `https://codeberg.org`). O Jira, o
Trello, o Linear, o YouTrack e o Redmine são apenas gestores de issues; o Gerrit é só de revisão (as suas alterações são os
pull requests, não tem issues). O Bitbucket Cloud não tem gestor de issues.

`account` é a identidade não secreta ao lado do token, vazia e omitida na maioria
dos tipos: o e-mail da conta Atlassian para `jira_cloud` e `bitbucket` (uma conta
Bitbucket vazia significa um token de acesso de workspace ou de repositório,
enviado como Bearer), a chave de API para `trello`, o nome de utilizador para `gerrit` (o token é a sua palavra-passe
HTTP, enviada como HTTP basic). O próprio token continua a ser
o único segredo. Para `azure_devops` o `url` é
`https://dev.azure.com/<organization>`; um URL
`https://<organization>.visualstudio.com` é lido como a mesma organização. Para
`azure_devops_server` é a raiz da coleção, como
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (a conta ligada) e `ssh` (a escolha de chave do perfil) são escritos por
si. O token vem de `GITPOULPI_TOKEN_<ID>` (id em maiúsculas, caracteres não
alfanuméricos como `_`), senão do token guardado, senão do seu credential helper
para esse anfitrião quando só um perfil o usa (apenas perfis GitHub e GitLab: os
outros tipos nunca leem o credential helper). Detalhes: [Integrações de
alojamento e de gestores de issues](hosting-integration.md). A antiga tabela
`[[hosting]]` *não é usada*.

Para `local_markdown` (uma pasta de notas Markdown como origem das issues) o `url` é a
pasta, relativa a cada repositório ou absoluta, e não há token. A sua tabela opcional
`[integrations.local]` define `statuses` (a ordem das colunas do quadro, por omissão
`todo`, `doing`, `done`), `done_status` (por omissão `done`), `reference_prefix`
(`LOC` mostra `LOC-12`) e `me` (o que significa «atribuídas a mim»).
Ver [Issues locais e Obsidian](local-issues.md).

### `[integrations.radar]` (listas de toda a conta)

Tabela opcional de um perfil de code host (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; não o Bitbucket Cloud). Ausente
significa todas as chaves desligadas. Escrita por `gitpoulpi hosting integrations radar`; lida por
`gitpoulpi hosting radar` e pelo Radar, e editada em Definições > Integrações (o bloco Radar de um perfil ligado).

```toml
[integrations.radar]
review_requests = true    # pedidos à espera da minha revisão, todos os repositórios
open_requests = false     # todos os pedidos abertos dos meus namespaces
include_drafts = false    # manter os pedidos em rascunho nestas listas
assigned_issues = true    # issues abertas atribuídas a mim, todos os repositórios
namespaces = ["acme"]     # organizações / grupos onde pesquisar pedidos abertos
review_mode = "ask"       # ask | clone | api
```

| Chave | Predefinição | Significado |
|---|---|---|
| `review_requests` | `false` | Pedidos abertos que pedem a sua revisão, com o repositório clonado ou não |
| `open_requests` | `false` | Todos os pedidos abertos do seu próprio namespace e de cada organização ou grupo a que pertence (ou de `namespaces`), nunca «tudo o que o token vê» |
| `include_drafts` | `false` | Os pedidos em rascunho ficam de fora destas listas, salvo se isto estiver ligado |
| `assigned_issues` | `false` | Issues abertas atribuídas a si, em todos os repositórios |
| `namespaces` | `[]` | Organizações / grupos onde pesquisar para `open_requests`; vazio significa o seu próprio namespace mais todas as associações (GitHub: pesquisam-se no máximo 12, GitLab: 10 grupos e 10 projetos próprios, Gitea, Forgejo e Gerrit: 10; o Azure DevOps usa nomes de projeto, o Bitbucket Data Center chaves de projeto, o Gerrit prefixos de caminho de projeto) |
| `review_mode` | `"ask"` | O que faz a revisão de um pedido cujo repositório não está clonado: `ask`, `clone` (cloná-lo e depois rever) ou `api` (rever através da API do host) |

As respostas ficam em cache por perfil em `radar-account.json`, junto de
`config.toml`, durante cinco minutos; o ficheiro é uma cache, pode apagar-se sem receio.

### O armazém de confiança (não está em `config.toml`)

Aquilo para que cada integração e cada perfil de IA com chave de API foi considerado de confiança (o seu
endereço, o id da conta, uma chave de certificado fixada, se o http simples foi permitido) fica em
`trust.toml` na pasta de dados privada, só do utilizador (`<dados>/private/trust.toml`:
`%APPDATA%\GitPoulpi\private\` no Windows, `~/.local/share/gitpoulpi/private/`
no Linux). Não faz parte de `config.toml` de propósito: quem pode editar a
configuração não deve poder alargar a confiança editando-a. Altere-o com
`gitpoulpi hosting integrations trust` e `gitpoulpi ai trust` (veja
[a verificação de segurança](hosting-integration.md#a-verificação-de-segurança)); o ficheiro é
reescrito pelo GitPoulpi, não à mão.

## `[mcp]` (o que os agentes de IA podem fazer)

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `mcp.allow_mutations` | `false` | Desativado: as ferramentas MCP que alteram seja o que for são recusadas; leituras, `ui_*` e propostas continuam a funcionar. `true` deixa os agentes alterar o repositório. |
| `mcp.mutation_allowlist` | `[]` | Não vazio: só estes nomes de ferramenta podem alterar coisas (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: recusa as ferramentas `ui_*` que mexem na sua janela (não `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (a janela mostra um chip que o utilizador confirma), `auto` (aplicado a menos que esteja ocupado), `off`. `--ui-control` no comando do servidor sobrepõe-se por agente. Aplica-se também a `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | Permitir `ui_propose_*` / `gitpoulpi ui propose`: caixas de diálogo pré-preenchidas que o próprio utilizador confirma. |
| `mcp.allow_history_purge` | `false` | Só no `config.toml` (sem linha nas Definições). `true` lista a ferramenta `rewrite_purge` para os agentes (continua a precisar de `allow_mutations`). Desativado, um agente nem vê a ferramenta: uma purga elimina de vez o histórico antigo de uma reescrita do histórico, o respetivo desfazer e os reflogs. A CLI e o TUI não são limitados por ela. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` e `gitpoulpi-mcp` recusam-se a arrancar, com uma mensagem que nomeia esta chave. Definições > Agentes e MCP > *Servidor MCP*. Lida quando um agente inicia o servidor. |
| `mcp.bind` | `"127.0.0.1:8808"` | O endereço em que `--http` escuta quando não indica nenhum (`--http ADDR` / `--bind` prevalecem). Apenas loopback, a menos que `--allow-remote`. Vazio ou o antigo `"stdio"` significam a predefinição. Definições > Agentes e MCP > *Porta do servidor MCP* edita a porta e mantém o anfitrião. |
| `mcp.http_auth` | `true` | `--http` só responde a um pedido com `Authorization: Bearer <token>` (o token de `gitpoulpi mcp token`). `false` remove a verificação; o servidor avisa então no stderr, e `--allow-remote` é recusado. Definições > Agentes e MCP > *Autenticação HTTP*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (um certificado gerado uma só vez na pasta de dados privada, com a sua impressão digital SHA-256 mostrada no arranque) ou `files` (`tls_cert` e `tls_key`). Com o TLS ativo, o servidor termina o TLS ele próprio e não existe nenhum recetor simples. Definições > Agentes e MCP > *TLS*. |
| `mcp.tls_cert` | `""` | Cadeia de certificados PEM usada com `tls = "files"`. |
| `mcp.tls_key` | `""` | Chave privada PEM usada com `tls = "files"`. |
| `mcp.behind_proxy` | `false` | Declara que um túnel ou proxy inverso à frente do endpoint assegura a cifra. Não se consegue verificar a partir de dentro do servidor, por isso `--allow-remote` sem TLS exige que isto esteja definido. O troço entre o proxy e o servidor circula em texto simples nessa rede. Definições > Agentes e MCP > *Atrás de um túnel ou proxy inverso*. |
| `mcp.allow_sensitive_staging` | `false` | Só em `config.toml`. Desativado, as ferramentas de stage do MCP não oferecem `allow_sensitive`, e um stage que acrescentaria um ficheiro assinalado (veja `[privacy]`) é recusado com uma mensagem a dizer ao agente que lhe pergunte. Ativado, um agente pode passar `allow_sensitive`. A CLI e a TUI não são controladas por isto: perguntam-lhe. |

`allow_mutations`, `mutation_allowlist` e `allow_ui` controlam apenas o servidor
MCP; a CLI é a sua própria shell e não é controlada por elas. Veja [MCP e
agentes](mcp-and-agents.md#permissões).

## `[ai]` (IA integrada, desativada por predefinição)

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `ai.enabled` | `false` | O interruptor. Desativado, nada de IA aparece em lado nenhum e nada é enviado. |
| `ai.default_profile` | `"default"` | Id do perfil que as ações usam quando não indicam nenhum. |
| `ai.global_instructions` | `""` | Texto acrescentado ao prompt de cada ação. Pode usar `${repo}`, `${uilang}`, ... (veja [IA > Variáveis do prompt](ai.md#variáveis-do-prompt)); o mesmo para as instruções por ação e do repositório. |
| `ai.actions.<action>.profile` | `""` | Perfil para uma ação; vazio = o predefinido. Ações: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`. Uma área de trabalho (`workspaces.toml`, `[workspace.ai_instructions]` e `[workspace.ai_overrides.<action>]`) e um repositório podem acrescentar ou substituir estas instruções: ver [IA](ai.md#instruções). |
| `ai.actions.<action>.instructions` | `""` | Texto acrescentado só para essa ação. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` ou `no`: se o agente local de um rascunho de revisão de código trabalha dentro do repositório nessa execução, sobrepondo-se ao `workdir` do perfil de IA. Um repositório pode definir o seu (`gitpoulpi ai review-context --repo`). Ver [IA > Contexto da revisão](ai.md#contexto-da-revisão). |
| `ai.review.include_claude_md` | `true` | Dar a um rascunho de revisão de código o `CLAUDE.md` do repositório (na raiz, lido na branch a que o pedido se destina; cortado aos 8 KB). |
| `ai.review.include_agents_md` | `true` | O mesmo para o `AGENTS.md`. |
| `ai.review.extra_file` | `""` | Mais um ficheiro para um rascunho de revisão de código: um caminho relativo à raiz do repositório, um caminho absoluto ou uma ligação para um ficheiro de um code host (lido através do perfil de integração correspondente, senão de forma anónima). Todos os ficheiros juntos são cortados aos 16 KB. |
| `ai.privacy.redact_secrets` | `true` | Substitui por `[REDACTED:<regra>]` todos os segredos que o analisador reconhece num pedido (um token, um bloco de chave privada, uma palavra-passe num URL, um valor `password = ...` que pareça aleatório) antes de ele sair. As Definições > IA mostram um aviso vermelho enquanto estiver desativado. |
| `ai.privacy.exclude_sensitive_files` | `true` | Deixa de fora do que é enviado o diff de todos os ficheiros que correspondam a `privacy.sensitive_patterns`. Um agente local que trabalhe na pasta do repositório ainda pode ler ficheiros ele próprio: veja [IA > Privacidade](ai.md#privacidade). |
| `ai.limits.conflict_bytes` | `40000` | Máximo de bytes que um conflito de merge (os seus blocos base, atual e recebido em conjunto) pode ter quando «Sugerir uma resolução» o envia. Acima disso, o GitPoulpi não falha: avisa e oferece *Enviar mesmo assim* (só este pedido), *Abrir definição* e *Cancelar*; um conflito nunca é cortado. O limite é uma proteção do próprio GitPoulpi, não do modelo: aumente-o quando o seu modelo tiver margem. Mantido entre 1000 e 50000000; definível em Definições > IA > Limites. Os outros limites abaixo funcionam da mesma forma e estão todos nesse grupo. |
| `ai.limits.diff_bytes` | `60000` | Máximo de bytes do diff que uma mensagem de commit, uma explicação, um nome de stash, uma descrição de pull request, um plano de commits ou uma divisão de commit pode levar, e do diff completo de uma revisão de código. Acima disso nada é cortado em silêncio: na TUI, *Acima do limite da IA* oferece *Enviar tudo* (só este pedido, nada cortado), *Enviar truncado* (o fim fica de fora, como antes), *Abrir definição* e *Cancelar*; a CLI e o MCP truncam e indicam-no em `truncated` (`--send-all` envia tudo). Mantido entre 1000 e 50000000. |
| `ai.limits.review_file_bytes` | `12000` | Máximo de bytes do diff de um ficheiro numa revisão de código. Acima disso, a mesma pergunta e as mesmas duas respostas; *Enviar truncado* encurta esse ficheiro com uma nota visível. Mesmo intervalo. |
| `ai.limits.review_text_bytes` | `4000` | Máximo de bytes da descrição do pull request revisto. Acima disso, *Enviar truncado* corta a descrição com uma nota `[text cut]`. Mesmo intervalo. |
| `ai.limits.review_threads_bytes` | `6000` | Máximo de bytes da lista de comentários de revisão que já existem (dada para que o modelo não os repita). Acima disso, *Enviar truncado* corta a lista. Mesmo intervalo. |
| `ai.limits.review_context_file_bytes` | `8192` | Máximo de bytes de um ficheiro de contexto do projeto (`CLAUDE.md`, `AGENTS.md`, o ficheiro adicional) dado a uma revisão de código. Acima disso, *Enviar truncado* encurta o ficheiro com uma nota. Mesmo intervalo. |
| `ai.limits.review_context_bytes` | `16384` | Máximo de bytes de todos os ficheiros de contexto em conjunto. Acima disso, *Enviar truncado* encurta o ficheiro que o ultrapassa e deixa de fora os seguintes. Mesmo intervalo. |
| `ai.sent_preview.<ação>` | `true` | Uma chave por ação (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`): mostra "O que é enviado" antes de o pedido sair. O "Não mostrar de novo para ..." do diálogo escreve `false`; Definições > IA > *Mostrar o que é enviado antes de...* volta a ativá-las. |

A tabela `[preferences.ai_review]` própria de um repositório (no seu `repo-state.toml`, ou a entrada de um repositório
não clonado em `remote-reviews.toml`) aceita as mesmas chaves, todas opcionais, mais `no_extra_file = true`: nenhum
ficheiro extra para esse repositório, diga o que disser `ai.review.extra_file`. Uma chave por definir, um `extra_file`
por definir e um `no_extra_file` em falta herdam o valor global; definir `extra_file` com um ficheiro limpa o
interruptor e vice-versa.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # vazio = a predefinição do tipo
base_url = ""           # vazio = a predefinição do tipo (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # onde a chave de API está guardada; nunca a chave
command = ""            # só local_agent: o programa exato ("claude", um alias, um wrapper, um caminho)
harness = ""            # só local_agent: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; vazio = inferido de `command`
args = []               # só local_agent, p. ex. ["-p", "--model", "haiku"]
run_mode = "headless"   # só local_agent: headless (predefinição) | terminal (o ecrã interativo do agente num terminal virtual)
terminal_args = []      # só local_agent: os argumentos do modo terminal (args mantém o conjunto headless)
workdir = "default"     # só local_agent: default | dedicated | repository (default = pasta dedicada no modo terminal, a pasta do GitPoulpi em headless)
timeout_secs = 0        # só local_agent: segundos antes de o agente ser parado; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # só local_agent, opcional: variáveis definidas apenas para esse processo
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = uma das variáveis do próprio GitPoulpi
```

`harness` escolhe as sugestões que as Definições > IA oferecem (argumentos sem
interface, opções, opção e lista de modelos, variáveis de ambiente)
independentemente de `command`; ambas as chaves são omitidas do ficheiro quando
vazias. Os valores de ambiente nunca são registados em log e ficam ocultos nas
Definições quando parecem segredos.

As Definições > IA oferecem predefinições para `harness`/`command`/`args`/`env` (as CLIs
de agentes comuns, com as suas opções sem interface, opção de modelo e variáveis
de ambiente úteis) e para `kind`/`base_url`/`model` (os fornecedores HTTP
alojados e locais); veja [IA > Configurar a IA](ai.md#configurar-a-ia).

Um perfil chamado *Default* é criado quando não há nenhum. Os ficheiros antigos
com `[[ai.presets]]` / `[ai.defaults]` são migrados ao carregar e mantidos em
`[[ai.legacy_presets]]`. Tudo o resto: [Assistência de IA](ai.md).

## `[repos]`, `[session]`

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `repos.clone_root` | `""` | Onde a caixa de diálogo Clonar começa; vazio = `%USERPROFILE%\source\repos` ou `~/src`. |
| `repos.clone_start_source` | `"last"` | A fonte em que a caixa de diálogo Clonar abre: `last` (predefinição, ou vazio) = a que mostrava quando foi fechada da última vez, `url` = sempre Clonar por URL, ou o id de um perfil de integração (`id` na sua entrada `[[integrations]]`). Um perfil que já não exista, esteja desativado ou não esteja ligado recorre a Clonar por URL. A última fonte em si é estado, guardado em `clone-source.toml` na pasta de configuração, não aqui. |
| `repos.browse_root` | `""` | Onde Procurar abre; vazio = a última pasta escolhida, depois a pasta pessoal. |
| `repos.show_hidden` | `false` | O seletor de pastas mostra pastas ocultas (`Alt+H` alterna e guarda). |
| `repos.show_siblings` | `false` | A árvore da esquerda do seletor de pastas lista também as outras pastas de cada nível aberto, como uma hierarquia real (`Alt+S` alterna e guarda). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Onde o GitPoulpi guarda o que memoriza sobre um repositório (filtros de refs, estado de recolha, a substituição pessoal do fluxo de trabalho, a cache de alojamento): `in_git_dir` = `<pasta git>/gitpoulpi` (partilhada por todos os worktrees, nunca aparece em `git status`), `in_worktree` = `<repo>/.gitpoulpi` (o único sítio onde uma equipa pode fazer commit de `workflow.toml`), `user_data` = `<pasta de dados do SO>/repos/<nome>-<id>` (`%APPDATA%\GitPoulpi\data` no Windows, `$XDG_DATA_HOME/gitpoulpi` no Linux, `~/Library/Application Support/GitPoulpi` no macOS). O valor antigo `".gitpoulpi"` continua a significar `in_worktree`. Alterá-lo não move nada de volta: os dados encontrados numa `<repo>/.gitpoulpi` antiga são movidos para o local escolhido na primeira vez que forem precisos. |

## `[log]` (o registo da aplicação)

O que o separador «Aplicação» do Registo de atividade mostra e o que «Copiar diagnóstico» cita. Detalhes: [Quando algo falha](troubleshooting.md#quando-algo-falha).

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `log.level` | `"info"` | A linha mais detalhada que é registada: `error`, `warn`, `info`, `debug` ou `trace`. A caixa de nível do Registo de atividade altera-a (e guarda-a aqui). |
| `log.retention_days` | `7` | Dias de ficheiros de registo mantidos, incluindo hoje: de 1 a 3650, dez anos (um valor maior ou menor é limitado). É sempre um número de dias, seja qual for a unidade em que as Definições o mostram. Os ficheiros mais antigos são eliminados quando o GitPoulpi arranca. O ficheiro de um dia também tem o limite de `log.max_file_mb`, por isso o pior caso é cerca de `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | A unidade em que as Definições mostram `log.retention_days`: `days`, `weeks`, `months`, `years` ou `auto`. Só de apresentação, nunca muda durante quanto tempo os ficheiros são mantidos. Uma semana são 7 dias, um mês são 30 dias (não um mês do calendário, para serem os mesmos dias seja quando for aplicado) e um ano são 365 dias. `auto`, e uma configuração escrita antes de esta chave existir, escolhe a maior unidade que divide os dias exatamente (14 dias lêem-se "2 semanas", 45 dias "45 dias"). Uma unidade que não divide os dias (`retention_days = 10` com `weeks`) é ignorada e os dias são mostrados. As Definições escrevem as duas chaves; um GitPoulpi mais antigo ignora a unidade e continua a ler os dias. |
| `log.max_file_mb` | `5` | O ficheiro de um dia é rodado para uma única cópia `.old` neste tamanho, por isso um dia nunca ocupa mais do dobro. |
| `log.max_file_unit` | `"auto"` | A unidade em que as Definições mostram o tamanho: `mib`, `gib` (1024 MiB) ou `auto`. Só de apresentação: o tamanho é sempre `max_file_mb` MiB, de 1 a 1024. |
| `log.timings` | `true` | O separador Operações Git do Registo de atividade mostra quanto tempo cada operação demorou (`Push - 123 ms`: abrir um repositório, atualizar, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). As durações são medidas pela janela, do pedido à resposta do daemon, por isso incluem a ida e volta por IPC. Desligado, oculta as durações e as entradas das operações bem-sucedidas. As falhas são sempre listadas. |
| `log.extended` | `false` | «Registo alargado»: o separador Operações Git lista também as operações em segundo plano (a atualização periódica que consulta o estado, o auto-fetch) com as durações. São sempre escritas no registo da aplicação ao nível `debug`. |
| `log.commit_count` | `"system-git"` | Como o `carregados / total` da barra de estado obtém o total. `"system-git"` executa `git rev-list --count` (um processo, o mais rápido numa máquina normal: o git lê o ficheiro commit-graph). `"libgit2"` conta no próprio processo com um revwalk da libgit2 e não inicia nada: escolha-o quando todos os processos de linha de comandos demoram a arrancar, tipicamente porque um antivírus com análise em tempo real inspeciona cada um; com 50 000 commits demora cerca de 200 ms em segundo plano em vez de 12 ms, mas nenhum processo é iniciado. `"off"` nunca conta: a barra mostra só os commits carregados. Aplica-se em direto. Uma contagem filtrada por caminho usa sempre o git do sistema. |
| `log.use_mailmap` | `true` | Mostrar cada autor como o `.mailmap` do repositório (e `mailmap.file`) o nomeia no Log, no blame e no histórico de um ficheiro: o daemon pede ao git `%aN`/`%aE` (o backend do sistema) ou aplica ele próprio os mesmos ficheiros (o backend libgit2), por isso a resposta é idêntica. Os commits guardados não são alterados e nada é reescrito. Pesquisar o Log por autor corresponde aos nomes tal como são mostrados. Aplica-se de imediato; o Log volta a ler dentro do seu intervalo de 2 s. |

Os ficheiros são `gitpoulpi-YYYY-MM-DD.log` na pasta de estado do sistema (`~/.local/state/gitpoulpi/logs` no Linux; `<cache>\logs` no Windows e no macOS; `<config>/cache/logs` com `GITPOULPI_CONFIG_DIR`). Os tokens, as palavras-passe e os cabeçalhos `Authorization` são mascarados antes de se escrever o que quer que seja.

## `[search]` (a pesquisa na área de trabalho)

O que `gitpoulpi search` (e a sua ferramenta MCP) percorre. Detalhes: [Áreas de trabalho e pesquisa na CLI](cli-reference.md#áreas-de-trabalho-e-pesquisa).

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `search.commit_limit` | `200000` | Commits (os mais recentes primeiro, sobre todas as branches, branches remotas e tags) cujas mensagens o índice de pesquisa mantém em memória por repositório, cerca de 100 bytes mais a mensagem cada. Um histórico maior é cortado aí e a resposta diz que é `partial`. `0` desliga a pesquisa de commits (branches, tags, stashes e worktrees continuam a ser pesquisados). Lida em direto: a pesquisa seguinte obedece a uma alteração. |

## `[rewrite]` (reescrita do histórico)

O que `gitpoulpi rewrite` e o painel de reescrita do histórico protegem. Pormenores: [Reescrita do histórico](history-rewrite.md).

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Branches (nomes, ou padrões em que `*` corresponde a qualquer texto) que uma reescrita deixa em paz quando já foram enviadas, a menos que a execução as liste (`--include-protected BRANCH`; `--include-all-protected` para o push; a caixa por branch no painel). Somam-se às bases que o `workflow.toml` de um repositório marca como `protected`. O push de uma reescrita ignora-as da mesma forma. É a proteção do próprio GitPoulpi: a proteção de branches do alojamento continua a decidir se o push forçado é aceite. Lida em direto: a pré-visualização seguinte respeita uma alteração. |
| `rewrite.external_scripts` | `["shell"]` | Que linguagens de scripts externos a regra Script pode usar: qualquer uma de `"shell"` (bash ou sh; pwsh ou powershell para scripts PowerShell), `"python"`, `"lua"` (um Lua instalado; o Lua integrado está sempre ativo e não precisa de entrada). **Um script externo não corre em sandbox: executa-se com as suas permissões.** Uma linguagem que não está listada fica esbatida no painel e é recusada (`scripts_disabled`) pela CLI, pelo MCP e pelo daemon. Mesmo listada, um script externo que vem do repositório ou de uma receita partilhada só corre depois de você confiar nele (`gitpoulpi rewrite script trust FICHEIRO`, ou a pergunta no painel); a confiança tem por chave o hash do conteúdo do script, por isso um script alterado volta a perguntar. Ative o Python editando o `config.toml` (uma lista não se pode definir com `config set`): `[rewrite]` e depois `external_scripts = ["shell", "python"]`. Lida em direto. |
| `rewrite.script_timeout_ms` | `5000` | Quanto tempo um script externo tem para responder a um commit. Num timeout, toda a árvore de processos é morta e a reescrita falha (`script_failed`, a indicar o commit); nada se moveu. `0` lê-se como a predefinição. Lida em direto. |
| `rewrite.script_timeout_unit` | `"auto"` | A unidade em que as Definições mostram o limite de tempo: `milliseconds`, `seconds` ou `auto`. Só de apresentação: o limite é sempre `script_timeout_ms` milissegundos. As Definições aceitam de 1 a 600000 (dez minutos); `0` continua a ler-se como a predefinição. Lida em direto. |
| `rewrite.script_total_timeout_s` | `3600` | Quanto tempo pode demorar no total uma passagem de um script externo. `0` lê-se como a predefinição. Lida em direto. |
| `rewrite.script_total_timeout_unit` | `"auto"` | A unidade em que as Definições mostram o limite de tempo: `seconds`, `minutes`, `hours` ou `auto`. Só de apresentação: o limite é sempre `script_total_timeout_s` segundos. As Definições aceitam de 1 a 86400 (um dia); `0` continua a ler-se como a predefinição. Lida em direto. |

## `[updates]` (procurar uma versão mais recente)

O que o chip de versão da barra de título faz quanto a versões mais recentes. Detalhes: [Atualizações](updates.md).

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` pergunta ao GitHub uma vez por dia, em segundo plano, se existe uma versão mais recente, e transforma o chip de versão em `vX.Y.Z disponível`. `"auto"` também transfere e prepara a atualização sozinho quando esta compilação veio de um arquivo de versão (continua a perguntar antes de reiniciar); para qualquer outra compilação comporta-se como `"notify"`. `"off"` nunca faz um pedido. Aplica-se em direto. |
| `updates.channel` | `"stable"` | `"stable"` considera apenas versões completas; `"prerelease"` oferece também versões candidatas (`-rc.1`). Aplica-se em direto. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | O `dono/nome` no github.com cujas versões são verificadas. Enquanto esse repositório for privado, o GitHub responde 404, a menos que haja um token guardado para o github.com (Definições > Integrações); sem ele, a verificação falha em silêncio e escreve uma linha no registo da aplicação. |

Um administrador pode definir `mode`, `channel` (como `release_channel`) e `repo`, além de algumas outras opções de atualização, num ficheiro de política para toda a máquina ou no registo do Windows. Uma chave que a política define prevalece sobre `config.toml` e aparece nas Definições como **Definido pelo seu administrador**. O `mode` na política é um teto: pode escolher um valor inferior, nunca superior. Veja [Distribuição empresarial](enterprise-distribution.md#o-ficheiro-de-política-e-o-registo).

## `[privacy]` (ficheiros sensíveis)

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Padrões de caminho ao estilo do gitignore. Um padrão sem `/` corresponde ao nome do ficheiro a qualquer profundidade, um com `/` ao caminho inteiro (um `/` inicial ancora-o na raiz); `*` e `?` ficam dentro de uma pasta, `**` atravessa pastas; um `!` volta a permitir o que um padrão anterior apanhou e a última correspondência vence; as maiúsculas são ignoradas. Usado pela proteção de stage e por `ai.privacy.exclude_sensitive_files`. |
| `privacy.staging_guard` | `true` | Colocar em stage um ficheiro que corresponda a um padrão, ou cujo conteúdo contenha um segredo reconhecido, pergunta primeiro (TUI), é recusado a menos que se indique `--allow-sensitive` (CLI) ou que `mcp.allow_sensitive_staging` deixe o agente passar `allow_sensitive` (MCP). Os ficheiros de texto são lidos até 1 MiB cada e 32 MiB no total; os ficheiros binários só são comparados pelo nome. Veja [Stage e commit](staging-and-committing.md#ficheiros-sensíveis). |

## `[conflict_prevention]` (listas de branches base)

Que branches a deteção proativa de conflitos pode tomar como base da branch atual é uma lista de padrões (veja [Resolver conflitos de merge](merge-conflicts.md#prevenção-de-conflitos)). As listas podem ter nome e ser partilhadas; as Definições > Prevenção de conflitos editam-nas.

| Chave | Predefinição | Valores e efeito |
|---|---|---|
| `conflict_prevention.default_list` | `""` | O nome da lista que um repositório usa quando não tem uma própria. Vazio, ou um nome que nenhuma lista tem, significa o `**` incorporado (todas as branches). Relido a cada verificação, por isso uma alteração aplica-se aos repositórios abertos sem reiniciar. |
| `conflict_prevention.lists` | nenhuma | As listas com nome, uma tabela cada: `[conflict_prevention.lists.release]` com `patterns = ["origin/release/**", "!origin/release/old"]`. Um `patterns` vazio permite todas as branches. Editar uma lista aplica-se a todos os repositórios que a seguem. |

A lista que um repositório segue está no seu `repo-state.toml` (abaixo). Precedência: os padrões próprios do repositório, depois a lista com nome que segue, depois `default_list`, depois `**`.

## `[keybindings]`

Reatribui os atalhos da TUI; as Definições > Teclado e rato editam-no.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # predefinição Ctrl+G
nav_prefix = "Alt+n"             # predefinição Ctrl+N
settings   = ["Ctrl+,", "F2"]    # uma lista = alternativas
tab_menu   = []                  # sem tecla

[keybindings.git_prefix]         # a tecla premida depois do prefixo git
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`; um valor substitui as
predefinições dessa ação, remover a linha restaura-as. Acordes: `Ctrl+Shift+N`,
`Alt+n`, `F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). Uma entrada incorreta
nunca bloqueia nada: a TUI ignora-a, mantém a predefinição e di-lo na linha de
estado. Contextos, ids de ações, predefinições e as regras de conflito:
[Atalhos de teclado](keybindings.md).

## Definições por repositório: `.gitpoulpi/repo-state.toml`

Guardado dentro do repositório; a pasta ignora-se a si própria (um `.gitignore`
com `*`), por isso nunca aparece como alteração. Escrito pela TUI e pela CLI;
raramente o edita.

| Chave | O que é | Definida com |
|---|---|---|
| `[preferences] encoding` | Sobrepõe-se a `git.default_encoding` aqui | Definições > Git e codificação |
| `[preferences] lfs_enabled` | Sobrepõe-se a `git.lfs_enabled` aqui | Definições > Git e codificação |
| `[preferences] profile` | O id de `[[profiles]]` que assina os commits aqui; em falta significa `general.active_profile` | a lista de perfis da barra de ferramentas, **Repositório: alterar perfil...** |
| `[preferences] autocrlf`, `eol` | *Ainda não usadas* | Definições > Git e codificação |
| `[preferences] push_after_commit` | Faz push da branch para o seu upstream depois de cada commit (desativado quando ausente) | Definições > Definições do repositório > Commit |
| `[preferences] skip_hooks_by_default` | O Ignorar hooks do formulário de commit começa assinalado (desativado quando ausente) | Definições > Definições do repositório > Commit |
| `[preferences] squash_merge` | O merge de uma branch local faz squash: coloca em stage, sem commit de merge (desativado quando ausente) | Definições > Definições do repositório > Opções de merge |
| `[preferences] template_apply`, `template_strip_comments` | Preenche um formulário de commit vazio com o modelo; remove as linhas de comentário `#` ao fazer o commit (ambos desativados quando ausentes) | Definições > Definições do repositório > Modelo de mensagem |
| `[preferences] conflict_detection` | Deteção proativa de conflitos para este repositório; ausente segue `general.conflict_detection` | Definições > Definições do repositório > Prevenção de conflitos |
| `[preferences] conflict_base_branches` | Os padrões das Branches base próprios do repositório, um por linha (ainda usados como **Lista própria**) | Definições > Definições do repositório > Prevenção de conflitos |
| `[preferences] conflict_base_source` | De onde vêm as Branches base: `default` (a lista predefinida global), `list` (`conflict_base_list`) ou `own` (`conflict_base_branches`). Ausente: os padrões próprios quando existem, senão a predefinida | Definições > Definições do repositório > Prevenção de conflitos |
| `[preferences] conflict_base_list` | A `[conflict_prevention.lists.<nome>]` que este repositório segue quando a origem é `list`; uma lista eliminada recua para a predefinida | Definições > Definições do repositório > Prevenção de conflitos |
| `commit-template.txt` (um ficheiro ao lado de `repo-state.toml`) | O modelo de commit; o `commit.template` do git aponta para ele | Definições > Definições do repositório > Modelo de mensagem |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: as instruções de IA deste repositório | Definições > Definições do repositório > IA |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` para uma ação aqui | Definições > Definições do repositório > IA |
| `default_remote` | O remoto que o Push/Pull/Fetch de um clique usa | menu de remotos, `gitpoulpi remote default <name>` |
| `hosting_integration` | O perfil de `[[integrations]]` fixado aqui | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | O projeto do fornecedor que as listas de PRs / issues leem, como `{ integration, project, filter }`. `filter` é opcional: uma cláusula extra nativa do fornecedor (um fragmento JQL do Jira, uma condição WIQL do Azure, nomes de listas do Trello) juntada com AND à consulta da lista; o GitHub e o GitLab ignoram-no. | secções da barra lateral, `gitpoulpi hosting link` |
| `ref_visibility` | Refs ocultas / isoladas (solo) | barra lateral Refs, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Remotos ocultos do grafo por inteiro (todas as branches, também as obtidas mais tarde) | o olho e o menu do remoto, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Secções da barra lateral e pastas de branches e tags recolhidas. `[folds]` guarda também `status_folders` (pastas recolhidas das árvores de ficheiros Unstaged / Staged) e `commit_folders` (pastas recolhidas da árvore de ficheiros alterados de um commit, igual para todos os commits), cada uma limitada a 500 caminhos, descartando os mais antigos primeiro | a barra lateral, as árvores de ficheiros |
| `pending_auto_stash` | Um stash que o GitPoulpi fez e ainda não repôs | escrito pelo daemon; veja [Auto-stash](log-and-history.md#auto-stash) |

## Outras variáveis de ambiente

| Variável | Efeito |
|---|---|
| `GITPOULPI_CONFIG_DIR` | A pasta de configuração (veja acima). |
| `GITPOULPI_TOKEN_<ID>` | Um token de alojamento para o perfil `<ID>`, por cima do guardado (CI, uso pontual). |
| `GITPOULPI_GIT` | O executável `git` a executar (predefinição: `git` no `PATH`; no Windows o verdadeiro `mingw64\bin\git.exe` por trás do atalho `cmd\git.exe` do Git for Windows, o que poupa um processo por chamada). |
| `GITPOULPI_LANG=fr` | O idioma da interface, por cima de `ui.language` (qualquer valor de `ui.language`; `qps` é um pseudo-locale para tradutores). Um valor desconhecido ou vazio é ignorado. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Sem imagens de todo, seja o que for que a configuração diga. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Sugestões de resolução das imagens (tamanho da célula em píxeis; sobreamostragem, predefinição 2, `1` poupa CPU). |
| `GITPOULPI_NO_BROWSER=1` | Nunca lançar um navegador web (o URL continua a ser mostrado). |
| `GITPOULPI_TIMING=1` | Escrever os tempos dos pedidos em `gitpoulpi-timing.log` na pasta temporária (relatórios de desempenho). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | Não esperar por Enter depois de a TUI sair com um erro. |
