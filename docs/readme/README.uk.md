[English](../../README.md) · [Čeština](README.cs.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Türkçe](README.tr.md) · **Українська** · [简体中文](README.zh-CN.md)

# GitPoulpi

<p align="center"><img src="../../assets/logo/gitpoulpi-512.png" alt="Восьминіг GitPoulpi" width="128"></p>

**Клієнт Git для термінала, що відчувається як програма для робочого столу.**
GitPoulpi дає повноцінний граф комітів, по якому можна клікати, який можна
перетягувати й викликати правим кліком, палітру команд із керуванням з
клавіатури, що сягає будь-якої функції, інтеграцію з хостингами коду (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit; хмарними та самостійно розгорнутими) і трекерами задач (Jira, Trello, Linear, YouTrack, Redmine або текою нотаток Markdown), а ті
самі операції доступні як CLI для скриптів і сервер MCP, тож ШІ-агент керує ним
достеменно так, як ви. GitPoulpi натхненний GitKraken і
повністю переписаний з нуля на Rust: нативні виконувані файли, легкі на диску й у пам'яті, без Electron, без облікового запису, без телеметрії. Він працює у вашому улюбленому терміналі ([в якому?](../../docs/user/uk/terminals.md)), бо дехто з нас живе саме там.

![GitPoulpi показує граф комітів із гілками, тегами та stash, бічну панель Refs ліворуч, а праворуч робочі зміни з формою коміту](../../docs/user/images/hero.png)

- **Миша й клавіатура нарівні.** Клікайте, двічі клікайте, викликайте правий
  клік і перетягуйте refs на коміти; або натисніть `Ctrl+P` і введіть, що
  потрібно. Кожен жест миші має двійника на клавіатурі, а кожна команда є в
  палітрі.
- **Справжній граф комітів.** Гілки, злиття, теги, stash і робоча копія в одному
  Журналі, що прокручується, намальованому пікселями в терміналах, які це
  вміють, і символами в усіх інших. Безліч темних і світлих колірних тем,
  аватари авторів, пошук.
- **Точна індексація.** Додавайте в індекс, прибирайте з нього чи відкидайте
  файл, теку, hunk або окремий рядок просто з diff. У стовпчик або поруч, із
  мінікартою.
- **Безпечне редагування історії.** Переформульовуйте або змінюйте будь-який коміт
  (повідомлення, автора, дати), робіть інтерактивний rebase з екраном перегляду,
  переписуйте багато комітів одразу з попереднім переглядом і одним скасуванням,
  робіть cherry-pick кількох комітів, fast-forward, створюйте worktree і патчі.
- **Хостинги та трекери задач вбудовано.** Переглядайте, отримуйте й створюйте pull і merge request (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket) і переглядайте зміни Gerrit. Переглядайте й створюйте задачі, робочі елементи та картки (також Jira, Trello, Linear, YouTrack, Redmine і локальні нотатки Markdown), за бажання на дошці Kanban. Клонуйте зі свого облікового запису й керуйте ключами SSH.
- **CLI і MCP для агентів.** `gitpoulpi <команда> --json` охоплює все, що робить
  інтерфейс, а `gitpoulpi-mcp` віддає це як інструменти MCP. Один фоновий демон
  на репозиторій підтримує всі інтерфейси живими й синхронними.
- **ШІ за бажанням, ніколи не обов'язковий.** Вимкнений за замовчуванням,
  обліковий запис не потрібен: використайте власний ключ API або локальний CLI
  агента, як-от `claude -p`. Створюйте повідомлення комітів, пояснюйте зміни,
  пишіть описи pull request, розбивайте робочі зміни на кілька комітів і готуйте
  чернетки правил переписування історії.
- **Git LFS, підмодулі, worktree, патчі, blame та історія файлу, diff
  зображень, вбудований редактор.**

## Знімки екрана

| | |
|---|---|
| ![Граф Журналу з палітрою доріжок Material і ввімкненим підсвічуванням рядків](../../docs/user/images/theme-material.png) **Граф комітів**, тут з палітрою доріжок Material і підсвічуванням рядків. | ![Diff зміненого файлу з кнопками Stage hunk і Відкинути hunk біля кожного hunk](../../docs/user/images/diff.png) **Diff**, з індексацією та відкиданням по hunk або по рядку. |
| ![Палітра команд, відфільтрована за словом stash](../../docs/user/images/palette.png) **Палітра команд** (`Ctrl+P`) сягає будь-якої команди. | ![Форма комітів пропонує чотири коміти для робочих змін, готові до перегляду](../../docs/user/images/composer.png) **Форма комітів** дає переглянути коміти, запропоновані ШІ, перш ніж щось буде закомічено. |
| ![Контекстне меню коміту з діями cherry-pick, revert, reset, rebase, гілка й тег](../../docs/user/images/context-menu.png) **Правий клік** по чому завгодно відкриває його дії. | ![Сторінка ШІ в налаштуваннях з профілями, інструкціями для кожної дії та перемикачем увімкнення](../../docs/user/images/settings-ai.png) **Налаштування**, з пошуком, зокрема з необов'язковою сторінкою ШІ. |

Більше малюнків із поясненнями є в [документації користувача](../../docs/user/uk/README.md).

## Встановлення

Потрібен системний `git`; `git-lfs` необов'язковий.

### Linux і macOS (`install.sh`)

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

### Інсталятор для Windows

Завантажте `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` з
[останнього випуску](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest) і
запустіть його. Для адміністраторів є MSI:

```
msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn
```

### Docker

```
docker pull poulpilab/gitpoulpi
```

Див. [Запуск у Docker](../user/uk/docker.md).

### cargo

```
cargo binstall gitpoulpi-cli          # prebuilt
cargo install --locked gitpoulpi-cli  # compiles
```

### Архів випуску

Завантажте його з
[останнього випуску](https://github.com/Poulpi-Lab/GitPoulpi/releases/latest),
розпакуйте та дійте за `INSTALL.txt` усередині.

### З вихідного коду

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Власний архів випуску, менша спеціальна збірка та нотатки щодо ОС:
[docs/user/uk/installation.md](../user/uk/installation.md) і
[docs/user/uk/custom-builds.md](../user/uk/custom-builds.md).

## Швидкий старт

```
cd path/to/your/repository
gitpoulpi
```

- `?` відкриває довідку з усіма клавішами та жестами, `Ctrl+P` палітру команд,
  `Ctrl+,` налаштування, `q` виходить.
- Клікніть коміт, щоб побачити його зміни; клікніть верхній рядок Журналу
  (робочі зміни), щоб додавати файли в індекс і робити коміт.
- У терміналі, що це підтримує (kitty, Ghostty, WezTerm, iTerm2 (а також Windows Terminal і foot через sixel)), граф
  малюється пікселями. В інших (звичайна консоль Windows, tmux) він
  автоматично перемикається на символи.

Покрокова версія є в розділі [Початок роботи](../../docs/user/uk/getting-started.md),
а [огляд інтерфейсу](../../docs/user/uk/interface-tour.md) називає кожну ділянку
екрана.

```
gitpoulpi --json status            # the CLI, scriptable
gitpoulpi --json log --limit 20
claude mcp add gitpoulpi -- gitpoulpi-mcp   # give an AI agent the same tools
```

ШІ необов'язковий і за замовчуванням вимкнений: увімкніть власний ШІ GitPoulpi
у *Налаштування > ШІ* ([Допомога ШІ](../../docs/user/uk/ai.md)); дозвольте зовнішньому
агентові керувати GitPoulpi через [MCP та агентів](../../docs/user/uk/mcp-and-agents.md)
(дозволи, усі інструменти) і навичку агента
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md).

## Документація

Повна документація користувача починається з **[docs/user/uk/README.md](../../docs/user/uk/README.md)**:

| | |
|---|---|
| [Початок роботи](../../docs/user/uk/getting-started.md) | Встановлення, відкриття репозиторію, перший коміт |
| [Огляд інтерфейсу](../../docs/user/uk/interface-tour.md) | Усі ділянки вікна з назвами |
| [Робота з Журналом](../../docs/user/uk/log-and-history.md) | Граф, refs, гілки, теги, stash, rebase, cherry-pick |
| [Індексація та коміти](../../docs/user/uk/staging-and-committing.md) | Стан, diff, hunk і рядки, форма коміту, редактор |
| [Налаштування](../../docs/user/uk/settings.md) | Кожна сторінка й параметр налаштувань |
| [Допомога ШІ](../../docs/user/uk/ai.md) | Налаштування вбудованого ШІ, усі дії ШІ, приватність |
| [Хостинги та трекери задач](../../docs/user/uk/hosting-integration.md) | GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine: токени, прив'язка репозиторію, pull і merge request |
| [CLI](../../docs/user/uk/cli-reference.md) і [MCP](../../docs/user/uk/mcp-and-agents.md) | Автоматизуйте або дайте керувати агентові (дозволи, усі інструменти) |
| [Скорочення клавіш](../../docs/user/uk/keybindings.md) (і всі команди палітри), [миша](../../docs/user/uk/mouse-and-gestures.md), [конфігурація](../../docs/user/uk/config-reference.md), [LFS](../../docs/user/uk/lfs.md), [скасування](../../docs/user/uk/undo-redo.md), [усунення несправностей](../../docs/user/uk/troubleshooting.md) | Довідка та допомога |

Розробникам: [CLAUDE.md](../../CLAUDE.md) і `docs/architecture/overview.md`
описують крейти та устрій; [CHANGELOG.md](../../CHANGELOG.md) перелічує, що ввійшло в
кожен реліз.

## Ліцензія

[MIT](../../LICENSE).

## Подяки та ліцензії сторонніх компонентів

GitPoulpi побудований на бібліотеках з відкритим кодом і використовує значення кольорів з опублікованих тем. Їхні ліцензії та подяки, включно з авторами тем, містяться у [THIRD-PARTY-LICENSES.txt](../../THIRD-PARTY-LICENSES.txt). Файл постачається з кожним випуском; його також показують вікно «Про GitPoulpi» і команда `gitpoulpi licenses`.

## Торговельні марки

GitKraken є торговельною маркою свого власника. GitPoulpi - незалежний проєкт, не пов'язаний із цим власником, не схвалений і не спонсорований ним; назву згадано лише для опису сумісності та джерела натхнення.
