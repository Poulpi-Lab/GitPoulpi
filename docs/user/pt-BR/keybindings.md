[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · **Português (Brasil)** · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Atalhos de teclado

Esta página lista as teclas **padrão**. Você pode reatribuir a maioria delas. O app
sempre mostra as teclas em vigor: na paleta de comandos, na ajuda `?`, nas
linhas de status, nos menus de contexto e nas dicas dos botões. Quando você muda uma tecla, esses
lugares mostram a nova tecla. Para o mouse, veja
[Mouse e gestos](mouse-and-gestures.md).

## Reatribuindo atalhos

Abra **Configurações > Teclado e mouse**. A página é descrita em
[Configurações](settings.md#teclado-e-mouse). As mudanças valem na hora e são salvas
na tabela `[keybindings]` do `config.toml`. Você também pode editar essa tabela
manualmente:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux e GNU screen costumam usar Ctrl+B e Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

A sintaxe, as regras de conflito e os avisos estão na
[Referência de configuração](config-reference.md) em `[keybindings]`. Em resumo:

- Um erro é informado na linha de status, e a tecla padrão permanece.
- Duas ações no mesmo acorde no mesmo contexto são recusadas.
- Um prefixo precisa de `Ctrl` ou `Alt`.
- Um acorde de painel que esconde um acorde global gera apenas um aviso.

A tabela abaixo lista todas as teclas que você pode reatribuir. Algumas teclas são fixas e não podem
ser reatribuídas: o movimento por listas (`j` `k` `h` `l`, as setas, `PageUp` / `PageDown`,
`Home` / `End`), `Enter`, `Esc`, `Tab`, os dígitos (o prefixo de contagem), as
teclas de edição de texto e as teclas `Enter` / `Tab` / setas dos popups. A paleta
tem `App: atalhos de teclado...` e `Atalhos: redefinir tudo...`.

## Atalhos padrão

Todas as teclas que você pode reatribuir, com o id a usar em `[keybindings.<contexto>]` e
o valor padrão. A tabela é gerada a partir do app (um teste a mantém atualizada).
As teclas depois de um prefixo aparecem com o prefixo (`Ctrl+G s`).

<!-- keymap:begin -->
| Contexto | Id da ação | O que faz | Teclas padrão |
|---|---|---|---|
| `global` | `git_prefix` | Prefixo das ações do Git | `Ctrl+G` |
| `global` | `nav_prefix` | Prefixo de navegação | `Ctrl+N` |
| `global` | `palette` | Paleta de comandos | `Ctrl+P` |
| `global` | `settings` | Configurações | `Ctrl+,` |
| `global` | `home` | Início (gerenciamento de repositórios) | `Ctrl+O` |
| `global` | `clone` | Clonar repositório | `Ctrl+Shift+N` |
| `global` | `new_tab` | Nova aba | `Alt+T` |
| `global` | `workspace_switch` | Trocar de espaço de trabalho | `Alt+W` |
| `global` | `radar` | Abrir Radar | `Alt+R` |
| `global` | `workspaces` | Abrir espaços de trabalho | `Alt+J` |
| `global` | `tab_menu` | Menu da aba | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Abrir em (lista) | `Alt+O` |
| `global` | `tab_open_in_default` | Abrir no padrão | `Ctrl+Alt+O` |
| `global` | `close_tab` | Fechar aba | `Ctrl+W` |
| `global` | `next_tab` | Próxima aba | `Ctrl+PageDown` |
| `global` | `prev_tab` | Aba anterior | `Ctrl+PageUp` |
| `global` | `help` | Ajuda | `?` |
| `global` | `quit` | Sair | `q` |
| `global` | `refresh` | Atualizar | `r` |
| `global` | `refresh_all` | Atualizar (fetch, pull requests, issues) | `F5` |
| `global` | `find_in_log` | Buscar no grafo de commits | `f` |
| `global` | `workspace_search` | Buscar no espaço de trabalho | `Ctrl+Alt+F` |
| `global` | `select_tab` | Selecionar aba | `t` |
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
| `git_prefix` | `stage_file` | Stage do arquivo selecionado | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage do arquivo selecionado | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Abrir pull request vinculado | `Ctrl+G g` |
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
| `git_prefix` | `delete_ref` | Excluir a ref selecionada ou descartar stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Abrir o menu LFS | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Abrir o menu Flow | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch de todos os objetos | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: checkout de todos os arquivos | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull de objetos | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push de todos os objetos | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: limpar objetos locais | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Abrir o painel de reescrita do histórico | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Recolher ou restaurar o painel Refs | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Recolher ou restaurar o painel Status | `Ctrl+N l` |
| `nav_prefix` | `settings` | Abrir configurações | `Ctrl+N ,` |
| `nav_prefix` | `about` | Abrir Sobre | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Abrir o Registro de atividade | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Abrir o registro de atividade global | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Mostrar as tarefas em segundo plano | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Mostrar o painel de atualização | `Ctrl+N u` |
| `nav_prefix` | `help` | Alternar ajuda | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Selecionar aba | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | Mudar tema | `Ctrl+N y` |
| `nav_prefix` | `change_language` | Mudar idioma | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | Mudar tema de cores do grafo | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | Alternar tom das linhas do grafo | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Aumentar a seção Refs | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Diminuir a seção Refs | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Aumentar o bloco de mensagem do commit | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Diminuir o bloco de mensagem do commit | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Alargar o painel Refs | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Estreitar o painel Refs | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Alargar o painel Status | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Estreitar o painel Status | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agente: mostrar a visualização pendente | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agente: voltar (ou dispensar a visualização pendente) | `Ctrl+N Backspace` |
| `refs` | `filter` | Filtrar refs por nome | `/` |
| `refs` | `solo` | Solo na seleção | `o` |
| `refs` | `clear_solo` | Parar o solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Alternar a hierarquia de branches | `y` |
| `refs` | `show_all` | Mostrar todas as refs ocultas | `Shift+H` |
| `refs` | `reveal` | Mostrar no grafo | `g` |
| `refs` | `mark_drop` | Marcar como origem do soltar | `m` |
| `refs` | `drop_marked` | Soltar aqui a ref marcada | `Shift+M` |
| `refs` | `expand_all` | Expandir todas as pastas | `=` |
| `refs` | `collapse_all` | Recolher todas as pastas | `-` |
| `refs` | `expand_below` | Expandir a pasta e suas subpastas | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Recolher a pasta e suas subpastas | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Arquivos: caminho ou árvore | `m` |
| `log` | `view_all_files` | Arquivos: ver todos os arquivos | `a` |
| `log` | `compare_selection` | Comparar as linhas selecionadas | `v` |
| `log` | `compare_swap` | Trocar as extremidades da comparação | `x` |
| `log` | `compare_next` | Ir para a próxima linha comparada | `n` |
| `log` | `pick_column` | Escolher uma coluna para mover ou redimensionar | `Shift+C` |
| `log` | `scroll_left` | Rolar as colunas para a esquerda | `Shift+Left` |
| `log` | `scroll_right` | Rolar as colunas para a direita | `Shift+Right` |
| `log` | `copy_mermaid` | Copiar Commits como gitGraph do Mermaid | `Shift+Y` |
| `log_columns` | `next` | Escolher a próxima coluna | `Tab` |
| `log_columns` | `prev` | Escolher a coluna anterior | `Shift+BackTab` |
| `log_columns` | `move_left` | Mover a coluna para a esquerda | `<` |
| `log_columns` | `move_right` | Mover a coluna para a direita | `>` |
| `log_columns` | `wider` | Alargar a coluna | `]` |
| `log_columns` | `narrower` | Estreitar a coluna | `[` |
| `log_columns` | `reset_width` | Redefinir as larguras das colunas | `0` |
| `log_columns` | `fit` | Ajustar a coluna ao conteúdo | `=` |
| `log_columns` | `fit_all` | Ajustar todas as colunas ao conteúdo | `+` |
| `log_columns` | `layout` | Rolar para os lados ou ajustar ao painel | `s` |
| `log_files` | `collapse_all` | Recolher ou expandir todas as pastas | `z` |
| `log_files` | `filter` | Filtrar os arquivos alterados | `/` |
| `log_files` | `expand_all` | Expandir todas as pastas | `=` |
| `log_files` | `collapse_all_folders` | Recolher todas as pastas | `-` |
| `log_files` | `expand_below` | Expandir a pasta e suas subpastas | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Recolher a pasta e suas subpastas | `_`, `Shift+Left` |
| `status` | `compose` | Focar a mensagem do commit | `c` |
| `status` | `toggle_file_mode` | Arquivos: caminho ou árvore | `m` |
| `status` | `view_all_files` | Arquivos: ver todos os arquivos | `a` |
| `status` | `amend` | Alternar amend | `Shift+A` |
| `status` | `skip_hooks` | Alternar Ignorar hooks | `Shift+H` |
| `status` | `collapse_all` | Recolher ou expandir todas as pastas | `z` |
| `status` | `stage_toggle` | Stage ou unstage da linha selecionada | `Ctrl+Space` |
| `status` | `open_diff` | Abrir o diff do arquivo selecionado | `d` |
| `status` | `expand_all` | Expandir todas as pastas | `=` |
| `status` | `collapse_all_folders` | Recolher todas as pastas | `-` |
| `status` | `expand_below` | Expandir a pasta e suas subpastas | `*`, `Shift+Right` |
| `status` | `collapse_below` | Recolher a pasta e suas subpastas | `_`, `Shift+Left` |
| `home` | `favorite` | Marcar ou desmarcar o repositório como favorito | `f`, `*` |
| `home` | `close_repo` | Fechar, remover ou desmarcar a linha | `x`, `Delete` |
| `home` | `browse` | Procurar um repositório | `b` |
| `home` | `clone` | Clonar um repositório | `c` |
| `home` | `init` | Inicializar um repositório | `i` |
| `home` | `collapse_all` | Recolher todas as seções | `-` |
| `home` | `expand_all` | Expandir todas as seções | `+`, `=` |
| `home` | `search` | Buscar nos repositórios | `/` |
| `home` | `workspace_new` | Novo espaço de trabalho | `n` |
| `home` | `workspace_edit` | Editar o espaço de trabalho | `e` |
| `home` | `workspace_from_tabs` | Novo espaço a partir das abas abertas | `a` |
| `home` | `workspace_fetch` | Fetch tudo no espaço de trabalho | `Shift+F` |
| `home` | `workspace_pull` | Pull tudo no espaço de trabalho | `Shift+P` |
| `home` | `workspace_clone_missing` | Clonar os repositórios ausentes | `m` |
| `home` | `workspace_locate` | Localizar o repositório ausente | `l` |
| `refs` | `hosting_filter` | Pull requests / issues: escolher o filtro | `v` |
| `refs` | `hosting_edit` | Pull request: editar no host | `e` |
| `refs` | `new_pull_request` | Pull requests: criar uma | `n` |
| `refs` | `add_remote` | Remotos: adicionar um remoto | `+` |
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
| `diff` | `toggle_hunks_only` | Só hunks ou arquivo todo | `Shift+H` |
| `diff` | `toggle_minimap` | Alternar o minimapa | `Shift+M` |
| `diff` | `choose_encoding` | Escolher a codificação | `Shift+C` |
| `diff` | `toggle_wrap` | Alternar quebra de linha | `w` |
| `diff` | `toggle_whitespace` | Mostrar ou ocultar alterações só de espaços em branco | `s` |
| `diff` | `merge_editor` | Abrir o editor de merge (arquivo em conflito) | `Shift+E` |
| `file_insight` | `close` | Fechar a visualização do arquivo | `q` |
| `file_insight` | `edit` | Editar no diretório de trabalho | `e` |
| `file_insight` | `copy` | Copiar a seleção ou a linha | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Ir para a última linha | `Shift+G` |
| `file_insight` | `open_commit` | Ir para o commit da linha | `Enter` |
| `history` | `show_in_log` | Mostrar no Log | `Enter` |
| `history` | `copy_sha` | Copiar o SHA do commit | `y` |
| `history` | `copy_summary` | Copiar o resumo | `Shift+Y` |
| `history` | `show_diff` | Mostrar o diff deste commit | `d`, `Alt+1` |
| `history` | `open_file` | Abrir o arquivo neste commit | `o`, `Alt+2` |
| `history` | `blame` | Blame neste commit | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Comparar com a árvore de trabalho | `c` |
| `history` | `restore` | Restaurar esta versão | `Shift+R` |
| `history` | `range_up` | Estender a seleção para cima | `Shift+Up` |
| `history` | `range_down` | Estender a seleção para baixo | `Shift+Down` |
| `history` | `switch_focus` | Alternar entre a lista e o diff | `Tab` |
| `history` | `widen_list` | Aumentar a largura da lista | `Alt+Right` |
| `history` | `narrow_list` | Diminuir a largura da lista | `Alt+Left` |
| `history` | `reset_split` | Redefinir a largura da lista | `Alt+0` |
| `commit_column` | `scroll_left` | Rolar a coluna para a esquerda | `Shift+Left` |
| `commit_column` | `scroll_right` | Rolar a coluna para a direita | `Shift+Right` |
| `commit_column` | `next_part` | Dimensionar a próxima parte | `Tab` |
| `commit_column` | `move_left` | Mover a parte para a esquerda | `<` |
| `commit_column` | `move_right` | Mover a parte para a direita | `>` |
| `commit_column` | `wider_part` | Alargar a parte | `]` |
| `commit_column` | `narrower_part` | Estreitar a parte | `[` |
| `commit_column` | `reset_widths` | Redefinir as larguras das partes | `0` |
| `file_insight` | `blame_card` | Blame: mostrar ou ocultar o cartão de detalhes | `i` |
| `file_insight` | `blame_next` | Blame: próximo hunk | `n` |
| `file_insight` | `blame_prev` | Blame: hunk anterior | `p` |
| `file_insight` | `blame_next_same` | Blame: próximo hunk do mesmo commit | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: hunk anterior do mesmo commit | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: ir para a linha | `g` |
| `file_insight` | `blame_copy_sha` | Blame: copiar o SHA do commit | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: diff do commit para este arquivo | `d` |
| `file_insight` | `blame_parent` | Blame: blame no pai do commit | `b` |
| `file_insight` | `blame_history` | Blame: abrir o histórico do arquivo | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: aumentar coluna de anotações | `>` |
| `file_insight` | `blame_narrower` | Blame: diminuir coluna de anotações | `<` |
| `image` | `zoom_in` | Aumentar a imagem | `+`, `=` |
| `image` | `zoom_out` | Diminuir a imagem | `-` |
| `image` | `fit` | Ajustar a imagem à visualização | `0` |
| `image` | `actual_size` | Mostrar a imagem em tamanho real | `1` |
| `image` | `flip_side` | Mostrar o outro lado (antes / depois) | `Tab` |
| `image` | `widen_before` | Alargar o painel Antes | `Alt+Right` |
| `image` | `widen_after` | Alargar o painel Depois | `Alt+Left` |
| `image` | `reset_split` | Redefinir a divisão antes / depois | `Alt+0` |
| `image` | `edit_in` | Abrir com... | `Shift+O` |
| `image` | `edit_default` | Abrir no aplicativo padrão | `o` |
| `editor` | `save` | Salvar | `Ctrl+S` |
| `editor` | `undo` | Desfazer | `Ctrl+Z` |
| `editor` | `redo` | Refazer | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Buscar | `Ctrl+F` |
| `editor` | `find_next` | Buscar próximo (Shift: anterior) | `F3` |
| `editor` | `paste` | Colar | `Ctrl+V` |
| `editor` | `toggle_minimap` | Alternar o minimapa | `Alt+M` |
| `editor_prompt` | `save` | Alterações não salvas: salvar e fechar | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Alterações não salvas: descartar e fechar | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Alterações não salvas: continuar editando | `c`, `Shift+C` |
| `merge` | `next_conflict` | Próximo conflito | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Conflito anterior | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Próximo arquivo em conflito | `Alt+PageDown` |
| `merge` | `prev_file` | Arquivo em conflito anterior | `Alt+PageUp` |
| `merge` | `next_pane` | Focar o próximo painel | `F6` |
| `merge` | `prev_pane` | Focar o painel anterior | `Shift+F6` |
| `merge` | `toggle_panes` | Mostrar ou ocultar os painéis laterais (Refs e Status) | `F9` |
| `merge` | `toggle_base` | Mostrar ou ocultar o painel Base | `Alt+D` |
| `merge` | `toggle_full_file` | Alternar diff do arquivo todo | `Alt+F` |
| `merge` | `undo` | Desfazer (escolhas e digitação) | `Ctrl+Z` |
| `merge` | `redo` | Refazer (escolhas e digitação) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Alargar o painel Atual (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Alargar o painel Recebido (B) | `Alt+Left` |
| `merge` | `reset_panes` | Redefinir os tamanhos dos painéis | `Alt+0` |
| `merge` | `save` | Salvar sem marcar como resolvido | `Alt+S` |
| `merge` | `save_resolve` | Salvar e marcar como resolvido | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Sugerir uma resolução (IA) | `Alt+I` |
| `merge` | `unpick` | Desfazer a escolha deste conflito (não resolvido) | `Alt+U` |
| `merge` | `reset_all` | Devolver todo o arquivo aos conflitos originais | `Alt+R` |
| `merge` | `focus_a` | Ir ao painel Atual (A) | `Alt+1` |
| `merge` | `focus_b` | Ir ao painel Recebido (B) | `Alt+2` |
| `merge` | `focus_output` | Ir ao painel Resultado | `Alt+3` |
| `merge` | `focus_base` | Ir ao painel Base | `Alt+4` |
| `merge_sides` | `toggle_line` | Escolher ou largar a linha | `Space` |
| `merge_sides` | `toggle_chunk` | Escolher ou largar o trecho todo | `Enter` |
| `merge_sides` | `take_a` | Aceitar só o Atual (A) | `a` |
| `merge_sides` | `take_b` | Aceitar só o Recebido (B) | `b` |
| `merge_sides` | `take_both` | Aceitar os dois, A e depois B | `c` |
| `merge_sides` | `take_both_reversed` | Aceitar os dois, B e depois A | `Shift+C` |
| `merge_sides` | `clear` | Não aceitar nada neste conflito | `x` |
| `merge_sides` | `unpick` | Desfazer a escolha deste conflito (não resolvido) | `u` |
| `merge_sides` | `reset_all` | Devolver todo o arquivo aos conflitos originais | `Shift+U` |
| `merge_sides` | `swap_order` | Trocar a ordem dos dois lados | `s` |
| `merge_sides` | `next_conflict` | Próximo conflito | `n` |
| `merge_sides` | `prev_conflict` | Conflito anterior | `Shift+N` |
| `merge_sides` | `take_all_a` | Aceitar todo o Atual (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Aceitar todo o Recebido (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Mostrar ou ocultar o painel Base | `d` |
| `merge_sides` | `scroll_left` | Rolar os painéis para a esquerda | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Rolar os painéis para a direita | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Alternar diff do arquivo todo | `f` |
| `merge_sides` | `cycle_height` | Alternar a altura do resultado | `z` |
| `merge_card` | `take_a` | Manter A (Atual) como está | `a` |
| `merge_card` | `take_b` | Aceitar B (Recebido) como está | `b` |
| `merge_card` | `take_base` | Aceitar a Base (ancestral comum) | `o` |
| `merge_card` | `keep` | Manter o arquivo que sobrou | `k` |
| `merge_card` | `delete` | Aceitar a exclusão | `Shift+D` |
| `merge_card` | `external_tool` | Abrir na ferramenta de merge externa | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (parar para amend) | `e` |
| `rebase` | `autosquash` | Alternar autosquash | `a` |
| `rebase` | `start` | Iniciar o plano | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Redefinir o plano | `Ctrl+R` |
| `rebase` | `split` | Dividir o commit da linha | `t` |
| `rebase` | `move_up` | Mover a linha para cima | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Mover a linha para baixo | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Criar os commits | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Alternar a seleção de linhas | `v` |
| `split` | `move_up` | Mover o commit para cima | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Mover o commit para baixo | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Redefinir a divisão | `Shift+R` |
| `rewrite` | `preview` | Gerar a prévia da reescrita | `F5` |
| `rewrite` | `apply` | Aplicar a reescrita | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Cancelar a prévia | `Ctrl+X` |
| `rewrite` | `add_rule` | Adicionar uma regra | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Ativar ou desativar a regra | `Space` |
| `rewrite` | `move_up` | Mover a regra para cima | `Alt+Up` |
| `rewrite` | `move_down` | Mover a regra para baixo | `Alt+Down` |
| `rewrite` | `delete_rule` | Excluir a regra | `d`, `Delete` |
| `rewrite` | `restore_rule` | Restaurar a regra excluída | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Duplicar a regra | `Ctrl+D` |
| `rewrite` | `next_tab` | Próxima aba da prévia | `Alt+Right` |
| `rewrite` | `prev_tab` | Aba anterior da prévia | `Alt+Left` |
| `rewrite` | `save_shared` | Salvar a receita no repositório | `Alt+S` |
| `rewrite` | `save_mine` | Salvar a receita só para mim | `Alt+M` |
| `rewrite` | `load` | Carregar uma receita | `Alt+L` |
| `rewrite` | `name` | Renomear a receita | `F2` |
| `rewrite` | `protected_all` | Incluir todas as branches protegidas | `Alt+A` |
| `rewrite` | `protected_none` | Deixar de fora todas as branches protegidas | `Alt+U` |
| `rewrite` | `check_tools` | Verificar as ferramentas de novo | `F6` |
| `rewrite` | `analyze` | Analisar o tamanho do histórico | `F7` |
| `rewrite` | `host_howto` | Explicar a proteção de branch do host | `Alt+H` |
| `rewrite` | `ai_rules` | Criar regras de reescrita com IA | `Alt+I` |
| `reword` | `apply` | Aplicar a nova mensagem | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Modificar o histórico mais a fundo | `Alt+H` |
| `composer` | `commit` | Commit pelo compositor | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Inserir uma referência de issue | `Alt+R` |
| `dialog` | `browse` | Procurar uma pasta | `Ctrl+B` |
| `dialog` | `sources` | Ir para as origens do clone | `Alt+S` |
| `dialog` | `submit` | Enviar o formulário de pull request | `Ctrl+S` |
| `dialog` | `web` | Abrir o formulário web do provedor | `Alt+W` |
| `folder_picker` | `choose` | Escolher a pasta | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Nova pasta | `Alt+N` |
| `folder_picker` | `filter` | Filtrar as entradas da pasta | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Mostrar ou ocultar pastas irmãs | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Mostrar ou ocultar pastas ocultas | `Alt+H` |
| `preferences` | `search` | Buscar nas configurações | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Adicionar um item à lista | `Insert` |
| `preferences` | `list_duplicate` | Duplicar o item selecionado da lista | `Ctrl+D` |
| `preferences` | `list_remove` | Remover o item selecionado da lista | `Delete` |
| `preferences` | `list_move_up` | Mover o item selecionado da lista para cima | `Alt+Up` |
| `preferences` | `list_move_down` | Mover o item selecionado da lista para baixo | `Alt+Down` |
| `preferences` | `prev_tab` | Aba anterior da página de configurações | `Alt+Left` |
| `preferences` | `next_tab` | Próxima aba da página de configurações | `Alt+Right` |
| `ai_popup` | `copy` | Copiar o resultado | `c` |
| `activity_log` | `scroll_left` | Rolar o Registro de atividade para a esquerda | `Left` |
| `activity_log` | `scroll_right` | Rolar o Registro de atividade para a direita | `Right` |
| `integrations` | `add` | Adicionar um perfil | `a` |
| `integrations` | `edit` | Editar o perfil selecionado | `e` |
| `integrations` | `open_token` | Abrir a página do token | `o` |
| `integrations` | `remove` | Remover o perfil selecionado | `d`, `Delete` |
| `prompt` | `ai_stash_message` | IA: gerar mensagem de stash | `Ctrl+G` |
<!-- keymap:end -->

### Teclas que o terminal não consegue enviar, e o protocolo de teclado do kitty

Um terminal legado não consegue enviar alguns acordes: `Ctrl+,`, `Ctrl+Shift+N`,
`Ctrl+Enter` e `Ctrl+<dígito>`. Ele não envia nada, ou envia outra tecla: por
exemplo, `Ctrl+Shift+N` chega como `Ctrl+N`, `Ctrl+I` como `Tab` e `Ctrl+M` como
`Enter`. A página de Configurações avisa sobre uma atribuição assim (“este terminal não
envia ...”).

Terminais com o **protocolo de teclado do kitty** conseguem enviar esses acordes. Exemplos:
kitty, WezTerm, Ghostty, foot, Alacritty 0.13 e posteriores, e tmux ou herdr quando
repassam o protocolo. O GitPoulpi usa o protocolo por padrão:

- Na inicialização, o GitPoulpi consulta o terminal e espera no máximo cerca de 100 ms pela
  resposta. Se o terminal não responder, as teclas legadas permanecem.
- Se o terminal responder, o GitPoulpi liga o protocolo. Ele o desliga
  de novo quando sai, falha ou trava.
- Os avisos da página de Configurações seguem o protocolo: quando ele está ligado, os
  acordes acima podem ser enviados.
- **Configurações > Teclado e mouse > Protocolo de teclado do Kitty** (`ui.kitty_keyboard`)
  definido como `off` nunca muda o modo de teclado do terminal. Use se um terminal
  ou um multiplexador se comportar mal. É preciso reiniciar.

Os consoles do Windows informam os modificadores por conta própria e não precisam do protocolo.

## Paleta de comandos (`Ctrl+P`)

A paleta lista **todos** os comandos do app, então alcança tudo sem
atalho. A lista completa está [abaixo](#todos-os-comandos-da-paleta).

| Tecla | Ação |
|---|---|
| `Ctrl+P` | Abre a paleta |
| *(digitar)* | Filtra a lista. Uma correspondência exata após o prefixo `Categoria: ` vem primeiro, depois uma correspondência em qualquer parte do rótulo, depois uma correspondência aproximada |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J`, `Home` / `End`, `PageUp` / `PageDown` | Movem o destaque (veja [Listas de popup filtráveis](#listas-de-popup-filtráveis)) |
| `Enter`, ou um clique duplo | Executa o comando destacado |
| `Esc` | Fecha |

`j` e `k` são digitados no filtro. Eles não movem o destaque.

Cada linha mostra seu atalho à direita. Um comando que não se aplica agora
(por exemplo, nenhuma ref selecionada) continua na lista, acinzentado, e o motivo
aparece sob a lista. Os rótulos começam com uma categoria, como `Navegação:`,
`Refs:`, `Git:`, `Exibir:`, `Diff:` ou `App:`. Digite uma categoria para ver o grupo
inteiro.

### Uma tecla para qualquer comando

Cada linha da paleta termina com um botão de tecla (`[k]`), e `Alt+K` faz o mesmo na linha destacada: abre **Configurações > Teclado e mouse** filtrado pelo atalho desse comando, pronto para redefinir. Um comando sem atalho próprio ganha um aqui: ele aparece em **Comandos da paleta**, sem tecla por padrão, e a tecla que você der executa o comando exatamente como escolhê-lo na paleta (um comando que não pode rodar agora diz o motivo). A linha passa a mostrar a sua tecla.

No `config.toml` é a tabela `[keybindings.palette]`, com o nome do comando (`toggle_minimap = "Ctrl+Alt+M"`). Uma tecla que conflita com outra ação aparece em vermelho, e `c` na linha lista todas as ações com as quais ela conflita.

A faixa de dica abaixo da lista mostra essa tecla (`Alt+K` por padrão). É a ação `palette.configure_shortcut`, então pode ser redefinida como qualquer outra.

### Todos os comandos da paleta

Gerada a partir do app (um teste a mantém atualizada). Os comandos de IA só aparecem
enquanto a IA está ligada; os demais ficam acinzentados, com o motivo, quando não se aplicam.
As teclas são as padrão.

<!-- palette:begin -->
| Comando | Tecla padrão |
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
| Painel esquerdo: mostrar ou ocultar seções... |  |
| Painel esquerdo: restaurar as seções padrão |  |
| Painel esquerdo: usar estas seções em todos os repositórios |  |
| Painel esquerdo: mostrar ou ocultar as branches locais |  |
| Painel esquerdo: mostrar ou ocultar os remotos |  |
| Painel esquerdo: mostrar ou ocultar os worktrees |  |
| Painel esquerdo: mostrar ou ocultar os stashes |  |
| Painel esquerdo: mostrar ou ocultar as tags |  |
| Painel esquerdo: mostrar ou ocultar as pull requests |  |
| Painel esquerdo: mostrar ou ocultar as issues |  |
| Painel esquerdo: mostrar ou ocultar os submódulos |  |
| Navegação: focar Status > Unstaged | `Ctrl+N <badge>` |
| Navegação: focar Status > Staged | `Ctrl+N <badge>` |
| Navegação: levar o foco ao próximo painel | `Tab` |
| Navegação: levar o foco ao painel anterior | `Shift+Tab` |
| Navegação: buscar commits... | `f` |
| Navegação: recolher/expandir o painel Refs | `Ctrl+N h` |
| Navegação: alargar o painel Refs | `Ctrl+N Right` |
| Navegação: estreitar o painel Refs | `Ctrl+N Left` |
| Navegação: alargar o painel Status | `Ctrl+N Shift+Left` |
| Navegação: estreitar o painel Status | `Ctrl+N Shift+Right` |
| Navegação: recolher/expandir o painel Status | `Ctrl+N l` |
| Navegação: aumentar a seção Refs | `Ctrl+N +` |
| Navegação: diminuir a seção Refs | `Ctrl+N -` |
| Navegação: redefinir os tamanhos das seções Refs |  |
| Navegação: redefinir o layout |  |
| Detalhes do commit: aumentar a mensagem | `Ctrl+N ]` |
| Detalhes do commit: diminuir a mensagem | `Ctrl+N [` |
| Detalhes do commit: redefinir o tamanho da mensagem |  |
| Detalhes do commit: rolar a mensagem para baixo |  |
| Detalhes do commit: rolar a mensagem para cima |  |
| Arquivos: alternar a ordem dos arquivos do commit (A-Z / Z-A) |  |
| Refs: checkout da ref selecionada | `clique duplo` |
| Refs: copiar nome da ref |  |
| Refs: copiar ref completa (refs/heads/...) |  |
| Refs: copiar SHA de destino da ref |  |
| Refs: Merge da branch selecionada na atual |  |
| Refs: Rebase da branch atual sobre a selecionada | `Ctrl+G r` |
| Refs: soltar a ref selecionada na branch em checkout... |  |
| Refs: marcar como origem do soltar | `m` |
| Refs: soltar aqui a ref marcada... | `M` |
| Refs: renomear a branch selecionada... |  |
| Refs: fast-forward da branch selecionada até a atual |  |
| Refs: fast-forward da branch atual até a selecionada |  |
| Refs: criar worktree a partir da branch ou tag selecionada... |  |
| Refs: Reflog da branch selecionada... |  |
| Refs: iniciar pull request a partir da branch selecionada... |  |
| Refs: abrir worktree em uma aba | `Enter / clique duplo` |
| Refs: atualizar submódulo (selecionado) |  |
| Refs: inicializar submódulo (selecionado) |  |
| Refs: abrir submódulo em uma aba | `Enter / clique duplo` |
| Git: atualizar todos os submódulos |  |
| Git: criar worktree no commit selecionado... |  |
| Git: criar patch a partir do commit selecionado... |  |
| Git: criar patch do commit selecionado até HEAD... |  |
| Git: criar patch das alterações da árvore de trabalho... |  |
| Reescrita: apagar o histórico antigo... |  |
| Git: aplicar patch... |  |
| Reescrita: painel do histórico | `Ctrl+G H` |
| Reescrita: substituir um autor... |  |
| Reescrita: anonimizar autores... |  |
| Reescrita: aplicar o .mailmap ao histórico... |  |
| Reescrita: Merge de identidades (.mailmap)... |  |
| Reescrita: deslocar as datas dos commits... |  |
| Reescrita: mudar o fuso horário dos commits... |  |
| Reescrita: manter commits dentro de um horário... |  |
| Reescrita: usar a data do autor como data do committer |  |
| Reescrita: localizar e substituir nas mensagens... |  |
| Reescrita: excluir linhas das mensagens... |  |
| Reescrita: adicionar ou remover trailers... |  |
| Reescrita: remover assinaturas... |  |
| Reescrita: corrigir a codificação das mensagens... |  |
| Reescrita: localizar e substituir nos arquivos... |  |
| Reescrita: remover arquivos do histórico... |  |
| Reescrita: remover arquivos grandes do histórico... |  |
| Reescrita: mover ou renomear caminhos... |  |
| Reescrita: migrar arquivos para o LFS... |  |
| Reescrita: trazer arquivos de volta do LFS... |  |
| Reescrita: converter finais de linha... |  |
| Reescrita: remover o bit de execução... |  |
| Reescrita: converter a codificação dos arquivos... |  |
| Reescrita: executar um script... |  |
| ✦ Reescrita: escrever um script com IA... |  |
| ✦ Reescrita: criar regras com IA... |  |
| Reescrita: do commit selecionado até HEAD... |  |
| Reescrita: commits selecionados... |  |
| LFS: inicializar neste repositório |  |
| LFS: rastrear padrão... |  |
| LFS: parar de rastrear padrão... |  |
| LFS: fetch de objetos |  |
| LFS: abrir menu | `Ctrl+G l` |
| LFS: Fetch de todos os objetos | `Ctrl+G F` |
| LFS: checkout de todos os arquivos | `Ctrl+G K` |
| LFS: Push de todos os objetos | `Ctrl+G J` |
| LFS: pull de objetos | `Ctrl+G L` |
| LFS: push de objetos |  |
| LFS: limpar objetos locais | `Ctrl+G V` |
| LFS: mostrar bloqueios |  |
| LFS: abrir página de instalação |  |
| LFS: alternar ponteiro bruto no diff | `r` |
| LFS: rastrear o arquivo selecionado |  |
| LFS: rastrear a extensão do arquivo selecionado |  |
| LFS: bloquear o arquivo selecionado |  |
| LFS: desbloquear o arquivo selecionado |  |
| LFS: forçar o desbloqueio do arquivo selecionado |  |
| Refs: excluir a branch ou tag selecionada... | `Ctrl+G d` |
| Refs: excluir a branch selecionada do seu remoto... |  |
| Refs: excluir a branch selecionada localmente e no remoto... |  |
| Refs: copiar link da branch selecionada no remoto |  |
| Git: copiar link do commit selecionado no remoto |  |
| Git: tag anotada no commit selecionado... |  |
| Refs: push da tag selecionada para o remoto padrão |  |
| Refs: excluir a tag selecionada do seu remoto... |  |
| Git: Fetch da tag selecionada |  |
| Refs: anotar a tag selecionada... |  |
| Refs: alternar ocultar no grafo |  |
| Refs: mostrar no grafo | `g` |
| Refs: alternar solo na ref selecionada | `o` |
| Refs: parar o solo (limpa toda ref em solo) | `O` |
| Refs: mostrar todas as refs ocultas | `H` |
| Refs: alternar ocultar na pasta/remoto selecionado |  |
| Refs: alternar solo na pasta/remoto selecionado | `o` |
| Refs: filtrar por nome... | `/` |
| Refs: limpar filtro de nome | `Esc` |
| Refs: expandir todas as pastas | `=` |
| Refs: recolher todas as pastas | `-` |
| Refs: expandir a pasta selecionada e suas subpastas | `*` |
| Refs: recolher a pasta selecionada e suas subpastas | `_` |
| Refs: próxima seção | `l` |
| Refs: seção anterior | `h` |
| Refs: expandir ou recolher a seção ativa | `Enter` |
| Refs: alternar a hierarquia de branches | `y` |
| Git: stage do arquivo selecionado | `Ctrl+G s` |
| Git: unstage do arquivo selecionado | `Ctrl+G u` |
| Git: stage de todas as alterações | `Ctrl+G a` |
| Alterações: descartar o arquivo selecionado... |  |
| Alterações: ignorar o arquivo selecionado |  |
| Alterações: ignorar a extensão do arquivo selecionado |  |
| Alterações: tratar o arquivo selecionado como binário... |  |
| Alterações: tratar o arquivo selecionado como texto... |  |
| Alterações: tratar a extensão do arquivo selecionado como binária... |  |
| Alterações: tratar a extensão do arquivo selecionado como texto... |  |
| Alterações: stash do arquivo selecionado |  |
| Alterações: histórico do arquivo selecionado |  |
| Alterações: blame do arquivo selecionado |  |
| Alterações: abrir o arquivo selecionado na ferramenta de diff externa |  |
| Alterações: abrir o arquivo selecionado no editor externo |  |
| Alterações: abrir o arquivo selecionado no programa padrão |  |
| Alterações: mostrar o arquivo selecionado na pasta |  |
| Alterações: copiar o caminho do arquivo selecionado |  |
| Alterações: criar patch do arquivo selecionado... |  |
| Alterações: editar o arquivo selecionado |  |
| Alterações: excluir o arquivo selecionado... |  |
| Alterações: abrir o menu da pasta selecionada |  |
| Alterações: criar arquivo |  |
| Alterações: descartar todas as alterações... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Fazer commit das alterações em stage agora | `Ctrl+G Enter` |
| Git: editar mensagem do commit WIP (no Log) | `c no Log` |
| Git: Fetch de todos os remotos | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward se possível) |  |
| Git: Pull (só fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: opções de pull... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push para um remoto... |  |
| Git: force push (com lease)... |  |
| Git: definir upstream... |  |
| Remoto: adicionar remoto... |  |
| Remoto: remover remoto... |  |
| Remoto: editar remoto... |  |
| Remoto: definir remoto padrão... |  |
| Remoto: copiar nome do remoto |  |
| Remoto: copiar link do remoto |  |
| Remoto: fetch do remoto... |  |
| Remoto: ações do remoto selecionado... | `.` |
| Repositório: checkout parcial... |  |
| Repositório: SSH e credenciais... |  |
| Repositório: testar conexão |  |
| Repositório: usar credenciais padrão |  |
| Flow: abrir o menu | `Ctrl+G W` |
| Flow: iniciar branch... |  |
| Flow: nova issue... |  |
| Flow: assumir branch do host... |  |
| Flow: concluir a branch atual... |  |
| Flow: sincronizar com o pai... |  |
| Flow: promover... |  |
| Flow: definir o pai... |  |
| Flow: continuar ou abortar a ação interrompida... |  |
| Flow: alternar branches agrupadas |  |
| Flow: configurações do fluxo de trabalho... |  |
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
| Rebase: pular |  |
| Rebase: abortar |  |
| git am: continuar |  |
| git am: pular este patch |  |
| git am: abortar |  |
| Repositório: remover index.lock obsoleto |  |
| Aviso: pop do auto-stash pendente |  |
| Aviso: concluir o auto-stash pendente (manter o resultado) |  |
| Aviso: mostrar o auto-stash pendente |  |
| Conflitos: mostrar conflitos previstos |  |
| Aviso: descartar o auto-stash pendente... |  |
| Aviso: recomeçar... |  |
| Git: reset da branch para o commit selecionado (soft - mantém índice e árvore de trabalho) |  |
| Git: reset da branch para o commit selecionado (mixed - mantém a árvore de trabalho) | `Ctrl+G R` |
| Git: reset da branch para o commit selecionado (hard - descarta tudo)... |  |
| Git: resolver conflito com Ours (esta branch) |  |
| Git: resolver conflito com Theirs (recebido) |  |
| Git: resolver conflito com Base (ancestral comum) |  |
| Conflitos: abrir o editor de merge | `Enter em um arquivo em conflito` |
| Merge: próximo conflito | `F7` |
| Merge: conflito anterior | `Shift+F7` |
| Merge: próximo arquivo em conflito | `Alt+PageDown` |
| Merge: arquivo em conflito anterior | `Alt+PageUp` |
| Merge: aceitar trecho de A |  |
| Merge: aceitar trecho de B |  |
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
| Merge: rolar os painéis para a esquerda | `Shift+Left` |
| Merge: rolar os painéis para a direita | `Shift+Right` |
| Merge: alternar o painel Base | `Alt+D` |
| Merge: alternar diff do arquivo todo | `Alt+F` |
| Merge: desfazer | `Ctrl+Z` |
| Merge: refazer | `Ctrl+Y` |
| Merge: salvar sem marcar como resolvido | `Alt+S` |
| Merge: salvar e marcar como resolvido | `Ctrl+S` |
| Merge: redefinir o resultado para o resultado do merge | `Alt+R` |
| Merge: alternar a altura do resultado | `z` |
| Merge: alargar o painel Atual | `Alt+Right` |
| Merge: alargar o painel Recebido | `Alt+Left` |
| Merge: redefinir os tamanhos dos painéis | `Alt+0` |
| Merge: desfazer a escolha deste conflito | `Alt+U` |
| Merge: ir ao painel Atual (A) | `Alt+1` |
| Merge: ir ao painel Recebido (B) | `Alt+2` |
| Merge: ir ao painel Resultado | `Alt+3` |
| Merge: ir ao painel Base | `Alt+4` |
| Merge: painel da esquerda (mesmo conflito) |  |
| Merge: painel da direita (mesmo conflito) |  |
| Merge: focar o próximo painel | `F6` |
| Merge: focar o painel anterior | `Shift+F6` |
| Merge: continuar a partir do arquivo em disco |  |
| Merge: recomeçar a partir do merge |  |
| Merge: desfazer resolução e reabrir |  |
| Merge: recarregar o arquivo |  |
| Merge: usar o arquivo Atual como está | `a` |
| Merge: usar o arquivo Recebido como está | `b` |
| Merge: usar o arquivo Base como está | `o` |
| Merge: manter o arquivo que sobrou | `k` |
| Merge: aceitar a exclusão | `D` |
| Merge: abrir na ferramenta de merge externa | `m` |
| ✦ IA: sugerir resolução de conflito de merge | `Alt+I` |
| Merge: fechar o editor de merge |  |
| Git: abortar merge, cherry-pick ou revert |  |
| Git: pular cherry-pick ou revert |  |
| Repositório: abrir ou trocar (gerenciamento de repositórios)... | `Ctrl+O` |
| Repositório: procurar um repositório... | `b (aba de gerenciamento)` |
| Repositório: criar novo (Init)... | `i (aba de gerenciamento)` |
| Repositórios: abrir o selecionado | `Enter` |
| Repositórios: adicionar/remover o selecionado dos favoritos | `f` |
| Repositórios: fechar ou remover o selecionado | `x` |
| Repositórios: menu do repositório selecionado... | `.` |
| Repositórios: buscar | `/ / Ctrl+F` |
| Repositórios: recolher todas as seções | `-` |
| Repositórios: expandir todas as seções | `+` |
| Repositórios: fechar todas as abas... |  |
| Repositórios: remover o selecionado da lista |  |
| Repositórios: remover todos os recentes |  |
| Espaço de trabalho: novo... | `n` |
| Espaço de trabalho: novo a partir das abas abertas... | `a` |
| Espaço de trabalho: trocar... | `Alt+W` |
| Espaço de trabalho: editar... | `e` |
| Espaço de trabalho: fechar (sem espaço de trabalho) |  |
| Espaço de trabalho: gerenciar... | `Alt+J` |
| Espaço de trabalho: fechar aba |  |
| Espaço de trabalho: localizar o repositório ausente... | `l` |
| Espaço de trabalho: excluir... |  |
| Espaço de trabalho: fetch tudo | `F` |
| Espaço de trabalho: pull tudo | `P` |
| Espaço de trabalho: clonar os ausentes | `m` |
| Espaço de trabalho: adicionar a aba ao espaço de trabalho |  |
| Espaço de trabalho: remover a aba do espaço de trabalho |  |
| Radar: abrir | `Alt+R` |
| Radar: atualizar | `r` |
| Radar: buscar | `/` |
| Radar: abrir no navegador | `o` |
| Radar: abrir repositório | `Enter` |
| Radar: fazer checkout da branch | `c` |
| Radar: criar worktree... | `t` |
| Radar: mostrar no repositório | `s` |
| Radar: começar a trabalhar numa issue... | `n` |
| Radar: fixar ou desafixar | `p` |
| Radar: adiar... | `z` |
| Radar: revisar solicitação... | `e` |
| Radar: revisar pela API |  |
| Radar: clonar e revisar... |  |
| Radar: editar responsáveis... |  |
| Radar: editar revisores... |  |
| Radar: editar rótulos... |  |
| Radar: definir marco... |  |
| Radar: escolher escopo | `w` |
| Radar: escolher visão | `v` |
| Radar: escolher filtro | `f` |
| Radar: exibir solicitações não clonadas | `a` |
| Radar: exibir rascunhos | `d` |
| Radar: ativar as solicitações de revisão da conta... | `T` |
| Radar: menu da linha... | `.` |
| Radar: escolher coluna | `C` |
| Radar: alargar coluna | `]` |
| Radar: estreitar coluna | `[` |
| Radar: redefinir larguras das colunas | `0` |
| Radar: voltar ao início | `Esc` |
| Radar: fechar aba |  |
| Aba: nova aba | `Alt+T` |
| Aba: nova... |  |
| Aba: menu de ações... | `Ctrl+T` |
| Aba: fechar outras abas... |  |
| Aba: fechar abas à direita... |  |
| Aba: adicionar aos/remover dos favoritos |  |
| Aba: copiar caminho do repositório |  |
| Aba: abrir no gerenciador de arquivos |  |
| Aba: abrir no terminal |  |
| Aba: abrir em... | `Alt+O` |
| Aba: abrir no padrão | `Ctrl+Alt+O` |
| Repositório: alterar perfil... |  |
| Ajuda: sobre o GitPoulpi | `Ctrl+N a` |
| Ajuda: licenças e créditos |  |
| Exibir: registro de atividade | `Ctrl+N A` |
| Exibir: registro de atividade global | `Ctrl+N G` |
| Ajuda: Notas da versão |  |
| Atualizações: mostrar painel de atualização | `Ctrl+N u` |
| Atualizações: abrir aviso de segurança |  |
| Atualizações: copiar comando de atualização |  |
| Atualizações: executar comando de atualização em um terminal |  |
| Atualizações: verificar agora |  |
| Atualizações: baixar |  |
| Atualizações: atualizar agora |  |
| Atualizações: reiniciar para atualizar |  |
| Atualizações: voltar à versão anterior |  |
| Atualizações: ignorar esta versão |  |
| Notas da versão: fechar aba |  |
| Tarefas: mostrar as tarefas em segundo plano | `Ctrl+N J` |
| Tarefas: cancelar tudo o que for possível |  |
| Exibir: copiar diagnóstico |  |
| Registro de atividade: mostrar ou ocultar o nível Erro |  |
| Registro de atividade: mostrar ou ocultar o nível Aviso |  |
| Registro de atividade: mostrar ou ocultar o nível Info |  |
| Registro de atividade: mostrar ou ocultar o nível Depuração |  |
| Registro de atividade: mostrar ou ocultar o nível Rastreamento |  |
| Registro de atividade: mostrar todos os níveis |  |
| Registro de atividade: não mostrar nenhum nível |  |
| Registro de atividade: limpar o registro do aplicativo |  |
| Registro de atividade: escolher o escopo |  |
| Registro de atividade: operações do Git |  |
| Registro de atividade: registro do aplicativo |  |
| Registro de atividade: rolar para a esquerda | `Left` |
| Registro de atividade: rolar para a direita | `Right` |
| Registro de atividade: copiar o último erro |  |
| Registro de atividade: executar a ação sugerida |  |
| Registro de atividade: abrir o último erro |  |
| Aba: sair e cd aqui |  |
| Repositório: clonar... | `Ctrl+Shift+N` |
| Repositório: próxima aba | `Ctrl+PageDown` |
| Repositório: aba anterior | `Ctrl+PageUp` |
| Aba: mover para a esquerda |  |
| Aba: mover para a direita |  |
| Repositório: fechar aba | `Ctrl+W` |
| Repositório: selecionar aba... | `t` |
| Buscar: espaço de trabalho... | `Ctrl+Alt+F` |
| Editar: desfazer | `Ctrl+Z` |
| Editar: refazer | `Ctrl+Y` |
| Editar: esquecer a entrada de desfazer bloqueada |  |
| Editar: histórico de desfazer... | `Alt+Z` |
| Editar: Reflog... |  |
| Editar: recuperar commits perdidos... |  |
| Editar: restaurar a branch para onde a ação bloqueada a deixou | `Alt+Shift+Z` |
| Revisão: mostrar visão geral |  |
| Revisão: mostrar arquivos |  |
| Revisão: mostrar commits |  |
| Revisão: comentar na linha... | `c` |
| Revisão: comentar na conversa... | `C` |
| Revisão: responder à thread... | `r` |
| Revisão: resolver ou reabrir a thread | `x` |
| Revisão: recolher ou expandir a thread | `z` |
| Revisão: editar comentário pendente... | `e` |
| Revisão: excluir comentário pendente | `d` |
| Revisão: marcar arquivo como visualizado | `v` |
| Revisão: próximo arquivo | `]` |
| Revisão: arquivo anterior | `[` |
| Revisão: concluir revisão... | `f` |
| Revisão: descartar revisão pendente... |  |
| Revisão: atualizar | `F5` |
| Revisão: abrir no navegador | `o` |
| Revisão: fazer checkout da branch | `b` |
| Revisão: fechar | `Esc` |
| Revisão: editar responsáveis... | `a` |
| Revisão: editar revisores... | `p` |
| Revisão: editar rótulos... | `l` |
| Revisão: definir marco... | `m` |
| Revisão: escolher patch set... | `P` |
| Revisão: comparar patch sets... | `D` |
| Revisão: submeter a solicitação... | `U` |
| Revisão: abandonar a solicitação... | `Q` |
| Revisão: alternar layout inline / dividido | `V` |
| Revisão: alternar arquivo todo / só hunks | `H` |
| Revisão: alternar alterações só de espaços em branco | `S` |
| Revisão: alternar quebra de linha | `W` |
| Revisão: próximo hunk | `J` |
| Revisão: hunk anterior | `K` |
| Revisão: escolher a codificação... | `E` |
| Revisão: alternar minimapa | `M` |
| Revisão: rolar o diff para a esquerda | `Left` |
| Revisão: rolar o diff para a direita | `Right` |
| Revisão: copiar o texto selecionado ou a linha | `Ctrl+C` |
| Quadro: mover cartão para... | `m` |
| Quadro: mover cartão para a esquerda |  |
| Quadro: mover cartão para a direita |  |
| Quadro: abrir cartão | `Enter` |
| Quadro: editar responsáveis... |  |
| Quadro: editar rótulos... |  |
| Quadro: definir marco... |  |
| Quadro: nova issue nesta coluna | `n` |
| Quadro: filtrar por responsável... | `a` |
| Quadro: filtrar por rótulo... | `l` |
| Quadro: filtrar por marco... | `M` |
| Quadro: filtrar por texto | `/` |
| Quadro: limpar filtros | `x` |
| Quadro: definir rótulos das colunas... | `c` |
| Quadro: atualizar | `r` |
| Quadro: fechar | `Esc` |
| Revisão: ✦ gerar rascunho de revisão... | `s` |
| Revisão: aceitar sugestão | `y` |
| Revisão: editar e aceitar sugestão... | `w` |
| Revisão: dispensar sugestão | `n` |
| Revisão: aceitar todas as sugestões | `A` |
| Revisão: dispensar todas as sugestões | `N` |
| Revisão: usar o resumo da IA no texto da revisão | `i` |
| Revisão: ✦ usar o rascunho... | `u` |
| Revisão: editar o texto do ✦ rascunho... | `t` |
| Revisão: descartar o ✦ rascunho | `N` |
| Histórico de desfazer: desfazer até aqui... | `u` |
| Histórico de desfazer: refazer até aqui... | `y` |
| Histórico de desfazer: restaurar só esta... | `r` |
| Histórico de desfazer: restaurar a branch para onde estava... | `o` |
| Histórico de desfazer: atualizar |  |
| Histórico de desfazer: fechar | `Esc` |
| Reflog: mostrar no Log | `s` |
| Reflog: criar branch aqui... | `b` |
| Reflog: Create Tag Here... | `t` |
| Reflog: Apply Changes to Working Tree | `w` |
| Reflog: Save Changes as Stash | `z` |
| Reflog: restaurar para esta posição... | `r` |
| Reflog: carregar mais | `m` |
| Reflog: atualizar |  |
| Trabalho perdido: criar branch aqui... | `b` |
| Lost Work: Create Tag Here... | `t` |
| Trabalho perdido: cherry-pick | `c` |
| Lost Work: Apply Changes to Working Tree | `w` |
| Lost Work: Save Changes as Stash | `z` |
| Trabalho perdido: buscar de novo |  |
| Exibir: atualizar | `r` |
| Exibir: atualizar (fetch, pull requests, issues) | `F5` |
| Exibir: alternar a ajuda | `?` |
| Exibir: alternar a coluna Ref |  |
| Exibir: alternar a coluna Grafo |  |
| Exibir: alternar a coluna Mensagem |  |
| Exibir: alternar a coluna Autor |  |
| Exibir: alternar a coluna Data |  |
| Exibir: alternar a coluna Alterações |  |
| Exibir: alternar a coluna SHA |  |
| Exibir: coluna do Log: escolher uma coluna para mover ou redimensionar | `C` |
| Exibir: coluna do Log: escolher a próxima coluna | `Tab` |
| Exibir: coluna do Log: escolher a coluna anterior | `Shift+BackTab` |
| Exibir: coluna do Log: mover a coluna para a esquerda | `<` |
| Exibir: coluna do Log: mover a coluna para a direita | `>` |
| Exibir: coluna do Log: alargar a coluna | `]` |
| Exibir: coluna do Log: estreitar a coluna | `[` |
| Exibir: coluna do Log: redefinir as larguras das colunas | `0` |
| Exibir: coluna do Log: ajustar a coluna ao conteúdo | `=` |
| Exibir: coluna do Log: ajustar todas as colunas ao conteúdo | `+` |
| Exibir: Log: rolar as colunas para a esquerda | `Shift+Left` |
| Exibir: Log: rolar as colunas para a direita | `Shift+Right` |
| Exibir: Log: alternar a rolagem para os lados das colunas | `s` |
| Exibir: Log: escolher as colunas a mostrar |  |
| Diff: abrir o diff do arquivo selecionado | `Enter` |
| Diff: fechar a visualização de diff | `Esc` |
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
| Diff: layout inline (+/- em uma coluna) | `m` |
| Diff: layout dividido (antes e depois) | `m` |
| Diff: alternar só hunks (arquivo todo / só as alterações) | `H` |
| Arquivos: alternar caminho/árvore | `m` |
| Arquivos: alternar ver todos os arquivos | `a` |
| Commit: alternar Ignorar hooks | `H` |
| Arquivos: abrir o menu da linha ou da seleção | `.` |
| Arquivos: expandir todas as pastas | `=` |
| Arquivos: recolher todas as pastas | `-` |
| Arquivos: expandir a pasta selecionada e suas subpastas | `*` |
| Arquivos: recolher a pasta selecionada e suas subpastas | `_` |
| Arquivos: selecionar tudo |  |
| Arquivos: limpar a seleção |  |
| Arquivos: alternar a seleção da linha atual |  |
| Log: copiar como gitGraph do Mermaid... | `Y` |
| Log: comparar a seleção | `v` |
| Log: trocar as extremidades da comparação | `x` |
| Log: ir para a próxima linha comparada | `n` |
| Diff: alternar quebra de linha | `w` |
| Diff: escolher a codificação... | `C` |
| Exibir: alternar o minimapa | `M` |
| Exibir: alternar a coluna de commits | `Alt+C` |
| Exibir: coluna de commits: ocultar |  |
| Exibir: coluna de commits: histórico do arquivo |  |
| Exibir: coluna de commits: todos os commits |  |
| Exibir: focar a coluna de commits |  |
| Exibir: alargar a coluna de commits |  |
| Exibir: estreitar a coluna de commits |  |
| Exibir: coluna de commits: rolar para a esquerda | `Shift+Left` |
| Exibir: coluna de commits: rolar para a direita | `Shift+Right` |
| Exibir: coluna de commits: dimensionar a próxima parte | `Tab` |
| Exibir: coluna de commits: mover a parte para a esquerda | `<` |
| Exibir: coluna de commits: mover a parte para a direita | `>` |
| Exibir: coluna de commits: alargar a parte | `]` |
| Exibir: coluna de commits: estreitar a parte | `[` |
| Exibir: coluna de commits: redefinir as larguras das partes | `0` |
| Exibir: coluna de commits: menu da linha selecionada | `.` |
| Exibir: coluna de commits: escolher as colunas |  |
| Exibir: coluna de commits: alternar Ref |  |
| Exibir: coluna de commits: alternar Grafo |  |
| Exibir: coluna de commits: alternar Autor |  |
| Exibir: coluna de commits: alternar Data |  |
| Exibir: coluna de commits: alternar Alterações |  |
| Exibir: coluna de commits: alternar SHA |  |
| Exibir: coluna de commits: mostrar a linha selecionada no Log |  |
| Exibir: coluna de commits: mostrar as alterações do arquivo na linha selecionada |  |
| Exibir: coluna de commits: mostrar o arquivo como era na linha selecionada |  |
| Exibir: coluna de commits: fazer blame do arquivo na linha selecionada |  |
| Editor: alternar seguir a seleção |  |
| Imagem: aumentar zoom | `+` |
| Imagem: diminuir zoom | `-` |
| Imagem: ajustar à visualização | `0` |
| Imagem: tamanho real (1:1) | `1` |
| Imagem: alternar antes / depois | `Tab` |
| Imagem: alargar painel Antes | `Alt+Right` |
| Imagem: alargar painel Depois | `Alt+Left` |
| Imagem: redefinir tamanhos dos painéis | `Alt+0` |
| Imagem: editar em... | `O` |
| Imagem: editar no editor padrão | `o` |
| Histórico do arquivo: mostrar no Log | `Enter` |
| Histórico do arquivo: copiar o SHA do commit | `y` |
| Histórico do arquivo: copiar o resumo | `Y` |
| Histórico do arquivo: mostrar o diff deste commit | `d` |
| Histórico do arquivo: abrir o arquivo neste commit | `o` |
| Histórico do arquivo: blame neste commit | `b` |
| Histórico do arquivo: comparar com a árvore de trabalho | `c` |
| Histórico do arquivo: restaurar esta versão | `R` |
| Histórico do arquivo: estender a seleção para cima | `Shift+Up` |
| Histórico do arquivo: estender a seleção para baixo | `Shift+Down` |
| Histórico do arquivo: alternar entre a lista e o diff | `Tab` |
| Histórico do arquivo: aumentar a largura da lista | `Alt+Right` |
| Histórico do arquivo: diminuir a largura da lista | `Alt+Left` |
| Histórico do arquivo: redefinir a largura da lista | `Alt+0` |
| Arquivo: abrir com... | `O` |
| Arquivo: abrir no aplicativo padrão | `o` |
| Diff: alternar alterações só de espaços em branco | `s` |
| ✦ IA: gerar mensagem de commit | `Ctrl+G i` |
| ✦ IA: gerar mensagem de stash |  |
| ✦ IA: inserir variável do prompt |  |
| ✦ IA: editar perfil... |  |
| ✦ IA: abrir o agente... |  |
| ✦ IA: explicar o commit selecionado | `Ctrl+G w` |
| ✦ IA: dividir o commit selecionado... |  |
| ✦ IA: explicar as alterações em andamento | `Ctrl+G w` |
| ✦ IA: compor commits a partir das alterações em andamento... | `Ctrl+G C` |
| Compositor: mover arquivo para um commit... | `m` |
| Diff: blame |  |
| Blame: mostrar o commit no Log | `Enter` |
| Blame: copiar o SHA do commit | `Y` |
| Blame: diff do commit para este arquivo | `d` |
| Blame: blame no pai do commit | `b` |
| Blame: abrir o histórico do arquivo | `H` |
| Blame: mostrar ou ocultar o cartão de detalhes | `i` |
| Blame: próximo hunk | `n` |
| Blame: hunk anterior | `p` |
| Blame: próximo hunk do mesmo commit | `N` |
| Blame: hunk anterior do mesmo commit | `P` |
| Blame: ir para a linha | `g` |
| Blame: aumentar coluna de anotações | `>` |
| Blame: diminuir coluna de anotações | `<` |
| Diff: histórico do arquivo |  |
| Diff: visualização do arquivo |  |
| Arquivo: editar no diretório de trabalho | `e` |
| Editor: alterar codificação |  |
| Editor: reabrir com codificação |  |
| Editor: converter para codificação |  |
| Editor: converter quebras de linha |  |
| Editor: alterar indentação |  |
| App: configurações... | `Ctrl+, / Ctrl+N ,` |
| App: buscar configurações... | `/ nas Configurações` |
| App: atalhos de teclado... |  |
| Agente: mostrar a visualização pendente | `Ctrl+N Enter` |
| Agente: dispensar a visualização pendente |  |
| Agente: voltar | `Ctrl+N Backspace` |
| Atalhos: redefinir tudo... |  |
| Integrações: abrir pull request vinculado | `Ctrl+G g` |
| Integrações: editar pull request vinculado no host | `e (em uma linha)` |
| Integrações: copiar URL do pull request vinculado |  |
| Integrações: atualizar pull requests e issues |  |
| Integrações: checkout de pull request... |  |
| Integrações: abrir referência do commit (#issue / !solicitação)... |  |
| Integrações: abrir issue... |  |
| Hospedagem: revisar solicitação... |  |
| Hospedagem: push para revisão... |  |
| Hospedagem: editar responsáveis... |  |
| Hospedagem: editar revisores... |  |
| Hospedagem: editar rótulos... |  |
| Hospedagem: definir marco... |  |
| Issues locais: fechar issue... |  |
| Issues locais: reabrir issue... |  |
| Issues locais: abrir no Obsidian... |  |
| Issues: abrir quadro |  |
| Issues locais: criar Base do Obsidian |  |
| Pull requests: criar pull request... | `n` |
| Hospedagem: nova issue... | `n` |
| Pull requests: buscar | `Enter na linha de busca` |
| Pull requests: escolher filtro (abertos / merged / todos)... | `v` |
| Pull requests: escolher provedor... | `Enter na linha do provedor` |
| Pull requests: escolher repositório... | `Enter na linha do repositório` |
| Pull requests: desvincular origem (nenhuma) |  |
| Issues: escolher provedor... | `Enter na linha do provedor` |
| Issues: desvincular origem (nenhuma) |  |
| Integrações: desvincular perfil (automático) |  |
| Issues: escolher repositório... | `Enter na linha do repositório` |
| Issues: buscar | `Enter na linha de busca` |
| Issues: escolher filtro... | `v` |
| Issues: editar consulta / listas... |  |
| Commit: Inserir referência de issue... |  |
| Integrações: usar perfil neste repositório... |  |
| Integrações: gerenciar o perfil deste repositório |  |
| Integrações: conectar integração... |  |
| Configurações: Opções do Radar de um perfil... |  |
| Integrações: revisar verificação de segurança |  |
| App: sair | `q` |
| App: tema... |  |
| Conflitos: editar listas de branches base... |  |
| Conflitos: branches base deste repositório... |  |
| App: mudar tema... | `Ctrl+N y` |
| App: mudar idioma... | `Ctrl+N g` |
| Lançador: criar atalho |  |
| Lançador: criar atalho da área de trabalho |  |
| Lançador: remover atalhos |  |
| Lançador: criar alias de comando |  |
| Lançador: remover alias de comando |  |
| Lançador: adicionar ao PATH |  |
| Lançador: remover do PATH |  |
| App: mudar tema de cores do grafo... | `Ctrl+N Y` |
| App: alternar tom das linhas do grafo | `Ctrl+N T` |
<!-- palette:end -->

## Movimentação

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab` | Foca o painel seguinte / anterior |
| `Ctrl+N`, depois um selo | Foca essa parte da janela (veja [Prefixo de navegação](#prefixo-de-navegação-ctrln)) |
| `j` / `k`, `Up` / `Down` | Movem a seleção, em todos os painéis |
| `PageUp` / `PageDown`, `Home` / `End` | Movem uma página; vão para a primeira / última linha |
| um número, depois `j` / `k` / `Up` / `Down` | Prefixo de contagem: `4j` move quatro linhas. Com `PageUp` / `PageDown`, a contagem é um número de páginas. A linha de status mostra a contagem pendente (`4…`). `Esc` ou qualquer outra tecla a limpa. Um `0` inicial nunca começa uma contagem |
| `h` / `l`, `Left` / `Right` | Painel Refs: troca de seção. Log: sai (`h`) ou entra (`l`) na lista de arquivos alterados de um commit |
| `?` | Abre ou fecha a ajuda (veja [Ajuda integrada](#ajuda-integrada-)) |
| `r` | Atualizar |
| `F5` | Faz fetch de todos os remotos, recarrega os pull requests e as issues, e atualiza |
| `q` | Sair |
| `Ctrl+Z` | **Desfaz** a última operação. Veja [Desfazer e refazer](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Refaz**. `Ctrl+Shift+Z` precisa do protocolo de teclado do kitty; `Ctrl+Y` sempre funciona |
| `.`, `Menu`, `Shift+F10` | Abrem o menu de contexto da linha selecionada |

`Ctrl+Z` e `Ctrl+Y` não fazem nada enquanto um campo de texto está com o teclado. O
editor tem seu próprio desfazer de texto.

## Listas de popup filtráveis

Todas as listas de popup filtráveis usam as mesmas teclas: a paleta de comandos e seus
seletores (tema, idioma, cores do grafo), o seletor de abas e todo menu suspenso
com uma linha de busca (opções das Configurações, o formulário de pull request, o seletor de
referência de issue, a lista **Abrir em**). Digite para filtrar a lista. Estas teclas
movem o destaque:

| Tecla | Ação |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Movem uma linha |
| `Home` / `End` | Primeira / última linha |
| `PageUp` / `PageDown` | Movem uma página visível |
| `Enter` | Escolhe a linha destacada |
| `Esc` | Fecha |

Nestes popups, `Home`, `End` e `Ctrl+K` movem a lista, não o cursor do
campo de filtro. `Ctrl+A` e `Ctrl+E` ainda movem o cursor para o início e o fim.
`j` e `k` simples são digitados no filtro.

Com o mouse:

- A roda **rola** a lista. Ela não move o destaque.
- Um clique **seleciona** uma linha. Um clique duplo a **escolhe**, como `Enter`.
- Uma lista mais alta que o popup mostra uma barra de rolagem na borda direita. Você pode
  arrastá-la.
- Um menu suspenso curto sem linha de busca (duas ou três opções) escolhe no
  primeiro clique.

**Pré-visualização ao vivo.** Os seletores de tema, tema de cores do grafo e idioma aplicam a
linha destacada na hora, atrás do popup. `Enter` a mantém. `Esc`, ou um clique
fora do popup, restaura o valor anterior.

**Filtro de temas.** O seletor de temas tem uma linha sob o campo de filtro: **Todos**,
**Escuros**, **Claros** e **Coloridos**. `Tab` / `Shift+Tab` vão para o seguinte /
anterior. `Alt+1` a `Alt+4` escolhem um diretamente. Você também pode clicar neles.

## Ajuda integrada (`?`)

`?`, `Ctrl+N ?` ou o `Exibir: alternar a ajuda` da paleta abre a ajuda.
Ela lista as teclas e os gestos do mouse de todas as partes do app. Abre sobre
qualquer coisa, inclusive a tela de Configurações.

| Tecla | Ação |
|---|---|
| `?` | Fecha a ajuda (e limpa o filtro) |
| *(digitar)* | Filtro: mantém só as seções que contêm o texto. `j`, `k` e `q` também são digitados |
| `Backspace`, `Ctrl+U` | Apagam o último caractere / limpam o filtro |
| `Esc` | Limpa o filtro; se ele estiver vazio, fecha a ajuda |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Rolam uma linha |
| `PageUp` / `PageDown`, `Home` / `End` | Rolam uma página; vão para o topo / o fim |
| `Ctrl+D`, ou um clique no link | Abre esta documentação no GitHub no navegador |

A roda rola a ajuda, e você pode arrastar a barra de rolagem dela. Um clique fora
da ajuda a fecha.

## Prefixo de navegação (`Ctrl+N`)

`Ctrl+N` espera mais uma tecla. A linha de status mostra as teclas que ele aceita. A
tecla seguinte executa uma ação, ou cancela.

| Tecla | Ação |
|---|---|
| `Ctrl+N` e depois um selo | Foca a parte da janela que mostra esse selo |
| `Ctrl+N h` / `Ctrl+N l` | Recolhe ou restaura o painel Refs / o painel da direita |
| `Ctrl+N Left` / `Ctrl+N Right` | Move o divisor entre o painel Refs e o Log um passo para a esquerda / direita (um painel Refs recolhido volta antes) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Move o divisor entre o Log e o painel da direita um passo para a esquerda / direita |
| `Ctrl+N +` / `Ctrl+N -` | Aumenta / diminui a seção Refs ativa em uma linha |
| `Ctrl+N ]` / `Ctrl+N [` | Aumenta / diminui o bloco da mensagem de commit do painel da direita em uma linha |
| `Ctrl+N t` | Escolhe uma aba (o seletor de abas; `t` sozinho faz o mesmo) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | Mudam o tema / o idioma / o tema de cores do grafo |
| `Ctrl+N Shift+T` | Liga ou desliga os tons de linha do grafo |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Mostram a visualização que um agente preparou / voltam (veja [MCP e agentes](mcp-and-agents.md)) |
| `Ctrl+N ,` | Abre as Configurações |
| `Ctrl+N ?` | Abre ou fecha a ajuda |
| `Esc`, ou qualquer outra tecla | Cancela |

### Selos

Enquanto o prefixo espera, cada parte da janela que pode receber o foco mostra
um selo de um caractere na borda superior esquerda. Pressione esse caractere para focá-la.
Os selos são redistribuídos a cada vez, na ordem da tela: `1` a `9` primeiro,
depois `a`, `b`, `c`... Então leia o selo na tela: uma parte nem sempre
recebe o mesmo selo.

Partes com selo: o filtro Refs e cada seção Refs visível, o Log (ou
a visualização Diff), a visualização de arquivo quando aberta, as listas Unstaged e Staged,
o campo de resumo do compositor de commits e os controles da lista de arquivos do commit.
Na aba Início, os selos são o campo de busca, as quatro seções e os botões da barra de ferramentas (o selo de um botão o aciona). Na aba Radar,
são o campo de filtro, as listas suspensas Escopo, Visão e Mostrar (o selo de uma lista
suspensa a abre), os botões Atualizar e Voltar e cada título de seção na tela. O selo
de uma seção move a seleção para o título dela; depois `Enter` ou `Space` a recolhe.

Na janela Procurar pasta e nos diálogos Clonar, Init, Criar worktree, Espaço de trabalho, Checkout esparso, Flow, Nova issue e Pull request, os selos são os campos, as caixas de seleção e os botões (o selo de um botão o aciona, o de um campo põe o cursor nele; em uma lista, um selo fica na primeira linha; a janela Procurar pasta tem um selo para a lista Acesso rápido e outro para a lista Pastas, ao lado da primeira linha de cada lista, e pressionar um põe o teclado nessa lista sem escolher uma linha). No Registro de atividade, são as duas abas, as caixas Escopo e Nível, o filtro, as caixas de seleção dos níveis, os botões e a primeira entrada da lista. Um selo faz o que um clique nesse controle faz. No seletor de pastas, `Ctrl+N` inicia os selos, então a tecla de Nova pasta é `Alt+N`.

No diálogo Alterar (e Editar mensagem do commit), os selos são o resumo, a descrição, o nome, o e-mail e a data do autor, **Modificar o histórico mais a fundo...**, **Cancelar**, **Aplicar** e a cruz de fechar; no painel de reescrita do histórico são a caixa de escopo, suas caixas de seleção e o botão de refs, o nome da receita, cada regra, sua caixa de seleção e seus botões, o botão **+ Adicionar regra**, os campos da regra selecionada, as abas da prévia, a caixa de filtro, as linhas das branches protegidas e seus Marcar tudo / Desmarcar tudo, e os botões Prévia, Aplicar, Cancelar, Salvar, Carregar e Criar regras com IA (as páginas Segredos, Identidades, Tamanho e resultado acrescentam seus próprios botões e linhas; as linhas da aba Commits, por serem uma lista, não têm). Um selo faz o que um clique faz. Nada recebe selo enquanto há uma lista suspensa, uma pergunta, um campo em edição, o seletor de pastas, a tela de purga, o editor de script, os diálogos de IA, o diff de blob ou o roteiro do host aberto, nem enquanto o diálogo Alterar pergunta "descartar?"; `Esc` tira os selos sem fechar nada.

Nas Configurações, os selos ficam na caixa de busca, na lista de páginas (um só selo, na página atual), nas subabas, na lista e seus botões, em cada campo da página que está na tela e no ✕ de fechar. Um selo faz o que um clique nesse controle faz: um campo de texto abre para edição, uma caixa de seleção é marcada, uma lista suspensa abre, um botão é acionado. `Ctrl+N` mantém seu significado próprio enquanto se digita em um campo ou há uma lista suspensa, um editor ou a busca aberta.

A paleta tem as mesmas ações, por exemplo
`Navegação: focar Refs > Local` ou `Navegação: focar Status > Staged`.

## Repositórios e abas

| Tecla | Ação |
|---|---|
| `Ctrl+O`, ou um clique em `⌂` | Vai para a aba Início (na aba Início: foca o campo de busca; com o Radar ou as notas da versão abertos: fecha-os primeiro) |
| `Ctrl+Shift+N` | Abre a aba Início com o diálogo Clonar |
| `Ctrl+PageDown` / `Ctrl+PageUp` | Aba seguinte / anterior |
| `t` | Escolhe uma aba em uma lista filtrável |
| `Ctrl+W` | Fecha a aba atual (pergunta antes; o repositório não é alterado) |
| `Ctrl+T`, ou um clique com o botão direito em uma aba | O menu da aba (as linhas dele estão listadas em [Linha de abas](interface-tour.md#linha-de-abas)) |
| `Alt+O` / `Ctrl+Alt+O` | Abre a pasta da aba: escolhe na lista **Abrir em** / usa a entrada padrão. Veja [Abrir em](open-in.md) |

### A aba Início

A aba `⌂` gerencia seus repositórios. Ela tem quatro seções: **Repositórios
abertos**, **Favoritos**, **Repositórios recentes** e **Todos os
repositórios**. Veja [Tour da interface](interface-tour.md#início).

| Tecla | Ação |
|---|---|
| `j` / `k`, setas, `PageUp` / `PageDown`, `Home` / `End` | Movem |
| `Enter`, ou um clique duplo | Abre o repositório (ou muda para a aba dele). Em um cabeçalho de seção: recolhe ou expande |
| `f` ou `*`, ou um clique na estrela | Adiciona aos / remove dos favoritos |
| `x`, `Delete`, ou o botão da linha | Fecha a aba (Abertos), remove da lista (Recentes), remove a estrela (Favoritos), esquece (Todos). Nada no disco é excluído |
| `.`, um clique com o botão direito, ou o `⋮` da linha | O menu da linha |
| `/` ou `Ctrl+F` | Busca em todas as seções por nome, branch ou caminho. `Enter` abre a primeira correspondência; `Esc` limpa a busca e depois sai do campo |
| `Space`, `Left` / `Right` | Recolhem / expandem a seção atual |
| `-` / `+` | Recolher tudo / Expandir tudo |
| `b` / `c` / `i` | Procurar (abrir uma pasta) / Clonar / Init |
| `Esc` | Limpa a busca, ou volta à aba de repositório ativa |
| `q` | Sair |

`Ctrl+P`, `Ctrl+,`, `?` e `Ctrl+N` funcionam na aba Início. `Ctrl+G`, `Ctrl+W`
e `Ctrl+T` precisam de uma aba de repositório: na aba Início eles só mostram uma mensagem.
Os comandos da paleta que precisam de um repositório ficam acinzentados.

### O seletor de pastas

**Procurar** (e o botão **Procurar...** dos diálogos Init, Clonar e Criar
worktree) abre o seletor de pastas. Ele tem um campo de caminho no topo, os
locais e a árvore de pastas à esquerda, e o conteúdo da pasta atual
à direita.

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab` | Campo de caminho, depois a coluna da esquerda, depois o conteúdo |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Movem na coluna focada. No campo de caminho: percorrem as sugestões de preenchimento |
| uma letra (conteúdo) | Vai para a próxima pasta que começa com essa letra |
| `Enter` / `Right` | Abre a pasta selecionada. No campo de caminho: vai para a pasta digitada; se for a pasta já exibida, a escolhe |
| `Backspace` / `Left` / `Alt+Up` | Pasta pai (no campo de caminho, `Backspace` apaga um caractere) |
| `Tab` (campo de caminho) | Aceita a única sugestão, ou a destacada |
| `Alt+N` | Nova pasta (`Ctrl+N` inicia os selos, veja [Selos](#selos)) |
| `Alt+H` | Mostra ou oculta pastas ocultas |
| `Ctrl+F`, ou `/` fora do campo de caminho | Filtra as entradas da pasta enquanto você digita (sem diferenciar maiúsculas; `..` fica; o cabeçalho conta "x de n"; `Enter` pula para a primeira correspondência, `Esc` limpa; ele se esvazia quando você troca de pasta). Também acessível com `Tab` ou um clique |
| `Ctrl+S`, `Ctrl+Enter`, ou o botão de confirmar | Escolhe a subpasta selecionada ou, senão, a pasta exibida |
| `Esc` | Fecha as sugestões, ou cancela |

Quando o campo de caminho mostra um caminho novo, o caminho inteiro fica selecionado: a primeira
tecla que você digitar o substitui.

### Diálogos Init e Clonar

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Campo ou botão seguinte / anterior |
| `Enter` | Pressiona o botão focado, ou cria / clona |
| `Space` | Alterna a caixa de seleção focada, ou pressiona o botão focado |
| `Ctrl+B` | Procura a pasta |
| `j` / `k`, `g` / `G` | Na lista de origens e na lista de repositórios do diálogo Clonar: desce / sobe, primeiro / último (não no campo de busca) |
| `Esc` | Fecha. Durante um clone, o clone continua em segundo plano |

No diálogo Clonar, a coluna da esquerda lista **Clonar por URL** e uma origem
por perfil de hospedagem conectado (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit).
Escolha uma origem com `Up` / `Down` ou um clique. Depois digite no campo de busca
**Repositório**, mova-se pela lista com as setas, `PageUp` / `PageDown` ou a
roda, e escolha uma linha com `Enter` ou um clique. Veja
[Primeiros passos](getting-started.md) para os campos.

## Painel Refs

| Tecla | Ação |
|---|---|
| `Enter` / `Space` | Expande ou recolhe a seção ativa, ou a pasta selecionada |
| `h` / `l`, `Left` / `Right` | Trocam de seção |
| `/` | Filtra por nome (`Enter` aplica, `Esc` cancela) |
| `y` | Liga ou desliga a visão hierárquica de branches: pastas a partir das `/` nos nomes das branches (LOCAL e REMOTO) |
| `o` | **Solo** na ref selecionada, ou em todas as refs sob a pasta ou o remoto selecionado. Faça solo de várias refs para ver a união delas |
| `Shift+O` | **Parar solo**: limpa todos os solos |
| `Shift+H` | **Mostrar tudo**: mostra de novo toda ref oculta |
| `g` | **Mostrar no grafo**: rola o Log até o commit da ref (ou do stash) selecionada e faz a linha piscar; a seleção não se move |
| `m` | **Marca** a ref selecionada como origem do soltar (`Esc`, ou `m` nela de novo, remove a marca) |
| `Shift+M` | **Solta a ref marcada** na ref selecionada: abre o menu que um arrastar e soltar abre (Merge, Rebase, Push, Fast-forward...) |
| `+` (seção REMOTO) | Adiciona um remoto |
| `.`, `Menu`, `Shift+F10`, ou um clique com o botão direito | O menu da linha: uma branch, uma tag, o nome de um remoto ou uma pasta |
| `Enter` em uma linha de WORKTREES | Abre esse worktree em sua própria aba |
| `Ctrl+G d` | Exclui a branch ou tag selecionada (pergunta antes). Em uma linha REMOTO: exclui essa branch remota |

Para o que os menus fazem, veja [Trabalhando com o Log](log-and-history.md).

### Pull requests e issues

Com uma integração correspondente, o painel Refs tem uma seção de pull requests (GitLab:
merge requests). Com um perfil de hospedagem conectado, ele também tem uma seção
ISSUES. Veja [Integrações de hospedagem e de rastreadores de issues](hosting-integration.md#5-pull-requests-e-issues-na-barra-lateral).

| Tecla | Ação |
|---|---|
| `Enter` | Abre a linha selecionada no navegador. Em uma linha `Reconectar`: abre as Configurações nesse perfil |
| `Space` | Expande ou recolhe a seção |
| `v` | Abre o filtro da seção. Pull requests: Abertos, Merged / fechados, Todos. Issues: Abertas, Atribuídas a mim, Fechadas, Todas |
| `n` | Seção de pull requests: cria um pull request. Seção ISSUES: nova issue. (O botão `+` do cabeçalho) |
| `e` | Seção de pull requests: edita o pull request selecionado no host (o lápis da linha) |
| `Enter` na linha de busca | Digita no campo de busca. `Enter`, `Tab` ou `Esc` saem dele; `Ctrl+U` o limpa |
| `Enter` em um cabeçalho de grupo | Recolhe ou expande o grupo (*Meus pull requests*, *Atribuídos a mim*, *Aguardando minha revisão*, *Todos os pull requests*) |
| `Enter` na linha de filtro, provedor ou repositório | Abre esse menu suspenso. A lista de provedores termina com **Nenhum (desvincular este repositório)** |
| `.`, ou um clique com o botão direito | O menu da linha: abrir no navegador, copiar URL ou número, fazer checkout da branch de origem (pull requests); criar uma branch a partir da issue (issues) |

### O quadro de issues

**Quadro** no cabeçalho de ISSUES (paleta: **Issues: abrir quadro**) abre o quadro Kanban da origem das issues. Veja
[o quadro](hosting-integration.md#5b-o-quadro-de-issues) e [issues locais](local-issues.md#o-quadro-kanban). Toda tecla
abaixo também é um comando da paleta (**Quadro: ...**).

| Tecla | Ação |
|---|---|
| Setas | Selecionam um cartão (`Cima`, `Baixo`) ou a coluna vizinha (`Esquerda`, `Direita`); `Home`, `End`, `PageUp`, `PageDown` saltam na coluna |
| `Shift+Left`, `Shift+Right` | Movem o cartão selecionado para a coluna anterior ou a seguinte |
| `m` | **Mover cartão para...**: uma lista suspensa das colunas |
| `Enter` | Abre a issue (a nota, para uma issue local; a página do host nos outros casos) |
| `e`, `.` | Editam responsáveis, rótulos ou marco; o menu do cartão |
| `n` | Nova issue na coluna selecionada |
| `a`, `l`, `M`, `/` | Filtram por responsável, rótulo, marco, texto. `x` limpa os filtros |
| `c` | Os rótulos que são as colunas de um quadro do GitHub / GitLab |
| `r` | Lê as issues de novo |
| `Esc` | Fecha o quadro |

Mouse: arraste um cartão para outra coluna para movê-lo, clique em um cartão para selecioná-lo, clique duplo para abri-lo, clique com o botão direito para o
menu do cartão, roda para rolar uma coluna, `Shift`+roda (ou a barra sob o quadro) para rolar as colunas.

### Flow (fluxos de trabalho de branches)

Com um fluxo de trabalho configurado para o repositório (veja [fluxos de trabalho de branches](branching-workflows.md)), a barra de ferramentas tem um
botão **Flow**, e a barra lateral agrupa LOCAL e REMOTE por tipo de branch. Cada linha do menu é um comando da paleta
(**Flow: ...**; **Flow: iniciar Feature...** e **Flow: nova issue de Feature...** são gerados para cada tipo de tópico do
fluxo de trabalho). As linhas conduzidas pelo host (*Nova issue de <Tipo>*, *Assumir uma branch criada pelo host*) e os diálogos delas, o cartão de espera e o
cartão "uma branch apareceu" estão descritos em [fluxos de trabalho de branches](branching-workflows.md#fluxos-conduzidos-pelo-host-no-aplicativo);
`Enter` executa o botão em foco (*Criar issue*, *Assumir*), `Esc` para de esperar ou fecha.

| Tecla | Ação |
|---|---|
| `Ctrl+G Shift+W` | Abre o menu Flow (o botão da barra de ferramentas) |
| `Enter`, `Tab`, `Shift+Tab` | No diálogo Iniciar / Concluir / Sincronizar / Promover / Definir pai: pré-visualizar ou executar, próximo campo, campo anterior |
| `Space` | Marca uma caixa de seleção (*Publicar*, *Executar os hooks*, *Atualizar também as branches empilhadas sobre ela*) |
| `Esc` | Fecha uma lista suspensa, volta de uma pré-visualização para o formulário, fecha o diálogo (não enquanto uma execução está em andamento) |
| Passar o mouse sobre uma branch | O tipo dela, o pai e quantos commits ela está atrás do pai |

## Painel Log

| Tecla | Ação |
|---|---|
| `Enter` / `l` | Entra na lista de arquivos alterados do commit selecionado |
| `Enter` / `l` em uma linha de arquivo | Abre o diff do arquivo no centro |
| `Esc` / `h` | Volta aos commits |
| `Right` | (commit com mais de uma ref) Abre a lista das outras refs: `Up` / `Down` movem, `Enter` faz checkout, `Esc` fecha |
| `f` | Abre a barra de busca (veja [Busca](#busca)) |
| `c` | (com alterações não commitadas) Seleciona a linha WIP e põe o cursor no campo de mensagem dela |
| `Enter` na linha WIP | Põe o cursor no campo de mensagem da linha WIP |
| `.`, `Menu`, `Shift+F10` | O menu do commit. `Up` / `Down` movem, `Right` / `l` / `Enter` abrem um submenu, `Left` / `h` / `Esc` voltam |
| `m` / `a` | Arquivos do painel da direita: `Caminho` ou `Árvore` / `Ver todos os arquivos` |
| `Shift+C` | Escolhe uma coluna do Log para mover ou redimensionar (veja abaixo) |

Com uma coluna escolhida, `Tab` / `Shift+Tab` escolhem outra, `<` / `>` a movem, `]` / `[` a alargam ou estreitam, `0` devolve todas as larguras ao automático e `Esc` ou `Enter` saem (contexto `log_columns` na tabela gerada acima).

O campo de mensagem da linha WIP é o campo de resumo do compositor de commits. Enquanto ele tem
o cursor, toda tecla é digitada, mas os acordes `Ctrl` ainda funcionam. `Enter` vai para
a descrição do compositor. `Esc` sai do campo e mantém o texto. Veja
[Stage e commit](staging-and-committing.md).

### Selecionando vários commits

| Tecla ou gesto | Ação |
|---|---|
| `Space` | Adiciona o commit atual à seleção, ou o remove |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Estendem a seleção (Alt para os terminais que ficam com o Shift) |
| `Ctrl+clique` / `Shift+clique` ou `Alt+clique` | Alterna um commit / seleciona um intervalo |
| `Esc` | Limpa a seleção |
| `Shift+Y` | [Copia a seleção como um gitGraph do Mermaid](log-and-history.md#copiando-commits-como-um-gitgraph-do-mermaid) (um popup com níveis de detalhe e uma pré-visualização ao vivo; `Enter` copia) |

Com vários commits selecionados, clique com o botão direito em um deles e escolha **Cherry-pick
de N commits em `<branch>`...**. Ele abre a mesma visualização de um rebase
interativo.

### Dando nome a uma branch ou tag

`Ctrl+G b`, `Ctrl+G t`, os botões **Branch** e **Tag** da barra de ferramentas e os itens
**Criar branch aqui...** / **Criar tag aqui...** dos menus pedem o nome em
uma caixa de texto na linha do commit. `Enter` cria a ref. `Esc` cancela. Um nome
inválido ou já existente aparece em vermelho, e a caixa continua aberta.

### Editando a mensagem de um commit

**Editar mensagem do commit...** no menu do commit, `Ctrl+G e`, ou o
`Git: editar mensagem do commit...` da paleta abre o popup **Reescrever mensagem do commit**. Veja
[Trabalhando com o Log](log-and-history.md) para o que ele altera.

| Tecla | Ação |
|---|---|
| `Enter` no resumo | Vai para a descrição |
| `Enter` na descrição | Nova linha. `Up` na primeira linha volta ao resumo |
| `Ctrl+Enter`, `Ctrl+S`, ou **Atualizar mensagem** | Salvam a nova mensagem |
| `Tab` / `Shift+Tab` | Movem entre o resumo, a descrição e os botões |
| `Esc`, ou **Cancelar** | Fecham sem alteração |

### Alterando um commit

`Ctrl+G Shift+E` (todos os campos) e `Ctrl+G Shift+A` (o autor) abrem o diálogo Alterar, assim como o menu do commit, as linhas `Commit: alterar...` da paleta e o painel de detalhes do commit; todos os caminhos estão descritos em [Alterando um commit](log-and-history.md#alterando-um-commit-mensagem-autor-datas). Os dois atalhos podem ser remapeados: `git_prefix.amend` e `git_prefix.edit_author`. Teclas dentro do diálogo:

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab` | Move entre os campos e os botões |
| `Enter` em um campo de uma linha | Próximo campo; depois do último, **Aplicar** |
| `Up` / `Down` | Campo anterior / próximo (na descrição, a partir da primeira / última linha) |
| `Ctrl+Enter`, `Ctrl+S` ou **Aplicar** | Aplica os campos alterados |
| `Esc` ou **Cancelar** | Fecha sem alterar nada |
| `Alt+H` ou **Modificar o histórico mais a fundo...** | Fecha o diálogo e abre o painel de reescrita do histórico a partir deste commit (só em compilações com reescrita do histórico); pergunta antes se você editou um campo (`Esc` confirma). Pode ser remapeado: `reword.deeper` |

### Criar worktree

**Criar worktree a partir de `<ref>`...** (menus de branch e tag) e **Criar
worktree aqui...** (menu do commit) abrem um diálogo: a pasta, uma nova
branch opcional e **Abrir o novo worktree como aba**. Teclas: `Tab` / `Shift+Tab`
movem, `Enter` cria, `Space` alterna a caixa de seleção, `Ctrl+B` procura,
`Esc` cancela.

### Rebase interativo

**Rebase interativo a partir daqui...** no menu do commit, ou
`Git: rebase interativo a partir do commit selecionado...`, abre a visualização de rebase. Um
cherry-pick de vários commits usa a mesma visualização. Veja
[Trabalhando com o Log](log-and-history.md).

| Tecla | Ação |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Definem a ação da linha: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup e Edit apenas em um rebase) |
| `Space` | Abre a lista de ações da linha (`Up` / `Down` escolhem, `Enter` aplica, `Esc` fecha a lista) |
| `Up` / `Down` | Movem a seleção |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Movem a linha selecionada para cima / para baixo |
| `a` | Liga ou desliga a prévia de autosquash |
| `Enter` em uma linha Reword | Edita a nova mensagem desse commit |
| `Ctrl+S`, `Ctrl+Enter`, ou **Iniciar rebase** / **Iniciar cherry-pick** | Executam o plano |
| `Ctrl+R`, ou **Redefinir** | Restauram o plano como estava quando a visualização abriu |
| `Esc`, ou **Cancelar** | Fecham sem alteração |

Quando um rebase para (em um conflito ou em uma etapa Edit), um banner acima do Log
oferece **Continuar**, **Pular** e **Abortar**. A paleta tem
`Rebase: continuar`, `Rebase: pular` e `Rebase: abortar`.

## Busca

`f`, o botão **Buscar** da barra de ferramentas, ou `Navegação: buscar commits...` abre
a barra de busca sobre o Log. Ela busca na mensagem completa do commit, no nome
e e-mail do autor, no SHA e nos nomes das refs. Ela não oculta linhas: as linhas que não
correspondem ficam esmaecidas.

| Tecla | Ação |
|---|---|
| *(digitar)* | Edita o texto da busca (toda tecla é digitada) |
| `Enter` / `Down`, ou o `↑` da barra | Próxima correspondência |
| `Shift+Enter` / `Up`, ou o `↓` da barra | Correspondência anterior |
| `Esc`, ou o `✕` da barra | Fecham a barra de busca |

## Visualização Diff

Um arquivo alterado abre no centro, no lugar do Log. Veja
[Stage e commit](staging-and-committing.md).

| Tecla | Ação |
|---|---|
| `Enter` / `l`, ou um clique duplo (em uma linha de arquivo) | Abrem o diff do arquivo |
| `Esc`, `q`, ou o `✕` no título | Fecham o diff |
| `j` / `k`, `Up` / `Down` | Rolam |
| `PageUp` / `PageDown`, `Home` / `End` | Rolam uma página; vão para o início / o fim |
| `h` / `l`, `Left` / `Right` | Rolam para os lados (não com quebra de linha) |
| `n` / `p` | Hunk seguinte / anterior |
| `m` | Layout inline ou dividido |
| `Shift+H` | Só hunks ou arquivo inteiro |
| `w` | Liga ou desliga a quebra de linha |
| `s` | Mostra ou oculta alterações só de espaços em branco |
| `Shift+M` | Liga ou desliga o minimapa |
| `Shift+C` | Escolhe a codificação em que o arquivo é lido (uma lista: Detecção automática primeiro) |
| `e` | Edita o arquivo no editor integrado (veja [Editor](#editor)) |
| `r` | (arquivo Git LFS) O cartão LFS ou o ponteiro bruto. Veja [Git LFS](lfs.md) |
| `Shift+E` | (arquivo em conflito) Abre o editor de merge. Veja [Resolver conflitos de merge](merge-conflicts.md) |
| `y` / `Ctrl+C` | Copiam o texto selecionado. Sem seleção: a linha sob o cursor, ou as linhas selecionadas |
| `.`, ou um clique com o botão direito em um hunk | O menu do hunk |

### Stage e descarte a partir do diff

Estas teclas funcionam somente em um diff das alterações em andamento (aberto pela
lista Unstaged ou Staged). Descartar funciona somente na lista Unstaged.

| Tecla | Ação |
|---|---|
| `Space` | Faz stage (Unstaged) / unstage (Staged) do hunk sob o cursor |
| `v` | Inicia ou descarta uma seleção de linhas; `j` / `k` a estendem |
| `Enter` | Faz stage / unstage das linhas selecionadas, ou da linha sob o cursor |
| `x` | Descarta o hunk sob o cursor (pergunta antes: `y` / `Enter` confirmam, `n` / `Esc` cancelam) |
| `Shift+X` | Descarta as linhas selecionadas, ou a linha sob o cursor (pergunta antes) |

Só as linhas alteradas contam: as linhas de contexto em uma seleção são ignoradas. Uma
seleção que alcança um segundo hunk age somente sobre o primeiro hunk.

## Visualização de imagem

Uma imagem na visualização Diff ou na visualização de arquivo. As teclas estão no contexto
`image` da tabela acima. Veja [Imagens](image-view.md).

## Visualização de arquivo, Blame e Histórico do arquivo

| Tecla | Ação |
|---|---|
| `Esc` / `q` | Fecham (com uma seleção de texto, `Esc` a limpa primeiro) |
| `j` / `k`, `Up` / `Down` | Movem a seleção |
| `PageUp` / `PageDown` | Rolam uma página |
| `Home` / `End`, `Shift+G` | Primeira / última linha |
| `h` / `l`, `Left` / `Right` | Rolam o texto para os lados |
| `Enter` | (Blame, Histórico do arquivo) Vai para o commit da linha no Log |
| `e` | Edita o arquivo no editor integrado |
| `y` / `Ctrl+C` | Copiam o texto selecionado. Sem seleção: a linha selecionada (Blame: só o código; Histórico do arquivo: o resumo) |

O Blame e o histórico do arquivo têm teclas próprias, listadas na tabela gerada acima (contextos `file_insight` e `history`) e como `Blame: ...` / `Histórico do arquivo: ...` na paleta; todas podem ser remapeadas. Detalhes em [Blame](log-and-history.md#blame) e [Histórico do arquivo](log-and-history.md#histórico-do-arquivo). Na lista do histórico, as letras soltas (`d` `o` `b` `c` `y`) agem sobre a linha selecionada, mas um acorde com Ctrl (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) mantém sempre seu significado global.

## Coluna de commits

A coluna à esquerda da visualização de um arquivo (veja [A coluna de commits](interface-tour.md#a-coluna-de-commits)). `Alt+C` a mostra ou oculta de qualquer lugar; as outras teclas funcionam quando ela tem o teclado (`Ctrl+N` e o selo dela, ou `Exibir: focar a coluna de commits`).

| Tecla | Ação |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Escolhem um commit |
| `Enter` / `Right` | Vão para a lista de arquivos do commit escolhido |
| `Esc` | Devolve o teclado à visualização |
| `.`, `Menu`, `Shift+F10` | O menu da linha escolhida |

As outras teclas dela (`Shift+Left` / `Shift+Right` para rolar para os lados, `Tab`, `]`, `[` e `0` para dimensionar as partes) estão na tabela gerada acima, contexto `commit_column`.

## Editor

**Editar**, ou `e` em um diff ou em uma visualização de arquivo, abre o editor integrado na
cópia do arquivo no diretório de trabalho. Veja
[Stage e commit](staging-and-committing.md).

| Tecla | Ação |
|---|---|
| *(digitar)* | Insere no cursor (substitui a seleção) |
| `Enter` | Nova linha, com a indentação da linha atual |
| `Tab` / `Shift+Tab` | Aumentam / diminuem a indentação (das linhas selecionadas, se houver várias) |
| setas, `Home` / `End` | Movem. `Home` vai ao primeiro caractere não branco e depois à coluna 0. `Shift` estende a seleção |
| `Ctrl+Left` / `Ctrl+Right` | Movem uma palavra |
| `Ctrl+Home` / `Ctrl+End` | Início / fim do arquivo |
| `PageUp` / `PageDown` | Uma página para cima / para baixo |
| `Ctrl+A` | Seleciona tudo |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copiar / recortar / colar |
| `Ctrl+U` / `Ctrl+K` | Apagam até o início / o fim da linha |
| `Ctrl+Z` | Desfazer |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Refazer |
| `Ctrl+S` | Salvar |
| `Ctrl+F` / `F3` | Buscar. No campo de busca: `Enter` / `Down` / `F3` próximo, `Shift+Enter` / `Up` anterior, `Esc` fecha |
| `Alt+M` | Liga ou desliga o minimapa (**Map**) |
| `Esc` | Fecha. Com alterações não salvas ele pergunta: **Salvar** (`S` / `Enter`), **Descartar** (`D`), **Cancelar** (`C` / `Esc`) |

`Tab` indenta no editor, então não move o foco. Os outros acordes `Ctrl`
(`Ctrl+P`, `Ctrl+N`) continuam funcionando.

A **codificação**, as **quebras de linha** e a **indentação** da linha de status são
botões: cada um abre uma lista suspensa (setas ou roda, digite para filtrar, `Enter` para
escolher, `Esc` para fechar). A paleta (`Ctrl+P`) abre os mesmos menus: *Editor: alterar codificação*, *Editor: reabrir com codificação*, *Editor: converter para codificação*, *Editor: converter quebras de linha*, *Editor: alterar indentação*.
Detalhes em [Stage e commit](staging-and-committing.md).

## Ações do Git (`Ctrl+G`)

`Ctrl+G` espera mais uma tecla:

| Tecla | Ação |
|---|---|
| `s` / `u` | Fazem stage / unstage do arquivo selecionado |
| `a` | Faz stage de todas as alterações |
| `x` | Descarta todas as alterações (pergunta antes) |
| `c` | Abre o compositor de commits |
| `Enter` | Faz agora o commit das alterações em stage, com a mensagem do formulário, de qualquer lugar (veja [Stage e commit](staging-and-committing.md#escrevendo-o-commit)) |
| `m` | Faz stash de todas as alterações com uma mensagem (vazia é permitido). Com a IA ligada, `Ctrl+G` neste prompt escreve a mensagem |
| `Shift+T` | Faz stash de todas as alterações (arquivos não rastreados incluídos), sem mensagem |
| `o` | Faz pop de um stash: a linha de stash selecionada no Log ou, senão, o stash mais recente |
| `p` | Pull, do jeito que o botão Pull está configurado (`git.pull_mode`) |
| `Shift+P` | Push para o remoto padrão (define o upstream no primeiro push), ou pergunta o remoto |
| `Shift+U` | Push de todas as tags |
| `f` | Fetch de todos os remotos, com prune |
| `g` | Abre no navegador o pull request da branch em checkout |
| `b` / `t` | Nova branch / nova tag em HEAD (veja [Dando nome a uma branch ou tag](#dando-nome-a-uma-branch-ou-tag)) |
| `d` | Exclui a branch ou tag selecionada (pergunta antes) |
| `e` | Edita a mensagem do commit selecionado |
| `Shift+E` / `Shift+A` | Altera o commit selecionado (todos os campos) / o autor dele (veja [Alterando um commit](#alterando-um-commit)) |
| `y` | Cherry-pick do commit selecionado |
| `v` | Revert do commit selecionado |
| `Shift+R` | Reset (mixed) para o commit selecionado |
| `r` | Rebase sobre a branch selecionada |
| `Shift+W` | Abre o menu Flow (em um repositório com fluxo de trabalho de branches) |
| `Shift+H` | Abre o painel de reescrita do histórico (só em compilações com reescrita do histórico) |
| `l` | Abre o menu LFS. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch all / checkout all / pull / push all / prune. Veja [Git LFS](lfs.md) |
| `i` | ✦ IA: escreve a mensagem de commit das alterações em stage |
| `w` | ✦ IA: explica o commit selecionado, ou as alterações em andamento |
| `Shift+C` | ✦ IA: propõe como dividir as alterações em andamento em commits |

As teclas de IA só funcionam enquanto a IA está ligada (veja [Assistência de IA](ai.md)). `Esc`
cancela uma solicitação de IA em andamento.

## Painel da direita (alterações em andamento e detalhes do commit)

Com a linha WIP selecionada, o painel da direita mostra as listas **Unstaged** e
**Staged** e o compositor de commits. Com um commit selecionado, mostra os
detalhes do commit e seus arquivos.

| Tecla | Ação |
|---|---|
| `j` / `k`, `Up` / `Down` | Movem. No fim de uma lista, a seleção continua na outra |
| `h` / `l`, `Left` / `Right` | Vão para a lista Unstaged / Staged |
| `Enter` | Recolhe ou expande a lista ativa |
| `Space` / `Insert` | Adicionar o arquivo atual à seleção, ou removê-lo |
| `Ctrl+A` | Selecionar todos os arquivos da lista |
| `Ctrl+Space` | Faz stage / unstage do arquivo selecionado, ou recolhe uma linha de pasta |
| `d` | Abre o diff do arquivo selecionado |
| `m` | `Caminho` (lista plana) ou `Árvore` (pastas) |
| `a` | `Ver todos os arquivos`: a árvore inteira, não só os arquivos alterados |
| `z` | Recolhe todas as pastas, ou as expande de novo |
| `/` | Filtra por nome os arquivos alterados de um commit |
| `.`, `Menu`, `Shift+F10` | O menu do arquivo |
| `c` | Foca o campo de resumo do compositor de commits |
| `Shift+A` | Liga ou desliga **Amend do commit anterior** |
| `Ctrl+N ]` / `Ctrl+N [` | Aumentam / diminuem o bloco da mensagem de commit |

No compositor de commits:

| Tecla | Ação |
|---|---|
| `Enter` | Resumo: vai para a descrição. Descrição: nova linha |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` faz o mesmo de qualquer lugar) |
| `Alt+R` | Insere uma referência de issue |
| `Esc` | Sai do compositor |

O menu do arquivo contém: Stage ou Unstage, Descartar alterações, Ignorar, Stash do arquivo,
Histórico do arquivo, Blame do arquivo, Abrir na ferramenta de diff externa, Abrir no editor externo,
Abrir arquivo no programa padrão, Mostrar na pasta, Copiar caminho do arquivo, Criar patch
das alterações do arquivo, Editar arquivo e Excluir arquivo. Em um arquivo em conflito ele começa
com **Abrir no editor de merge** e as três linhas **Resolver com**. Com o Git
LFS ligado, ele tem linhas de LFS. **Descartar alterações** e **Excluir arquivo** perguntam antes.

## Botões da janela

Todo botão tem um equivalente no teclado:

| Botão | Tecla |
|---|---|
| 🐙 à esquerda da barra de título (Sobre) | `Ajuda: sobre o GitPoulpi` na paleta |
| **Abrir em** na barra de título | `Alt+O`, `Ctrl+Alt+O` |
| `×` à direita da barra de título (sair) | `q` |
| A versão na barra de título (Notas da versão) | `Ajuda: Notas da versão` na paleta |
| `⌕ Busca global` na barra de título, à esquerda do `⚙` | `Ctrl+Alt+F` |
| `⚙` na barra de título (Configurações) | `Ctrl+,` |
| A etiqueta do espaço de trabalho na linha de abas | `Alt+W` |
| `+` depois da última aba | `Alt+T` |
| A aba **Radar** | `Alt+R` |
| A aba **Espaços de trabalho** | `Alt+J` |
| **Atividade** na linha de status | `Ctrl+N Shift+A` |
| Barra de ferramentas: **Desfazer**, **Refazer** | `Ctrl+Z`, `Ctrl+Y` |
| Barra de ferramentas: **Pull** / seu `▾` | `Ctrl+G p` / `Git: opções de pull...` |
| Barra de ferramentas: **Push** / seu `▾` | `Ctrl+G Shift+P` / `Git: Push para um remoto...` |
| Barra de ferramentas: **Branch**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Barra de ferramentas: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Barra de ferramentas: **LFS** (quando o LFS está ligado) | `Ctrl+G l` |
| Barra de ferramentas: **Flow** (quando o repositório tem um fluxo de trabalho) | `Ctrl+G Shift+W` |
| Barra de ferramentas: **Buscar** | `f` |
| Barra de ferramentas: o pull request vinculado (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Barra de ferramentas: a lista de perfis (com dois ou mais perfis) | `Repositório: alterar perfil...` |

A barra de ferramentas é descrita no [Tour da interface](interface-tour.md#barra-de-ferramentas).

## Configurações

`Ctrl+,`, `Ctrl+N ,`, a engrenagem `⚙`, ou `App: configurações (preferências)...` abre
a tela de Configurações.

| Tecla | Ação |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Movem entre a lista de páginas e as opções |
| `j` / `k`, `Up` / `Down` | Movem a seleção |
| `Enter` / `Space` | Abrem uma página, alternam uma caixa de seleção, editam um campo, abrem um menu suspenso ou pressionam um botão |
| `/`, `Ctrl+F` | Busca em todas as opções |
| `Alt+Left` / `Alt+Right` | Troca de subaba (Geral / a lista) |
| `Esc` / `q` | Fecham as Configurações. Enquanto um campo é editado, `Esc` cancela só a edição |
| `?` | Abre a ajuda sobre a tela de Configurações |

Veja [Configurações](settings.md).

## Campos de texto

Todos os campos de texto compartilham as mesmas teclas: filtros, campos de busca, a paleta
de comandos, prompts, o compositor de commits, o campo da linha WIP, campos de diálogo e
campos das Configurações. Enquanto um campo está com o teclado, estas teclas prevalecem sobre os
atalhos do painel.

| Tecla | Ação |
|---|---|
| um caractere | Digita no cursor (substitui a seleção) |
| `Left` / `Right` | Movem um caractere (com seleção: vão para o início / o fim dela) |
| `Ctrl+Left` / `Ctrl+Right` | Movem uma palavra |
| `Home` / `End` | Início / fim da linha |
| `Shift` + um movimento acima | Estende ou reduz a seleção |
| `Up` / `Down` (só na descrição) | Linha anterior / seguinte |
| `Ctrl+Home` / `Ctrl+End` (só na descrição) | Início / fim do texto |
| `Ctrl+A` | Seleciona tudo |
| `Backspace` / `Delete` | Apagam o caractere antes / depois do cursor, ou a seleção |
| `Ctrl+U` | Apaga do cursor até o início da linha (ou a seleção) |
| `Ctrl+K` | Apaga do cursor até o fim da linha (ou a seleção) |
| `Ctrl+C` / `Ctrl+X` | Copiam / recortam a seleção (não em um campo de senha) |
| colar (o colar do próprio terminal) | Insere no cursor, substituindo a seleção |

Na descrição do commit, `Enter` digita uma nova linha e `Ctrl+Enter` faz commit.
Nem todos os terminais enviam `Ctrl+Enter` do mesmo jeito, então `Alt+Enter` e
`Ctrl+J` também fazem commit.

Alguns consoles enviam `Ctrl+letra` como um caractere de controle (`Ctrl+A` como `0x01`).
O GitPoulpi entende as duas formas. Os caracteres `AltGr` (`@`, `#`, `€` em teclados suíços
ou franceses) são digitados.

## Dicas

Uma dica aparece quando o ponteiro fica cerca de 0,6 s sobre um alvo (uma célula do Log,
um botão da barra de ferramentas, o pull request vinculado). Um toque de tecla, um clique, uma rolagem, um
redimensionamento, um popup ou menu aberto, ou o ponteiro saindo do alvo a oculta.
Depois disso, ela só volta quando o ponteiro se mexe de novo.
