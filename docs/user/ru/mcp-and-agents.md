[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · **Русский** · [Svenska](../sv/mcp-and-agents.md) · [Türkçe](../tr/mcp-and-agents.md) · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP и агенты

ИИ-агент (Claude Code, Cursor, Codex, ...) может управлять GitPoulpi через его
**сервер MCP** или через **CLI**. Оба работают через тот же фоновый демон, что и
окно, поэтому действия агента сразу видны в вашем открытом GitPoulpi, а для
каждой команды оба запускают один и тот же код.

Эта страница посвящена *внешним* агентам. Собственные ИИ-кнопки GitPoulpi (сообщения
commit'ов, пояснения, композитор commit'ов) настраиваются отдельно: см.
[Помощь ИИ](ai.md).

## Быстрая настройка

1. Добавьте `gitpoulpi` и `gitpoulpi-mcp` в `PATH` ([Установка](installation.md)).
2. Зарегистрируйте сервер у вашего агента. Он работает через stdio: ничего
   запускать не нужно, порт не нужен.
3. При желании выберите в таблице `[mcp]` файла `config.toml` (ниже), что агенту
   разрешено. По умолчанию он может читать репозиторий (изменения отклоняются,
   пока вы не зададите `allow_mutations = true`), показывать вам что-либо только
   после вашего подтверждения и никогда не заполняет диалоги заранее.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` вместо этого записывает его в `.mcp.json` репозитория, так что
он появляется у всех, кто откроет проект. Тот же файл вручную, с необязательными
дополнениями:

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

- `--ui-control auto|ask|off` переопределяет `[mcp] ui_control` только для этого
  агента.
- `GITPOULPI_CONFIG_DIR` даёт агенту собственный `config.toml` (свои ограничения
  `[mcp]`, без доступа к вашим настройкам). Не указывайте, чтобы использовать
  ваш.
- `gitpoulpi --repo <path> mcp serve --stdio` привязывает сервер к одному
  репозиторию; иначе инструмент работает с репозиторием вокруг рабочей папки
  сервера или с аргументом `repo` вызова.

**Другие клиенты**: любой клиент MCP, запускающий сервер stdio.

| Клиент | Где | Что |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | JSON-файл MCP клиента | блок `mcpServers` выше (в VS Code ключ называется `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Всё остальное | его документация | команда `gitpoulpi-mcp` (отдельный исполняемый файл, тот же сервер, аргументы не нужны) |

**По HTTP** (клиенты, подключающиеся по URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, по умолчанию http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

Без состояния, JSON, те же инструменты, те же ограничения `[mcp]`. Каждому запросу
нужен bearer-токен (`[mcp] http_auth`, по умолчанию включено): выведите его командой
`gitpoulpi mcp token --show` (или `--copy`, `--rotate`; токен хранится в файле,
доступном только пользователю, в закрытой папке данных) и передайте клиенту как
заголовок:

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

Отсутствующий или неверный токен получает `401` с `WWW-Authenticate: Bearer`.
Заменённый токен действует сразу, без перезапуска сервера. Выключать `http_auth`
стоит только для эндпоинта на loopback, где вы доверяете каждой локальной учётной
записи; сервер тогда предупреждает в stderr.

### Доступ к эндпоинту с другого компьютера

Самой безопасной настройке удалённый слушатель вообще не нужен: оставьте адрес
loopback и пробросьте порт по SSH с компьютера, на котором работает агент:
`ssh -L 8808:127.0.0.1:8808 host`. Тогда `--allow-remote` не участвует.

`--allow-remote` (адрес не loopback) отклоняется, пока не включена аутентификация
с готовым токеном **и** трафик не зашифрован. Отказ называет, чего не хватает.
Зашифровать можно двумя способами:

- **TLS в сервере** (`[mcp] tls = "self_signed"` или `"files"`). Сервер сам
  завершает TLS: он держит ключ, проводит рукопожатие, и до уровня HTTP доходит
  только завершённое рукопожатие; клиент с обычным HTTP не проходит рукопожатие, а
  обычного слушателя рядом нет. Так что шифрование - это факт, который сервер
  устанавливает при каждом соединении, а не предположение. При `self_signed`
  первый запуск создаёт пару в закрытой папке данных и выводит её отпечаток
  SHA-256 (он есть и в настройках); клиент доверяет серверу, закрепив этот отпечаток
  или файл сертификата. При `files` укажите в `tls_cert` и `tls_key` ваши файлы PEM,
  и клиенты проверяют сертификат по его центру сертификации. Клиенты проходят
  аутентификацию bearer-токеном; клиентские сертификаты не используются.
- **Туннель или обратный прокси** (`[mcp] behind_proxy = true`). Агент туннеля или
  прокси подключается как любой локальный клиент или клиент локальной сети, а его
  заголовки можно подделать, поэтому сервер изнутри не может определить, есть ли он.
  Потому это явное подтверждение (в настройках оно показано с красным
  предупреждением), а не обнаружение. Запускайте туннель на том же компьютере и по
  возможности оставляйте адрес loopback. При адресе не loopback за прокси участок
  между прокси и сервером передаётся открытым текстом по этой сети.

**Без MCP** агент с доступом к оболочке использует CLI: `gitpoulpi --json <command>`
(см. [справочник по CLI](cli-reference.md)). Дайте ему готовый навык
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md), который
говорит, когда и как использовать GitPoulpi вместо «сырого» `git`.

## Разрешения

| Переключатель | По умолчанию | Действие |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` отказываются запускаться (Настройки > Агенты и MCP > *Сервер MCP*). Читается при запуске сервера: работающий сервер сохраняет свои настройки. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Адрес, который использует `--http`, если адрес не указан (только loopback, если нет `--allow-remote`, которому нужны аутентификация и шифрование, см. выше). Настройки > Агенты и MCP > *Порт сервера MCP* меняет его порт и сохраняет хост. |
| `[mcp] http_auth` | `true` | `--http` требует `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Настройки > Агенты и MCP > *Аутентификация HTTP*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` или `files` (ваши пути PEM): сервер сам завершает TLS. Настройки > Агенты и MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Вы ручаетесь, что туннель или обратный прокси шифрует трафик, что позволяет `--allow-remote` работать без TLS. |
| `[mcp] allow_mutations` | `false` | Выкл.: отклоняется каждый инструмент, что-либо меняющий; чтение, `ui_*` и предложения работают. `true` позволяет агенту делать commit, push, reset... |
| `[mcp] mutation_allowlist` | `[]` | Не пусто: менять что-либо могут только эти инструменты, например `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: агент не может управлять вашим окном (`ui_state` / `ui_viewers` по-прежнему отвечают). |
| `[mcp] ui_control` | `"ask"` | `ask`: в вашем окне появляется плашка, которую вы подтверждаете; `auto`: применяется, если вы не заняты; `off`: отклоняется. Для отдельного агента: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Позволяет агенту открывать заполненные заранее диалоги, которые вы подтверждаете (**Настройки > Агенты и MCP > Разрешить ИИ-агентам предлагать изменения для моего подтверждения**). |
| `[mcp] allow_sensitive_staging` | `false` | Только в `config.toml`. Выключено: инструмент `stage` не предлагает `allow_sensitive`, а добавление, которое внесло бы конфиденциальный файл (закрытый ключ, `.env`, распознанный токен), отклоняется с `{error: "sensitive_files", files}`: агент должен спросить вас. `true` позволяет ему передавать `allow_sensitive`. См. [Индекс и commit'ы](staging-and-committing.md#конфиденциальные-файлы). |
| `[mcp] allow_history_purge` | `false` | Только в `config.toml`. `true` показывает инструмент `rewrite_purge`, который навсегда удаляет старую историю переписывания (и её отмену). Выключено: агент его не видит. Ему по-прежнему нужен `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | Главный выключатель вашего окна (**Настройки > Агенты и MCP > Разрешить ИИ-агентам управлять видом**). Выкл.: ни один агент не достаёт до вашего окна. |
| `[ui] launch` | `"off"` | Может ли агент открыть окно GitPoulpi, когда ни одного не открыто (см. [ниже](#когда-нет-открытого-окна-или-оно-в-другой-сессии)). |

Что это значит на практике:

- **По умолчанию агент работает только на чтение**: он может читать, показывать
  вам что-то в вашем окне (`ui_*`) и, если вы разрешите, предлагать изменения,
  которые вы подтверждаете. Чтобы разрешить ему менять репозиторий (commit, push,
  reset...), задайте `allow_mutations = true`; для узкого разрешения оставьте
  `true` вместе с `mutation_allowlist`. Каждый инструмент несёт аннотации MCP
  (`readOnlyHint`, `destructiveHint`, `openWorldHint`), поэтому такой клиент, как
  Claude Code, всё равно спросит вас перед рискованными.
- **Конфликты**: `conflicts_check` (прогноз до любого merge), `conflicts_list`, `conflicts_show` и `conflicts_state` - чтение;
  `conflicts_resolve` (отбрасывает не выбранную сторону, `--delete` удаляет файл)
  и `conflicts_abort` (отбрасывает весь merge/rebase/cherry-pick) разрушительны;
  `conflicts_unresolve` - обычная запись. Отклонённый вызов (`stale`,
  `not_conflicted`, `pick_incomplete`...) отвечает `{"refused": {...}}` с кодом
  выхода 2, а не завершается ошибкой.
- **Зависшие состояния**: `conflicts_am_continue` - запись, `conflicts_am_skip`,
  `conflicts_am_abort` и `index_lock_remove` разрушительны, `index_lock_show` -
  чтение.
- **Переписывание истории**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (анализ размеров), `rewrite_scan_secrets` (находки со скрытым фрагментом, но не значением),
  `rewrite_script_check`, `rewrite_script_test` и `rewrite_discard` - чтение
  (предпросмотр ничего не меняет: он лишь держит план в демоне и на диске, в
  git-каталоге, поэтому `rewrite_apply` работает и после перезапуска).
  `rewrite_apply` и `rewrite_run` разрушительны: они сдвигают много ссылок на
  переписанные commit'ы одной записью отмены (`undo` возвращает их) и ничего не
  отправляют. Рецепт передавайте как `recipe_json` (или `recipe` - путь), секреты как
  `secrets_file`, но не прямо в вызове. Скрипты shell, Python и внешнего Lua
  (`script_language`) не в песочнице, и **агент не может им доверять**: инструмента
  `rewrite_script_trust` и аргумента `trust` нет, поэтому такой скрипт запускается,
  только если человек выполнил `gitpoulpi rewrite script trust FILE` для именно этого
  текста (изменённый скрипт снова отклоняется: `script_not_trusted` с хэшем в
  `refused.script`) и только если `rewrite.external_scripts` включает его язык
  (`scripts_disabled`). Отказ отвечает
  `{"refused": {"reason", "message", "detail"}}`, а план, который сдвигает
  отправленные ссылки, отвечает `{"needs_confirmation": ...}` (код выхода 3), пока
  в вызове нет `yes` - сначала покажите его пользователю.
- **Агенту недоступно никогда**: сохранение токена (`hosting login`), создание и
  загрузка ключа SSH, `config set` (он мог бы переписать собственные ограничения)
  и управление жизненным циклом демона. Токен агенту никогда не показывается.
- **Списки по всему аккаунту**: `hosting_radar` - чтение (запросы, ждущие вашего
  ревью, открытые запросы, назначенные задачи по всем репозиториям профиля,
  из кэша на пять минут, если нет `refresh`); `hosting_integrations_radar`
  меняет таблицу `[integrations.radar]` профиля, поэтому его ограничивает `allow_mutations`
  (даже при вызове без параметров, который только выводит настройки).
- **Контекст ревью**: `ai_review_context` показывает, что черновик код-ревью получает
  помимо diff (CLAUDE.md, AGENTS.md, дополнительный файл, где запускается локальный агент). С
  аргументом настройки он сохраняет значение по умолчанию (глобальное, либо собственное для `repo` или `project`),
  поэтому его ограничивает `allow_mutations`, как и другие инструменты, меняющие настройку; без аргументов он
  только выводит. Как и все инструменты `ai_*`, он виден, только пока включён `ai.enabled`.
  `hosting_review_suggest` принимает те же аргументы, но только для одного запуска.
- **Ограничения `[mcp]` действуют на сервер MCP.** CLI - это ваша собственная
  оболочка, и `allow_mutations` на неё не действует; только `ui_control` и
  `allow_ui_propose` действуют также на `gitpoulpi ui ...`.
- Отклонённый вызов - обычная ошибка инструмента (`isError: true`, причина в
  `structuredContent.error`), а не молчаливое отсутствие действия.

```toml
[mcp]
allow_mutations = false        # true: агент может менять репозиторий
mutation_allowlist = []        # например ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" позволяет агенту менять ваш вид без вопросов
allow_ui_propose = false       # true: агент может заранее заполнять диалоги Reword / Split / commit / ветки / tag'а
allow_sensitive_staging = false  # true: агент может добавлять в индекс файлы, помеченные защитой индекса
allow_history_purge = false    # true: агент видит rewrite_purge (навсегда удаляет старую историю)
```

## Инструменты

Один инструмент на команду CLI, названный по её пути через `_` (`branch create` - это `branch_create`). Каждый принимает аргументы команды плюс `repo`
(репозиторий; по умолчанию репозиторий сервера) и `session`. `structuredContent`
успешного вызова - ровно то, что печатает `gitpoulpi --json <command>`. **Kind**
(вид): *read* никогда ничего не меняет; *write* и *destructive* - изменения (с
ограничениями); *network* обращается к remote или хосту; *view* только двигает
ваше окно; *propose* открывает диалог, который вы подтверждаете.

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
| `worktree_list` | `gitpoulpi worktree list` | read |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
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
<!-- mcp-tools:end -->

### Действия рабочих процессов ветвления: сначала план, потом выполнение

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` и `flow_set_parent`
([Рабочие процессы ветвления](branching-workflows.md#действия)) никогда не выполняются при простом вызове.
`dry_run: true` возвращает план (каждый шаг и команду git, предупреждения, что Отмена не вернёт, препятствия) как
обычный результат; `yes: true` выполняет его; без того и другого вызов отклоняется и несёт план. `[hooks]` файла
рабочего процесса - это команды оболочки из файла, общего для команды, поэтому они запускаются только с явным
`run_hooks: true`: план перечисляет их (`hooks`, `hooks_run`), чтобы вы могли показать пользователю точные команды
заранее. Слияние, остановившееся на конфликтах, отвечает `status: "conflicted"` (результат-ошибка, код выхода 2);
когда инструменты для конфликтов разрешили файлы, вызовите тот же инструмент с `continue_: true` или `abort: true`.
Открытие запроса на слияние (`finish` в режиме `merge-request`) отменить нельзя, и план об этом говорит.

### То, что агент делает обычным `git`, тоже попадает в историю отмен

Агент, который создаёт ветки и worktree, делает commit'ы, reset'ы или сбрасывает
stash'и собственным `git` (не через эти инструменты), всё равно виден: демон
сравнивает ссылки репозитория с тем, что видел в последний раз, и записывает
каждое изменение как запись `undo --list` с `"source": "outside"`, причиной от
git (`outside.message`) и автором, которого записал git. Инструменты `undo`,
`redo` и `undo_restore` принимают эти записи как любые другие, поэтому агент
(или пользователь с `Ctrl+Z`) может вернуть удалённую ветку. Правки файлов не
фиксируются. См. [Отмена и повтор](undo-redo.md#изменения-сделанные-вне-gitpoulpi).

## Показ пользователю

Агент может показать на что-то, а не описывать: `ui_show_commit`, `ui_show_diff`
(файл на строке, inline или split), `ui_show_file` (содержимое, blame или
история на строке), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`,
`ui_show_conflict` (редактор слияния на файле с конфликтом, на конфликте),
`ui_show_wip` (`composer` ставит курсор в заголовок commit'а, ничего не вводя),
`ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette` (запрос
вводится, но никогда не выполняется), `ui_show_keys`, `ui_show_settings`
(страница и параметр, никогда не изменяются), `ui_highlight` (подсвечивает строки
открытого diff или файла), `ui_notify` (уведомление) и `ui_back` (отменяет
последнее изменение вида агентом). `ui_state` читает, что показывает окно
(`text: true` добавляет видимый текст); `ui_viewers` перечисляет окна.

Эти инструменты **никогда не меняют репозиторий** и не могут вводить текст,
щёлкать или изменять настройку. Ревизии и пути проверяются до того, как что-либо
дойдёт до окна.

**Что видит и решает пользователь**:

- `ask` (по умолчанию), а также когда вы печатаете, открыт диалог или Настройки,
  вы перетаскиваете или нажали клавишу за последние 1,5 с: агент сразу получает
  `deferred`, а в вашей строке состояния появляется плашка *«claude-code хочет
  показать: ...»*. `Ctrl+N Enter` или щелчок показывает; `Esc` или
  `Ctrl+N Backspace` отклоняет. Через 60 с она истекает.
- Применено: вид сдвигается, уведомление на 4 с говорит, кто его сдвинул, а
  вкладка получает фиолетовый `✦` до вашего следующего нажатия. `Ctrl+N
  Backspace` (палитра `Агент: назад`) возвращает назад; окно помнит десять видов.
- Ничего никогда не раскрывается и не переключается: commit, скрытый вашим
  фильтром solo, отвечает `partial` (`hidden_by_filter`); список PR или задач,
  не связанный с провайдером, отвечает `hosting_not_linked`.

**Ответы**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, с `state` окна при применении.
Коды выхода CLI: 0 applied/partial/proposed, 3 deferred, 1 в остальных случаях.
Вызовы ограничены по частоте (всплеск из 5, затем один в секунду).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Предложение изменения (`ui_propose_*`)

При `allow_ui_propose = true` агент может открыть один из ваших диалогов
**заполненным заранее** с пометкой *✦ Предложено: <агент>*. Ничего не
происходит, пока вы не нажмёте подтверждение самого диалога; сначала можно
отредактировать или отменить.

| Инструмент | Открывает | Вы подтверждаете |
|---|---|---|
| `ui_propose_reword` (rev, message) | окно изменения сообщения | *Обновить сообщение* (для уже отправленного commit'а по-прежнему спрашивает про force push) |
| `ui_propose_commit_message` (summary, body) | форму commit'а, заполненную; сам commit не делает | кнопка Commit |
| `ui_propose_branch` (name, start point) | встроенное поле «Новая ветка» | `Enter` |
| `ui_propose_tag` (name, target, message) | встроенное поле нового tag'а (аннотированного, если есть сообщение) | `Enter` |
| `ui_propose_split` (rev, plan file) | редактор разделения над commit'ом, заполненный из плана | *Разделить* (`Ctrl+S`); `Esc` отклоняет |
| `ui_propose_rewrite` (recipe file) | окно переписывания истории с правилами рецепта в списке; предпросмотр не запускается, ничего не применяется | ваши собственные *Предпросмотр* и затем *Применить* (`Ctrl+Enter`); `Esc` отклоняет. Рецепт со скриптом отклоняется, секретные значения убираются |
| `ui_propose_resolution` (path, pick или файл с содержимым) | редактор слияния на файле с конфликтом, где выборы (или весь текст) уже в результате | *Сохранить и разрешить* (`Ctrl+S`); закрытие отклоняет |

Одно предложение за раз; предложение сообщения commit'а отклоняется, пока в
редакторе лежит ваш черновик, а предложение решения - пока редактор слияния
хранит ваши несохранённые правки или если стадии файла изменились с тех пор, как
агент их прочитал. Решение, которое вы изменили до подтверждения, помечается как
`accepted_edited`. Ответ содержит id `proposal`; ваше решение отражено в
`proposals` из `ui_state` (`offered`, `open`, `accepted`, `accepted_edited`,
`rejected`, `expired`, `failed`).

### Когда нет открытого окна или оно в другой сессии

- Окно принадлежит сессии, в которой оно запущено. Если у сессии агента окна нет,
  `ui show` ищет **одно** окно в другой сессии, где открыт этот репозиторий, и
  использует его (`routed_session` в ответе). Подходящих несколько: `no_viewer` с
  их перечислением; передайте `--session`.
- **Открытие окна для агента** по умолчанию выключено. Задайте `[ui] launch`
  (**Настройки > Агенты и MCP > Открывать окно для агентов**) как шаблон терминала
  или мультиплексора либо команду `custom`; тогда вызов с `launch: true` (CLI
  `gitpoulpi ui --launch show ...`) открывает окно и ждёт до 10 с. При
  `ui_control = "auto"` окно открывается и без флага; при `ask` агент получает
  `no_viewer` и должен сообщить вам.

| Шаблон | Команда |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (ваш терминал по умолчанию) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (новое окно), `zellij` (новая панель), `cmux` (новое рабочее пространство) | внутри запущенного мультиплексора, в папке репозитория |
| `custom` | `[ui] launch_command`, например `my-term --cd {repo} {gitpoulpi} --session {session}`; делится на аргументы, без оболочки |

Если шаблон не работает с вашей версией программы, используйте `custom`.

## Ошибки

- Некорректный вызов (неизвестный инструмент или аргумент, отсутствующий
  обязательный) - ошибка JSON-RPC `-32602 invalid_params`.
- Отказ, а также ошибка git или демона (неверная ревизия, конфликт, ...) -
  ошибка инструмента: `isError: true` с сообщением в `content` и
  `structuredContent.error`.
