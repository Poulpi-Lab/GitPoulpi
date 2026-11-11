[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · **Українська** · [简体中文](../zh-CN/keybindings.md)

# Скорочення клавіатури

На цій сторінці перелічено **типові** клавіші. Більшість із них можна змінити. Застосунок
завжди показує чинні клавіші: у палітрі команд, у довідці `?`, у рядках стану, у
контекстних меню та в підказках кнопок. Коли ви переносите клавішу, ці місця показують нову.
Про мишу див. [Миша та жести](mouse-and-gestures.md).

## Перепризначення

Відкрийте **Налаштування > Клавіатура й миша**. Сторінку описано в
[Налаштуваннях](settings.md#клавіатура-й-миша). Зміни застосовуються одразу й зберігаються
в таблиці `[keybindings]` файлу `config.toml`. Цю таблицю можна також редагувати вручну:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

Синтаксис, правила конфліктів і попередження описано в
[Довіднику конфігурації](config-reference.md) у розділі `[keybindings]`. Коротко:

- Про помилку повідомляє рядок стану, а типова клавіша залишається.
- Дві дії на одній комбінації в одному контексті відхиляються.
- Префікс потребує `Ctrl` або `Alt`.
- Комбінація панелі, що затуляє глобальну, дає лише попередження.

У таблиці нижче перелічено кожну клавішу, яку можна перепризначити. Деякі клавіші
фіксовані й не перепризначаються: рух списками (`j` `k` `h` `l`, стрілки, `PageUp` /
`PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`, цифри (префікс-лічильник), клавіші
редагування тексту та клавіші `Enter` / `Tab` / стрілки спливних вікон. У палітрі є
`Застосунок: скорочення...` і `Скорочення: скинути все...`.

## Типові скорочення

Кожна клавіша, яку можна перепризначити, з ідентифікатором для `[keybindings.<контекст>]`
та її типовим значенням. Таблицю згенеровано із застосунку (тест підтримує її актуальною).
Клавіші після префікса показано разом із префіксом (`Ctrl+G s`).

<!-- keymap:begin -->
| Контекст | Id дії | Що робить | Типові клавіші |
|---|---|---|---|
| `global` | `git_prefix` | Префікс дій Git | `Ctrl+G` |
| `global` | `nav_prefix` | Префікс навігації | `Ctrl+N` |
| `global` | `palette` | Палітра команд | `Ctrl+P` |
| `global` | `settings` | Налаштування | `Ctrl+,` |
| `global` | `home` | Головна (керування репозиторіями) | `Ctrl+O` |
| `global` | `clone` | Клонувати репозиторій | `Ctrl+Shift+N` |
| `global` | `new_tab` | Нова вкладка | `Alt+T` |
| `global` | `workspace_switch` | Перемкнути робочий простір | `Alt+W` |
| `global` | `radar` | Відкрити Radar | `Alt+R` |
| `global` | `workspaces` | Відкрити робочі простори | `Alt+J` |
| `global` | `tab_menu` | Меню вкладки | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Відкрити в (список) | `Alt+O` |
| `global` | `tab_open_in_default` | Відкрити в типовому | `Ctrl+Alt+O` |
| `global` | `close_tab` | Закрити вкладку | `Ctrl+W` |
| `global` | `next_tab` | Наступна вкладка | `Ctrl+PageDown` |
| `global` | `prev_tab` | Попередня вкладка | `Ctrl+PageUp` |
| `global` | `help` | Довідка | `?` |
| `global` | `quit` | Вийти | `q` |
| `global` | `refresh` | Оновити | `r` |
| `global` | `refresh_all` | Оновити (fetch, pull requests, issues) | `F5` |
| `global` | `find_in_log` | Пошук у графі commit'ів | `f` |
| `global` | `workspace_search` | Пошук у робочому просторі | `Ctrl+Alt+F` |
| `global` | `select_tab` | Вибрати вкладку | `t` |
| `global` | `undo` | Скасувати останню операцію | `Ctrl+Z` |
| `global` | `redo` | Повторити скасовану операцію | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Відкрити історію скасувань | `Alt+Z` |
| `global` | `commit_column` | Показати або сховати стовпець commit'ів | `Alt+C` |
| `global` | `undo_offer` | Відновити гілку туди, де її залишила заблокована дія | `Alt+Shift+Z` |
| `global` | `context_menu` | Контекстне меню | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Написати повідомлення commit'у | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Commit змін зі stage зараз | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | ШІ: розбити зміни на commits | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | ШІ: пояснити зміни | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash з повідомленням | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | ШІ: написати повідомлення commit'у | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage усіх змін | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Відкинути всі зміни | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage вибраного файлу | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage вибраного файлу | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Відкрити пов'язаний pull request | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push усіх tags | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch усіх remote | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Новий tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Нова гілка | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash змін | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick вибраного commit'у | `Ctrl+G y` |
| `git_prefix` | `revert` | Revert вибраного commit'у | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Змінити повідомлення commit'у | `Ctrl+G e` |
| `git_prefix` | `amend` | Змінити вибраний commit | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Змінити автора вибраного commit'у | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (Mixed) до вибраного commit'у | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase на вибрану гілку | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Видалити вибраний ref або drop stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Відкрити меню LFS | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Відкрити меню Flow | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: fetch усіх об'єктів | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Checkout усіх файлів | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: pull об'єктів | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: push усіх об'єктів | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: очистити локальні об'єкти | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Відкрити майстерню переписування історії | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Згорнути чи відновити панель Refs | `Ctrl+N h` |
| `nav_prefix` | `collapse_status` | Згорнути чи відновити панель Status | `Ctrl+N l` |
| `nav_prefix` | `settings` | Відкрити налаштування | `Ctrl+N ,` |
| `nav_prefix` | `about` | Відкрити «Про програму» | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Відкрити Журнал активності | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Відкрити загальний журнал активності | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Показати фонові завдання | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Показати панель оновлення | `Ctrl+N u` |
| `nav_prefix` | `help` | Перемкнути довідку | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Вибрати вкладку | `Ctrl+N t` |
| `nav_prefix` | `change_theme` | Змінити тему | `Ctrl+N y` |
| `nav_prefix` | `change_language` | Змінити мову | `Ctrl+N g` |
| `nav_prefix` | `change_graph_color_theme` | Змінити колірну тему графа | `Ctrl+N Shift+Y` |
| `nav_prefix` | `toggle_graph_row_tints` | Перемкнути підсвітку рядків графа | `Ctrl+N Shift+T` |
| `nav_prefix` | `grow_section` | Збільшити розділ Refs | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Зменшити розділ Refs | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Збільшити блок повідомлення commit'у | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Зменшити блок повідомлення commit'у | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Розширити панель Refs | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Звузити панель Refs | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Розширити панель Status | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Звузити панель Status | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Агент: показати відкладений вигляд | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Агент: назад (або відхилити відкладений вигляд) | `Ctrl+N Backspace` |
| `refs` | `filter` | Фільтр refs за назвою | `/` |
| `refs` | `solo` | Solo для вибраного | `o` |
| `refs` | `clear_solo` | Припинити solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Перемкнути ієрархію гілок | `y` |
| `refs` | `show_all` | Показати всі приховані refs | `Shift+H` |
| `refs` | `reveal` | Показати в графі | `g` |
| `refs` | `mark_drop` | Позначити як джерело перенесення | `m` |
| `refs` | `drop_marked` | Перенести сюди позначене посилання | `Shift+M` |
| `refs` | `expand_all` | Розгорнути всі папки | `=` |
| `refs` | `collapse_all` | Згорнути всі папки | `-` |
| `refs` | `expand_below` | Розгорнути папку з підпапками | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Згорнути папку з підпапками | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Файли: шлях чи дерево | `m` |
| `log` | `view_all_files` | Файли: усі файли | `a` |
| `log` | `compare_selection` | Порівняти вибрані рядки | `v` |
| `log` | `compare_swap` | Поміняти сторони порівняння | `x` |
| `log` | `compare_next` | Перейти до наступного порівнюваного рядка | `n` |
| `log` | `pick_column` | Вибрати стовпець для пересування або зміни ширини | `Shift+C` |
| `log` | `scroll_left` | Прокрутити стовпці ліворуч | `Shift+Left` |
| `log` | `scroll_right` | Прокрутити стовпці праворуч | `Shift+Right` |
| `log` | `copy_mermaid` | Копіювати commits як Mermaid gitGraph | `Shift+Y` |
| `log_columns` | `next` | Вибрати наступний стовпець | `Tab` |
| `log_columns` | `prev` | Вибрати попередній стовпець | `Shift+BackTab` |
| `log_columns` | `move_left` | Пересунути стовпець ліворуч | `<` |
| `log_columns` | `move_right` | Пересунути стовпець праворуч | `>` |
| `log_columns` | `wider` | Розширити стовпець | `]` |
| `log_columns` | `narrower` | Звузити стовпець | `[` |
| `log_columns` | `reset_width` | Скинути ширину стовпців | `0` |
| `log_columns` | `fit` | Підігнати стовпець під вміст | `=` |
| `log_columns` | `fit_all` | Підігнати всі стовпці під вміст | `+` |
| `log_columns` | `layout` | Прокрутка вбік або підгонка під панель | `s` |
| `log_files` | `collapse_all` | Згорнути чи розгорнути всі папки | `z` |
| `log_files` | `filter` | Фільтр змінених файлів | `/` |
| `log_files` | `expand_all` | Розгорнути всі папки | `=` |
| `log_files` | `collapse_all_folders` | Згорнути всі папки | `-` |
| `log_files` | `expand_below` | Розгорнути папку з підпапками | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Згорнути папку з підпапками | `_`, `Shift+Left` |
| `status` | `compose` | Фокус на повідомленні commit'у | `c` |
| `status` | `toggle_file_mode` | Файли: шлях чи дерево | `m` |
| `status` | `view_all_files` | Файли: усі файли | `a` |
| `status` | `amend` | Перемкнути amend | `Shift+A` |
| `status` | `skip_hooks` | Перемкнути пропуск хуків | `Shift+H` |
| `status` | `collapse_all` | Згорнути чи розгорнути всі папки | `z` |
| `status` | `stage_toggle` | Stage чи unstage вибраного рядка | `Ctrl+Space` |
| `status` | `open_diff` | Відкрити diff вибраного файлу | `d` |
| `status` | `expand_all` | Розгорнути всі папки | `=` |
| `status` | `collapse_all_folders` | Згорнути всі папки | `-` |
| `status` | `expand_below` | Розгорнути папку з підпапками | `*`, `Shift+Right` |
| `status` | `collapse_below` | Згорнути папку з підпапками | `_`, `Shift+Left` |
| `home` | `favorite` | Додати репозиторій в обране чи прибрати | `f`, `*` |
| `home` | `close_repo` | Закрити, прибрати чи зняти обране з рядка | `x`, `Delete` |
| `home` | `browse` | Знайти репозиторій (огляд) | `b` |
| `home` | `clone` | Клонувати репозиторій | `c` |
| `home` | `init` | Ініціалізувати репозиторій | `i` |
| `home` | `collapse_all` | Згорнути всі розділи | `-` |
| `home` | `expand_all` | Розгорнути всі розділи | `+`, `=` |
| `home` | `search` | Пошук репозиторіїв | `/` |
| `home` | `workspace_new` | Новий робочий простір | `n` |
| `home` | `workspace_edit` | Змінити робочий простір | `e` |
| `home` | `workspace_from_tabs` | Новий робочий простір з відкритих вкладок | `a` |
| `home` | `workspace_fetch` | Fetch все в робочому просторі | `Shift+F` |
| `home` | `workspace_pull` | Pull все в робочому просторі | `Shift+P` |
| `home` | `workspace_clone_missing` | Клонувати відсутні репозиторії | `m` |
| `home` | `workspace_locate` | Знайти відсутній репозиторій | `l` |
| `refs` | `hosting_filter` | Pull requests / issues: вибрати фільтр | `v` |
| `refs` | `hosting_edit` | Pull request: змінити на хостингу | `e` |
| `refs` | `new_pull_request` | Створити pull request | `n` |
| `refs` | `add_remote` | Remotes: додати remote | `+` |
| `diff` | `close` | Закрити diff | `q` |
| `diff` | `edit` | Змінити в робочій папці | `e` |
| `diff` | `lfs_raw` | LFS: картка чи сирий вказівник | `r` |
| `diff` | `copy` | Копіювати виділене чи рядок | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage чи unstage hunk'а | `Space` |
| `diff` | `stage_lines` | Stage чи unstage вибраних рядків | `Enter` |
| `diff` | `discard_hunk` | Відкинути hunk | `x` |
| `diff` | `discard_lines` | Відкинути вибрані рядки | `Shift+X` |
| `diff` | `select_lines` | Почати чи скинути виділення рядків | `v` |
| `diff` | `next_hunk` | Наступний hunk | `n` |
| `diff` | `prev_hunk` | Попередній hunk | `p` |
| `diff` | `toggle_layout` | Вигляд: єдиний чи поруч | `m` |
| `diff` | `toggle_hunks_only` | Лише hunks чи весь файл | `Shift+H` |
| `diff` | `toggle_minimap` | Перемкнути мінікарту | `Shift+M` |
| `diff` | `choose_encoding` | Вибрати кодування | `Shift+C` |
| `diff` | `toggle_wrap` | Перемкнути перенесення слів | `w` |
| `diff` | `toggle_whitespace` | Показати чи сховати зміни лише пробілів | `s` |
| `diff` | `merge_editor` | Відкрити редактор merge (файл із конфліктом) | `Shift+E` |
| `file_insight` | `close` | Закрити перегляд файлу | `q` |
| `file_insight` | `edit` | Змінити в робочій папці | `e` |
| `file_insight` | `copy` | Копіювати виділене чи рядок | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Перейти до останнього рядка | `Shift+G` |
| `file_insight` | `open_commit` | Перейти до commit'у рядка | `Enter` |
| `history` | `show_in_log` | Показати в Log | `Enter` |
| `history` | `copy_sha` | Копіювати SHA commit'у | `y` |
| `history` | `copy_summary` | Копіювати короткий опис | `Shift+Y` |
| `history` | `show_diff` | Показати diff цього commit'у | `d`, `Alt+1` |
| `history` | `open_file` | Відкрити файл на цьому commit'і | `o`, `Alt+2` |
| `history` | `blame` | Blame на цьому commit'і | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Порівняти з робочим деревом | `c` |
| `history` | `restore` | Відновити цю версію | `Shift+R` |
| `history` | `range_up` | Розширити виділення вгору | `Shift+Up` |
| `history` | `range_down` | Розширити виділення вниз | `Shift+Down` |
| `history` | `switch_focus` | Перемкнутися між списком і diff | `Tab` |
| `history` | `widen_list` | Зробити список ширшим | `Alt+Right` |
| `history` | `narrow_list` | Зробити список вужчим | `Alt+Left` |
| `history` | `reset_split` | Скинути ширину списку | `Alt+0` |
| `commit_column` | `scroll_left` | Прокрутити стовпець ліворуч | `Shift+Left` |
| `commit_column` | `scroll_right` | Прокрутити стовпець праворуч | `Shift+Right` |
| `commit_column` | `next_part` | Вибрати наступну частину | `Tab` |
| `commit_column` | `move_left` | Пересунути частину ліворуч | `<` |
| `commit_column` | `move_right` | Пересунути частину праворуч | `>` |
| `commit_column` | `wider_part` | Розширити частину | `]` |
| `commit_column` | `narrower_part` | Звузити частину | `[` |
| `commit_column` | `reset_widths` | Скинути ширину частин | `0` |
| `file_insight` | `blame_card` | Blame: показати або сховати картку відомостей | `i` |
| `file_insight` | `blame_next` | Blame: наступний hunk | `n` |
| `file_insight` | `blame_prev` | Blame: попередній hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: наступний hunk того ж commit'у | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: попередній hunk того ж commit'у | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: перейти до рядка | `g` |
| `file_insight` | `blame_copy_sha` | Blame: копіювати SHA commit'у | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: показати diff commit'у для файлу | `d` |
| `file_insight` | `blame_parent` | Blame: Blame на батьківському commit'і | `b` |
| `file_insight` | `blame_history` | Blame: відкрити історію файлу | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: зробити колонку анотацій ширшою | `>` |
| `file_insight` | `blame_narrower` | Blame: зробити колонку анотацій вужчою | `<` |
| `image` | `zoom_in` | Збільшити зображення | `+`, `=` |
| `image` | `zoom_out` | Зменшити зображення | `-` |
| `image` | `fit` | Вписати зображення у вікно | `0` |
| `image` | `actual_size` | Показати зображення в реальному розмірі | `1` |
| `image` | `flip_side` | Показати іншу сторону (до / після) | `Tab` |
| `image` | `widen_before` | Розширити панель «До» | `Alt+Right` |
| `image` | `widen_after` | Розширити панель «Після» | `Alt+Left` |
| `image` | `reset_split` | Скинути поділ до / після | `Alt+0` |
| `image` | `edit_in` | Відкрити за допомогою... | `Shift+O` |
| `image` | `edit_default` | Відкрити в застосунку за замовчуванням | `o` |
| `editor` | `save` | Зберегти | `Ctrl+S` |
| `editor` | `undo` | Скасувати | `Ctrl+Z` |
| `editor` | `redo` | Повторити | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Знайти | `Ctrl+F` |
| `editor` | `find_next` | Знайти далі (Shift: назад) | `F3` |
| `editor` | `paste` | Вставити | `Ctrl+V` |
| `editor` | `toggle_minimap` | Перемкнути мінікарту | `Alt+M` |
| `editor_prompt` | `save` | Незбережені зміни: зберегти й закрити | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Незбережені зміни: відкинути й закрити | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Незбережені зміни: продовжити редагування | `c`, `Shift+C` |
| `merge` | `next_conflict` | Наступний конфлікт | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Попередній конфлікт | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Наступний файл із конфліктом | `Alt+PageDown` |
| `merge` | `prev_file` | Попередній файл із конфліктом | `Alt+PageUp` |
| `merge` | `next_pane` | Фокус на наступну панель | `F6` |
| `merge` | `prev_pane` | Фокус на попередню панель | `Shift+F6` |
| `merge` | `toggle_panes` | Показати чи сховати бічні панелі (Refs і Status) | `F9` |
| `merge` | `toggle_base` | Показати чи сховати панель «Основа» | `Alt+D` |
| `merge` | `toggle_full_file` | Перемкнути diff всього файлу | `Alt+F` |
| `merge` | `undo` | Скасувати (вибір і введення) | `Ctrl+Z` |
| `merge` | `redo` | Повторити (вибір і введення) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Розширити панель «Поточне» (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Розширити панель «Вхідне» (B) | `Alt+Left` |
| `merge` | `reset_panes` | Скинути розміри панелей | `Alt+0` |
| `merge` | `save` | Зберегти без позначки «вирішено» | `Alt+S` |
| `merge` | `save_resolve` | Зберегти й позначити вирішеним | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Запропонувати вирішення (ШІ) | `Alt+I` |
| `merge` | `unpick` | Скасувати вибір цього конфлікту (знову не вирішено) | `Alt+U` |
| `merge` | `reset_all` | Скинути весь файл до початкових конфліктів | `Alt+R` |
| `merge` | `focus_a` | Фокус на панель «Поточне» (A) | `Alt+1` |
| `merge` | `focus_b` | Фокус на панель «Вхідне» (B) | `Alt+2` |
| `merge` | `focus_output` | Фокус на панель результату | `Alt+3` |
| `merge` | `focus_base` | Фокус на панель «Основа» | `Alt+4` |
| `merge_sides` | `toggle_line` | Вибрати чи скинути рядок | `Space` |
| `merge_sides` | `toggle_chunk` | Вибрати чи скинути весь блок | `Enter` |
| `merge_sides` | `take_a` | Лише «Поточне» (A) | `a` |
| `merge_sides` | `take_b` | Лише «Вхідне» (B) | `b` |
| `merge_sides` | `take_both` | Обидві: A, потім B | `c` |
| `merge_sides` | `take_both_reversed` | Обидві: B, потім A | `Shift+C` |
| `merge_sides` | `clear` | Нічого не брати для цього конфлікту | `x` |
| `merge_sides` | `unpick` | Скасувати вибір цього конфлікту (знову не вирішено) | `u` |
| `merge_sides` | `reset_all` | Скинути весь файл до початкових конфліктів | `Shift+U` |
| `merge_sides` | `swap_order` | Поміняти порядок двох сторін | `s` |
| `merge_sides` | `next_conflict` | Наступний конфлікт | `n` |
| `merge_sides` | `prev_conflict` | Попередній конфлікт | `Shift+N` |
| `merge_sides` | `take_all_a` | Взяти все з панелі «Поточне» (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Взяти все з панелі «Вхідне» (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Показати чи сховати панель «Основа» | `d` |
| `merge_sides` | `scroll_left` | Прокрутити панелі вліво | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Прокрутити панелі вправо | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Перемкнути diff всього файлу | `f` |
| `merge_sides` | `cycle_height` | Змінити висоту результату по колу | `z` |
| `merge_card` | `take_a` | Залишити A («Поточне») як є | `a` |
| `merge_card` | `take_b` | Взяти B («Вхідне») як є | `b` |
| `merge_card` | `take_base` | Взяти основу (спільного предка) | `o` |
| `merge_card` | `keep` | Залишити наявний файл | `k` |
| `merge_card` | `delete` | Прийняти видалення | `Shift+D` |
| `merge_card` | `external_tool` | Відкрити в зовнішньому інструменті merge | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (зупинитися для amend) | `e` |
| `rebase` | `autosquash` | Перемкнути autosquash | `a` |
| `rebase` | `start` | Запустити план | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Скинути план | `Ctrl+R` |
| `rebase` | `split` | Розділити commit рядка | `t` |
| `rebase` | `move_up` | Перемістити рядок угору | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Перемістити рядок униз | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Створити commits | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Перемкнути вибір рядків | `v` |
| `split` | `move_up` | Перемістити commit угору | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Перемістити commit униз | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Скинути розділення | `Shift+R` |
| `rewrite` | `preview` | Переглянути переписування | `F5` |
| `rewrite` | `apply` | Застосувати переписування | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Скасувати перегляд | `Ctrl+X` |
| `rewrite` | `add_rule` | Додати правило | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Увімкнути або вимкнути правило | `Space` |
| `rewrite` | `move_up` | Перемістити правило вгору | `Alt+Up` |
| `rewrite` | `move_down` | Перемістити правило вниз | `Alt+Down` |
| `rewrite` | `delete_rule` | Видалити правило | `d`, `Delete` |
| `rewrite` | `restore_rule` | Відновити видалене правило | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Дублювати правило | `Ctrl+D` |
| `rewrite` | `next_tab` | Наступна вкладка перегляду | `Alt+Right` |
| `rewrite` | `prev_tab` | Попередня вкладка перегляду | `Alt+Left` |
| `rewrite` | `save_shared` | Зберегти рецепт у репозиторії | `Alt+S` |
| `rewrite` | `save_mine` | Зберегти рецепт лише для себе | `Alt+M` |
| `rewrite` | `load` | Відкрити збережений рецепт | `Alt+L` |
| `rewrite` | `name` | Перейменувати рецепт | `F2` |
| `rewrite` | `protected_all` | Вибрати всі захищені гілки | `Alt+A` |
| `rewrite` | `protected_none` | Зняти вибір з усіх захищених гілок | `Alt+U` |
| `rewrite` | `check_tools` | Ще раз перевірити інструменти | `F6` |
| `rewrite` | `analyze` | Проаналізувати розміри історії | `F7` |
| `rewrite` | `host_howto` | Пояснити захист гілок на хості | `Alt+H` |
| `rewrite` | `ai_rules` | Створити правила переписування за допомогою ШІ | `Alt+I` |
| `reword` | `apply` | Застосувати нове повідомлення | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Змінити історію глибше | `Alt+H` |
| `composer` | `commit` | Commit із композера | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Вставити посилання на issue | `Alt+R` |
| `dialog` | `browse` | Вибрати папку (огляд) | `Ctrl+B` |
| `dialog` | `sources` | Перейти до джерел клонування | `Alt+S` |
| `dialog` | `submit` | Надіслати форму pull request | `Ctrl+S` |
| `dialog` | `web` | Відкрити веб-форму провайдера | `Alt+W` |
| `folder_picker` | `choose` | Вибрати папку | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Нова папка | `Alt+N` |
| `folder_picker` | `filter` | Фільтрувати елементи папки | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Показати чи сховати сусідні папки | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Показати чи сховати приховані папки | `Alt+H` |
| `preferences` | `search` | Пошук налаштувань | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Додати елемент списку | `Insert` |
| `preferences` | `list_duplicate` | Дублювати вибраний елемент списку | `Ctrl+D` |
| `preferences` | `list_remove` | Видалити вибраний елемент списку | `Delete` |
| `preferences` | `list_move_up` | Перемістити вибраний елемент списку вгору | `Alt+Up` |
| `preferences` | `list_move_down` | Перемістити вибраний елемент списку вниз | `Alt+Down` |
| `preferences` | `prev_tab` | Попередня вкладка сторінки налаштувань | `Alt+Left` |
| `preferences` | `next_tab` | Наступна вкладка сторінки налаштувань | `Alt+Right` |
| `ai_popup` | `copy` | Копіювати результат | `c` |
| `activity_log` | `scroll_left` | Прокрутити Журнал активності ліворуч | `Left` |
| `activity_log` | `scroll_right` | Прокрутити Журнал активності праворуч | `Right` |
| `integrations` | `add` | Додати профіль | `a` |
| `integrations` | `edit` | Змінити вибраний профіль | `e` |
| `integrations` | `open_token` | Відкрити сторінку токена | `o` |
| `integrations` | `remove` | Видалити вибраний профіль | `d`, `Delete` |
| `prompt` | `ai_stash_message` | ШІ: згенерувати повідомлення stash'у | `Ctrl+G` |
<!-- keymap:end -->

### Клавіші, яких термінал не може надіслати, і протокол клавіатури kitty

Застарілий термінал не може надіслати деякі комбінації: `Ctrl+,`, `Ctrl+Shift+N`,
`Ctrl+Enter` і `Ctrl+<цифра>`. Він не надсилає нічого або надсилає іншу клавішу: наприклад,
`Ctrl+Shift+N` приходить як `Ctrl+N`, `Ctrl+I` як `Tab`, а `Ctrl+M` як `Enter`. Сторінка
Налаштувань попереджає про таке прив'язування («цей термінал не надсилає ...»).

Термінали з **протоколом клавіатури kitty** можуть надсилати ці комбінації. Приклади:
kitty, WezTerm, Ghostty, foot, Alacritty 0.13 і новіші, а також tmux чи herdr, коли вони
пропускають протокол. GitPoulpi типово використовує протокол:

- Під час запуску GitPoulpi питає термінал і чекає на відповідь щонайбільше близько 100 мс.
  Якщо термінал не відповідає, залишаються застарілі клавіші.
- Якщо термінал відповідає, GitPoulpi вмикає протокол. Він вимикає його знову, коли
  завершується, завершується з помилкою або аварійно.
- Попередження на сторінці Налаштувань залежать від протоколу: коли його ввімкнено,
  наведені вище комбінації можна надіслати.
- **Налаштування > Клавіатура й миша > Клавіатурний протокол Kitty** (`ui.kitty_keyboard`),
  встановлений на `off`, ніколи не змінює режим клавіатури терміналу. Скористайтеся цим,
  якщо термінал чи мультиплексор поводиться неправильно. Потрібен перезапуск.

Консолі Windows самі повідомляють про модифікатори й не потребують протоколу.

## Палітра команд (`Ctrl+P`)

Палітра перелічує **усі** команди застосунку, тож дає доступ до всього без скорочень. Повний
список [нижче](#усі-команди-палітри).

| Клавіша | Дія |
|---|---|
| `Ctrl+P` | Відкрити палітру |
| *(введення)* | Відфільтрувати список. Спершу йде точний збіг після префікса `Категорія: `, потім збіг будь-де в назві, потім нечіткий збіг |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J`, `Home` / `End`, `PageUp` / `PageDown` | Перемістити підсвічування (див. [Спливні списки з фільтром](#спливні-списки-з-фільтром)) |
| `Enter` або подвійне клацання | Виконати підсвічену команду |
| `Esc` | Закрити |

`j` і `k` вводяться у фільтр. Вони не рухають підсвічування.

Кожен рядок показує своє скорочення праворуч. Команда, що зараз не застосовна (наприклад,
не вибрано жодного ref), залишається у списку сірою, а причина показується під списком. Назви
починаються з категорії, як-от `Навігація:`, `Refs:`, `Git:`, `Вигляд:`, `Diff:` чи
`Застосунок:`. Введіть категорію, щоб побачити всю її групу.

### Клавіша для будь-якої команди

Кожен рядок палітри закінчується кнопкою клавіші (`[k]`), і `Alt+K` робить те саме для виділеного рядка: відкриває **Налаштування > Клавіатура й миша**, відфільтровані за скороченням цієї команди, готові до перепризначення. Команда без власного скорочення отримує його тут: вона міститься в розділі **Команди палітри**, за замовчуванням без клавіші, а призначена вами клавіша запускає команду так само, як її вибір у палітрі (команда, яку зараз не можна виконати, повідомляє причину). Після цього рядок показує вашу клавішу.

У `config.toml` це таблиця `[keybindings.palette]`, названа за командою (`toggle_minimap = "Ctrl+Alt+M"`). Клавіша, що конфліктує з іншою дією, показана червоним, а `c` на її рядку перелічує всі дії, з якими вона конфліктує.

Рядок підказки під списком називає цю клавішу (за замовчуванням `Alt+K`). Це дія `palette.configure_shortcut`, тож її можна перепризначити, як будь-яку іншу.

### Усі команди палітри

Згенеровано із застосунку (тест підтримує список актуальним). Команди ШІ з'являються лише
тоді, коли ШІ ввімкнено; решта стають сірими, з причиною, коли не застосовні. Клавіші
типові.

<!-- palette:begin -->
| Команда | Типова клавіша |
|---|---|
| Навігація: фокус на Refs | `Ctrl+N <badge>` |
| Навігація: фокус на Log | `Ctrl+N <badge>` |
| Навігація: фокус на Status | `Ctrl+N <badge>` |
| Навігація: фокус на фільтр Refs | `Ctrl+N <badge>` |
| Навігація: фокус на Refs > Local | `Ctrl+N <badge>` |
| Навігація: фокус на Refs > Remote | `Ctrl+N <badge>` |
| Навігація: фокус на Refs > Worktrees | `Ctrl+N <badge>` |
| Навігація: фокус на Refs > Submodules | `Ctrl+N <badge>` |
| Навігація: фокус на Refs > Stashes | `Ctrl+N <badge>` |
| Навігація: фокус на Refs > Tags | `Ctrl+N <badge>` |
| Ліва панель: показати або сховати розділи... |  |
| Ліва панель: типові розділи |  |
| Ліва панель: використати ці розділи для всіх репозиторіїв |  |
| Ліва панель: показати або сховати локальні гілки |  |
| Ліва панель: показати або сховати remote |  |
| Ліва панель: показати або сховати worktree |  |
| Ліва панель: показати або сховати Stashes |  |
| Ліва панель: показати або сховати Tags |  |
| Ліва панель: показати або сховати Pull requests |  |
| Ліва панель: показати або сховати issues |  |
| Ліва панель: показати або сховати підмодулі |  |
| Навігація: фокус на Status > Unstaged | `Ctrl+N <badge>` |
| Навігація: фокус на Status > Staged | `Ctrl+N <badge>` |
| Навігація: фокус на наступну панель | `Tab` |
| Навігація: фокус на попередню панель | `Shift+Tab` |
| Навігація: пошук commit'ів... | `f` |
| Навігація: згорнути/розгорнути Refs | `Ctrl+N h` |
| Навігація: Розширити панель Refs | `Ctrl+N Right` |
| Навігація: Звузити панель Refs | `Ctrl+N Left` |
| Навігація: Розширити панель Status | `Ctrl+N Shift+Left` |
| Навігація: Звузити панель Status | `Ctrl+N Shift+Right` |
| Навігація: згорнути/розгорнути Status | `Ctrl+N l` |
| Навігація: збільшити розділ Refs | `Ctrl+N +` |
| Навігація: зменшити розділ Refs | `Ctrl+N -` |
| Навігація: скинути розміри розділів Refs |  |
| Навігація: скинути розкладку |  |
| Деталі commit'у: збільшити повідомлення | `Ctrl+N ]` |
| Деталі commit'у: зменшити повідомлення | `Ctrl+N [` |
| Деталі commit'у: скинути розмір повідомлення |  |
| Деталі commit'у: прокрутити повідомлення вниз |  |
| Деталі commit'у: прокрутити повідомлення вгору |  |
| Файли: сортування файлів commit'у (А-Я / Я-А) |  |
| Refs: checkout вибраного ref | `подвійне клацання` |
| Refs: копіювати назву ref |  |
| Refs: копіювати повний ref (refs/heads/...) |  |
| Refs: копіювати SHA цілі ref |  |
| Refs: merge вибраної гілки в поточну |  |
| Refs: rebase поточної гілки на вибрану | `Ctrl+G r` |
| Refs: перенести вибраний ref на поточну гілку... |  |
| Refs: Позначити як джерело перенесення | `m` |
| Refs: Перенести сюди позначене посилання... | `M` |
| Refs: перейменувати вибрану гілку... |  |
| Refs: fast-forward вибраної гілки до поточної |  |
| Refs: fast-forward поточної гілки до вибраної |  |
| Refs: створити worktree з вибраної гілки чи tag'а... |  |
| Refs: Reflog вибраної гілки... |  |
| Refs: створити pull request з вибраної гілки... |  |
| Refs: відкрити worktree у вкладці | `Enter / подвійне клацання` |
| Refs: оновити підмодуль (вибраний) |  |
| Refs: ініціалізувати підмодуль (вибраний) |  |
| Refs: відкрити підмодуль у вкладці | `Enter / подвійне клацання` |
| Git: оновити всі підмодулі |  |
| Git: створити worktree на вибраному commit'і... |  |
| Git: створити патч із вибраного commit'у... |  |
| Git: створити патч від вибраного commit'у до HEAD... |  |
| Git: створити патч зі змін робочого дерева... |  |
| Переписування: очистити стару історію... |  |
| Git: застосувати патч... |  |
| Переписування: майстерня історії | `Ctrl+G H` |
| Переписування: замінити автора... |  |
| Переписування: анонімізувати авторів... |  |
| Переписування: застосувати .mailmap до історії... |  |
| Переписування: Merge осіб (.mailmap)... |  |
| Переписування: зсунути дати commit'ів... |  |
| Переписування: змінити часовий пояс commit'ів... |  |
| Переписування: обмежити commits дозволеними годинами... |  |
| Переписування: дата комітера = дата автора |  |
| Переписування: пошук і заміна в повідомленнях... |  |
| Переписування: видалити рядки повідомлень... |  |
| Переписування: додати або видалити трейлери... |  |
| Переписування: прибрати підписи... |  |
| Переписування: виправити кодування повідомлень... |  |
| Переписування: пошук і заміна у файлах... |  |
| Переписування: прибрати файли з історії... |  |
| Переписування: прибрати великі файли з історії... |  |
| Переписування: перемістити або перейменувати шляхи... |  |
| Переписування: перенести файли в LFS... |  |
| Переписування: повернути файли з LFS... |  |
| Переписування: перетворити кінці рядків... |  |
| Переписування: прибрати біт виконання... |  |
| Переписування: перетворити кодування файлів... |  |
| Переписування: запустити скрипт... |  |
| ✦ Переписування: написати скрипт за допомогою ШІ... |  |
| ✦ Переписування: створити правила за допомогою ШІ... |  |
| Переписування: від вибраного commit'у до HEAD... |  |
| Переписування: вибрані commits... |  |
| LFS: ініціалізувати для цього репозиторію |  |
| LFS: відстежувати шаблон... |  |
| LFS: не відстежувати шаблон... |  |
| LFS: fetch об'єктів |  |
| LFS: відкрити меню | `Ctrl+G l` |
| LFS: fetch усіх об'єктів | `Ctrl+G F` |
| LFS: Checkout усіх файлів | `Ctrl+G K` |
| LFS: push усіх об'єктів | `Ctrl+G J` |
| LFS: pull об'єктів | `Ctrl+G L` |
| LFS: push об'єктів |  |
| LFS: очистити локальні об'єкти | `Ctrl+G V` |
| LFS: показати блокування |  |
| LFS: відкрити сторінку встановлення |  |
| LFS: сирий вказівник у diff | `r` |
| LFS: відстежувати вибраний файл |  |
| LFS: відстежувати розширення вибраного файлу |  |
| LFS: заблокувати вибраний файл |  |
| LFS: розблокувати вибраний файл |  |
| LFS: примусово розблокувати вибраний файл |  |
| Refs: видалити вибрану гілку чи tag... | `Ctrl+G d` |
| Refs: видалити вибрану гілку з її remote... |  |
| Refs: видалити вибрану гілку локально й на remote... |  |
| Refs: копіювати посилання на вибрану гілку в remote |  |
| Git: копіювати посилання на вибраний commit у remote |  |
| Git: анотований tag на вибраному commit'і... |  |
| Refs: push вибраного tag'а на типовий remote |  |
| Refs: видалити вибраний tag з його remote... |  |
| Git: Fetch вибраного tag'а |  |
| Refs: анотувати вибраний tag... |  |
| Refs: сховати/показати в графі |  |
| Refs: показати в графі | `g` |
| Refs: solo для вибраного ref | `o` |
| Refs: припинити solo (скинути всі) | `O` |
| Refs: показати всі приховані ref | `H` |
| Refs: сховати/показати папку чи remote |  |
| Refs: solo для папки чи remote | `o` |
| Refs: фільтр за назвою... | `/` |
| Refs: скинути фільтр за назвою | `Esc` |
| Refs: розгорнути всі папки | `=` |
| Refs: згорнути всі папки | `-` |
| Refs: розгорнути вибрану папку з підпапками | `*` |
| Refs: згорнути вибрану папку з підпапками | `_` |
| Refs: наступний розділ | `l` |
| Refs: попередній розділ | `h` |
| Refs: згорнути/розгорнути активний розділ | `Enter` |
| Refs: ієрархія гілок | `y` |
| Git: stage вибраний файл | `Ctrl+G s` |
| Git: unstage вибраний файл | `Ctrl+G u` |
| Git: stage всі зміни | `Ctrl+G a` |
| Зміни: відкинути вибраний файл... |  |
| Зміни: ігнорувати вибраний файл |  |
| Зміни: ігнорувати розширення вибраного файлу |  |
| Зміни: вважати вибраний файл двійковим... |  |
| Зміни: вважати вибраний файл текстовим... |  |
| Зміни: вважати розширення файлу двійковим... |  |
| Зміни: вважати розширення файлу текстовим... |  |
| Зміни: stash вибраного файлу |  |
| Зміни: історія вибраного файлу |  |
| Зміни: blame вибраного файлу |  |
| Зміни: відкрити файл у зовнішньому diff-інструменті |  |
| Зміни: відкрити файл у зовнішньому редакторі |  |
| Зміни: відкрити файл у типовій програмі |  |
| Зміни: показати файл у папці |  |
| Зміни: копіювати шлях вибраного файлу |  |
| Зміни: створити патч із вибраного файлу... |  |
| Зміни: змінити вибраний файл |  |
| Зміни: видалити вибраний файл... |  |
| Зміни: меню вибраної папки |  |
| Зміни: створити файл |  |
| Зміни: відкинути всі зміни... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: зробити commit змін зі stage зараз | `Ctrl+G Enter` |
| Git: змінити повідомлення WIP-commit'у (у Log) | `c у Log` |
| Git: Fetch усіх remote | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (fast-forward, якщо можливо) |  |
| Git: Pull (лише fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: параметри Pull... |  |
| Git: Push | `Ctrl+G P` |
| Git: Push на remote... |  |
| Git: force push (із lease)... |  |
| Git: задати upstream... |  |
| Remote: додати remote... |  |
| Remote: видалити remote... |  |
| Remote: змінити remote... |  |
| Remote: задати типовий remote... |  |
| Remote: копіювати назву remote |  |
| Remote: копіювати посилання на remote |  |
| Remote: fetch remote... |  |
| Remote: дії для вибраного remote... | `.` |
| Репозиторій: розріджений checkout... |  |
| Repository: SSH і облікові дані... |  |
| Repository: перевірити з'єднання |  |
| Repository: використати типові облікові дані |  |
| Flow: відкрити меню | `Ctrl+G W` |
| Flow: почати гілку... |  |
| Flow: нове issue... |  |
| Flow: підхопити гілку хоста... |  |
| Flow: завершити поточну гілку... |  |
| Flow: синхронізувати з батьківською... |  |
| Flow: просунути... |  |
| Flow: задати батьківську... |  |
| Flow: продовжити або перервати зупинену дію... |  |
| Flow: перемкнути групування гілок |  |
| Flow: налаштування робочого процесу... |  |
| Remote: приховати remote у графі... |  |
| Remote: показати remote у графі... |  |
| Git: нова гілка на HEAD... | `Ctrl+G b` |
| Git: новий tag на HEAD... | `Ctrl+G t` |
| Git: push усіх ненадісланих tag'ів | `Ctrl+G U` |
| Git: stash усіх змін | `Ctrl+G T` |
| Git: stash усіх змін з повідомленням... | `Ctrl+G m` |
| Git: Stash pop | `Ctrl+G o` |
| Git: Stash apply |  |
| Git: Stash drop... |  |
| Git: cherry-pick вибраного commit'у на HEAD | `Ctrl+G y` |
| Git: checkout вибраного commit'у (detached HEAD) |  |
| Git: створити гілку на вибраному commit'і... |  |
| Git: створити tag на вибраному commit'і... |  |
| Commit: копіювати повідомлення |  |
| Commit: копіювати ім'я автора |  |
| Commit: копіювати email автора |  |
| Commit: копіювати дату |  |
| Commit: копіювати короткий SHA |  |
| Commit: копіювати повний SHA |  |
| Git: revert вибраного commit'у... | `Ctrl+G v` |
| Git: змінити повідомлення commit'у... | `Ctrl+G e` |
| Commit: змінити... | `Ctrl+G E` |
| Commit: змінити автора... | `Ctrl+G A` |
| Commit: змінити заголовок... |  |
| Commit: змінити опис... |  |
| Git: розділити вибраний commit... |  |
| Git: інтерактивний rebase від вибраного commit'у... |  |
| Rebase: продовжити |  |
| Rebase: пропустити |  |
| Rebase: перервати |  |
| git am: продовжити |  |
| git am: пропустити цей патч |  |
| git am: перервати |  |
| Репозиторій: видалити застарілий index.lock |  |
| Банер: pop відкладеного auto-stash |  |
| Банер: завершити відкладений auto-stash (залишити результат) |  |
| Банер: показати відкладений auto-stash |  |
| Конфлікти: показати передбачені конфлікти |  |
| Банер: drop відкладеного auto-stash... |  |
| Банер: почати спочатку... |  |
| Git: reset гілки до вибраного commit'у (soft - залишити індекс і дерево) |  |
| Git: reset гілки до вибраного commit'у (mixed - залишити дерево) | `Ctrl+G R` |
| Git: reset гілки до вибраного commit'у (hard - відкинути все)... |  |
| Git: вирішити конфлікт через ours (ця гілка) |  |
| Git: вирішити конфлікт через theirs (вхідне) |  |
| Git: вирішити конфлікт через основу (спільний предок) |  |
| Конфлікти: відкрити редактор merge | `Enter на файлі з конфліктом` |
| Merge: наступний конфлікт | `F7` |
| Merge: попередній конфлікт | `Shift+F7` |
| Merge: наступний файл із конфліктом | `Alt+PageDown` |
| Merge: попередній файл із конфліктом | `Alt+PageUp` |
| Merge: взяти блок з A |  |
| Merge: взяти блок з B |  |
| Merge: взяти «Поточне» для цього конфлікту | `a` |
| Merge: взяти «Вхідне» для цього конфлікту | `b` |
| Merge: взяти обидві сторони, A, потім B | `c` |
| Merge: взяти обидві сторони, B, потім A | `C` |
| Merge: взяти основу для цього конфлікту |  |
| Merge: очистити цей конфлікт | `x` |
| Merge: поміняти порядок | `s` |
| Merge: взяти все з «Поточне» | `A` |
| Merge: взяти все з «Вхідне» | `B` |
| Merge: змінити конфлікт у результаті |  |
| Merge: показати/сховати бічні панелі | `F9` |
| Merge: прокрутити панелі вліво | `Shift+Left` |
| Merge: прокрутити панелі вправо | `Shift+Right` |
| Merge: показати/сховати панель «Основа» | `Alt+D` |
| Merge: diff по всьому файлу | `Alt+F` |
| Merge: скасувати | `Ctrl+Z` |
| Merge: повторити | `Ctrl+Y` |
| Merge: зберегти без позначки «вирішено» | `Alt+S` |
| Merge: зберегти й позначити вирішеним | `Ctrl+S` |
| Merge: скинути результат до результату merge | `Alt+R` |
| Merge: змінити висоту результату по колу | `z` |
| Merge: розширити панель «Поточне» | `Alt+Right` |
| Merge: розширити панель «Вхідне» | `Alt+Left` |
| Merge: скинути розміри панелей | `Alt+0` |
| Merge: скасувати вибір конфлікту | `Alt+U` |
| Merge: фокус на панель «Поточне» (A) | `Alt+1` |
| Merge: фокус на панель «Вхідне» (B) | `Alt+2` |
| Merge: фокус на панель результату | `Alt+3` |
| Merge: фокус на панель «Основа» | `Alt+4` |
| Merge: до лівої панелі (той самий конфлікт) |  |
| Merge: до правої панелі (той самий конфлікт) |  |
| Merge: фокус на наступну панель | `F6` |
| Merge: фокус на попередню панель | `Shift+F6` |
| Merge: продовжити з файлу на диску |  |
| Merge: почати заново з merge |  |
| Merge: скасувати вирішення й відкрити |  |
| Merge: перезавантажити файл |  |
| Merge: взяти файл «Поточне» як є | `a` |
| Merge: взяти файл «Вхідне» як є | `b` |
| Merge: взяти файл «Основа» як є | `o` |
| Merge: залишити наявний файл | `k` |
| Merge: прийняти видалення | `D` |
| Merge: відкрити в зовнішньому інструменті | `m` |
| ✦ ШІ: запропонувати вирішення конфлікту merge | `Alt+I` |
| Merge: закрити редактор |  |
| Git: перервати merge, cherry-pick чи revert |  |
| Git: пропустити cherry-pick чи revert |  |
| Репозиторій: відкрити чи перейти (керування)... | `Ctrl+O` |
| Репозиторій: огляд репозиторію... | `b (вкладка керування)` |
| Репозиторій: створити новий (Init)... | `i (вкладка керування)` |
| Репозиторії: відкрити вибраний | `Enter` |
| Репозиторії: додати/прибрати з обраного | `f` |
| Репозиторії: закрити чи прибрати вибраний | `x` |
| Репозиторії: меню вибраного репозиторію... | `.` |
| Репозиторії: пошук | `/ / Ctrl+F` |
| Репозиторії: згорнути всі розділи | `-` |
| Репозиторії: розгорнути всі розділи | `+` |
| Репозиторії: закрити всі вкладки... |  |
| Репозиторії: прибрати вибраний зі списку |  |
| Репозиторії: прибрати всі нещодавні |  |
| Робочий простір: новий... | `n` |
| Робочий простір: новий з відкритих вкладок... | `a` |
| Робочий простір: перемкнути... | `Alt+W` |
| Робочий простір: змінити... | `e` |
| Робочий простір: закрити (без робочого простору) |  |
| Робочий простір: керування... | `Alt+J` |
| Робочий простір: закрити вкладку |  |
| Робочий простір: знайти відсутній репозиторій... | `l` |
| Робочий простір: видалити... |  |
| Робочий простір: Fetch все | `F` |
| Робочий простір: Pull все | `P` |
| Робочий простір: клонувати відсутні | `m` |
| Робочий простір: додати вкладку до робочого простору |  |
| Робочий простір: прибрати вкладку з робочого простору |  |
| Radar: відкрити | `Alt+R` |
| Radar: оновити | `r` |
| Radar: пошук | `/` |
| Radar: відкрити в браузері | `o` |
| Radar: відкрити репозиторій | `Enter` |
| Radar: checkout гілки | `c` |
| Radar: створити worktree... | `t` |
| Radar: показати в репозиторії | `s` |
| Radar: почати роботу над issue... | `n` |
| Radar: закріпити або відкріпити | `p` |
| Radar: відкласти... | `z` |
| Radar: рев'ю запиту... | `e` |
| Radar: рев'ю через API |  |
| Radar: клонувати й зробити рев'ю... |  |
| Radar: змінити виконавців... |  |
| Radar: змінити рев'юерів... |  |
| Radar: змінити мітки... |  |
| Radar: задати віху... |  |
| Radar: вибрати область | `w` |
| Radar: вибрати вигляд | `v` |
| Radar: вибрати фільтр | `f` |
| Radar: показувати неклоновані запити | `a` |
| Radar: показувати чернетки | `d` |
| Radar: увімкнути рев'ю запитів усього облікового запису... | `T` |
| Radar: меню рядка... | `.` |
| Radar: вибрати стовпець | `C` |
| Radar: розширити стовпець | `]` |
| Radar: звузити стовпець | `[` |
| Radar: скинути ширину стовпців | `0` |
| Radar: назад на головну | `Esc` |
| Radar: закрити вкладку |  |
| Вкладка: нова вкладка | `Alt+T` |
| Вкладка: нова... |  |
| Вкладка: меню дій... | `Ctrl+T` |
| Вкладка: закрити інші вкладки... |  |
| Вкладка: закрити вкладки праворуч... |  |
| Вкладка: додати/прибрати з обраного |  |
| Вкладка: копіювати шлях репозиторію |  |
| Вкладка: відкрити у файловому менеджері |  |
| Вкладка: відкрити в терміналі |  |
| Вкладка: відкрити в... | `Alt+O` |
| Вкладка: відкрити в типовому | `Ctrl+Alt+O` |
| Репозиторій: змінити профіль... |  |
| Довідка: про GitPoulpi | `Ctrl+N a` |
| Довідка: ліцензії та подяки |  |
| Вигляд: журнал активності | `Ctrl+N A` |
| Вигляд: загальний журнал активності | `Ctrl+N G` |
| Довідка: нотатки до випуску |  |
| Оновлення: показати панель оновлення | `Ctrl+N u` |
| Оновлення: відкрити сповіщення безпеки |  |
| Оновлення: скопіювати команду оновлення |  |
| Оновлення: виконати команду оновлення в терміналі |  |
| Оновлення: перевірити зараз |  |
| Оновлення: завантажити |  |
| Оновлення: оновити зараз |  |
| Оновлення: перезапустити й оновити |  |
| Оновлення: повернути попередню версію |  |
| Оновлення: пропустити цю версію |  |
| Нотатки до випуску: закрити вкладку |  |
| Завдання: показати фонові завдання | `Ctrl+N J` |
| Завдання: скасувати все, що можна скасувати |  |
| Вигляд: копіювати діагностику |  |
| Журнал активності: показати або сховати рівень Помилка |  |
| Журнал активності: показати або сховати рівень Попередження |  |
| Журнал активності: показати або сховати рівень Інфо |  |
| Журнал активності: показати або сховати рівень Налагодження |  |
| Журнал активності: показати або сховати рівень Трасування |  |
| Журнал активності: показати всі рівні |  |
| Журнал активності: не показувати жодного рівня |  |
| Журнал активності: очистити журнал застосунку |  |
| Журнал активності: вибрати область |  |
| Журнал активності: операції git |  |
| Журнал активності: журнал застосунку |  |
| Журнал активності: прокрутити ліворуч | `Left` |
| Журнал активності: прокрутити праворуч | `Right` |
| Журнал активності: копіювати останню помилку |  |
| Журнал активності: виконати запропоновану дію |  |
| Журнал активності: відкрити останню помилку |  |
| Вкладка: вийти й cd сюди |  |
| Репозиторій: клонувати... | `Ctrl+Shift+N` |
| Репозиторій: наступна вкладка | `Ctrl+PageDown` |
| Репозиторій: попередня вкладка | `Ctrl+PageUp` |
| Вкладка: перемістити ліворуч |  |
| Вкладка: перемістити праворуч |  |
| Репозиторій: закрити вкладку | `Ctrl+W` |
| Репозиторій: вибрати вкладку... | `t` |
| Пошук: робочий простір... | `Ctrl+Alt+F` |
| Правка: скасувати | `Ctrl+Z` |
| Правка: повторити | `Ctrl+Y` |
| Правка: забути заблокований запис скасування |  |
| Правка: історія скасувань... | `Alt+Z` |
| Правка: Reflog... |  |
| Правка: відновити втрачені commit'и... |  |
| Правка: відновити гілку туди, де її залишила заблокована дія | `Alt+Shift+Z` |
| Рев'ю: показати огляд |  |
| Рев'ю: показати файли |  |
| Рев'ю: показати commits |  |
| Рев'ю: коментувати рядок... | `c` |
| Рев'ю: коментувати бесіду... | `C` |
| Рев'ю: відповісти в обговоренні... | `r` |
| Рев'ю: вирішити або знову відкрити обговорення | `x` |
| Рев'ю: згорнути або розгорнути обговорення | `z` |
| Рев'ю: змінити незавершений коментар... | `e` |
| Рев'ю: видалити незавершений коментар | `d` |
| Рев'ю: позначити файл переглянутим | `v` |
| Рев'ю: наступний файл | `]` |
| Рев'ю: попередній файл | `[` |
| Рев'ю: завершити рев'ю... | `f` |
| Рев'ю: відкинути незавершене рев'ю... |  |
| Рев'ю: оновити | `F5` |
| Рев'ю: відкрити в браузері | `o` |
| Рев'ю: checkout гілки | `b` |
| Рев'ю: закрити | `Esc` |
| Рев'ю: змінити виконавців... | `a` |
| Рев'ю: змінити рев'юерів... | `p` |
| Рев'ю: змінити мітки... | `l` |
| Рев'ю: задати віху... | `m` |
| Рев'ю: вибрати набір патчів... | `P` |
| Рев'ю: порівняти набори патчів... | `D` |
| Рев'ю: влити зміну... | `U` |
| Рев'ю: покинути зміну... | `Q` |
| Рев'ю: перемкнути вбудований або розділений вигляд | `V` |
| Рев'ю: перемкнути весь файл або лише hunk'и | `H` |
| Рев'ю: перемкнути зміни лише пробілів | `S` |
| Рев'ю: перемкнути перенесення слів | `W` |
| Рев'ю: наступний hunk | `J` |
| Рев'ю: попередній hunk | `K` |
| Рев'ю: вибрати кодування... | `E` |
| Рев'ю: перемкнути мінікарту | `M` |
| Рев'ю: прокрутити diff ліворуч | `Left` |
| Рев'ю: прокрутити diff праворуч | `Right` |
| Рев'ю: копіювати виділений текст або рядок | `Ctrl+C` |
| Дошка: перемістити картку до... | `m` |
| Дошка: перемістити картку ліворуч |  |
| Дошка: перемістити картку праворуч |  |
| Дошка: відкрити картку | `Enter` |
| Дошка: змінити виконавців... |  |
| Дошка: змінити мітки... |  |
| Дошка: задати віху... |  |
| Дошка: нове issue в цій колонці | `n` |
| Дошка: фільтр за виконавцем... | `a` |
| Дошка: фільтр за міткою... | `l` |
| Дошка: фільтр за віхою... | `M` |
| Дошка: фільтр за текстом | `/` |
| Дошка: очистити фільтри | `x` |
| Дошка: задати мітки колонок... | `c` |
| Дошка: оновити | `r` |
| Дошка: закрити | `Esc` |
| Рев'ю: ✦ чернетка рев'ю... | `s` |
| Рев'ю: прийняти пропозицію | `y` |
| Рев'ю: змінити й прийняти пропозицію... | `w` |
| Рев'ю: відхилити пропозицію | `n` |
| Рев'ю: прийняти всі пропозиції | `A` |
| Рев'ю: відхилити всі пропозиції | `N` |
| Рев'ю: використати підсумок ШІ в тексті рев'ю | `i` |
| Рев'ю: ✦ використати чернетку... | `u` |
| Рев'ю: змінити текст ✦ чернетки... | `t` |
| Рев'ю: відкинути ✦ чернетку | `N` |
| Історія скасувань: скасувати до цього місця... | `u` |
| Історія скасувань: повторити до цього місця... | `y` |
| Історія скасувань: відновити лише це... | `r` |
| Історія скасувань: відновити гілку туди, де вона була... | `o` |
| Історія скасувань: оновити |  |
| Історія скасувань: закрити | `Esc` |
| Reflog: показати в Log | `s` |
| Reflog: створити гілку тут... | `b` |
| Reflog: Create Tag Here... | `t` |
| Reflog: Apply Changes to Working Tree | `w` |
| Reflog: Save Changes as Stash | `z` |
| Reflog: відновити на цю позицію... | `r` |
| Reflog: завантажити ще | `m` |
| Reflog: оновити |  |
| Втрачена робота: створити гілку тут... | `b` |
| Lost Work: Create Tag Here... | `t` |
| Втрачена робота: cherry-pick | `c` |
| Lost Work: Apply Changes to Working Tree | `w` |
| Lost Work: Save Changes as Stash | `z` |
| Втрачена робота: сканувати знову |  |
| Вигляд: оновити | `r` |
| Вигляд: оновити (fetch, pull requests, issues) | `F5` |
| Вигляд: довідка | `?` |
| Вигляд: стовпець Ref |  |
| Вигляд: стовпець Graph |  |
| Вигляд: стовпець Message |  |
| Вигляд: стовпець Author |  |
| Вигляд: стовпець Date |  |
| Вигляд: стовпець Changes |  |
| Вигляд: стовпець SHA |  |
| Вигляд: стовпець Log: Вибрати стовпець для пересування або зміни ширини | `C` |
| Вигляд: стовпець Log: Вибрати наступний стовпець | `Tab` |
| Вигляд: стовпець Log: Вибрати попередній стовпець | `Shift+BackTab` |
| Вигляд: стовпець Log: Пересунути стовпець ліворуч | `<` |
| Вигляд: стовпець Log: Пересунути стовпець праворуч | `>` |
| Вигляд: стовпець Log: Розширити стовпець | `]` |
| Вигляд: стовпець Log: Звузити стовпець | `[` |
| Вигляд: стовпець Log: Скинути ширину стовпців | `0` |
| Вигляд: стовпець Log: Підігнати стовпець під вміст | `=` |
| Вигляд: стовпець Log: Підігнати всі стовпці під вміст | `+` |
| Вигляд: Log: Прокрутити стовпці ліворуч | `Shift+Left` |
| Вигляд: Log: Прокрутити стовпці праворуч | `Shift+Right` |
| Вигляд: Log: Перемкнути прокрутку стовпців убік | `s` |
| Вигляд: Log: Вибрати стовпці для показу |  |
| Diff: відкрити diff вибраного файлу | `Enter` |
| Diff: закрити diff | `Esc` |
| Diff: наступний hunk | `n` |
| Diff: попередній hunk | `p` |
| Diff: stage hunk під курсором | `Space` |
| Diff: unstage hunk під курсором | `Space` |
| Diff: вибрати рядки (або скинути вибір) | `v` |
| Diff: stage вибрані рядки | `Enter` |
| Diff: unstage вибрані рядки | `Enter` |
| Diff: відкинути hunk під курсором... | `x` |
| Diff: відкинути вибрані рядки... | `X` |
| Копіювати: виділений текст (спершу виділіть) | `Ctrl+C` |
| Копіювати: рядок під курсором (або вибрані) | `y` |
| Diff: копіювати hunk під курсором |  |
| Diff: розкладка Inline (+/- в одному стовпці) | `m` |
| Diff: розкладка Split (до і після) | `m` |
| Diff: лише hunks (весь файл / лише зміни) | `H` |
| Файли: шлях/дерево | `m` |
| Файли: усі файли | `a` |
| Commit: пропуск хуків | `H` |
| Файли: відкрити меню рядка або виділення | `.` |
| Файли: розгорнути всі папки | `=` |
| Файли: згорнути всі папки | `-` |
| Файли: розгорнути вибрану папку з підпапками | `*` |
| Файли: згорнути вибрану папку з підпапками | `_` |
| Файли: виділити все |  |
| Файли: зняти виділення |  |
| Файли: перемкнути виділення поточного рядка |  |
| Log: копіювати як Mermaid gitGraph... | `Y` |
| Log: порівняти вибране | `v` |
| Log: поміняти сторони порівняння | `x` |
| Log: до наступного порівнюваного рядка | `n` |
| Diff: перенесення слів | `w` |
| Diff: вибрати кодування... | `C` |
| Вигляд: показати чи сховати мінікарту | `M` |
| Вигляд: показати чи сховати стовпець commit'ів | `Alt+C` |
| Вигляд: стовпець commit'ів: сховати |  |
| Вигляд: стовпець commit'ів: історія файлу |  |
| Вигляд: стовпець commit'ів: усі commit'и |  |
| Вигляд: перейти до стовпця commit'ів |  |
| Вигляд: розширити стовпець commit'ів |  |
| Вигляд: звузити стовпець commit'ів |  |
| Вигляд: стовпець commit'ів: прокрутити ліворуч | `Shift+Left` |
| Вигляд: стовпець commit'ів: прокрутити праворуч | `Shift+Right` |
| Вигляд: стовпець commit'ів: вибрати наступну частину | `Tab` |
| Вигляд: стовпець commit'ів: пересунути частину ліворуч | `<` |
| Вигляд: стовпець commit'ів: пересунути частину праворуч | `>` |
| Вигляд: стовпець commit'ів: розширити частину | `]` |
| Вигляд: стовпець commit'ів: звузити частину | `[` |
| Вигляд: стовпець commit'ів: скинути ширину частин | `0` |
| Вигляд: стовпець commit'ів: меню вибраного рядка | `.` |
| Вигляд: стовпець commit'ів: вибрати стовпці |  |
| Вигляд: стовпець commit'ів: перемкнути стовпець Ref |  |
| Вигляд: стовпець commit'ів: перемкнути стовпець Граф |  |
| Вигляд: стовпець commit'ів: перемкнути стовпець Автор |  |
| Вигляд: стовпець commit'ів: перемкнути стовпець Дата |  |
| Вигляд: стовпець commit'ів: перемкнути стовпець Зміни |  |
| Вигляд: стовпець commit'ів: перемкнути стовпець SHA |  |
| Вигляд: стовпець commit'ів: показати вибраний рядок у Log |  |
| Вигляд: стовпець commit'ів: показати зміни файлу у вибраному рядку |  |
| Вигляд: стовпець commit'ів: показати файл таким, яким він був у вибраному рядку |  |
| Вигляд: стовпець commit'ів: Blame файлу у вибраному рядку |  |
| Редактор: перемкнути стеження за вибором |  |
| Зображення: збільшити | `+` |
| Зображення: зменшити | `-` |
| Зображення: вписати у вікно | `0` |
| Зображення: реальний розмір (1:1) | `1` |
| Зображення: перемкнути до / після | `Tab` |
| Зображення: розширити панель «До» | `Alt+Right` |
| Зображення: розширити панель «Після» | `Alt+Left` |
| Зображення: скинути розміри панелей | `Alt+0` |
| Зображення: редагувати в... | `O` |
| Зображення: редагувати у типовому редакторі | `o` |
| Історія файлу: показати в Log | `Enter` |
| Історія файлу: копіювати SHA commit'у | `y` |
| Історія файлу: копіювати короткий опис | `Y` |
| Історія файлу: показати diff цього commit'у | `d` |
| Історія файлу: відкрити файл на цьому commit'і | `o` |
| Історія файлу: Blame на цьому commit'і | `b` |
| Історія файлу: порівняти з робочим деревом | `c` |
| Історія файлу: відновити цю версію | `R` |
| Історія файлу: розширити виділення вгору | `Shift+Up` |
| Історія файлу: розширити виділення вниз | `Shift+Down` |
| Історія файлу: перемкнутися між списком і diff | `Tab` |
| Історія файлу: зробити список ширшим | `Alt+Right` |
| Історія файлу: зробити список вужчим | `Alt+Left` |
| Історія файлу: скинути ширину списку | `Alt+0` |
| Файл: відкрити за допомогою... | `O` |
| Файл: відкрити у програмі за замовчуванням | `o` |
| Diff: зміни лише пробілів | `s` |
| ✦ ШІ: згенерувати повідомлення commit'у | `Ctrl+G i` |
| ✦ ШІ: згенерувати повідомлення stash |  |
| ✦ ШІ: вставити змінну запиту |  |
| ✦ ШІ: змінити профіль... |  |
| ✦ ШІ: відкрити агента... |  |
| ✦ ШІ: пояснити вибраний commit | `Ctrl+G w` |
| ✦ ШІ: розділити вибраний commit... |  |
| ✦ ШІ: пояснити робочі зміни | `Ctrl+G w` |
| ✦ ШІ: скласти commit'и з робочих змін... | `Ctrl+G C` |
| Композер: перемістити файл до commit'у... | `m` |
| Diff: Blame |  |
| Blame: показати commit у Log | `Enter` |
| Blame: копіювати SHA commit'у | `Y` |
| Blame: показати diff commit'у для файлу | `d` |
| Blame: Blame на батьківському commit'і | `b` |
| Blame: відкрити історію файлу | `H` |
| Blame: показати або сховати картку відомостей | `i` |
| Blame: наступний hunk | `n` |
| Blame: попередній hunk | `p` |
| Blame: наступний hunk того ж commit'у | `N` |
| Blame: попередній hunk того ж commit'у | `P` |
| Blame: перейти до рядка | `g` |
| Blame: зробити колонку анотацій ширшою | `>` |
| Blame: зробити колонку анотацій вужчою | `<` |
| Diff: історія файлу |  |
| Diff: перегляд файлу |  |
| Файл: змінити в робочій папці | `e` |
| Редактор: змінити кодування |  |
| Редактор: відкрити заново в кодуванні |  |
| Редактор: перетворити на кодування |  |
| Редактор: перетворити розриви рядків |  |
| Редактор: змінити відступи |  |
| Застосунок: налаштування... | `Ctrl+, / Ctrl+N ,` |
| Застосунок: пошук налаштувань... | `/ у налаштуваннях` |
| Застосунок: скорочення... |  |
| Агент: показати відкладений вигляд | `Ctrl+N Enter` |
| Агент: відхилити відкладений вигляд |  |
| Агент: назад | `Ctrl+N Backspace` |
| Скорочення: скинути все... |  |
| Інтеграції: відкрити пов'язаний pull request | `Ctrl+G g` |
| Інтеграції: змінити пов'язаний pull request на хостингу | `e (на рядку)` |
| Інтеграції: копіювати URL пов'язаного pull request |  |
| Інтеграції: оновити pull requests і issues |  |
| Інтеграції: checkout pull request... |  |
| Інтеграції: відкрити посилання commit'у (#issue / !request)... |  |
| Інтеграції: відкрити issue... |  |
| Хостинг: рев'ю запиту... |  |
| Хостинг: Push на рев'ю... |  |
| Хостинг: змінити виконавців... |  |
| Хостинг: змінити рев'юерів... |  |
| Хостинг: змінити мітки... |  |
| Хостинг: задати віху... |  |
| Локальні issues: закрити issue... |  |
| Локальні issues: відкрити issue знову... |  |
| Локальні issues: відкрити в Obsidian... |  |
| Issues: відкрити дошку |  |
| Локальні issues: створити Obsidian Base |  |
| Pull requests: створити pull request... | `n` |
| Хостинг: нове issue... | `n` |
| Pull requests: пошук | `Enter на рядку пошуку` |
| Pull requests: фільтр (відкриті / merged / усі)... | `v` |
| Pull requests: провайдер... | `Enter на рядку провайдера` |
| Pull requests: вибрати репозиторій... | `Enter на рядку репозиторію` |
| Pull requests: відв'язати джерело (немає) |  |
| Issues: провайдер... | `Enter на рядку провайдера` |
| Issues: відв'язати джерело (немає) |  |
| Інтеграції: відв'язати профіль (автоматично) |  |
| Issues: вибрати репозиторій... | `Enter на рядку репозиторію` |
| Issues: пошук | `Enter на рядку пошуку` |
| Issues: вибрати фільтр... | `v` |
| Issues: змінити запит / списки... |  |
| Commit: вставити посилання на issue... |  |
| Інтеграції: профіль для цього репозиторію... |  |
| Інтеграції: профіль цього репозиторію |  |
| Інтеграції: підключити інтеграцію... |  |
| Налаштування: параметри Radar профілю... |  |
| Інтеграції: переглянути перевірку безпеки |  |
| Застосунок: вийти | `q` |
| Застосунок: тема... |  |
| Конфлікти: редагувати списки базових гілок... |  |
| Конфлікти: базові гілки цього репозиторію... |  |
| Застосунок: змінити тему... | `Ctrl+N y` |
| Застосунок: змінити мову... | `Ctrl+N g` |
| Лаунчер: створити ярлик |  |
| Лаунчер: створити ярлик на робочому столі |  |
| Лаунчер: видалити ярлики |  |
| Лаунчер: створити псевдонім команди |  |
| Лаунчер: видалити псевдонім команди |  |
| Лаунчер: додати до PATH |  |
| Лаунчер: прибрати з PATH |  |
| Застосунок: змінити колірну тему графа... | `Ctrl+N Y` |
| Застосунок: перемкнути підсвітку рядків графа | `Ctrl+N T` |
<!-- palette:end -->

## Пересування

| Клавіша | Дія |
|---|---|
| `Tab` / `Shift+Tab` | Фокус на наступну / попередню панель |
| `Ctrl+N`, потім значок | Фокус на цю частину вікна (див. [Префікс навігації](#префікс-навігації-ctrln)) |
| `j` / `k`, `Up` / `Down` | Перемістити вибір в усіх панелях |
| `PageUp` / `PageDown`, `Home` / `End` | Перейти на сторінку; на перший / останній рядок |
| число, потім `j` / `k` / `Up` / `Down` | Префікс-лічильник: `4j` зсуває на чотири рядки. З `PageUp` / `PageDown` лічильник означає кількість сторінок. Рядок стану показує набраний лічильник (`4…`). `Esc` або будь-яка інша клавіша скидає його. Початковий `0` ніколи не починає лічильник |
| `h` / `l`, `Left` / `Right` | Панель Refs: перемкнути розділ. Log: вийти (`h`) зі списку змінених файлів commit'у або увійти (`l`) в нього |
| `?` | Відкрити чи закрити довідку (див. [Довідка в застосунку](#довідка-в-застосунку-)) |
| `r` | Оновити |
| `F5` | Виконати fetch для всіх remote, перезавантажити pull requests та issues і оновити |
| `q` | Вийти |
| `Ctrl+Z` | **Скасувати** останню операцію. Див. [Скасування й повторення](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Повторити**. `Ctrl+Shift+Z` потребує протоколу клавіатури kitty; `Ctrl+Y` працює завжди |
| `.`, `Menu`, `Shift+F10` | Відкрити контекстне меню вибраного рядка |

`Ctrl+Z` і `Ctrl+Y` нічого не роблять, поки клавіатуру займає текстове поле. Редактор має
власне скасування тексту.

## Спливні списки з фільтром

Усі спливні списки з фільтром використовують однакові клавіші: палітра команд і її
списки вибору (тема, мова, кольори графа), вибір вкладки і кожен випадаючий список із рядком
пошуку (варіанти в Налаштуваннях, форма pull request, вибір посилання на issue, список
**Відкрити в**). Введіть текст, щоб відфільтрувати список. Ці клавіші рухають підсвічування:

| Клавіша | Дія |
|---|---|
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Перейти на один рядок |
| `Home` / `End` | Перший / останній рядок |
| `PageUp` / `PageDown` | Перейти на одну видиму сторінку |
| `Enter` | Вибрати підсвічений рядок |
| `Esc` | Закрити |

У цих вікнах `Home`, `End` і `Ctrl+K` рухають список, а не курсор поля фільтра. `Ctrl+A` і
`Ctrl+E` досі переносять курсор на початок і в кінець. Звичайні `j` і `k` вводяться у фільтр.

Мишею:

- Коліщатко **прокручує** список. Воно не рухає підсвічування.
- Клацання **виділяє** рядок. Подвійне клацання **вибирає** його, як `Enter`.
- Список, вищий за своє вікно, показує смугу прокручування на правому краї. Її можна
  перетягувати.
- Короткий випадаючий список без рядка пошуку (два-три варіанти) вибирає з першого клацання.

**Попередній перегляд наживо.** Списки вибору теми, колірної теми графа й мови застосовують підсвічений
рядок одразу, позаду вікна. `Enter` залишає його. `Esc` або клацання поза вікном повертає
попереднє значення.

**Фільтр тем.** У списку вибору теми під полем фільтра є рядок: **Усі**, **Темні**, **Світлі** та
**Кольорові**. `Tab` / `Shift+Tab` переходять до наступного / попереднього. `Alt+1` ...
`Alt+4` вибирають один одразу. На них можна також клацнути.

## Довідка в застосунку (`?`)

`?`, `Ctrl+N ?` або команда палітри `Вигляд: довідка` відкриває довідку. Вона перелічує
клавіші та жести миші кожної частини застосунку. Вона відкривається поверх будь-чого,
зокрема й екрана Налаштувань.

| Клавіша | Дія |
|---|---|
| `?` | Закрити довідку (і очистити фільтр) |
| *(введення)* | Фільтр: залишає лише розділи, що містять текст. `j`, `k` і `q` теж вводяться |
| `Backspace`, `Ctrl+U` | Видалити останній символ / очистити фільтр |
| `Esc` | Очистити фільтр; якщо він порожній, закрити довідку |
| `Up` / `Down`, `Ctrl+P` / `Ctrl+N`, `Ctrl+K` / `Ctrl+J` | Прокрутити на один рядок |
| `PageUp` / `PageDown`, `Home` / `End` | Прокрутити на сторінку; на початок / у кінець |
| `Ctrl+D` або клацання по посиланню | Відкрити цю документацію на GitHub у браузері |

Коліщатко прокручує довідку, а її смугу прокручування можна перетягувати. Клацання поза довідкою
закриває її.

## Префікс навігації (`Ctrl+N`)

`Ctrl+N` чекає ще на одну клавішу. Рядок стану показує клавіші, які він приймає. Наступна
клавіша виконує дію або скасовує.

| Клавіша | Дія |
|---|---|
| `Ctrl+N`, потім значок | Фокус на частину вікна з цим значком |
| `Ctrl+N h` / `Ctrl+N l` | Згорнути чи відновити панель Refs / праву панель |
| `Ctrl+N Left` / `Ctrl+N Right` | Зсунути роздільник між панеллю Refs і Log на крок вліво / вправо (згорнута панель Refs спершу повертається) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Зсунути роздільник між Log і правою панеллю на крок вліво / вправо |
| `Ctrl+N +` / `Ctrl+N -` | Збільшити / зменшити активний розділ Refs на один рядок |
| `Ctrl+N ]` / `Ctrl+N [` | Збільшити / зменшити блок повідомлення commit'у правої панелі на один рядок |
| `Ctrl+N t` | Вибрати вкладку (список вибору вкладок; `t` сама робить те саме) |
| `Ctrl+N y` / `Ctrl+N g` / `Ctrl+N Shift+Y` | Змінити тему / мову / колірну тему графа |
| `Ctrl+N Shift+T` | Увімкнути чи вимкнути відтінки рядків графа |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Показати вигляд, підготовлений агентом / повернутися (див. [MCP та агенти](mcp-and-agents.md)) |
| `Ctrl+N ,` | Відкрити Налаштування |
| `Ctrl+N ?` | Відкрити чи закрити довідку |
| `Esc` або будь-яка інша клавіша | Скасувати |

### Значки

Поки префікс чекає, кожна частина вікна, що може отримати фокус, показує односимвольний
значок на лівій верхній межі. Натисніть цей символ, щоб сфокусуватися на ній. Значки
видаються щоразу наново, у порядку на екрані: спершу `1`-`9`, потім `a`, `b`, `c`... Тож
читайте значок на екрані: частина не завжди отримує той самий значок.

Частини зі значком: фільтр Refs і кожен видимий розділ Refs, Log (або подання Diff), Перегляд файлу, коли він відкритий, списки Unstaged і Staged, поле заголовка композера commit'у
та елементи керування списку файлів commit'у. На вкладці Головна значки мають поле пошуку
й чотири розділи та кнопки панелі інструментів (значок кнопки натискає її).
На вкладці Radar це поле фільтра, випадні списки «Область», «Вигляд» і «Показати» (значок
списку відкриває його), кнопки «Оновити» і «Назад» та кожен заголовок розділу на екрані.
Значок розділу переносить виділення на його заголовок; далі `Enter` або `Space` згортає розділ.

У вибірнику папок Огляд та в діалогах Клонувати, Init, Створити worktree, Робочий простір, Розріджений checkout, Flow, Нова issue та форма pull request значки стоять на полях, прапорцях і кнопках (значок кнопки натискає її, значок поля ставить у нього курсор; у списку один значок стоїть на першому рядку; вибірник папок Огляд має по одному значку для списків швидкого доступу й папок, біля першого рядка кожного списку, і натискання значка ставить клавіатуру в цей список, не вибираючи рядка). У Журналі активності це дві вкладки, поля «Область» і «Рівень», фільтр, прапорці рівнів, кнопки та перший запис списку. Значок робить те саме, що й клацання на цьому елементі. У вибірнику папок `Ctrl+N` запускає значки, тому клавіша «Нова папка» там `Alt+N`.

У вікні Змінити (і Змінити повідомлення commit'у) значки стоять на заголовку, описі, імені, email і даті автора, на **Змінити історію глибше...**, **Скасувати**, **Застосувати** та хрестику закриття; у майстерні переписування історії - на полі області, її прапорцях і кнопці ref'ів, назві рецепта, кожному правилі, його прапорці й кнопках, кнопці **+ Додати правило**, полях вибраного правила, вкладках попереднього перегляду, полі фільтра, рядках захищених гілок із Вибрати все / Зняти все та кнопках Попередній перегляд, Застосувати, Скасувати, Зберегти, Завантажити й Створити правила за допомогою ШІ (сторінки Секрети, Особи, Розмір і результату додають свої кнопки та рядки; рядки вкладки Коміти, бувши списком, їх не мають). Значок робить те саме, що й клацання. Нічого не позначається, поки відкрито випадний список, питання, поле, у яке друкують, вибірник папок, екран очищення, редактор скрипту, діалоги ШІ, blob diff чи довідку про хост, а також поки вікно Змінити питає «відхилити?»; `Esc` прибирає значки, нічого не закриваючи.

У Налаштуваннях значки стоять біля поля пошуку, списку сторінок (один значок, на поточній сторінці), вкладених вкладок, списку та його кнопок, кожного поля сторінки на екрані й хрестика закриття. Значок робить те саме, що й клацання на цьому елементі: текстове поле відкривається для редагування, прапорець перемикається, випадаючий список відкривається, кнопка виконується. `Ctrl+N` зберігає власне значення, поки в полі вводиться текст або відкрито випадаючий список, редактор чи пошук.

У палітри є ті самі дії, наприклад `Навігація: фокус на Refs > Local` чи
`Навігація: фокус на Status > Staged`.

## Репозиторії та вкладки

| Клавіша | Дія |
|---|---|
| `Ctrl+O` або клацання по `⌂` | Перейти на вкладку Головна (на вкладці Головна: фокус на полі пошуку; з відкритим Radar чи відкритими нотатками до випуску: спершу закрити його) |
| `Ctrl+Shift+N` | Відкрити вкладку Головна з діалогом клонування |
| `Ctrl+PageDown` / `Ctrl+PageUp` | Наступна / попередня вкладка |
| `t` | Вибрати вкладку зі списку з фільтром |
| `Ctrl+W` | Закрити поточну вкладку (спершу питає; репозиторій не змінюється) |
| `Ctrl+T` або праве клацання по вкладці | Меню вкладки (його пункти перелічено в розділі [Рядок вкладок](interface-tour.md#рядок-вкладок)) |
| `Alt+O` / `Ctrl+Alt+O` | Відкрити папку вкладки: вибрати зі списку **Відкрити в** / використати типовий пункт. Див. [Відкрити в](open-in.md) |

### Вкладка Головна

Вкладка `⌂` керує вашими репозиторіями. Вона має чотири розділи: **Відкриті репозиторії**,
**Обране**, **Нещодавні репозиторії** та **Усі репозиторії**. Див.
[Огляд інтерфейсу](interface-tour.md#головна).

| Клавіша | Дія |
|---|---|
| `j` / `k`, стрілки, `PageUp` / `PageDown`, `Home` / `End` | Рухатися |
| `Enter` або подвійне клацання | Відкрити репозиторій (або перейти на його вкладку). На заголовку розділу: згорнути чи розгорнути його |
| `f` або `*`, чи клацання по зірці | Додати в обране / прибрати з обраного |
| `x`, `Delete` або кнопка рядка | Закрити його вкладку (Відкриті), прибрати зі списку (Нещодавні), прибрати зірку (Обране), забути його (Усі). Нічого на диску не видаляється |
| `.`, праве клацання або `⋮` рядка | Меню рядка |
| `/` або `Ctrl+F` | Шукати в усіх розділах за назвою, гілкою чи шляхом. `Enter` відкриває перший збіг; `Esc` очищає пошук, а потім залишає поле |
| `Space`, `Left` / `Right` | Згорнути / розгорнути поточний розділ |
| `-` / `+` | Згорнути все / Розгорнути все |
| `b` / `c` / `i` | Огляд (відкрити папку) / Клонувати / Init |
| `Esc` | Очистити пошук або повернутися на вкладку активного репозиторію |
| `q` | Вийти |

`Ctrl+P`, `Ctrl+,`, `?` і `Ctrl+N` працюють на вкладці Головна. `Ctrl+G`, `Ctrl+W` і
`Ctrl+T` потребують вкладки репозиторію: на вкладці Головна вони лише показують
повідомлення. Команди палітри, що потребують репозиторію, стають сірими.

### Вибір папки

**Огляд** (і кнопка **Огляд...** діалогів Init, клонування та створення worktree) відкриває
вікно вибору папки. Угорі в нього поле шляху, ліворуч місця й дерево папок, праворуч вміст поточної
папки.

| Клавіша | Дія |
|---|---|
| `Tab` / `Shift+Tab` | Поле шляху, потім ліва колонка, потім вміст |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Рухатися у сфокусованій колонці. У полі шляху: рухатися за доповненнями |
| літера (вміст) | Перейти до наступної папки, що починається з цієї літери |
| `Enter` / `Right` | Відкрити вибрану папку. У полі шляху: перейти до введеної папки; якщо вона вже показана, вибрати її |
| `Backspace` / `Left` / `Alt+Up` | Батьківська папка (у полі шляху `Backspace` видаляє символ) |
| `Tab` (поле шляху) | Прийняти єдине або підсвічене доповнення |
| `Alt+N` | Нова папка (`Ctrl+N` запускає значки, див. [Значки](#значки)) |
| `Alt+H` | Показати чи сховати приховані папки |
| `Ctrl+F` або `/` поза полем шляху | Фільтрувати вміст папки під час введення (без урахування регістру; `..` лишається; заголовок рахує «x з n»; `Enter` переходить до першого збігу, `Esc` очищає; фільтр очищується, коли ви змінюєте папку). Також доступний через `Tab` або клацання |
| `Ctrl+S`, `Ctrl+Enter` або кнопка підтвердження | Вибрати вибрану підпапку, а якщо її немає, показану папку |
| `Esc` | Закрити доповнення або скасувати |

Коли поле шляху показує новий шлях, його виділено повністю: перша введена клавіша
замінює його.

### Діалоги Init і клонування

| Клавіша | Дія |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Наступне / попереднє поле чи кнопка |
| `Enter` | Натиснути сфокусовану кнопку або створити / клонувати |
| `Space` | Перемкнути сфокусований прапорець або натиснути сфокусовану кнопку |
| `Ctrl+B` | Вибрати папку через огляд |
| `j` / `k`, `g` / `G` | У списку джерел і списку репозиторіїв діалогу клонування: вниз / вгору, перший / останній (не в полі пошуку) |
| `Esc` | Закрити. Під час клонування клонування триває у фоні |

У діалозі клонування ліва колонка перелічує **Клонувати за URL** і по одному джерелу на
кожен підключений профіль хостингу (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Виберіть
джерело клавішами `Up` / `Down` або клацанням. Потім вводьте в полі пошуку **Репозиторій**,
рухайтеся списком стрілками, `PageUp` / `PageDown` або коліщатком і вибирайте рядок клавішею
`Enter` чи клацанням. Про поля див. [Початок роботи](getting-started.md).

## Панель Refs

| Клавіша | Дія |
|---|---|
| `Enter` / `Space` | Розгорнути чи згорнути активний розділ або вибрану папку |
| `h` / `l`, `Left` / `Right` | Перемкнути розділ |
| `/` | Фільтрувати за назвою (`Enter` застосовує, `Esc` скасовує) |
| `y` | Увімкнути чи вимкнути ієрархічний вигляд гілок: папки за `/` у назвах гілок (LOCAL і REMOTE) |
| `o` | **Solo** вибраного ref або кожного ref під вибраною папкою чи remote. Позначте solo кілька ref, щоб побачити їх об'єднання |
| `Shift+O` | **Зняти solo**: зняти кожне solo |
| `Shift+H` | **Показати все**: знову показати кожен прихований ref |
| `g` | **Показати в графі**: прокрутити Log до commit'у вибраного ref (чи stash) і блиснути його рядком; вибір не переміщується |
| `m` | **Позначити** вибраний ref як джерело перенесення (`Esc` або повторне `m` на ньому знімає позначку) |
| `Shift+M` | **Перенести позначений ref** на вибраний: відкриває меню, яке відкриває перетягування (Merge, Rebase, Push, Fast-forward...) |
| `+` (розділ REMOTE) | Додати remote |
| `.`, `Menu`, `Shift+F10` або праве клацання | Меню рядка: гілки, tag, імені remote або папки |
| `Enter` на рядку WORKTREES | Відкрити цей worktree в окремій вкладці |
| `Ctrl+G d` | Видалити вибрану гілку чи tag (спершу питає). На рядку REMOTE: видалити цю віддалену гілку |

Що роблять меню, див. у розділі [Робота з Log](log-and-history.md).

### Pull requests та issues

За наявності відповідної інтеграції панель Refs має розділ pull requests (GitLab: merge
requests). Із підключеним профілем хостингу вона має також розділ ISSUES. Див.
[Хостинги й трекери issues](hosting-integration.md#5-pull-requests-та-issues-на-бічній-панелі).

| Клавіша | Дія |
|---|---|
| `Enter` | Відкрити вибраний рядок у браузері. На рядку `Підключити знову`: відкрити Налаштування на цьому профілі |
| `Space` | Розгорнути чи згорнути розділ |
| `v` | Відкрити фільтр розділу. Pull requests: Відкриті, Merged / закриті, Усі. Issues: Відкриті, Призначені мені, Закриті, Усі |
| `n` | Розділ pull requests: створити pull request. Розділ ISSUES: нове issue. (Кнопка `+` заголовка) |
| `e` | Розділ pull requests: редагувати вибраний запит на хостингу (олівець рядка) |
| `Enter` на рядку пошуку | Вводити в полі пошуку. `Enter`, `Tab` або `Esc` залишає його; `Ctrl+U` очищає |
| `Enter` на заголовку групи | Згорнути чи розгорнути групу (*Мої pull requests*, *Призначені мені*, *Чекають мого рев'ю*, *Усі pull requests*) |
| `Enter` на рядку фільтра, провайдера чи репозиторію | Відкрити цей випадаючий список. Список провайдерів закінчується пунктом **Немає (відв'язати цей репозиторій)** |
| `.` або праве клацання | Меню рядка: відкрити в браузері, копіювати URL чи номер, перейти на вихідну гілку (pull requests); створити гілку з issue (issues) |

### Дошка issues

**Дошка** в заголовку ISSUES (палітра: **Issues: відкрити дошку**) відкриває дошку Kanban джерела issues. Див.
[дошку](hosting-integration.md#5b-дошка-issues) і [локальні issues](local-issues.md#дошка-kanban). Кожна клавіша
нижче також є командою палітри (**Дошка: ...**).

| Клавіша | Дія |
|---|---|
| Стрілки | Вибрати картку (`Вгору`, `Вниз`) або сусідню колонку (`Вліво`, `Вправо`); `Home`, `End`, `PageUp`, `PageDown` переходять по колонці |
| `Shift+Вліво`, `Shift+Вправо` | Перемістити вибрану картку до попередньої чи наступної колонки |
| `m` | **Перемістити картку до...**: випадаючий список колонок |
| `Enter` | Відкрити issue (нотатку - для локального issue; інакше сторінку хоста) |
| `e`, `.` | Змінити виконавців, мітки чи віху; меню картки |
| `n` | Нове issue у вибраній колонці |
| `a`, `l`, `M`, `/` | Фільтр за виконавцем, міткою, віхою, текстом. `x` очищає фільтри |
| `c` | Мітки, що є колонками дошки GitHub / GitLab |
| `r` | Прочитати issues знову |
| `Esc` | Закрити дошку |

Миша: перетягніть картку на іншу колонку, щоб перемістити її, клацніть картку, щоб вибрати, подвійне клацання відкриває її, праве клацання - меню картки,
коліщатко прокручує колонку, `Shift`+коліщатко (або смуга під дошкою) прокручує колонки.

### Flow (робочі процеси гілок)

Коли для репозиторію налаштовано робочий процес (див. [робочі процеси гілок](branching-workflows.md)),
на панелі інструментів є кнопка **Flow**, а бічна панель групує LOCAL і REMOTE за типом гілки. Кожен
рядок меню - це команда палітри (**Flow: ...**; **Flow: почати Feature...** і **Flow: нове issue типу Feature...**
створюються для кожного типу гілки робочого процесу). Рядки під керуванням хоста (*Нове issue типу <Тип>*,
*Підхопити гілку, створену хостом*), їхні діалоги, картку очікування та картку «з'явилася гілка» описано в
[робочих процесах гілок](branching-workflows.md#потоки-під-керуванням-хоста-в-застосунку);
`Enter` виконує кнопку з фокусом (*Створити issue*, *Підхопити*), `Esc` припиняє очікування або закриває.

| Клавіша | Дія |
|---|---|
| `Ctrl+G Shift+W` | Відкрити меню Flow (кнопка на панелі інструментів) |
| `Enter`, `Tab`, `Shift+Tab` | У діалозі Почати / Завершити / Синхронізувати / Просунути / Задати батьківську: перегляд чи виконання, наступне поле, попереднє поле |
| `Space` | Поставити прапорець (*Опублікувати*, *Виконати хуки*, *Також оновити гілки, що стоять поверх неї*) |
| `Esc` | Закрити випадаючий список, повернутися з перегляду до форми, закрити діалог (не під час виконання) |
| Наведення на гілку | Її тип, батьківська гілка і наскільки вона позаду батьківської |
## Панель Log

| Клавіша | Дія |
|---|---|
| `Enter` / `l` | Увійти в список змінених файлів вибраного commit'у |
| `Enter` / `l` на рядку файлу | Відкрити diff файлу в центрі |
| `Esc` / `h` | Повернутися до commit'ів |
| `Right` | (commit із кількома ref) Відкрити список його інших ref: `Up` / `Down` рухають, `Enter` робить checkout, `Esc` закриває |
| `f` | Відкрити рядок пошуку (див. [Пошук](#пошук)) |
| `c` | (за наявності незакомічених змін) Вибрати рядок WIP і поставити курсор у його поле повідомлення |
| `Enter` на рядку WIP | Поставити курсор у поле повідомлення рядка WIP |
| `.`, `Menu`, `Shift+F10` | Меню commit'у. `Up` / `Down` рухають, `Right` / `l` / `Enter` відкривають підменю, `Left` / `h` / `Esc` повертають назад |
| `m` / `a` | Файли правої панелі: `Шлях` чи `Дерево` / `Усі файли` |
| `Shift+C` | Вибрати стовпець Log для пересування або зміни ширини (див. нижче) |

Після вибору стовпця `Tab` / `Shift+Tab` вибирають інший, `<` / `>` пересувають його, `]` / `[` розширюють або звужують, `0` повертає всім автоматичну ширину, а `Esc` або `Enter` виходить (контекст `log_columns` у згенерованій таблиці вище).

Поле повідомлення рядка WIP є заголовком композера commit'у. Поки в ньому курсор,
вводиться кожна клавіша, але комбінації з `Ctrl` досі працюють. `Enter` переходить до опису
композера. `Esc` залишає поле й зберігає текст. Див.
[Stage і commit](staging-and-committing.md).

### Вибір кількох commit'ів

| Клавіша чи жест | Дія |
|---|---|
| `Space` | Додати поточний commit до вибору або прибрати його |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Розширити вибір (Alt для терміналів, що залишають Shift собі) |
| `Ctrl+клацання` / `Shift+клацання` або `Alt+клацання` | Перемкнути commit / вибрати діапазон |
| `Esc` | Очистити вибір |
| `Shift+Y` | [Копіювати вибране як Mermaid gitGraph](log-and-history.md#копіювання-commitів-як-mermaid-gitgraph) (вікно з рівнями деталізації та живим переглядом; `Enter` копіює) |

Коли вибрано кілька commit'ів, клацніть правою кнопкою на одному з них і виберіть **Cherry
pick N commit'ів на `<гілка>`...**. Це відкриває той самий вигляд, що й інтерактивний rebase.

### Назва гілки чи tag

`Ctrl+G b`, `Ctrl+G t`, кнопки панелі інструментів **Гілка** і **Tag** та пункти меню
**Створити гілку тут...** / **Створити tag тут...** питають назву в текстовому полі на
рядку commit'у. `Enter` створює ref. `Esc` скасовує. Неправильна чи наявна назва
показується червоним, а поле залишається відкритим.

### Зміна повідомлення commit'у

**Змінити повідомлення commit'у...** в меню commit'у, `Ctrl+G e` або команда палітри
`Git: змінити повідомлення commit'у...` відкриває вікно **Зміна повідомлення commit'у**. Що
воно змінює, див. у розділі [Робота з Log](log-and-history.md).

| Клавіша | Дія |
|---|---|
| `Enter` у заголовку | Перейти до опису |
| `Enter` в описі | Новий рядок. `Up` на першому рядку повертає до заголовка |
| `Ctrl+Enter`, `Ctrl+S` або **Оновити повідомлення** | Зберегти нове повідомлення |
| `Tab` / `Shift+Tab` | Перемикатися між заголовком, описом і кнопками |
| `Esc` або **Скасувати** | Закрити без змін |

### Зміна commit'у

`Ctrl+G Shift+E` (усі поля) і `Ctrl+G Shift+A` (автор) відкривають вікно зміни commit'у, як і меню
commit'у, рядки палітри `Commit: змінити...` і панель відомостей про commit; усі шляхи до нього описано
в розділі [Зміна commit'у](log-and-history.md#зміна-commitу-повідомлення-автор-дати). Обидві комбінації
можна перепризначити: `git_prefix.amend` і `git_prefix.edit_author`. Клавіші у вікні:

| Клавіша | Дія |
|---|---|
| `Tab` / `Shift+Tab` | Перемикатися між полями й кнопками |
| `Enter` в однорядковому полі | Наступне поле; після останнього - **Застосувати** |
| `Up` / `Down` | Попереднє / наступне поле (в описі - з його першого / останнього рядка) |
| `Ctrl+Enter`, `Ctrl+S` або **Застосувати** | Застосувати змінені поля |
| `Esc` або **Скасувати** | Закрити без змін |
| `Alt+H` або **Змінити історію глибше...** | Закрити вікно й відкрити майстерню переписування історії від цього commit'у (лише в збірках із переписуванням історії); якщо ви змінили поле, спершу питає (`Esc` підтверджує). Можна перепризначити: `reword.deeper` |

### Створення worktree

**Створити worktree з `<ref>`...** (меню гілок і tag) та **Створити worktree тут...** (меню
commit'у) відкривають діалог: папка, необов'язкова нова гілка та **Відкрити новий worktree як
вкладку**. Клавіші: `Tab` / `Shift+Tab` переміщують, `Enter` створює, `Space` перемикає
прапорець, `Ctrl+B` відкриває огляд, `Esc` скасовує.

### Інтерактивний rebase

**Інтерактивний rebase звідси...** в меню commit'у або
`Git: інтерактивний rebase від вибраного commit'у...` відкриває вигляд rebase. Cherry-pick
кількох commit'ів використовує той самий вигляд. Див. [Робота з Log](log-and-history.md).

| Клавіша | Дія |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Задати дію рядка: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup і Edit лише в rebase) |
| `Space` | Відкрити список дій рядка (`Up` / `Down` вибирають, `Enter` застосовує, `Esc` закриває список) |
| `Up` / `Down` | Перемістити вибір |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Перемістити вибраний рядок вгору / вниз |
| `a` | Увімкнути чи вимкнути попередній перегляд autosquash |
| `Enter` на рядку Reword | Змінити нове повідомлення цього commit'у |
| `Ctrl+S`, `Ctrl+Enter` або **Почати rebase** / **Почати cherry pick** | Виконати план |
| `Ctrl+R` або **Скинути** | Відновити план у стані, в якому він був під час відкриття вигляду |
| `Esc` або **Скасувати** | Закрити без змін |

Коли rebase зупиняється (на конфлікті чи кроці Edit), банер над Log пропонує
**Продовжити**, **Пропустити** та **Перервати**. У палітрі є `Rebase: продовжити`,
`Rebase: пропустити` та `Rebase: перервати`.

## Пошук

`f`, кнопка панелі інструментів **Пошук** або `Навігація: пошук commit'ів...` відкриває
рядок пошуку над Log. Він шукає за повним повідомленням commit'у, іменем і email автора, SHA
та назвами ref. Він не ховає рядки: рядки, що не збігаються, тьмяніють.

| Клавіша | Дія |
|---|---|
| *(введення)* | Редагувати текст пошуку (вводиться кожна клавіша) |
| `Enter` / `Down` або `↑` рядка | Наступний збіг |
| `Shift+Enter` / `Up` або `↓` рядка | Попередній збіг |
| `Esc` або `✕` рядка | Закрити рядок пошуку |

## Подання Diff

Змінений файл відкривається в центрі замість Log. Див.
[Stage і commit](staging-and-committing.md).

| Клавіша | Дія |
|---|---|
| `Enter` / `l` або подвійне клацання (на рядку файлу) | Відкрити diff файлу |
| `Esc`, `q` або `✕` у заголовку | Закрити diff |
| `j` / `k`, `Up` / `Down` | Прокрутити |
| `PageUp` / `PageDown`, `Home` / `End` | Прокрутити на сторінку; на початок / у кінець |
| `h` / `l`, `Left` / `Right` | Прокрутити вбік (не з перенесенням рядків) |
| `n` / `p` | Наступний / попередній hunk |
| `m` | Розкладка Inline чи Split |
| `Shift+H` | Лише hunk'и чи весь файл |
| `w` | Увімкнути чи вимкнути перенесення рядків |
| `s` | Показати чи сховати зміни лише пробілів |
| `Shift+M` | Увімкнути чи вимкнути мінікарту |
| `Shift+C` | Вибрати кодування, у якому читається файл (список: Автовизначення першим) |
| `e` | Редагувати файл у вбудованому редакторі (див. [Редактор](#редактор)) |
| `r` | (файл Git LFS) Картка LFS або сирий вказівник. Див. [Git LFS](lfs.md) |
| `Shift+E` | (файл із конфліктом) Відкрити редактор merge. Див. [Вирішення конфліктів merge](merge-conflicts.md) |
| `y` / `Ctrl+C` | Копіювати вибраний текст. Без вибору: рядок під курсором або вибрані рядки |
| `.` або праве клацання по hunk | Меню hunk |

### Stage і відкидання з diff

Ці клавіші працюють лише в diff робочих змін (відкритому зі списку Unstaged чи Staged).
Відкидання працює лише зі списку Unstaged.

| Клавіша | Дія |
|---|---|
| `Space` | Stage (Unstaged) / unstage (Staged) hunk під курсором |
| `v` | Почати чи скасувати вибір рядків; `j` / `k` розширюють його |
| `Enter` | Stage / unstage вибраних рядків або рядка під курсором |
| `x` | Відкинути hunk під курсором (спершу питає: `y` / `Enter` підтверджують, `n` / `Esc` скасовують) |
| `Shift+X` | Відкинути вибрані рядки або рядок під курсором (спершу питає) |

Враховуються лише змінені рядки: контекстні рядки у виборі ігноруються. Вибір, що сягає
другого hunk, діє лише на перший hunk.

## Перегляд зображення

Зображення в поданні Diff або в Перегляді файлу. Клавіші є в контексті `image` таблиці вище. Див.
[Зображення](image-view.md).

## File View, Blame і історія файлу

| Клавіша | Дія |
|---|---|
| `Esc` / `q` | Закрити (за вибраного тексту `Esc` спершу скидає вибір) |
| `j` / `k`, `Up` / `Down` | Рухати вибір |
| `PageUp` / `PageDown` | Прокрутити на сторінку |
| `Home` / `End`, `Shift+G` | Перший / останній рядок |
| `h` / `l`, `Left` / `Right` | Прокрутити текст убік |
| `Enter` | (Blame, історія файлу) Перейти до commit'у рядка в Log |
| `e` | Редагувати файл у вбудованому редакторі |
| `y` / `Ctrl+C` | Копіювати вибраний текст. Без вибору: вибраний рядок (Blame: лише код; Історія файлу: короткий опис) |

Blame та історія файлу мають власні клавіші, перелічені в згенерованій таблиці вище (контексти `file_insight` і `history`) і як `Blame: ...` / `Історія файлу: ...` у палітрі; усі можна перепризначити. Докладніше в [Blame](log-and-history.md#blame) та [Історія файлу](log-and-history.md#історія-файлу). У списку історії окремі літери (`d` `o` `b` `c` `y`) діють на вибраний рядок, але сполучення з Ctrl (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) завжди зберігає глобальне значення.

## Стовпець commit'ів

Стовпець ліворуч від виду файлу (див. [Стовпець commit'ів](interface-tour.md#стовпець-commitів)). `Alt+C` показує або ховає його звідки завгодно; інші клавіші працюють, коли клавіатура в стовпця (`Ctrl+N`, потім його значок, або `Вигляд: перейти до стовпця commit'ів`).

| Клавіша | Дія |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Вибрати commit |
| `Enter` / `Right` | Перейти до списку файлів вибраного commit'у |
| `Esc` | Повернути клавіатуру виду |
| `.`, `Menu`, `Shift+F10` | Меню вибраного рядка |

Інші його клавіші (`Shift+Left` / `Shift+Right` для прокручування вбік, `Tab`, `]`, `[` і `0` для ширини частин) є в згенерованій таблиці вище, контекст `commit_column`.

## Редактор

**Змінити** або `e` у diff чи Перегляді файлу відкриває вбудований редактор на копії файлу в
робочій папці. Див. [Stage і commit](staging-and-committing.md).

| Клавіша | Дія |
|---|---|
| *(введення)* | Вставити в позиції курсора (замінює вибране) |
| `Enter` | Новий рядок з відступом поточного рядка |
| `Tab` / `Shift+Tab` | Збільшити / зменшити відступ (вибраних рядків, якщо їх кілька) |
| стрілки, `Home` / `End` | Рухатися. `Home` переходить до першого непробільного символу, потім до стовпця 0. `Shift` розширює вибір |
| `Ctrl+Left` / `Ctrl+Right` | Перейти на слово |
| `Ctrl+Home` / `Ctrl+End` | Початок / кінець файлу |
| `PageUp` / `PageDown` | Сторінка вгору / вниз |
| `Ctrl+A` | Вибрати все |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Копіювати / вирізати / вставити |
| `Ctrl+U` / `Ctrl+K` | Видалити до початку / кінця рядка |
| `Ctrl+Z` | Скасувати |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Повторити |
| `Ctrl+S` | Зберегти |
| `Ctrl+F` / `F3` | Знайти. У полі пошуку: `Enter` / `Down` / `F3` наступний, `Shift+Enter` / `Up` попередній, `Esc` закриває |
| `Alt+M` | Увімкнути чи вимкнути мінікарту (**Map**) |
| `Esc` | Закрити. За незбережених змін питає: **Зберегти** (`S` / `Enter`), **Відкинути** (`D`), **Скасувати** (`C` / `Esc`) |

`Tab` у редакторі додає відступ, тож не переміщує фокус. Інші комбінації з `Ctrl`
(`Ctrl+P`, `Ctrl+N`) досі працюють.

**Кодування**, **закінчення рядків** і **відступи** в рядку стану - кнопки: кожна відкриває випадаючий
список (стрілки або коліщатко, введення для фільтра, `Enter` - обрати, `Esc` - закрити). Палітра (`Ctrl+P`)
відкриває ті самі меню: *Редактор: змінити кодування*, *Редактор: відкрити заново в кодуванні*, *Редактор: перетворити на кодування*, *Редактор: перетворити розриви рядків*, *Редактор: змінити відступи*. Подробиці - в [Stage і commit](staging-and-committing.md).

## Дії Git (`Ctrl+G`)

`Ctrl+G` чекає ще на одну клавішу:

| Клавіша | Дія |
|---|---|
| `s` / `u` | Stage / unstage вибраного файлу |
| `a` | Stage усіх змін |
| `x` | Відкинути всі зміни (спершу питає) |
| `c` | Відкрити композер commit'у |
| `Enter` | Одразу зробити commit змін зі staged з повідомленням форми, звідки завгодно (див. [Stage і commit'и](staging-and-committing.md#написання-commitу)) |
| `m` | Stash усіх змін із повідомленням (порожнє дозволено). Коли ШІ ввімкнено, `Ctrl+G` у цьому запиті пише повідомлення |
| `Shift+T` | Stash усіх змін (з untracked-файлами), без повідомлення |
| `o` | Pop stash: вибраний рядок stash у Log, а якщо його немає, останній stash |
| `p` | Pull так, як налаштовано кнопку Pull (`git.pull_mode`) |
| `Shift+P` | Push на типовий remote (на першому push задає upstream) або запитати remote |
| `Shift+U` | Push усіх tag |
| `f` | Fetch усіх remote, з prune |
| `g` | Відкрити в браузері pull request гілки, на якій зроблено checkout |
| `b` / `t` | Нова гілка / новий tag на HEAD (див. [Назва гілки чи tag](#назва-гілки-чи-tag)) |
| `d` | Видалити вибрану гілку чи tag (спершу питає) |
| `e` | Змінити повідомлення вибраного commit'у |
| `Shift+E` / `Shift+A` | Змінити вибраний commit (усі поля) / його автора (див. [Зміна commit'у](#зміна-commitу)) |
| `y` | Cherry-pick вибраного commit'у |
| `v` | Revert вибраного commit'у |
| `Shift+R` | Reset (mixed) до вибраного commit'у |
| `r` | Rebase на вибрану гілку |
| `Shift+W` | Відкрити меню Flow (у репозиторії з робочим процесом гілок) |
| `Shift+H` | Відкрити майстерню переписування історії (лише в збірках із переписуванням історії) |
| `l` | Відкрити меню LFS. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch усього / checkout усього / pull / push усього / prune. Див. [Git LFS](lfs.md) |
| `i` | ✦ ШІ: написати повідомлення commit'у для змін у staged |
| `w` | ✦ ШІ: пояснити вибраний commit або робочі зміни |
| `Shift+C` | ✦ ШІ: запропонувати, як розбити робочі зміни на commit'и |

Клавіші ШІ працюють лише тоді, коли ШІ ввімкнено (див. [Допомога ШІ](ai.md)). `Esc`
скасовує запит ШІ, що виконується.

## Права панель (робочі зміни та подробиці commit'у)

Коли вибрано рядок WIP, права панель показує списки **Unstaged** і **Staged** та композер
commit'у. Коли вибрано commit, вона показує подробиці commit'у та його файли.

| Клавіша | Дія |
|---|---|
| `j` / `k`, `Up` / `Down` | Рухатися. У кінці одного списку вибір переходить у інший |
| `h` / `l`, `Left` / `Right` | Перейти до списку Unstaged / Staged |
| `Enter` | Згорнути чи розгорнути активний список |
| `Space` / `Insert` | Додати поточний файл до вибору або прибрати його |
| `Ctrl+A` | Виділити всі файли списку |
| `Ctrl+Space` | Stage / unstage вибраного файлу або згорнути рядок папки |
| `d` | Відкрити diff вибраного файлу |
| `m` | `Шлях` (плоский список) або `Дерево` (папки) |
| `a` | `Усі файли`: усе дерево, а не лише змінені файли |
| `z` | Згорнути всі папки або знову розгорнути |
| `/` | Фільтрувати змінені файли commit'у за назвою |
| `.`, `Menu`, `Shift+F10` | Меню файлу |
| `c` | Фокус на полі заголовка композера commit'у |
| `Shift+A` | Увімкнути чи вимкнути **Amend попереднього commit'у** |
| `Ctrl+N ]` / `Ctrl+N [` | Збільшити / зменшити блок повідомлення commit'у |

У композері commit'у:

| Клавіша | Дія |
|---|---|
| `Enter` | Заголовок: перейти до опису. Опис: новий рядок |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` робить це звідки завгодно) |
| `Alt+R` | Вставити посилання на issue |
| `Esc` | Залишити композер |

Меню файлу містить: Stage чи Unstage, Відкинути зміни, Ігнорувати, Stash файлу, Історія
файлу, Blame файлу, Відкрити в зовнішньому інструменті (diff), Відкрити в зовнішньому
редакторі, Відкрити файл у типовій програмі, Показати в папці, Копіювати шлях до файлу,
Змінити файл, Створити патч зі змін файлу... і Видалити файл... Для файлу з конфліктом воно
починається з **Відкрити в редакторі merge** і трьох рядків **Вирішити через**. Із
увімкненим Git LFS воно має рядки LFS. **Відкинути зміни...** і **Видалити файл...** спершу
питають.

## Кнопки вікна

Кожна кнопка має клавіатурний відповідник:

| Кнопка | Клавіша |
|---|---|
| 🐙 ліворуч у рядку заголовка (Про GitPoulpi) | `Довідка: про GitPoulpi` у палітрі |
| **Відкрити в** у рядку заголовка | `Alt+O`, `Ctrl+Alt+O` |
| `×` праворуч у рядку заголовка (вихід) | `q` |
| Версія в рядку заголовка (нотатки до випуску) | `Довідка: нотатки до випуску` у палітрі |
| `⌕ Глобальний пошук` у рядку заголовка, ліворуч від `⚙` | `Ctrl+Alt+F` |
| `⚙` у рядку заголовка (Налаштування) | `Ctrl+,` |
| Чип робочого простору в рядку вкладок | `Alt+W` |
| `+` після останньої вкладки | `Alt+T` |
| Вкладка **Radar** | `Alt+R` |
| Вкладка **Робочі простори** | `Alt+J` |
| **Активність** у рядку стану | `Ctrl+N Shift+A` |
| Панель інструментів: **Скасувати**, **Повторити** | `Ctrl+Z`, `Ctrl+Y` |
| Панель інструментів: **Pull** / її `▾` | `Ctrl+G p` / `Git: параметри Pull...` |
| Панель інструментів: **Push** / її `▾` | `Ctrl+G Shift+P` / `Git: Push на remote...` |
| Панель інструментів: **Гілка**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Панель інструментів: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Панель інструментів: **LFS** (коли LFS увімкнено) | `Ctrl+G l` |
| Панель інструментів: **Flow** (коли репозиторій має робочий процес) | `Ctrl+G Shift+W` |
| Панель інструментів: **Пошук** | `f` |
| Панель інструментів: прив'язаний pull request (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Панель інструментів: список профілів (від двох профілів) | `Репозиторій: змінити профіль...` |

Панель інструментів описано в [Огляді інтерфейсу](interface-tour.md#панель-інструментів).

## Налаштування

`Ctrl+,`, `Ctrl+N ,`, шестерня `⚙` або `Застосунок: налаштування...` відкриває екран
Налаштувань.

| Клавіша | Дія |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Перемикатися між списком сторінок і параметрами |
| `j` / `k`, `Up` / `Down` | Рухати вибір |
| `Enter` / `Space` | Відкрити сторінку, перемкнути прапорець, редагувати поле, відкрити випадаючий список або натиснути кнопку |
| `/`, `Ctrl+F` | Шукати в усіх параметрах |
| `Alt+Left` / `Alt+Right` | Перемкнути вкладену вкладку («Загальні» / список) |
| `Esc` / `q` | Закрити Налаштування. Поки редагується поле, `Esc` скасовує лише редагування |
| `?` | Відкрити довідку поверх екрана Налаштувань |

Див. [Налаштування](settings.md).

## Текстові поля

Усі текстові поля мають однакові клавіші: фільтри, поля пошуку, палітра команд, запити,
композер commit'у, поле рядка WIP, поля діалогів і поля Налаштувань. Поки клавіатуру
займає поле, ці клавіші мають пріоритет над скороченнями панелей.

| Клавіша | Дія |
|---|---|
| символ | Ввести його в позиції курсора (замінює вибране) |
| `Left` / `Right` | Перейти на символ (за наявного вибору: на його початок / кінець) |
| `Ctrl+Left` / `Ctrl+Right` | Перейти на слово |
| `Home` / `End` | Початок / кінець рядка |
| `Shift` + рух вище | Розширити чи зменшити вибір |
| `Up` / `Down` (лише опис) | Попередній / наступний рядок |
| `Ctrl+Home` / `Ctrl+End` (лише опис) | Початок / кінець тексту |
| `Ctrl+A` | Вибрати все |
| `Backspace` / `Delete` | Видалити символ перед / після курсора або вибране |
| `Ctrl+U` | Видалити від курсора до початку рядка (або вибране) |
| `Ctrl+K` | Видалити від курсора до кінця рядка (або вибране) |
| `Ctrl+C` / `Ctrl+X` | Копіювати / вирізати вибране (не в полі пароля) |
| вставлення (власне вставлення терміналу) | Вставити в позиції курсора, замінивши вибране |

В описі commit'у `Enter` вводить новий рядок, а `Ctrl+Enter` робить commit. Не всі термінали
однаково надсилають `Ctrl+Enter`, тож commit роблять також `Alt+Enter` і `Ctrl+J`.

Деякі консолі надсилають `Ctrl+літера` як керівний символ (`Ctrl+A` як `0x01`). GitPoulpi
розуміє обидві форми. Символи `AltGr` (`@`, `#`, `€` на швейцарській чи французькій
розкладці) вводяться.

## Підказки

Підказка з'являється, коли вказівник залишається приблизно 0,6 с на цілі (клітинка Log,
кнопка панелі інструментів, прив'язаний pull request). Натискання клавіші, клацання,
прокрутка, зміна розміру, відкрите вікно чи меню або відхід вказівника з цілі ховає її.
Після цього вона повертається, лише коли вказівник знову рухається.
