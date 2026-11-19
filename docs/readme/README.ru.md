[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · **Русский** · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="Осьминог GitPoulpi" width="128"></p>

**Git-клиент для терминала, который ощущается как настольное приложение.**
GitPoulpi даёт полноценный граф коммитов, по которому можно щёлкать, который
можно перетаскивать и вызывать правым кликом, палитру команд с управлением с
клавиатуры, достающую до любой функции, интеграцию с хостингами кода (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; облачными и самостоятельно развёрнутыми) и трекерами задач (Jira, Trello, Linear, YouTrack, Redmine или папкой заметок Markdown),
а те же операции доступны как скриптуемый CLI и MCP-сервер, так что ИИ-агент
управляет им точно так же, как вы. GitPoulpi вдохновлён GitKraken и
полностью переписан с нуля на Rust: нативные бинарные файлы, лёгкие на диске и в памяти, без Electron, без аккаунта, без телеметрии. Он работает в вашем любимом терминале ([в каком?](../../docs/user/ru/terminals.md)), потому что некоторые из нас живут именно там.

![GitPoulpi показывает граф коммитов с ветками, тегами и stash, боковую панель Refs слева, а справа рабочие изменения с формой коммита](../../docs/user/images/hero.png)

- **Мышь и клавиатура на равных.** Щёлкайте, дважды щёлкайте, вызывайте правый
  клик и перетаскивайте refs на коммиты; или нажмите `Ctrl+P` и введите, что
  нужно. У каждого жеста мыши есть двойник на клавиатуре, а каждая команда есть
  в палитре.
- **Настоящий граф коммитов.** Ветки, слияния, теги, stash и рабочая копия в
  одном прокручиваемом Log, нарисованном пикселями в терминалах, которые это
  умеют, и символами во всех остальных. Множество тёмных и светлых цветовых
  тем, аватары авторов, поиск.
- **Точное добавление в индекс.** Добавляйте в индекс, убирайте из него или
  отбрасывайте файл, папку, hunk или отдельную строку прямо из diff. В одну
  колонку или бок о бок, с миникартой.
- **Безопасное редактирование истории.** Переформулируйте или исправляйте любой
  коммит (сообщение, автор, даты), делайте интерактивный rebase с экраном
  проверки, переписывайте сразу много коммитов с предпросмотром и одной отменой,
  делайте cherry-pick сразу нескольких коммитов, fast-forward, создавайте
  worktree и патчи.
- **Хостинги и трекеры задач встроены.** Просматривайте, получайте и создавайте pull и merge request (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) и проверяйте изменения Gerrit. Просматривайте и создавайте задачи, рабочие элементы и карточки (также Jira, Trello, Linear, YouTrack, Redmine и локальные заметки Markdown), при желании на доске Kanban. Клонируйте из своего аккаунта и управляйте SSH-ключами.
- **CLI и MCP для агентов.** `gitpoulpi <команда> --json` покрывает всё, что
  делает интерфейс, а `gitpoulpi-mcp` отдаёт это как инструменты MCP. Один
  фоновый демон на репозиторий держит все интерфейсы живыми и синхронными.
- **ИИ по желанию, никогда не обязательный.** Выключен по умолчанию, аккаунт не
  нужен: используйте свой ключ API или локальный CLI агента, например
  `claude -p`. Создавайте сообщения коммитов, объясняйте изменения, пишите
  описания pull request, разбивайте рабочие изменения на несколько коммитов и
  набрасывайте правила переписывания истории.
- **Git LFS, подмодули, worktree, патчи, blame и история файла, diff
  изображений, встроенный редактор.**

## Скриншоты

| | |
|---|---|
| ![Граф Log с цветовой темой Material и включённой подсветкой строк](../../docs/user/images/theme-material.png) **Граф коммитов**, здесь в теме Material с подсветкой строк. | ![Diff изменённого файла с кнопками Stage hunk и Отбросить hunk у каждого hunk](../../docs/user/images/diff.png) **Diff**, с добавлением в индекс и отбрасыванием по hunk или по строке. |
| ![Палитра команд, отфильтрованная по слову stash](../../docs/user/images/palette.png) **Палитра команд** (`Ctrl+P`) достаёт до любой команды. | ![Композер коммитов предлагает четыре коммита для рабочих изменений, готовые к проверке](../../docs/user/images/composer.png) **Композер коммитов** даёт проверить коммиты, предложенные ИИ, прежде чем что-либо будет закоммичено. |
| ![Контекстное меню коммита с действиями cherry-pick, revert, reset, rebase, ветка и тег](../../docs/user/images/context-menu.png) **Правый клик** по чему угодно открывает его действия. | ![Страница ИИ в настройках с профилями, инструкциями для каждого действия и переключателем включения](../../docs/user/images/settings-ai.png) **Настройки**, с поиском, включая необязательную страницу ИИ. |

Больше картинок с пояснениями есть в [документации пользователя](../../docs/user/ru/README.md).

## Установка

Нужен системный `git`; `git-lfs` необязателен.

### Linux и macOS (`install.sh`)

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

### Homebrew (macOS, Linux)

```
brew install Poulpi-Lab/gitpoulpi/gitpoulpi
```

### apt (Debian, Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

### Scoop (Windows)

```
scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi
scoop install gitpoulpi
```

### Установщик для Windows

Скачайте `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` из
[последнего выпуска](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest) и
запустите его. Для администраторов есть MSI:

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

См. [Запуск в Docker](../user/ru/docker.md).

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### Архив выпуска

Скачайте его из
[последнего выпуска](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest),
распакуйте и следуйте `INSTALL.txt` внутри.

### Из исходников

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Собственный архив выпуска, сборка поменьше и заметки по ОС:
[docs/user/ru/installation.md](../user/ru/installation.md) и
[docs/user/ru/custom-builds.md](../user/ru/custom-builds.md).

## Быстрый старт

```
cd path/to/your/repository
gitpoulpi
```

- `?` открывает справку со всеми клавишами и жестами, `Ctrl+P` палитру команд,
  `Ctrl+,` настройки, `q` выходит.
- Щёлкните коммит, чтобы увидеть его изменения; щёлкните верхнюю строку Log
  (рабочие изменения), чтобы добавлять файлы в индекс и делать коммит.
- В терминале, который это поддерживает (kitty, Ghostty, WezTerm, iTerm2 (а также Windows Terminal и foot через sixel)), граф
  рисуется пикселями. В остальных (обычная консоль Windows, tmux) он
  автоматически переключается на символы.

Пошаговая версия есть в разделе [Начало работы](../../docs/user/ru/getting-started.md),
а [обзор интерфейса](../../docs/user/ru/interface-tour.md) называет каждую область
экрана.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

ИИ необязателен и по умолчанию выключен: включите собственный ИИ GitPoulpi в
*Настройки > ИИ* ([Помощь ИИ](../../docs/user/ru/ai.md)); позвольте внешнему агенту
управлять GitPoulpi через [MCP и агентов](../../docs/user/ru/mcp-and-agents.md)
(разрешения, все инструменты) и навык агента
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md).

## Документация

Полная документация пользователя начинается с **[docs/user/ru/README.md](../../docs/user/ru/README.md)**:

| | |
|---|---|
| [Начало работы](../../docs/user/ru/getting-started.md) | Установка, открытие репозитория, первый коммит |
| [Обзор интерфейса](../../docs/user/ru/interface-tour.md) | Все области окна с названиями |
| [Работа с Log](../../docs/user/ru/log-and-history.md) | Граф, refs, ветки, теги, stash, rebase, cherry-pick |
| [Индекс и коммиты](../../docs/user/ru/staging-and-committing.md) | Статус, diff, hunk и строки, композер, редактор |
| [Настройки](../../docs/user/ru/settings.md) | Каждая страница и параметр настроек |
| [Помощь ИИ](../../docs/user/ru/ai.md) | Настройка встроенного ИИ, все действия ИИ, конфиденциальность |
| [Хостинги и трекеры задач](../../docs/user/ru/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: токены, привязка репозитория, pull и merge request |
| [CLI](../../docs/user/ru/cli-reference.md) и [MCP](../../docs/user/ru/mcp-and-agents.md) | Автоматизируйте или дайте управлять агенту (разрешения, все инструменты) |
| [Сочетания клавиш](../../docs/user/ru/keybindings.md) (и все команды палитры), [мышь](../../docs/user/ru/mouse-and-gestures.md), [конфигурация](../../docs/user/ru/config-reference.md), [LFS](../../docs/user/ru/lfs.md), [отмена](../../docs/user/ru/undo-redo.md), [устранение неполадок](../../docs/user/ru/troubleshooting.md) | Справка и помощь |

Разработчикам: [CLAUDE.md](../../CLAUDE.md) и `docs/architecture/overview.md`
описывают крейты и устройство; [CHANGELOG.md](../../CHANGELOG.md) перечисляет, что
вошло в каждый релиз.

## Лицензия

[MIT](../../LICENSE).

## Благодарности и лицензии сторонних компонентов

GitPoulpi построен на библиотеках с открытым исходным кодом и использует значения цветов из опубликованных тем. Их лицензии и благодарности, включая авторов тем, находятся в [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt). Файл поставляется с каждым выпуском; его также показывают окно «О программе» и команда `gitpoulpi licenses`.

## Товарные знаки

GitKraken является товарным знаком его владельца. GitPoulpi - независимый проект, не связанный с этим владельцем, не одобренный и не спонсируемый им; название упоминается только для описания совместимости и источника вдохновения.
