[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · **Українська** · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP та агенти

ШІ-агент (Claude Code, Cursor, Codex, ...) може керувати GitPoulpi через свій
**сервер MCP** або **CLI**. Обидва проходять через той самий фоновий демон, що й
вікно, тож те, що робить агент, наживо з'являється у вашому відкритому GitPoulpi,
а обидва виконують той самий код для кожної команди.

Ця сторінка стосується *зовнішніх* агентів. Власні ШІ-кнопки GitPoulpi
(повідомлення commit'ів, пояснення, композер commit'ів) налаштовуються окремо:
див. [Допомога ШІ](ai.md).

## Швидке встановлення

1. Додайте `gitpoulpi` і `gitpoulpi-mcp` до свого `PATH` ([Встановлення](installation.md)).
2. Зареєструйте сервер у свого агента. Він говорить через stdio: нічого запускати
   не потрібно, жодного порту.
3. За бажанням виберіть, що агент може робити, у таблиці `[mcp]` файлу
   `config.toml` (нижче). Типово він може читати репозиторій (зміни відхиляються, доки ви не задасте
   `allow_mutations = true`), показувати вам щось лише після вашого підтвердження
   і ніколи не заповнює діалоги наперед.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` натомість записує це в `.mcp.json` репозиторію, тож його
отримують усі, хто відкриває проєкт. Той самий файл уручну, з необов'язковими
додатками:

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "command": "gitpoulpi",
      "args": ["mcp", "serve", "--stdio", "--ui-control", "auto"],
      "env": { "GITPOULPI_CONFIG_DIR": "/home/me/.config/gitpoulpi-agent" }
    }
  }
}
```

- `--ui-control auto|ask|off` замінює `[mcp] ui_control` лише для цього агента.
- `GITPOULPI_CONFIG_DIR` дає агентові власний `config.toml` (власні обмеження
  `[mcp]`, без доступу до ваших налаштувань). Пропустіть, щоб ділити ваші.
- `gitpoulpi --repo <path> mcp serve --stdio` прив'язує сервер до одного репозиторію;
  інакше інструмент діє на репозиторій, що охоплює робочу папку сервера, або на
  аргумент `repo` виклику.

**Інші клієнти**: будь-який клієнт MCP, що запускає stdio-сервер.

| Клієнт | Де | Що |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | JSON-файл MCP клієнта | блок `mcpServers` вище (у VS Code ключ називається `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Будь-який інший | його документація | команда `gitpoulpi-mcp` (окремий двійковий файл, той самий сервер, аргументи не потрібні) |

**Через HTTP** (клієнти, що підключаються за URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

Безстановий JSON, ті самі інструменти, ті самі обмеження `[mcp]`. Кожен запит
потребує bearer-токена (`[mcp] http_auth`, типово ввімкнено): виведіть його через
`gitpoulpi mcp token --show` (або `--copy`, `--rotate`; токен лежить у файлі лише
для користувача в приватній теці даних) і передайте клієнту як заголовок:

```
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp \
  --header "Authorization: Bearer <token>"
```

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "type": "http",
      "url": "http://127.0.0.1:8808/mcp",
      "headers": { "Authorization": "Bearer <token>" }
    }
  }
}
```

Відсутній чи хибний токен отримує `401` із `WWW-Authenticate: Bearer`. Замінений
токен діє одразу, без перезапуску сервера. Вимикати `http_auth` варто для точки на
loopback, де ви довіряєте кожному локальному обліковому запису; сервер тоді
попереджає в stderr.

### Доступ до точки з іншої машини

Найбезпечніше налаштування взагалі не потребує віддаленого слухача: лишіть адресу
loopback і прокиньте порт через SSH з машини, на якій працює агент,
`ssh -L 8808:127.0.0.1:8808 host`. Тоді `--allow-remote` не залучається.

`--allow-remote` (адреса не з loopback) відхиляється, якщо автентифікацію не
ввімкнено з токеном на місці **і** трафік не зашифровано. Відмова називає, чого
бракує. Є два способи бути зашифрованим:

- **TLS у сервері** (`[mcp] tls = "self_signed"` або `"files"`). Сервер сам
  завершує TLS: він тримає ключ, виконує рукостискання, і до шару HTTP доходить
  лише завершене рукостискання; клієнт зі звичайним HTTP зазнає невдачі на
  рукостисканні, а звичайного слухача поряд немає. Тож шифрування - це факт, який
  сервер встановлює на кожному з'єднанні, а не припущення. З `self_signed` перший
  запуск створює пару в приватній теці даних і виводить її відбиток SHA-256 (він
  є й у Налаштуваннях); клієнт довіряє серверу, закріплюючи цей відбиток чи
  файл сертифіката. З `files` задайте `tls_cert` і `tls_key` як ваші файли PEM, і
  клієнти перевіряють сертифікат за його ЦС. Клієнти автентифікуються bearer-токеном;
  клієнтські сертифікати не використовуються.
- **Тунель або зворотний проксі** (`[mcp] behind_proxy = true`). Агент тунелю чи
  проксі підключається як будь-який локальний чи LAN-клієнт, а його заголовки
  можна підробити, тож сервер зсередини не може сказати, чи він там є. Тому це
  явне підтвердження (показане з червоним попередженням у Налаштуваннях), а не
  виявлення. Запускайте тунель на тій самій машині й, коли можете, лишайте адресу
  loopback. З адресою не з loopback за проксі ділянка між проксі та сервером
  передається відкритим текстом у тій мережі.

**Без MCP** агент із доступом до shell користується CLI: `gitpoulpi --json <команда>`
(див. [довідник CLI](cli-reference.md)). Дайте йому готовий навик (skill)
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), що підказує, коли
й як користуватися GitPoulpi замість «голого» `git`.

## Дозволи

| Перемикач | Типово | Дія |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` відмовляються запускатися (Налаштування > Агенти й MCP > *Сервер MCP*). Зчитується під час запуску сервера: запущений сервер зберігає свої налаштування. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Адреса, яку `--http` використовує, коли жодної не вказано (лише loopback, окрім `--allow-remote`, що потребує автентифікації й шифрування, див. вище). Налаштування > Агенти й MCP > *Порт сервера MCP* змінює його порт і зберігає хост. |
| `[mcp] http_auth` | `true` | `--http` потребує `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Налаштування > Агенти й MCP > *HTTP-автентифікація*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` або `files` (ваші шляхи PEM): сервер сам завершує TLS. Налаштування > Агенти й MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Ви ручаєтеся, що тунель або зворотний проксі шифрує трафік, що дозволяє `--allow-remote` працювати без TLS. |
| `[mcp] allow_mutations` | `false` | Вимкнено: кожен інструмент, що щось змінює, відхиляється; читання, `ui_*` і пропозиції все ще працюють. `true` дозволяє агентові робити commit, push, reset... |
| `[mcp] mutation_allowlist` | `[]` | Не порожній: лише ці інструменти можуть щось змінювати, напр. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: агент не може рухати ваше вікно (`ui_state` / `ui_viewers` все ще відповідають). |
| `[mcp] ui_control` | `"ask"` | `ask`: у вашому вікні з'являється чип, який ви підтверджуєте; `auto`: застосовується, якщо ви не зайняті; `off`: відхиляється. Для окремого агента: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Дозволяє агентові відкривати наперед заповнені діалоги, які ви підтверджуєте (Налаштування > Агенти й MCP > *Дозволити ШІ-агентам пропонувати зміни на моє підтвердження*). |
| `[mcp] allow_sensitive_staging` | `false` | Лише в `config.toml`. Вимкнено: інструмент `stage` не пропонує `allow_sensitive`, а додавання, що внесло б чутливий файл (приватний ключ, `.env`, розпізнаний токен), відхиляється з `{error: "sensitive_files", files}`: агент має спитати вас. `true` дозволяє йому передати `allow_sensitive`. Див. [Staging і commit'и](staging-and-committing.md#чутливі-файли). |
| `[mcp] allow_history_purge` | `false` | Лише в `config.toml`. `true` показує інструмент `rewrite_purge`, що остаточно видаляє стару історію переписування (і її скасування). Вимкнено: агент його не бачить. Йому все одно потрібен `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | Загальний перемикач вашого вікна (Налаштування > Агенти й MCP > *Дозволити ШІ-агентам керувати виглядом*). Вимкнено: жоден агент не може дістатися вашого вікна. |
| `[ui] launch` | `"off"` | Чи може агент відкрити вікно GitPoulpi, коли жодного не відкрито (див. [нижче](#коли-жодного-вікна-не-відкрито-або-воно-в-іншій-сесії)). |

Що це означає на практиці:

- **З коробки агент лише читає**: він може читати, показувати вам щось у вашому
  вікні (`ui_*`) і, якщо ви дозволите, пропонувати зміни, які ви підтверджуєте. Щоб
  дозволити йому змінювати репозиторій (commit, push, reset...), задайте
  `allow_mutations = true`; для вузького дозволу залиште `true` разом із
  `mutation_allowlist`. Кожен інструмент має анотації MCP (`readOnlyHint`,
  `destructiveHint`, `openWorldHint`), тож клієнт на кшталт Claude Code все одно
  питає вас перед найризикованішими.
- **Конфлікти**: `conflicts_check` (прогноз, ще до будь-якого merge), `conflicts_list`, `conflicts_show` і `conflicts_state` - це читання;
  `conflicts_resolve` (відкидає сторону, яку не збережено, `--delete` видаляє файл) і
  `conflicts_abort` (скасовує весь merge/rebase/cherry-pick) руйнівні;
  `conflicts_unresolve` - звичайний запис. Відхилений виклик (`stale`,
  `not_conflicted`, `pick_incomplete`...) відповідає `{"refused": {...}}` із кодом
  завершення 2 замість збою.
- **Застряглі стани**: `conflicts_am_continue` - запис, `conflicts_am_skip`,
  `conflicts_am_abort` і `index_lock_remove` руйнівні, `index_lock_show` - читання.
- **Переписування історії**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (аналіз розмірів), `rewrite_scan_secrets` (знахідки з прихованим фрагментом, ніколи значення),
  `rewrite_script_check`, `rewrite_script_test` і `rewrite_discard` - читання
  (перегляд нічого не змінює: він лише зберігає план у демоні та на диску в
  git-каталозі, тож `rewrite_apply` працює й після перезапуску). `rewrite_apply` і
  `rewrite_run` руйнівні: вони переміщують багато ref на переписані коміти одним
  записом скасування (`undo` повертає їх) і нічого не надсилають. Рецепт передавайте
  як `recipe_json` (або `recipe` - шлях), а секрети - через `secrets_file`, ніколи не
  вбудовуйте їх у виклик.
  Скрипти shell, Python і зовнішньої Lua (`script_language`) працюють без пісочниці,
  і **агент не може їм довіряти**: інструмента `rewrite_script_trust` і аргументу
  `trust` немає, тож такий скрипт запускається лише після того, як людина виконала
  `gitpoulpi rewrite script trust FILE` для цього точного тексту (змінений скрипт
  знову відхиляється, `script_not_trusted` з хешем у `refused.script`), і лише коли
  `rewrite.external_scripts` вмикає його мову (`scripts_disabled`).
  Відмова відповідає `{"refused": {"reason", "message", "detail"}}`,
  а план, що переміщує вже надіслані ref, відповідає `{"needs_confirmation": ...}`
  (код завершення 3), доки виклик не задасть `yes` - спершу покажіть це користувачеві.
- **Ніколи не доступне агентові**: збереження токена (`hosting login`), створення й
  надсилання ключа SSH, `config set` (він міг би переписати власні обмеження) та
  життєвий цикл демона. Токен агентові ніколи не показується.
- **Списки на рівні облікового запису**: `hosting_radar` - це читання (запити, що чекають на ваше
  рев'ю, відкриті запити, призначені issues у всіх репозиторіях профілю,
  з п'ятихвилинного кешу, якщо не `refresh`); `hosting_integrations_radar`
  змінює таблицю `[integrations.radar]` профілю, тож його контролює `allow_mutations`
  (навіть коли викликано без параметрів, що лише друкує налаштування).
- **Контекст рев'ю**: `ai_review_context` показує, що чернетка рев'ю коду отримує
  крім diff (CLAUDE.md, AGENTS.md, додатковий файл, де запускається локальний агент). З
  аргументом налаштування він зберігає типове значення (глобальне або власне `repo` чи `project`),
  тож його контролює `allow_mutations`, як і інші інструменти, що змінюють налаштування; без нього
  лише друкує. Як і кожен інструмент `ai_*`, він у списку лише поки `ai.enabled` увімкнено.
  `hosting_review_suggest` приймає ті самі аргументи лише для одного запуску.
- **Обмеження `[mcp]` стосуються сервера MCP.** CLI - це ваш власний shell, і
  `allow_mutations` його не контролює; лише `ui_control` і `allow_ui_propose`
  діють також на `gitpoulpi ui ...`.
- Відхилений виклик - звичайна помилка інструмента (`isError: true`, причина в
  `structuredContent.error`), а не мовчазна відсутність дії.

```toml
[mcp]
allow_mutations = false        # true: the agent may change the repository
mutation_allowlist = []        # e.g. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" lets the agent move your view without asking
allow_ui_propose = false       # true: the agent may pre-fill Reword / Split / commit / branch / tag dialogs
allow_sensitive_staging = false  # true: the agent may stage files the staging guard flags
allow_history_purge = false    # true: the agent sees rewrite_purge (deletes old history for good)
```

## Інструменти

Один інструмент на команду CLI, названий за її шляхом із `_` (`branch create`
стає `branch_create`). Кожен приймає аргументи команди плюс `repo` (репозиторій;
типово репозиторій сервера) і `session`. `structuredContent` успішного виклику - 
це точно те, що виводить `gitpoulpi --json <команда>`. **Kind** (стовпець «Kind»,
англійською, як на сервері): *read* ніколи нічого не змінює; *write* і
*destructive* змінюють репозиторій (контролюються обмеженнями); *network* звертається до remote'а чи
хостингу; *view* лише рухає ваше вікно; *propose* відкриває діалог, який ви
підтверджуєте.

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `ai_worktree_name` | `gitpoulpi ai worktree-name` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
| `apply_commit` | `gitpoulpi apply-commit` | write |
| `autostash_start_over` | `gitpoulpi autostash start-over` | destructive |
| `blame` | `gitpoulpi blame` | read |
| `branch_create` | `gitpoulpi branch create` | write |
| `branch_delete` | `gitpoulpi branch delete` | destructive |
| `branch_delete_remote` | `gitpoulpi branch delete-remote` | destructive, network |
| `branch_fast_forward` | `gitpoulpi branch fast-forward` | write |
| `branch_list` | `gitpoulpi branch list` | read |
| `branch_rename` | `gitpoulpi branch rename` | write |
| `branch_set_upstream` | `gitpoulpi branch set-upstream` | write |
| `branch_track` | `gitpoulpi branch track` | write |
| `branch_unset_upstream` | `gitpoulpi branch unset-upstream` | write |
| `branch_upstream_add` | `gitpoulpi branch upstream add` | write |
| `branch_upstream_list` | `gitpoulpi branch upstream list` | read |
| `branch_upstream_primary` | `gitpoulpi branch upstream primary` | write |
| `branch_upstream_pull` | `gitpoulpi branch upstream pull` | write |
| `branch_upstream_remove` | `gitpoulpi branch upstream remove` | write |
| `checkout` | `gitpoulpi checkout` | write |
| `cherry_pick` | `gitpoulpi cherry-pick` | write |
| `clone` | `gitpoulpi clone` | write, network |
| `commit` | `gitpoulpi commit` | write |
| `config_get` | `gitpoulpi config get` | read |
| `config_path` | `gitpoulpi config path` | read |
| `conflicts_abort` | `gitpoulpi conflicts abort` | destructive |
| `conflicts_am_abort` | `gitpoulpi conflicts am-abort` | destructive |
| `conflicts_am_continue` | `gitpoulpi conflicts am-continue` | write |
| `conflicts_am_skip` | `gitpoulpi conflicts am-skip` | destructive |
| `conflicts_check` | `gitpoulpi conflicts check` | read |
| `conflicts_list` | `gitpoulpi conflicts list` | read |
| `conflicts_resolve` | `gitpoulpi conflicts resolve` | destructive |
| `conflicts_show` | `gitpoulpi conflicts show` | read |
| `conflicts_skip` | `gitpoulpi conflicts skip` | destructive |
| `conflicts_state` | `gitpoulpi conflicts state` | read |
| `conflicts_unresolve` | `gitpoulpi conflicts unresolve` | write |
| `convert` | `gitpoulpi convert` | write |
| `diff` | `gitpoulpi diff` | read |
| `discard` | `gitpoulpi discard` | destructive |
| `fetch` | `gitpoulpi fetch` | write, network |
| `file_history` | `gitpoulpi file-history` | read |
| `flow_config_init` | `gitpoulpi flow config init` | write |
| `flow_config_show` | `gitpoulpi flow config show` | read |
| `flow_detect` | `gitpoulpi flow detect` | read |
| `flow_finish` | `gitpoulpi flow finish` | destructive, network |
| `flow_import` | `gitpoulpi flow import` | write |
| `flow_issue_new` | `gitpoulpi flow issue new` | write, network |
| `flow_issue_wait` | `gitpoulpi flow issue wait` | write, network |
| `flow_pick_up` | `gitpoulpi flow pick-up` | write, network |
| `flow_promote` | `gitpoulpi flow promote` | destructive |
| `flow_set_parent` | `gitpoulpi flow set-parent` | write |
| `flow_start` | `gitpoulpi flow start` | write, network |
| `flow_status` | `gitpoulpi flow status` | read |
| `flow_sync` | `gitpoulpi flow sync` | destructive |
| `flow_templates` | `gitpoulpi flow templates` | read |
| `hosting_board_move` | `gitpoulpi hosting board move` | write, network |
| `hosting_board_show` | `gitpoulpi hosting board show` | read, network |
| `hosting_checkout_pr` | `gitpoulpi hosting checkout-pr` | write, network |
| `hosting_clone` | `gitpoulpi hosting clone` | write, network |
| `hosting_edit_url` | `gitpoulpi hosting edit-url` | read, network |
| `hosting_file` | `gitpoulpi hosting file` | read, network |
| `hosting_integrations_add` | `gitpoulpi hosting integrations add` | write |
| `hosting_integrations_list` | `gitpoulpi hosting integrations list` | read |
| `hosting_integrations_radar` | `gitpoulpi hosting integrations radar` | write |
| `hosting_integrations_remove` | `gitpoulpi hosting integrations remove` | destructive |
| `hosting_integrations_test` | `gitpoulpi hosting integrations test` | read, network |
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
| `hosting_issue_create` | `gitpoulpi hosting issue create` | write, network |
| `hosting_issue_edit` | `gitpoulpi hosting issue edit` | write, network |
| `hosting_issue_move` | `gitpoulpi hosting issue move` | write |
| `hosting_issue_options` | `gitpoulpi hosting issue options` | read, network |
| `hosting_issue_reopen` | `gitpoulpi hosting issue reopen` | write |
| `hosting_issues` | `gitpoulpi hosting issues` | read, network |
| `hosting_labels` | `gitpoulpi hosting labels` | read, network |
| `hosting_link` | `gitpoulpi hosting link` | write |
| `hosting_list` | `gitpoulpi hosting list` | read |
| `hosting_logout` | `gitpoulpi hosting logout` | destructive |
| `hosting_members` | `gitpoulpi hosting members` | read, network |
| `hosting_milestones` | `gitpoulpi hosting milestones` | read, network |
| `hosting_open` | `gitpoulpi hosting open` | read |
| `hosting_pr` | `gitpoulpi hosting pr` | write, network |
| `hosting_pr_edit` | `gitpoulpi hosting pr-edit` | write, network |
| `hosting_pr_for_branch` | `gitpoulpi hosting pr-for-branch` | read, network |
| `hosting_projects` | `gitpoulpi hosting projects` | read, network |
| `hosting_prs` | `gitpoulpi hosting prs` | read, network |
| `hosting_radar` | `gitpoulpi hosting radar` | read, network |
| `hosting_refs` | `gitpoulpi hosting refs` | read |
| `hosting_review_abandon` | `gitpoulpi hosting review abandon` | write, network |
| `hosting_review_comment` | `gitpoulpi hosting review comment` | write, network |
| `hosting_review_diff` | `gitpoulpi hosting review diff` | read, network |
| `hosting_review_fetch` | `gitpoulpi hosting review fetch` | write, network |
| `hosting_review_push` | `gitpoulpi hosting review push` | write, network |
| `hosting_review_reply` | `gitpoulpi hosting review reply` | write, network |
| `hosting_review_resolve` | `gitpoulpi hosting review resolve` | write, network |
| `hosting_review_show` | `gitpoulpi hosting review show` | read, network |
| `hosting_review_submit` | `gitpoulpi hosting review submit` | write, network |
| `hosting_review_submit_change` | `gitpoulpi hosting review submit-change` | write, network |
| `hosting_review_suggest` | `gitpoulpi hosting review suggest` | read, network |
| `hosting_review_threads` | `gitpoulpi hosting review threads` | read, network |
| `hosting_ssh_key_delete` | `gitpoulpi hosting ssh-key delete` | destructive, network |
| `hosting_ssh_key_list` | `gitpoulpi hosting ssh-key list` | read, network |
| `hosting_use` | `gitpoulpi hosting use` | write |
| `ignore` | `gitpoulpi ignore` | write |
| `index_lock_remove` | `gitpoulpi index-lock remove` | destructive |
| `index_lock_show` | `gitpoulpi index-lock show` | read |
| `job_cancel` | `gitpoulpi job cancel` | write |
| `job_list` | `gitpoulpi job list` | read |
| `job_wait` | `gitpoulpi job wait` | read |
| `lfs_checkout` | `gitpoulpi lfs checkout` | write |
| `lfs_fetch` | `gitpoulpi lfs fetch` | write, network |
| `lfs_files` | `gitpoulpi lfs files` | read |
| `lfs_install` | `gitpoulpi lfs install` | write |
| `lfs_lock` | `gitpoulpi lfs lock` | write, network |
| `lfs_locks` | `gitpoulpi lfs locks` | read, network |
| `lfs_prune` | `gitpoulpi lfs prune` | destructive |
| `lfs_pull` | `gitpoulpi lfs pull` | write, network |
| `lfs_push` | `gitpoulpi lfs push` | destructive, network |
| `lfs_status` | `gitpoulpi lfs status` | read |
| `lfs_track` | `gitpoulpi lfs track` | write |
| `lfs_unlock` | `gitpoulpi lfs unlock` | destructive, network |
| `lfs_untrack` | `gitpoulpi lfs untrack` | write |
| `licenses` | `gitpoulpi licenses` | read |
| `log` | `gitpoulpi log` | read |
| `lost_commits` | `gitpoulpi lost-commits` | read |
| `mailmap_add` | `gitpoulpi mailmap add` | write |
| `mailmap_check` | `gitpoulpi mailmap check` | read |
| `mailmap_list` | `gitpoulpi mailmap list` | read |
| `merge` | `gitpoulpi merge` | write |
| `patch_apply` | `gitpoulpi patch apply` | write |
| `patch_create` | `gitpoulpi patch create` | write |
| `path_status` | `gitpoulpi path status` | read |
| `pull` | `gitpoulpi pull` | write, network |
| `push` | `gitpoulpi push` | destructive, network |
| `rebase` | `gitpoulpi rebase` | destructive |
| `rebase_plan` | `gitpoulpi rebase-plan` | read |
| `redo` | `gitpoulpi redo` | destructive |
| `reflog` | `gitpoulpi reflog` | read |
| `reflog_restore` | `gitpoulpi reflog-restore` | destructive |
| `refs_hide` | `gitpoulpi refs hide` | write |
| `refs_show` | `gitpoulpi refs show` | write |
| `refs_solo` | `gitpoulpi refs solo` | write |
| `remote_add` | `gitpoulpi remote add` | write |
| `remote_clear_push_url` | `gitpoulpi remote clear-push-url` | write |
| `remote_default` | `gitpoulpi remote default` | write |
| `remote_list` | `gitpoulpi remote list` | read |
| `remote_remove` | `gitpoulpi remote remove` | destructive |
| `remote_rename` | `gitpoulpi remote rename` | write |
| `remote_set_url` | `gitpoulpi remote set-url` | write |
| `remote_group_abort` | `gitpoulpi remote-group abort` | destructive |
| `remote_group_continue` | `gitpoulpi remote-group continue` | destructive, network |
| `remote_group_plan` | `gitpoulpi remote-group plan` | read, network |
| `remote_group_retry` | `gitpoulpi remote-group retry` | write, network |
| `remote_group_status` | `gitpoulpi remote-group status` | read |
| `repo_close` | `gitpoulpi repo close` | write |
| `repo_discover` | `gitpoulpi repo discover` | read |
| `repo_init` | `gitpoulpi repo init` | write |
| `repo_list` | `gitpoulpi repo list` | read |
| `repo_open` | `gitpoulpi repo open` | write |
| `repo_recent` | `gitpoulpi repo recent` | read |
| `repo_auth_show` | `gitpoulpi repo-auth show` | read |
| `repo_auth_test` | `gitpoulpi repo-auth test` | write, network |
| `repo_config_set` | `gitpoulpi repo-config set` | write |
| `repo_config_show` | `gitpoulpi repo-config show` | read |
| `repo_config_unset` | `gitpoulpi repo-config unset` | write |
| `reset` | `gitpoulpi reset` | destructive |
| `revert` | `gitpoulpi revert` | write |
| `reword` | `gitpoulpi reword` | destructive |
| `rewrite_analyze` | `gitpoulpi rewrite analyze` | read |
| `rewrite_apply` | `gitpoulpi rewrite apply` | destructive |
| `rewrite_check_signing` | `gitpoulpi rewrite check-signing` | read |
| `rewrite_diff` | `gitpoulpi rewrite diff` | read |
| `rewrite_discard` | `gitpoulpi rewrite discard` | read |
| `rewrite_maps` | `gitpoulpi rewrite maps` | write |
| `rewrite_note` | `gitpoulpi rewrite note` | read |
| `rewrite_preview` | `gitpoulpi rewrite preview` | read |
| `rewrite_purge` | `gitpoulpi rewrite purge` | destructive |
| `rewrite_push` | `gitpoulpi rewrite push` | destructive, network |
| `rewrite_rows` | `gitpoulpi rewrite rows` | read |
| `rewrite_run` | `gitpoulpi rewrite run` | destructive |
| `rewrite_scan_secrets` | `gitpoulpi rewrite scan-secrets` | read |
| `rewrite_script_check` | `gitpoulpi rewrite script check` | read |
| `rewrite_script_test` | `gitpoulpi rewrite script test` | read |
| `rewrite_tools` | `gitpoulpi rewrite tools` | read |
| `rm_file` | `gitpoulpi rm-file` | destructive |
| `search` | `gitpoulpi search` | read |
| `session_delete` | `gitpoulpi session delete` | destructive |
| `session_list` | `gitpoulpi session list` | read |
| `session_new` | `gitpoulpi session new` | write |
| `session_rename` | `gitpoulpi session rename` | write |
| `show` | `gitpoulpi show` | read |
| `sparse_checkout_add` | `gitpoulpi sparse-checkout add` | write |
| `sparse_checkout_disable` | `gitpoulpi sparse-checkout disable` | write |
| `sparse_checkout_set` | `gitpoulpi sparse-checkout set` | write |
| `sparse_checkout_status` | `gitpoulpi sparse-checkout status` | read |
| `split_commit` | `gitpoulpi split-commit` | destructive |
| `split_source` | `gitpoulpi split-source` | read |
| `ssh_command_clear` | `gitpoulpi ssh-command clear` | write |
| `ssh_command_set` | `gitpoulpi ssh-command set` | write |
| `ssh_command_show` | `gitpoulpi ssh-command show` | read |
| `stage` | `gitpoulpi stage` | write |
| `stash_apply` | `gitpoulpi stash apply` | write |
| `stash_drop` | `gitpoulpi stash drop` | destructive |
| `stash_from_commit` | `gitpoulpi stash from-commit` | write |
| `stash_list` | `gitpoulpi stash list` | read |
| `stash_pop` | `gitpoulpi stash pop` | write |
| `stash_push` | `gitpoulpi stash push` | write |
| `status` | `gitpoulpi status` | read |
| `submodule_init` | `gitpoulpi submodule init` | write |
| `submodule_list` | `gitpoulpi submodule list` | read |
| `submodule_update` | `gitpoulpi submodule update` | write, network |
| `tag_annotate` | `gitpoulpi tag annotate` | write |
| `tag_create` | `gitpoulpi tag create` | write |
| `tag_delete` | `gitpoulpi tag delete` | destructive, network |
| `tag_delete_remote` | `gitpoulpi tag delete-remote` | destructive, network |
| `tag_fetch` | `gitpoulpi tag fetch` | write, network |
| `tag_list` | `gitpoulpi tag list` | read |
| `tag_push` | `gitpoulpi tag push` | write, network |
| `theme_export` | `gitpoulpi theme export` | read |
| `theme_list` | `gitpoulpi theme list` | read |
| `treat` | `gitpoulpi treat` | write |
| `ui_back` | `gitpoulpi ui back` | view |
| `ui_highlight` | `gitpoulpi ui highlight` | view |
| `ui_notify` | `gitpoulpi ui notify` | view |
| `ui_propose_branch` | `gitpoulpi ui propose branch` | propose |
| `ui_propose_commit_message` | `gitpoulpi ui propose commit-message` | propose |
| `ui_propose_resolution` | `gitpoulpi ui propose resolution` | propose |
| `ui_propose_reword` | `gitpoulpi ui propose reword` | propose |
| `ui_propose_rewrite` | `gitpoulpi ui propose rewrite` | propose |
| `ui_propose_split` | `gitpoulpi ui propose split` | propose |
| `ui_propose_tag` | `gitpoulpi ui propose tag` | propose |
| `ui_show_commit` | `gitpoulpi ui show commit` | view |
| `ui_show_conflict` | `gitpoulpi ui show conflict` | view |
| `ui_show_diff` | `gitpoulpi ui show diff` | view |
| `ui_show_file` | `gitpoulpi ui show file` | view |
| `ui_show_home` | `gitpoulpi ui show home` | view |
| `ui_show_issue` | `gitpoulpi ui show issue` | view |
| `ui_show_keys` | `gitpoulpi ui show keys` | view |
| `ui_show_palette` | `gitpoulpi ui show palette` | view |
| `ui_show_pr` | `gitpoulpi ui show pr` | view |
| `ui_show_ref` | `gitpoulpi ui show ref` | view |
| `ui_show_search` | `gitpoulpi ui show search` | view |
| `ui_show_settings` | `gitpoulpi ui show settings` | view |
| `ui_show_tab` | `gitpoulpi ui show tab` | view |
| `ui_show_wip` | `gitpoulpi ui show wip` | view |
| `ui_state` | `gitpoulpi ui state` | view |
| `ui_viewers` | `gitpoulpi ui viewers` | view |
| `undo` | `gitpoulpi undo` | destructive |
| `undo_forget` | `gitpoulpi undo-forget` | write |
| `undo_restore` | `gitpoulpi undo-restore` | write |
| `unstage` | `gitpoulpi unstage` | write |
| `workspace_add` | `gitpoulpi workspace add` | write |
| `workspace_create` | `gitpoulpi workspace create` | write |
| `workspace_delete` | `gitpoulpi workspace delete` | destructive |
| `workspace_list` | `gitpoulpi workspace list` | read |
| `workspace_remove` | `gitpoulpi workspace remove` | write |
| `workspace_rename` | `gitpoulpi workspace rename` | write |
| `workspace_set_ai_instructions` | `gitpoulpi workspace set-ai-instructions` | write |
| `workspace_set_profile` | `gitpoulpi workspace set-profile` | write |
| `workspace_show` | `gitpoulpi workspace show` | read |
| `worktree_add` | `gitpoulpi worktree add` | write |
| `worktree_changes` | `gitpoulpi worktree changes` | read |
| `worktree_list` | `gitpoulpi worktree list` | read |
| `worktree_remove` | `gitpoulpi worktree remove` | destructive |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi hosting integrations trust` | accepting a changed address, account or certificate is the user's decision; an agent is told the security check needs the user |
| `gitpoulpi ai trust` | binding an API key to another address is the user's decision |
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
| `gitpoulpi mcp token` | prints, copies and rotates the key that guards this very server: an agent must not read or replace it |
| `gitpoulpi completions` | shell completion generation, not a GitBackend operation |
| `gitpoulpi shell-init` | prints a wrapper function for the user's own shell, not a GitBackend operation |
| `gitpoulpi desktop shortcut status` | launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer |
| `gitpoulpi desktop shortcut install` | writes files into the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop shortcut remove` | deletes files from the user's launcher, Start menu and Desktop |
| `gitpoulpi alias create` | writes a command into the user's PATH; an agent never changes the user's shell |
| `gitpoulpi alias remove` | deletes a command from the user's PATH; an agent never changes the user's shell |
| `gitpoulpi path add` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi path remove` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi uninstall` | removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine |
| `gitpoulpi launch` | starts a terminal window on the user's desktop; the launcher shortcuts' own entry point |
| `gitpoulpi daemon run` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon start` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon stop` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon restart` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon status` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi rewrite script trust` | trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted |
| `gitpoulpi config set` | an agent must not be able to rewrite its own gating config |
| `gitpoulpi hosting ssh-key create` | writes a private key and uploads it; too dangerous to let an agent automate |
| `gitpoulpi hosting login` | takes a secret token on stdin; a token must not pass through an agent's tool-call transcript |
| `gitpoulpi repo-auth set` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi repo-auth clear` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi flow trust-hooks` | trusting the shell commands a repository commits is a person's decision; an agent can only run hooks a person trusted |
<!-- mcp-tools:end -->

### Дії робочого процесу гілок: план, потім виконання

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` і `flow_set_parent`
([Робочі процеси гілок](branching-workflows.md#дії)) ніколи не виконуються на голий
виклик. `dry_run: true` повертає план (кожен крок git і команду, попередження,
що Undo не може повернути, перешкоди) як звичайний результат; `yes: true` виконує
його; без жодного з них виклик відхиляється і несе план. `[hooks]` файлу робочого
процесу - це команди оболонки з файлу, яким ділиться команда, тож вони виконуються
лише з явним `run_hooks: true`: план перелічує їх (`hooks`, `hooks_run`), щоб ви
могли спершу показати користувачеві точні команди. Merge, що зупинився на
конфліктах, відповідає `status: "conflicted"` (результат-помилка, код виходу 2);
після того як інструменти конфліктів вирішили файли, викличте той самий
інструмент із `continue_: true` або `abort: true`. Відкриття merge request
(`finish` у режимі `merge-request`) не скасовується, і план про це каже.

### Те, що агент робить звичайним `git`, теж є в історії скасувань

Агент, який створює гілки й worktree, робить commit'и, reset'и чи відкидає
stash'і власним `git` (не через ці інструменти), усе одно видимий: демон
порівнює ref'и репозиторію з тим, що бачив востаннє, і записує кожну зміну як
запис `undo --list` із `"source": "outside"`, причиною від git
(`outside.message`) і committer'ом, якого записав git. Інструменти `undo`, `redo`
та `undo_restore` беруть ці записи, як будь-які інші, тож агент (або
користувач через `Ctrl+Z`) може повернути видалену гілку. Зміни файлів ніколи не
фіксуються. Див. [Скасування й повторення](undo-redo.md#зміни-зроблені-поза-gitpoulpi).

## Показ речей користувачеві

Агент може вказати на щось, замість того щоб це описувати: `ui_show_commit`,
`ui_show_diff` (файл на одному рядку, inline або поруч), `ui_show_file`
(вміст, blame чи історія на одному рядку), `ui_show_ref`, `ui_show_pr`,
`ui_show_issue`, `ui_show_conflict` (редактор merge на файлі з конфліктом, на
одному конфлікті), `ui_show_wip` (`composer` ставить курсор у заголовок commit'у,
нічого не вводячи), `ui_show_tab`, `ui_show_home`, `ui_show_search`,
`ui_show_palette` (введений запит, ніколи не виконується), `ui_show_keys`,
`ui_show_settings` (сторінка й параметр, ніколи не змінюється), `ui_highlight`
(підсвітити рядки відкритого diff чи файлу), `ui_notify` (сповіщення) і `ui_back`
(скасувати останню зміну вигляду агентом). `ui_state` читає, що показує вікно
(`text: true` додає видимий текст); `ui_viewers` перелічує вікна.

Ці інструменти **ніколи не змінюють репозиторій** і не можуть ні вводити текст,
ні клацати, ні змінювати параметр. Ревізії та шляхи перевіряються до того, як щось
дійде до вікна.

**Що бачить і вирішує користувач**:

- `ask` (типово), або щоразу, коли ви вводите текст, відкрито діалог чи Налаштування,
  ви перетягуєте або натиснули клавішу за останні 1,5 секунди: агент одразу отримує
  `deferred`, а ваш рядок стану показує чип *«claude-code хоче показати ...»*.
  `Ctrl+N Enter` або клацання показує; `Esc` чи `Ctrl+N Backspace`
  відхиляє. Він спливає за 60 с.
- Застосовано: вигляд рухається, 4-секундне сповіщення каже, хто його перемістив, а
  вкладка отримує фіолетовий `✦` до вашого наступного натискання. `Ctrl+N Backspace`
  (команда палітри `Агент: назад`) повертає назад; вікно пам'ятає десять останніх видів.
- Нічого ніколи не розкривається й не checkout'иться: commit, який ваш solo-фільтр
  приховує, відповідає `partial` (`hidden_by_filter`); список PR чи issues, не
  прив'язаний до провайдера, відповідає `hosting_not_linked`.

**Відповіді**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, зі `state` вікна, коли застосовано.
Коди завершення CLI: 0 applied/partial/proposed, 3 deferred, інакше 1. Виклики
обмежено за частотою (сплеск із 5, далі один на секунду).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Пропозиція зміни (`ui_propose_*`)

За `allow_ui_propose = true` агент може відкрити один із ваших діалогів,
**заповнений наперед** і позначений *✦ Запропоновано <agent>*. Нічого не
відбувається, доки ви не натиснете підтвердження цього діалогу; перед тим ви
можете змінити або скасувати.

| Інструмент | Відкриває | Ви підтверджуєте через |
|---|---|---|
| `ui_propose_reword` (rev, message) | вікно Reword | *Оновити повідомлення* (надісланий commit усе одно питає про force push) |
| `ui_propose_commit_message` (summary, body) | форму commit'у, заповнену; ніколи не робить commit | кнопку commit'у |
| `ui_propose_branch` (name, start point) | inline-поле «Нова гілка» | `Enter` |
| `ui_propose_split` (rev, plan file) | редактор розділення над commit'ом, заповнений планом | *Розділити* (`Ctrl+S`); `Esc` відхиляє |
| `ui_propose_rewrite` (файл рецепта) | майстерня переписування історії з правилами рецепта; нічого не переглядається й не застосовується | ваші власні *Перегляд*, а потім *Застосувати* (`Ctrl+Enter`); `Esc` відхиляє. Рецепт зі скриптом відхиляється, секретні значення відкидаються |
| `ui_propose_tag` (name, target, message) | inline-поле нового tag (анотований, коли є повідомлення) | `Enter` |
| `ui_propose_resolution` (path, pick або content file) | редактор merge на файлі з конфліктом, з уже вибраними варіантами (або всім текстом) у Результаті | *Зберегти й позначити вирішеним* (`Ctrl+S`); закриття відхиляє |

Одна пропозиція за раз; пропозицію повідомлення commit'у відхиляють, доки у вашому
композері є ваша власна чернетка, а пропозицію вирішення - доки редактор merge
містить ваші незбережені правки або коли стан індексу файлу змінився відтоді, як агент його
прочитав. Вирішення, яке ви змінюєте перед підтвердженням, позначається як
`accepted_edited`. Відповідь містить id `proposal`; що ви вирішили, видно в
`proposals` з `ui_state` (`offered`, `open`, `accepted`, `accepted_edited`,
`rejected`, `expired`, `failed`).

### Коли жодного вікна не відкрито або воно в іншій сесії

- Вікно належить сесії, у якій його запущено. Якщо в сесії агента його немає,
  `ui show` шукає **одне** вікно в іншій сесії, що має відкритий цей репозиторій, і
  використовує його (`routed_session` у відповіді). Якщо збігається кілька:
  `no_viewer` із їхніми назвами; передайте `--session`.
- **Відкриття вікна для агента** типово вимкнено. Задайте `[ui] launch`
  (Налаштування > Агенти й MCP > *Відкривати вікно для агентів*) як шаблон
  терміналу чи мультиплексора або команду `custom`; тоді виклик з `launch: true`
  (CLI `gitpoulpi ui --launch show ...`) відкриває вікно й чекає до 10 с. За
  `ui_control = "auto"` воно відкривається й без цього параметра; за `ask` агент
  отримує `no_viewer` і має сказати вам про це.

| Шаблон | Команда |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (ваш типовий термінал) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (нове вікно), `zellij` (нова панель), `cmux` (новий робочий простір) | у запущеному мультиплексорі, у папці репозиторію |
| `custom` | `[ui] launch_command`, напр. `my-term --cd {repo} {gitpoulpi} --session {session}`; розбивається на аргументи, без shell |

Якщо шаблон не працює з вашою версією програми, використайте `custom`.

## Помилки

- Неправильно сформований виклик (невідомий інструмент чи аргумент, бракує
  обов'язкового аргумента) - це помилка JSON-RPC `-32602 invalid_params`.
- Відмова або помилка git чи демона (хибна ревізія, конфлікт, ...) - це помилка
  інструмента: `isError: true` з повідомленням у `content` і
  `structuredContent.error`.
