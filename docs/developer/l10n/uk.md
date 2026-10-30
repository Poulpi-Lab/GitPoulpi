# Ukrainian (uk) terminology and style

Reviewed by a localizer pass (2026-10). Future edits to `crates/gitpoulpi-i18n/locales/uk.toml` follow this page. Sources: git's own Ukrainian translation (git-l10n `po/uk`), Microsoft terminology and style guide for Ukrainian, VS Code / JetBrains / GitHub Desktop / TortoiseGit Ukrainian UIs, GNOME/KDE Ukrainian. This is not the Russian catalog translated: several choices differ on purpose (see "Not Russian" below). Mechanics (tokens, plurals, lints) are in `../i18n.md`.

## Style decisions

| Topic | Decision |
|---|---|
| Address | Formal plural "ви" in imperatives and sentences ("Виберіть", "Натисніть", "ваші зміни"). Never "ти". Most strings avoid addressing the user. |
| Buttons, menu rows, palette commands | Infinitive, capitalised first word: "Створити гілку...", "Відкрити в браузері". Progress / status lines: verbal noun in lower case ("завантаження...", "збереження {path}...", also the in-flight link actions: "додавання в індекс..."; a git word that starts one is lower case too: "fetch...", "commit: {message}"), never first person ("Відкриваю", "Зберігаю"). Results: neuter participle ("збережено", "видалено"). |
| Palette and keymap | `Категорія: дія`, colon right after the category, no space before it, lower case after the colon ("Навігація: фокус на Refs", "Файл: відкрити за допомогою…"). Git words keep their capital when they start the action ("Git: Pull", "Git: Push"). Categories are translated (Вигляд, Файл, Правка, Репозиторій, Застосунок, Навігація, Історія, Зміни); `Git`, `LFS`, `Refs`, `Remote`, `Log`, `Blame`, `Merge` stay Latin; in local mode `Diff:` becomes `Різниця:`, `Merge:` `Злиття:`, `Rebase:` `Перебазування:`. |
| Titles, tabs, columns | Sentence case ("Журнал активності"), not English Title Case. Sidebar section headings are upper case and follow the heading term (СХОВАНКИ / STASHES via `{$STASHES}`). Quotes «ялинки» for UI names and quoted values; plain ASCII `'` apostrophe (catalog test); never `’`. |
| Reasons for a greyed row | Lower case: "спершу виберіть гілку на панелі Refs". |
| Dash | Never an em dash; " - ". |
| Ellipsis | Always the three ASCII dots `...` (never `…`), as in English, on commands that open a dialog and on progress text. |
| Keys | `Ctrl`, `Shift`, `Alt`, `Enter`, `Esc`, `Tab`, `Home`, `End`, `PgUp`, `PgDn` stay Latin. Words without a printed form: "Пробіл", "Вгору/Вниз/Вліво/Вправо". |
| Mouse | "клацання" / "подвійне клацання" / "праве клацання" / "клацніть" (Microsoft), never "клік". |
| Dates | `{day} {month} {year}` with the month in the genitive from `[month_of]` ("5 березня 2024"), standalone months and weekdays lower case ("березень 2024", "вівторок"), abbreviations "січ лют бер кві тра чер лип сер вер жов лис гру" and "пн вт ср чт пт сб нд", "5 хвилин тому", "щойно". No trailing "р.". |
| Plurals | CLDR uk: `one` (1, 21...), `few` (2-4, 22-24...), `many` (0, 5-20...), `other` = fractions (write as `many`). A bare number before a noun is a plural table; after a verbal noun the noun is genitive plural for 2-4 as well ("додавання 3 рядків"). |
| Generic wording | "Налаштування" (never "Параметри"), "Головна" (the Home tab, not "Домівка"), "папка" (folder; "тека" is Linux-only), "робоче дерево" (working tree), "робоча папка" (working directory), "двійковий" (binary, never "бінарний"), "залишити" (never colloquial "лишити"), "переривати" for abort, "скасувати" for cancel/undo, "вирішити конфлікт", "ШІ", "провайдер", "палітра команд", "підмодуль", "гілка", "ref" (Latin), "панель" (pane), "вигляд" (view), "стежити за" (follow, never "слідувати"), "в центрі" (never "по центру"), "помаранчевий". `issue`, `email`, `worktree`, `hunk`, `blame`, `ref`, `upstream`, `remote`, `solo`, `lease` stay Latin (git-l10n and Ukrainian developers keep them). |

## Glossary

`git` = default vocabulary (the English git word, as Ukrainian developers say it; inflected with an apostrophe where needed: "commit'у", "tag'а", "stash'ів"). `local` = "Translate git terms" mode: proper Ukrainian, applied consistently everywhere. Local forms in the `[term]` table are nominative (inanimate), so `{$term}` fits subject and object positions; where another case or a verb is needed write a `{ git = "..", local = ".." }` pair. Verb terms (`push`, `pull`, `fetch`...) are verbal nouns in the table and do not fit "Push failed" noun slots without care: write a pair.

| Concept | git vocabulary | local vocabulary | Notes |
|---|---|---|---|
| commit (n.) | commit (m.) | коміт (m.) | verb in prose: "закомітити"; state: "незакомічені зміни" in both modes |
| push | push | надсилання / надіслати | "force push" = "примусове надсилання"; never "відправка" |
| pull | pull | отримання / отримати | toolbar button "Отримати" (a verb: "Отримання" is 9 cells too wide next to the others, and "Надсилання" is 10 so the Push button is "Надіслати"); never mixed with fetch |
| fetch | fetch | завантаження / завантажити | "Fetch All" = "Завантажити все" |
| merge | merge | злиття (n.) | "редактор злиття", "Злиття A у B", "запит на злиття" |
| rebase | rebase | перебазування (n.) / перебазувати | |
| fast-forward | fast-forward | перемотування / перемотати | git-l10n; in git mode the English word stays |
| cherry-pick, revert, fixup, reset, amend, autosquash, upstream, checkout, hunk, blame, ref, SHA, LFS, worktree, WIP, solo, lease, prune, history | same | same | git-l10n does not translate them either; "checkout" is also a verb in prose |
| pick / reword / squash / drop / edit (rebase actions) | same | взяти / змінити опис / об'єднати / відкинути / змінити | `[term]` table |
| stash | stash (m.) | схованка (f.) | verb "сховати"; Pop = "Повернути"; Apply = "Застосувати"; Drop of a stash = "Видалити"; auto-stash = "автосхованка"; `stash@{n}` is a ref name and stays |
| tag | tag (m.) | тег (m.) | genitive "тега" (local) / "tag'а" (git) |
| diff | diff (m.) | різниця (f.) | "подання різниці", "відкрити різницю"; the palette category `Diff:` is `Різниця:`; "інструмент порівняння" for an external diff tool |
| stage / unstage / staged / unstaged | stage, unstage, staged, unstaged | додати в індекс, прибрати з індексу, в індексі, без індексу | halves: "Файли в індексі" / "Файли без індексу" |
| pull request / merge request | pull request / merge request | запит на злиття (both hosts) | PR/MR abbreviations stay Latin; the "Pull Request" / "Merge Request" setting choices stay English (indistinguishable otherwise) |
| branch, HEAD, remote | гілка, HEAD, remote | same | "відокремлений HEAD" (git-l10n), "віддалена гілка" |
| working tree / index | робоче дерево / індекс | same | |
| conflict, resolve | конфлікт, вирішити | same | merge editor sides: "Поточне" (A) / "Вхідне" (B) / "Основа" / "Результат" |
| repository, submodule, settings, palette | репозиторій, підмодуль, налаштування, палітра команд | same | |
| sparse checkout | розріджений checkout | same | not "частковий" |
| issue, label, profile, integration, token | issue, мітка, профіль, інтеграція, токен | same | |
| Home / Log / Refs / Status | «Головна» / Log / Refs / Status | same | pane and view names stay Latin, the Home tab is «Головна» |

## Not Russian

Differences from `ru.md` that must not be "harmonised": push = "надсилання" (ru "отправка"), stash = "схованка" (ru "стэш"), diff = "різниця" (ru "различия"), abort = "перервати", folder = "папка", click = "клацання", binary = "двійковий", the lower-case palette action after the colon is shared, but Ukrainian keeps `issue` Latin (ru "задача"). Ukrainian apostrophe `'` inside words ("об'єкт", "ім'я", "п'ятниця") is the ASCII one.

## Known residue (lint baseline `tests/git_vocabulary/uk.toml`)

Inherent, cannot be written without the English stem: placeholders or key names that are code (`{commit}`, `{tag}`, `{stage}`, `{{diff.edit}}`, `{{git_prefix.stash_pop}}`, `git.pull_mode`, `git.push_behavior`, `general.auto_stash`, `--autostash`, `diff.tool`, `merge.tool`), the `stash@{n}` ref name, the English prompt chips sent to the model (`ai_page.chip_*_text`, "conventional commits"), the "Pull Request"/"Merge Request" setting choices and the help that explains the setting itself. The baseline may only shrink.

## User guide wording (`docs/user/uk/`)

Choices made by the user-guide review (2026-10) where the catalog is silent or inconsistent. The guide quotes catalog labels exactly (git vocabulary variant); this table covers the prose around them.

| Concept | Ukrainian in the guide | Notes |
|---|---|---|
| git words in prose | commit, tag, stash, diff, merge, rebase, push, pull, hunk, Log | The default (git) vocabulary, inflected with an apostrophe ("commit'ів", "tag'и", "hunk'а"). Never "коміт", "тег", "ханк", "Журнал" for the Log pane. The Activity Log stays "Журнал активності". |
| issues | issue(s) | Not "задача". "трекери issues". |
| click / double-click / right-click | клацання, подвійне клацання, праве клацання, "клацніть правою кнопкою" | Never "клік". |
| folder | папка, підпапка, "робоча папка" | Never "тека". |
| Home tab | «Головна» (indeclinable: "на вкладці Головна") | Never "Домівка". |
| ref pill | "мітка ref" (plural "мітки") | Not "пігулка". "чип" stays for the toolbar chip (`GH #12`). |
| File View / Diff View | "Перегляд файлу" / "подання Diff" | As in the catalog; "вигляд" is used only for a generic view. |
| sparse checkout | "розріджений checkout" | Badge `розріджений`. |
| abort | "перервати" (button **Перервати**) | "скасувати" is cancel/undo only. |
| scroll wheel | "коліщатко" | Not "колесо". |
| dropdown | "випадаючий список" | Not "випадний". |
| summary / description fields | **Заголовок** / **Опис** | The composer labels; English "Summary" is not left in prose. |
| follow (a selection, the tab folder) | "стежити за", "відповідати", "залежати від" | Never "слідувати". |
| reopen as "binary" | "двійковий файл" | Never "бінарний". |
| page names in links | "Staging і commit'и", "Огляд інтерфейсу", "Робота з Log", "Скорочення клавіатури", "Допомога ШІ", "Скасування й повторення", "Усунення несправностей і FAQ" | Keep the link text equal to the page's H1. |

## Undo history, Reflog, Lost work (2026-10)

| Concept | Ukrainian | Notes |
|---|---|---|
| Undo history | «Історія скасувань» | panel, tab and palette entry ("Правка: історія скасувань...") |
| Undo up to here / Redo up to here | «Скасувати до цього місця» / «Повторити до цього місця» | |
| Restore only this | «Відновити лише це» | the single-entry restore; the CLI keeps `undo-restore` |
| Reflog | Reflog (Latin, git term) | `Reflog...`, `Правка: Reflog...` |
| Lost work | «Втрачена робота» | the panel; its palette entry is "Правка: відновити втрачені commit'и..." (`{$commits}` pair: commit'и / коміти) |
| undo pin | «закріплення скасування» | hidden ref that keeps an undone object alive |
| dangling | «висячий» | git-l10n |
| blocked / undone / restorable | «заблоковано» / «скасовано» / «можна відновити окремо» | row tags |
| `<op> done - <key> to undo` | «{op} виконано - {key}, щоб скасувати» | `{op}` is a lower-case verbal noun ("видалення гілки"); the git-word ones are `{git, local}` pairs ("видалення tag'а" / "видалення тега") |
| undo (verb in prose of these panels) | «скасувати» | never "відкотити" in labels; "відкотити" is allowed only as a plain verb for rolling a ref back |

Palette twins of the row actions (2026-10): `Історія скасувань: ...`, `Reflog: ...`, `Втрачена робота: ...`, the category is the panel name, the action follows the colon in lower case ("сканувати знову" = Scan Again, "оновити" = Refresh, "відновити на цю позицію" = Restore to This Position). `sixel` stays Latin: "зображення sixel".

## Outside changes, merge revert, undo row labels (2026-10)

| Concept | Ukrainian | Notes |
|---|---|---|
| outside (tag) | «зовнішнє» | row tag; status line "зовнішню зміну записано - {key}, щоб скасувати" |
| made outside GitPoulpi | «поза GitPoulpi» | |
| revert of a merge commit | «Revert коміту злиття» (local) / «Revert merge-commit'а» (git) | parent = «батьківський» (git-l10n), "основна лінія" = mainline |
| `[undo_label]` | action word = the catalog's own (Відкинути, Відновити, Створити/Видалити гілку, Задати, Скинути, Перервати, Вирішити); stage/unstage/merge/rebase/reword via `{$Term}` | stash/pop/apply/drop rows are `{git, local}` pairs ("Повернути схованку", not `{$Pop} {$stash}`: the local noun is nominative); commit plurals are explicit pairs (commit'и / коміти, many: commit'ів / комітів); `amend`, `checkout`, `revert`, `reset`, `upstream`, `remote`, `hunk`, `worktree` stay Latin |
| old tip | «колишня верхівка» | |
| Yesterday | «Учора» | |

## Workspaces, search, Radar (2026-10)

| Concept | Ukrainian | Notes |
|---|---|---|
| workspace | «робочий простір» (plural «робочі простори») | one term everywhere (catalog, help, guide); never "середовище" or "простір" alone. The tab-bar chip is "Немає робочого простору" |
| switch vs edit a workspace | «перемкнути» vs «змінити» | "Змінити" is edit only; switching is "Перемкнути робочий простір" |
| Fetch all / Pull all | `{ git, local }` pairs: «Fetch все» / «Завантажити все», «Pull все» / «Отримати все» | the verbal-noun `[term]` form does not fit "all" ("завантаження все"), so the local form is the infinitive; progress lines use "завантаження всього в {name}" |
| issues in Radar | «Issues», «Issues, призначені мені», "над issue" | `issue` stays Latin (glossary), never "Завдання" |
| pin / snooze | «Закріпити вгорі» / «Відкріпити»; «Відкласти...» / «Більше не відкладати», tab «Відкладені» | |
| check out a branch (Radar) | «Checkout гілки» | checkout stays Latin |
| Radar status line | verbal noun, no first person: «опитування GitHub і GitLab...» | |
| palette rows | lower case after the colon: «Radar: відкрити», «Робочий простір: новий...», «Вкладка: нова вкладка» | git words keep their capital: «Робочий простір: Fetch все» |
| folder in these strings | «папка» ("Додати папку...") | not "тека" |

## Review, metadata, board, local issues, Gerrit (round 4, 2026-10)

| Concept | Ukrainian | Notes |
|---|---|---|
| code review / review (n.) | «рев'ю» (indeclinable) | already used by `pr_terminology` ("запит на рев'ю"); palette category `Рев'ю:`, "Завершити рев'ю", "незавершене рев'ю". Never "рецензія" |
| reviewer / assignee | «рев'юер» (pl. рев'юери) / «виконавець» (pl. виконавці) | GitLab uk "Виконавець"; "призначено" only as a participle ("призначені мені") |
| label / milestone | «мітка» / «віха» | instrumental "віхою", never "вехою" |
| request (generic PR/MR/change) | «запит» | unchanged from the PR term |
| thread / conversation | «обговорення» / «бесіда» | `гілка` is reserved for branch, so a comment thread is never "гілка коментарів". "вирішити / відкрити знову обговорення" |
| pending comment / pending review | «незавершений коментар» / «незавершене рев'ю» | plural few "незавершені коментарі", many "незавершених коментарів" |
| verdict | «вердикт» | Comment = «Коментар», Approve = «Схвалити», Request changes = «Вимагати змін»; badges «схвалено», «вимагає змін», «прокоментовано», «відхилено» |
| Submit / Finish / Discard review | «Подати рев'ю» / «Завершити рев'ю» / «Відкинути рев'ю» | "Подати" (not "Надіслати") so it does not read like push |
| suggestion (AI) | «пропозиція» | Accept «Прийняти», Edit «Змінити», Dismiss «Відхилити»; severities «інформація», «попередження», «проблема»; chip text sent to the model stays English |
| patch set / vote | «набір патчів» / «голос» | Gerrit labels (`Code-Review`, `Verified`) stay Latin |
| checks (CI) | «перевірки» | «перевірки пройдено / не пройдено / виконуються» |
| Mentioning me | «Згадують мене» | Radar group and filter |
| encoding | «кодування» | «Автовизначення» for Auto-detect, button «Авто» |
| board / column / card | «дошка» / «колонка» / «картка» | Trello and Jira uk wording; "Дошка issues"; column labels = «мітки колонок»; «Відкриті» / «Закриті» for the two fixed columns |
| notes folder / vault | «папка нотаток» / «сховище Obsidian» | local issues; not to be confused with stash («схованка») |
| local issues | «локальні issues» | palette category `Локальні issues:`; provider name «Локальний Markdown» |
| push for review (Gerrit) | `{$Push}` + «на рев'ю» | "Хостинг: Push на рев'ю..." |
| `fetch` progress in review | «fetch коду запиту...» | git-mode lint needs the stem; no verbal noun |

## Branching workflows, Mermaid, Gerrit follow-ups (round 5, 2026-10)

| Concept | Ukrainian | Notes |
|---|---|---|
| branching workflow / workflow | «робочий процес» (гілок) | whole definition; settings page «Робочий процес»; never "воркфлоу" |
| `Flow` | `Flow` (Latin) | toolbar button and palette category `Flow:`; lower case after the colon ("Flow: почати гілку...") |
| base branch / protected / environment branch | «базова гілка» / «захищена» / «гілка середовища» | badge `[базова]`, `[базова, захищена]` |
| topic type / topic branch | «тип гілки» / «тематична гілка» | the type's `label` row is «Підпис» (not «Мітка», which is label/tag of issues) |
| parent branch | «батьківська гілка» (short «батьківська») | "Задати батьківську...", "Нова батьківська" |
| start / finish / sync / promote | «почати» / «завершити» / «синхронізувати» / «просунути» | buttons: Почати, Завершити, Синхронізувати, Просунути; dialog titles are verbal nouns («Завершення {branch}») |
| pick up (a host or bot branch) | «підхопити» (reference field «Перейняти») | |
| hook | «хук» (pl. хуки) | "Після початку" / "Перед завершенням" |
| issue flow (host-driven) | «потік issue (керований хостом)», guide: «потоки під керуванням хоста» | |
| host | «хост» | the server side of an integration, as in the existing error texts |
| slug | `Slug` (Latin) | token name |
| name pattern / token | «шаблон назви» / «токен» | |
| Mermaid detail levels | Структура / Refs / Заголовки / Повний | `Refs` stays Latin; "Заголовки" = commit subjects (composer label «Заголовок»); config values stay `structure`, `refs`, `subjects`, `full` |
| Mermaid Direction | «Напрямок» («Зліва направо» / «Зверху вниз») | |
| Gerrit Submit / Abandon / Compare | «Влити» / «Покинути» / «Порівняння» | "Подати" stays reserved for submitting a review; a Gerrit change is «зміна» |
| fast-forward merge strategy | `{ git = "fast-forward", local = "перемотування" }` | |
