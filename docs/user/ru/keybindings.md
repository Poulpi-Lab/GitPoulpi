[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · **Русский** · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Сочетания клавиш

Эта страница перечисляет клавиши **по умолчанию**. Большинство из них можно
переназначить. Приложение всегда показывает действующие клавиши: в палитре
команд, в справке `?`, в строках состояния, контекстных меню и подсказках кнопок.
Когда вы меняете клавишу, в этих местах появляется новая. О мыши см.
[Мышь и жесты](mouse-and-gestures.md).

## Переназначение

Откройте **Настройки > Клавиатура и мышь**. Страница описана в
[Настройках](settings.md#клавиатура-и-мышь). Изменения применяются сразу и
сохраняются в таблице `[keybindings]` файла `config.toml`. Эту таблицу можно
править и вручную:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux и GNU screen часто используют Ctrl+B и Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

Синтаксис, правила конфликтов и предупреждения описаны в
[Справочнике по конфигурации](config-reference.md) в разделе `[keybindings]`.
Коротко:

- Ошибка сообщается в строке состояния, а клавиша по умолчанию остаётся.
- Два действия на одном сочетании в одном контексте отклоняются.
- Префиксу нужен `Ctrl` или `Alt`.
- Сочетание панели, скрывающее глобальное сочетание, вызывает только
  предупреждение.

В таблице ниже перечислены все клавиши, которые можно переназначить. Некоторые
клавиши фиксированы и не переназначаются: перемещение по списку (`j` `k` `h`
`l`, стрелки, `PageUp` / `PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`,
цифры (префикс числа), клавиши редактирования текста и клавиши `Enter` / `Tab` /
стрелок во всплывающих окнах. В палитре есть `Приложение: сочетания клавиш...` и
`Сочетания клавиш: сбросить все...`.

## Сочетания клавиш по умолчанию

Все клавиши, которые можно переназначить, с идентификатором для
`[keybindings.<контекст>]` и значением по умолчанию. Таблица генерируется из
приложения (тест поддерживает её актуальность). Клавиши после префикса показаны
вместе с префиксом (`Ctrl+G s`).

<!-- keymap:begin -->
| Контекст | Id действия | Что делает | Клавиши по умолчанию |
|---|---|---|---|
| `global` | `git_prefix` | Префикс действий Git | `Ctrl+G` |
| `global` | `nav_prefix` | Префикс навигации | `Ctrl+N` |
| `global` | `palette` | Палитра команд | `Ctrl+P` |
| `global` | `settings` | Настройки | `Ctrl+,` |
| `global` | `home` | Главная (управление репозиториями) | `Ctrl+O` |
| `global` | `clone` | Клонировать репозиторий | `Ctrl+Shift+N` |
| `global` | `new_tab` | Новая вкладка | `Alt+T` |
| `global` | `workspace_switch` | Сменить рабочее пространство | `Alt+W` |
| `global` | `radar` | Открыть Radar | `Alt+R` |
| `global` | `workspaces` | Открыть рабочие пространства | `Alt+J` |
| `global` | `tab_menu` | Меню вкладки | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Открыть в (список) | `Alt+O` |
| `global` | `tab_open_in_default` | Открыть в записи по умолчанию | `Ctrl+Alt+O` |
| `global` | `close_tab` | Закрыть вкладку | `Ctrl+W` |
| `global` | `next_tab` | Следующая вкладка | `Ctrl+PageDown` |
| `global` | `prev_tab` | Предыдущая вкладка | `Ctrl+PageUp` |
| `global` | `help` | Справка | `?` |
| `global` | `quit` | Выход | `q` |
| `global` | `refresh` | Обновить | `r` |
| `global` | `refresh_all` | Обновить (fetch, pull requests, задачи) | `F5` |
| `global` | `find_in_log` | Поиск по графу commit'ов | `f` |
| `global` | `workspace_search` | Поиск в рабочем пространстве | `Ctrl+Alt+F` |
| `global` | `select_tab` | Выбрать вкладку | `t` |
| `global` | `undo` | Отменить последнюю операцию | `Ctrl+Z` |
| `global` | `redo` | Повторить отменённую операцию | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Открыть историю отмен | `Alt+Z` |
| `global` | `commit_column` | Показать или скрыть колонку commit'ов | `Alt+C` |
| `global` | `undo_offer` | Восстановить ветку туда, где её оставило заблокированное действие | `Alt+Shift+Z` |
| `global` | `context_menu` | Контекстное меню | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Написать сообщение commit'а | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Сразу сделать commit staged-изменений | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | ИИ: разбить изменения на commits | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | ИИ: объяснить изменения | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash с сообщением | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | ИИ: написать сообщение commit'а | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage всех изменений | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Отбросить все изменения | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage выбранного файла | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage выбранного файла | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Открыть связанный pull request | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push всех tags | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch всех remote | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Новый tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Новая ветка | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash изменений | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick выбранного commit'а | `Ctrl+G y` |
| `git_prefix` | `revert` | Revert выбранного commit'а | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Изменить сообщение commit'а | `Ctrl+G e` |
| `git_prefix` | `amend` | Изменить выбранный commit | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Изменить автора выбранного commit'а | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (Mixed) к выбранному commit'у | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase на выбранную ветку | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Удалить выбранную ссылку или drop stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Открыть меню LFS | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Открыть меню Flow | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: fetch всех объектов | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: checkout всех файлов | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: pull объектов | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: push всех объектов | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: очистить локальные объекты | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Открыть переписывание истории | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Свернуть или восстановить панель Refs | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Свернуть или восстановить панель Status | `Ctrl+N l` |
| `nav_prefix` | `settings` | Открыть настройки | `Ctrl+N ,` |
| `nav_prefix` | `about` | Открыть «О программе» | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Открыть Журнал активности | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Открыть общий журнал активности | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Показать фоновые задачи | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Показать панель обновлений | `Ctrl+N u` |
| `nav_prefix` | `help` | Показать/скрыть справку | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Выбрать вкладку | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | Сменить тему | `Ctrl+N y` |
| `nav_prefix` | `change_language` | Сменить язык | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | Сменить цветовую тему графа | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | Переключить подсветку строк графа | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Увеличить раздел Refs | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Уменьшить раздел Refs | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Увеличить блок сообщения commit'а | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Уменьшить блок сообщения commit'а | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Расширить панель Refs | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Сузить панель Refs | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Расширить панель Status | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Сузить панель Status | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Агент: показать ожидающий вид | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Агент: назад (или закрыть ожидающий вид) | `Ctrl+N Backspace` |
| `refs` | `filter` | Фильтр ссылок по имени | `/` |
| `refs` | `solo` | Solo для выбранного | `o` |
| `refs` | `clear_solo` | Снять solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Переключить иерархию веток | `y` |
| `refs` | `show_all` | Показать все скрытые ссылки | `Shift+H` |
| `refs` | `reveal` | Показать в графе | `g` |
| `refs` | `mark_drop` | Пометить как источник перетаскивания | `m` |
| `refs` | `drop_marked` | Перетащить сюда помеченную ссылку | `Shift+M` |
| `refs` | `expand_all` | Развернуть все папки | `=` |
| `refs` | `collapse_all` | Свернуть все папки | `-` |
| `refs` | `expand_below` | Развернуть папку и её вложенные папки | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Свернуть папку и её вложенные папки | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Файлы: путь или дерево | `m` |
| `log` | `view_all_files` | Файлы: показать все файлы | `a` |
| `log` | `compare_selection` | Сравнить выбранные строки | `v` |
| `log` | `compare_swap` | Поменять стороны сравнения | `x` |
| `log` | `compare_next` | Перейти к следующей сравниваемой строке | `n` |
| `log` | `pick_column` | Выбрать колонку для сдвига или изменения ширины | `Shift+C` |
| `log` | `scroll_left` | Прокрутить колонки влево | `Shift+Left` |
| `log` | `scroll_right` | Прокрутить колонки вправо | `Shift+Right` |
| `log` | `copy_mermaid` | Копировать commit'ы как Mermaid gitGraph | `Shift+Y` |
| `log_columns` | `next` | Выбрать следующую колонку | `Tab` |
| `log_columns` | `prev` | Выбрать предыдущую колонку | `Shift+BackTab` |
| `log_columns` | `move_left` | Сдвинуть колонку влево | `<` |
| `log_columns` | `move_right` | Сдвинуть колонку вправо | `>` |
| `log_columns` | `wider` | Расширить колонку | `]` |
| `log_columns` | `narrower` | Сузить колонку | `[` |
| `log_columns` | `reset_width` | Сбросить ширину колонок | `0` |
| `log_columns` | `fit` | Подогнать колонку под содержимое | `=` |
| `log_columns` | `fit_all` | Подогнать все колонки под содержимое | `+` |
| `log_columns` | `layout` | Прокрутка вбок или по размеру панели | `s` |
| `log_files` | `collapse_all` | Свернуть или развернуть все папки | `z` |
| `log_files` | `filter` | Фильтр изменённых файлов | `/` |
| `log_files` | `expand_all` | Развернуть все папки | `=` |
| `log_files` | `collapse_all_folders` | Свернуть все папки | `-` |
| `log_files` | `expand_below` | Развернуть папку и её вложенные папки | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Свернуть папку и её вложенные папки | `_`, `Shift+Left` |
| `status` | `compose` | Фокус на сообщении commit'а | `c` |
| `status` | `toggle_file_mode` | Файлы: путь или дерево | `m` |
| `status` | `view_all_files` | Файлы: показать все файлы | `a` |
| `status` | `amend` | Переключить amend | `Shift+A` |
| `status` | `skip_hooks` | Переключить пропуск хуков | `Shift+H` |
| `status` | `collapse_all` | Свернуть или развернуть все папки | `z` |
| `status` | `stage_toggle` | Stage или unstage выбранной строки | `Ctrl+Space` |
| `status` | `open_diff` | Открыть diff выбранного файла | `d` |
| `status` | `expand_all` | Развернуть все папки | `=` |
| `status` | `collapse_all_folders` | Свернуть все папки | `-` |
| `status` | `expand_below` | Развернуть папку и её вложенные папки | `*`, `Shift+Right` |
| `status` | `collapse_below` | Свернуть папку и её вложенные папки | `_`, `Shift+Left` |
| `home` | `favorite` | Добавить репозиторий в избранное или убрать | `f`, `*` |
| `home` | `close_repo` | Закрыть, убрать или снять звезду со строки | `x`, `Delete` |
| `home` | `browse` | Найти репозиторий (Обзор) | `b` |
| `home` | `clone` | Клонировать репозиторий | `c` |
| `home` | `init` | Инициализировать репозиторий | `i` |
| `home` | `collapse_all` | Свернуть все разделы | `-` |
| `home` | `expand_all` | Развернуть все разделы | `+`, `=` |
| `home` | `search` | Искать репозитории | `/` |
| `home` | `workspace_new` | Новое рабочее пространство | `n` |
| `home` | `workspace_edit` | Изменить рабочее пространство | `e` |
| `home` | `workspace_from_tabs` | Новое рабочее пространство из открытых вкладок | `a` |
| `home` | `workspace_fetch` | Fetch все в рабочем пространстве | `Shift+F` |
| `home` | `workspace_pull` | Pull все в рабочем пространстве | `Shift+P` |
| `home` | `workspace_clone_missing` | Клонировать отсутствующие репозитории | `m` |
| `home` | `workspace_locate` | Найти отсутствующий репозиторий | `l` |
| `refs` | `hosting_filter` | Pull requests / задачи: выбрать фильтр | `v` |
| `refs` | `hosting_edit` | Pull request: править на хостинге | `e` |
| `refs` | `new_pull_request` | Создать pull request | `n` |
| `refs` | `add_remote` | Remotes: добавить remote | `+` |
| `diff` | `close` | Закрыть diff | `q` |
| `diff` | `edit` | Править в рабочей папке | `e` |
| `diff` | `lfs_raw` | LFS: карточка или сырой указатель | `r` |
| `diff` | `copy` | Копировать выделенное или строку | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage или unstage hunk'а | `Space` |
| `diff` | `stage_lines` | Stage или unstage выбранных строк | `Enter` |
| `diff` | `discard_hunk` | Отбросить hunk | `x` |
| `diff` | `discard_lines` | Отбросить выбранные строки | `Shift+X` |
| `diff` | `select_lines` | Начать или снять выделение строк | `v` |
| `diff` | `next_hunk` | Следующий hunk | `n` |
| `diff` | `prev_hunk` | Предыдущий hunk | `p` |
| `diff` | `toggle_layout` | Вид: единый или рядом | `m` |
| `diff` | `toggle_hunks_only` | Только hunk'и или весь файл | `Shift+H` |
| `diff` | `toggle_minimap` | Переключить миникарту | `Shift+M` |
| `diff` | `choose_encoding` | Выбрать кодировку | `Shift+C` |
| `diff` | `toggle_wrap` | Переключить перенос строк | `w` |
| `diff` | `toggle_whitespace` | Показать или скрыть изменения только пробелов | `s` |
| `diff` | `merge_editor` | Открыть редактор merge (файл с конфликтом) | `Shift+E` |
| `file_insight` | `close` | Закрыть просмотр файла | `q` |
| `file_insight` | `edit` | Править в рабочей папке | `e` |
| `file_insight` | `copy` | Копировать выделенное или строку | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | К последней строке | `Shift+G` |
| `file_insight` | `open_commit` | Перейти к commit'у строки | `Enter` |
| `history` | `show_in_log` | Показать в Log | `Enter` |
| `history` | `copy_sha` | Копировать SHA commit'а | `y` |
| `history` | `copy_summary` | Копировать краткое описание | `Shift+Y` |
| `history` | `show_diff` | Показать diff этого commit'а | `d`, `Alt+1` |
| `history` | `open_file` | Открыть файл на этом commit'е | `o`, `Alt+2` |
| `history` | `blame` | Blame на этом commit'е | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Сравнить с рабочим деревом | `c` |
| `history` | `restore` | Восстановить эту версию | `Shift+R` |
| `history` | `range_up` | Расширить выделение вверх | `Shift+Up` |
| `history` | `range_down` | Расширить выделение вниз | `Shift+Down` |
| `history` | `switch_focus` | Переключиться между списком и diff'ом | `Tab` |
| `history` | `widen_list` | Сделать список шире | `Alt+Right` |
| `history` | `narrow_list` | Сделать список уже | `Alt+Left` |
| `history` | `reset_split` | Сбросить ширину списка | `Alt+0` |
| `commit_column` | `scroll_left` | Прокрутить колонку влево | `Shift+Left` |
| `commit_column` | `scroll_right` | Прокрутить колонку вправо | `Shift+Right` |
| `commit_column` | `next_part` | Ширина следующей части | `Tab` |
| `commit_column` | `move_left` | Сдвинуть часть влево | `<` |
| `commit_column` | `move_right` | Сдвинуть часть вправо | `>` |
| `commit_column` | `wider_part` | Расширить часть | `]` |
| `commit_column` | `narrower_part` | Сузить часть | `[` |
| `commit_column` | `reset_widths` | Сбросить ширину частей | `0` |
| `file_insight` | `blame_card` | Blame: Показать или скрыть карточку сведений | `i` |
| `file_insight` | `blame_next` | Blame: Следующий hunk | `n` |
| `file_insight` | `blame_prev` | Blame: Предыдущий hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: Следующий hunk того же commit'а | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Предыдущий hunk того же commit'а | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Перейти к строке | `g` |
| `file_insight` | `blame_copy_sha` | Blame: Копировать SHA commit'а | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Показать diff commit'а для файла | `d` |
| `file_insight` | `blame_parent` | Blame: Blame на родителе commit'а | `b` |
| `file_insight` | `blame_history` | Blame: Открыть историю файла | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Сделать колонку аннотаций шире | `>` |
| `file_insight` | `blame_narrower` | Blame: Сделать колонку аннотаций уже | `<` |
| `image` | `zoom_in` | Увеличить изображение | `+`, `=` |
| `image` | `zoom_out` | Уменьшить изображение | `-` |
| `image` | `fit` | Вписать изображение в окно | `0` |
| `image` | `actual_size` | Показать изображение в реальном размере | `1` |
| `image` | `flip_side` | Показать другую сторону (до / после) | `Tab` |
| `image` | `widen_before` | Расширить панель «До» | `Alt+Right` |
| `image` | `widen_after` | Расширить панель «После» | `Alt+Left` |
| `image` | `reset_split` | Сбросить разделение до / после | `Alt+0` |
| `image` | `edit_in` | Открыть с помощью... | `Shift+O` |
| `image` | `edit_default` | Открыть в приложении по умолчанию | `o` |
| `editor` | `save` | Сохранить | `Ctrl+S` |
| `editor` | `undo` | Отменить | `Ctrl+Z` |
| `editor` | `redo` | Повторить | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Найти | `Ctrl+F` |
| `editor` | `find_next` | Найти далее (Shift: назад) | `F3` |
| `editor` | `paste` | Вставить | `Ctrl+V` |
| `editor` | `toggle_minimap` | Переключить миникарту | `Alt+M` |
| `editor_prompt` | `save` | Несохранённые изменения: сохранить и закрыть | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Несохранённые изменения: отбросить и закрыть | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Несохранённые изменения: продолжить правку | `c`, `Shift+C` |
| `merge` | `next_conflict` | Следующий конфликт | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Предыдущий конфликт | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Следующий файл с конфликтом | `Alt+PageDown` |
| `merge` | `prev_file` | Предыдущий файл с конфликтом | `Alt+PageUp` |
| `merge` | `next_pane` | Фокус на следующей панели | `F6` |
| `merge` | `prev_pane` | Фокус на предыдущей панели | `Shift+F6` |
| `merge` | `toggle_panes` | Показать или скрыть боковые панели (Refs и Status) | `F9` |
| `merge` | `toggle_base` | Показать или скрыть панель «База» | `Alt+D` |
| `merge` | `toggle_full_file` | Переключить diff по всему файлу | `Alt+F` |
| `merge` | `undo` | Отмена (выбор и ввод) | `Ctrl+Z` |
| `merge` | `redo` | Повтор (выбор и ввод) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Расширить панель «Текущее» (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Расширить панель «Входящее» (B) | `Alt+Left` |
| `merge` | `reset_panes` | Сбросить размеры панелей | `Alt+0` |
| `merge` | `save` | Сохранить без пометки «разрешено» | `Alt+S` |
| `merge` | `save_resolve` | Сохранить и пометить разрешённым | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Предложить решение (ИИ) | `Alt+I` |
| `merge` | `unpick` | Снять выбор с конфликта (вернуть как неразрешённый) | `Alt+U` |
| `merge` | `reset_all` | Сбросить весь файл к исходным конфликтам | `Alt+R` |
| `merge` | `focus_a` | Фокус на панели «Текущее» (A) | `Alt+1` |
| `merge` | `focus_b` | Фокус на панели «Входящее» (B) | `Alt+2` |
| `merge` | `focus_output` | Фокус на панели результата | `Alt+3` |
| `merge` | `focus_base` | Фокус на панели «База» | `Alt+4` |
| `merge_sides` | `toggle_line` | Выбрать или убрать строку | `Space` |
| `merge_sides` | `toggle_chunk` | Выбрать или убрать весь блок | `Enter` |
| `merge_sides` | `take_a` | Только «Текущее» (A) | `a` |
| `merge_sides` | `take_b` | Только «Входящее» (B) | `b` |
| `merge_sides` | `take_both` | Обе стороны: A, затем B | `c` |
| `merge_sides` | `take_both_reversed` | Обе стороны: B, затем A | `Shift+C` |
| `merge_sides` | `clear` | Ничего не брать для этого конфликта | `x` |
| `merge_sides` | `unpick` | Снять выбор с конфликта (вернуть как неразрешённый) | `u` |
| `merge_sides` | `reset_all` | Сбросить весь файл к исходным конфликтам | `Shift+U` |
| `merge_sides` | `swap_order` | Поменять порядок двух сторон | `s` |
| `merge_sides` | `next_conflict` | Следующий конфликт | `n` |
| `merge_sides` | `prev_conflict` | Предыдущий конфликт | `Shift+N` |
| `merge_sides` | `take_all_a` | Взять всё из панели «Текущее» (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Взять всё из панели «Входящее» (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Показать или скрыть панель «База» | `d` |
| `merge_sides` | `scroll_left` | Прокрутить панели влево | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Прокрутить панели вправо | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Переключить diff по всему файлу | `f` |
| `merge_sides` | `cycle_height` | Сменить высоту результата | `z` |
| `merge_card` | `take_a` | Оставить A («Текущее») как есть | `a` |
| `merge_card` | `take_b` | Взять B («Входящее») как есть | `b` |
| `merge_card` | `take_base` | Взять базу (общего предка) | `o` |
| `merge_card` | `keep` | Оставить имеющийся файл | `k` |
| `merge_card` | `delete` | Принять удаление | `Shift+D` |
| `merge_card` | `external_tool` | Открыть во внешнем инструменте merge | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (остановка для amend) | `e` |
| `rebase` | `autosquash` | Переключить autosquash | `a` |
| `rebase` | `start` | Запустить план | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Сбросить план | `Ctrl+R` |
| `rebase` | `split` | Разделить commit строки | `t` |
| `rebase` | `move_up` | Переместить строку вверх | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Переместить строку вниз | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Создать commits | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Переключить выбор по строкам | `v` |
| `split` | `move_up` | Переместить commit вверх | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Переместить commit вниз | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Сбросить разделение | `Shift+R` |
| `rewrite` | `preview` | Предпросмотр переписывания | `F5` |
| `rewrite` | `apply` | Применить переписывание | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Отменить предпросмотр | `Ctrl+X` |
| `rewrite` | `add_rule` | Добавить правило | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Включить или выключить правило | `Space` |
| `rewrite` | `move_up` | Поднять правило | `Alt+Up` |
| `rewrite` | `move_down` | Опустить правило | `Alt+Down` |
| `rewrite` | `delete_rule` | Удалить правило | `d`, `Delete` |
| `rewrite` | `restore_rule` | Вернуть удалённое правило | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Дублировать правило | `Ctrl+D` |
| `rewrite` | `next_tab` | Следующая вкладка предпросмотра | `Alt+Right` |
| `rewrite` | `prev_tab` | Предыдущая вкладка предпросмотра | `Alt+Left` |
| `rewrite` | `save_shared` | Сохранить рецепт в репозитории | `Alt+S` |
| `rewrite` | `save_mine` | Сохранить рецепт только для меня | `Alt+M` |
| `rewrite` | `load` | Загрузить рецепт | `Alt+L` |
| `rewrite` | `name` | Переименовать рецепт | `F2` |
| `rewrite` | `protected_all` | Включить все защищённые ветки | `Alt+A` |
| `rewrite` | `protected_none` | Исключить все защищённые ветки | `Alt+U` |
| `rewrite` | `check_tools` | Проверить инструменты снова | `F6` |
| `rewrite` | `analyze` | Анализ размеров истории | `F7` |
| `rewrite` | `host_howto` | Объяснить защиту ветки на хостинге | `Alt+H` |
| `rewrite` | `ai_rules` | Создать правила переписывания с помощью ИИ | `Alt+I` |
| `reword` | `apply` | Применить новое сообщение | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Изменить историю глубже | `Alt+H` |
| `composer` | `commit` | Commit из композитора | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Вставить ссылку на задачу | `Alt+R` |
| `dialog` | `browse` | Выбрать папку (Обзор) | `Ctrl+B` |
| `dialog` | `sources` | Перейти к источникам клонирования | `Alt+S` |
| `dialog` | `submit` | Отправить форму pull request | `Ctrl+S` |
| `dialog` | `web` | Открыть веб-форму провайдера | `Alt+W` |
| `folder_picker` | `choose` | Выбрать папку | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Новая папка | `Alt+N` |
| `folder_picker` | `filter` | Фильтр записей папки | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Показать или скрыть соседние папки | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Показать или скрыть скрытые папки | `Alt+H` |
| `preferences` | `search` | Поиск по настройкам | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Добавить элемент списка | `Insert` |
| `preferences` | `list_duplicate` | Дублировать выбранный элемент списка | `Ctrl+D` |
| `preferences` | `list_remove` | Удалить выбранный элемент списка | `Delete` |
| `preferences` | `list_move_up` | Переместить выбранный элемент списка вверх | `Alt+Up` |
| `preferences` | `list_move_down` | Переместить выбранный элемент списка вниз | `Alt+Down` |
| `preferences` | `prev_tab` | Предыдущая вкладка страницы настроек | `Alt+Left` |
| `preferences` | `next_tab` | Следующая вкладка страницы настроек | `Alt+Right` |
| `ai_popup` | `copy` | Копировать результат | `c` |
| `activity_log` | `scroll_left` | Прокрутить журнал активности влево | `Left` |
| `activity_log` | `scroll_right` | Прокрутить журнал активности вправо | `Right` |
| `integrations` | `add` | Добавить профиль | `a` |
| `integrations` | `edit` | Изменить выбранный профиль | `e` |
| `integrations` | `open_token` | Открыть страницу токена | `o` |
| `integrations` | `remove` | Удалить выбранный профиль | `d`, `Delete` |
| `prompt` | `ai_stash_message` | ИИ: написать сообщение stash'а | `Ctrl+G` |
<!-- keymap:end -->

### Клавиши, которые терминал не может отправить, и клавиатурный протокол kitty

Устаревший терминал не может отправить некоторые сочетания: `Ctrl+,`,
`Ctrl+Shift+N`, `Ctrl+Enter` и `Ctrl+<цифра>`. Он не отправляет ничего или
отправляет другую клавишу: например, `Ctrl+Shift+N` приходит как `Ctrl+N`,
`Ctrl+I` как `Tab`, а `Ctrl+M` как `Enter`. Страница Настроек предупреждает о
таком назначении («этот терминал не отправляет ...»).

Терминалы с **клавиатурным протоколом kitty** могут отправлять эти сочетания.
Примеры: kitty, WezTerm, Ghostty, foot, Alacritty 0.13 и новее, а также tmux или
herdr, когда они пропускают протокол. GitPoulpi по умолчанию использует
протокол:

- При запуске GitPoulpi спрашивает терминал и ждёт ответа не дольше около
  100 мс. Если терминал не отвечает, остаются устаревшие клавиши.
- Если терминал отвечает, GitPoulpi включает протокол. Он снова выключает его
  при выходе, ошибке или сбое.
- Предупреждения на странице Настроек следуют за протоколом: когда он включён,
  перечисленные выше сочетания отправляются.
- **Настройки > Клавиатура и мышь > Клавиатурный протокол Kitty**
  (`ui.kitty_keyboard`), установленный в `off`, никогда не меняет клавиатурный
  режим терминала. Используйте его, если терминал или мультиплексор ведёт себя
  неправильно. Нужен перезапуск.

Консоли Windows сами сообщают модификаторы и в протоколе не нуждаются.

## Палитра команд (`Ctrl+P`)

Палитра перечисляет **все** команды приложения, так что через неё доступно всё
без сочетаний клавиш. Полный список [ниже](#все-команды-палитры).

| Клавиша | Действие |
|---|---|
| `Ctrl+P` | Открыть палитру |
| *(ввод)* | Фильтровать список. Точное совпадение после префикса `Категория: ` идёт первым, затем совпадение в любом месте названия, затем нечёткое совпадение |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J`, `Home` / `End`, `PageUp` / `PageDown` | Переместить подсветку (см. [Списки во всплывающих окнах с фильтром](#списки-во-всплывающих-окнах-с-фильтром)) |
| `Enter` или двойной щелчок | Выполнить подсвеченную команду |
| `Esc` | Закрыть |

`j` и `k` вводятся в фильтр. Они не двигают подсветку.

В каждой строке справа показано её сочетание. Команда, которая сейчас не
применима (например, не выбрана ссылка), остаётся в списке серой, а причина
показана под списком. Названия начинаются с категории, например `Навигация:`,
`Refs:`, `Git:`, `Вид:`, `Diff:` или `Приложение:`. Введите категорию, чтобы
увидеть всю её группу.

### Клавиша для любой команды

Каждая строка палитры заканчивается кнопкой клавиши (`[k]`), и `Alt+K` делает то же для выделенной строки: открывает **Настройки > Клавиатура и мышь**, отфильтрованные по сочетанию этой команды, готовые к переназначению. Команда без собственного сочетания получает его здесь: она указана в разделе **Команды палитры**, по умолчанию без клавиши, а назначенная вами клавиша запускает команду так же, как её выбор в палитре (команда, которую сейчас нельзя выполнить, сообщает причину). После этого строка показывает вашу клавишу.

В `config.toml` это таблица `[keybindings.palette]`, названная по команде (`toggle_minimap = "Ctrl+Alt+M"`). Клавиша, конфликтующая с другим действием, показана красным, а `c` на её строке перечисляет все действия, с которыми она конфликтует.

Строка подсказки под списком называет эту клавишу (по умолчанию `Alt+K`). Это действие `palette.configure_shortcut`, поэтому его можно переназначить, как любое другое.

### Все команды палитры

Сгенерировано из приложения (тест поддерживает актуальность). Команды ИИ
появляются только при включённом ИИ; остальные серые, с указанием причины, когда
неприменимы. Клавиши показаны по умолчанию.

У Настроек есть собственные строки, которых здесь нет: по одной строке `Настройки: переключить <параметр>` на каждый параметр «вкл/выкл», `Настройки: <параметр>...` на каждый параметр с выбором или полем и `Настройки: <параметр>: <вариант>` на каждый вариант. Они генерируются из экрана Настроек, поэтому список всегда совпадает с ним; введите `настройки` в палитре, чтобы увидеть их.

<!-- palette:begin -->
| Команда | Клавиша по умолчанию |
|---|---|
| Навигация: фокус на панели Refs | `Ctrl+N <badge>` |
| Навигация: фокус на панели Log | `Ctrl+N <badge>` |
| Навигация: фокус на панели Status | `Ctrl+N <badge>` |
| Навигация: фокус на фильтре Refs | `Ctrl+N <badge>` |
| Навигация: фокус на Refs > Local | `Ctrl+N <badge>` |
| Навигация: фокус на Refs > Remote | `Ctrl+N <badge>` |
| Навигация: фокус на Refs > Worktrees | `Ctrl+N <badge>` |
| Навигация: фокус на Refs > Подмодули | `Ctrl+N <badge>` |
| Навигация: фокус на Refs > Stashes | `Ctrl+N <badge>` |
| Навигация: фокус на Refs > Tags | `Ctrl+N <badge>` |
| Левая панель: показать/скрыть разделы... |  |
| Левая панель: вернуть разделы по умолчанию |  |
| Левая панель: использовать эти разделы для всех репозиториев |  |
| Левая панель: показать/скрыть локальные ветки |  |
| Левая панель: показать/скрыть Remote |  |
| Левая панель: показать/скрыть Worktrees |  |
| Левая панель: показать/скрыть Stashes |  |
| Левая панель: показать/скрыть Tags |  |
| Левая панель: показать/скрыть Pull requests |  |
| Левая панель: показать/скрыть задачи |  |
| Левая панель: показать/скрыть подмодули |  |
| Навигация: фокус на Status > Unstaged | `Ctrl+N <badge>` |
| Навигация: фокус на Status > Staged | `Ctrl+N <badge>` |
| Навигация: фокус на следующую панель | `Tab` |
| Навигация: фокус на предыдущую панель | `Shift+Tab` |
| Навигация: поиск commit'ов... | `f` |
| Навигация: свернуть/развернуть панель Refs | `Ctrl+N h` |
| Навигация: расширить панель Refs | `Ctrl+N Right` |
| Навигация: сузить панель Refs | `Ctrl+N Left` |
| Навигация: расширить панель Status | `Ctrl+N Shift+Left` |
| Навигация: сузить панель Status | `Ctrl+N Shift+Right` |
| Навигация: свернуть/развернуть панель Status | `Ctrl+N l` |
| Навигация: увеличить раздел Refs | `Ctrl+N +` |
| Навигация: уменьшить раздел Refs | `Ctrl+N -` |
| Навигация: сбросить размеры разделов Refs |  |
| Навигация: сбросить раскладку |  |
| Детали commit'а: увеличить сообщение | `Ctrl+N ]` |
| Детали commit'а: уменьшить сообщение | `Ctrl+N [` |
| Детали commit'а: сбросить размер сообщения |  |
| Детали commit'а: прокрутить сообщение вниз |  |
| Детали commit'а: прокрутить сообщение вверх |  |
| Файлы: порядок сортировки файлов commit'а (А-Я / Я-А) |  |
| Refs: checkout выбранной ссылки | `двойной щелчок` |
| Refs: копировать имя ссылки |  |
| Refs: копировать полную ссылку (refs/heads/...) |  |
| Refs: копировать SHA цели ссылки |  |
| Refs: сделать merge выбранной ветки в текущую |  |
| Refs: rebase текущей ветки на выбранную | `Ctrl+G r` |
| Refs: перетащить выбранную ссылку на текущую ветку... |  |
| Refs: пометить как источник перетаскивания | `m` |
| Refs: перетащить сюда помеченную ссылку... | `M` |
| Refs: переименовать выбранную ветку... |  |
| Refs: fast-forward выбранной ветки до текущей |  |
| Refs: fast-forward текущей ветки до выбранной |  |
| Refs: создать worktree из выбранной ветки или tag'а... |  |
| Refs: Reflog выбранной ветки... |  |
| Refs: создать pull request из выбранной ветки... |  |
| Refs: открыть worktree во вкладке | `Enter / двойной щелчок` |
| Refs: обновить подмодуль (выбранный) |  |
| Refs: инициализировать подмодуль (выбранный) |  |
| Refs: открыть подмодуль во вкладке | `Enter / двойной щелчок` |
| Git: обновить все подмодули |  |
| Git: создать worktree в выбранном commit'е... |  |
| Git: создать патч из выбранного commit'а... |  |
| Git: создать патч от выбранного commit'а до HEAD... |  |
| Git: создать патч из изменений рабочего дерева... |  |
| Переписывание: очистить старую историю... |  |
| Git: применить патч... |  |
| Переписывание: окно истории | `Ctrl+G H` |
| Переписывание: заменить автора... |  |
| Переписывание: анонимизировать авторов... |  |
| Переписывание: применить .mailmap к истории... |  |
| Переписывание: merge авторов (.mailmap)... |  |
| Переписывание: сдвинуть даты commit'ов... |  |
| Переписывание: сменить часовой пояс commit'ов... |  |
| Переписывание: оставить время commit'ов в пределах часов... |  |
| Переписывание: дата коммитера = дата автора |  |
| Переписывание: найти и заменить в сообщениях... |  |
| Переписывание: удалить строки сообщений... |  |
| Переписывание: добавить или убрать трейлеры... |  |
| Переписывание: убрать подписи... |  |
| Переписывание: исправить кодировку сообщений... |  |
| Переписывание: найти и заменить в файлах... |  |
| Переписывание: убрать файлы из истории... |  |
| Переписывание: убрать большие файлы из истории... |  |
| Переписывание: переместить или переименовать пути... |  |
| Переписывание: перенести файлы в LFS... |  |
| Переписывание: вернуть файлы из LFS... |  |
| Переписывание: преобразовать окончания строк... |  |
| Переписывание: убрать бит исполняемости... |  |
| Переписывание: преобразовать кодировку файлов... |  |
| Переписывание: запустить скрипт... |  |
| ✦ Переписывание: написать скрипт с помощью ИИ... |  |
| ✦ Переписывание: создать правила с помощью ИИ... |  |
| Переписывание: от выбранного commit'а до HEAD... |  |
| Переписывание: выбранные commits... |  |
| LFS: инициализировать для этого репозитория |  |
| LFS: отслеживать шаблон... |  |
| LFS: не отслеживать шаблон... |  |
| LFS: fetch объектов |  |
| LFS: открыть меню | `Ctrl+G l` |
| LFS: fetch всех объектов | `Ctrl+G F` |
| LFS: checkout всех файлов | `Ctrl+G K` |
| LFS: push всех объектов | `Ctrl+G J` |
| LFS: pull объектов | `Ctrl+G L` |
| LFS: push объектов |  |
| LFS: очистить локальные объекты | `Ctrl+G V` |
| LFS: показать блокировки |  |
| LFS: открыть страницу установки |  |
| LFS: сырой указатель в diff | `r` |
| LFS: отслеживать выбранный файл |  |
| LFS: отслеживать расширение выбранного файла |  |
| LFS: заблокировать выбранный файл |  |
| LFS: разблокировать выбранный файл |  |
| LFS: принудительно разблокировать файл |  |
| Refs: удалить выбранную ветку или tag... | `Ctrl+G d` |
| Refs: удалить выбранную ветку с её remote... |  |
| Refs: удалить ветку локально и на remote... |  |
| Refs: копировать ссылку на ветку на remote |  |
| Git: копировать ссылку на commit на remote |  |
| Git: аннотированный tag в выбранном commit'е... |  |
| Refs: push tag на remote по умолчанию |  |
| Refs: удалить выбранный tag с его remote... |  |
| Git: Fetch выбранный tag |  |
| Refs: аннотировать выбранный tag... |  |
| Refs: скрыть/показать в графе |  |
| Refs: показать в графе | `g` |
| Refs: solo для выбранной ссылки | `o` |
| Refs: снять solo (со всех ссылок) | `O` |
| Refs: показать все скрытые ссылки | `H` |
| Refs: скрыть/показать папку или remote |  |
| Refs: solo для папки или remote | `o` |
| Refs: фильтр по имени... | `/` |
| Refs: сбросить фильтр по имени | `Esc` |
| Refs: развернуть все папки | `=` |
| Refs: свернуть все папки | `-` |
| Refs: развернуть выбранную папку и её вложенные папки | `*` |
| Refs: свернуть выбранную папку и её вложенные папки | `_` |
| Refs: следующий раздел | `l` |
| Refs: предыдущий раздел | `h` |
| Refs: развернуть/свернуть активный раздел | `Enter` |
| Refs: иерархия веток | `y` |
| Git: stage выбранный файл | `Ctrl+G s` |
| Git: unstage выбранный файл | `Ctrl+G u` |
| Git: stage все изменения | `Ctrl+G a` |
| Изменения: отбросить выбранный файл... |  |
| Изменения: игнорировать выбранный файл |  |
| Изменения: игнорировать расширение файла |  |
| Изменения: считать выбранный файл двоичным... |  |
| Изменения: считать выбранный файл текстовым... |  |
| Изменения: считать расширение файла двоичным... |  |
| Изменения: считать расширение файла текстовым... |  |
| Изменения: stash выбранного файла |  |
| Изменения: история выбранного файла |  |
| Изменения: blame выбранного файла |  |
| Изменения: открыть файл во внешнем diff-инструменте |  |
| Изменения: открыть файл во внешнем редакторе |  |
| Изменения: открыть файл в программе по умолчанию |  |
| Изменения: показать файл в папке |  |
| Изменения: копировать путь файла |  |
| Изменения: создать патч из файла... |  |
| Изменения: править выбранный файл |  |
| Изменения: удалить выбранный файл... |  |
| Изменения: меню выбранной папки |  |
| Изменения: создать файл |  |
| Изменения: отбросить все изменения... | `Ctrl+G x` |
| Git: commit... | `Ctrl+G c` |
| Git: сразу сделать commit staged-изменений | `Ctrl+G Enter` |
| Git: изменить commit-сообщение WIP (в Log) | `c в Log` |
| Git: fetch всех remote | `Ctrl+G f` |
| Git: pull | `Ctrl+G p` |
| Git: pull (fast-forward, если возможно) |  |
| Git: pull (только fast-forward) |  |
| Git: pull (rebase) |  |
| Git: параметры pull... |  |
| Git: push | `Ctrl+G P` |
| Git: push на remote... |  |
| Git: force push (с lease)... |  |
| Git: задать upstream... |  |
| Remote: добавить remote... |  |
| Remote: удалить remote... |  |
| Remote: изменить remote... |  |
| Remote: задать remote по умолчанию... |  |
| Remote: копировать имя remote |  |
| Remote: копировать ссылку на remote |  |
| Remote: fetch remote... |  |
| Remote: действия для выбранного remote... | `.` |
| Репозиторий: Частичный checkout... |  |
| Репозиторий: SSH и учётные данные... |  |
| Репозиторий: проверить подключение |  |
| Репозиторий: учётные данные по умолчанию |  |
| Flow: открыть меню | `Ctrl+G W` |
| Flow: начать ветку... |  |
| Flow: новая задача... |  |
| Flow: подхватить ветку хоста... |  |
| Flow: завершить текущую ветку... |  |
| Flow: синхронизировать с родителем... |  |
| Flow: продвинуть... |  |
| Flow: задать родителя... |  |
| Flow: продолжить или прервать остановленное действие... |  |
| Flow: переключить группировку веток |  |
| Flow: настройки рабочего процесса... |  |
| Remote: скрыть remote в графе... |  |
| Remote: показать remote в графе... |  |
| Git: новая ветка в HEAD... | `Ctrl+G b` |
| Git: новый tag в HEAD... | `Ctrl+G t` |
| Git: push всех неотправленных tag'ов | `Ctrl+G U` |
| Git: stash всех изменений | `Ctrl+G T` |
| Git: stash всех изменений с сообщением... | `Ctrl+G m` |
| Git: stash pop | `Ctrl+G o` |
| Git: stash apply |  |
| Git: stash drop... |  |
| Git: cherry-pick выбранного commit'а на HEAD | `Ctrl+G y` |
| Git: checkout выбранного commit'а (detached HEAD) |  |
| Git: создать ветку в выбранном commit'е... |  |
| Git: создать tag в выбранном commit'е... |  |
| Commit: копировать сообщение |  |
| Commit: копировать имя автора |  |
| Commit: копировать email автора |  |
| Commit: копировать дату |  |
| Commit: копировать короткий SHA |  |
| Commit: копировать полный SHA |  |
| Git: revert выбранного commit'а... | `Ctrl+G v` |
| Git: изменить сообщение commit'а... | `Ctrl+G e` |
| Commit: изменить... | `Ctrl+G E` |
| Commit: изменить автора... | `Ctrl+G A` |
| Commit: изменить заголовок... |  |
| Commit: изменить описание... |  |
| Git: разделить выбранный commit... |  |
| Git: интерактивный rebase от выбранного commit'а... |  |
| Rebase: продолжить |  |
| Rebase: пропустить |  |
| Rebase: прервать |  |
| git am: Продолжить |  |
| git am: Пропустить этот патч |  |
| git am: Прервать |  |
| Репозиторий: удалить устаревший index.lock |  |
| Баннер: pop ожидающего auto-stash'а |  |
| Баннер: завершить ожидающий auto-stash (оставить результат) |  |
| Баннер: показать ожидающий auto-stash |  |
| Конфликты: показать предсказанные конфликты |  |
| Баннер: drop ожидающего auto-stash'а... |  |
| Баннер: начать заново... |  |
| Git: reset ветки к выбранному commit'у (soft - сохранить индекс и дерево) |  |
| Git: reset ветки к выбранному commit'у (mixed - сохранить дерево) | `Ctrl+G R` |
| Git: reset ветки к выбранному commit'у (hard - отбросить всё)... |  |
| Git: разрешить конфликт через Ours (эта ветка) |  |
| Git: разрешить конфликт через Theirs (входящее) |  |
| Git: разрешить конфликт через базу (общий предок) |  |
| Конфликты: открыть редактор merge | `Enter на файле с конфликтом` |
| Merge: следующий конфликт | `F7` |
| Merge: предыдущий конфликт | `Shift+F7` |
| Merge: следующий файл с конфликтом | `Alt+PageDown` |
| Merge: предыдущий файл с конфликтом | `Alt+PageUp` |
| Merge: взять блок из A |  |
| Merge: взять блок из B |  |
| Merge: взять «Текущее» для этого конфликта | `a` |
| Merge: взять «Входящее» для этого конфликта | `b` |
| Merge: взять обе стороны, A затем B | `c` |
| Merge: взять обе стороны, B затем A | `C` |
| Merge: взять базу для этого конфликта |  |
| Merge: очистить этот конфликт | `x` |
| Merge: поменять порядок | `s` |
| Merge: взять всё из «Текущее» | `A` |
| Merge: взять всё из «Входящее» | `B` |
| Merge: править конфликт в результате |  |
| Merge: показать/скрыть боковые панели | `F9` |
| Merge: прокрутить панели влево | `Shift+Left` |
| Merge: прокрутить панели вправо | `Shift+Right` |
| Merge: показать/скрыть панель «База» | `Alt+D` |
| Merge: diff по всему файлу | `Alt+F` |
| Merge: отмена | `Ctrl+Z` |
| Merge: повтор | `Ctrl+Y` |
| Merge: сохранить без пометки «разрешено» | `Alt+S` |
| Merge: сохранить и пометить разрешённым | `Ctrl+S` |
| Merge: сбросить результат к итогу merge | `Alt+R` |
| Merge: сменить высоту результата | `z` |
| Merge: расширить панель «Текущее» | `Alt+Right` |
| Merge: расширить панель «Входящее» | `Alt+Left` |
| Merge: сбросить размеры панелей | `Alt+0` |
| Merge: снять выбор с конфликта | `Alt+U` |
| Merge: фокус на панели «Текущее» (A) | `Alt+1` |
| Merge: фокус на панели «Входящее» (B) | `Alt+2` |
| Merge: фокус на панели результата | `Alt+3` |
| Merge: фокус на панели «База» | `Alt+4` |
| Merge: к левой панели (тот же конфликт) |  |
| Merge: к правой панели (тот же конфликт) |  |
| Merge: фокус на следующую панель | `F6` |
| Merge: фокус на предыдущую панель | `Shift+F6` |
| Merge: продолжить с файла на диске |  |
| Merge: начать заново с merge |  |
| Merge: отменить разрешение и открыть заново |  |
| Merge: перезагрузить файл |  |
| Merge: взять файл «Текущее» как есть | `a` |
| Merge: взять файл «Входящее» как есть | `b` |
| Merge: взять файл «База» как есть | `o` |
| Merge: оставить имеющийся файл | `k` |
| Merge: принять удаление | `D` |
| Merge: открыть во внешнем инструменте | `m` |
| ✦ ИИ: предложить решение конфликта merge | `Alt+I` |
| Merge: закрыть редактор merge |  |
| Git: прервать merge, cherry-pick или revert |  |
| Git: пропустить cherry-pick или revert |  |
| Репозиторий: открыть или переключить (управление)... | `Ctrl+O` |
| Репозиторий: найти репозиторий (Обзор)... | `b (вкладка управления)` |
| Репозиторий: создать новый (Init)... | `i (вкладка управления)` |
| Репозитории: открыть выбранный | `Enter` |
| Репозитории: в избранное или из избранного | `f` |
| Репозитории: закрыть или убрать выбранный | `x` |
| Репозитории: меню выбранного репозитория... | `.` |
| Репозитории: поиск | `/ / Ctrl+F` |
| Репозитории: свернуть все разделы | `-` |
| Репозитории: развернуть все разделы | `+` |
| Репозитории: закрыть все вкладки... |  |
| Репозитории: убрать выбранный из списка |  |
| Репозитории: убрать все недавние |  |
| Рабочее пространство: новое... | `n` |
| Рабочее пространство: новое из открытых вкладок... | `a` |
| Рабочее пространство: сменить... | `Alt+W` |
| Рабочее пространство: изменить... | `e` |
| Рабочее пространство: закрыть (без рабочего пространства) |  |
| Рабочее пространство: управление... | `Alt+J` |
| Рабочее пространство: закрыть вкладку |  |
| Рабочее пространство: найти отсутствующий репозиторий... | `l` |
| Рабочее пространство: удалить... |  |
| Рабочее пространство: fetch все | `F` |
| Рабочее пространство: pull все | `P` |
| Рабочее пространство: клонировать отсутствующие | `m` |
| Рабочее пространство: добавить вкладку в рабочее пространство |  |
| Рабочее пространство: убрать вкладку из рабочего пространства |  |
| Radar: открыть | `Alt+R` |
| Radar: обновить | `r` |
| Radar: поиск | `/` |
| Radar: открыть в браузере | `o` |
| Radar: открыть репозиторий | `Enter` |
| Radar: переключиться на ветку | `c` |
| Radar: создать worktree... | `t` |
| Radar: показать в репозитории | `s` |
| Radar: начать работу над задачей... | `n` |
| Radar: закрепить или открепить | `p` |
| Radar: отложить... | `z` |
| Radar: ревью запроса... | `e` |
| Radar: ревью через API |  |
| Radar: клонировать и сделать ревью... |  |
| Radar: изменить исполнителей... |  |
| Radar: изменить рецензентов... |  |
| Radar: изменить метки... |  |
| Radar: задать этап... |  |
| Radar: выбрать область | `w` |
| Radar: выбрать вид | `v` |
| Radar: выбрать фильтр | `f` |
| Radar: показать запросы неклонированных репозиториев | `a` |
| Radar: показать черновики | `d` |
| Radar: включить запросы на ревью по всему аккаунту... | `T` |
| Radar: меню строки... | `.` |
| Radar: выбрать колонку | `C` |
| Radar: расширить колонку | `]` |
| Radar: сузить колонку | `[` |
| Radar: сбросить ширину колонок | `0` |
| Radar: назад на главную | `Esc` |
| Radar: закрыть вкладку |  |
| Вкладка: новая вкладка | `Alt+T` |
| Вкладка: новая... |  |
| Вкладка: меню действий... | `Ctrl+T` |
| Вкладка: закрыть остальные... |  |
| Вкладка: закрыть вкладки справа... |  |
| Вкладка: в избранное или из избранного |  |
| Вкладка: копировать путь репозитория |  |
| Вкладка: открыть в файловом менеджере |  |
| Вкладка: открыть в терминале |  |
| Вкладка: открыть в... | `Alt+O` |
| Вкладка: открыть в записи по умолчанию | `Ctrl+Alt+O` |
| Репозиторий: сменить профиль... |  |
| Справка: о программе GitPoulpi | `Ctrl+N a` |
| Справка: лицензии и благодарности |  |
| Вид: Журнал активности | `Ctrl+N A` |
| Вид: Общий журнал активности | `Ctrl+N G` |
| Справка: заметки о выпуске |  |
| Обновления: показать обновление | `Ctrl+N u` |
| Обновления: открыть уведомление о безопасности |  |
| Обновления: копировать команду обновления |  |
| Обновления: выполнить команду обновления в терминале |  |
| Обновления: проверить наличие сейчас |  |
| Обновления: скачать |  |
| Обновления: обновить сейчас |  |
| Обновления: перезапустить для обновления |  |
| Обновления: вернуться к предыдущей версии |  |
| Обновления: пропустить эту версию |  |
| Заметки о выпуске: закрыть вкладку |  |
| Задачи: показать фоновые задачи | `Ctrl+N J` |
| Задачи: отменить всё, что можно отменить |  |
| Вид: Копировать диагностику |  |
| Журнал активности: показать или скрыть уровень Ошибка |  |
| Журнал активности: показать или скрыть уровень Предупреждение |  |
| Журнал активности: показать или скрыть уровень Инфо |  |
| Журнал активности: показать или скрыть уровень Отладка |  |
| Журнал активности: показать или скрыть уровень Трассировка |  |
| Журнал активности: показать все уровни |  |
| Журнал активности: не показывать ни один уровень |  |
| Журнал активности: очистить журнал приложения |  |
| Журнал активности: выбрать область |  |
| Журнал активности: операции git |  |
| Журнал активности: журнал приложения |  |
| Журнал активности: прокрутить влево | `Left` |
| Журнал активности: прокрутить вправо | `Right` |
| Журнал активности: копировать последнюю ошибку |  |
| Журнал активности: выполнить предложенное действие |  |
| Журнал активности: открыть последнюю ошибку |  |
| Вкладка: выйти и cd сюда |  |
| Репозиторий: клонировать... | `Ctrl+Shift+N` |
| Репозиторий: следующая вкладка | `Ctrl+PageDown` |
| Репозиторий: предыдущая вкладка | `Ctrl+PageUp` |
| Вкладка: переместить влево |  |
| Вкладка: переместить вправо |  |
| Репозиторий: закрыть вкладку | `Ctrl+W` |
| Репозиторий: выбрать вкладку... | `t` |
| Поиск: рабочее пространство... | `Ctrl+Alt+F` |
| Правка: отмена | `Ctrl+Z` |
| Правка: повтор | `Ctrl+Y` |
| Правка: забыть заблокированную запись отмены |  |
| Правка: история отмен... | `Alt+Z` |
| Правка: Reflog... |  |
| Правка: найти потерянные commit'ы... |  |
| Правка: восстановить ветку туда, где её оставило заблокированное действие | `Alt+Shift+Z` |
| Ревью: показать обзор |  |
| Ревью: показать файлы |  |
| Ревью: показать commits |  |
| Ревью: комментарий к строке... | `c` |
| Ревью: комментарий к беседе... | `C` |
| Ревью: ответить в обсуждении... | `r` |
| Ревью: разрешить обсуждение или открыть снова | `x` |
| Ревью: свернуть или развернуть обсуждение | `z` |
| Ревью: изменить неотправленный комментарий... | `e` |
| Ревью: удалить неотправленный комментарий | `d` |
| Ревью: отметить файл как просмотренный | `v` |
| Ревью: следующий файл | `]` |
| Ревью: предыдущий файл | `[` |
| Ревью: завершить ревью... | `f` |
| Ревью: отменить неотправленное ревью... |  |
| Ревью: обновить | `F5` |
| Ревью: открыть в браузере | `o` |
| Ревью: переключиться на ветку | `b` |
| Ревью: закрыть | `Esc` |
| Ревью: изменить исполнителей... | `a` |
| Ревью: изменить рецензентов... | `p` |
| Ревью: изменить метки... | `l` |
| Ревью: задать этап... | `m` |
| Ревью: выбрать набор патчей... | `P` |
| Ревью: сравнить наборы патчей... | `D` |
| Ревью: влить изменение... | `U` |
| Ревью: отозвать изменение... | `Q` |
| Ревью: вид «единый» или «рядом» | `V` |
| Ревью: весь файл или только hunk'и | `H` |
| Ревью: изменения только пробелов | `S` |
| Ревью: перенос строк | `W` |
| Ревью: следующий hunk | `J` |
| Ревью: предыдущий hunk | `K` |
| Ревью: выбрать кодировку... | `E` |
| Ревью: переключить миникарту | `M` |
| Ревью: прокрутить diff влево | `Left` |
| Ревью: прокрутить diff вправо | `Right` |
| Ревью: копировать выделенное или строку | `Ctrl+C` |
| Доска: перенести карточку в... | `m` |
| Доска: перенести карточку влево |  |
| Доска: перенести карточку вправо |  |
| Доска: открыть карточку | `Enter` |
| Доска: изменить исполнителей... |  |
| Доска: изменить метки... |  |
| Доска: задать этап... |  |
| Доска: новая задача в этой колонке | `n` |
| Доска: фильтр по исполнителю... | `a` |
| Доска: фильтр по метке... | `l` |
| Доска: фильтр по этапу... | `M` |
| Доска: фильтр по тексту | `/` |
| Доска: сбросить фильтры | `x` |
| Доска: задать метки колонок... | `c` |
| Доска: обновить | `r` |
| Доска: закрыть | `Esc` |
| Ревью: ✦ составить черновик ревью... | `s` |
| Ревью: принять предложение | `y` |
| Ревью: изменить и принять предложение... | `w` |
| Ревью: отклонить предложение | `n` |
| Ревью: принять все предложения | `A` |
| Ревью: отклонить все предложения | `N` |
| Ревью: взять сводку ИИ в текст ревью | `i` |
| Ревью: ✦ использовать черновик... | `u` |
| Ревью: изменить текст черновика ✦... | `t` |
| Ревью: отбросить черновик ✦ | `N` |
| История отмен: отменить до этого места... | `u` |
| История отмен: повторить до этого места... | `y` |
| История отмен: восстановить только это... | `r` |
| История отмен: восстановить ветку туда, где она была... | `o` |
| История отмен: обновить |  |
| История отмен: закрыть | `Esc` |
| Reflog: показать в Log | `s` |
| Reflog: создать ветку здесь... | `b` |
| Reflog: восстановить на эту позицию... | `r` |
| Reflog: загрузить ещё | `m` |
| Reflog: обновить |  |
| Потерянная работа: показать в Log | `s` |
| Потерянная работа: создать ветку здесь... | `b` |
| Потерянная работа: cherry-pick | `c` |
| Потерянная работа: искать заново |  |
| Вид: обновить | `r` |
| Вид: обновить (fetch, pull requests, задачи) | `F5` |
| Вид: справка | `?` |
| Вид: колонка Ref |  |
| Вид: колонка «Граф» |  |
| Вид: колонка «Сообщение» |  |
| Вид: колонка «Автор» |  |
| Вид: колонка «Дата» |  |
| Вид: колонка «Изменения» |  |
| Вид: колонка SHA |  |
| Вид: колонка Log: выбрать колонку для сдвига или изменения ширины | `C` |
| Вид: колонка Log: выбрать следующую колонку | `Tab` |
| Вид: колонка Log: выбрать предыдущую колонку | `Shift+BackTab` |
| Вид: колонка Log: сдвинуть колонку влево | `<` |
| Вид: колонка Log: сдвинуть колонку вправо | `>` |
| Вид: колонка Log: расширить колонку | `]` |
| Вид: колонка Log: сузить колонку | `[` |
| Вид: колонка Log: сбросить ширину колонок | `0` |
| Вид: колонка Log: подогнать колонку под содержимое | `=` |
| Вид: колонка Log: подогнать все колонки под содержимое | `+` |
| Вид: Log: прокрутить колонки влево | `Shift+Left` |
| Вид: Log: прокрутить колонки вправо | `Shift+Right` |
| Вид: Log: переключить прокрутку колонок вбок | `s` |
| Вид: Log: выбрать колонки для показа |  |
| Diff: открыть diff выбранного файла | `Enter` |
| Diff: закрыть diff | `Esc` |
| Diff: следующий hunk | `n` |
| Diff: предыдущий hunk | `p` |
| Diff: stage hunk под курсором | `Space` |
| Diff: unstage hunk под курсором | `Space` |
| Diff: выбрать строки (или снять выбор) | `v` |
| Diff: stage выбранные строки | `Enter` |
| Diff: unstage выбранные строки | `Enter` |
| Diff: отбросить hunk под курсором... | `x` |
| Diff: отбросить выбранные строки... | `X` |
| Копировать: выделенный текст (сначала выделите) | `Ctrl+C` |
| Копировать: строку под курсором (или выбранные) | `y` |
| Diff: копировать hunk под курсором |  |
| Diff: раскладка «единый» (+/- в одной колонке) | `m` |
| Diff: раскладка «рядом» (до и после) | `m` |
| Diff: только hunk'и (весь файл / только изменения) | `H` |
| Файлы: путь/дерево | `m` |
| Файлы: показать все файлы | `a` |
| Commit: переключить пропуск хуков | `H` |
| Файлы: открыть меню строки или выделения | `.` |
| Файлы: развернуть все папки | `=` |
| Файлы: свернуть все папки | `-` |
| Файлы: развернуть выбранную папку и её вложенные папки | `*` |
| Файлы: свернуть выбранную папку и её вложенные папки | `_` |
| Файлы: выделить всё |  |
| Файлы: снять выделение |  |
| Файлы: переключить выделение текущей строки |  |
| Log: копировать как Mermaid gitGraph... | `Y` |
| Log: Сравнить выбранное | `v` |
| Log: Поменять стороны сравнения | `x` |
| Log: к следующей сравниваемой строке | `n` |
| Diff: перенос строк | `w` |
| Diff: выбрать кодировку... | `C` |
| Вид: показать или скрыть миникарту | `M` |
| Вид: показать или скрыть колонку commit'ов | `Alt+C` |
| Вид: колонка commit'ов: скрыть |  |
| Вид: колонка commit'ов: история файла |  |
| Вид: колонка commit'ов: все commit'ы |  |
| Вид: перейти к колонке commit'ов |  |
| Вид: расширить колонку commit'ов |  |
| Вид: сузить колонку commit'ов |  |
| Вид: колонка commit'ов: прокрутить влево | `Shift+Left` |
| Вид: колонка commit'ов: прокрутить вправо | `Shift+Right` |
| Вид: колонка commit'ов: ширина следующей части | `Tab` |
| Вид: колонка commit'ов: сдвинуть часть влево | `<` |
| Вид: колонка commit'ов: сдвинуть часть вправо | `>` |
| Вид: колонка commit'ов: расширить часть | `]` |
| Вид: колонка commit'ов: сузить часть | `[` |
| Вид: колонка commit'ов: сбросить ширину частей | `0` |
| Вид: колонка commit'ов: меню выбранной строки | `.` |
| Вид: колонка commit'ов: выбрать колонки |  |
| Вид: колонка commit'ов: показать или скрыть «Ref» |  |
| Вид: колонка commit'ов: показать или скрыть «Граф» |  |
| Вид: колонка commit'ов: показать или скрыть «Автор» |  |
| Вид: колонка commit'ов: показать или скрыть «Дата» |  |
| Вид: колонка commit'ов: показать или скрыть «Измен.» |  |
| Вид: колонка commit'ов: показать или скрыть «SHA» |  |
| Вид: колонка commit'ов: показать выбранную строку в Log |  |
| Вид: колонка commit'ов: показать изменения файла в выбранной строке |  |
| Вид: колонка commit'ов: показать файл таким, каким он был в выбранной строке |  |
| Вид: колонка commit'ов: Blame файла в выбранной строке |  |
| Редактор: переключить следование за выбором |  |
| Изображение: увеличить | `+` |
| Изображение: уменьшить | `-` |
| Изображение: вписать в окно | `0` |
| Изображение: реальный размер (1:1) | `1` |
| Изображение: переключить до / после | `Tab` |
| Изображение: расширить панель «До» | `Alt+Right` |
| Изображение: расширить панель «После» | `Alt+Left` |
| Изображение: сбросить размеры панелей | `Alt+0` |
| Изображение: править в... | `O` |
| Изображение: править в редакторе по умолчанию | `o` |
| История файла: показать в Log | `Enter` |
| История файла: копировать SHA commit'а | `y` |
| История файла: копировать краткое описание | `Y` |
| История файла: показать diff этого commit'а | `d` |
| История файла: открыть файл на этом commit'е | `o` |
| История файла: Blame на этом commit'е | `b` |
| История файла: сравнить с рабочим деревом | `c` |
| История файла: восстановить эту версию | `R` |
| История файла: расширить выделение вверх | `Shift+Up` |
| История файла: расширить выделение вниз | `Shift+Down` |
| История файла: переключиться между списком и diff'ом | `Tab` |
| История файла: сделать список шире | `Alt+Right` |
| История файла: сделать список уже | `Alt+Left` |
| История файла: сбросить ширину списка | `Alt+0` |
| Файл: открыть с помощью... | `O` |
| Файл: открыть в приложении по умолчанию | `o` |
| Diff: изменения только пробелов | `s` |
| ✦ ИИ: создать сообщение commit'а | `Ctrl+G i` |
| ✦ ИИ: создать сообщение stash |  |
| ✦ ИИ: вставить переменную запроса |  |
| ✦ ИИ: изменить профиль... |  |
| ✦ ИИ: открыть агента... |  |
| ✦ ИИ: объяснить выбранный commit | `Ctrl+G w` |
| ✦ ИИ: разделить выбранный commit... |  |
| ✦ ИИ: объяснить рабочие изменения | `Ctrl+G w` |
| ✦ ИИ: составить commit'ы из рабочих изменений... | `Ctrl+G C` |
| Композитор: переместить файл в commit... | `m` |
| Diff: Blame |  |
| Blame: Показать commit в Log | `Enter` |
| Blame: Копировать SHA commit'а | `Y` |
| Blame: Показать diff commit'а для файла | `d` |
| Blame: Blame на родителе commit'а | `b` |
| Blame: Открыть историю файла | `H` |
| Blame: Показать или скрыть карточку сведений | `i` |
| Blame: Следующий hunk | `n` |
| Blame: Предыдущий hunk | `p` |
| Blame: Следующий hunk того же commit'а | `N` |
| Blame: Предыдущий hunk того же commit'а | `P` |
| Blame: Перейти к строке | `g` |
| Blame: Сделать колонку аннотаций шире | `>` |
| Blame: Сделать колонку аннотаций уже | `<` |
| Diff: история файла |  |
| Diff: просмотр файла |  |
| Файл: править в рабочей папке | `e` |
| Редактор: сменить кодировку |  |
| Редактор: открыть заново в кодировке |  |
| Редактор: преобразовать в кодировку |  |
| Редактор: преобразовать переводы строк |  |
| Редактор: изменить отступы |  |
| Приложение: настройки... | `Ctrl+, / Ctrl+N ,` |
| Приложение: поиск настроек... | `/ в настройках` |
| Приложение: сочетания клавиш... |  |
| Агент: показать ожидающий вид | `Ctrl+N Enter` |
| Агент: закрыть ожидающий вид |  |
| Агент: назад | `Ctrl+N Backspace` |
| Сочетания клавиш: сбросить все... |  |
| Интеграции: открыть связанный pull request | `Ctrl+G g` |
| Интеграции: править связанный pull request на хостинге | `e (на строке)` |
| Интеграции: копировать URL связанного pull request |  |
| Интеграции: обновить pull requests и задачи |  |
| Интеграции: checkout pull request... |  |
| Интеграции: открыть ссылку из commit'а (#задача / !запрос)... |  |
| Интеграции: открыть задачу... |  |
| Хостинг: ревью запроса... |  |
| Хостинг: push на ревью... |  |
| Хостинг: изменить исполнителей... |  |
| Хостинг: изменить рецензентов... |  |
| Хостинг: изменить метки... |  |
| Хостинг: задать этап... |  |
| Локальные задачи: закрыть задачу... |  |
| Локальные задачи: открыть задачу снова... |  |
| Локальные задачи: открыть в Obsidian... |  |
| Задачи: открыть доску |  |
| Локальные задачи: создать Obsidian Base |  |
| Pull requests: создать pull request... | `n` |
| Хостинг: новая задача... | `n` |
| Pull requests: поиск | `Enter на строке поиска` |
| Pull requests: фильтр (открытые / merged / все)... | `v` |
| Pull requests: провайдер... | `Enter на строке провайдера` |
| Pull requests: выбрать репозиторий... | `Enter на строке репозитория` |
| Pull requests: отвязать источник (нет) |  |
| Задачи: провайдер... | `Enter на строке провайдера` |
| Задачи: отвязать источник (нет) |  |
| Интеграции: отвязать профиль (автоматически) |  |
| Задачи: выбрать репозиторий... | `Enter на строке репозитория` |
| Задачи: поиск | `Enter на строке поиска` |
| Задачи: выбрать фильтр... | `v` |
| Задачи: изменить запрос / списки... |  |
| Commit: вставить ссылку на задачу... |  |
| Интеграции: профиль для этого репозитория... |  |
| Интеграции: профиль этого репозитория |  |
| Интеграции: подключить интеграцию... |  |
| Настройки: параметры Radar для профиля... |  |
| Интеграции: проверка безопасности |  |
| Приложение: выход | `q` |
| Приложение: тема... |  |
| Приложение: сменить тему... | `Ctrl+N y` |
| Приложение: сменить язык... | `Ctrl+N g` |
| Лаунчер: создать ярлык |  |
| Лаунчер: создать ярлык на рабочем столе |  |
| Лаунчер: удалить ярлыки |  |
| Лаунчер: создать псевдоним команды |  |
| Лаунчер: удалить псевдоним команды |  |
| Лаунчер: добавить в PATH |  |
| Лаунчер: убрать из PATH |  |
| Приложение: сменить цветовую тему графа... | `Ctrl+N Y` |
| Приложение: переключить подсветку строк графа | `Ctrl+N T` |
<!-- palette:end -->

## Перемещение

| Клавиша | Действие |
|---|---|
| `Tab` / `Shift+Tab` | Фокус на следующую / предыдущую панель |
| `Ctrl+N`, затем значок | Фокус на эту часть окна (см. [Префикс навигации](#префикс-навигации-ctrln)) |
| `j` / `k`, `Up` / `Down` | Переместить выделение, в любой панели |
| `PageUp` / `PageDown`, `Home` / `End` | На страницу; к первой / последней строке |
| число, затем `j` / `k` / `Up` / `Down` | Префикс числа: `4j` перемещает на четыре строки. С `PageUp` / `PageDown` число означает количество страниц. Строка состояния показывает накопленное число (`4…`). `Esc` или любая другая клавиша сбрасывает его. Ведущий `0` никогда не начинает число |
| `h` / `l`, `Left` / `Right` | Панель Refs: переключить раздел. Log: выйти из списка изменённых файлов commit'а (`h`) или войти в него (`l`) |
| `?` | Открыть или закрыть справку (см. [Справка в приложении](#справка-в-приложении-)) |
| `r` | Обновить |
| `F5` | Сделать fetch всех remote, перезагрузить pull request и задачи и обновить |
| `q` | Выйти |
| `Ctrl+Z` | **Отменить** последнюю операцию. См. [Отмена и повтор](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Повторить**. `Ctrl+Shift+Z` требует клавиатурного протокола kitty; `Ctrl+Y` работает всегда |
| `.`, `Menu`, `Shift+F10` | Открыть контекстное меню выбранной строки |

`Ctrl+Z` и `Ctrl+Y` ничего не делают, пока клавиатура у текстового поля. У
редактора своя отмена текста.

## Списки во всплывающих окнах с фильтром

Все всплывающие списки с фильтром используют одни и те же клавиши: палитра
команд и её выбиралки (тема, язык, цвета графа), выбор вкладки и каждый
выпадающий список со строкой поиска (варианты в Настройках, форма pull request,
выбор ссылки на задачу, список **Открыть в**). Вводите текст, чтобы фильтровать
список. Подсветку двигают эти клавиши:

| Клавиша | Действие |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | На одну строку |
| `Home` / `End` | Первая / последняя строка |
| `PageUp` / `PageDown` | На одну видимую страницу |
| `Enter` | Выбрать подсвеченную строку |
| `Esc` | Закрыть |

В этих окнах `Home`, `End` и `Ctrl+K` двигают список, а не курсор поля фильтра.
`Ctrl+A` и `Ctrl+E` по-прежнему переносят курсор в начало и в конец. Простые `j`
и `k` вводятся в фильтр.

С мышью:

- Колесо **прокручивает** список. Оно не двигает подсветку.
- Щелчок **выделяет** строку. Двойной щелчок **выбирает** её, как `Enter`.
- Список выше своего окна показывает полосу прокрутки у правого края. Её можно
  перетаскивать.
- Короткий выпадающий список без строки поиска (два-три варианта) выбирает по
  первому щелчку.

**Живой предпросмотр.** Выбиралки темы, цветовой темы графа и языка сразу
применяют подсвеченную строку позади окна. `Enter` оставляет её. `Esc` или
щелчок вне окна возвращает прежнее значение.

**Фильтр тем.** У выбиралки тем под полем фильтра есть строка: **Все**,
**Тёмные**, **Светлые** и **Цветные**. `Tab` / `Shift+Tab` переходят к
следующему / предыдущему. `Alt+1` ... `Alt+4` выбирают нужный напрямую. На них
можно и щёлкнуть.

## Справка в приложении (`?`)

`?`, `Ctrl+N ?` или команда палитры `Вид: справка` открывает справку. Она
перечисляет клавиши и жесты мыши каждой части приложения. Она открывается поверх
чего угодно, включая экран Настроек.

| Клавиша | Действие |
|---|---|
| `?` | Закрыть справку (и очистить фильтр) |
| *(ввод)* | Фильтр: оставить только разделы, содержащие текст. `j`, `k` и `q` тоже вводятся |
| `Backspace`, `Ctrl+U` | Удалить последний символ / очистить фильтр |
| `Esc` | Очистить фильтр; если он пуст, закрыть справку |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Прокрутить на строку |
| `PageUp` / `PageDown`, `Home` / `End` | Прокрутить на страницу; в начало / в конец |
| `Ctrl+D` или щелчок по ссылке | Открыть эту документацию на GitHub в браузере |

Колесо прокручивает справку, а её полосу прокрутки можно перетаскивать. Щелчок
вне справки закрывает её.

## Префикс навигации (`Ctrl+N`)

`Ctrl+N` ждёт ещё одну клавишу. Строка состояния показывает, какие клавиши он
принимает. Следующая клавиша выполняет действие или отменяет.

| Клавиша | Действие |
|---|---|
| `Ctrl+N`, затем значок | Фокус на часть окна, показывающую этот бейдж |
| `Ctrl+N h` / `Ctrl+N l` | Свернуть или восстановить панель Refs / правую панель |
| `Ctrl+N Left` / `Ctrl+N Right` | Сдвинуть разделитель между панелью Refs и Log на шаг влево / вправо (свёрнутая панель Refs сначала возвращается) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Сдвинуть разделитель между Log и правой панелью на шаг влево / вправо |
| `Ctrl+N +` / `Ctrl+N -` | Увеличить / уменьшить активный раздел Refs на одну строку |
| `Ctrl+N ]` / `Ctrl+N [` | Увеличить / уменьшить блок сообщения commit'а в правой панели на одну строку |
| `Ctrl+N t` | Выбрать вкладку (выбор вкладки; одна `t` делает то же) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | Сменить тему / язык / цветовую тему графа |
| `Ctrl+N Shift+T` | Включить или выключить подсветку строк графа |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Показать вид, подготовленный агентом / вернуться (см. [MCP и агенты](mcp-and-agents.md)) |
| `Ctrl+N ,` | Открыть Настройки |
| `Ctrl+N ?` | Открыть или закрыть справку |
| `Esc` или любая другая клавиша | Отмена |

### Значки

Пока префикс ждёт, каждая часть окна, способная получить фокус, показывает
односимвольный значок на своей верхней левой границе. Нажмите этот символ, чтобы
перевести на неё фокус. Значки раздаются заново каждый раз, в порядке на экране:
сначала `1` ... `9`, затем `a`, `b`, `c`... Поэтому читайте значок на экране:
у части не всегда один и тот же значок.

Части со значком: фильтр Refs и каждый видимый раздел Refs, Log (или вид Diff),
просмотр файла, когда он открыт, списки unstaged и staged, поле заголовка
композитора commit'а и элементы управления списка файлов commit'а. На вкладке
«Главная» значки стоят у поля поиска, четырёх разделов и кнопок панели инструментов (значок кнопки нажимает её). На вкладке Radar они стоят у
поля фильтра, выпадающих списков «Область», «Вид» и «Показать» (значок списка
раскрывает его), кнопок «Обновить» и «Назад» и у каждого заголовка раздела на экране.
Значок раздела переносит выделение на его заголовок; затем `Enter` или `Space`
сворачивает раздел.

В окне выбора папки «Обзор» и в диалогах «Клонировать», Init, «Создать worktree», «Рабочее пространство», «Sparse checkout», Flow, «Новая задача» и «Pull request» значки стоят у полей, флажков и кнопок (значок кнопки нажимает её, значок поля ставит в него курсор; в списке один значок стоит на его первой строке; в окне выбора папки «Обзор» по одному значку у списков «Быстрый доступ» и «Папки», рядом с первой строкой каждого, и нажатие на него ставит клавиатуру в этот список, не выбирая строку). В журнале активности это обе вкладки, поля «Область» и «Уровень», фильтр, флажки уровней, кнопки и первая запись списка. Значок делает то же, что щелчок по этому элементу. В окне выбора папки `Ctrl+N` включает значки, поэтому клавиша «Новая папка» там `Alt+N`.

В окне «Изменить» (и «Изменить сообщение commit'а») значки стоят у заголовка, описания, имени, email и даты автора, кнопок **Изменить историю глубже...**, **Отмена**, **Применить** и крестика закрытия; в окне переписывания истории - у поля области, его флажков и кнопки ссылок, имени рецепта, каждого правила, его флажка и кнопок, кнопки **+ Добавить правило**, полей выбранного правила, вкладок предпросмотра, поля фильтра, строк защищённых веток и их «Отметить все» / «Снять все», а также у кнопок «Предпросмотр», «Применить», «Отмена», «Сохранить», «Загрузить» и «Создать правила с ИИ» (страницы Secrets, Identities, Size и результата добавляют свои кнопки и строки; строки вкладки Commits, будучи списком, значков не имеют). Значок делает то же, что щелчок. Ничего не получает значков, пока открыт выпадающий список, вопрос, поле, в которое вводится текст, окно выбора папки, экран очистки, редактор скриптов, диалоги ИИ, diff blob'а или справка по хостингу, а также пока окно «Изменить» спрашивает «отменить?»; `Esc` убирает значки, ничего не закрывая.

В Настройках значки стоят у поля поиска, списка страниц (один значок, на текущей странице), подвкладок, списка и его кнопок, каждого поля страницы на экране и крестика закрытия. Значок делает то же, что щелчок по этому элементу: текстовое поле открывается для редактирования, флажок переключается, выпадающий список раскрывается, кнопка выполняется. `Ctrl+N` сохраняет своё значение, пока вводится текст в поле или открыт выпадающий список, редактор или поиск.

В палитре есть те же действия, например
`Навигация: фокус на Refs > Local` или `Навигация: фокус на Status > Staged`.

## Репозитории и вкладки

| Клавиша | Действие |
|---|---|
| `Ctrl+O` или щелчок по `⌂` | Перейти на вкладку «Главная» (на ней: фокус на поле поиска; если открыты Radar или заметки о выпуске: сначала закройте их) |
| `Ctrl+Shift+N` | Открыть вкладку «Главная» с диалогом клонирования |
| `Ctrl+PageDown` / `Ctrl+PageUp` | Следующая / предыдущая вкладка |
| `t` | Выбрать вкладку из списка с фильтром |
| `Ctrl+W` | Закрыть текущую вкладку (сначала спрашивает; репозиторий не меняется) |
| `Ctrl+T` или правый щелчок по вкладке | Меню вкладки (его строки перечислены в разделе [Строка вкладок](interface-tour.md#строка-вкладок)) |
| `Alt+O` / `Ctrl+Alt+O` | Открыть папку вкладки: выбрать из списка **Открыть в** / использовать запись по умолчанию. См. [Открыть в](open-in.md) |

### Вкладка «Главная»

Вкладка `⌂` управляет вашими репозиториями. В ней четыре раздела: **Открытые
репозитории**, **Избранное**, **Недавние репозитории** и **Все репозитории**. См.
[Обзор интерфейса](interface-tour.md#главная).

| Клавиша | Действие |
|---|---|
| `j` / `k`, стрелки, `PageUp` / `PageDown`, `Home` / `End` | Перемещение |
| `Enter` или двойной щелчок | Открыть репозиторий (или перейти на его вкладку). На заголовке раздела: свернуть или развернуть его |
| `f` или `*`, или щелчок по звезде | Добавить в избранное / убрать из него |
| `x`, `Delete` или кнопка строки | Закрыть её вкладку (Открытые), убрать из списка (Недавние), убрать звезду (Избранное), забыть (Все). На диске ничего не удаляется |
| `.`, правый щелчок или `⋮` строки | Меню строки |
| `/` или `Ctrl+F` | Искать во всех разделах по имени, ветке или пути. `Enter` открывает первое совпадение; `Esc` очищает поиск, затем покидает поле |
| `Space`, `Left` / `Right` | Свернуть / развернуть текущий раздел |
| `-` / `+` | Свернуть все / Развернуть все |
| `b` / `c` / `i` | Обзор (открыть папку) / Клон / Init |
| `Esc` | Очистить поиск или вернуться на активную вкладку репозитория |
| `q` | Выйти |

`Ctrl+P`, `Ctrl+,`, `?` и `Ctrl+N` работают на вкладке «Главная». `Ctrl+G`, `Ctrl+W`
и `Ctrl+T` требуют вкладки репозитория: на вкладке «Главная» они лишь показывают
сообщение. Команды палитры, которым нужен репозиторий, серые.

### Выбор папки

**Обзор** (и кнопка **Обзор...** диалогов Init, Clone и создания worktree)
открывает выбор папки. Вверху у него поле пути, слева места и дерево папок,
справа содержимое текущей папки.

| Клавиша | Действие |
|---|---|
| `Tab` / `Shift+Tab` | Поле пути, затем левая колонка, затем содержимое |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Перемещение в колонке с фокусом. В поле пути: перемещение по вариантам дополнения |
| буква (содержимое) | Перейти к следующей папке, начинающейся с этой буквы |
| `Enter` / `Right` | Открыть выбранную папку. В поле пути: перейти в введённую папку; если она уже показана, выбрать её |
| `Backspace` / `Left` / `Alt+Up` | Родительская папка (в поле пути `Backspace` удаляет символ) |
| `Tab` (поле пути) | Принять единственный вариант дополнения или подсвеченный |
| `Alt+N` | Новая папка (`Ctrl+N` включает значки, см. [Значки](#значки)) |
| `Alt+H` | Показать или скрыть скрытые папки |
| `Ctrl+F` или `/` вне поля пути | Фильтровать записи папки по мере ввода (без учёта регистра; `..` остаётся; заголовок показывает «x из n»; `Enter` переходит к первому совпадению, `Esc` очищает; при смене папки фильтр очищается). Также доступен через `Tab` или щелчком |
| `Ctrl+S`, `Ctrl+Enter` или кнопка подтверждения | Выбрать выделенную подпапку, а при её отсутствии показанную папку |
| `Esc` | Закрыть варианты дополнения или отменить |

Когда поле пути показывает новый путь, он выделяется целиком: первая введённая
клавиша заменяет его.

### Диалоги Init и Clone

| Клавиша | Действие |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Следующее / предыдущее поле или кнопка |
| `Enter` | Нажать кнопку с фокусом либо создать / клонировать |
| `Space` | Переключить флажок с фокусом или нажать кнопку с фокусом |
| `Ctrl+B` | Выбрать папку |
| `j` / `k`, `g` / `G` | В списке источников и списке репозиториев диалога Clone: вниз / вверх, первый / последний (не в поле поиска) |
| `Esc` | Закрыть. Во время клонирования клонирование продолжается в фоне |

В диалоге Clone левая колонка перечисляет **Клонировать по URL** и по одному
источнику на каждый подключённый профиль хостинга (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Выберите источник клавишами `Up` / `Down` или щелчком. Затем вводите
текст в поле поиска **Репозиторий**, перемещайтесь по списку стрелками,
`PageUp` / `PageDown` или колесом и выбирайте строку клавишей `Enter` или
щелчком. О полях см. [Начало работы](getting-started.md).

## Панель Refs

| Клавиша | Действие |
|---|---|
| `Enter` / `Space` | Развернуть или свернуть активный раздел либо выбранную папку |
| `h` / `l`, `Left` / `Right` | Переключить раздел |
| `/` | Фильтр по имени (`Enter` применяет, `Esc` отменяет) |
| `y` | Включить или выключить иерархию веток: папки по `/` в именах веток (LOCAL и REMOTE) |
| `o` | **Solo** для выбранной ссылки либо всех ссылок в выбранной папке или remote. Включите solo для нескольких ссылок, чтобы увидеть их объединение |
| `Shift+O` | **Снять solo**: сбросить все solo |
| `Shift+H` | **Показать все**: снова показать все скрытые ссылки |
| `g` | **Показать в графе**: прокрутить Log к commit'у выбранной ссылки (или stash'а) и мигнуть его строкой; выделение не двигается |
| `m` | **Пометить** выбранную ссылку как источник перетаскивания (`Esc` или повторный `m` на ней снимает пометку) |
| `Shift+M` | **Перетащить помеченную ссылку** на выбранную: открывает меню, которое открывает перетаскивание (Merge, Rebase, Push, Fast-forward...) |
| `+` (раздел REMOTE) | Добавить remote |
| `.`, `Menu`, `Shift+F10` или правый щелчок | Меню строки: ветка, tag, имя remote или папка |
| `Enter` на строке WORKTREES | Открыть этот worktree в отдельной вкладке |
| `Ctrl+G d` | Удалить выбранную ветку или tag (сначала спрашивает). На строке REMOTE: удалить эту ветку remote |

О том, что делают меню, см. [Работа с Log](log-and-history.md).

### Pull request и задачи

При подходящей интеграции в панели Refs есть раздел pull request (GitLab: merge
request). При подключённом профиле хостинга есть и раздел ЗАДАЧИ. См.
[Интеграции с хостингами и трекерами задач](hosting-integration.md#5-pull-request-и-задачи-на-боковой-панели).

| Клавиша | Действие |
|---|---|
| `Enter` | Открыть выбранную строку в браузере. На строке `Reconnect`: открыть Настройки на этом профиле |
| `Space` | Развернуть или свернуть раздел |
| `v` | Открыть фильтр раздела. Pull request: Открытые, Merged / закрытые, Все. Задачи: Открытые, Назначенные мне, Закрытые, Все |
| `n` | Раздел pull request: создать pull request. Раздел ЗАДАЧИ: новая задача. (Кнопка `+` в заголовке) |
| `e` | Раздел pull request: править выбранный запрос на хосте (карандаш в строке) |
| `Enter` на строке поиска | Вводить в поле поиска. `Enter`, `Tab` или `Esc` покидает его; `Ctrl+U` очищает |
| `Enter` на заголовке группы | Свернуть или развернуть группу (*Мои pull requests*, *Назначенные мне*, *Ждут моей проверки*, *Все pull requests*) |
| `Enter` на строке фильтра, провайдера или репозитория | Открыть этот выпадающий список. Список провайдеров заканчивается пунктом **Нет (отвязать этот репозиторий)** |
| `.` или правый щелчок | Меню строки: открыть в браузере, скопировать URL или номер, сделать checkout исходной ветки (pull request); создать ветку из задачи (задачи) |

### Доска задач

Кнопка **Доска** в заголовке ЗАДАЧИ (палитра: **Задачи: открыть доску**) открывает канбан-доску источника задач. См.
[доску](hosting-integration.md#5b-доска-задач) и [локальные задачи](local-issues.md#доска-канбан). Каждая клавиша ниже
есть и как команда палитры (**Доска: ...**).

| Клавиша | Действие |
|---|---|
| Стрелки | Выбрать карточку (`Вверх`, `Вниз`) или соседнюю колонку (`Влево`, `Вправо`); `Home`, `End`, `PageUp`, `PageDown` переходят внутри колонки |
| `Shift+←`, `Shift+→` | Перенести выбранную карточку в предыдущую или следующую колонку |
| `m` | **Перенести карточку в...**: выпадающий список колонок |
| `Enter` | Открыть задачу (заметку - для локальной задачи; иначе страницу хоста) |
| `e`, `.` | Изменить исполнителей, метки или этап; меню карточки |
| `n` | Новая задача в выбранной колонке |
| `a`, `l`, `M`, `/` | Фильтр по исполнителю, метке, этапу, тексту. `x` сбрасывает фильтры |
| `c` | Метки, которые служат колонками доски GitHub / GitLab |
| `r` | Прочитать задачи заново |
| `Esc` | Закрыть доску |

Мышь: перетащите карточку в другую колонку, чтобы перенести её, щёлкните карточку, чтобы выбрать, двойной щелчок
открывает её, правый щелчок - меню карточки, колесо прокручивает колонку, `Shift` с колесом (или полоса под доской)
прокручивает колонки.

### Flow (рабочие процессы ветвления)

Если для репозитория настроен рабочий процесс (см. [рабочие процессы ветвления](branching-workflows.md)), на панели
инструментов появляется кнопка **Flow**, а боковая панель группирует LOCAL и REMOTE по типам веток. Каждая строка
меню - команда палитры (`Flow: ...`; `Flow: начать Feature...` и `Flow: новая задача (Feature)...` создаются для
каждого типа тематических веток рабочего процесса). Строки, управляемые хостом (*Новая задача (<тип>)*, *Подхватить
ветку, созданную хостом*), их окна, карточка ожидания и карточка «появилась ветка» описаны в
[рабочих процессах ветвления](branching-workflows.md#процессы-управляемые-хостом-в-приложении); `Enter` запускает
кнопку в фокусе (*Создать задачу*, *Подхватить*), `Esc` прекращает ожидание или закрывает окно.

| Клавиша | Действие |
|---|---|
| `Ctrl+G Shift+W` | Открыть меню Flow (кнопка на панели инструментов) |
| `Enter`, `Tab`, `Shift+Tab` | В окне «Начать» / «Завершить» / «Синхронизировать» / «Продвинуть» / «Задать родителя»: просмотр или запуск, следующее поле, предыдущее поле |
| `Space` | Отметить флажок (*Опубликовать ветку*, *Запустить хуки*, *Обновить и ветки, построенные поверх неё*) |
| `Esc` | Закрыть выпадающий список, вернуться из просмотра к форме, закрыть окно (не во время выполнения) |
| Наведение на ветку | Её тип, родитель и на сколько она отстаёт от родителя |

## Панель Log

| Клавиша | Действие |
|---|---|
| `Enter` / `l` | Войти в список изменённых файлов выбранного commit'а |
| `Enter` / `l` на строке файла | Открыть diff файла в центре |
| `Esc` / `h` | Вернуться к commit'ам |
| `Right` | (commit с более чем одной ссылкой) Открыть список его других ссылок: `Up` / `Down` перемещают, `Enter` делает checkout, `Esc` закрывает |
| `f` | Открыть строку поиска (см. [Поиск](#поиск)) |
| `c` | (при незафиксированных изменениях) Выбрать строку WIP и поставить курсор в её поле сообщения |
| `Enter` на строке WIP | Поставить курсор в поле сообщения строки WIP |
| `.`, `Menu`, `Shift+F10` | Меню commit'а. `Up` / `Down` перемещают, `Right` / `l` / `Enter` открывают подменю, `Left` / `h` / `Esc` возвращают назад |
| `m` / `a` | Файлы правой панели: `Путь` или `Дерево` / `Показать все файлы` |
| `Shift+C` | Выбрать колонку Log для сдвига или изменения ширины (см. ниже) |

С выбранной колонкой `Tab` / `Shift+Tab` выбирают другую, `<` / `>` сдвигают её, `]` / `[` расширяют или сужают, `0` возвращает автоматическую ширину, а `Esc` или `Enter` выходит (контекст `log_columns` в сгенерированной таблице выше).

Поле сообщения строки WIP - это заголовок композитора commit'а. Пока в нём
курсор, вводятся все клавиши, но сочетания с `Ctrl` по-прежнему работают. `Enter`
переходит к описанию композитора. `Esc` покидает поле и сохраняет текст. См.
[Индекс и commit'ы](staging-and-committing.md).

### Выбор нескольких commit'ов

| Клавиша или жест | Действие |
|---|---|
| `Space` | Добавить текущий commit в выделение или убрать его |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Расширить выделение (Alt для терминалов, которые оставляют Shift себе) |
| `Ctrl+щелчок` / `Shift+щелчок` или `Alt+щелчок` | Переключить commit / выделить диапазон |
| `Esc` | Сбросить выделение |
| `Shift+Y` | [Копировать выделение как Mermaid gitGraph](log-and-history.md#копирование-commitов-как-mermaid-gitgraph) (окно с уровнями детализации и живым просмотром; `Enter` копирует) |

Если выбрано несколько commit'ов, щёлкните по одному из них правой кнопкой и
выберите **Cherry pick выбранных commit'ов (N) на `<ветка>`...**. Откроется тот же вид, что
и у интерактивного rebase.

### Имя для ветки или tag

`Ctrl+G b`, `Ctrl+G t`, кнопки **Ветка** и **Tag** на панели инструментов и
пункты меню **Создать ветку здесь...** / **Создать tag здесь...** запрашивают имя
в текстовом поле на строке commit'а. `Enter` создаёт ссылку. `Esc` отменяет.
Недопустимое или уже существующее имя показывается красным, а поле остаётся
открытым.

### Изменение сообщения commit'а

**Изменить сообщение commit'а...** в меню commit'а, `Ctrl+G e` или команда
палитры `Git: изменить сообщение commit'а...` открывает окно **Изменение
сообщения commit'а**. Что оно меняет, см. в [Работе с Log](log-and-history.md).

| Клавиша | Действие |
|---|---|
| `Enter` в заголовке | Перейти к описанию |
| `Enter` в описании | Новая строка. `Up` на первой строке возвращает к заголовку |
| `Ctrl+Enter`, `Ctrl+S` или **Обновить сообщение** | Сохранить новое сообщение |
| `Tab` / `Shift+Tab` | Перемещение между заголовком, описанием и кнопками |
| `Esc` или **Отмена** | Закрыть без изменений |

### Изменение commit'а

`Ctrl+G Shift+E` (все поля) и `Ctrl+G Shift+A` (автор) открывают окно изменения
commit'а; так же его открывают меню commit'а, команды палитры `Commit: изменить...`
и панель сведений о commit'е. Все способы описаны в разделе
[Изменение commit'а](log-and-history.md#изменение-commitа-сообщение-автор-даты).
Оба сочетания можно переназначить: `git_prefix.amend` и `git_prefix.edit_author`.
Клавиши внутри окна:

| Клавиша | Действие |
|---|---|
| `Tab` / `Shift+Tab` | Перемещение между полями и кнопками |
| `Enter` в однострочном поле | Следующее поле; после последнего - **Применить** |
| `Up` / `Down` | Предыдущее / следующее поле (в описании - с его первой / последней строки) |
| `Ctrl+Enter`, `Ctrl+S` или **Применить** | Применить изменённые поля |
| `Esc` или **Отмена** | Закрыть без изменений |
| `Alt+H` или **Изменить историю глубже...** | Закрыть окно и открыть окно переписывания истории с этого commit'а (только в сборках с переписыванием истории); если вы изменили поле, сначала спрашивает (`Esc` подтверждает). Переназначается: `reword.deeper` |

### Создание worktree

**Создать worktree из `<ссылка>`...** (меню веток и tag'ов) и **Создать worktree
здесь...** (меню commit'а) открывают диалог: папка, необязательная новая ветка и
**Открыть новый worktree во вкладке**. Клавиши: `Tab` / `Shift+Tab` перемещают,
`Enter` создаёт, `Space` переключает флажок, `Ctrl+B` выбирает папку, `Esc`
отменяет.

### Интерактивный rebase

**Интерактивный rebase отсюда...** в меню commit'а или
`Git: интерактивный rebase от выбранного commit'а...` открывает вид rebase.
Cherry-pick нескольких commit'ов использует тот же вид. См.
[Работа с Log](log-and-history.md).

| Клавиша | Действие |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Задать действие строки: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup и Edit только в rebase) |
| `Space` | Открыть список действий строки (`Up` / `Down` выбирают, `Enter` применяет, `Esc` закрывает список) |
| `Up` / `Down` | Переместить выделение |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Переместить выбранную строку вверх / вниз |
| `a` | Включить или выключить предпросмотр autosquash |
| `Enter` на строке Reword | Изменить новое сообщение этого commit'а |
| `Ctrl+S`, `Ctrl+Enter` или **Начать rebase** / **Начать cherry-pick** | Выполнить план |
| `Ctrl+R` или **Сброс** | Восстановить план таким, каким он был при открытии вида |
| `Esc` или **Отмена** | Закрыть без изменений |

Когда rebase останавливается (на конфликте или шаге Edit), баннер над Log
предлагает **Продолжить**, **Пропустить** и **Прервать**. В палитре есть
`Rebase: продолжить`, `Rebase: пропустить` и `Rebase: прервать`.

## Поиск

`f`, кнопка **Поиск** на панели инструментов или `Навигация: поиск commit'ов...`
открывает строку поиска над Log. Она ищет по полному сообщению commit'а, имени
и адресу автора, SHA и именам ссылок. Строки она не скрывает: несовпавшие строки
тускнеют.

| Клавиша | Действие |
|---|---|
| *(ввод)* | Править текст поиска (вводятся все клавиши) |
| `Enter` / `Down` или `↑` строки | Следующее совпадение |
| `Shift+Enter` / `Up` или `↓` строки | Предыдущее совпадение |
| `Esc` или `✕` строки | Закрыть строку поиска |

## Вид Diff

Изменённый файл открывается в центре вместо Log. См.
[Индекс и commit'ы](staging-and-committing.md).

| Клавиша | Действие |
|---|---|
| `Enter` / `l` или двойной щелчок (на строке файла) | Открыть diff файла |
| `Esc`, `q` или `✕` в заголовке | Закрыть diff |
| `j` / `k`, `Up` / `Down` | Прокрутка |
| `PageUp` / `PageDown`, `Home` / `End` | Прокрутка на страницу; в начало / в конец |
| `h` / `l`, `Left` / `Right` | Прокрутка вбок (не при переносе строк) |
| `n` / `p` | Следующий / предыдущий hunk |
| `m` | Раскладка «Единый» или «Рядом» |
| `Shift+H` | Только hunk'и или весь файл |
| `w` | Включить или выключить перенос строк |
| `s` | Показать или скрыть изменения только пробелов |
| `Shift+M` | Включить или выключить миникарту |
| `Shift+C` | Выбрать кодировку, в которой читается файл (список: сначала «Определять автоматически») |
| `e` | Править файл во встроенном редакторе (см. [Редактор](#редактор)) |
| `r` | (файл Git LFS) Карточка LFS или сырой указатель. См. [Git LFS](lfs.md) |
| `Shift+E` | (файл с конфликтом) Открыть редактор слияния. См. [Разрешение конфликтов слияния](merge-conflicts.md) |
| `y` / `Ctrl+C` | Скопировать выделенный текст. Без выделения: строку под курсором или выбранные строки |
| `.` или правый щелчок по hunk'у | Меню hunk'а |

### Stage и отбрасывание из diff

Эти клавиши работают только в diff рабочих изменений (открытом из списка
unstaged или staged). Отбрасывание работает только в списке unstaged.

| Клавиша | Действие |
|---|---|
| `Space` | Stage (unstaged) / unstage (staged) для hunk'а под курсором |
| `v` | Начать или сбросить выделение строк; `j` / `k` расширяют его |
| `Enter` | Stage / unstage для выбранных строк или строки под курсором |
| `x` | Отбросить hunk под курсором (сначала спрашивает: `y` / `Enter` подтверждают, `n` / `Esc` отменяют) |
| `Shift+X` | Отбросить выбранные строки или строку под курсором (сначала спрашивает) |

Учитываются только изменённые строки: строки контекста в выделении
игнорируются. Выделение, заходящее во второй hunk, действует только на первый
hunk.

## Просмотр изображений

Картинка в виде Diff или в просмотре файла. Клавиши указаны в контексте `image`
таблицы выше. См. [Изображения в виде Diff и в просмотре файла](image-view.md).

## Просмотр файла, Blame и история файла

| Клавиша | Действие |
|---|---|
| `Esc` / `q` | Закрыть (при выделенном тексте `Esc` сначала снимает выделение) |
| `j` / `k`, `Up` / `Down` | Переместить выделение |
| `PageUp` / `PageDown` | Прокрутка на страницу |
| `Home` / `End`, `Shift+G` | Первая / последняя строка |
| `h` / `l`, `Left` / `Right` | Прокрутка текста вбок |
| `Enter` | (Blame, история файла) Перейти к commit'у этой строки в Log |
| `e` | Править файл во встроенном редакторе |
| `y` / `Ctrl+C` | Скопировать выделенный текст. Без выделения: выбранную строку (Blame: только код; «История файла»: заголовок) |

У Blame и истории файла есть собственные клавиши, перечисленные в сгенерированной таблице выше (контексты `file_insight` и `history`) и как `Blame: ...` / `История файла: ...` в палитре; все можно переназначить. Подробнее в [Blame](log-and-history.md#blame) и [История файла](log-and-history.md#история-файла). В списке истории одиночные буквы (`d` `o` `b` `c` `y`) действуют на выбранную строку, но сочетание с Ctrl (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) всегда сохраняет глобальный смысл.

## Колонка commit'ов

Колонка слева от вида файла (см. [Колонка commit'ов](interface-tour.md#колонка-commitов)). `Alt+C` показывает или скрывает её откуда угодно; остальные клавиши работают, когда у колонки клавиатура (`Ctrl+N` и её значок, либо `Вид: перейти к колонке commit'ов`).

| Клавиша | Действие |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Выбрать commit |
| `Enter` / `Right` | Перейти к списку файлов выбранного commit'а |
| `Esc` | Вернуть клавиатуру виду |
| `.`, `Menu`, `Shift+F10` | Меню выбранной строки |

Остальные её клавиши (`Shift+Left` / `Shift+Right` для прокрутки вбок, `Tab`, `]`, `[` и `0` для ширины частей) есть в сгенерированной таблице выше, контекст `commit_column`.

## Редактор

**Правка**, либо `e` в diff или просмотре файла, открывает встроенный редактор
на копии файла в рабочей папке. См. [Индекс и commit'ы](staging-and-committing.md).

| Клавиша | Действие |
|---|---|
| *(ввод)* | Вставить у курсора (заменяет выделение) |
| `Enter` | Новая строка с отступом текущей строки |
| `Tab` / `Shift+Tab` | Увеличить / уменьшить отступ (выбранных строк, если их несколько) |
| стрелки, `Home` / `End` | Перемещение. `Home` переходит к первому непробельному символу, затем к столбцу 0. `Shift` расширяет выделение |
| `Ctrl+Left` / `Ctrl+Right` | На слово |
| `Ctrl+Home` / `Ctrl+End` | Начало / конец файла |
| `PageUp` / `PageDown` | На страницу вверх / вниз |
| `Ctrl+A` | Выделить всё |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Копировать / вырезать / вставить |
| `Ctrl+U` / `Ctrl+K` | Удалить до начала / до конца строки |
| `Ctrl+Z` | Отменить |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Повторить |
| `Ctrl+S` | Сохранить |
| `Ctrl+F` / `F3` | Поиск. В поле поиска: `Enter` / `Down` / `F3` далее, `Shift+Enter` / `Up` назад, `Esc` закрывает |
| `Alt+M` | Включить или выключить миникарту (**Карта**) |
| `Esc` | Закрыть. При несохранённых изменениях спрашивает: **Сохранить** (`S` / `Enter`), **Отбросить** (`D`), **Отмена** (`C` / `Esc`) |

`Tab` в редакторе делает отступ, поэтому не переводит фокус. Другие сочетания с
`Ctrl` (`Ctrl+P`, `Ctrl+N`) по-прежнему работают.

**Кодировка**, **окончания строк** и **отступы** в строке состояния - кнопки: каждая открывает выпадающий
список (стрелки или колесо, ввод для фильтра, `Enter` - выбрать, `Esc` - закрыть). Палитра (`Ctrl+P`) открывает
те же меню: *Редактор: сменить кодировку*, *Редактор: открыть заново в кодировке*, *Редактор: преобразовать в кодировку*, *Редактор: преобразовать переводы строк*, *Редактор: изменить отступы*. Подробности - в [Индекс и commit'ы](staging-and-committing.md).

## Действия Git (`Ctrl+G`)

`Ctrl+G` ждёт ещё одну клавишу:

| Клавиша | Действие |
|---|---|
| `s` / `u` | Stage / unstage выбранного файла |
| `a` | Stage всех изменений |
| `x` | Отбросить все изменения (сначала спрашивает) |
| `c` | Открыть композитор commit'ов |
| `Enter` | Сразу сделать commit staged-изменений с сообщением из формы, откуда угодно (см. [Индекс и commit'ы](staging-and-committing.md#написание-commitа)) |
| `m` | Stash всех изменений с сообщением (пустое допустимо). При включённом ИИ `Ctrl+G` в этом запросе пишет сообщение |
| `Shift+T` | Stash всех изменений (включая неотслеживаемые файлы) без сообщения |
| `o` | Pop stash: строка stash'а, выбранная в Log, а иначе последний stash |
| `p` | Pull так, как настроена кнопка «Pull» (`git.pull_mode`) |
| `Shift+P` | Push в remote по умолчанию (при первом push задаёт upstream) либо запросить remote |
| `Shift+U` | Push всех tag'ов |
| `f` | Fetch всех remote, с prune |
| `g` | Открыть pull request ветки с checkout в браузере |
| `b` / `t` | Новая ветка / новый tag на HEAD (см. [Имя для ветки или tag](#имя-для-ветки-или-tag)) |
| `d` | Удалить выбранную ветку или tag (сначала спрашивает) |
| `e` | Изменить сообщение выбранного commit'а |
| `Shift+E` / `Shift+A` | Изменить выбранный commit (все поля) / его автора (см. [Изменение commit'а](#изменение-commitа)) |
| `y` | Cherry-pick выбранного commit'а |
| `v` | Revert выбранного commit'а |
| `Shift+R` | Reset (mixed) к выбранному commit'у |
| `r` | Rebase на выбранную ветку |
| `Shift+W` | Открыть меню Flow (в репозитории с процессом ветвления) |
| `Shift+H` | Открыть окно переписывания истории (только в сборках с переписыванием истории) |
| `l` | Открыть меню LFS. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch всех / checkout всех / pull / push всех / prune. См. [Git LFS](lfs.md) |
| `i` | ✦ ИИ: написать сообщение commit'а для изменений в индексе |
| `w` | ✦ ИИ: объяснить выбранный commit или рабочие изменения |
| `Shift+C` | ✦ ИИ: предложить, как разбить рабочие изменения на commit'ы |

Клавиши ИИ работают только при включённом ИИ (см. [Помощь ИИ](ai.md)). `Esc`
отменяет выполняющийся запрос к ИИ.

## Правая панель (рабочие изменения и детали commit'а)

Если выбрана строка WIP, правая панель показывает списки **Файлы unstaged** и
**Файлы staged** и композитор commit'а. Если выбран commit, она показывает
детали commit'а и его файлы.

| Клавиша | Действие |
|---|---|
| `j` / `k`, `Up` / `Down` | Перемещение. В конце одного списка выделение переходит в другой |
| `h` / `l`, `Left` / `Right` | Перейти к списку unstaged / staged |
| `Enter` | Свернуть или развернуть активный список |
| `Space` / `Insert` | Добавить текущий файл в выделение или убрать его |
| `Ctrl+A` | Выделить все файлы списка |
| `Ctrl+Space` | Stage / unstage выбранного файла либо свернуть строку папки |
| `d` | Открыть diff выбранного файла |
| `m` | `Путь` (плоский список) или `Дерево` (папки) |
| `a` | `Показать все файлы`: всё дерево, а не только изменённые файлы |
| `z` | Свернуть все папки или снова развернуть их |
| `/` | Фильтровать изменённые файлы commit'а по имени |
| `.`, `Menu`, `Shift+F10` | Меню файла |
| `c` | Фокус на поле заголовка композитора commit'а |
| `Shift+A` | Включить или выключить **Amend предыдущего commit'а** |
| `Ctrl+N ]` / `Ctrl+N [` | Увеличить / уменьшить блок сообщения commit'а |

В композиторе commit'а:

| Клавиша | Действие |
|---|---|
| `Enter` | Заголовок: перейти к описанию. Описание: новая строка |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` делает это откуда угодно) |
| `Alt+R` | Вставить ссылку на задачу |
| `Esc` | Покинуть композитор |

В меню файла: Stage или Unstage, Отбросить изменения, Игнорировать, Stash файла,
История файла, Blame файла, Открыть во внешнем diff-инструменте, Открыть во
внешнем редакторе, Открыть файл в программе по умолчанию, Показать в папке,
Копировать путь файла, Создать патч из изменений файла, Править файл и Удалить
файл. Для файла с конфликтом меню начинается с **Открыть в редакторе слияния** и
трёх строк **Разрешить через**. При включённом Git LFS в нём есть строки LFS.
**Отбросить изменения** и **Удалить файл** сначала спрашивают.

## Кнопки окна

У каждой кнопки есть клавиатурный двойник:

| Кнопка | Клавиша |
|---|---|
| 🐙 слева в строке заголовка (О программе) | `Справка: о программе GitPoulpi` в палитре |
| **Открыть в** в строке заголовка | `Alt+O`, `Ctrl+Alt+O` |
| `×` справа в строке заголовка (выход) | `q` |
| Версия в строке заголовка (заметки о выпуске) | `Справка: заметки о выпуске` в палитре |
| `⌕ Глобальный поиск` в строке заголовка, слева от `⚙` | `Ctrl+Alt+F` |
| `⚙` в строке заголовка (Настройки) | `Ctrl+,` |
| Значок рабочего пространства в строке вкладок | `Alt+W` |
| `+` после последней вкладки | `Alt+T` |
| Вкладка **Radar** | `Alt+R` |
| Вкладка **Рабочие пространства** | `Alt+J` |
| **Активность** в строке состояния | `Ctrl+N Shift+A` |
| Панель инструментов: **Отменить**, **Повторить** | `Ctrl+Z`, `Ctrl+Y` |
| Панель инструментов: **Pull** / её `▾` | `Ctrl+G p` / `Git: параметры pull...` |
| Панель инструментов: **Push** / её `▾` | `Ctrl+G Shift+P` / `Git: push на remote...` |
| Панель инструментов: **Ветка**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Панель инструментов: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Панель инструментов: **LFS** (при включённом LFS) | `Ctrl+G l` |
| Панель инструментов: **Flow** (когда в репозитории задан процесс ветвления) | `Ctrl+G Shift+W` |
| Панель инструментов: **Поиск** | `f` |
| Панель инструментов: связанный pull request (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Панель инструментов: список профилей (при двух и более профилях) | `Репозиторий: сменить профиль...` |

Панель инструментов описана в [Обзоре интерфейса](interface-tour.md#панель-инструментов).

## Настройки

`Ctrl+,`, `Ctrl+N ,`, шестерёнка `⚙` или `Приложение: настройки...` открывает
экран Настроек.

| Клавиша | Действие |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Переход между списком страниц и настройками |
| `j` / `k`, `Up` / `Down` | Переместить выделение |
| `Enter` / `Space` | Открыть страницу, переключить флажок, править поле, открыть выпадающий список или нажать кнопку |
| `/`, `Ctrl+F` | Искать по всем настройкам |
| `Alt+Left` / `Alt+Right` | Переключить вложенную вкладку (Общие / список) |
| `Esc` / `q` | Закрыть Настройки. Пока правится поле, `Esc` отменяет только правку |
| `?` | Открыть справку поверх экрана Настроек |

См. [Настройки](settings.md).

## Текстовые поля

У всех текстовых полей одни и те же клавиши: фильтры, поля поиска, палитра
команд, запросы, композитор commit'а, поле строки WIP, поля диалогов и поля
Настроек. Пока клавиатура у поля, эти клавиши имеют приоритет над сочетаниями
панелей.

| Клавиша | Действие |
|---|---|
| символ | Ввести его у курсора (заменяет выделение) |
| `Left` / `Right` | На один символ (при выделении: к его началу / концу) |
| `Ctrl+Left` / `Ctrl+Right` | На слово |
| `Home` / `End` | Начало / конец строки |
| `Shift` + перемещение выше | Расширить или сузить выделение |
| `Up` / `Down` (только описание) | Предыдущая / следующая строка |
| `Ctrl+Home` / `Ctrl+End` (только описание) | Начало / конец текста |
| `Ctrl+A` | Выделить всё |
| `Backspace` / `Delete` | Удалить символ перед курсором / после него либо выделение |
| `Ctrl+U` | Удалить от курсора до начала строки (или выделение) |
| `Ctrl+K` | Удалить от курсора до конца строки (или выделение) |
| `Ctrl+C` / `Ctrl+X` | Копировать / вырезать выделение (не в поле пароля) |
| вставка (собственная вставка терминала) | Вставить у курсора, заменяя выделение |

В описании commit'а `Enter` вводит новую строку, а `Ctrl+Enter` делает commit.
Не все терминалы одинаково отправляют `Ctrl+Enter`, поэтому commit делают и
`Alt+Enter`, и `Ctrl+J`.

Некоторые консоли отправляют `Ctrl+буква` как управляющий символ (`Ctrl+A` как
`0x01`). GitPoulpi понимает обе формы. Символы `AltGr` (`@`, `#`, `€` на
швейцарской или французской клавиатуре) вводятся.

## Подсказки

Подсказка появляется, когда указатель около 0,6 с остаётся на цели (ячейке Log,
кнопке панели инструментов, связанном pull request). Нажатие клавиши, щелчок,
прокрутка, изменение размера, открытое окно или меню либо уход указателя с цели
скрывают её. После этого она возвращается, только когда указатель снова
двигается.
