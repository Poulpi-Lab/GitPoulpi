[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · **Português (Portugal)** · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Atalhos de teclado

Esta página lista as teclas **predefinidas**. Pode reatribuir a maioria delas. A aplicação
mostra sempre as teclas em vigor: na paleta de comandos, na ajuda `?`, nas
barras de estado, nos menus de contexto e nas dicas dos botões. Quando muda uma tecla, estes
locais mostram a nova tecla. Para o rato, consulte
[Rato e gestos](mouse-and-gestures.md).

## Reatribuir atalhos

Abra **Definições > Teclado e rato**. A página está descrita em
[Definições](settings.md#teclado-e-rato). As alterações aplicam-se de imediato e são guardadas
na tabela `[keybindings]` do `config.toml`. Pode também editar essa tabela à
mão:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # o tmux e o GNU screen costumam usar Ctrl+B e Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

A sintaxe, as regras de conflito e os avisos estão na
[Referência de configuração](config-reference.md) em `[keybindings]`. Em resumo:

- Um erro é comunicado na barra de estado, e a tecla predefinida mantém-se.
- Duas ações no mesmo acorde no mesmo contexto são recusadas.
- Um prefixo precisa de `Ctrl` ou `Alt`.
- Um acorde de painel que esconde um acorde global gera apenas um aviso.

A tabela abaixo lista todas as teclas que pode reatribuir. Algumas teclas são fixas e não podem
ser reatribuídas: o movimento nas listas (`j` `k` `h` `l`, as setas, `PageUp` / `PageDown`,
`Home` / `End`), `Enter`, `Esc`, `Tab`, os dígitos (o prefixo de contagem), as
teclas de edição de texto, e as teclas `Enter` / `Tab` / setas dos popups. A paleta
tem `Aplicação: atalhos de teclado...` e `Atalhos: repor tudo...`.

## Atalhos predefinidos

Todas as teclas que pode reatribuir, com o id a usar em `[keybindings.<contexto>]` e
a sua predefinição. A tabela é gerada a partir da aplicação (um teste mantém-na atualizada).
As teclas depois de um prefixo são mostradas com o prefixo (`Ctrl+G s`).

<!-- keymap:begin -->
| Contexto | Id da ação | O que faz | Teclas predefinidas |
|---|---|---|---|
| `global` | `git_prefix` | Prefixo das ações do Git | `Ctrl+G` |
| `global` | `nav_prefix` | Prefixo de navegação | `Ctrl+N` |
| `global` | `palette` | Paleta de comandos | `Ctrl+P` |
| `global` | `settings` | Definições | `Ctrl+,` |
| `global` | `home` | Início (gestão de repositórios) | `Ctrl+O` |
| `global` | `clone` | Clonar repositório | `Ctrl+Shift+N` |
| `global` | `new_tab` | Novo separador | `Alt+T` |
| `global` | `workspace_switch` | Mudar de área de trabalho | `Alt+W` |
| `global` | `radar` | Abrir o Radar | `Alt+R` |
| `global` | `workspaces` | Abrir as áreas de trabalho | `Alt+J` |
| `global` | `tab_menu` | Menu do separador | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Abrir em (lista) | `Alt+O` |
| `global` | `tab_open_in_default` | Abrir no predefinido | `Ctrl+Alt+O` |
| `global` | `close_tab` | Fechar separador | `Ctrl+W` |
| `global` | `next_tab` | Próximo separador | `Ctrl+PageDown`, `Ctrl+Tab` |
| `global` | `prev_tab` | Separador anterior | `Ctrl+PageUp`, `Ctrl+Shift+Tab` |
| `global` | `help` | Ajuda | `?` |
| `global` | `quit` | Sair | `q` |
| `global` | `refresh` | Atualizar | `r` |
| `global` | `refresh_all` | Atualizar (fetch, pull requests, issues) | `F5` |
| `global` | `find_in_log` | Pesquisar no grafo de commits | `f` |
| `global` | `workspace_search` | Pesquisar na área de trabalho | `Ctrl+Alt+F` |
| `global` | `select_tab` | Selecionar separador | `t` |
| `global` | `undo` | Desfazer a última operação | `Ctrl+Z` |
| `global` | `redo` | Refazer a operação desfeita | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Abrir o histórico de desfazer | `Alt+Z` |
| `global` | `commit_column` | Mostrar ou ocultar a coluna de commits | `Alt+C` |
| `global` | `undo_offer` | Restaurar a branch para onde a ação bloqueada a deixou | `Alt+Shift+Z` |
| `global` | `context_menu` | Menu de contexto | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Compor mensagem de commit | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Fazer commit das alterações em stage agora | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | IA: dividir alterações em commits | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | IA: explicar alterações | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash com mensagem | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | IA: escrever mensagem de commit | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage de todas as alterações | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Descartar todas as alterações | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage do ficheiro selecionado | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage do ficheiro selecionado | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `pull_all_branches` | Pull de todas as branches | `Ctrl+G Ctrl+P` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Abrir pull request associado | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push de todas as tags | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch de todos os remotos | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Nova tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Nova branch | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash das alterações | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick do commit selecionado | `Ctrl+G y` |
| `git_prefix` | `revert` | Reverter o commit selecionado | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Editar mensagem do commit | `Ctrl+G e` |
| `git_prefix` | `amend` | Alterar o commit selecionado | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Alterar o autor do commit selecionado | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (misto) para o commit selecionado | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase sobre a branch selecionada | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop do stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Eliminar a ref selecionada ou descartar stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Abrir o menu LFS | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Abrir o menu Flow | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch de todos os objetos | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Checkout de todos os ficheiros | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull de objetos | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push de todos os objetos | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Limpar objetos locais | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Abrir o painel de reescrita do histórico | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Recolher ou restaurar o painel Refs | `Ctrl+N Shift+H` |
| `nav_prefix` | `collapse_status` | Recolher ou restaurar o painel Status | `Ctrl+N Shift+L` |
| `nav_prefix` | `settings` | Abrir definições | `Ctrl+N ,` |
| `nav_prefix` | `about` | Abrir Acerca de | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Abrir o Registo de atividade | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Abrir o registo de atividade global | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Mostrar as tarefas em segundo plano | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Mostrar o painel de atualizações | `Ctrl+N u` |
| `nav_prefix` | `help` | Alternar ajuda | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Selecionar separador | `Ctrl+N Tab` |
| `nav_prefix` | `change_theme` | Mudar tema | `Ctrl+N t` |
| `nav_prefix` | `change_language` | Mudar idioma | `Ctrl+N l` |
| `nav_prefix` | `change_graph_color_theme` | Mudar tema de cores do grafo | `Ctrl+N Shift+T` |
| `nav_prefix` | `toggle_graph_row_tints` | Alternar tom das linhas do grafo | `Ctrl+N Shift+Y` |
| `nav_prefix` | `grow_section` | Aumentar a secção Refs | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Diminuir a secção Refs | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Aumentar o bloco de mensagem do commit | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Diminuir o bloco de mensagem do commit | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Alargar o painel Refs | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Estreitar o painel Refs | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Alargar o painel Status | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Estreitar o painel Status | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agente: mostrar a vista pendente | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agente: voltar (ou dispensar a vista pendente) | `Ctrl+N Backspace` |
| `nav_prefix` | `arrange_dialog` | Arranjar a janela aberta (mover, redimensionar) | `Ctrl+N m` |
| `refs` | `filter` | Filtrar refs por nome | `/` |
| `refs` | `solo` | Solo na seleção | `o` |
| `refs` | `clear_solo` | Parar o solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Alternar a hierarquia de branches | `y` |
| `refs` | `show_all` | Mostrar todas as refs ocultas | `Shift+H` |
| `refs` | `reveal` | Mostrar no grafo | `g` |
| `refs` | `mark_drop` | Marcar como origem do largar | `m` |
| `refs` | `drop_marked` | Largar aqui a ref marcada | `Shift+M` |
| `refs` | `expand_all` | Expandir todas as pastas | `=` |
| `refs` | `collapse_all` | Recolher todas as pastas | `-` |
| `refs` | `expand_below` | Expandir a pasta e as suas subpastas | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Recolher a pasta e as suas subpastas | `_`, `Shift+Left` |
| `refs` | `scroll_left` | Scroll the Pane Left | `Alt+Left` |
| `refs` | `scroll_right` | Scroll the Pane Right | `Alt+Right` |
| `log` | `toggle_file_mode` | Ficheiros: caminho ou árvore | `m` |
| `log` | `view_all_files` | Ficheiros: ver todos os ficheiros | `a` |
| `log` | `compare_selection` | Comparar as linhas selecionadas | `v` |
| `log` | `compare_swap` | Trocar as extremidades da comparação | `x` |
| `log` | `compare_next` | Ir para a próxima linha comparada | `n` |
| `log` | `pick_column` | Escolher uma coluna para mover ou redimensionar | `Shift+C` |
| `log` | `scroll_left` | Deslocar as colunas para a esquerda | `Shift+Left` |
| `log` | `scroll_right` | Deslocar as colunas para a direita | `Shift+Right` |
| `log` | `details_scroll_left` | Scroll the Pane Left | `Alt+Left` |
| `log` | `details_scroll_right` | Scroll the Pane Right | `Alt+Right` |
| `log` | `copy_mermaid` | Copiar commits como Mermaid gitGraph | `Shift+Y` |
| `log_columns` | `next` | Escolher a coluna seguinte | `Tab` |
| `log_columns` | `prev` | Escolher a coluna anterior | `Shift+BackTab` |
| `log_columns` | `move_left` | Mover a coluna para a esquerda | `<` |
| `log_columns` | `move_right` | Mover a coluna para a direita | `>` |
| `log_columns` | `wider` | Alargar a coluna | `]` |
| `log_columns` | `narrower` | Estreitar a coluna | `[` |
| `log_columns` | `reset_width` | Repor as larguras das colunas | `0` |
| `log_columns` | `fit` | Ajustar a coluna ao conteúdo | `=` |
| `log_columns` | `fit_all` | Ajustar todas as colunas ao conteúdo | `+` |
| `log_columns` | `layout` | Deslocamento lateral ou ajuste ao painel | `s` |
| `log_files` | `collapse_all` | Recolher ou expandir todas as pastas | `z` |
| `log_files` | `filter` | Filtrar os ficheiros alterados | `/` |
| `log_files` | `expand_all` | Expandir todas as pastas | `=` |
| `log_files` | `collapse_all_folders` | Recolher todas as pastas | `-` |
| `log_files` | `expand_below` | Expandir a pasta e as suas subpastas | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Recolher a pasta e as suas subpastas | `_`, `Shift+Left` |
| `log_files` | `scroll_left` | Scroll the Pane Left | `Alt+Left` |
| `log_files` | `scroll_right` | Scroll the Pane Right | `Alt+Right` |
| `status` | `compose` | Focar a mensagem do commit | `c` |
| `status` | `toggle_file_mode` | Ficheiros: caminho ou árvore | `m` |
| `status` | `view_all_files` | Ficheiros: ver todos os ficheiros | `a` |
| `status` | `amend` | Alternar amend | `Shift+A` |
| `status` | `skip_hooks` | Alternar ignorar hooks | `Shift+H` |
| `status` | `collapse_all` | Recolher ou expandir todas as pastas | `z` |
| `status` | `stage_toggle` | Stage ou unstage da linha selecionada | `Ctrl+Space` |
| `status` | `open_diff` | Abrir o diff do ficheiro selecionado | `d` |
| `status` | `expand_all` | Expandir todas as pastas | `=` |
| `status` | `collapse_all_folders` | Recolher todas as pastas | `-` |
| `status` | `expand_below` | Expandir a pasta e as suas subpastas | `*`, `Shift+Right` |
| `status` | `collapse_below` | Recolher a pasta e as suas subpastas | `_`, `Shift+Left` |
| `status` | `scroll_left` | Scroll the Pane Left | `Alt+Left` |
| `status` | `scroll_right` | Scroll the Pane Right | `Alt+Right` |
| `home` | `favorite` | Marcar ou desmarcar o repositório como favorito | `f`, `*` |
| `home` | `close_repo` | Fechar, remover ou desmarcar a linha | `x`, `Delete` |
| `home` | `copy_path` | Copiar o caminho do repositório | `y` |
| `home` | `open_in` | Abrir o repositório na ferramenta predefinida | `o` |
| `home` | `browse` | Procurar um repositório | `b` |
| `home` | `clone` | Clonar um repositório | `c` |
| `home` | `init` | Inicializar um repositório | `i` |
| `home` | `collapse_all` | Recolher todas as secções | `-` |
| `home` | `expand_all` | Expandir todas as secções | `+`, `=` |
| `home` | `search` | Pesquisar nos repositórios | `/` |
| `home` | `workspace_new` | Nova área de trabalho | `n` |
| `home` | `workspace_edit` | Editar a área de trabalho | `e` |
| `home` | `workspace_from_tabs` | Nova área a partir dos separadores abertos | `a` |
| `home` | `workspace_fetch` | Fetch tudo na área de trabalho | `Shift+F` |
| `home` | `workspace_pull` | Pull tudo na área de trabalho | `Shift+P` |
| `home` | `workspace_clone_missing` | Clonar os repositórios em falta | `m` |
| `home` | `workspace_locate` | Localizar o repositório em falta | `l` |
| `refs` | `hosting_filter` | Pull requests / issues: escolher o filtro | `v` |
| `refs` | `hosting_edit` | Pull request: editar no host | `e` |
| `refs` | `new_pull_request` | Pull requests: criar uma | `n` |
| `refs` | `add_remote` | Remotos: adicionar um remoto | `+` |
| `refs` | `remotes` | Branch: editar os seus remotos | `u` |
| `diff` | `close` | Fechar o diff | `q` |
| `diff` | `edit` | Editar no diretório de trabalho | `e` |
| `diff` | `lfs_raw` | LFS: cartão ou ponteiro bruto | `r` |
| `diff` | `copy` | Copiar a seleção ou a linha | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage ou unstage do hunk | `Space` |
| `diff` | `stage_lines` | Stage ou unstage das linhas selecionadas | `Enter` |
| `diff` | `discard_hunk` | Descartar o hunk | `x` |
| `diff` | `discard_lines` | Descartar as linhas selecionadas | `Shift+X` |
| `diff` | `select_lines` | Iniciar ou abandonar uma seleção de linhas | `v` |
| `diff` | `next_hunk` | Próximo hunk | `n` |
| `diff` | `prev_hunk` | Hunk anterior | `p` |
| `diff` | `toggle_layout` | Layout: unificado ou lado a lado | `m` |
| `diff` | `toggle_hunks_only` | Só hunks ou ficheiro todo | `Shift+H` |
| `diff` | `toggle_minimap` | Alternar o minimapa | `Shift+M` |
| `diff` | `choose_encoding` | Escolher a codificação | `Shift+C` |
| `diff` | `toggle_wrap` | Alternar quebra de linha | `w` |
| `diff` | `toggle_whitespace` | Mostrar ou ocultar alterações só de espaços em branco | `s` |
| `diff` | `merge_editor` | Abrir o editor de merge (ficheiro em conflito) | `Shift+E` |
| `file_insight` | `close` | Fechar a vista do ficheiro | `q` |
| `file_insight` | `edit` | Editar no diretório de trabalho | `e` |
| `file_insight` | `copy` | Copiar a seleção ou a linha | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Ir para a última linha | `Shift+G` |
| `file_insight` | `open_commit` | Ir para o commit da linha | `Enter` |
| `history` | `show_in_log` | Mostrar no Log | `Enter` |
| `history` | `copy_sha` | Copiar o SHA do commit | `y` |
| `history` | `copy_summary` | Copiar o resumo | `Shift+Y` |
| `history` | `show_diff` | Mostrar o diff deste commit | `d`, `Alt+1` |
| `history` | `open_file` | Abrir o ficheiro neste commit | `o`, `Alt+2` |
| `history` | `blame` | Blame neste commit | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Comparar com a árvore de trabalho | `c` |
| `history` | `restore` | Restaurar esta versão | `Shift+R` |
| `history` | `range_up` | Estender a seleção para cima | `Shift+Up` |
| `history` | `range_down` | Estender a seleção para baixo | `Shift+Down` |
| `history` | `switch_focus` | Alternar entre a lista e o diff | `Tab` |
| `history` | `widen_list` | Alargar a lista | `Alt+Right` |
| `history` | `narrow_list` | Estreitar a lista | `Alt+Left` |
| `history` | `reset_split` | Repor a largura da lista | `Alt+0` |
| `commit_column` | `scroll_left` | Deslocar a coluna para a esquerda | `Shift+Left` |
| `commit_column` | `scroll_right` | Deslocar a coluna para a direita | `Shift+Right` |
| `commit_column` | `next_part` | Dimensionar a parte seguinte | `Tab` |
| `commit_column` | `move_left` | Mover a parte para a esquerda | `<` |
| `commit_column` | `move_right` | Mover a parte para a direita | `>` |
| `commit_column` | `wider_part` | Alargar a parte | `]` |
| `commit_column` | `narrower_part` | Estreitar a parte | `[` |
| `commit_column` | `reset_widths` | Repor as larguras das partes | `0` |
| `commit_column` | `fit` | Ajustar a coluna ao conteúdo | `=` |
| `commit_column` | `fit_all` | Ajustar todas as colunas ao conteúdo | `+` |
| `file_insight` | `blame_card` | Blame: mostrar ou ocultar o cartão de detalhes | `i` |
| `file_insight` | `blame_next` | Blame: hunk seguinte | `n` |
| `file_insight` | `blame_prev` | Blame: hunk anterior | `p` |
| `file_insight` | `blame_next_same` | Blame: hunk seguinte do mesmo commit | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: hunk anterior do mesmo commit | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: ir para a linha | `g` |
| `file_insight` | `blame_copy_sha` | Blame: copiar o SHA do commit | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: diff do commit para este ficheiro | `d` |
| `file_insight` | `blame_parent` | Blame: blame no pai do commit | `b` |
| `file_insight` | `blame_history` | Blame: abrir o histórico do ficheiro | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: alargar a coluna de anotações | `>` |
| `file_insight` | `blame_narrower` | Blame: estreitar a coluna de anotações | `<` |
| `image` | `zoom_in` | Aumentar a imagem | `+`, `=` |
| `image` | `zoom_out` | Diminuir a imagem | `-` |
| `image` | `fit` | Ajustar a imagem à vista | `0` |
| `image` | `actual_size` | Mostrar a imagem em tamanho real | `1` |
| `image` | `flip_side` | Mostrar o outro lado (antes / depois) | `Tab` |
| `image` | `widen_before` | Alargar o painel Antes | `Alt+Right` |
| `image` | `widen_after` | Alargar o painel Depois | `Alt+Left` |
| `image` | `reset_split` | Repor a divisão antes / depois | `Alt+0` |
| `image` | `edit_in` | Abrir com... | `Shift+O` |
| `image` | `edit_default` | Abrir na aplicação predefinida | `o` |
| `editor` | `save` | Guardar | `Ctrl+S` |
| `editor` | `undo` | Desfazer | `Ctrl+Z` |
| `editor` | `redo` | Refazer | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Pesquisar | `Ctrl+F` |
| `editor` | `find_next` | Pesquisar próximo (Shift: anterior) | `F3` |
| `editor` | `paste` | Colar | `Ctrl+V` |
| `editor` | `toggle_minimap` | Alternar o minimapa | `Alt+M` |
| `editor_prompt` | `save` | Alterações não guardadas: guardar e fechar | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Alterações não guardadas: descartar e fechar | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Alterações não guardadas: continuar a editar | `c`, `Shift+C` |
| `merge` | `next_conflict` | Próximo conflito | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Conflito anterior | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Próximo ficheiro em conflito | `Alt+PageDown` |
| `merge` | `prev_file` | Ficheiro em conflito anterior | `Alt+PageUp` |
| `merge` | `next_pane` | Focar o próximo painel | `F6` |
| `merge` | `prev_pane` | Focar o painel anterior | `Shift+F6` |
| `merge` | `toggle_panes` | Mostrar ou ocultar os painéis laterais (Refs e Status) | `F9` |
| `merge` | `toggle_base` | Mostrar ou ocultar o painel Base | `Alt+D` |
| `merge` | `toggle_full_file` | Alternar diff do ficheiro todo | `Alt+F` |
| `merge` | `undo` | Desfazer (escolhas e escrita) | `Ctrl+Z` |
| `merge` | `redo` | Refazer (escolhas e escrita) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Alargar o painel Atual (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Alargar o painel Recebido (B) | `Alt+Left` |
| `merge` | `reset_panes` | Repor os tamanhos dos painéis | `Alt+0` |
| `merge` | `save` | Guardar sem marcar como resolvido | `Alt+S` |
| `merge` | `save_resolve` | Guardar e marcar como resolvido | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Sugerir uma resolução (IA) | `Alt+I` |
| `merge` | `unpick` | Desfazer a escolha deste conflito (não resolvido) | `Alt+U` |
| `merge` | `reset_all` | Devolver todo o ficheiro aos conflitos originais | `Alt+R` |
| `merge` | `focus_a` | Ir ao painel Atual (A) | `Alt+1` |
| `merge` | `focus_b` | Ir ao painel Recebido (B) | `Alt+2` |
| `merge` | `focus_output` | Ir ao painel Resultado | `Alt+3` |
| `merge` | `focus_base` | Ir ao painel Base | `Alt+4` |
| `merge_sides` | `toggle_line` | Escolher ou largar a linha | `Space` |
| `merge_sides` | `toggle_chunk` | Escolher ou largar o excerto todo | `Enter` |
| `merge_sides` | `take_a` | Aceitar só o Atual (A) | `a` |
| `merge_sides` | `take_b` | Aceitar só o Recebido (B) | `b` |
| `merge_sides` | `take_both` | Aceitar os dois, A e depois B | `c` |
| `merge_sides` | `take_both_reversed` | Aceitar os dois, B e depois A | `Shift+C` |
| `merge_sides` | `clear` | Não aceitar nada neste conflito | `x` |
| `merge_sides` | `unpick` | Desfazer a escolha deste conflito (não resolvido) | `u` |
| `merge_sides` | `reset_all` | Devolver todo o ficheiro aos conflitos originais | `Shift+U` |
| `merge_sides` | `swap_order` | Trocar a ordem dos dois lados | `s` |
| `merge_sides` | `next_conflict` | Próximo conflito | `n` |
| `merge_sides` | `prev_conflict` | Conflito anterior | `Shift+N` |
| `merge_sides` | `take_all_a` | Aceitar todo o Atual (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Aceitar todo o Recebido (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Mostrar ou ocultar o painel Base | `d` |
| `merge_sides` | `scroll_left` | Deslocar os painéis para a esquerda | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Deslocar os painéis para a direita | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Alternar diff do ficheiro todo | `f` |
| `merge_sides` | `cycle_height` | Alternar a altura do resultado | `z` |
| `merge_card` | `take_a` | Manter A (Atual) como está | `a` |
| `merge_card` | `take_b` | Aceitar B (Recebido) como está | `b` |
| `merge_card` | `take_base` | Aceitar a Base (ancestral comum) | `o` |
| `merge_card` | `keep` | Manter o ficheiro que sobrou | `k` |
| `merge_card` | `delete` | Aceitar a eliminação | `Shift+D` |
| `merge_card` | `external_tool` | Abrir na ferramenta de merge externa | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (parar para amend) | `e` |
| `rebase` | `autosquash` | Alternar autosquash | `a` |
| `rebase` | `start` | Iniciar o plano | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Repor o plano | `Ctrl+R` |
| `rebase` | `split` | Dividir o commit da linha | `t` |
| `rebase` | `move_up` | Mover a linha para cima | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Mover a linha para baixo | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Criar os commits | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Alternar a seleção por linhas | `v` |
| `split` | `move_up` | Mover o commit para cima | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Mover o commit para baixo | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Repor a divisão | `Shift+R` |
| `rewrite` | `preview` | Pré-visualizar a reescrita | `F5` |
| `rewrite` | `apply` | Aplicar a reescrita | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Cancelar a pré-visualização | `Ctrl+X` |
| `rewrite` | `add_rule` | Adicionar uma regra | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Ligar ou desligar a regra | `Space` |
| `rewrite` | `move_up` | Mover a regra para cima | `Alt+Up` |
| `rewrite` | `move_down` | Mover a regra para baixo | `Alt+Down` |
| `rewrite` | `delete_rule` | Eliminar a regra | `d`, `Delete` |
| `rewrite` | `restore_rule` | Restaurar a regra eliminada | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Duplicar a regra | `Ctrl+D` |
| `rewrite` | `next_tab` | Separador seguinte da pré-visualização | `Alt+Right` |
| `rewrite` | `prev_tab` | Separador anterior da pré-visualização | `Alt+Left` |
| `rewrite` | `save_shared` | Guardar a receita no repositório | `Alt+S` |
| `rewrite` | `save_mine` | Guardar a receita só para mim | `Alt+M` |
| `rewrite` | `load` | Carregar uma receita | `Alt+L` |
| `rewrite` | `name` | Renomear a receita | `F2` |
| `rewrite` | `protected_all` | Incluir todas as branches protegidas | `Alt+A` |
| `rewrite` | `protected_none` | Deixar de fora todas as branches protegidas | `Alt+U` |
| `rewrite` | `check_tools` | Verificar as ferramentas de novo | `F6` |
| `rewrite` | `analyze` | Analisar os tamanhos do histórico | `F7` |
| `rewrite` | `host_howto` | Explicar a proteção de branches do alojamento | `Alt+H` |
| `rewrite` | `ai_rules` | Criar regras de reescrita com IA | `Alt+I` |
| `reword` | `apply` | Aplicar a nova mensagem | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Modificar o histórico mais a fundo | `Alt+H` |
| `composer` | `commit` | Commit pelo compositor | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Inserir uma referência de issue | `Alt+R` |
| `md_editor` | `preview` | Editor de Markdown: alternar entre Escrever e Prévia | `Alt+P` |
| `md_editor` | `split` | Editor de Markdown: alternar a vista lado a lado | `Alt+L` |
| `md_editor` | `submit` | Editor de Markdown: enviar o texto | `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` |
| `md_editor` | `ai` | Editor de Markdown: escrever com IA | `Alt+E` |
| `dialog` | `browse` | Procurar uma pasta | `Ctrl+B` |
| `dialog` | `sources` | Ir para as origens do clone | `Alt+S` |
| `dialog` | `submit` | Enviar o formulário de pull request | `Ctrl+S` |
| `dialog` | `web` | Abrir o formulário web do fornecedor | `Alt+W` |
| `folder_picker` | `choose` | Escolher a pasta | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Nova pasta | `Alt+N` |
| `folder_picker` | `filter` | Filtrar as entradas da pasta | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Mostrar ou ocultar pastas vizinhas | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Mostrar ou ocultar pastas ocultas | `Alt+H` |
| `preferences` | `search` | Pesquisar nas definições | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Adicionar um item à lista | `Insert` |
| `preferences` | `list_duplicate` | Duplicar o item selecionado da lista | `Ctrl+D` |
| `preferences` | `list_remove` | Remover o item selecionado da lista | `Delete` |
| `preferences` | `list_move_up` | Mover para cima o item selecionado da lista | `Alt+Up` |
| `preferences` | `list_move_down` | Mover para baixo o item selecionado da lista | `Alt+Down` |
| `preferences` | `prev_tab` | Separador anterior da página de definições | `Alt+Left` |
| `preferences` | `next_tab` | Separador seguinte da página de definições | `Alt+Right` |
| `ai_popup` | `copy` | Copiar o resultado | `c` |
| `activity_log` | `scroll_left` | Deslocar o registo de atividade para a esquerda | `Left` |
| `activity_log` | `scroll_right` | Deslocar o registo de atividade para a direita | `Right` |
| `integrations` | `add` | Adicionar um perfil | `a` |
| `integrations` | `edit` | Editar o perfil selecionado | `e` |
| `integrations` | `open_token` | Abrir a página do token | `o` |
| `integrations` | `remove` | Remover o perfil selecionado | `d`, `Delete` |
| `prompt` | `ai_stash_message` | IA: gerar mensagem de stash | `Ctrl+G` |
<!-- keymap:end -->

### Teclas que o terminal não consegue enviar, e o protocolo de teclado do kitty

Um terminal legado não consegue enviar alguns acordes: `Ctrl+,`, `Ctrl+Shift+N`,
`Ctrl+Enter` e `Ctrl+<dígito>`. Não envia nada, ou envia outra tecla: por
exemplo, `Ctrl+Shift+N` chega como `Ctrl+N`, `Ctrl+I` como `Tab` e `Ctrl+M` como
`Enter`. A página de Definições avisa sobre uma atribuição assim ("este terminal não envia
...").

Os terminais com o **protocolo de teclado do kitty** conseguem enviar estes acordes. Exemplos:
kitty, WezTerm, Ghostty, foot, Alacritty 0.13 e posterior, e zellij (o tmux não
responde: veja [Multiplexadores](multiplexers.md#tmux)). O GitPoulpi usa o protocolo por predefinição:

- No arranque, o GitPoulpi pergunta ao terminal e espera no máximo cerca de 100 ms pela
  resposta. Se o terminal não responder, as teclas legadas mantêm-se.
- Se o terminal responder, o GitPoulpi ativa o protocolo. Desativa-o
  de novo quando termina, falha ou deixa de funcionar.
- Os avisos da página de Definições seguem o protocolo: quando está ativo, os
  acordes acima podem ser enviados.
- **Definições > Teclado e rato > Protocolo de teclado do Kitty** (`ui.kitty_keyboard`)
  definido como `off` nunca altera o modo de teclado do terminal. Use-o se um terminal
  ou um multiplexador se portar mal. É preciso reiniciar.

As consolas do Windows comunicam elas próprias os modificadores e não precisam do protocolo.

## Paleta de comandos (`Ctrl+P`)

A paleta lista **todos** os comandos da aplicação, por isso chega a tudo
sem um atalho. A lista completa está [abaixo](#todos-os-comandos-da-paleta).

| Tecla | Ação |
|---|---|
| `Ctrl+P` | Abrir a paleta |
| *(escrever)* | Filtrar a lista. Uma correspondência exata a seguir ao prefixo `Categoria: ` vem primeiro, depois uma correspondência em qualquer ponto da etiqueta, e depois uma correspondência aproximada |
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Mover o realce (consulte [Listas de popup filtráveis](#listas-de-popup-filtráveis)) |
| `Enter`, ou um clique duplo | Executar o comando realçado |
| `Esc` | Fechar |

`j` e `k` são escritos no filtro. Não movem o realce.

Cada linha mostra o seu atalho à direita. Um comando que não se aplica agora
(por exemplo, nenhuma ref selecionada) fica na lista, a cinzento, e o motivo
aparece por baixo da lista. As etiquetas começam por uma categoria, como `Navegação:`,
`Refs:`, `Git:`, `Mostrar:`, `Diff:` ou `Aplicação:`. Escreva uma categoria para ver todo o
seu grupo.

### Uma tecla para qualquer comando

Cada linha da paleta termina com um botão de tecla (`[k]`), e `Alt+K` faz o mesmo na linha realçada: abre **Definições > Teclado e rato** filtrado pelo atalho desse comando, pronto a reatribuir. Um comando sem atalho próprio recebe um aqui: aparece em **Comandos da paleta**, sem tecla por predefinição, e a tecla que lhe der executa o comando exatamente como escolhê-lo na paleta (um comando que não pode correr agora diz porquê). A linha passa a mostrar a sua tecla.

No `config.toml` é a tabela `[keybindings.palette]`, com o nome do comando (`toggle_minimap = "Ctrl+Alt+M"`). Uma tecla que colide com outra ação aparece a vermelho, e `c` na linha lista todas as ações com que colide.

A faixa de ajuda abaixo da lista indica esta tecla (`Alt+K` por predefinição). É a ação `palette.configure_shortcut`, por isso pode ser reatribuída como qualquer outra.

A paleta e os seletores (tema, idioma, separador, pesquisa de workspaces) movem-se e redimensionam-se como uma janela: arraste a margem superior, prima `Ctrl+N` e depois `m` (como sobre qualquer janela), ou prima `Alt+G`. `Alt+G` é a ação `palette.arrange` ("Janela: mover ou redimensionar a janela da frente"): uma via extra, válida sobre qualquer janela e popup, para iniciar o modo de arranjo na que está à frente (as linhas da paleta `Janela: mover ou redimensionar` e `Janela: repor posição e tamanho` atuam sobre a própria paleta enquanto ela está aberta). As setas movem-na, `Shift`+setas redimensionam-na, `0` repõe-na, `Enter` ou `Esc` saem do modo e o popup fica aberto. `ui.dialog_geometry` decide se o lugar é mantido.

`Ctrl+N` é também aqui o prefixo de navegação: mostra um selo na caixa de filtro, na primeira linha da lista e na cruz de fechar, e a tecla seguinte prime esse controlo, ou `m` arranja o popup. Já não desce o realce, e `Ctrl+P` não o sobe: só as setas, `PageUp` / `PageDown` e `Home` / `End` movem uma lista com campo de filtro.

### Todos os comandos da paleta

Gerada a partir da aplicação (um teste mantém-na atualizada). Os comandos de IA só aparecem
enquanto a IA está ativa; os restantes ficam a cinzento, com o motivo, quando não se aplicam.
As teclas são as predefinidas.

<!-- palette:begin -->
| Comando | Tecla predefinida |
|---|---|
| Navegação: focar o painel Refs | `Ctrl+N <badge>` |
| Navegação: focar o painel Log | `Ctrl+N <badge>` |
| Navegação: focar o painel Status | `Ctrl+N <badge>` |
| Navegação: focar o filtro de Refs | `Ctrl+N <badge>` |
| Navegação: focar Refs > Local | `Ctrl+N <badge>` |
| Navegação: focar Refs > Remoto | `Ctrl+N <badge>` |
| Navegação: focar Refs > Worktrees | `Ctrl+N <badge>` |
| Navegação: focar Refs > Submódulos | `Ctrl+N <badge>` |
| Navegação: focar Refs > Stashes | `Ctrl+N <badge>` |
| Navegação: focar Refs > Tags | `Ctrl+N <badge>` |
| Painel esquerdo: mostrar/ocultar secções... |  |
| Painel esquerdo: repor as secções predefinidas |  |
| Painel esquerdo: usar estas secções em todos os repositórios |  |
| Painel esquerdo: mostrar/ocultar branches locais |  |
| Painel esquerdo: mostrar/ocultar remotos |  |
| Painel esquerdo: mostrar/ocultar worktrees |  |
| Painel esquerdo: mostrar a secção Worktrees |  |
| Painel esquerdo: mostrar/ocultar stashes |  |
| Painel esquerdo: mostrar/ocultar tags |  |
| Painel esquerdo: mostrar/ocultar pull requests |  |
| Painel esquerdo: mostrar/ocultar issues |  |
| Painel esquerdo: mostrar/ocultar submódulos |  |
| Navegação: focar Status > Unstaged | `Ctrl+N <badge>` |
| Navegação: focar Status > Staged | `Ctrl+N <badge>` |
| Navegação: levar o foco ao próximo painel | `Tab` |
| Navegação: levar o foco ao painel anterior | `Shift+Tab` |
| Navegação: pesquisar commits... | `f` |
| Navegação: recolher/expandir o painel Refs | `Ctrl+N H` |
| Navegação: Alargar o painel Refs | `Ctrl+N Right` |
| Navegação: Estreitar o painel Refs | `Ctrl+N Left` |
| Navegação: Alargar o painel Status | `Ctrl+N Shift+Left` |
| Navegação: Estreitar o painel Status | `Ctrl+N Shift+Right` |
| Navegação: recolher/expandir o painel Status | `Ctrl+N L` |
| Navegação: aumentar a secção Refs | `Ctrl+N +` |
| Navegação: diminuir a secção Refs | `Ctrl+N -` |
| Navegação: repor os tamanhos das secções Refs |  |
| Navegação: repor a disposição |  |
| Detalhes do commit: aumentar a mensagem | `Ctrl+N ]` |
| Detalhes do commit: diminuir a mensagem | `Ctrl+N [` |
| Detalhes do commit: repor o tamanho da mensagem |  |
| Detalhes do commit: deslocar a mensagem para baixo |  |
| Detalhes do commit: deslocar a mensagem para cima |  |
| Detalhes do commit: abrir ligação na mensagem... |  |
| Ficheiros: alternar a ordem dos ficheiros do commit (A-Z / Z-A) |  |
| Refs: checkout da ref selecionada | `clique duplo` |
| Refs: copiar nome da ref |  |
| Refs: copiar ref completa (refs/heads/...) |  |
| Refs: copiar SHA de destino da ref |  |
| Refs: Merge da branch selecionada na atual |  |
| Refs: Rebase da branch atual sobre a selecionada | `Ctrl+G r` |
| Refs: largar a ref selecionada na branch em checkout... |  |
| Refs: Marcar como origem do largar | `m` |
| Refs: Largar aqui a ref marcada... | `M` |
| Refs: renomear a branch selecionada... |  |
| Refs: fast-forward da branch selecionada até à atual |  |
| Refs: fast-forward da branch atual até à selecionada |  |
| Refs: criar worktree a partir da branch ou tag selecionada... |  |
| Refs: Reflog da branch selecionada... |  |
| Refs: iniciar pull request a partir da branch selecionada... |  |
| Refs: abrir worktree num separador | `Enter / clique duplo` |
| Refs: abrir worktree em... |  |
| Refs: merge da branch do worktree em... |  |
| Refs: remover worktree... |  |
| Refs: criar worktree... | `+` |
| Refs: criar worktree a partir da branch atual |  |
| Refs: criar worktree escolhendo uma branch |  |
| Refs: atualizar submódulo (selecionado) |  |
| Refs: inicializar submódulo (selecionado) |  |
| Refs: abrir submódulo num separador | `Enter / clique duplo` |
| Git: atualizar todos os submódulos |  |
| Git: criar worktree no commit selecionado... |  |
| Git: criar patch a partir do commit selecionado... |  |
| Git: criar patch do commit selecionado até HEAD... |  |
| Git: criar patch das alterações da árvore de trabalho... |  |
| Reescrever: purgar o histórico antigo... |  |
| Git: aplicar patch... |  |
| Reescrever: painel do histórico | `Ctrl+G H` |
| Reescrever: substituir um autor... |  |
| Reescrever: anonimizar autores... |  |
| Reescrever: aplicar o .mailmap ao histórico... |  |
| Reescrever: Merge de identidades (.mailmap)... |  |
| Reescrever: deslocar as datas dos commits... |  |
| Reescrever: mudar o fuso horário dos commits... |  |
| Reescrever: manter commits dentro de um horário... |  |
| Reescrever: data do committer igual à data do autor |  |
| Reescrever: procurar e substituir nas mensagens... |  |
| Reescrever: apagar linhas das mensagens... |  |
| Reescrever: adicionar ou remover trailers... |  |
| Reescrever: remover assinaturas... |  |
| Reescrever: corrigir a codificação das mensagens... |  |
| Reescrever: procurar e substituir nos ficheiros... |  |
| Reescrever: remover ficheiros do histórico... |  |
| Reescrever: remover ficheiros grandes do histórico... |  |
| Reescrever: mover ou renomear caminhos... |  |
| Reescrever: migrar ficheiros para o LFS... |  |
| Reescrever: trazer ficheiros de volta do LFS... |  |
| Reescrever: converter fins de linha... |  |
| Reescrever: remover o bit de execução... |  |
| Reescrever: converter a codificação dos ficheiros... |  |
| Reescrever: executar um script... |  |
| ✦ Reescrever: escrever um script com IA... |  |
| ✦ Reescrever: criar regras com IA... |  |
| Reescrever: do commit selecionado até HEAD... |  |
| Reescrever: commits selecionados... |  |
| LFS: inicializar neste repositório |  |
| LFS: acompanhar padrão... |  |
| LFS: parar de acompanhar padrão... |  |
| LFS: fetch de objetos |  |
| LFS: abrir menu | `Ctrl+G l` |
| LFS: Fetch de todos os objetos | `Ctrl+G F` |
| LFS: checkout de todos os ficheiros | `Ctrl+G K` |
| LFS: Push de todos os objetos | `Ctrl+G J` |
| LFS: pull de objetos | `Ctrl+G L` |
| LFS: push de objetos |  |
| LFS: limpar objetos locais | `Ctrl+G V` |
| LFS: mostrar bloqueios |  |
| LFS: abrir página de instalação |  |
| LFS: alternar ponteiro bruto no diff | `r` |
| LFS: acompanhar o ficheiro selecionado |  |
| LFS: acompanhar a extensão do ficheiro selecionado |  |
| LFS: bloquear o ficheiro selecionado |  |
| LFS: desbloquear o ficheiro selecionado |  |
| LFS: forçar o desbloqueio do ficheiro selecionado |  |
| Refs: eliminar a branch ou tag selecionada... | `Ctrl+G d` |
| Refs: eliminar a branch selecionada do seu remoto... |  |
| Refs: eliminar a branch selecionada localmente e no remoto... |  |
| Refs: copiar a ligação da branch selecionada no remoto |  |
| Git: copiar a ligação do commit selecionado no remoto |  |
| Git: tag anotada no commit selecionado... |  |
| Refs: push da tag selecionada para o remoto predefinido |  |
| Refs: eliminar a tag selecionada do seu remoto... |  |
| Git: Fetch da Tag selecionada |  |
| Git: Use the Remote's Version of the Selected Tag |  |
| Git: Force Push the Selected Tag to Its Remote |  |
| Refs: anotar a tag selecionada... |  |
| Refs: Move Selected Tag to Selected Commit... |  |
| Refs: alternar ocultar no grafo |  |
| Refs: mostrar no grafo | `g` |
| Refs: alternar solo na ref selecionada | `o` |
| Refs: parar o solo (limpa cada ref em solo) | `O` |
| Refs: mostrar todas as refs ocultas | `H` |
| Refs: alternar ocultar na pasta/remoto selecionado |  |
| Refs: alternar solo na pasta/remoto selecionado | `o` |
| Refs: filtrar por nome... | `/` |
| Refs: limpar filtro de nome | `Esc` |
| Refs: expandir todas as pastas | `=` |
| Refs: recolher todas as pastas | `-` |
| Refs: expandir a pasta selecionada e as suas subpastas | `*` |
| Refs: recolher a pasta selecionada e as suas subpastas | `_` |
| View: Scroll the Pane Left | `Alt+Left` |
| View: Scroll the Pane Right | `Alt+Right` |
| Refs: próxima secção | `l` |
| Refs: secção anterior | `h` |
| Refs: expandir ou recolher a secção ativa | `Enter` |
| Refs: alternar a hierarquia de branches | `y` |
| Git: stage do ficheiro selecionado | `Ctrl+G s` |
| Git: unstage do ficheiro selecionado | `Ctrl+G u` |
| Git: stage de todas as alterações | `Ctrl+G a` |
| Alterações: descartar o ficheiro selecionado... |  |
| Alterações: ignorar o ficheiro selecionado |  |
| Alterações: ignorar a extensão do ficheiro selecionado |  |
| Alterações: tratar o ficheiro selecionado como binário... |  |
| Alterações: tratar o ficheiro selecionado como texto... |  |
| Alterações: tratar a extensão do ficheiro selecionado como binária... |  |
| Alterações: tratar a extensão do ficheiro selecionado como texto... |  |
| Alterações: stash do ficheiro selecionado |  |
| Alterações: histórico do ficheiro selecionado |  |
| Alterações: blame do ficheiro selecionado |  |
| Alterações: abrir o ficheiro selecionado na ferramenta de diff externa |  |
| Alterações: abrir o ficheiro selecionado no editor externo |  |
| Alterações: abrir o ficheiro selecionado no programa predefinido |  |
| Alterações: mostrar o ficheiro selecionado na pasta |  |
| Alterações: copiar o caminho do ficheiro selecionado |  |
| Alterações: criar patch do ficheiro selecionado... |  |
| Alterações: editar o ficheiro selecionado |  |
| Alterações: eliminar o ficheiro selecionado... |  |
| Alterações: abrir o menu da pasta selecionada |  |
| Alterações: criar ficheiro |  |
| Alterações: descartar todas as alterações... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: fazer commit das alterações em stage agora | `Ctrl+G Enter` |
| Git: editar mensagem do commit WIP (no Log) | `c no Log` |
| Git: Fetch de todos os remotos | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward se possível) |  |
| Git: Pull (só fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: Pull de todas as branches | `Ctrl+G Ctrl+P` |
| Git: Pull de todas as branches (fast-forward se possível) |  |
| Git: Pull de todas as branches (só fast-forward) |  |
| Git: Pull de todas as branches (rebase) |  |
| Git: opções de pull... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push para um remoto... |  |
| Git: Push para remotos selecionados... |  |
| Git: Push para todos os destinos |  |
| Git: Pull de todas as fontes |  |
| Git: Pull de um remoto... |  |
| Git: Mostrar o push em grupo |  |
| Git: Continuar o push em grupo |  |
| Git: Abortar o push em grupo |  |
| Git: Repetir os remotos falhados |  |
| Git: force push (com lease)... |  |
| Git: Remotos da branch... |  |
| Remoto: adicionar remoto... |  |
| Remoto: remover remoto... |  |
| Remoto: editar remoto... |  |
| Remoto: definir remoto predefinido... |  |
| Remoto: copiar nome do remoto |  |
| Remoto: copiar a ligação do remoto |  |
| Remoto: fetch do remoto... |  |
| Remoto: ações do remoto selecionado... | `.` |
| Repositório: checkout parcial... |  |
| Repositório: SSH e credenciais... |  |
| Repositório: testar ligação |  |
| Repositório: usar credenciais predefinidas |  |
| Flow: abrir menu | `Ctrl+G W` |
| Flow: iniciar branch... |  |
| Flow: nova issue... |  |
| Flow: assumir branch do host... |  |
| Flow: concluir a branch atual... |  |
| Flow: sincronizar com o pai... |  |
| Flow: promover... |  |
| Flow: definir o pai... |  |
| Flow: continuar ou abortar a ação interrompida... |  |
| Flow: alternar o agrupamento de branches |  |
| Flow: definições do fluxo de trabalho... |  |
| Remoto: ocultar remoto no grafo... |  |
| Remoto: mostrar remoto no grafo... |  |
| Git: nova branch em HEAD... | `Ctrl+G b` |
| Git: nova tag em HEAD... | `Ctrl+G t` |
| Git: Push de todas as tags não enviadas | `Ctrl+G U` |
| Git: Stash de todas as alterações | `Ctrl+G T` |
| Git: Stash de todas as alterações com mensagem... | `Ctrl+G m` |
| Git: pop do stash | `Ctrl+G o` |
| Git: apply do stash |  |
| Git: drop do stash... |  |
| Git: Cherry-pick do commit selecionado em HEAD | `Ctrl+G y` |
| Git: checkout do commit selecionado (HEAD desanexado) |  |
| Git: criar branch no commit selecionado... |  |
| Git: criar tag no commit selecionado... |  |
| Commit: copiar mensagem |  |
| Commit: copiar nome do autor |  |
| Commit: copiar e-mail do autor |  |
| Commit: copiar data |  |
| Commit: copiar SHA curto |  |
| Commit: copiar SHA completo |  |
| Git: reverter o commit selecionado... | `Ctrl+G v` |
| Git: editar mensagem do commit... | `Ctrl+G e` |
| Commit: alterar... | `Ctrl+G E` |
| Commit: alterar autor... | `Ctrl+G A` |
| Commit: alterar resumo... |  |
| Commit: alterar descrição... |  |
| Git: dividir o commit selecionado... |  |
| Git: rebase interativo a partir do commit selecionado... |  |
| Rebase: continuar |  |
| Rebase: saltar |  |
| Rebase: abortar |  |
| git am: continuar |  |
| git am: saltar este patch |  |
| git am: abortar |  |
| Repositório: remover index.lock obsoleto |  |
| Aviso: pop do auto-stash pendente |  |
| Aviso: concluir o auto-stash pendente (manter o resultado) |  |
| Aviso: mostrar o auto-stash pendente |  |
| Conflitos: mostrar os conflitos previstos |  |
| Aviso: descartar o auto-stash pendente... |  |
| Aviso: recomeçar... |  |
| Git: reset da branch para o commit selecionado (soft - mantém índice e árvore de trabalho) |  |
| Git: reset da branch para o commit selecionado (mixed - mantém a árvore de trabalho) | `Ctrl+G R` |
| Git: reset da branch para o commit selecionado (hard - descarta tudo)... |  |
| Git: resolver conflito com Ours (esta branch) |  |
| Git: resolver conflito com Theirs (recebido) |  |
| Git: resolver conflito com Base (ancestral comum) |  |
| Conflitos: abrir o editor de merge | `Enter num ficheiro em conflito` |
| Merge: próximo conflito | `F7` |
| Merge: conflito anterior | `Shift+F7` |
| Merge: próximo ficheiro em conflito | `Alt+PageDown` |
| Merge: ficheiro em conflito anterior | `Alt+PageUp` |
| Merge: aceitar excerto de A |  |
| Merge: aceitar excerto de B |  |
| Merge: aceitar o Atual neste conflito | `a` |
| Merge: aceitar o Recebido neste conflito | `b` |
| Merge: aceitar os dois, A e depois B | `c` |
| Merge: aceitar os dois, B e depois A | `C` |
| Merge: aceitar a Base neste conflito |  |
| Merge: limpar este conflito | `x` |
| Merge: trocar a ordem | `s` |
| Merge: aceitar todo o Atual | `A` |
| Merge: aceitar todo o Recebido | `B` |
| Merge: editar este conflito no resultado |  |
| Merge: mostrar ou ocultar os painéis laterais | `F9` |
| Merge: deslocar os painéis para a esquerda | `Shift+Left` |
| Merge: deslocar os painéis para a direita | `Shift+Right` |
| Merge: alternar o painel Base | `Alt+D` |
| Merge: alternar diff do ficheiro todo | `Alt+F` |
| Merge: desfazer | `Ctrl+Z` |
| Merge: refazer | `Ctrl+Y` |
| Merge: guardar sem marcar como resolvido | `Alt+S` |
| Merge: guardar e marcar como resolvido | `Ctrl+S` |
| Merge: repor o resultado para o resultado do merge | `Alt+R` |
| Merge: alternar a altura do resultado | `z` |
| Merge: alargar o painel Atual | `Alt+Right` |
| Merge: alargar o painel Recebido | `Alt+Left` |
| Merge: repor os tamanhos dos painéis | `Alt+0` |
| Merge: desfazer a escolha deste conflito | `Alt+U` |
| Merge: ir ao painel Atual (A) | `Alt+1` |
| Merge: ir ao painel Recebido (B) | `Alt+2` |
| Merge: ir ao painel Resultado | `Alt+3` |
| Merge: ir ao painel Base | `Alt+4` |
| Merge: painel da esquerda (mesmo conflito) |  |
| Merge: painel da direita (mesmo conflito) |  |
| Merge: focar o próximo painel | `F6` |
| Merge: focar o painel anterior | `Shift+F6` |
| Merge: continuar a partir do ficheiro em disco |  |
| Merge: recomeçar a partir do merge |  |
| Merge: desfazer resolução e reabrir |  |
| Merge: recarregar o ficheiro |  |
| Merge: usar o ficheiro Atual como está | `a` |
| Merge: usar o ficheiro Recebido como está | `b` |
| Merge: usar o ficheiro Base como está | `o` |
| Merge: manter o ficheiro que sobrou | `k` |
| Merge: aceitar a eliminação | `D` |
| Merge: abrir na ferramenta de merge externa | `m` |
| ✦ IA: sugerir resolução de conflito de merge | `Alt+I` |
| Merge: fechar o editor de merge |  |
| Git: abortar merge, cherry-pick ou revert |  |
| Git: saltar cherry-pick ou revert |  |
| Repositório: abrir ou mudar (gestão de repositórios)... | `Ctrl+O` |
| Repositório: procurar um repositório... | `b (separador de gestão)` |
| Repositório: criar novo (Init)... | `i (separador de gestão)` |
| Repositórios: abrir o selecionado | `Enter` |
| Repositórios: adicionar/remover o selecionado dos favoritos | `f` |
| Repositórios: fechar ou remover o selecionado | `x` |
| Repositórios: menu do repositório selecionado... | `.` |
| Repositórios: copiar o caminho do selecionado | `y` |
| Repositórios: abrir o selecionado na ferramenta predefinida | `o` |
| Repositórios: pesquisar | `/ / Ctrl+F` |
| Repositórios: recolher todas as secções | `-` |
| Repositórios: expandir todas as secções | `+` |
| Repositórios: fechar todos os separadores... |  |
| Repositórios: remover o selecionado da lista |  |
| Repositórios: remover todos os recentes |  |
| Área de trabalho: nova... | `n` |
| Área de trabalho: nova a partir dos separadores abertos... | `a` |
| Área de trabalho: mudar... | `Alt+W` |
| Área de trabalho: editar... | `e` |
| Área de trabalho: fechar (sem área de trabalho) |  |
| Área de trabalho: gerir... | `Alt+J` |
| Área de trabalho: fechar separador |  |
| Área de trabalho: localizar o repositório em falta... | `l` |
| Área de trabalho: eliminar... |  |
| Área de trabalho: fetch tudo | `F` |
| Área de trabalho: pull tudo | `P` |
| Área de trabalho: clonar os que faltam | `m` |
| Área de trabalho: adicionar o separador à área de trabalho |  |
| Área de trabalho: remover o separador da área de trabalho |  |
| Radar: abrir | `Alt+R` |
| Radar: atualizar | `F5` |
| Radar: pesquisar | `/` |
| Radar: abrir no navegador | `o` |
| Radar: abrir repositório | `Enter` |
| Radar: fazer checkout da branch | `c` |
| Radar: criar worktree... | `t` |
| Radar: mostrar no repositório | `s` |
| Radar: iniciar trabalho na issue... | `n` |
| Radar: afixar ou desafixar | `p` |
| Radar: adiar... | `z` |
| Radar: rever pedido... | `e` |
| Radar: abrir a conversa da issue | `i` |
| Radar: rever através da API |  |
| Radar: clonar e rever... |  |
| Radar: editar responsáveis... |  |
| Radar: editar revisores... |  |
| Radar: editar etiquetas... |  |
| Radar: definir marco... |  |
| Radar: escolher âmbito | `w` |
| Radar: escolher vista | `v` |
| Radar: escolher filtro | `f` |
| Radar: mostrar pedidos não clonados | `a` |
| Radar: mostrar rascunhos | `d` |
| Radar: ativar os pedidos de revisão da conta... | `T` |
| Radar: menu da linha... | `.` |
| Radar: escolher coluna | `C` |
| Radar: alargar coluna | `]` |
| Radar: estreitar coluna | `[` |
| Radar: repor larguras das colunas | `0` |
| Radar: Ajustar a coluna ao conteúdo | `=` |
| Radar: Ajustar todas as colunas ao conteúdo | `+` |
| Radar: voltar ao início | `Esc` |
| Radar: fechar separador |  |
| Separador: novo separador | `Alt+T` |
| Separador: novo... |  |
| Separador: menu de ações... | `Ctrl+T` |
| Separador: fechar outros separadores... |  |
| Separador: fechar separadores à direita... |  |
| Separador: adicionar aos/remover dos favoritos |  |
| Separador: copiar caminho do repositório |  |
| Separador: abrir no gestor de ficheiros |  |
| Separador: abrir no terminal |  |
| Separador: abrir em... | `Alt+O` |
| Tab: Terminal Here |  |
| ✦ Tab: Open AI Agent Here |  |
| Separador: abrir no predefinido | `Ctrl+Alt+O` |
| Repositório: alterar perfil... |  |
| Ajuda: acerca do GitPoulpi | `Ctrl+N a` |
| Ajuda: licenças e créditos |  |
| Ver: registo de atividade | `Ctrl+N A` |
| Ver: registo de atividade global | `Ctrl+N G` |
| Ajuda: notas de versão |  |
| Atualizações: mostrar atualização | `Ctrl+N u` |
| Atualizações: abrir aviso de segurança |  |
| Atualizações: copiar comando de atualização |  |
| Atualizações: executar comando de atualização num terminal |  |
| Atualizações: procurar atualizações agora |  |
| Atualizações: transferir |  |
| Atualizações: atualizar agora |  |
| Atualizações: reiniciar para atualizar |  |
| Atualizações: reverter para a versão anterior |  |
| Atualizações: ignorar esta versão |  |
| Notas de versão: fechar separador |  |
| Tarefas: mostrar as tarefas em segundo plano | `Ctrl+N J` |
| Tarefas: cancelar tudo o que for possível |  |
| Ver: copiar diagnóstico |  |
| Registo de atividade: mostrar ou ocultar o nível Erro |  |
| Registo de atividade: mostrar ou ocultar o nível Aviso |  |
| Registo de atividade: mostrar ou ocultar o nível Info |  |
| Registo de atividade: mostrar ou ocultar o nível Depuração |  |
| Registo de atividade: mostrar ou ocultar o nível Rasto |  |
| Registo de atividade: mostrar todos os níveis |  |
| Registo de atividade: não mostrar nenhum nível |  |
| Registo de atividade: limpar o registo da aplicação |  |
| Registo de atividade: escolher o âmbito |  |
| Registo de atividade: operações Git |  |
| Registo de atividade: registo da aplicação |  |
| Registo de atividade: mostrar a operação desta linha |  |
| Registo de atividade: operação falhada seguinte |  |
| Registo de atividade: operação falhada anterior |  |
| Registo de atividade: deslocar para a esquerda | `Left` |
| Registo de atividade: deslocar para a direita | `Right` |
| Registo de atividade: copiar o último erro |  |
| Registo de atividade: executar a ação sugerida |  |
| Registo de atividade: abrir o último erro |  |
| Separador: sair e cd aqui |  |
| Repositório: clonar... | `Ctrl+Shift+N` |
| Repositório: próximo separador | `Ctrl+PageDown / Ctrl+Tab` |
| Repositório: separador anterior | `Ctrl+PageUp / Ctrl+Shift+Tab` |
| Separador: mover para a esquerda |  |
| Separador: mover para a direita |  |
| Repositório: fechar separador | `Ctrl+W` |
| Repositório: selecionar separador... | `t` |
| Pesquisar: área de trabalho... | `Ctrl+Alt+F` |
| Editar: desfazer | `Ctrl+Z` |
| Editar: refazer | `Ctrl+Y` |
| Editar: esquecer a entrada de desfazer bloqueada |  |
| Editar: histórico de desfazer... | `Alt+Z` |
| Editar: Reflog... |  |
| Editar: recuperar commits perdidos... |  |
| Editar: restaurar a branch para onde a ação bloqueada a deixou | `Alt+Shift+Z` |
| Revisão: mostrar resumo |  |
| Revisão: mostrar ficheiros |  |
| Revisão: mostrar commits |  |
| Revisão: comentar a linha... | `c` |
| Revisão: comentar a conversa... | `C` |
| Revisão: responder à discussão... | `r` |
| Revisão: resolver ou reabrir a discussão | `x` |
| Revisão: voltar ao comentário no resumo | `Esc` |
| Revisão: recolher ou expandir a discussão | `z` |
| Revisão: editar comentário pendente... | `e` |
| Revisão: eliminar comentário pendente | `d` |
| Revisão: marcar ficheiro como visto | `v` |
| Revisão: próximo ficheiro | `]` |
| Revisão: ficheiro anterior | `[` |
| Revisão: concluir revisão... | `f` |
| Revisão: descartar revisão pendente... |  |
| Revisão: atualizar | `F5` |
| Revisão: abrir no navegador | `o` |
| Revisão: fazer checkout da branch | `b` |
| Revisão: fechar | `Esc` |
| Revisão: editar responsáveis... | `a` |
| Revisão: editar revisores... | `p` |
| Revisão: editar etiquetas... | `l` |
| Revisão: definir marco... | `m` |
| Revisão: escolher patch set... | `P` |
| Revisão: comparar patch sets... | `D` |
| Revisão: submeter alteração... | `U` |
| Revisão: abandonar alteração... | `Q` |
| Revisão: alternar layout inline / dividido | `V` |
| Revisão: alternar ficheiro todo / só os hunks | `H` |
| Revisão: alternar alterações só de espaços em branco | `S` |
| Revisão: alternar quebra de linha | `W` |
| Revisão: próximo hunk | `J` |
| Revisão: hunk anterior | `K` |
| Revisão: escolher codificação... | `E` |
| Revisão: alternar minimapa | `M` |
| Revisão: deslocar o diff para a esquerda | `Left` |
| Revisão: deslocar o diff para a direita | `Right` |
| Revisão: copiar o texto selecionado ou a linha | `Ctrl+C` |
| Quadro: mover cartão para... | `m` |
| Quadro: mover cartão para a esquerda |  |
| Quadro: mover cartão para a direita |  |
| Quadro: abrir cartão | `Enter` |
| Quadro: editar responsáveis... |  |
| Quadro: editar etiquetas... |  |
| Quadro: definir marco... |  |
| Quadro: nova issue nesta coluna | `n` |
| Quadro: filtrar por responsável... | `a` |
| Quadro: filtrar por etiqueta... | `l` |
| Quadro: filtrar por marco... | `M` |
| Quadro: filtrar por texto | `/` |
| Quadro: limpar filtros | `x` |
| Quadro: definir etiquetas das colunas... | `c` |
| Quadro: atualizar | `r` |
| Quadro: fechar | `Esc` |
| Conversa: voltar |  |
| Conversa: atualizar | `F5` |
| Conversa: carregar entradas mais antigas | `m` |
| Conversa: entrada anterior | `Up` |
| Conversa: entrada seguinte | `Down` |
| Conversa: próxima entrada nova | `n` |
| Conversa: responder à entrada | `r` |
| Conversa: citar a entrada | `Q` |
| Conversa: novo comentário | `c` |
| Conversa: ligações da entrada... | `l` |
| Conversa: abrir o diagrama da entrada | `g` |
| Conversa: abrir o código da discussão | `G` |
| Conversa: copiar a ligação da entrada | `y` |
| Conversation: Copy Text | `Ctrl+C` |
| Conversa: abrir no navegador | `o` |
| Conversa: editar comentário | `e` |
| Conversa: eliminar comentário | `d` |
| Conversa: mostrar ou ocultar eventos | `v` |
| Conversa: alternar entre Escrever e Prévia |  |
| Conversa: alternar a vista lado a lado |  |
| Conversa: enviar o comentário |  |
| Conversa: alternar nota interna |  |
| Conversa: fechar | `Esc` |
| Editor de Markdown: escrever com IA |  |
| Editor de Markdown: atualizar o contexto da IA |  |
| Alojamento: abrir a conversa de uma issue... |  |
| Revisão: ✦ redigir revisão... | `s` |
| Revisão: aceitar sugestão | `y` |
| Revisão: editar e aceitar sugestão... | `w` |
| Revisão: dispensar sugestão | `n` |
| Revisão: aceitar todas as sugestões | `A` |
| Revisão: dispensar todas as sugestões | `N` |
| Revisão: usar o resumo da IA no texto da revisão | `i` |
| Revisão: ✦ usar o rascunho... | `u` |
| Revisão: editar o texto do rascunho ✦... | `t` |
| Revisão: descartar o rascunho ✦ | `N` |
| Histórico de desfazer: desfazer até aqui... | `u` |
| Histórico de desfazer: refazer até aqui... | `y` |
| Histórico de desfazer: restaurar só esta... | `r` |
| Histórico de desfazer: restaurar a branch para onde estava... | `o` |
| Histórico de desfazer: atualizar |  |
| Histórico de desfazer: fechar | `Esc` |
| Reflog: mostrar no Log | `l` |
| Reflog: criar branch aqui... | `b` |
| Reflog: criar tag aqui... | `t` |
| Reflog: aplicar alterações à árvore de trabalho | `w` |
| Reflog: guardar alterações como stash | `s` |
| Reflog: restaurar para esta posição... | `r` |
| Reflog: carregar mais | `m` |
| Reflog: atualizar |  |
| Trabalho perdido: criar branch aqui... | `b` |
| Trabalho perdido: criar tag aqui... | `t` |
| Trabalho perdido: Cherry-pick | `c` |
| Trabalho perdido: aplicar alterações à árvore de trabalho | `w` |
| Trabalho perdido: guardar alterações como stash | `s` |
| Trabalho perdido: procurar de novo |  |
| Reflog / Trabalho perdido: painel seguinte | `Tab` |
| Reflog / Trabalho perdido: lista de ficheiros em caminhos ou árvore | `v` |
| Reflog / Trabalho perdido: repor tamanhos dos painéis |  |
| Janela: mover ou redimensionar | `Ctrl+N m` |
| Janela: repor posição e tamanho |  |
| Ver: atualizar | `r` |
| Ver: atualizar (fetch, pull requests, issues) | `F5` |
| Ver: alternar a ajuda | `?` |
| Ver: alternar a coluna Ref |  |
| Ver: alternar a coluna Grafo |  |
| Ver: alternar a coluna Mensagem |  |
| Ver: alternar a coluna Autor |  |
| Ver: alternar a coluna Data |  |
| Ver: alternar a coluna Alterações |  |
| Ver: alternar a coluna SHA |  |
| Ver: coluna do Log: Escolher uma coluna para mover ou redimensionar | `C` |
| Ver: coluna do Log: Escolher a coluna seguinte | `Tab` |
| Ver: coluna do Log: Escolher a coluna anterior | `Shift+BackTab` |
| Ver: coluna do Log: Mover a coluna para a esquerda | `<` |
| Ver: coluna do Log: Mover a coluna para a direita | `>` |
| Ver: coluna do Log: Alargar a coluna | `]` |
| Ver: coluna do Log: Estreitar a coluna | `[` |
| Ver: coluna do Log: Repor as larguras das colunas | `0` |
| Ver: coluna do Log: Ajustar a coluna ao conteúdo | `=` |
| Ver: coluna do Log: Ajustar todas as colunas ao conteúdo | `+` |
| Ver: Log: deslocar as colunas para a esquerda | `Shift+Left` |
| Ver: Log: deslocar as colunas para a direita | `Shift+Right` |
| Ver: Log: alternar o deslocamento lateral das colunas | `s` |
| Ver: Log: escolher as colunas a mostrar |  |
| Diff: abrir o diff do ficheiro selecionado | `Enter` |
| Diff: fechar a vista de diff | `Esc` |
| Diff: próximo hunk | `n` |
| Diff: hunk anterior | `p` |
| Diff: stage do hunk sob o cursor | `Space` |
| Diff: unstage do hunk sob o cursor | `Space` |
| Diff: selecionar linhas (ou limpar a seleção) | `v` |
| Diff: stage das linhas selecionadas | `Enter` |
| Diff: unstage das linhas selecionadas | `Enter` |
| Diff: descartar o hunk sob o cursor... | `x` |
| Diff: descartar as linhas selecionadas... | `X` |
| Copiar: texto selecionado (arraste sobre o texto antes) | `Ctrl+C` |
| Copiar: linha sob o cursor (ou as linhas selecionadas) | `y` |
| Diff: copiar o hunk sob o cursor |  |
| Diff: layout inline (+/- numa coluna) | `m` |
| Diff: layout dividido (antes e depois) | `m` |
| Diff: alternar só hunks (ficheiro todo / só as alterações) | `H` |
| Ficheiros: alternar caminho/árvore | `m` |
| Ficheiros: alternar ver todos os ficheiros | `a` |
| Commit: alternar ignorar hooks | `H` |
| Ficheiros: abrir o menu da linha ou da seleção | `.` |
| Ficheiros: expandir todas as pastas | `=` |
| Ficheiros: recolher todas as pastas | `-` |
| Ficheiros: expandir a pasta selecionada e as suas subpastas | `*` |
| Ficheiros: recolher a pasta selecionada e as suas subpastas | `_` |
| Ficheiros: selecionar tudo |  |
| Ficheiros: limpar a seleção |  |
| Ficheiros: alternar a seleção da linha atual |  |
| Log: copiar como Mermaid gitGraph... | `Y` |
| Log: comparar a seleção | `v` |
| Log: trocar as extremidades da comparação | `x` |
| Log: ir para a próxima linha comparada | `n` |
| Diff: alternar quebra de linha | `w` |
| Diff: escolher codificação... | `C` |
| Ver: alternar o minimapa | `M` |
| Diff: Toggle Changed Words Highlight |  |
| Ver: alternar a coluna de commits | `Alt+C` |
| Ver: coluna de commits: ocultar |  |
| Ver: coluna de commits: histórico do ficheiro |  |
| Ver: coluna de commits: todos os commits |  |
| Ver: focar a coluna de commits |  |
| Ver: alargar a coluna de commits |  |
| Ver: estreitar a coluna de commits |  |
| Ver: coluna de commits: deslocar para a esquerda | `Shift+Left` |
| Ver: coluna de commits: deslocar para a direita | `Shift+Right` |
| Ver: coluna de commits: dimensionar a parte seguinte | `Tab` |
| Ver: coluna de commits: mover a parte para a esquerda | `<` |
| Ver: coluna de commits: mover a parte para a direita | `>` |
| Ver: coluna de commits: alargar a parte | `]` |
| Ver: coluna de commits: estreitar a parte | `[` |
| Ver: coluna de commits: repor as larguras das partes | `0` |
| Ver: coluna de commits: Ajustar a coluna ao conteúdo | `=` |
| Ver: coluna de commits: Ajustar todas as colunas ao conteúdo | `+` |
| Ver: coluna de commits: menu da linha selecionada | `.` |
| Ver: coluna de commits: escolher as colunas |  |
| Ver: coluna de commits: alternar Ref |  |
| Ver: coluna de commits: alternar Grafo |  |
| Ver: coluna de commits: alternar Autor |  |
| Ver: coluna de commits: alternar Data |  |
| Ver: coluna de commits: alternar Alterações |  |
| Ver: coluna de commits: alternar SHA |  |
| Ver: coluna de commits: mostrar a linha selecionada no Log |  |
| Ver: coluna de commits: mostrar as alterações do ficheiro na linha selecionada |  |
| Ver: coluna de commits: mostrar o ficheiro como era na linha selecionada |  |
| Ver: coluna de commits: blame do ficheiro na linha selecionada |  |
| Editor: alternar seguir a seleção |  |
| Imagem: aumentar zoom | `+` |
| Imagem: diminuir zoom | `-` |
| Imagem: ajustar à vista | `0` |
| Imagem: tamanho real (1:1) | `1` |
| Imagem: alternar antes / depois | `Tab` |
| Imagem: alargar painel Antes | `Alt+Right` |
| Imagem: alargar painel Depois | `Alt+Left` |
| Imagem: repor tamanhos dos painéis | `Alt+0` |
| Imagem: editar em... | `O` |
| Imagem: editar no editor predefinido | `o` |
| Histórico do ficheiro: mostrar no Log | `Enter` |
| Histórico do ficheiro: copiar o SHA do commit | `y` |
| Histórico do ficheiro: copiar o resumo | `Y` |
| Histórico do ficheiro: mostrar o diff deste commit | `d` |
| Histórico do ficheiro: abrir o ficheiro neste commit | `o` |
| Histórico do ficheiro: Blame neste commit | `b` |
| Histórico do ficheiro: comparar com a árvore de trabalho | `c` |
| Histórico do ficheiro: restaurar esta versão | `R` |
| Histórico do ficheiro: estender a seleção para cima | `Shift+Up` |
| Histórico do ficheiro: estender a seleção para baixo | `Shift+Down` |
| Histórico do ficheiro: alternar entre a lista e o diff | `Tab` |
| Histórico do ficheiro: alargar a lista | `Alt+Right` |
| Histórico do ficheiro: estreitar a lista | `Alt+Left` |
| Histórico do ficheiro: repor a largura da lista | `Alt+0` |
| Ficheiro: abrir com... | `O` |
| Ficheiro: abrir na aplicação predefinida | `o` |
| Diff: alternar alterações só de espaços em branco | `s` |
| ✦ IA: gerar mensagem de commit | `Ctrl+G i` |
| ✦ IA: gerar mensagem de stash |  |
| ✦ IA: inserir variável de prompt |  |
| ✦ IA: editar perfil... |  |
| ✦ IA: abrir o agente... |  |
| ✦ IA: explicar o commit selecionado | `Ctrl+G w` |
| ✦ IA: dividir o commit selecionado... |  |
| ✦ IA: explicar as alterações em curso | `Ctrl+G w` |
| ✦ IA: compor commits a partir das alterações em curso... | `Ctrl+G C` |
| Ficheiros sensíveis: mostrar no ficheiro | `S` |
| Compositor: mover ficheiro para um commit... | `m` |
| Diff: blame |  |
| Blame: mostrar o commit no Log | `Enter` |
| Blame: copiar o SHA do commit | `Y` |
| Blame: diff do commit para este ficheiro | `d` |
| Blame: blame no pai do commit | `b` |
| Blame: abrir o histórico do ficheiro | `H` |
| Blame: mostrar ou ocultar o cartão de detalhes | `i` |
| Blame: hunk seguinte | `n` |
| Blame: hunk anterior | `p` |
| Blame: hunk seguinte do mesmo commit | `N` |
| Blame: hunk anterior do mesmo commit | `P` |
| Blame: ir para a linha | `g` |
| Blame: alargar a coluna de anotações | `>` |
| Blame: estreitar a coluna de anotações | `<` |
| Diff: histórico do ficheiro |  |
| Diff: vista do ficheiro |  |
| Ficheiro: editar no diretório de trabalho | `e` |
| Editor: alterar codificação |  |
| Editor: reabrir com codificação |  |
| Editor: converter para codificação |  |
| Editor: converter fins de linha |  |
| Editor: alterar indentação |  |
| Aplicação: definições... | `Ctrl+, / Ctrl+N ,` |
| Aplicação: pesquisar definições... | `/ nas Definições` |
| Aplicação: atalhos de teclado... |  |
| Agente: mostrar a vista pendente | `Ctrl+N Enter` |
| Agente: dispensar a vista pendente |  |
| Agente: voltar | `Ctrl+N Backspace` |
| Sequence: Cancel |  |
| Sequence: Continue |  |
| Sequence: Review This Repository's Sequences |  |
| Sequence: Trust This Repository's Sequences |  |
| Sequence: Ignore This Repository's Sequences |  |
| Atalhos: repor tudo... |  |
| Integrações: abrir pull request associado | `Ctrl+G g` |
| Integrações: editar pull request associado no host | `e (numa linha)` |
| Integrações: copiar URL do pull request associado |  |
| Integrações: atualizar pull requests e issues |  |
| Integrações: checkout de pull request... |  |
| Integrações: abrir referência do commit (#issue / !pedido)... |  |
| Integrações: abrir issue... |  |
| Alojamento: rever pedido... |  |
| Alojamento: push para revisão... |  |
| Alojamento: editar responsáveis... |  |
| Alojamento: editar revisores... |  |
| Alojamento: editar etiquetas... |  |
| Alojamento: definir marco... |  |
| Issues locais: fechar issue... |  |
| Issues locais: reabrir issue... |  |
| Issues locais: abrir no Obsidian... |  |
| Issues: abrir quadro |  |
| Issues locais: criar Base do Obsidian |  |
| Pull requests: criar pull request... | `n` |
| Alojamento: nova issue... | `n` |
| Pull requests: pesquisar | `Enter na linha de pesquisa` |
| Pull requests: escolher filtro (abertos / merged / todos)... | `v` |
| Pull requests: escolher fornecedor... | `Enter na linha do fornecedor` |
| Pull requests: escolher repositório... | `Enter na linha do repositório` |
| Pull requests: desassociar origem (nenhuma) |  |
| Issues: escolher fornecedor... | `Enter na linha do fornecedor` |
| Issues: desassociar origem (nenhuma) |  |
| Integrações: desassociar perfil (automático) |  |
| Issues: escolher repositório... | `Enter na linha do repositório` |
| Issues: pesquisar | `Enter na linha de pesquisa` |
| Issues: escolher filtro... | `v` |
| Issues: editar consulta / listas... |  |
| Commit: Inserir referência de issue... |  |
| Integrações: usar perfil neste repositório... |  |
| Integrações: gerir o perfil deste repositório |  |
| Integrações: ligar integração... |  |
| Definições: Opções do Radar de um perfil... |  |
| Integrações: rever verificação de segurança |  |
| Aplicação: sair | `q` |
| Aplicação: tema... |  |
| Conflitos: editar as listas de branches base... |  |
| Conflitos: branches base deste repositório... |  |
| Aplicação: mudar tema... | `Ctrl+N t` |
| Aplicação: mudar idioma... | `Ctrl+N l` |
| Lançador: criar atalho |  |
| Lançador: criar atalho do ambiente de trabalho |  |
| Lançador: remover atalhos |  |
| Lançador: criar alias de comando |  |
| Lançador: remover alias de comando |  |
| MCP: sugerir uma porta livre |  |
| Lançador: adicionar ao PATH |  |
| Lançador: remover do PATH |  |
| Lançador: instalar o menu Abrir no GitPoulpi |  |
| Lançador: remover o menu Abrir no GitPoulpi |  |
| Lançador: tornar principal a instalação |  |
| Lançador: remover lançadores antigos |  |
| Lançador: esquecer instalações desaparecidas |  |
| Aplicação: mudar tema de cores do grafo... | `Ctrl+N T` |
| Aplicação: alternar tom das linhas do grafo | `Ctrl+N Y` |
<!-- palette:end -->

## Mover-se

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab` | Focar o painel seguinte / anterior |
| `Ctrl+N`, depois um selo | Focar essa parte da janela, ou premir esse controlo da janela ou do popup da frente (consulte [Prefixo de navegação](#prefixo-de-navegação-ctrln)) |
| `j` / `k`, `Up` / `Down` | Mover a seleção, em todos os painéis |
| `PageUp` / `PageDown`, `Home` / `End` | Mover uma página; ir para a primeira / última linha |
| um número, depois `j` / `k` / `Up` / `Down` | Prefixo de contagem: `4j` move quatro linhas. Com `PageUp` / `PageDown`, a contagem é um número de páginas. A barra de estado mostra a contagem pendente (`4...`). `Esc` ou qualquer outra tecla limpa-a. Um `0` inicial nunca começa uma contagem |
| `h` / `l`, `Left` / `Right` | Painel Refs: mudar de secção. Log: sair (`h`) ou entrar (`l`) na lista de ficheiros alterados de um commit |
| `?` | Abrir ou fechar a ajuda (consulte [Ajuda integrada](#ajuda-integrada-)) |
| `r` | Atualizar |
| `F5` | Fazer fetch de todos os remotos, recarregar os pull requests e as issues, e atualizar |
| `q` | Sair |
| `Ctrl+Z` | **Desfazer** a última operação. Consulte [Desfazer e refazer](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Refazer**. `Ctrl+Shift+Z` precisa do protocolo de teclado do kitty; `Ctrl+Y` funciona sempre |
| `.`, `Menu`, `Shift+F10` | Abrir o menu de contexto da linha selecionada |

### Ctrl+Z / Ctrl+Y num campo de texto

Enquanto um campo de texto tem o teclado, `Ctrl+Z` e `Ctrl+Y` (também
`Ctrl+Shift+Z`) atuam sobre o texto desse campo, nunca sobre o repositório: um campo
que tem o teclado ganha sempre, pelo que `Ctrl+Z` na mensagem de commit não consegue
desfazer um commit. Em todos os outros sítios mantêm o seu significado global (o
registo de desfazer / refazer acima). Isto vale para todos os campos de texto e
editores de várias linhas: a mensagem de commit, os campos e editores de texto das
Definições, os formulários de pull request e de issue, as janelas de IA e todas as
caixas de filtro ou de pesquisa.

- **Escrever** desfaz-se palavra a palavra (uma sequência de letras, uma de espaços,
  uma de pontuação), e escrever com uma pausa de mais de um segundo começa um novo
  passo. Vários retrocessos seguidos são um só passo. Uma colagem, um recorte,
  `Ctrl+U` / `Ctrl+K`, um excerto inserido ou uma conclusão aceite é um passo cada.
- Um desfazer repõe o texto, o cursor e a seleção. Uma nova edição depois de um
  desfazer descarta o que se poderia refazer.
- Cada campo mantém um histórico limitado (os últimos 200 passos, cerca de 1 MB de
  texto), e esquece-o quando o campo é substituído a partir de fora (uma pasta
  escolhida no seletor, um formulário recarregado) ou quando outro campo fica com o
  teclado.
- Sem nada para desfazer, `Ctrl+Z` não faz nada: nunca passa para o desfazer do
  repositório enquanto um campo tem o teclado.
- O **editor de instruções** das definições de IA tem também botões **Anular** e
  **Refazer**, junto a Guardar e Cancelar, desativados enquanto não há nada a que
  passar.
- O editor de código e o editor de merge mantêm o seu próprio desfazer (`Edit: Undo`,
  `Merge: Undo`), que cobre também as suas escolhas.

## Listas de popup filtráveis

Todas as listas de popup filtráveis usam as mesmas teclas: a paleta de comandos e os seus
seletores (tema, idioma, cores do grafo), o seletor de separadores, e todas as listas pendentes
com linha de pesquisa (opções das Definições, o formulário de pull request, o seletor
de referências de issues, a lista **Abrir em**). Escreva para filtrar a lista. Estas teclas
movem o realce:

| Tecla | Ação |
|---|---|
| `Up` / `Down`, `Ctrl+K` / `Ctrl+J` | Mover uma linha para cima / para baixo |
| `Home` / `End` | Primeira / última linha |
| `PageUp` / `PageDown` | Mover uma página visível |
| `Enter` | Escolher a linha realçada |
| `Esc` | Fechar |

Nestes popups, `Home` e `End` movem a lista, não o cursor do campo de filtro, e
`Ctrl+K` / `Ctrl+J` movem também a lista, pelo que `Ctrl+K` já não apaga até ao fim do
filtro (`Ctrl+U` continua a limpá-lo; num campo de texto vulgar fora de uma lista
`Ctrl+K` continua a apagar até ao fim da linha). `Ctrl+A` e `Ctrl+E` continuam a mover o
cursor para o início e o fim. `Ctrl+J` / `Ctrl+K` movem também todas as outras listas,
com ou sem campo de texto (`Ctrl+N` é o prefixo de navegação e `Ctrl+P` a paleta de
comandos, pelo que não são teclas de lista). `j` e `k` simples são escritos no filtro;
nas listas sem campo de texto, `j` / `k` são uma via extra para descer / subir.

Com o rato:

- A roda **desloca** a lista. Não move o realce.
- Um clique **seleciona** uma linha. Um clique duplo **escolhe**-a, como `Enter`.
- Uma lista mais alta do que o seu popup mostra uma barra de deslocação na margem direita. Pode
  arrastá-la.
- Uma lista pendente curta sem linha de pesquisa (duas ou três opções) escolhe ao
  primeiro clique.

**Pré-visualização em direto.** Os seletores de tema, de tema de cores do grafo e de idioma aplicam a
linha realçada de imediato, por trás do popup. `Enter` mantém-na. `Esc`, ou um clique
fora do popup, repõe o valor anterior.

**Filtro de temas.** O seletor de temas tem uma linha por baixo do campo de filtro: **Todos**,
**Escuros**, **Claros** e **Coloridos**. `Tab` / `Shift+Tab` vão para o seguinte /
anterior. `Alt+1` a `Alt+4` escolhem um diretamente. Também pode clicar neles.

## Ajuda integrada (`?`)

`?`, `Ctrl+N ?` ou o comando `Ver: alternar a ajuda` da paleta abre a ajuda.
Lista as teclas e os gestos do rato de todas as partes da aplicação. Abre por cima de
tudo, incluindo o ecrã das Definições.

| Tecla | Ação |
|---|---|
| `?` | Fechar a ajuda (e limpar o filtro) |
| *(escrever)* | Filtrar: manter apenas as secções que contêm o texto. `j`, `k` e `q` também são escritos |
| `Backspace`, `Ctrl+U` | Apagar o último carácter / limpar o filtro |
| `Esc` | Limpar o filtro; se estiver vazio, fechar a ajuda |
| `Up` / `Down` | Deslocar uma linha |
| `PageUp` / `PageDown`, `Home` / `End` | Deslocar uma página; ir para o topo / fim |
| `Ctrl+D`, ou um clique na ligação | Abrir esta documentação no GitHub, no navegador |

A roda desloca a ajuda, e pode arrastar a sua barra de deslocação. Um clique fora
da ajuda fecha-a.

## Prefixo de navegação (`Ctrl+N`)

`Ctrl+N` espera mais uma tecla. A barra de estado mostra as teclas que aceita. A
tecla seguinte executa uma ação, ou cancela.

| Tecla | Ação |
|---|---|
| `Ctrl+N` e depois um selo | Focar a parte da janela que mostra este selo |
| `Ctrl+N Shift+H` / `Ctrl+N Shift+L` | Recolher ou repor o painel Refs / o painel direito |
| `Ctrl+N Left` / `Ctrl+N Right` | Mover o divisor entre o painel Refs e o Log um passo para a esquerda / direita (um painel Refs recolhido volta primeiro) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Mover o divisor entre o Log e o painel direito um passo para a esquerda / direita |
| `Ctrl+N +` / `Ctrl+N -` | Aumentar / reduzir a secção Refs ativa em uma linha |
| `Ctrl+N ]` / `Ctrl+N [` | Aumentar / reduzir o bloco da mensagem de commit do painel direito em uma linha |
| `Ctrl+N Tab` | Escolher um separador (o seletor de separadores; `t` sozinho faz o mesmo) |
| `Ctrl+N t` / `Ctrl+N l` / `Ctrl+N Shift+T` | Mudar o tema / o idioma / o tema de cores do grafo |
| `Ctrl+N Shift+Y` | Ligar ou desligar as tonalidades das linhas do grafo |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Mostrar a vista que um agente preparou / voltar atrás (consulte [MCP e agentes](mcp-and-agents.md)) |
| `Ctrl+N ,` | Abrir as Definições |
| `Ctrl+N ?` | Abrir ou fechar a ajuda |
| `Esc`, ou qualquer outra tecla | Cancelar |

### Selos

Enquanto o prefixo espera, cada parte da janela que pode receber o foco mostra
um selo de um carácter na sua margem superior esquerda. Prima esse carácter para a focar.
Os selos são distribuídos de novo de cada vez, pela ordem do ecrã: primeiro `1` a `9`,
depois `a`, `b`, `c`... Por isso leia o selo no ecrã: uma parte nem sempre
recebe o mesmo selo.

Partes com selo: o filtro Refs e cada secção Refs visível, o Log (ou
a vista Diff), a vista de Ficheiro quando está aberta, as listas Unstaged e Staged,
o campo de resumo do compositor de commits, e os controlos da lista de ficheiros do commit.
No separador Início, os selos são o campo de pesquisa, as quatro secções e os botões da barra de ferramentas (o selo de um botão aciona-o). No
separador Radar, são o campo de filtro, as listas pendentes Âmbito, Vista e
Mostrar (o selo de uma lista pendente abre-a), os botões Atualizar e Voltar e cada
título de secção no ecrã. O selo de uma secção move a seleção para o respetivo
título; depois `Enter` ou `Space` recolhe-a.

Nos diálogos Procurar pasta (o seletor de pastas), Clonar, Init, Criar worktree, Área de trabalho, Checkout parcial, Flow, Nova issue e Criar pull request, os selos são os campos, as caixas de verificação e os botões (o selo de um botão aciona-o, o de um campo põe o cursor nele; numa lista, há um selo na primeira linha; o seletor de pastas tem um selo para a lista de Acesso rápido e outro para a lista de Pastas, junto à primeira linha de cada lista, e premir um põe o teclado nessa lista sem escolher uma linha). No Registo de atividade, são os dois separadores, as caixas Âmbito e Nível, o filtro, as caixas de verificação dos níveis, os botões e a primeira entrada da lista. Um selo faz o que um clique nesse controlo faz. No seletor de pastas, `Ctrl+N` inicia os selos, pelo que a sua tecla de Nova pasta é `Alt+N`.

No diálogo Alterar (e Editar mensagem do commit), os selos são o resumo, a descrição, o nome, o e-mail e a data do autor, **Modificar o histórico mais a fundo...**, **Cancelar**, **Aplicar** e a cruz de fechar; no painel de reescrita do histórico são a caixa de âmbito, as suas caixas de verificação e o botão das refs, o nome da receita, cada regra, a sua caixa de verificação e botões, o botão **+ Adicionar regra**, os campos da regra selecionada, os separadores da pré-visualização, a caixa de filtro, as linhas das branches protegidas e os seus Marcar todas / Desmarcar todas, e os botões Pré-visualizar, Aplicar, Cancelar, Guardar, Carregar e Criar regras com IA... (as páginas Segredos, Identidades, Tamanho e de resultado acrescentam os seus botões e linhas; as linhas do separador Commits, por serem uma lista, não têm). Um selo faz o que um clique faz. Nada leva selos enquanto estiver aberta uma lista pendente, uma pergunta, um campo em que se escreve, o seletor de pastas, o ecrã de purga, o editor de scripts, os diálogos de IA, o diff de blobs ou a explicação do host, nem enquanto o diálogo Alterar pergunta «descartar?»; `Esc` retira os selos sem fechar nada.

Nas Definições, os selos ficam na caixa de pesquisa, na lista de páginas (um só selo, na página atual), nos subseparadores, na lista e nos respetivos botões, em cada campo da página que está no ecrã e no ✕ de fechar. Um selo faz o que faria um clique nesse controlo: um campo de texto abre para edição, uma caixa de seleção é assinalada, uma lista pendente abre-se, um botão é acionado. `Ctrl+N` mantém o seu significado próprio enquanto se escreve num campo ou há uma lista pendente, um editor ou a pesquisa aberta.

A paleta tem as mesmas ações, por exemplo
`Navegação: focar Refs > Local` ou `Navegação: focar Status > Staged`.

## Repositórios e separadores

| Tecla | Ação |
|---|---|
| `Ctrl+O`, ou um clique em `⌂` | Ir para o separador Início (no separador Início: focar o campo de pesquisa; com o Radar ou as notas de versão abertos: fechá-lo primeiro) |
| `Ctrl+Shift+N` | Abrir o separador Início com o diálogo Clonar |
| `Ctrl+PageDown` / `Ctrl+PageUp`, `Ctrl+Tab` / `Ctrl+Shift+Tab` | Separador seguinte / anterior (o par `Ctrl+Tab` só onde o terminal o envia) |
| `t` | Escolher um separador a partir de uma lista filtrável |
| `Ctrl+W` | Fechar o separador atual (pergunta primeiro; o repositório não é alterado) |
| `Ctrl+T`, ou um clique com o botão direito num separador | O menu do separador (as suas linhas estão listadas em [Linha de separadores](interface-tour.md#linha-de-separadores)) |
| `Alt+O` / `Ctrl+Alt+O` | Abrir a pasta do separador: escolher na lista **Abrir em** / usar a entrada predefinida. Consulte [Abrir em](open-in.md) |

### O separador Início

O separador `⌂` gere os seus repositórios. Tem quatro secções: **Repositórios
abertos**, **Favoritos**, **Repositórios recentes** e **Todos os
repositórios**. Consulte [Visita guiada à interface](interface-tour.md#início).

| Tecla | Ação |
|---|---|
| `j` / `k`, setas, `PageUp` / `PageDown`, `Home` / `End` | Mover |
| `Enter`, ou um clique duplo | Abrir o repositório (ou mudar para o seu separador). No cabeçalho de uma secção: recolhê-la ou expandi-la |
| `f` ou `*`, ou um clique na estrela | Adicionar aos / remover dos favoritos |
| `x`, `Delete`, ou o botão da linha | Fechar o seu separador (Abertos), removê-lo da lista (Recentes), retirar a estrela (Favoritos), esquecê-lo (Todos). Nada no disco é eliminado |
| `.`, um clique com o botão direito, ou o `⋮` da linha | O menu da linha |
| `/` ou `Ctrl+F` | Pesquisar em todas as secções por nome, branch ou caminho. `Enter` abre a primeira correspondência; `Esc` limpa a pesquisa e depois sai do campo |
| `Space`, `Left` / `Right` | Recolher / expandir a secção atual |
| `-` / `+` | Recolher tudo / Expandir tudo |
| `b` / `c` / `i` | Procurar (abrir uma pasta) / Clonar / Init |
| `Esc` | Limpar a pesquisa, ou voltar ao separador de repositório ativo |
| `q` | Sair |

`Ctrl+P`, `Ctrl+,`, `?` e `Ctrl+N` funcionam no separador Início. `Ctrl+G`, `Ctrl+W`
e `Ctrl+T` precisam de um separador de repositório: no separador Início apenas mostram uma mensagem.
Os comandos da paleta que precisam de um repositório ficam a cinzento.

### O seletor de pastas

**Procurar** (e o botão **Procurar...** dos diálogos Init, Clonar e Criar
worktree) abre o seletor de pastas. Tem um campo de caminho no topo, os
locais e a árvore de pastas à esquerda, e o conteúdo da pasta atual
à direita.

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab` | Campo de caminho, depois a coluna da esquerda, depois o conteúdo |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Mover na coluna com foco. No campo de caminho: percorrer as sugestões |
| uma letra (conteúdo) | Ir para a pasta seguinte que começa por essa letra |
| `Enter` / `Right` | Abrir a pasta selecionada. No campo de caminho: ir para a pasta escrita; se for a pasta já mostrada, escolhê-la |
| `Backspace` / `Left` / `Alt+Up` | Pasta superior (no campo de caminho, `Backspace` apaga um carácter) |
| `Tab` (campo de caminho) | Aceitar a única sugestão, ou a realçada |
| `Alt+N` | Nova pasta (`Ctrl+N` inicia os selos, veja [Selos](#selos)) |
| `Alt+H` | Mostrar ou ocultar pastas ocultas |
| `Ctrl+F`, ou `/` fora do campo de caminho | Filtrar as entradas da pasta à medida que escreve (sem distinguir maiúsculas; `..` fica; o cabeçalho conta "x de n"; `Enter` salta para a primeira correspondência, `Esc` limpa; esvazia-se quando muda de pasta). Também acessível com `Tab` ou um clique |
| `Ctrl+S`, `Ctrl+Enter`, ou o botão de confirmar | Escolher a subpasta selecionada ou, se não houver, a pasta mostrada |
| `Esc` | Fechar as sugestões, ou cancelar |

Quando o campo de caminho mostra um novo caminho, todo o caminho fica selecionado: a primeira
tecla que escrever substitui-o.

### Diálogos Init e Clonar

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Campo ou botão seguinte / anterior |
| `Enter` | Premir o botão com foco, ou criar / clonar |
| `Space` | Alternar a caixa de verificação com foco, ou premir o botão com foco |
| `Ctrl+B` | Procurar a pasta |
| `j` / `k`, `g` / `G` | Na lista de origens e na lista de repositórios do diálogo Clonar: para baixo / para cima, primeira / última (não no campo de pesquisa) |
| `Esc` | Fechar. Durante um clone, o clone continua em segundo plano |

No diálogo Clonar, a coluna da esquerda lista **Clonar por URL** e uma origem
por perfil de alojamento ligado (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit).
Escolha uma origem com `Up` / `Down` ou um clique. Depois escreva no campo de pesquisa
**Repositório**, mova-se na lista com as setas, `PageUp` / `PageDown` ou a
roda, e escolha uma linha com `Enter` ou um clique. Consulte
[Primeiros passos](getting-started.md) para os campos.

## Painel Refs

| Tecla | Ação |
|---|---|
| `Enter` / `Space` | Expandir ou recolher a secção ativa, ou a pasta selecionada |
| `h` / `l`, `Left` / `Right` | Mudar de secção |
| `/` | Filtrar por nome (`Enter` aplica, `Esc` cancela) |
| `y` | Ligar ou desligar a vista de hierarquia de branches: pastas a partir dos `/` nos nomes das branches (LOCAL e REMOTO) |
| `o` | **Solo** da ref selecionada, ou de todas as refs sob a pasta ou o remoto selecionado. Faça solo de várias refs para ver a sua união |
| `Shift+O` | **Parar solo**: limpar todos os solos |
| `Shift+H` | **Mostrar tudo**: mostrar de novo todas as refs ocultas |
| `g` | **Mostrar no grafo**: desloca o Log até ao commit da ref (ou do stash) selecionada e faz a linha piscar; a seleção mantém-se |
| `m` | **Marcar** a ref selecionada como origem do largar (`Esc`, ou `m` nela outra vez, limpa a marca) |
| `Shift+M` | **Largar a ref marcada** na ref selecionada: abre o menu que um arrastar e largar abre (Merge, Rebase, Push, Fast-forward...) |
| `+` (secção REMOTO) | Adicionar um remoto |
| `.`, `Menu`, `Shift+F10`, ou um clique com o botão direito | O menu da linha: uma branch, uma tag, o nome de um remoto, ou uma pasta |
| `Enter` numa linha de WORKTREES | Abrir esse worktree no seu próprio separador |
| `Ctrl+G d` | Eliminar a branch ou tag selecionada (pergunta primeiro). Numa linha de REMOTO: eliminar essa branch remota |

Para saber o que fazem os menus, consulte [Trabalhar com o Log](log-and-history.md).

### Pull requests e issues

Com uma integração correspondente, o painel Refs tem uma secção de pull requests (GitLab:
merge requests). Com um perfil de alojamento ligado, tem também uma secção
ISSUES. Consulte [Alojamento e gestores de issues](pull-requests-and-issues.md#pull-requests-e-issues-na-barra-lateral).

| Tecla | Ação |
|---|---|
| `Enter` | Abrir a linha selecionada: uma issue (e um pedido no Azure DevOps ou no Bitbucket) como [conversa](conversations.md), um pull request na [revisão](review.md) nos hosts que a têm, tudo o resto no navegador. Numa linha `Religar`: abrir as Definições nesse perfil |
| `Space` | Expandir ou recolher a secção |
| `v` | Abrir o filtro da secção. Pull requests: Abertos, Merged / fechados, Todos. Issues: Abertas, Atribuídas a mim, Fechadas, Todas |
| `n` | Secção de pull requests: criar um pull request. Secção ISSUES: nova issue. (O botão `+` do cabeçalho) |
| `e` | Secção de pull requests: editar o pedido selecionado no host (o lápis da linha) |
| `Enter` na linha de pesquisa | Escrever no campo de pesquisa. `Enter`, `Tab` ou `Esc` sai dele; `Ctrl+U` limpa-o |
| `Enter` no cabeçalho de um grupo | Recolher ou expandir o grupo (*Meus pull requests*, *Atribuídos a mim*, *A aguardar minha revisão*, *Todos os pull requests*) |
| `Enter` na linha do filtro, do fornecedor ou do repositório | Abrir essa lista pendente. A lista de fornecedores termina com **Nenhum (desvincular este repositório)** |
| `.`, ou um clique com o botão direito | O menu da linha: abrir a conversa, abrir no navegador, copiar URL ou número, fazer checkout da branch de origem (pull requests); criar uma branch a partir da issue (issues) |

### O quadro de issues

**Quadro** no cabeçalho ISSUES (paleta: **Issues: abrir quadro**) abre o quadro Kanban da origem de issues. Ver
[o quadro](pull-requests-and-issues.md#o-quadro-de-issues) e [as issues locais](local-issues.md#o-quadro-kanban). Cada tecla
abaixo é também um comando da paleta (**Quadro: ...**).

| Tecla | Ação |
|---|---|
| Setas | Selecionar um cartão (`Up`, `Down`) ou a coluna vizinha (`Left`, `Right`); `Home`, `End`, `PageUp`, `PageDown` saltam na coluna |
| `Shift+Left`, `Shift+Right` | Mover o cartão selecionado para a coluna anterior ou seguinte |
| `m` | **Mover cartão para...**: uma lista das colunas |
| `Enter` | Abrir a [conversa](conversations.md) da issue (a página do host onde o host não tem nenhuma para ler) |
| `e`, `.` | Editar responsáveis, etiquetas ou marco; o menu do cartão |
| `n` | Nova issue na coluna selecionada |
| `a`, `l`, `M`, `/` | Filtrar por responsável, etiqueta, marco, texto. `x` limpa os filtros |
| `c` | As etiquetas que são as colunas de um quadro do GitHub / GitLab |
| `r` | Ler as issues de novo |
| `Esc` | Fechar o quadro |

Rato: arraste um cartão para outra coluna para o mover, clique num cartão para o selecionar, clique duas vezes para o abrir, clique com o botão direito para
o menu do cartão, roda para deslocar uma coluna, `Shift`+roda (ou a barra sob o quadro) para deslocar as colunas.

### A caixa de texto Markdown e a vista de conversa

Cada caixa cujo texto vai para um host (um comentário, uma resposta, um resumo de revisão, a descrição de uma nova issue
ou de um pull request) é a mesma caixa de texto. As suas teclas estão na tabela `md_editor` acima e podem ser reatribuídas.

| Tecla | Ação |
|---|---|
| `Alt+P` | Alternar **Escrever** e **Prévia** (a partir de **Lado a lado** vai para a Prévia) |
| `Alt+L` | **Lado a lado** ligado, ou de volta a Escrever (precisa de uma janela com 100 colunas) |
| `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` | Enviar o texto (o botão do próprio formulário faz o mesmo) |
| `Alt+E` | **Escrever com IA**: abre ou fecha o painel sob a barra de modos (só com a IA ativa) |
| `Alt+R` | Só o compositor de commits: inserir uma referência de issue |
| `Alt+W` | Os formulários de nova issue e de novo pull request: criar na própria página web do fornecedor |
| `Esc` | No painel de **Escrever com IA**: parar um rascunho em curso, senão fechar o painel |

Na [vista de conversa](conversations.md) de uma issue ou de um pedido (linhas da paleta
*Conversa: ...*; as linhas com tecla estão na lista da paleta acima):

| Tecla | Ação |
|---|---|
| `r` | Responder à entrada selecionada (no seu tópico, ou com o seu texto citado num host sem tópicos) |
| `Q` | Citar a entrada selecionada num novo comentário |
| `c` | Novo comentário |
| `e`, `d` | Editar, eliminar um dos seus comentários |
| `Alt+I` | Na caixa de comentário: **Nota interna** (GitLab, Redmine) |
| `l`, `y`, `o` | Ligações da entrada, copiar o seu endereço, abri-la no navegador |
| `n` | Próxima entrada nova |
| `v`, `m`, `F5` | Mostrar ou ocultar eventos, carregar entradas mais antigas, atualizar |
| `g` | Abrir o diagrama da entrada (um bloco `mermaid`) |
| `Alt+Esquerda`, `Backspace` | Voltar pela pilha de itens abertos a partir de referências |
| `Tab` | Cronologia, caixa de comentário, botões do cabeçalho |
| `q`, `Esc` | Fechar |

No Radar, `i` abre a conversa de uma linha de issue.

Numa linha de pull request ou de issue das secções da barra lateral, `Enter` abre a conversa e `o` abre a linha no navegador.

### Flow (fluxos de trabalho de branches)

Com um fluxo de trabalho definido para o repositório (consulte [fluxos de trabalho de branches](branching-workflows.md)), a barra de
ferramentas tem um botão **Flow** e a barra lateral agrupa LOCAL e REMOTE por tipo de branch. Cada linha do menu é um comando
da paleta (**Flow: ...**; **Flow: iniciar feature...** e **Flow: nova issue de feature...** são gerados para cada tipo de tópico do
fluxo de trabalho). As linhas guiadas pelo host (*Nova issue de <tipo>*, *Assumir uma branch criada pelo host*) e os seus diálogos, o cartão
de espera e o cartão «apareceu uma branch» estão descritos em [fluxos de trabalho de branches](branching-workflows.md#fluxos-guiados-pelo-host-na-aplicação);
`Enter` executa o botão com foco (*Criar issue*, *Assumir*), `Esc` deixa de esperar ou fecha.

| Tecla | Ação |
|---|---|
| `Ctrl+G Shift+W` | Abrir o menu Flow (o botão da barra de ferramentas) |
| `Enter`, `Tab`, `Shift+Tab` | No diálogo Iniciar / Concluir / Sincronizar / Promover / Definir pai: pré-visualizar ou executar, campo seguinte, campo anterior |
| `Space` | Marcar uma caixa (*Publicar*, *Executar os hooks*, *Atualizar também as branches empilhadas sobre ela*) |
| `Esc` | Fechar uma lista, voltar da pré-visualização ao formulário, fechar o diálogo (não enquanto uma execução decorre) |
| Passar o rato sobre uma branch | O seu tipo, o pai e quantos commits está atrás do pai |

## Painel Log

| Tecla | Ação |
|---|---|
| `Enter` / `l` | Entrar na lista de ficheiros alterados do commit selecionado |
| `Enter` / `l` numa linha de ficheiro | Abrir o diff do ficheiro no centro |
| `Esc` / `h` | Voltar aos commits |
| `Right` | (commit com mais de uma ref) Abrir a lista das suas outras refs: `Up` / `Down` movem, `Enter` faz checkout, `Esc` fecha |
| `f` | Abrir a barra de pesquisa (consulte [Pesquisa](#pesquisa)); nunca foi `/`, pelo que a pesquisa do Log não muda |
| `/` | Pôr o foco no campo de filtro das Refs (a mesma tecla faz o mesmo quando o painel Refs tem o foco); escrita como caráter quando um campo de texto tem o foco |
| `c` | (com alterações por commitar) Selecionar a linha WIP e pôr o cursor no seu campo de mensagem |
| `Enter` na linha WIP | Pôr o cursor no campo de mensagem da linha WIP |
| `.`, `Menu`, `Shift+F10` | O menu do commit. `Up` / `Down` movem, `Right` / `l` / `Enter` abrem um submenu, `Left` / `h` / `Esc` voltam atrás |
| `m` / `a` | Ficheiros do painel direito: `Caminho` ou `Árvore` / `Ver todos os ficheiros` |
| `Shift+C` | Escolher uma coluna do Log para mover ou redimensionar (veja abaixo) |

Com uma coluna escolhida, `Tab` / `Shift+Tab` escolhem outra, `<` / `>` movem-na, `]` / `[` alargam-na ou estreitam-na, `0` repõe todas as larguras no automático e `Esc` ou `Enter` saem (contexto `log_columns` na tabela gerada acima).

O campo de mensagem da linha WIP é o resumo do compositor de commits. Enquanto tem
o cursor, todas as teclas são escritas, mas os acordes `Ctrl` continuam a funcionar. `Enter` passa para
a descrição do compositor. `Esc` sai do campo e mantém o texto. Consulte
[Staging e commits](staging-and-committing.md).

### Selecionar vários commits

| Tecla ou gesto | Ação |
|---|---|
| `Space` | Adicionar o commit atual à seleção, ou removê-lo |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Estender a seleção (Alt para os terminais que ficam com o Shift) |
| `Ctrl+clique` / `Shift+clique` ou `Alt+clique` | Alternar um commit / selecionar um intervalo |
| `Esc` | Limpar a seleção |
| `Shift+Y` | [Copiar a seleção como Mermaid gitGraph](log-and-history.md#copiar-commits-como-mermaid-gitgraph) (um popup com níveis de detalhe e pré-visualização em direto; `Enter` copia) |

Com vários commits selecionados, clique com o botão direito num deles e escolha **Cherry-pick de
N commits em `<branch>`...**. Abre a mesma vista que um rebase
interativo.

### Dar nome a uma branch ou tag

`Ctrl+G b`, `Ctrl+G t`, os botões **Branch** e **Tag** da barra de ferramentas e os
**Criar branch aqui...** / **Criar tag aqui...** dos menus pedem o nome numa
caixa de texto na linha do commit. `Enter` cria a ref. `Esc` cancela. Um nome
inválido ou já existente é mostrado a vermelho, e a caixa fica aberta.

Quando essa linha não está no ecrã (um diff aberto, o Log isolado, a coluna
Ref oculta), o nome escreve-se, em vez disso, numa pequena caixa com moldura,
com uma cruz de fechar e os botões **Cancelar** e **Criar**. Mudar o nome de uma
branch escreve-se sobre a sua linha no painel Refs, ou na célula Ref da sua
linha do Log; um ficheiro novo escreve-se na lista de Alterações em curso,
debaixo da pasta. As mesmas teclas valem em todo o lado: `Enter` confirma,
`Esc` cancela.

Nada se escreve na linha de estado. Os pedidos que precisam de mais espaço
são diálogos com botões a sério: a mensagem do stash, a mensagem de uma tag
anotada (`Ctrl+Enter` ou `Ctrl+S` aplica, `Enter` acrescenta uma linha),
Definir upstream (uma lista pesquisável de branches remotas, mais um botão
Remover upstream) e os padrões de acompanhamento e de fim de acompanhamento do
Git LFS. Uma definição numérica ou de texto escolhida na paleta abre as
Definições nessa linha, pronta a editar, e o filtro de Refs (`/`) escreve na
caixa no topo do painel Refs.

### Editar a mensagem de um commit

**Editar mensagem do commit...** no menu do commit, `Ctrl+G e`, ou o
`Git: editar mensagem do commit...` da paleta abre o popup **Reword commit message**. Consulte
[Trabalhar com o Log](log-and-history.md) para ver o que altera.

| Tecla | Ação |
|---|---|
| `Enter` no resumo | Ir para a descrição |
| `Enter` na descrição | Nova linha. `Up` na primeira linha volta ao resumo |
| `Ctrl+Enter`, `Ctrl+S`, ou **Atualizar mensagem** | Guardar a nova mensagem |
| `Tab` / `Shift+Tab` | Mover entre o resumo, a descrição e os botões |
| `Esc`, ou **Cancelar** | Fechar sem alterações |

### Alterar um commit

`Ctrl+G Shift+E` (todos os campos) e `Ctrl+G Shift+A` (o autor) abrem o diálogo Alterar, tal como o menu do commit, as linhas `Commit: alterar...` da paleta e o painel de detalhes do commit; todas as formas de o abrir estão descritas em [Alterar um commit](log-and-history.md#alterar-um-commit-mensagem-autor-datas). Os dois atalhos podem ser reatribuídos: `git_prefix.amend` e `git_prefix.edit_author`. Teclas dentro do diálogo:

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab` | Mover entre os campos e os botões |
| `Enter` num campo de uma linha | Campo seguinte; depois do último, **Aplicar** |
| `Up` / `Down` | Campo anterior / seguinte (na descrição, a partir da primeira / última linha) |
| `Ctrl+Enter`, `Ctrl+S`, ou **Aplicar** | Aplicar os campos alterados |
| `Esc`, ou **Cancelar** | Fechar sem alterações |
| `Alt+H`, ou **Modificar o histórico mais a fundo...** | Fechar o diálogo e abrir o painel de reescrita do histórico a partir deste commit (só nas compilações com reescrita do histórico); pergunta antes se editou um campo (`Esc` confirma). Reatribuível: `reword.deeper` |

### Criar worktree

**Criar worktree a partir de `<ref>`...** (menus de branch e tag) e **Criar
worktree aqui...** (menu do commit) abrem um diálogo: a pasta, uma nova
branch opcional, e **Abrir o novo worktree como separador**. Teclas: `Tab` / `Shift+Tab`
movem, `Enter` cria, `Space` alterna a caixa de verificação, `Ctrl+B` procura,
`Esc` cancela.

### Rebase interativo

**Rebase interativo a partir daqui...** no menu do commit, ou
`Git: rebase interativo a partir do commit selecionado...`, abre a vista de rebase. Um
cherry-pick de vários commits usa a mesma vista. Consulte
[Trabalhar com o Log](log-and-history.md).

| Tecla | Ação |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Definir a ação da linha: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup e Edit apenas num rebase) |
| `Space` | Abrir a lista de ações da linha (`Up` / `Down` escolhem, `Enter` aplica, `Esc` fecha a lista) |
| `Up` / `Down` | Mover a seleção |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Mover a linha selecionada para cima / para baixo |
| `a` | Ligar ou desligar a pré-visualização do autosquash |
| `Enter` numa linha Reword | Editar a nova mensagem desse commit |
| `Ctrl+S`, `Ctrl+Enter`, ou **Iniciar rebase** / **Iniciar cherry-pick** | Executar o plano |
| `Ctrl+R`, ou **Repor** | Repor o plano como estava quando a vista abriu |
| `Esc`, ou **Cancelar** | Fechar sem alterações |

Quando um rebase pára (num conflito ou num passo Edit), um aviso por cima do Log
oferece **Continuar**, **Saltar** e **Abortar**. A paleta tem
`Rebase: continuar`, `Rebase: saltar` e `Rebase: abortar`.

## Pesquisa

`f`, o botão **Pesquisar** da barra de ferramentas, ou `Navegação: pesquisar commits...` abre
a barra de pesquisa sobre o Log. Pesquisa a mensagem de commit completa, o nome e o e-mail
do autor, o SHA e os nomes das refs. Não oculta linhas: as linhas que
não correspondem ficam esbatidas.

| Tecla | Ação |
|---|---|
| *(escrever)* | Editar o texto de pesquisa (todas as teclas são escritas) |
| `Enter` / `Down`, ou o `↑` da barra | Correspondência seguinte |
| `Shift+Enter` / `Up`, ou o `↓` da barra | Correspondência anterior |
| `Esc`, ou o `✕` da barra | Fechar a barra de pesquisa |

## Vista Diff

Um ficheiro alterado abre no centro, no lugar do Log. Consulte
[Staging e commits](staging-and-committing.md).

| Tecla | Ação |
|---|---|
| `Enter` / `l`, ou um clique duplo (numa linha de ficheiro) | Abrir o diff do ficheiro |
| `Esc`, `q`, ou o `✕` no título | Fechar o diff |
| `j` / `k`, `Up` / `Down` | Deslocar |
| `PageUp` / `PageDown`, `Home` / `End` | Deslocar uma página; ir para o início / fim |
| `h` / `l`, `Left` / `Right` | Deslocar para os lados (não com quebra de linha) |
| `n` / `p` | Hunk seguinte / anterior |
| `m` | Disposição inline ou dividida |
| `Shift+H` | Só hunks ou ficheiro todo |
| `w` | Ligar ou desligar a quebra de linha |
| `s` | Mostrar ou ocultar alterações só de espaços em branco |
| `Shift+M` | Ligar ou desligar o minimapa |
| `Shift+C` | Escolher a codificação com que o ficheiro é lido (uma lista: Deteção automática primeiro) |
| `e` | Editar o ficheiro no editor integrado (consulte [Editor](#editor)) |
| `r` | (ficheiro Git LFS) O cartão LFS ou o ponteiro em bruto. Consulte [Git LFS](lfs.md) |
| `Shift+E` | (ficheiro em conflito) Abrir o editor de merge. Consulte [Resolver conflitos de merge](merge-conflicts.md) |
| `y` / `Ctrl+C` | Copiar o texto selecionado. Sem seleção: a linha sob o cursor, ou as linhas selecionadas |
| `.`, ou um clique com o botão direito num hunk | O menu do hunk |

### Staging e descarte a partir do diff

Estas teclas funcionam apenas num diff das alterações em curso (aberto a partir da
lista Unstaged ou Staged). Descartar funciona apenas na lista Unstaged.

| Tecla | Ação |
|---|---|
| `Space` | Fazer stage (Unstaged) / unstage (Staged) do hunk sob o cursor |
| `v` | Iniciar ou largar uma seleção de linhas; `j` / `k` estendem-na |
| `Enter` | Fazer stage / unstage das linhas selecionadas, ou da linha sob o cursor |
| `x` | Descartar o hunk sob o cursor (pergunta primeiro: `y` / `Enter` confirmam, `n` / `Esc` cancelam) |
| `Shift+X` | Descartar as linhas selecionadas, ou a linha sob o cursor (pergunta primeiro) |

Só contam as linhas alteradas: as linhas de contexto numa seleção são ignoradas. Uma
seleção que chega a um segundo hunk atua apenas no primeiro hunk.

## Vista de imagem

Uma imagem na vista Diff ou na vista de Ficheiro. As teclas estão no contexto
`image` da tabela acima. Consulte [Imagens](image-view.md).

## Vista de ficheiro, Blame e Histórico do ficheiro

| Tecla | Ação |
|---|---|
| `Esc` / `q` | Fechar (com uma seleção de texto, `Esc` limpa-a primeiro) |
| `j` / `k`, `Up` / `Down` | Mover a seleção |
| `PageUp` / `PageDown` | Deslocar uma página |
| `Home` / `End`, `Shift+G` | Primeira / última linha |
| `h` / `l`, `Left` / `Right` | Deslocar o texto para os lados |
| `Enter` | (Blame, Histórico do ficheiro) Ir para o commit da linha no Log |
| `e` | Editar o ficheiro no editor integrado |
| `y` / `Ctrl+C` | Copiar o texto selecionado. Sem seleção: a linha selecionada (Blame: apenas o código; Histórico do ficheiro: o resumo) |

O Blame e o histórico do ficheiro têm teclas próprias, listadas na tabela gerada acima (contextos `file_insight` e `history`) e como `Blame: ...` / `Histórico do ficheiro: ...` na paleta; todas podem ser remapeadas. Detalhes em [Blame](log-and-history.md#blame) e [Histórico do ficheiro](log-and-history.md#histórico-do-ficheiro). Na lista do histórico, as letras soltas (`d` `o` `b` `c` `y`) atuam sobre a linha selecionada, mas um acorde com Ctrl (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) mantém sempre o seu significado global.

## Coluna de commits

A coluna à esquerda da vista de um ficheiro (veja [A coluna de commits](interface-tour.md#a-coluna-de-commits)). `Alt+C` mostra-a ou oculta-a a partir de qualquer lugar; as outras teclas funcionam quando ela tem o teclado (`Ctrl+N` e depois o seu selo, ou `Ver: focar a coluna de commits`).

| Tecla | Ação |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Escolher um commit |
| `Enter` / `Right` | Passar à lista de ficheiros do commit escolhido |
| `Esc` | Devolver o teclado à vista |
| `.`, `Menu`, `Shift+F10` | O menu da linha escolhida |

As outras teclas (`Shift+Left` / `Shift+Right` para deslocar para os lados, `Tab`, `]`, `[` e `0` para dimensionar as partes) estão na tabela gerada acima, contexto `commit_column`.

## Editor

**Editar**, ou `e` num diff ou numa vista de Ficheiro, abre o editor integrado na
cópia do ficheiro no diretório de trabalho. Consulte
[Staging e commits](staging-and-committing.md).

| Tecla | Ação |
|---|---|
| *(escrever)* | Inserir no cursor (substitui a seleção) |
| `Enter` | Nova linha, com a indentação da linha atual |
| `Tab` / `Shift+Tab` | Indentar / desindentar (as linhas selecionadas, se forem várias) |
| setas, `Home` / `End` | Mover. `Home` vai para o primeiro carácter não vazio, depois para a coluna 0. `Shift` estende a seleção |
| `Ctrl+Left` / `Ctrl+Right` | Mover uma palavra |
| `Ctrl+Home` / `Ctrl+End` | Início / fim do ficheiro |
| `PageUp` / `PageDown` | Uma página para cima / para baixo |
| `Ctrl+A` | Selecionar tudo |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copiar / cortar / colar |
| `Ctrl+U` / `Ctrl+K` | Apagar até ao início / fim da linha |
| `Ctrl+Z` | Desfazer |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Refazer |
| `Ctrl+S` | Guardar |
| `Ctrl+F` / `F3` | Pesquisar. No campo de pesquisa: `Enter` / `Down` / `F3` seguinte, `Shift+Enter` / `Up` anterior, `Esc` fecha |
| `Alt+M` | Ligar ou desligar o minimapa (**Map**) |
| `Esc` | Fechar. Com alterações por guardar pergunta: **Guardar** (`S` / `Enter`), **Descartar** (`D`), **Cancelar** (`C` / `Esc`) |

`Tab` indenta no editor, por isso não move o foco. Os outros acordes `Ctrl`
(`Ctrl+P`, `Ctrl+N`) continuam a funcionar.

A **codificação**, as **quebras de linha** e a **indentação** da barra de estado são
botões: cada um abre uma lista pendente (setas ou roda, escreva para filtrar, `Enter`
para escolher, `Esc` para fechar). A paleta (`Ctrl+P`) abre os mesmos menus: *Editor: alterar codificação*, *Editor: reabrir com codificação*, *Editor: converter para codificação*, *Editor: Converter quebras de linha*, *Editor: alterar indentação*.
Detalhes em [Staging e commits](staging-and-committing.md).

## Ações Git (`Ctrl+G`)

`Ctrl+G` espera mais uma tecla:

| Tecla | Ação |
|---|---|
| `s` / `u` | Fazer stage / unstage do ficheiro selecionado |
| `a` | Fazer stage de todas as alterações |
| `x` | Descartar todas as alterações (pergunta primeiro) |
| `c` | Abrir o compositor de commits |
| `Enter` | Fazer agora o commit das alterações em stage, com a mensagem do formulário, a partir de qualquer lugar (veja [Stage e commit](staging-and-committing.md#escrever-o-commit)) |
| `m` | Fazer stash de todas as alterações com uma mensagem, escrita num pequeno diálogo (vazia é permitido). Com a IA ativa, o seu botão **Mensagem da IA** (`Ctrl+G` no diálogo) escreve a mensagem |
| `Shift+T` | Fazer stash de todas as alterações (ficheiros não acompanhados incluídos), sem mensagem |
| `o` | Fazer pop de um stash: a linha de stash selecionada no Log, ou então o último stash |
| `p` | Pull, conforme o botão Pull está configurado (`git.pull_mode`) |
| `Shift+P` | Push para o remoto predefinido (define o upstream no primeiro push), ou pergunta o remoto |
| `Shift+U` | Fazer push de todas as tags |
| `f` | Fazer fetch de todos os remotos, com prune |
| `g` | Abrir no navegador o pull request da branch em checkout |
| `b` / `t` | Nova branch / nova tag em HEAD (consulte [Dar nome a uma branch ou tag](#dar-nome-a-uma-branch-ou-tag)) |
| `d` | Eliminar a branch ou tag selecionada (pergunta primeiro) |
| `e` | Editar a mensagem do commit selecionado |
| `Shift+E` / `Shift+A` | Alterar o commit selecionado (todos os campos) / o seu autor (veja [Alterar um commit](#alterar-um-commit)) |
| `y` | Cherry-pick do commit selecionado |
| `v` | Revert do commit selecionado |
| `Shift+R` | Reset (mixed) para o commit selecionado |
| `r` | Rebase sobre a branch selecionada |
| `Shift+W` | Abrir o menu Flow (num repositório com um fluxo de trabalho de branches) |
| `Shift+H` | Abrir o painel de reescrita do histórico (só nas compilações com reescrita do histórico) |
| `l` | Abrir o menu LFS. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch de tudo / checkout de tudo / pull / push de tudo / prune. Consulte [Git LFS](lfs.md) |
| `i` | ✦ IA: escrever a mensagem de commit para as alterações em stage |
| `w` | ✦ IA: explicar o commit selecionado, ou as alterações em curso |
| `Shift+C` | ✦ IA: propor como dividir as alterações em curso em commits |

As teclas de IA só funcionam enquanto a IA está ativa (consulte [Assistência de IA](ai.md)). `Esc`
cancela um pedido de IA em curso.

## Painel direito (alterações em curso e detalhes do commit)

Com a linha WIP selecionada, o painel direito mostra as listas **Unstaged** e
**Staged** e o compositor de commits. Com um commit selecionado, mostra os
detalhes do commit e os seus ficheiros.

| Tecla | Ação |
|---|---|
| `j` / `k`, `Up` / `Down` | Mover. No fim de uma lista, a seleção continua para a outra |
| `h` / `l`, `Left` / `Right` | Ir para a lista Unstaged / Staged |
| `Enter` | Recolher ou expandir a lista ativa |
| `Space` / `Insert` | Adicionar o ficheiro atual à seleção, ou removê-lo |
| `Ctrl+A` | Selecionar todos os ficheiros da lista |
| `Ctrl+Space` | Fazer stage / unstage do ficheiro selecionado, ou recolher uma linha de pasta |
| `d` | Abrir o diff do ficheiro selecionado |
| `m` | `Caminho` (lista plana) ou `Árvore` (pastas) |
| `a` | `Ver todos os ficheiros`: a árvore toda, não apenas os ficheiros alterados |
| `z` | Recolher todas as pastas, ou expandi-las de novo |
| `/` | Filtrar por nome os ficheiros alterados de um commit |
| `.`, `Menu`, `Shift+F10` | O menu do ficheiro |
| `c` | Focar o campo de resumo do compositor de commits |
| `Shift+A` | Ligar ou desligar **Amend do commit anterior** |
| `Ctrl+N ]` / `Ctrl+N [` | Aumentar / reduzir o bloco da mensagem de commit |

No compositor de commits:

| Tecla | Ação |
|---|---|
| `Enter` | Resumo: ir para a descrição. Descrição: nova linha |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` fá-lo a partir de qualquer lugar) |
| `Alt+R` | Inserir uma referência de issue |
| `Esc` | Sair do compositor |

O menu do ficheiro contém: Stage ou Unstage, Descartar alterações, Ignorar, Stash do ficheiro,
Histórico do ficheiro, Blame do ficheiro, Abrir na ferramenta de diff externa, Abrir no editor externo,
Abrir ficheiro no programa predefinido, Mostrar na pasta, Copiar caminho do ficheiro, Criar patch
das alterações do ficheiro, Editar ficheiro e Eliminar ficheiro. Num ficheiro em conflito começa
com **Abrir no editor de merge** e as três linhas **Resolver com**. Com o Git
LFS ativo, tem linhas de LFS. **Descartar alterações** e **Eliminar ficheiro** perguntam primeiro.

## Botões da janela

Cada botão tem um equivalente de teclado:

| Botão | Tecla |
|---|---|
| <img src="../../../assets/logo/gitpoulpi.png" alt="Ícone do GitPoulpi" height="16"> à esquerda da barra de título (Acerca de) | `Ajuda: acerca do GitPoulpi` na paleta |
| **Abrir em** na barra de título | `Alt+O`, `Ctrl+Alt+O` |
| `×` à direita da barra de título (sair) | `q` |
| A versão na barra de título (Notas de versão) | `Ajuda: notas de versão` na paleta |
| `⌕ Pesquisa global` na barra de título, à esquerda da `⚙` | `Ctrl+Alt+F` |
| `⚙` na barra de título (Definições) | `Ctrl+,` |
| A etiqueta da área de trabalho na linha de separadores | `Alt+W` |
| `+` a seguir ao último separador | `Alt+T` |
| O separador **Radar** | `Alt+R` |
| O separador **Áreas de trabalho** | `Alt+J` |
| **Atividade** na barra de estado | `Ctrl+N Shift+A` |
| Barra de ferramentas: **Desfazer**, **Refazer** | `Ctrl+Z`, `Ctrl+Y` |
| Barra de ferramentas: **Pull** / o seu `▾` | `Ctrl+G p` / `Git: opções de pull...` |
| **Pull de todas as branches** (linha do `▾` de Pull) | `Ctrl+G Ctrl+P` |
| Barra de ferramentas: **Push** / o seu `▾` | `Ctrl+G Shift+P` / `Git: push para um remoto...` |
| Barra de ferramentas: **Branch**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Barra de ferramentas: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Barra de ferramentas: **LFS** (quando o LFS está ativo) | `Ctrl+G l` |
| Barra de ferramentas: **Flow** (quando o repositório tem um fluxo de trabalho) | `Ctrl+G Shift+W` |
| Barra de ferramentas: **Pesquisar** | `f` |
| Barra de ferramentas: o pull request ligado (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Barra de ferramentas: a lista de perfis (com dois ou mais perfis) | `Repositório: alterar perfil...` |

A barra de ferramentas está descrita em [Visita guiada à interface](interface-tour.md#barra-de-ferramentas).

## Definições

`Ctrl+,`, `Ctrl+N ,`, a roda dentada `⚙`, ou `Aplicação: definições...` abre
o ecrã de Definições.

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Mover entre a lista de páginas e as opções |
| `j` / `k`, `Up` / `Down` | Mover a seleção |
| `Enter` / `Space` | Abrir uma página, alternar uma caixa de verificação, editar um campo, abrir uma lista pendente, ou premir um botão |
| `/`, `Ctrl+F` | Pesquisar em todas as opções |
| `Alt+Left` / `Alt+Right` | Mudar de subseparador (Geral / a lista) |
| `Esc` / `q` | Fechar as Definições. Enquanto um campo é editado, `Esc` cancela apenas a edição |
| `?` | Abrir a ajuda por cima do ecrã de Definições |

Consulte [Definições](settings.md).

## Campos de texto

Todos os campos de texto partilham as mesmas teclas: filtros, campos de pesquisa, a paleta de
comandos, pedidos, o compositor de commits, o campo da linha WIP, campos de diálogos e
campos das Definições. Enquanto um campo tem o teclado, estas teclas prevalecem sobre os
atalhos dos painéis.

| Tecla | Ação |
|---|---|
| um carácter | Escrevê-lo no cursor (substitui a seleção) |
| `Left` / `Right` | Mover um carácter (com uma seleção: ir para o seu início / fim) |
| `Ctrl+Left` / `Ctrl+Right` | Mover uma palavra |
| `Home` / `End` | Início / fim da linha |
| `Shift` + um movimento acima | Estender ou reduzir a seleção |
| `Up` / `Down` (apenas descrição) | Linha anterior / seguinte |
| `Ctrl+Home` / `Ctrl+End` (apenas descrição) | Início / fim do texto |
| `Ctrl+A` | Selecionar tudo |
| `Backspace` / `Delete` | Apagar o carácter antes / depois do cursor, ou a seleção |
| `Ctrl+U` | Apagar do cursor ao início da linha (ou a seleção) |
| `Ctrl+K` | Apagar do cursor ao fim da linha (ou a seleção) |
| `Ctrl+C` / `Ctrl+X` | Copiar / cortar a seleção (não num campo de palavra-passe) |
| colar (a colagem do próprio terminal) | Inserir no cursor, substituindo a seleção |

Na descrição do commit, `Enter` escreve uma nova linha e `Ctrl+Enter` faz commit.
Nem todos os terminais enviam `Ctrl+Enter` da mesma forma, por isso `Alt+Enter` e
`Ctrl+J` também fazem commit.

Algumas consolas enviam `Ctrl+letra` como um carácter de controlo (`Ctrl+A` como `0x01`).
O GitPoulpi compreende ambas as formas. Os carateres `AltGr` (`@`, `#`, `€` em teclados
suíços ou franceses) são escritos.

## Dicas

Uma dica aparece quando o ponteiro fica cerca de 0,6 s sobre um alvo (uma célula do Log,
um botão da barra de ferramentas, o pull request ligado). Premir uma tecla, um clique, uma deslocação, um
redimensionamento, um popup ou menu aberto, ou o ponteiro sair do alvo oculta-a.
Depois disso, só volta quando o ponteiro se mexe de novo.
