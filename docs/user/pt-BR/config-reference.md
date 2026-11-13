[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · [Français](../fr/config-reference.md) · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Referência de configuração

Cada chave do `config.toml`, seu valor padrão e o que ela faz. Você raramente
precisa editar o arquivo à mão: a [tela de Configurações](settings.md)
(`Ctrl+,`) o grava conforme você muda as coisas, e `gitpoulpi config set`
também.

## Onde fica e como um valor é escolhido

| | |
|---|---|
| Arquivo | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` o imprime. |
| Outra pasta | `GITPOULPI_CONFIG_DIR=<dir>` move a pasta de configuração inteira (o arquivo, `sessions/`, os arquivos de porta do daemon): uma instalação portátil, um teste, ou um agente mantido longe das suas próprias configurações. |
| Precedência | padrão embutido, depois `config.toml`, depois a variável `GITPOULPI_*`, depois uma opção de linha de comando. Vence a mais alta. Um valor errado (uma escolha com erro de digitação no arquivo, uma variável inválida) é um erro, nunca ignorado em silêncio. |
| Variáveis | Coloque a chave em maiúsculas e troque os pontos por sublinhados: `general.auto_prune` é `GITPOULPI_GENERAL_AUTO_PRUNE`. Só para chaves que o GitPoulpi realmente lê (não as marcadas como *não usadas* abaixo, nem listas como `ui.log_columns_hidden`). |
| Opções | `--backend git2\|system` substitui `general.default_backend` para um daemon que esse comando inicia. |
| Tokens e chaves | Nunca neste arquivo. Os tokens de hospedagem e as chaves de API de IA vão para o auxiliar de credenciais do seu sistema (Git Credential Manager, libsecret / `pass`, Keychain); o arquivo guarda só uma referência (`token_ref`). Sem um helper, guardar um token falha com uma mensagem e o token dura só a sessão. |

```
gitpoulpi config path                          # o arquivo, e se ele já existe
gitpoulpi config get general.auto_stash        # o valor, a camada de onde veio (default/file/env/flag), e se algo o lê
gitpoulpi --json config get ui                 # uma seção inteira
gitpoulpi config set general.auto_stash always # validado contra o esquema, depois gravado
```

`config set` mantém o arquivo enxuto (só as chaves que alguém definiu) e mantém
as chaves sobre as quais não foi perguntado, mas descarta os comentários. Ele
recusa listas e tabelas inteiras, exceto `keybindings.*` (vários acordes
separados por espaços; vazio remove o atalho). Edite `[[profiles]]`,
`[[integrations]]` e `[[ai.profiles]]` nas Configurações ou à mão.

Nas tabelas, **Reiniciar** significa que o valor é lido na inicialização; todo o
resto vale na hora (ou na próxima atualização, fetch ou fechamento das
Configurações).

## `[general]`

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, recorrendo ao programa `git` para staging, merge, rebase, stash e rede) ou `system` (somente o `git` instalado). Um daemon em execução mantém o seu backend: **reinicie-o** (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Quando alterações locais bloqueiam um checkout, pull, merge ou rebase: `ask` (um diálogo na TUI; a CLI e o MCP falham e citam `--autostash`), `always` (stash, executa, pop) ou `never` (recusa). Veja [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | O auto-stash também leva os arquivos untracked. |
| `general.initial_commits_in_graph` | `5000` | Commits que o Log carrega primeiro (no mínimo 500). Mais custa memória e tempo de desenho em repositórios enormes. |
| `general.lazy_load_commits_in_graph` | `true` | Carrega commits mais antigos conforme você rola perto do fim. Desativado: o Log para na contagem inicial. |
| `general.commits_per_lazy_page` | `1000` | Commits por página em segundo plano (no mínimo 50). |
| `general.max_commits_in_graph` | `100000` | Onde o carregamento sob demanda para (cerca de 300 bytes por commit); `0` = sem limite. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch All em segundo plano a cada N minutos, de 0 a 60; `0` = desativado. **Reiniciar.** |
| `general.auto_fetch_interval_unit` | `"auto"` | A unidade em que as Configurações mostram o intervalo: `minutes`, `hours` ou `auto`. Só de exibição: o intervalo é sempre `auto_fetch_interval_minutes` minutos. `auto` (e uma configuração sem a chave) escolhe a maior unidade que divide os minutos exatamente (60 aparece como “1 hora”, 45 como “45 minutos”); uma unidade que não serve é ignorada. As Configurações gravam as duas chaves; um GitPoulpi mais antigo ignora a unidade e continua lendo os minutos. |
| `general.fetch_on_tab_open` | `true` | Fetch All quando uma aba abre (pulado se o GitPoulpi fez fetch desse repositório há menos de 60 s). |
| `general.auto_prune` | `true` | Os fetches também fazem prune das branches de rastreamento remoto que sumiram. |
| `general.auto_prune_tags` | `false` | Um fetch também exclui as tags locais que o remoto excluiu (só as que estavam nele; uma tag que você nunca enviou é mantida). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Atualização das listas de pull requests / issues e do chip da barra de ferramentas, de 0 a 120; `0` = só sob demanda e por eventos. |
| `general.hosting_refresh_unit` | `"auto"` | A unidade em que as Configurações mostram a atualização: `minutes`, `hours` ou `auto`, lida como `general.auto_fetch_interval_unit`. Só de exibição. |
| `general.keep_submodules_up_to_date` | `false` | Depois de um checkout ou pull, `git submodule update --init --recursive` quando um submódulo está fora de sincronia. Veja [Submódulos](submodules.md). |
| `general.default_branch_name` | `"main"` | Primeira branch de um repositório criado com Init; vazio deixa o git decidir. Não renomeia nada. |
| `general.delete_orig_files_after_merge` | `false` | Depois de um merge que conclui, exclui os arquivos de backup `*.orig`. |
| `general.remember_tabs` | `true` | Reabre as abas da última sessão. Desativado, uma inicialização dentro de um repositório abre só esse repositório, e uma inicialização fora de um repositório abre só o Início. **Reiniciar.** |
| `general.radar_enabled` | `true` | O [Radar](radar.md): a aba dele, os comandos da paleta, o `Alt+R` e a busca em segundo plano. Desativado, tudo isso some e nada é buscado para ele. Aplicado na hora; não aparece em uma build sem o recurso `radar`. |
| `general.startup_tab_in_repo` | `"current-repo"` | A aba em que uma inicialização abre quando a pasta de onde parte está dentro de um repositório: `"current-repo"` (a aba desse repositório, aberta se preciso), `"last"` (a aba ativa no último encerramento), `"home"` ou `"radar"`. Um repositório indicado com `--repo` abre sempre a própria aba. `"last"` vira Início com `remember_tabs` desativado ou se não houver nada lembrado; `"radar"` vira Início se o Radar estiver desativado ou fora do build. Lido uma vez na inicialização; exige reinício. |
| `general.startup_tab_outside_repo` | `"home"` | A aba em que uma inicialização abre quando a pasta de onde parte não é um repositório (uma pasta de usuário, por exemplo): `"home"`, `"last"` ou `"radar"`. `"last"` vira Início com `remember_tabs` desativado ou se não houver nada lembrado; `"radar"` vira Início se o Radar estiver desativado ou fora do build. Lido uma vez na inicialização; exige reinício. |
| `general.startup_tab_launcher` | `"last"` | A aba em que uma inicialização abre quando o GitPoulpi é iniciado pelo atalho da área de trabalho ou do menu Iniciar que ele criou (passam `--launcher`), seja qual for a pasta: `"home"`, `"last"` ou `"radar"`. Tem prioridade sobre as duas configurações acima. `"last"` vira Início com `remember_tabs` desativado ou se não houver nada lembrado; `"radar"` vira Início se o Radar estiver desativado ou fora do build. Lido uma vez na inicialização; exige reinício. |
| `general.group_worktree_tabs` | `true` | Mantém as abas dos worktrees vinculados de um repositório logo depois da aba desse repositório: quando um worktree abre, quando a aba de um worktree é arrastada para longe (ela volta para junto do seu grupo; a aba do repositório é a cabeça do grupo e os seus worktrees a seguem) e quando a opção é ativada. Desativado, as abas ficam onde você as colocou. |
| `general.undo_depth` | `100` | Quantas operações o Desfazer pode voltar; `0` = ilimitado. |
| `general.conflict_detection` | `true` | Detecção proativa de conflitos: um selo no Log quando um merge da branch atual na branch de onde ela partiu daria conflito, calculado em segundo plano (um merge em memória: nada muda nos seus arquivos, no índice nem nas refs). Um repositório pode desativá-la para si (`[preferences] conflict_detection`). As branches base com que ela compara têm listas próprias: [`[conflict_prevention]`](#conflict_prevention-listas-de-branches-base). Veja [Resolver conflitos de merge](merge-conflicts.md#prevenção-de-conflitos). |
| `general.dev_mode` | ausente (automático) | Para quem compila o GitPoulpi a partir de um checkout; nunca aparece nas Configurações. Ausente: ligado quando o executável em uso fica em `<checkout>/target/<profile>/` (`Cargo.toml` e `crates/gitpoulpi-tui` ao lado de `target`). `true` / `false` força o valor. O modo de desenvolvimento só libera `ui.launcher_build`. |
| `general.journal_outside` | `true` | Registra o que um shell ou um agente de IA faz em branches, tags, stashes e HEAD como entradas do histórico de desfazer, marcadas como externas, para que `Ctrl+Z` possa reverter. Edições em arquivos nunca são capturadas. Exige `undo_enabled`. Veja [Desfazer e refazer](undo-redo.md#alterações-feitas-fora-do-gitpoulpi). |
| `general.undo_enabled` | `true` | `false`: nenhum controle de desfazer (o mais rápido), o histórico de desfazer é apagado, os botões Desfazer/Refazer desaparecem. Veja [Desfazer e refazer](undo-redo.md). |
| `general.active_profile` | `""` | Id da identidade de `[[profiles]]` com a qual os commits são feitos. Vazio, ou um perfil com nome ou e-mail em branco: `user.name` / `user.email` do git. |
| `general.external_editor` | `""` | Comando para *Abrir no editor externo* de um arquivo, por exemplo `code -r {file}`. Vazio: o VS Code quando instalado, senão o padrão do sistema. |
| `general.external_diff_tool` | `""` | Comando para *Abrir na ferramenta de diff externa*: `{local}` (versão commitada / HEAD), `{remote}` (versão da árvore de trabalho / índice). Vazio: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Comando para *Abrir na ferramenta de merge* em um cartão de conflito (um arquivo binário, LFS ou grande demais): `{base}`, `{local}` (A, atual), `{remote}` (B, recebido) e `{merged}` (o arquivo em que a ferramenta grava o resultado; o GitPoulpi faz stage dele quando a ferramenta termina com sucesso). Vazio: `git mergetool --no-prompt`. |
| `general.file_explorer` | por SO | Comando para *Mostrar na pasta*: `{path}` (o arquivo), `{dir}` (a pasta dele). Padrões: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

Os três modelos de comando são divididos primeiro em argumentos (`"..."` agrupa
uma palavra), e depois os marcadores são preenchidos, de modo que caminhos com
espaços continuam sendo um único argumento. Nenhum shell é envolvido.

```toml
[[profiles]]                # commit identities (Settings > Profiles)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log e grafo**

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (uma imagem onde o terminal pode desenhar uma, caracteres nos demais), `image`, `text`. **Reiniciar.** |
| `ui.image_protocol` | `"auto"` | Como as imagens (grafo, avatares) chegam ao terminal: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = protocolo Kitty no kitty, no Ghostty, no iTerm2 3.7.7 ou mais recente e em um herdr com `kitty_graphics = true`; o protocolo do iTerm2 no WezTerm e no Rio (transparência misturada), no iTerm2 mais antigo e no mintty (fundo opaco); sixel (a coluna do grafo e os avatares) no Windows Terminal, foot, Tabby, Konsole, mlterm, no tmux quando o terminal por trás dele aceita sixel e em qualquer terminal cujas respostas na inicialização citem sixel (um xterm iniciado com `-ti vt340`, um terminal do VS Code com imagens ativadas); caracteres no zellij, no screen e em terminais desconhecidos. No tmux, as imagens do kitty exigem `kitty` aqui mais `allow-passthrough on` ([Multiplexadores](multiplexers.md)). Onde o pseudoconsole do terminal descarta ou desloca as imagens (WezTerm 20240203, Rio e mintty no Windows), use `ui.graph_renderer = "text"` ([Escolhendo um terminal](terminals.md#windows)). **Reiniciar.** |
| `ui.theme` | `"system"` | O tema de cores: um slug embutido (`gitpoulpi theme list` lista todos), o slug de um arquivo em `<config>/themes/`, ou `system` (segue o fundo claro ou escuro do terminal, detectado na inicialização; o valor antigo `default` significa o mesmo). `config set` rejeita nomes desconhecidos. Veja [Temas](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Paleta das faixas: `theme` (segue o tema) ou qualquer nome de paleta de faixas: `gitpoulpi`, `light`, `material`, `serie`, e as paletas dos outros temas. |
| `ui.graph_lane_background` | `"off"` | `off` ou `row_tint` (uma faixa tênue da cor da faixa atrás do grafo, mais uma barra na borda dele). **App: alternar tom das linhas do grafo** (`Ctrl+N Shift+T`) a alterna; **App: mudar tema de cores do grafo...** (`Ctrl+N Shift+Y`) escolhe `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (só com `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` e `right` desenham uma coluna; uma janela com menos de 80 colunas volta para `bottom`. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never`: se um diálogo que você moveu ou redimensionou (arraste a barra de título ou as bordas, ou `Ctrl+N m`) mantém o lugar. Uma só configuração para todos os diálogos (cada um mantém a própria posição e o próprio tamanho): até o GitPoulpi fechar (padrão), em disco para a próxima abertura (todos os diálogos no único `dialog_geometry.txt` na pasta de dados) ou de forma alguma (todo diálogo abre na posição e no tamanho padrão). |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: onde os botões ficam ao longo da barra (esquerda, meio, direita de uma linha; topo, meio, base de uma coluna). |
| `ui.toolbar_pr_pinned` | `true` | O botão de pull request fica fixado na extremidade da barra de ferramentas; `false` o deixa fluir com os outros botões. |
| `ui.statusbar_position` | `"bottom"` | `top` (acima das abas de repositório) ou `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (a última linha da janela, abaixo da linha de status) ou `left` (uma coluna na borda esquerda; com menos de 80 colunas fica no topo). |
| `ui.graph_show_stashes` | `true` | Uma linha por stash no Log. |
| `ui.ref_connector` | `true` | Uma linha fina da pílula de uma ref até o commit dela. |
| `ui.column_resize` | `"line"` | Como se pega o limite de uma coluna no Log, no Radar e na coluna de commits das visões de diff, arquivo e blame. `line`: cada limite é um espaço de uma célula com uma pequena marca no cabeçalho; enquanto o ponteiro está sobre o limite, no cabeçalho ou em qualquer ponto da tabela, uma linha fina de destaque é desenhada em toda a altura e um arrasto a partir de qualquer ponto dela redimensiona. `header`: sem espaço nem marca; o primeiro caractere da célula do cabeçalho depois do limite se acende e um arrasto a partir dele redimensiona (só no cabeçalho). As teclas de dimensionamento e as linhas da paleta são as mesmas nos dois modos. |
| `ui.ref_tree` | `true` | Agrupa as branches e as tags em `/` em pastas na barra lateral Refs (`y` alterna). |
| `ui.graph_ghost_ref` | `true` | A coluna Ref do Log mostra uma pílula esmaecida com o nome da branch (senão a branch remota) a que pertence um commit sob o mouse, quando o commit não tem ref própria e a coluna tem espaço; nunca uma tag. Uma branch excluída é deduzida do assunto do commit de merge que a mesclou e desenhada riscada. Passar o mouse na pílula mostra o tipo. |
| `ui.reveal_on_ref_click` | `true` | Um único clique em uma ref (branch, tag, branch remota, stash) no painel Refs rola o Log até o commit dela e faz a linha piscar; a seleção de commits não se move. |
| `ui.log_wip_counts_in_message` | `true` | Contagens de alterações do WIP na mensagem do WIP enquanto a coluna Alterações está oculta. |
| `ui.log_columns_layout` | `"fit"` | Como o Log organiza suas colunas. `fit`: cada coluna visível cabe exatamente no painel e só `message` se ajusta; quando o painel é estreito, colunas inteiras cedem. `scroll`: as colunas mantêm suas larguras (`message` também), nada cede, e a tabela rola para os lados quando é mais larga que o painel. |
| `ui.log_column_widths_scroll` | vazio | As larguras das colunas na disposição `scroll`, nome -> células (`message` é permitida, ao contrário de `ui.log_column_widths`, que continua sendo a da disposição `fit` e nunca contém `message`). Gravada quando você arrasta, dimensiona ou ajusta uma coluna enquanto a disposição é `scroll`. |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | vazio | As colunas do Log (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`), gravadas quando você oculta, arrasta ou redimensiona uma coluna. `message` não pode ser ocultada. |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | vazio | As partes da [coluna de commits](interface-tour.md#a-coluna-de-commits) ao lado de um diff, por conta própria e à parte das do Log: as partes exibidas pelo nome (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; vazio é faixas, mensagem, autor e data), a ordem delas da esquerda para a direita pelos mesmos nomes (vazio é a ordem que acabou de ser listada; um nome omitido vem depois, nessa ordem) e as larguras arrastadas, em células por parte (uma parte omitida é automática), gravadas quando você escolhe, move ou dimensiona as partes. `message` não pode ser ocultada. Um arquivo de sessão gravado antes da existência dessas chaves é lido uma vez para elas. |
| `ui.sidebar_sections_hidden` | vazio | As seções do painel esquerdo que você ocultou (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), gravadas pelo menu do botão direito do painel ou pela paleta. Pelo menos uma seção existente sempre continua visível. Este é o padrão: cada repositório pode ocultar suas próprias seções (guardadas no arquivo de estado dele) e esta chave vale para um repositório sem escolha própria; *Usar em todos os repositórios* a grava. |
| `ui.sidebar_sections_scope` | `"repository"` | De quem é a escolha das seções do painel esquerdo: `repository` (cada repositório guarda a sua, e `ui.sidebar_sections_hidden` é o padrão) ou `global` (todos os repositórios mostram `ui.sidebar_sections_hidden`, e uma alteração é gravada ali). Alternar nunca perde nenhum dos dois conjuntos: as escolhas por repositório continuam nos arquivos de estado enquanto `global` está ativo. Nas Configurações: *Seções do painel esquerdo*. |
| `ui.language` | `"auto"` | O idioma da interface: `auto` (o locale do sistema, senão inglês), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Ao vivo.** `GITPOULPI_LANG` o substitui. Um idioma que não está nesta compilação mostra inglês. **App: mudar idioma...** (`Ctrl+N g`) também o define, com pré-visualização ao vivo. A saída da CLI e do MCP continua em inglês. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... continuam em inglês em todos os idiomas. `true`: o vocabulário git do idioma escolhido (em francês “remiser”, “étiquette”, “valider”...; em português do Brasil “enviar”, “puxar”, “mesclar”, “etiqueta”...). **Ao vivo.** |
| `ui.git_terms_english` | `false` | Só ja, ko e zh-CN: mostra as palavras git em inglês (stage, unstage, staged, unstaged...) em vez da transcrição. Sem efeito enquanto `ui.translate_git_terms` for `true`. **Ao vivo.** |
| `ui.mouse_enabled` | `true` | `false` deixa o mouse (e a seleção de texto) para o terminal. **Reiniciar.** |
| `ui.mouse_scroll_lines` | `3` | Linhas do Log por clique da roda, de 1 a 20. **Reiniciar.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` pede ao terminal o protocolo de teclado do kitty (para que `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` cheguem; só em Unix, o tmux nunca responde: veja [Multiplexadores](multiplexers.md#tmux)); `off` nunca pede. **Reiniciar.** |
| `ui.logo_color` | `""` | Combinação de cores do logo na janela Sobre, um slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); vazio ou desconhecido significa o padrão da compilação (`lavender`). Gravada quando você clica no logo ou escolhe uma cor em Sobre. |

**Avatares** (desativados por padrão; ativá-los envia um hash SHA-256 dos e-mails
dos autores ao Gravatar, e consulta a sua conta conectada do GitHub/GitLab)

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (no círculo do commit do grafo de imagem), `author` (na coluna Autor), `off`. O antigo `ui.show_avatars` (`true`) ainda é lido como `author` enquanto `ui.avatars` estiver ausente. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Ordem de consulta: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. Um host GitLab só é consultado por meio de um perfil conectado. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (gerado localmente) ou `initials`. Em um terminal sem protocolo de imagem, o selo de iniciais é sempre usado. |

**Exibição do diff e do editor**

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` ou `split` (`m` em um diff). |
| `ui.diff_hunks_only` | `true` | `false` mostra o arquivo inteiro com as alterações no lugar (`H`). |
| `ui.diff_show_whitespace` | `true` | Mostra alterações só de espaço em branco (`s`). |
| `ui.diff_word_wrap` | `false` | Quebra linhas longas do diff (`w`; Configurações > Diff e editor). |
| `ui.minimap` | `false` | Um minimapa ao lado da barra de rolagem do diff e do editor (`M`). |
| `ui.mermaid_detail` | `refs` | Nível de detalhe do Copiar como gitGraph do Mermaid do Log: `structure`, `refs`, `subjects`, `full` (lembrado a partir do popup; o padrão de `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Direção desse grafo: `lr` (da esquerda para a direita) ou `tb` (de cima para baixo). |
| `ui.mermaid_fence` | `false` | Envolve o script do Mermaid copiado em um bloco de código `mermaid`. |
| `ui.mermaid_subject_width` | `40` | Os assuntos dos commits são cortados neste número de caracteres nos ids do Mermaid (`subjects`, `full`). |
| `ui.text_cursor` | `"terminal"` | Onde o ponto de inserção de um campo de texto em foco aparece: `terminal` coloca ali o cursor do próprio terminal (uma barra piscante, então as configurações de cursor do terminal e a janela de candidatos de um método de entrada o acompanham; oculto quando nenhum campo tem o teclado e restaurado ao seu estilo de cursor ao sair), `drawn` desenha o glifo de cursor do próprio GitPoulpi e mantém o cursor do terminal oculto. Ao vivo. |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` ou `16`: com quantas cores desenhar. `auto` é truecolor, exceto no Apple Terminal (256), no console do Linux (16) e em terminais `*-256color` sem `COLORTERM` (256). Ao vivo. Veja [Temas](themes.md). |
| `ui.input_color` | `"theme"` | Fundo dos campos de texto: `theme` (o do próprio tema), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | A cor `custom`, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | O compositor de commits com IA abre com os arquivos de cada proposta `expanded` ou `collapsed`. |
| `ui.blame_side` | `"right"` | Qual lado da visão Blame tem a coluna de anotações (avatar, autor, idade, resumo): `right` (padrão) ou `left`. |
| `ui.blame_width` | `46` | Largura da coluna de anotações do Blame em células (24 a 90). Arraste sua borda ou pressione `<` / `>` na visão; a nova largura é salva aqui. |
| `ui.pr_terminology` | `"auto"` | `auto` (pull request no GitHub, merge request no GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Oculta o chip da barra de ferramentas da pull ou merge request mais recente da branch em checkout quando ela é fechada ou integrada; o chip esmaecido "+ PR" (iniciar uma nova) aparece no lugar, como se a branch não tivesse nenhuma. `false`: o chip sempre mostra a solicitação mais recente, seja qual for o estado. |

Os alternadores do diff que você muda dentro de um diff são salvos aqui na hora
e usados por todo diff aberto depois.

**Datas**: `ui.date_format` (`"YYYY-MM-DD"`, a coluna de data do Blame / Histórico do arquivo),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, a coluna Data do Log) e
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, os detalhes do commit, a dica
da Data e *Copiar > Data*). Tokens: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (dia da semana, completo / abreviado) `HH` `mm` `ss`; qualquer outra
coisa é copiada como está. Os nomes de mês e de dia da semana seguem
`ui.language`, e um mês escrito depois de um número de dia fica no genitivo
onde o idioma o tem (russo, ucraniano, polonês, tcheco). Uma sequência
`D MMMM YYYY` é escrita com o ano primeiro em japonês, chinês e coreano
(`2024年3月5日`). `ui.date_locale` (`"system"`, o padrão, ou `"english"`) mantém
os nomes em inglês seja qual for o idioma.

**Agentes, terminais e a pasta da aba**

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | A janela aceita `ui show` / `ui propose` de agentes (Configurações > Agentes e MCP > *Permitir que agentes de IA controlem a visualização*). Desativado, nenhum agente a alcança. Veja [MCP e agentes](mcp-and-agents.md#mostrar-coisas-ao-usuário). |
| `ui.launch` | `"off"` | Se um agente pode abrir uma janela do GitPoulpi quando nenhuma está aberta: `off`, uma predefinição (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) ou `custom`. |
| `ui.launch_command` | `""` | O modelo `custom`: `{repo}`, `{session}`, `{gitpoulpi}`. Dividido em argumentos, sem shell. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (a fileira de abas oferece *Sair e cd aqui*) ou `auto` (toda saída muda a pasta do shell, e uma nova aba executa a entrada *Abrir em* padrão). Veja [Seguindo a pasta da aba ativa](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Legado, veja abaixo e [Abrir em](open-in.md). Como o antigo *Terminal aqui* abria um terminal: `auto` (a predefinição de `ui.launch`, senão `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), uma predefinição, ou `custom`. |
| `ui.terminal_launch_command` | `""` | O modelo `custom`; `{repo}` é a pasta. |
| `ui.open_in` | *(ausente)* | A lista [Abrir em](open-in.md), um array de tabelas `[[ui.open_in]]` com `label` e `command`. Ausente significa “ainda não configurada”: a lista é calculada a partir das chaves legadas `ui.terminal_launch*` (um valor diferente de `auto` vira a primeira entrada), do gerenciador de arquivos do SO, dos multiplexadores e terminais detectados no ambiente (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) e dos editores encontrados no `PATH`. O primeiro salvamento pelas Configurações grava a lista. `open_in = []` é uma lista vazia. Não pode ser definida com `config set` (edite o arquivo). |
| `ui.open_in_default` | `""` | Rótulo da entrada que “Abrir no padrão” executa; vazio ou desconhecido significa a primeira entrada. |
| `ui.launch_terminal` | `"auto"` | O terminal em que os atalhos da área de trabalho (Configurações > Lançador) e a reabertura no Windows iniciam o GitPoulpi. `auto` escolhe entre os instalados (Windows: Windows Terminal, senão WezTerm, Alacritty, o console simples; Linux: `xdg-terminal-exec`, senão o primeiro terminal encontrado; macOS: iTerm2, Ghostty, WezTerm ou kitty se instalados, senão Terminal), `default` é o do sistema operacional, ou um id de terminal como `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Só no modo de desenvolvimento (veja `general.dev_mode`; só então aparece em Configurações > Lançador, e é ignorado nos outros casos). `"current"`: os atalhos e o alias de comando iniciam a compilação que os criou. `"newest"`: eles iniciam a compilação mais recente entre `target/dist` e `target/release` do checkout, decidida a cada clique, e iniciar qualquer uma das duas nunca os regrava. |
| `ui.command_alias` | `""` | O nome do comando que inicia o GitPoulpi em um terminal, por exemplo `gp` (Configurações > Lançador, `gitpoulpi alias create`). Vazio: nenhum. Só letras, dígitos, `-`, `_` e `.`. É só o nome que o botão **Criar: alias de comando** usa; nada existe até você o criar. |
| `ui.command_alias_add` | `""` | O nome do segundo comando, por exemplo `gpa`, que executa `gitpoulpi --add`: ele abre a pasta atual em uma nova aba do GitPoulpi que já está aberto (Configurações > Lançador, `gitpoulpi alias create gpa --add`). Vazio: nenhum. Mesmas regras de nome; criado junto com `ui.command_alias` pelo botão **Criar**. |
| `ui.relaunch_in_terminal` | `true` | Somente Windows: quando o exe é iniciado com clique duplo em um console próprio, o GitPoulpi se reabre em `ui.launch_terminal` e encerra. Um terminal que executa o exe diretamente (`wezterm-gui start -- gitpoulpi.exe`, `wt.exe gitpoulpi.exe`, Rio, Tabby, Alacritty `-e`) também é um console próprio, mas não uma janela de console clássica: ele nunca é relançado. Desativado: ele fica nesse console. |
| `ui.reuse_window` | `true` | Um clique em um atalho do GitPoulpi (`gitpoulpi launch`) mostra a janela de um GitPoulpi que já está aberto, em vez de iniciar outro. Desativado: cada clique inicia uma nova janela. |

Os **comandos de `ui.open_in`** são divididos primeiro em argumentos (espaço em
branco, `'...'`, `"..."`), e depois as variáveis são substituídas dentro de cada
argumento, de modo que uma pasta com espaços continua sendo um único argumento;
nenhum shell é envolvido, a menos que o comando inicie um (`sh -c '... "$GITPOULPI_CWD"'`).
Variáveis: `${cwd}` (a pasta da aba), `${repo}` (a raiz do repositório), `${name}`
(o nome-base da pasta), `${branch}` (vazio quando detached), `${web_url}` (a
página web do remoto `origin` quando é um projeto do GitHub ou GitLab, senão
vazio; uma entrada que a use se recusa a executar quando vazio), `${env:NAME}`
(vazio quando não definida). O antigo `{repo}` ainda significa `${cwd}`. Qualquer
outro `${...}` é um erro que o cita. Exemplo:

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

Legado: `ui.terminal_launch` e `ui.terminal_launch_command` ainda são lidos (eles
semeiam a lista enquanto `ui.open_in` estiver ausente: `custom` vira “Terminal
(custom)” com `{repo}` reescrito para `${cwd}`, o id de uma predefinição vira
essa predefinição), mas foram substituídos por `ui.open_in`.

### Imagens

| Chave | Padrão | Significado |
|---|---|---|
| `ui.image_view` | `true` | Mostra um arquivo binário que é uma imagem (PNG, JPEG, GIF, WebP, BMP, ...) como imagem na visualização de Diff e na Visualização de arquivo. Desativado: o aviso binário simples. |
| `ui.image_view_max_mib` | `32` | Imagens maiores que isto (MiB) não são decodificadas; uma ficha de informações aparece no lugar. |
| `ui.image_view_max_unit` | `"auto"` | A unidade em que as Configurações mostram o limite: `mib`, `gib` (1024 MiB) ou `auto`. Só de exibição: o limite é sempre `image_view_max_mib` MiB, de 1 a 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | Em um terminal sem protocolo de imagem: `halfblock` (caracteres de meio bloco coloridos) ou `card` (só a ficha de informações). |
| `ui.image_view_pixelated_from` | `"2x"` | Zoom a partir do qual a ampliação mostra pixels quadrados em vez de suavização: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | Os painéis antes e depois compartilham um zoom e uma posição de rolagem. |
| `ui.image_editors` | *(ausente)* | Os editores de imagem de “Editar em”, um array de tabelas `[[ui.image_editors]]` com `label` e `command`. Ausente significa “ainda não configurada”: a lista são os editores detectados nesta máquina (programas no `PATH`, instalações Flatpak, pastas de instalação conhecidas no Windows, `/Applications` no macOS), depois o aplicativo padrão do SO. O primeiro salvamento pelas Configurações grava a lista. `image_editors = []` é uma lista vazia. Não pode ser definida com `config set` (edite o arquivo). |
| `ui.image_editor_default` | `""` | Rótulo do editor que “Editar no editor padrão” executa; vazio ou desconhecido significa a primeira entrada. |

Os **comandos de `ui.image_editors`** são divididos primeiro em argumentos
(espaço em branco, `'...'`, `"..."`), e depois as variáveis são substituídas
dentro de cada argumento, de modo que um caminho com espaços continua sendo um
único argumento; nenhum shell é envolvido. Variáveis: `${file}` (o caminho
absoluto da imagem), `${dir}` (a pasta dela), `${name}` (o nome do arquivo),
`${repo}` (a raiz do repositório, vazia para uma cópia temporária), `${env:NAME}`
(vazio quando não definida). Qualquer outro `${...}` é um erro que o cita. Uma
imagem que não é o arquivo da árvore de trabalho (a versão de um commit, o
índice, um arquivo excluído) é aberta como uma **cópia temporária somente
leitura**, para que um editor não possa tocar no repositório. Editores sem linha
de comando (Affinity Photo da Microsoft Store, algumas instalações do Photoshop)
não estão nas predefinições: escreva uma entrada personalizada. Exemplo:

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

**Ainda não usadas** (aceitas, salvas, mas nada as lê):
`ui.graph_colorization` (removidas das Configurações;
ainda aceitas ao carregar), `ui.date_word_format` (removida das Configurações;
ainda aceita ao carregar). O antigo
`ui.diff_view_mode` é lido uma vez como `diff_layout` / `diff_hunks_only`.

## `[editor]`

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `editor.tab_size` | `4` | Colunas por tabulação, de 1 a 8 (editor e diffs). |
| `editor.eol` | `"auto"` | Fim de linha que o editor integrado grava: `auto` (mantém o do arquivo), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Cores da linguagem no editor. |
| `editor.line_numbers` | `true` | Números de linha no editor e nos diffs. |
| `editor.follow_selection` | `true` | Com o editor aberto, selecionar outro arquivo no painel direito também o abre no editor, depois de perguntar se as alterações não salvas devem ser salvas. Um arquivo que não pode ser editado abre na visualização habitual. Um arquivo de um commit sai do editor para o diff dele. |

## `[merge]`

O editor de conflitos de merge ([Resolvendo conflitos de merge](merge-conflicts.md)). Configurações > Diff e editor o mostra.

| Chave | Padrão | Significado |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | A largura, em colunas, da área do editor a partir da qual A e B são mostrados lado a lado (linhas longas rolam para o lado: Shift+roda, Shift+setas). Mais estreito, ele mostra um lado de cada vez atrás de abas. `40` é o menor valor, `0` significa sempre lado a lado (o aviso abaixo de 40 colunas permanece). O editor oculta os painéis Refs e Status enquanto está aberto, então a área dele costuma ser a janela inteira. |

## `[git]`, `[credentials]`, `[ssh]`

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `git.pull_mode` | `"merge"` | O que o Pull (botão, `Ctrl+G p`, `Git: Pull`) faz: `merge` (fast-forward quando possível), `ff_only`, `rebase`. Sempre passado explicitamente ao git. O antigo `fetch_all` é lido como `merge`. |
| `git.push_behavior` | `"direct"` | `direct` faz push para o remoto padrão; `ask` mostra antes o menu de remotos. |
| `git.merge_fast_forward` | `"never"` | O que um merge que você inicia (menu da branch, arrastar e soltar, paleta, `gitpoulpi merge`) faz quando a branch poderia avançar por fast-forward: `never` sempre cria um commit de merge (`--no-ff`), `allowed` faz fast-forward quando possível (o padrão do git, vale o `merge.ff`), `only` faz fast-forward ou recusa. A escolha do próprio repositório (Configurações > Opções de merge, `merge_fast_forward` no `repo-state.toml` dele) vence; `--ff`, `--no-ff` e `--ff-only` da CLI vencem as duas. Aplicado na hora. |
| `git.default_encoding` | `"utf-8"` | Codificação para texto que **não** é UTF-8 válido (conteúdo de arquivos, blame, diffs, mensagens de commit sem cabeçalho `encoding`): qualquer rótulo WHATWG (`windows-1252`, `shift_jis`, ...). Só exibição; o staging nunca recodifica. Um repositório pode substituí-la. |
| `git.lfs_enabled` | `true` | Mostra e controla o Git LFS. Um repositório pode substituí-la. Veja [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Antes de um pull ou push de um repositório que usa LFS, instala os filtros LFS / hook pre-push dele (`git lfs install --local`, só este repositório) quando ausentes. Veja [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Não usadas*: `core.autocrlf` / `core.eol` de um repositório são configurações do próprio git, editadas em Configurações > Git e codificação (ou `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | Autenticação HTTPS para fetch, pull, push, clone: `credential_helper` (o do git), `prompt` (ignora helpers; o git pergunta em um terminal, então falha no daemon em segundo plano), `none` (só remotos públicos). Valores antigos `encrypted_file` são lidos como `credential_helper`. |
| `ssh.key_type` | `"ed25519"` | Tipo criado por *Gerar par de chaves*: `ed25519`, `rsa` (4096 bits), `ecdsa` (NIST P-256); o arquivo é `gitpoulpi_<type>`. O próprio SSH sempre usa o seu ssh-agent e a sua configuração do ssh. |
| `ssh.private_key_path`, `ssh.public_key_path` | não definidas | Gravadas por *Gerar par de chaves*. Quando `ssh.private_key_path` está definida, fetch, pull, push e clone executam o git com `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`; `ssh.public_key_path` faz o mesmo (o ssh escolhe a chave correspondente do agente) quando o caminho privado está vazio. Um `GIT_SSH_COMMAND`/`GIT_SSH` no ambiente ou um `core.sshCommand` na configuração do git (a fixação por repositório, `gitpoulpi ssh-command set --key`) vence. Lidas antes de cada transferência, então as mudanças valem para a próxima. |

## `[[integrations]]` (hospedagem e issue trackers)

Gravado por Configurações > Integrações ou `gitpoulpi hosting integrations add`.
Todo tipo tem um cliente (só tokens pessoais, sem OAuth).

```toml
[[integrations]]
id = "corp"                         # o que --id recebe; o token é guardado em integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # a raiz da API é derivada dela
token_ref = "integration:corp"      # uma referência, nunca o token
name = "Corp GitLab"                # optional display name
account = "me@example.com"          # optional, see below
enabled = true                      # false: ignorada ao identificar o host
```

Um servidor auto-hospedado pode ser http simples: `url = "http://localhost:3000"` é mantida
como digitada (o esquema e a porta chegam a cada requisição, e um remoto nesse host
corresponde ao perfil). Um token enviado por http simples trafega sem criptografia, então use
http apenas em uma rede local ou confiável.

`kind` também aceita `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` e `gerrit`
(o Gitea e o Forgejo compartilham um cliente; um perfil Forgejo tem como padrão `https://codeberg.org`). Jira,
Trello, Linear, YouTrack e Redmine são apenas issue trackers; o Gerrit é só de revisão (as suas changes são os
pull requests, ele não tem issues). O Bitbucket Cloud não tem issue tracker.

`account` é a identidade não secreta ao lado do token, vazia e omitida para a
maioria dos tipos: o e-mail da conta Atlassian para `jira_cloud` e `bitbucket`
(uma conta Bitbucket vazia significa um token de acesso de workspace ou de
repositório, enviado como Bearer), a chave de API para `trello`, o nome de usuário para `gerrit` (o token é a sua senha
HTTP, enviada como HTTP basic). O próprio token
continua sendo o único segredo. Para `azure_devops`, a `url` é
`https://dev.azure.com/<organization>`; uma URL
`https://<organization>.visualstudio.com` é lida como a mesma organização. Para
`azure_devops_server`, é a raiz da coleção, como
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (a conta conectada) e `ssh` (a escolha de chave do perfil) são gravados
para você. O token vem de `GITPOULPI_TOKEN_<ID>` (id em maiúsculas, caracteres
não alfanuméricos como `_`), senão do token guardado, senão do seu credential
helper para esse host quando só um perfil o usa (apenas perfis GitHub e GitLab:
os outros tipos nunca leem o auxiliar de credenciais). Detalhes: [Integrações de
hospedagem e de rastreadores de issues](hosting-integration.md). A antiga
tabela `[[hosting]]` *não é usada*.

Para `local_markdown` (uma pasta de notas Markdown como fonte de issues) a `url` é a
pasta, relativa a cada repositório ou absoluta, e não há token. A sua tabela
opcional `[integrations.local]` define `statuses` (a ordem das colunas do quadro,
padrão `todo`, `doing`, `done`), `done_status` (padrão `done`),
`reference_prefix` (`LOC` mostra `LOC-12`) e `me` (o que "atribuídas a mim" quer dizer).
Veja [Issues locais e Obsidian](local-issues.md).

### `[integrations.radar]` (listas de toda a conta)

Tabela opcional de um perfil de host de código (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; não o Bitbucket Cloud). Ausente
significa todas as chaves desativadas. Escrita por `gitpoulpi hosting integrations radar`; lida por
`gitpoulpi hosting radar` e pelo Radar, e editada em Configurações > Integrações (o bloco Radar de um perfil conectado).

```toml
[integrations.radar]
review_requests = true    # solicitações aguardando a minha revisão, todos os repositórios
open_requests = false     # todas as solicitações abertas dos meus namespaces
include_drafts = false    # manter solicitações em rascunho nestas listas
assigned_issues = true    # issues abertas atribuídas a mim, todos os repositórios
namespaces = ["acme", "ptm/produits"]  # uma entrada por linha nas Configurações; vazio = sem limite
namespaces_regex = false  # true: cada entrada é uma expressão regular sobre o caminho completo do repositório
review_mode = "ask"       # ask | clone | api
```

| Chave | Padrão | Significado |
|---|---|---|
| `review_requests` | `false` | Solicitações abertas que pedem a sua revisão, o repositório estando clonado ou não |
| `open_requests` | `false` | Todas as solicitações abertas do seu próprio namespace e de cada org ou grupo de que você é membro (ou de `namespaces`), nunca "tudo o que o token enxerga" |
| `include_drafts` | `false` | As solicitações em rascunho ficam fora destas listas, a menos que isto esteja ativado |
| `assigned_issues` | `false` | Issues abertas atribuídas a você, todos os repositórios |
| `namespaces` | `[]` | Quais repositórios cada lista mantém (solicitações de revisão, issues atribuídas, solicitações abertas); vazio significa sem limite, e `open_requests` então pesquisa o seu próprio namespace mais todas as participações (GitHub: no máximo 12 são pesquisados, GitLab: 10 grupos e 10 projetos próprios, Gitea, Forgejo e Gerrit: 10). Uma entrada corresponde ao caminho completo de um repositório (`group/sub/project`, no Azure DevOps `Project/repo`, no Bitbucket Data Center `KEY/repo`) quando é igual a ele ou é um prefixo de caminho que termina em `/`: `ptm` mantém `ptm/produits/x`, não `ptmx/y`; maiúsculas e minúsculas são ignoradas. As Configurações mostram uma entrada por linha; um arquivo antigo pode ter uma só string separada por vírgulas (`"ptm, formation"`), que ainda carrega e é salva de volta como lista |
| `namespaces_regex` | `false` | Lê cada entrada de `namespaces` como uma expressão regular comparada com o caminho completo do repositório, como você a escreve (então `^ptm/(produits\|infra)/` ancora no início; sem `^` ou `$` ela corresponde em qualquer ponto do caminho; acrescente `(?i)` para ignorar maiúsculas e minúsculas). `open_requests` então pesquisa as suas participações e as expressões filtram as linhas. Uma expressão inválida aparece sob o campo nas Configurações e não é salva; em um arquivo editado à mão ela deixa as listas vazias com uma nota |
| `review_mode` | `"ask"` | O que faz revisar uma solicitação cujo repositório não está clonado: `ask`, `clone` (clonar e depois revisar) ou `api` (revisar pela API do host) |

As respostas ficam em cache por perfil em `radar-account.json`, ao lado do
`config.toml`, por cinco minutos; o arquivo é um cache e pode ser apagado com segurança.

### O repositório de confiança (não está no `config.toml`)

Aquilo para que cada integração e cada perfil de IA com chave de API recebeu confiança (o endereço
dela, o id da conta, uma chave de certificado fixada, se o http simples foi permitido) fica em
`trust.toml`, na pasta de dados privada só do usuário (`<data>/private/trust.toml`:
`%APPDATA%\GitPoulpi\private\` no Windows, `~/.local/share/gitpoulpi/private/`
no Linux). Ele não faz parte do `config.toml` de propósito: quem pode editar a
configuração não deve conseguir ampliar a confiança editando-a. Altere-o com
`gitpoulpi hosting integrations trust` e `gitpoulpi ai trust` (veja
[a verificação de segurança](hosting-integration.md#a-verificação-de-segurança)); o arquivo é
reescrito pelo GitPoulpi, não à mão.

## `[mcp]` (o que os agentes de IA podem fazer)

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `mcp.allow_mutations` | `false` | Desativado: ferramentas MCP que alteram qualquer coisa são recusadas; leituras, `ui_*` e propostas continuam funcionando. `true` deixa os agentes alterarem o repositório. |
| `mcp.mutation_allowlist` | `[]` | Não vazia: só estes nomes de ferramenta podem alterar coisas (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: recusa as ferramentas `ui_*` que movem a sua janela (não `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (a janela mostra um chip que você confirma), `auto` (aplicado a menos que você esteja ocupado), `off`. `--ui-control` no comando do servidor o substitui por agente. Também vale para `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | Permite `ui_propose_*` / `gitpoulpi ui propose`: diálogos pré-preenchidos que você mesmo confirma. |
| `mcp.allow_history_purge` | `false` | Somente no `config.toml` (sem linha nas Configurações). `true` lista a ferramenta `rewrite_purge` para os agentes (ela ainda exige `allow_mutations`). Desativado, o agente não vê a ferramenta: um purge apaga de vez o histórico antigo de uma reescrita de histórico, o desfazer dela e os reflogs. A CLI e a TUI não são limitadas por ele. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` e `gitpoulpi-mcp` se recusam a iniciar, com uma mensagem que cita esta chave. Configurações > Agentes e MCP > *Servidor MCP*. Lida quando um agente inicia o servidor. |
| `mcp.bind` | `"127.0.0.1:8808"` | O endereço em que `--http` escuta quando você não informa nenhum (`--http ADDR` / `--bind` vencem). Só loopback, a menos que haja `--allow-remote`. Vazio ou o antigo `"stdio"` significam o padrão. Configurações > Agentes e MCP > *Porta do servidor MCP* edita a porta e mantém o host. |
| `mcp.http_auth` | `true` | `--http` só responde a uma requisição com `Authorization: Bearer <token>` (o token de `gitpoulpi mcp token`). `false` remove a verificação; o servidor então avisa no stderr, e `--allow-remote` é recusado. Configurações > Agentes e MCP > *Autenticação HTTP*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (um certificado gerado uma vez na pasta de dados privada, com a impressão digital SHA-256 impressa ao iniciar) ou `files` (`tls_cert` e `tls_key`). Com o TLS ativado, o servidor encerra o TLS por conta própria e não existe um listener simples. Configurações > Agentes e MCP > *TLS*. |
| `mcp.tls_cert` | `""` | Cadeia de certificados PEM usada com `tls = "files"`. |
| `mcp.tls_key` | `""` | Chave privada PEM usada com `tls = "files"`. |
| `mcp.behind_proxy` | `false` | Você declara que um túnel ou proxy reverso na frente do endpoint cuida da criptografia. Isso não dá para conferir de dentro do servidor, então `--allow-remote` sem TLS exige que esteja definido. O trecho entre o proxy e o servidor é texto simples nessa rede. Configurações > Agentes e MCP > *Atrás de um túnel ou proxy reverso*. |
| `mcp.allow_sensitive_staging` | `false` | Somente no `config.toml`. Desativado, as ferramentas de stage do MCP não oferecem `allow_sensitive`, e um stage que adicionaria um arquivo sinalizado (veja `[privacy]`) é recusado com uma mensagem dizendo ao agente para perguntar a você. Ativado, um agente pode passar `allow_sensitive`. A CLI e a TUI não são controladas por ele: elas perguntam a você. |

`allow_mutations`, `mutation_allowlist` e `allow_ui` controlam apenas o servidor
MCP; a CLI é o seu próprio shell e não é controlada por elas. Veja [MCP e
agentes](mcp-and-agents.md#permissões).

## `[ai]` (IA integrada, desativada por padrão)

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `ai.enabled` | `false` | A chave geral. Desativada, nada de IA aparece em lugar nenhum e nada é enviado. |
| `ai.default_profile` | `"default"` | Id do perfil que as ações usam quando não indicam nenhum. |
| `ai.global_instructions` | `""` | Texto acrescentado ao prompt de toda ação. Pode usar `${repo}`, `${uilang}`, ... (veja [IA > Variáveis do prompt](ai.md#variáveis-do-prompt)); o mesmo vale para as instruções por ação e do repositório. |
| `ai.actions.<action>.profile` | `""` | Perfil de uma ação; vazio = o padrão. Ações: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`. Um espaço de trabalho (`workspaces.toml`, `[workspace.ai_instructions]` e `[workspace.ai_overrides.<action>]`) e um repositório podem acrescentar ou substituir essas instruções: veja [IA](ai.md#instruções). |
| `ai.actions.<action>.instructions` | `""` | Texto acrescentado só para essa ação. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` ou `no`: se o agente local de um rascunho de revisão de código trabalha dentro do repositório nessa execução, substituindo o `workdir` do perfil de IA. Um repositório pode definir o seu (`gitpoulpi ai review-context --repo`). Veja [IA > Contexto da revisão](ai.md#contexto-da-revisão). |
| `ai.review.include_claude_md` | `true` | Entrega a um rascunho de revisão de código o `CLAUDE.md` do repositório (raiz, lido na branch de destino da solicitação; cortado em 8 KB). |
| `ai.review.include_agents_md` | `true` | O mesmo para o `AGENTS.md`. |
| `ai.review.extra_file` | `""` | Mais um arquivo para um rascunho de revisão de código: um caminho relativo à raiz do repositório, um caminho absoluto ou um link para um arquivo de um host de código (lido pelo perfil de integração correspondente, senão de forma anônima). Todos os arquivos juntos são cortados em 16 KB. |
| `ai.privacy.redact_secrets` | `true` | Substitui todo segredo que o scanner reconhece em uma requisição (um token, um bloco de chave privada, uma senha em uma URL, um valor `password = ...` que parece aleatório) por `[REDACTED:<rule>]` antes de ela sair. As Configurações > IA mostram um aviso vermelho enquanto está desativado. |
| `ai.privacy.exclude_sensitive_files` | `true` | Deixa de fora do que é enviado o diff de todo arquivo que corresponde a `privacy.sensitive_patterns`. Um agente local que trabalha na pasta do repositório ainda pode ler arquivos por conta própria: veja [IA > Privacidade](ai.md#privacidade). |
| `ai.sent_preview.<action>` | `true` | Uma chave por ação (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`): mostra "O que é enviado" antes de a requisição sair. O "Não mostrar de novo para ..." do diálogo grava `false`; Configurações > IA > *Mostrar o que é enviado antes de...* os ativa de novo. |
| `ai.limits.conflict_bytes` | `40000` | Máximo de bytes que um conflito de merge (seus blocos base, atual e recebido juntos) pode ter quando "Sugerir resolução" o envia. Acima disso, o GitPoulpi não falha: avisa e oferece *Enviar mesmo assim* (só esta requisição), *Abrir configuração* e *Cancelar*; um conflito nunca é cortado. O limite é uma proteção do próprio GitPoulpi, não do modelo: aumente-o quando o seu modelo tiver espaço. Mantido entre 1000 e 50000000; configurável em Configurações > IA > Limites. Os outros limites abaixo funcionam do mesmo jeito e ficam todos nesse grupo. |
| `ai.limits.diff_bytes` | `60000` | Máximo de bytes do diff que a mensagem de um commit, uma explicação, um nome de stash, a descrição de um pull request, um plano de commits ou uma divisão de commit pode levar, e do diff inteiro de uma revisão de código. Acima disso nada é cortado em silêncio: no TUI, *Acima do limite da IA* oferece *Enviar tudo* (só esta requisição, nada cortado), *Enviar truncado* (o final fica de fora, como antes), *Abrir configuração* e *Cancelar*; a CLI e o MCP truncam e informam em `truncated` (`--send-all` envia tudo). Mantido entre 1000 e 50000000. |
| `ai.limits.review_file_bytes` | `12000` | Máximo de bytes do diff de um arquivo em uma revisão de código. Acima disso, a mesma pergunta e as mesmas duas respostas; *Enviar truncado* encurta esse arquivo com uma nota visível. Mesma faixa. |
| `ai.limits.review_text_bytes` | `4000` | Máximo de bytes da descrição do pull request revisado. Acima disso, *Enviar truncado* corta a descrição com uma nota `[text cut]`. Mesma faixa. |
| `ai.limits.review_threads_bytes` | `6000` | Máximo de bytes da lista de comentários de revisão que já existem (dada para que o modelo não os repita). Acima disso, *Enviar truncado* corta a lista. Mesma faixa. |
| `ai.limits.review_context_file_bytes` | `8192` | Máximo de bytes de um arquivo de contexto do projeto (`CLAUDE.md`, `AGENTS.md`, o arquivo extra) dado a uma revisão de código. Acima disso, *Enviar truncado* encurta o arquivo com uma nota. Mesma faixa. |
| `ai.limits.review_context_bytes` | `16384` | Máximo de bytes de todos os arquivos de contexto juntos. Acima disso, *Enviar truncado* encurta o arquivo que passa do limite e deixa de fora os seguintes. Mesma faixa. |

A tabela `[preferences.ai_review]` própria de um repositório (no `repo-state.toml` dele, ou a entrada de um repositório que não está clonado em `remote-reviews.toml`) aceita as mesmas chaves, todas opcionais, mais `no_extra_file = true`: nenhum arquivo extra para esse repositório, seja o que for que `ai.review.extra_file` diga. Uma chave não definida, um `extra_file` não definido e um `no_extra_file` ausente herdam o valor global; definir `extra_file` como um arquivo limpa a chave e vice-versa.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # vazio = o padrão do tipo
base_url = ""           # vazio = o padrão do tipo (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # onde a chave de API é guardada; nunca a chave em si
command = ""            # somente local_agent: o programa exato ("claude", um alias, um wrapper, um caminho)
harness = ""            # somente local_agent: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; vazio = inferido de `command`
args = []               # somente local_agent, por exemplo ["-p", "--model", "haiku"]
run_mode = "headless"   # somente local_agent: headless (padrão) | terminal (a tela interativa do agente em um terminal virtual)
terminal_args = []      # somente local_agent: os argumentos do modo terminal (args mantém o conjunto headless)
workdir = "default"     # somente local_agent: default | dedicated | repository (default = pasta dedicada no modo terminal, pasta do GitPoulpi no modo headless)
timeout_secs = 0        # somente local_agent: segundos até o agente ser interrompido; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # somente local_agent, opcional: variáveis definidas só para esse processo
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = uma das variáveis do próprio GitPoulpi
```

`harness` escolhe as sugestões que Configurações > IA oferece (argumentos
headless, opções, opção e lista de modelos, variáveis de ambiente)
independentemente de `command`; as duas chaves são omitidas do arquivo quando
vazias. Os valores de ambiente nunca são registrados em log e ficam ocultos nas
Configurações quando parecem segredos.

Configurações > IA oferece predefinições para `harness`/`command`/`args`/`env` (as CLIs de agente comuns, com
suas opções headless, opção de modelo e variáveis de ambiente úteis) e para `kind`/`base_url`/`model` (os
provedores HTTP hospedados e locais); veja [IA > Configurar a IA](ai.md#configurar-a-ia).

Um perfil chamado *Default* é criado quando não há nenhum. Arquivos antigos com
`[[ai.presets]]` / `[ai.defaults]` são migrados ao carregar e mantidos em
`[[ai.legacy_presets]]`. Todo o resto: [Assistência de IA](ai.md).

## `[repos]`, `[session]`

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `repos.clone_root` | `""` | Onde o diálogo Clonar começa; vazio = `%USERPROFILE%\source\repos` ou `~/src`. |
| `repos.clone_start_source` | `"last"` | A fonte em que o diálogo Clonar abre: `last` (padrão, ou vazio) = a que ele mostrava quando foi fechado pela última vez, `url` = sempre Clonar por URL, ou o id de um perfil de integração (`id` na sua entrada `[[integrations]]`). Um perfil que não existe mais, está desativado ou não está conectado volta para Clonar por URL. A última fonte em si é estado, guardado em `clone-source.toml` na pasta de configuração, não aqui. |
| `repos.browse_root` | `""` | Onde Procurar abre; vazio = a última pasta escolhida, depois a pasta pessoal. |
| `repos.show_hidden` | `false` | O seletor de pastas mostra pastas ocultas (`Alt+H` alterna e salva). |
| `repos.show_siblings` | `false` | A árvore da esquerda do seletor de pastas também lista as outras pastas de cada nível aberto, como uma hierarquia real (`Alt+S` alterna e salva). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Onde o GitPoulpi guarda o que lembra sobre um repositório (filtros de refs, estado de recolhimento, a substituição pessoal do fluxo de trabalho, o cache de hospedagem): `in_git_dir` = `<git dir>/gitpoulpi` (compartilhada por todos os worktrees, nunca aparece em `git status`), `in_worktree` = `<repo>/.gitpoulpi` (o único lugar onde uma equipe pode fazer commit do `workflow.toml`), `user_data` = `<pasta de dados do SO>/repos/<name>-<id>` (`%APPDATA%\GitPoulpi\data` no Windows, `$XDG_DATA_HOME/gitpoulpi` no Linux, `~/Library/Application Support/GitPoulpi` no macOS). O valor antigo `".gitpoulpi"` ainda significa `in_worktree`. Mudar isto não move nada de volta: os dados encontrados em um `<repo>/.gitpoulpi` antigo são movidos para o lugar escolhido na primeira vez em que forem necessários. |

## `[log]` (o registro do aplicativo)

O que a aba “Aplicativo” do Registro de atividade mostra e o que “Copiar diagnóstico” cita. Detalhes: [Quando algo falha](troubleshooting.md#quando-algo-falha).

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `log.level` | `"info"` | A linha mais detalhada que é registrada: `error`, `warn`, `info`, `debug` ou `trace`. A caixa de nível do Registro de atividade a altera (e a salva aqui). |
| `log.retention_days` | `7` | Dias de arquivos de registro mantidos, incluindo hoje: de 1 a 3650, dez anos (um valor maior ou menor é ajustado ao limite). Sempre um número de dias, qualquer que seja a unidade em que as Configurações o mostrem. Os arquivos mais antigos são excluídos quando o GitPoulpi inicia. O arquivo de um dia também é limitado por `log.max_file_mb`, então o pior caso é cerca de `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | A unidade em que as Configurações mostram `log.retention_days`: `days`, `weeks`, `months`, `years` ou `auto`. Só de exibição, nunca muda por quanto tempo os arquivos são mantidos. Uma semana tem 7 dias, um mês tem 30 dias (não é um mês do calendário, então são os mesmos dias seja quando for aplicado) e um ano tem 365 dias. `auto`, e uma configuração escrita antes de esta chave existir, escolhe a maior unidade que divide os dias exatamente (14 dias aparecem como “2 semanas”, 45 dias como “45 dias”). Uma unidade que não divide os dias (`retention_days = 10` com `weeks`) é ignorada e os dias são mostrados. As Configurações gravam as duas chaves; um GitPoulpi mais antigo ignora a unidade e continua lendo os dias. |
| `log.max_file_mb` | `5` | O arquivo de um dia é rotacionado para um único backup `.old` nesse tamanho, então um dia nunca ocupa mais que o dobro. |
| `log.max_file_unit` | `"auto"` | A unidade em que as Configurações mostram o tamanho: `mib`, `gib` (1024 MiB) ou `auto`. Só de exibição: o tamanho é sempre `max_file_mb` MiB, de 1 a 1024. |
| `log.timings` | `true` | A aba Operações do Git do Registro de atividade mostra quanto tempo cada operação levou (`Push - 123 ms`: abrir um repositório, atualizar, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). As durações são medidas pela janela, do pedido até a resposta do daemon, então incluem a ida e volta por IPC. Desligado, oculta as durações e as entradas das operações bem-sucedidas. As falhas são sempre listadas. |
| `log.extended` | `false` | “Registro estendido”: a aba Operações do Git também lista as operações em segundo plano (a atualização periódica que consulta o status, o auto-fetch) com as durações. Elas são sempre gravadas no registro do aplicativo no nível `debug`. |
| `log.commit_count` | `"system-git"` | Como o `carregados / total` da linha de status descobre o total. `"system-git"` executa `git rev-list --count` (um processo, o mais rápido em uma máquina normal: o git lê o arquivo commit-graph). `"libgit2"` conta dentro do processo com um revwalk do libgit2 e não inicia nada: escolha-o quando todo processo de linha de comando demora a iniciar, tipicamente porque um antivírus com verificação em tempo real inspeciona cada um; com 50.000 commits leva cerca de 200 ms em segundo plano em vez de 12 ms, mas nenhum processo é iniciado. `"off"` nunca conta: a linha de status mostra só os commits carregados. Aplicado ao vivo. Uma contagem filtrada por caminho sempre usa o git do sistema. |
| `log.use_mailmap` | `true` | Mostra cada autor como o `.mailmap` do repositório (e `mailmap.file`) o nomeia no Log, no blame e no histórico de arquivo: o daemon pede ao git `%aN`/`%aE` (o backend de sistema) ou aplica ele mesmo os mesmos arquivos (o backend libgit2), então a resposta é idêntica. Os commits gravados não mudam e nada é reescrito. A busca por autor no Log casa com os nomes como são exibidos. Aplicado ao vivo; o Log relê dentro do seu polling de 2 s. |

Os arquivos são `gitpoulpi-YYYY-MM-DD.log` na pasta de estado do sistema (`~/.local/state/gitpoulpi/logs` no Linux; `<cache>\logs` no Windows e no macOS; `<config>/cache/logs` com `GITPOULPI_CONFIG_DIR`). Tokens, senhas e cabeçalhos `Authorization` são mascarados antes de qualquer coisa ser gravada.

## `[search]` (a busca no espaço de trabalho)

O que `gitpoulpi search` (e sua ferramenta MCP) examina. Detalhes: [Espaços de trabalho e busca na CLI](cli-reference.md#espaços-de-trabalho-e-busca).

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `search.commit_limit` | `200000` | Commits (do mais recente, em todas as branches, branches remotas e tags) cujas mensagens o índice de busca mantém na memória por repositório, cerca de 100 bytes mais a mensagem cada um. Um histórico maior é cortado aí e a resposta diz que é `partial`. `0` desliga a busca de commits (branches, tags, stashes e worktrees continuam sendo buscados). Lido ao vivo: a próxima busca obedece a uma alteração. |

## `[rewrite]` (reescrita de histórico)

O que `gitpoulpi rewrite` e o painel de reescrita do histórico protegem. Detalhes: [Reescrita do histórico](history-rewrite.md).

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Branches (nomes, ou padrões em que `*` casa com qualquer texto) que uma reescrita deixa em paz quando já foram publicadas, a menos que a execução as cite (`--include-protected BRANCH`; `--include-all-protected` para o push; a caixa de cada branch no painel). Somadas às bases que o `workflow.toml` de um repositório marca como `protected`. O push de uma reescrita as ignora do mesmo jeito. É a proteção do próprio GitPoulpi: a proteção de branch do host ainda decide se o push forçado é aceito. Lido ao vivo: a próxima prévia obedece a uma mudança. |
| `rewrite.external_scripts` | `["shell"]` | Quais linguagens de script externo a regra de Script pode usar: qualquer uma entre `"shell"` (bash ou sh; pwsh ou powershell para scripts PowerShell), `"python"`, `"lua"` (um Lua instalado; o Lua embutido está sempre ativo e não precisa de entrada). **Um script externo não é isolado: ele roda com os seus direitos.** Uma linguagem que não está na lista fica esmaecida no painel e é recusada (`scripts_disabled`) pela CLI, pelo MCP e pelo daemon. Mesmo estando na lista, um script externo que vem do repositório ou de uma receita compartilhada só roda depois que você confiou nele (`gitpoulpi rewrite script trust ARQUIVO`, ou a pergunta no painel); a confiança é ligada ao hash do conteúdo do script, então um script alterado pergunta de novo. Para ativar o Python, edite o `config.toml` (uma lista não pode ser definida com `config set`): `[rewrite]` e depois `external_scripts = ["shell", "python"]`. Lido ao vivo. |
| `rewrite.script_timeout_ms` | `5000` | Quanto tempo um script externo tem para responder a um commit. Em um timeout, toda a árvore de processos é encerrada e a reescrita falha (`script_failed`, citando o commit); nada foi movido. `0` vale como o padrão. Lido ao vivo. |
| `rewrite.script_timeout_unit` | `"auto"` | A unidade em que as Configurações mostram o tempo limite: `milliseconds`, `seconds` ou `auto`. Só de exibição: o limite é sempre `script_timeout_ms` milissegundos. As Configurações aceitam de 1 a 600000 (dez minutos); `0` continua valendo como o padrão. Lido ao vivo. |
| `rewrite.script_total_timeout_s` | `3600` | Quanto tempo uma passagem de um script externo pode levar no total. `0` vale como o padrão. Lido ao vivo. |
| `rewrite.script_total_timeout_unit` | `"auto"` | A unidade em que as Configurações mostram o tempo limite: `seconds`, `minutes`, `hours` ou `auto`. Só de exibição: o limite é sempre `script_total_timeout_s` segundos. As Configurações aceitam de 1 a 86400 (um dia); `0` continua valendo como o padrão. Lido ao vivo. |

## `[updates]` (procurando uma release mais nova)

O que o chip de versão da barra de título faz sobre releases mais novas. Detalhes: [Atualizações](updates.md).

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` pergunta ao GitHub uma vez por dia, em segundo plano, se existe uma release mais nova, e transforma o chip de versão em `vX.Y.Z disponível`. `"auto"` também baixa e prepara a atualização sozinho quando esta build veio de um arquivo de release (ele ainda pergunta antes de reiniciar); para qualquer outra build se comporta como `"notify"`. `"off"` nunca faz uma requisição. Aplicado ao vivo. |
| `updates.channel` | `"stable"` | `"stable"` considera só releases completas; `"prerelease"` também oferece candidatas a lançamento (`-rc.1`). Aplicado ao vivo. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | O `owner/name` no github.com cujas releases são verificadas. Enquanto esse repositório é privado, o GitHub responde 404, a menos que um token esteja guardado para o github.com (Configurações > Integrações); sem um, a verificação falha em silêncio e escreve uma linha no registro do aplicativo. |

Um administrador pode definir `mode`, `channel` (como `release_channel`) e `repo`, além de mais algumas opções de atualização, em um arquivo de política válido para a máquina inteira ou no registro do Windows. Uma chave que a política define vence o `config.toml` e aparece nas Configurações como **Definido pelo seu administrador**. O `mode` na política é um teto: você pode escolher um valor menor, nunca um maior. Veja [Distribuição corporativa](enterprise-distribution.md#o-arquivo-de-política-e-o-registro).

## `[privacy]` (arquivos sensíveis)

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Padrões de caminho no estilo do gitignore. Um padrão sem `/` corresponde ao nome do arquivo em qualquer profundidade, um com `/` ao caminho inteiro (um `/` no início o ancora na raiz); `*` e `?` ficam dentro de uma pasta, `**` atravessa pastas; um `!` permite de novo o que um padrão anterior pegou e a última correspondência vence; maiúsculas e minúsculas são ignoradas. Usado pela proteção de stage e por `ai.privacy.exclude_sensitive_files`. |
| `privacy.staging_guard` | `true` | Fazer stage de um arquivo que corresponde a um padrão, ou cujo conteúdo tem um segredo reconhecido, pergunta antes (TUI), é recusado a menos que `--allow-sensitive` seja dado (CLI) ou que `mcp.allow_sensitive_staging` deixe o agente passar `allow_sensitive` (MCP). Arquivos de texto de até 1 MiB cada e 32 MiB no total são lidos; arquivos binários só são comparados pelo nome. Veja [Stage e commit](staging-and-committing.md#arquivos-sensíveis). |

## `[conflict_prevention]` (listas de branches base)

As branches que a detecção proativa de conflitos pode tomar como base da branch atual formam uma lista de padrões (veja [Resolver conflitos de merge](merge-conflicts.md#prevenção-de-conflitos)). As listas podem ter nome e ser compartilhadas; Configurações > Prevenção de conflitos as edita.

| Chave | Padrão | Valores e efeito |
|---|---|---|
| `conflict_prevention.default_list` | `""` | O nome da lista que um repositório usa quando não tem uma própria. Vazio, ou um nome que nenhuma lista tem, significa o `**` integrado (todas as branches). Lido de novo a cada verificação, então uma mudança vale para os repositórios abertos sem reiniciar. |
| `conflict_prevention.lists` | nenhuma | As listas com nome, uma tabela cada: `[conflict_prevention.lists.release]` com `patterns = ["origin/release/**", "!origin/release/old"]`. Um `patterns` vazio permite todas as branches. Editar uma lista vale para todo repositório que a segue. |

Qual lista um repositório segue fica no `repo-state.toml` dele (abaixo). Precedência: os padrões próprios do repositório, depois a lista com nome que ele segue, depois `default_list`, depois `**`.

## `[keybindings]`

Remapeia os atalhos da TUI; Configurações > Teclado e mouse o edita.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # padrão Ctrl+G
nav_prefix = "Alt+n"             # padrão Ctrl+N
settings   = ["Ctrl+,", "F2"]    # uma lista = alternativas
tab_menu   = []                  # unbound

[keybindings.git_prefix]         # a tecla pressionada depois do prefixo git
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`; um valor substitui os padrões
dessa ação, remover a linha os restaura. Acordes: `Ctrl+Shift+N`, `Alt+n`, `F2`,
`PageDown`, `Space`, `,`, `P` (= `Shift+p`). Uma entrada inválida nunca bloqueia
nada: a TUI a ignora, mantém o padrão e avisa na linha de status. Contextos, ids
de ação, padrões e as regras de conflito: [Atalhos de teclado](keybindings.md).

## Configurações por repositório: `.gitpoulpi/repo-state.toml`

Guardado dentro do repositório; a pasta se ignora sozinha (um `.gitignore` com
`*`), então nunca aparece como alteração. Gravado pela TUI e pela CLI; você
raramente o edita.

| Chave | O que é | Definida com |
|---|---|---|
| `[preferences] encoding` | Substitui `git.default_encoding` aqui | Configurações > Git e codificação |
| `[preferences] lfs_enabled` | Substitui `git.lfs_enabled` aqui | Configurações > Git e codificação |
| `[preferences] profile` | O id de `[[profiles]]` que assina os commits aqui; ausente significa `general.active_profile` | a lista de perfis da barra de ferramentas, **Repositório: alterar perfil...** |
| `[preferences] autocrlf`, `eol` | *Ainda não usadas* | Configurações > Git e codificação |
| `[preferences] push_after_commit` | Faz push da branch para o seu upstream depois de cada commit (desativado quando ausente) | Configurações > Configurações do repositório > Commit |
| `[preferences] skip_hooks_by_default` | O Ignorar hooks do formulário de commit começa marcado (desativado quando ausente) | Configurações > Configurações do repositório > Commit |
| `[preferences] squash_merge` | O merge de uma branch local faz squash: coloca em stage, sem commit de merge (desativado quando ausente) | Configurações > Configurações do repositório > Opções de merge |
| `[preferences] template_apply`, `template_strip_comments` | Preenche um formulário de commit vazio com o modelo; remove as linhas de comentário `#` na hora do commit (as duas desativadas quando ausentes) | Configurações > Configurações do repositório > Modelo de mensagem |
| `[preferences] conflict_detection` | Detecção proativa de conflitos neste repositório; ausente, segue `general.conflict_detection` | Configurações > Configurações do repositório > Prevenção de conflitos |
| `[preferences] conflict_base_branches` | Os padrões próprios de Branches base do repositório, um por linha (ainda usados como **Lista própria**) | Configurações > Configurações do repositório > Prevenção de conflitos |
| `[preferences] conflict_base_source` | De onde vêm as Branches base: `default` (a lista padrão global), `list` (`conflict_base_list`) ou `own` (`conflict_base_branches`). Ausente: os padrões próprios quando existem, senão o padrão | Configurações > Configurações do repositório > Prevenção de conflitos |
| `[preferences] conflict_base_list` | A `[conflict_prevention.lists.<name>]` que este repositório segue quando a origem é `list`; uma lista excluída volta ao padrão | Configurações > Configurações do repositório > Prevenção de conflitos |
| `commit-template.txt` (um arquivo ao lado de `repo-state.toml`) | O modelo de commit; o `commit.template` do git aponta para ele | Configurações > Configurações do repositório > Modelo de mensagem |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: as instruções de IA deste repositório | Configurações > Configurações do repositório > IA |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` de uma ação aqui | Configurações > Configurações do repositório > IA |
| `default_remote` | O remoto que Push/Pull/Fetch de um clique usam | menu de remotos, `gitpoulpi remote default <name>` |
| `hosting_integration` | O perfil de `[[integrations]]` fixado aqui | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | O projeto do provedor que as listas de PRs / issues leem, como `{ integration, project, filter }`. `filter` é opcional: uma cláusula extra nativa do provedor (um fragmento JQL do Jira, uma condição WIQL do Azure, nomes de listas do Trello) acrescentada com AND à consulta da lista; GitHub e GitLab a ignoram. | seções da barra lateral, `gitpoulpi hosting link` |
| `ref_visibility` | Refs ocultas / em solo | Barra lateral Refs, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Remotos ocultos do grafo por inteiro (todas as branches, inclusive as buscadas depois) | o olho e o menu do remoto, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Seções da barra lateral e pastas de branches e de tags recolhidas. `[folds]` também guarda `status_folders` (pastas recolhidas das árvores de arquivos Unstaged / Staged) e `commit_folders` (pastas recolhidas da árvore de arquivos alterados de um commit, a mesma para todo commit), cada uma limitada a 500 caminhos, descartando os mais antigos primeiro | a barra lateral, as árvores de arquivos |
| `pending_auto_stash` | Um stash que o GitPoulpi fez e ainda não devolveu | gravado pelo daemon; veja [Auto-stash](log-and-history.md#auto-stash) |

## Outras variáveis de ambiente

| Variável | Efeito |
|---|---|
| `GITPOULPI_CONFIG_DIR` | A pasta de configuração (veja acima). |
| `GITPOULPI_TOKEN_<ID>` | Um token de hospedagem para o perfil `<ID>`, sobre o guardado (CI, uso pontual). |
| `GITPOULPI_GIT` | O executável `git` a rodar (padrão: `git` no `PATH`; no Windows o verdadeiro `mingw64\bin\git.exe` por trás do shim `cmd\git.exe` do Git for Windows, o que economiza um processo por chamada). |
| `GITPOULPI_LANG=fr` | O idioma da interface, sobre `ui.language` (qualquer valor de `ui.language`; `qps` é um pseudo-locale para tradutores). Um valor desconhecido ou vazio é ignorado. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Nenhuma imagem, seja o que for que a configuração diga. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Dicas de resolução de imagem (tamanho da célula em pixels; superamostragem, padrão 2, `1` economiza CPU). |
| `GITPOULPI_NO_BROWSER=1` | Nunca abrir um navegador (a URL continua sendo mostrada). |
| `GITPOULPI_TIMING=1` | Grava os tempos das requisições em `gitpoulpi-timing.log` na pasta temporária (relatórios de desempenho). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | Não esperar por Enter depois que a TUI sai com um erro. |
