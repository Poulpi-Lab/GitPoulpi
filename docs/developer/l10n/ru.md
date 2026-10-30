# Russian (ru) terminology and style

Reviewed by a localizer pass (2026-10). Future edits to `crates/gitpoulpi-i18n/locales/ru.toml` follow this page. Sources: git's own Russian translation (git-l10n `po/ru`), Microsoft terminology and style guide for Russian, GitHub / VS Code / JetBrains / TortoiseGit Russian UIs. Mechanics (tokens, plurals, lints) are in `../i18n.md`.

## Style decisions

| Topic | Decision |
|---|---|
| Address | Formal "вы" in imperatives and sentences ("Выберите", "Нажмите", "ваши изменения"). Never "ты". Most strings avoid addressing the user. |
| Buttons, menu rows, palette commands | Infinitive, capitalised first word: "Создать ветку...", "Открыть в браузере". Progress: verbal noun ("Загрузка...", "Применение патчей..."), never first person ("Открываю"). Results: participle ("{path} сохранён"). |
| Palette and keymap | `Категория: действие`, colon directly after the category, no space before it, lower case after the colon (Russian orthography; proper names such as `Blame`, `Log`, `Refs` keep their case). Categories are translated (Вид, Файл, Правка, Репозиторий, Приложение, Навигация) except `Git`, `LFS`, `Refs`, `Remote`, `Log`, `Blame`, `Merge`... that name an interface part; in local mode `Diff:`, `Merge:`, `Rebase:` become `Различия:`, `Слияние:`, `Перебазирование:`. |
| Titles, tabs, columns | Sentence case ("Журнал активности"), not English Title Case. Section headings in the sidebar are upper case and follow the heading text exactly (ЗАДАЧИ, ПОДМОДУЛИ, ТЕГИ, СТЭШИ). Prose that names a heading, column or button uses the Russian label in «ёлочки» ("колонка «Дата»", "кнопка «Подтянуть»"). |
| Reasons for a greyed row | Lower case: "сначала выберите ветку в панели Refs". |
| Quotes | «ёлочки» for UI names and quoted values; plain ASCII apostrophe `'` is required by the catalog test (also in git-mode inflections "commit'а", "tag'ов"); never `’`. |
| Dash | Never an em dash; " - " (hyphen with spaces). |
| Ellipsis | Three ASCII dots `...` everywhere (English dropped `…` in 2026-10): commands that open a dialog, progress texts, placeholders. Never `…`. |
| Status-line messages | Start lower case, like English ("сохранение {path}...", "не удалось сохранить {path}: ...", "фильтры LFS установлены"); the message is not a sentence of its own. |
| Keys | `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Home`, `End`, `PgUp`, `PgDn` stay Latin (the chord printer shows them). Words without a printed form: "Пробел", "Вверх/Вниз/Влево/Вправо". |
| Mouse | "щелчок" / "двойной щелчок" / "правый щелчок" (Microsoft), never "клик". |
| Dates | `{day} {month} {year}` with the month in the genitive from `[month_of]` ("5 марта 2024"), standalone months and weekdays lower case ("март 2024", "вторник"), "5 минут назад", "только что". No trailing "г.". |
| Plurals | CLDR ru: `one` (1, 21...), `few` (2-4, 22-24...), `many` (0, 5-20...), `other` = fractions (write as `many`). A message that shows a bare number in front of a noun is a plural table; where English is a plain string with `{n}`, reword so the noun does not agree ("Cherry pick выбранных коммитов ({n})"). |
| Undo safety (2026-10) | "История отмен" (Undo history), "Восстановить только это" (Restore only this), "Отменить до этого места" / "Повторить до этого места", "Потерянная работа" (Lost work), "пин отмены" (undo pin), "Reflog" stays Latin (git's own word), "Показать в Log", "Создать ветку здесь". Counts of actions are plural tables ("Можно отменить 12 действий (до 100)"). The bare `{$commits}` term gives the English plural "commits"; in running text write a pair ("commit'ы" / "коммиты"). |
| Generic wording | "Настройки" (never "Параметры"/"Предпочтения"), "Главная" (the Home tab), "рабочее дерево" (working tree), "рабочая папка" (working directory), "двоичный" (binary, never "бинарный"), "хостинг", "провайдер", "задача" (issue), "ИИ", "Журнал активности", "палитра команд", "подмодуль", "ветка", "ссылка" (ref), "панель" (pane), "вид" (view). |

## Glossary

`git` = default vocabulary (the English git word, as Russian developers say it; inflected with an apostrophe where needed: "commit'а", "tag'ов", "stash'ей"). `local` = "Translate git terms" mode: proper Russian, applied consistently. Where both are the same, the English word is the Russian word. Local forms in the `[term]` table are nominative/accusative (inanimate), so `{$term}` fits object and subject positions; where another case is needed write a `{ git = "..", local = ".." }` pair. Verb terms (`push`, `pull`, `fetch`, `stage`...) are infinitives and do not fit noun slots ("Push failed", "after a fetch"): write a pair with the verbal noun.

| Concept | git vocabulary | local vocabulary | Notes |
|---|---|---|---|
| commit (n.) | commit (m.) | коммит (m.) | "Коммит" button; verb in prose: "сделать коммит" (git-l10n "коммит"); never "фиксация" for the object (but "зафиксированный/незафиксированный" is fine for state) |
| push | push | отправить / отправка | noun "отправка"; "force push" = "принудительная отправка" |
| pull | pull | подтянуть / получение изменений | button "Подтянуть"; noun "получение изменений"; never mix with fetch |
| fetch | fetch | загрузить / загрузка | noun "загрузка" |
| merge | merge | слияние (n.) | "Идёт слияние", "редактор слияния", "Слияние A в B"; verb "слить" only in free prose |
| rebase | rebase | перебазирование (n.) / перебазировать | |
| cherry-pick, revert, fixup, bisect, reset, amend, autosquash, upstream, checkout, hunk, blame, ref, SHA, LFS, worktree, WIP, solo, lease, prune | same | same | git-l10n does not translate them either. "checkout" is also used as the verb ("делает checkout"); in local mode prose may say "переключиться на ветку" |
| pick / reword / squash / drop / edit (rebase actions) | same | взять / переформулировать / объединить / убрать / изменить | `[term]` table; "Drop" of a stash is "Удалить" (token `{$Stash_drop}`). Menu rows that act (merge, rebase) are pairs with the verb in local mode ("Слить", "Перебазировать"), because the `{$merge}` / `{$rebase}` terms are nouns |
| stash | stash (m.) | стэш (m.) | verb "спрятать в стэш"; Pop = "Достать" (button) / "извлечение" (noun); Apply = "Применить"; auto-stash = "автостэш" |
| tag | tag (m.) | тег (m.) | not "метка" (that is the issue label, "метка") |
| diff | diff (m.) | различия (pl.) | "представление различий", "открыть различия"; the genitive plural needs a pair |
| stage / unstage / staged / unstaged | stage, unstage, staged, unstaged | добавить в индекс, убрать из индекса, в индексе, вне индекса | halves: "Файлы в индексе" / "Файлы вне индекса" |
| pull request / merge request | pull request / merge request | запрос на слияние (both hosts) | PR/MR abbreviations stay Latin; the "Pull Request"/"Merge Request" setting choices stay English (indistinguishable otherwise) |
| branch, remote, HEAD | ветка, remote, HEAD | same | "remote" stays Latin everywhere; "удалённая ветка" for remote-tracking branches; "HEAD отсоединён" |
| working tree / index | рабочее дерево / индекс | same | |
| conflict, resolve | конфликт, разрешить | same | "Текущее" (A) / "Входящее" (B) / "База" / "Результат" in the merge editor |
| repository, submodule, settings, palette | репозиторий, подмодуль, настройки, палитра команд | same | |
| issue, label, profile, integration, token | задача, метка, профиль, интеграция, токен | same | |
| Home / Log / Refs / Status | «Главная» / Log / Refs / Status | same | pane and view names stay Latin, the Home tab is «Главная» |

## Known residue (lint baseline `tests/git_vocabulary/ru.toml`)

Inherent, cannot be written without the English stem: placeholder or key names that are code (`{commit}`, `{tag}`, `{stage}`, `{{diff.edit}}`, `{{git_prefix.stash_pop}}`, `git.pull_mode`, `general.auto_stash`, `merge.tool`), the `stash@{n}` ref name, the English prompt chips sent to the model (`ai_page.chip_*_text`), the setting choices "Pull Request"/"Merge Request" and the help that explains the setting itself. The baseline may only shrink.

## User guide (`docs/user/ru/`) conventions

The guide describes the default UI (git vocabulary), so it quotes the catalog exactly and follows it in prose.

| Topic | Decision |
|---|---|
| commit, tag, stash, hunk, pull, push, fetch | Latin, inflected with an apostrophe ("commit'а", "tag'ов", "hunk'и"), as in the catalog. "Коммит", "тег", "ханк" are not used in the guide; the verb is "сделать commit", "попадает в commit". The local vocabulary (коммит, тег) is only named where Settings > Translate git terms is explained. |
| Home tab | «Главная» (the catalog's name), never "Home". |
| Nav badges (`Ctrl+N` then a character) | "значок", as in the catalog's `<значок>`; not "бейдж". |
| Composer | "композитор" (`Композитор commit'ов`), not "композер". |
| Footer vs status line | The bottom line of the window is "нижняя строка" (the catalog's "плашка внизу"); "строка состояния" is the message line / the editor's status bar. |
| A single setting | "настройка" (the screen is "Настройки"); "параметр" only for a CLI flag or a config key. |
| Binary | "двоичный файл" for a file git cannot diff; "исполняемый файл" for the program binaries. |
| Settings paths | Bold, `**Настройки > Страница > Строка**`, each part exactly as in the catalog (e.g. "Глубина цвета", "Клавиатура и мышь"). |
| Palette commands | Backticks, exactly as in the generated list of `keybindings.md` (lower case after the colon, `✦ ИИ: ...` with the sparkle). |
| Code comments | Translated in TOML/shell examples; keys, values and commands stay as they are. |
| Terminals page (`terminals.md`) | "Выбор терминала"; "изображения" for pictures, "полублоки" for half-blocks, "подложка" for backdrop, "мультиплексор"; the capability table, sources and dates stay as in English; the settings paths of other applications (iTerm2, Windows Terminal) stay English in italics. |

## Outside changes, merge revert, undo labels (2026-10)

| Topic | Decision |
|---|---|
| Outside change | "внешнее" (the tag), "Сделано вне GitPoulpi: {who}"; setting "Записывать изменения, сделанные вне GitPoulpi"; heading "История отмен". |
| Merge revert | "коммит слияния" (local) / "merge-commit'а" (git); "родитель" (parent), "основная линия" (mainline). |
| `[undo_label]` | One template per action; verbs in the infinitive, same words as the menus ("Отбросить", "Удалить", "Переименовать", "Задать", "Сбросить" for unset). Terms are `{$Stage}`, `{$Stash}`, `{$Pop}`... so both vocabularies work; counted commits are plural pairs ("{n} commit'а" / "{n} коммита"). Latin kept for `hunk`, `remote`, `upstream`, `worktree`, `checkout`, `reset`, `revert`, `sparse checkout`, reset modes. Binary/text of "treat as" is "двоичные данные" / "текст" (accusative after "Обрабатывать ... как"). |
| CLI help | Always local vocabulary ("коммит"); `reflog`, `undo-restore`, `lost-commits` entries translated; `reflog`, `stash`, `remote`, reset modes stay Latin. |

## Workspaces, Radar, search, New tab (2026-10)

| Topic | Decision |
|---|---|
| Workspace | "рабочее пространство" everywhere (never "рабочая область"); "значок рабочего пространства" for the tab-bar chip; space-saving "пространство" only in the 36-cell profile field ("{profile} - из пространства {workspace}"). |
| Radar | Stays Latin, a feature name. Actions are bare infinitives like English ("Открыть репозиторий", "Переключиться на ветку", "Показать в репозитории", "Начать работу над задачей..."). Pin / snooze: "Закрепить сверху" / "Открепить", "Отложить..." / "Вернуть из отложенных"; groups "Закреплённые", "Отложенные". |
| Pull/merge request in running text | `{$pr}` / `{$prs}` are nominative; where another case is needed write a `{ git, local }` pair ("у запроса на слияние"), as in `radar.msg_not_pr`. |
| Status messages with an item | Participle first, item after a colon ("закреплено: {item}", "отложено: {item} ({how})"), because the item's gender is unknown. |
| Verb tokens with "все" | "{$Fetch} все" / "{$Pull} все" (fetch/pull all) follow the existing "{$Fetch} {remote_ref} как ветку" precedent; palette rows use lower case after the colon ("Рабочее пространство: {$fetch} все"). |
| Plural of repositories | CLDR ru table with `other` written as `many` ("{n} репозиториев"). |
| New tab | "Новая вкладка"; "Загрузить ещё" for the reflog's Load more. |
| Home tab in prose | «Главная» in guillemets, also inside help lines. |

## Review, metadata, board, local issues (round 4, 2026-10)

| Concept | Decision | Notes |
|---|---|---|
| review (the whole), code review | "ревью" (as in the Radar's "Ждут моего ревью"); AI action "Код-ревью" | verb forms: "Завершить ревью", "Отправить ревью", "Отменить ревью..."; palette category `Ревью:` |
| request (pull / merge request / Gerrit change, host not the point) | "запрос" | `{$pr}` where a list or dialog is host-specific; Gerrit's own noun "изменение" only in Gerrit text |
| thread | "обсуждение" | never "ветка" (that is a branch) or "тред"; the non-code comments of the Overview are "беседа" (conversation) |
| resolve / reopen (a thread) | "Разрешить" / "Открыть снова"; badges "разрешено", "устарело" | same verb as conflict resolution |
| pending comment / pending review | "неотправленный комментарий" / "неотправленное ревью"; header counter "{n} неотправленных комментариев" | not "отложенный" (that is snoozed in the Radar) and not "черновик"; badge "Не отправлено - строка 12" |
| verdict, Approve, Request changes, Comment | "Решение": "Одобрить", "Запросить изменения", "Комментарий"; badges "одобрено", "запрошены изменения", "прокомментировано", "отклонено" | |
| suggestion (AI) | "предложение"; "Принять" / "Править" / "Отклонить", "Принять все" / "Отклонить все", "сводка ИИ" | severities "инфо", "предупреждение", "проблема" |
| patch set, vote (Gerrit) | "набор патчей", "голос" (labels `Code-Review`, `Verified` stay Latin) | "Push на ревью" for push for review (`refs/for/` stays Latin) |
| checks, conflicts | "проверки пройдены / не пройдены / выполняются", "конфликты", "нет конфликтов" | |
| assignee / reviewer / label / milestone | "исполнитель" / "рецензент" / "метка" / "этап" | as in GitLab's Russian UI; buttons "Исполнители", "Рецензенты", "Метки", "Этап"; palette "изменить исполнителей...", "задать этап..." |
| encoding | "кодировка"; "Определять автоматически" for Auto-detect, "Авто" on the bar | encoding names (UTF-8, Windows-1252...) stay Latin |
| board / column / card | "доска" / "колонка" / "карточка"; "Перенести карточку", "Метки колонок" | "перенос" not "перемещение" for moving a card; columns "Открытые" / "Закрытые" |
| local issues | "локальные задачи"; "папка заметок", "заметка", "хранилище Obsidian" | `Obsidian Base` stays Latin |
| AI instructions of a workspace | "Инструкции для ИИ"; "Сочетание": "Добавить к глобальным" / "Заменить глобальные"; counts "{n} набор / набора / наборов" | |
| status lines of the new views | lower case clause, no final period; participle first when an item of unknown gender follows ("перенесено: {caption} в {column}", "закрыто: {item}") | |
| supported-host lists | GitHub, GitLab, Gitea, Forgejo (then Gerrit) | |

## Branching workflows, Mermaid, Gerrit actions (round 5, 2026-10)

| Concept | Decision | Notes |
|---|---|---|
| workflow (the whole definition) | "рабочий процесс" (Settings page "Рабочий процесс", "Шаблон рабочего процесса") | `Flow` (toolbar button, palette category `Flow:`, undo label `Flow {rest}`) stays Latin, like `Log` and `Refs`; Git Flow / GitHub Flow / GitLab Flow / Trunk-based / Custom stay Latin |
| base branch / long-lived | "базовая ветка" (badge `[базовая]`, `[базовая, защищена]`) | "основная ветка (trunk)" only for the parentless base in prose; `Protected` = "Защищена" |
| topic branch / topic type | "тематическая ветка" (as in Pro Git ru) / "тип ветки" | sidebar group "Прочие" for the branches no type claims |
| parent / environment branch | "родитель" / "родительская ветка" / "ветка окружения" | button "Задать родителя", "Новый родитель" |
| start / finish / sync / promote | "Начать" / "Завершить" / "Синхронизировать" / "Продвинуть"; dialog titles are verbal nouns ("Завершение {branch}", "Синхронизация {branch}", "Продвижение") | "Finish by" = "Завершить через"; "Preview" = "Предпросмотр"; "Abort" = "Прервать", "Continue" = "Продолжить" |
| pick up (take over a host-made branch) | "подхватить" ("Подхватить ветку, созданную хостом...", status "подхвачена ветка {name}") | the form label "Take over" is "Что взять" |
| hook | "хук" ("Хуки", "Запустить хуки"); "команды оболочки" for the shell commands | never "перехватчик" |
| name pattern, token, slug | "шаблон имени", "подстановка" (never "токен": that is the auth token), "краткое имя" for the `{slug}` field | field labels "Номер задачи", "Ключ задачи" |
| issue flow (host-driven) | "процесс задач (управляется хостом)"; modes "Локально" / "Ветку создаёт хост" / "Ветку создаёт бот" | "Wait for" = "Ожидать" |
| Detect / Import (Settings) | "Определить по веткам" / "Импортировать"; "оценка {score} из 100" | the plural "Looked at n branch names" is written without agreement ("Проверено имён веток: {n}.") in all four forms |
| Mermaid popup | "Копировать как Mermaid gitGraph"; detail levels "Структура", "Ссылки", "Заголовки", "Полный"; "Направление": "Слева направо" / "Сверху вниз"; combobox label "Детализация" (12 cells) | the stored words `structure`, `refs`, `subjects`, `full`, `lr`, `tb` stay Latin in guide and CLI help; "ограждение" for the code fence |
| Gerrit submit / abandon | "Влить" / "Отозвать"; status "{id} влито" / "{id} отозвано"; compare combobox "Сравнение: база" / "Сравнение: набор патчей {n}" | the prompt says GitPoulpi cannot undo it, restore in Gerrit |
| CLI help of `flow` | local vocabulary (коммит, слияние) like the rest of the help; `remote`, `upstream`, `stash`, `checkout`, `squash`, `fast-forward` stay Latin | |
| Verb tokens in sentences | where a noun slot needs "fetch" / "push" / "commit" the string is a `{ git, local }` pair ("сначала сделайте fetch" / "сначала выполните загрузку") | same rule as the earlier rounds |
