[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · **Русский** · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · [Українська](../uk/installation.md) · [简体中文](../zh-CN/installation.md)

# Установка

У GitPoulpi три исполняемых файла:

| Исполняемый файл | Что это |
|---|---|
| `gitpoulpi-tui` | Терминальный интерфейс. |
| `gitpoulpi` | Интерфейс командной строки (`--json` для скриптов). Без подкоманды открывает интерфейс. |
| `gitpoulpi-mcp` | MCP-сервер для ИИ-агентов. См. [MCP и агенты](mcp-and-agents.md). |

Рядом с ними в архиве лежит `gitpoulpi-launch`, крошечная программа, которую запускают ярлыки, чтобы показать уже открытое окно GitPoulpi (`gitpoulpi launch`). В Windows у неё нет окна консоли, поэтому щелчок никогда не вызывает его мигания. Она устанавливается, обновляется и удаляется вместе с остальными.

## Что вам нужно

- **Системный `git`** в вашем `PATH`. GitPoulpi использует libgit2 для большинства
  операций, а ваш `git` для остальных.
  Нужна версия **2.34 или новее**. В macOS подходит `git` из Apple Command Line Tools, а также из Homebrew.
- **Терминал** с поддержкой мыши. kitty, Ghostty, WezTerm и iTerm2 (а также Windows Terminal и foot через sixel) рисуют граф
  commit'ов как изображения. Другие терминалы (консоль Windows, tmux и
  большинство остальных) показывают символьный граф. См.
  [Выбор терминала](terminals.md).
- Шрифт **Nerd Font** или шрифт с эмодзи для значков на панели инструментов и в
  боковой панели.
- По желанию: **`git-lfs`** 3.0 или новее для репозиториев Git LFS. Без него
  файлы LFS показываются как небольшие файлы-указатели, и GitPoulpi объясняет
  почему. Как его установить, см. в [Git LFS](lfs.md).
- По желанию: **credential helper** (Git Credential Manager, libsecret, `pass`,
  Keychain) для хранения токенов хостингов и ключей API ИИ. См.
  [Хостинги и трекеры задач](hosting-integration.md).

## Из архива релиза

Архив релиза называется `gitpoulpi-<version>-<target>.zip` (Windows) или
`.tar.gz` (Linux, macOS), рядом с ним лежит файл `.sha256`. Сверьте архив с
файлом `.sha256`, распакуйте его и поместите три исполняемых файла в папку из
вашего `PATH` (оставьте `gitpoulpi-updater` и маркер `.gitpoulpi-dist` рядом с
ними: маркер гласит `channel=archive`, так что сборка может обновлять себя, см.
[Обновления](updates.md)). `INSTALL.txt` в архиве описывает те же шаги.

### Проверка скачанного файла

Каждый файл выпуска подписан ключом проекта [minisign](https://jedisct1.github.io/minisign/), и GitPoulpi сам проверяет эту подпись перед установкой обновления. Для файла, скачанного вручную, подпись лежит рядом в файле `.minisig`. Открытый ключ (идентификатор ключа `B461FFCBF6F6A4F2`) закоммичен в репозитории как `packaging/minisign.pub`:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Прочтите доверенный комментарий, который выводит команда: он должен быть `file:<имя файла> version:<ожидаемая вами версия>`. Подпись, которая верна, но называет другой файл или другую версию, не является подписью вашего файла.

Если выпуск собран с аттестациями происхождения сборки (переменная репозитория `ATTESTATIONS`), GitHub CLI проверяет, где файл был собран:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

Файл `.sha256` лишь выявляет повреждённую загрузку. Подпись - это то, что показывает, что файл принадлежит проекту.

## С установщиком

К каждому выпуску прилагается установщик для каждой ОС. Оба делают одни и те же
выборы (ярлыки, PATH, псевдоним команды вроде `gp`), вызывая сам установленный
`gitpoulpi` - тот же код, что стоит за Настройки > Общие > Лаунчер, поэтому то,
что настроил установщик, и то, что показывают Настройки, всегда совпадает.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Первый диалог спрашивает, для кого установка:

| Выбор | Куда | Права администратора | Что это значит для обновлений |
|---|---|---|---|
| **Только для меня** (по умолчанию) | `%LOCALAPPDATA%\Programs\GitPoulpi` | не нужны | GitPoulpi заменяет собственные файлы: **Обновить и перезапустить** в [панели обновлений](updates.md), без запроса. |
| **Для всех пользователей** | `Program Files\GitPoulpi` | один запрос при установке | Каждое обновление просит подтверждение администратора (запрос Windows), а сеансы GitPoulpi других пользователей на компьютере сначала закрываются. |

Тот же выбор работает и без диалога: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` или `/ALLUSERS`.

Флажки: ярлык в меню «Пуск», ярлык на рабочем столе, добавление в PATH. На отдельной странице спрашивается **псевдоним команды**: по умолчанию `gp`, можно любое имя, пусто - без псевдонима (`/ALIAS=name` или `/ALIAS=` в командной строке). В PowerShell `gp` уже занят командой `Get-ItemProperty`, поэтому там выберите другое имя или пользуйтесь `gitpoulpi`.

Установщик копирует исполняемые файлы (включая `gitpoulpi-updater`, небольшую программу, которая подменяет файлы во время обновления) и маркер `.gitpoulpi-dist`. Маркер записывает, как был установлен GitPoulpi (`inno-user` или `inno-machine`), и именно это панель обновлений показывает как **Установлено через**. Повторный запуск установщика сохраняет ваш выбор (область, ярлыки, псевдоним).

Удаляйте через Параметры Windows > Приложения: запускается `gitpoulpi uninstall` (псевдоним, записи PATH, ярлыки), затем файлы удаляются, включая остатки `.old` от обновления. Ваши настройки и репозитории остаются.

### Linux и macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Скрипт выбирает архив для вашей ОС и процессора, сверяет его `.sha256`, ставит (канал `archive`: GitPoulpi обновляет себя на месте) в
`~/.local/opt/gitpoulpi` и связывает `gitpoulpi` и `gitpoulpi-mcp` ссылками в
`~/.local/bin`. Root не нужен, за пределами домашней папки ничего не
затрагивается. Без `--yes` он спрашивает про псевдоним (по умолчанию `gp`), PATH
и ярлыки.

| Флаг | Действие |
|---|---|
| `--prefix DIR` | Поставить файлы программы в `DIR` (абсолютный путь). |
| `--version X.Y.Z` | Поставить этот выпуск вместо последнего. |
| `--archive FILE` | Взять локальный архив; рядом должен лежать `FILE.sha256` (`--no-verify` это пропускает). Сеть не используется. |
| `--alias NAME` / `--no-alias` | Сделать так, чтобы `NAME` запускал GitPoulpi (`gitpoulpi alias create`), либо без псевдонима. |
| `--no-path` | Не добавлять GitPoulpi в PATH новых оболочек (иначе выполняется `gitpoulpi path add`). |
| `--desktop` | Создать также ярлыки в меню приложений и на рабочем столе. |
| `--yes` | Ничего не спрашивать: действуют флаги и значения по умолчанию (псевдоним `gp`, PATH да, ярлыки нет). |
| `--uninstall` | Выполнить `gitpoulpi uninstall`, затем удалить ссылки и папку. |

Для поиска последнего выпуска репозиторий должен быть открытым; для закрытого
скачайте архив в браузере и используйте `--archive`.

## Через менеджер пакетов

Каждый канал ниже записывает рядом с программами один и тот же маркер `.gitpoulpi-dist` со своим именем канала. GitPoulpi читает его, чтобы знать, кто владеет обновлениями: ими владеет менеджер пакетов, поэтому GitPoulpi никогда не подменяет файлы у него за спиной. Панель обновлений (щёлкните по версии в строке заголовка) тогда выполняет собственную команду менеджера пакетов или показывает её, чтобы вы могли скопировать. См. [Обновления](updates.md).

| Канал | Установка | Примечания |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Устанавливает для всех пользователей, поэтому обновления просят подтверждение администратора. |
| **winget** (Windows) | `winget install --id GitPoulpi.GitPoulpi` | Добавьте `--scope user` или `--scope machine` для выбора; область machine просит подтверждение администратора при установке и при каждом обновлении. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi`, затем `scoop install gitpoulpi` | Для пользователя, права администратора не нужны. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | Tap называется `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | см. [репозиторий apt](#apt-debian-и-ubuntu) | Подписанный репозиторий; `.deb` также прилагается к каждому выпуску. |
| **MSI** (Windows, управляемая) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Для администраторов: для всех пользователей, без обновлений из приложения. См. [Корпоративное распространение](enterprise-distribution.md#msi-с-intune-или-групповой-политикой). |
| **AUR** (Arch Linux) | `yay -S gitpoulpi-bin` (или `paru -S gitpoulpi-bin`) | Без помощника: `git clone https://aur.archlinux.org/gitpoulpi-bin.git`, затем `cd gitpoulpi-bin && makepkg -si`. |
| **cargo** | `cargo binstall gitpoulpi-cli` | Скачивает архив выпуска вместо компиляции. |
| **cargo, из исходников** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Компилирует; нужен набор инструментов Rust. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | См. [Запуск в Docker](docker.md). Тот же образ есть на `ghcr.io/poulpi-lab/gitpoulpi`. |

Названия выше - те, в которые публикует процесс выпуска (см. [Выпуск версий](../../developer/releasing.md)): канал, который ещё не заработал, отвечает «не найдено», а архив выпуска или установщик работают всегда.

Действует собственная обработка PATH менеджера пакетов: Chocolatey, winget, Scoop и Homebrew сами добавляют программы в PATH, и ни один из них не создаёт псевдоним `gp` (выполните `gitpoulpi alias create gp` или Настройки > Общие > Лаунчер).

### apt (Debian и Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

Репозиторий поддерживает `amd64` и `arm64` и подписан ключом, принадлежащим проекту. Программы устанавливаются в `/usr/lib/gitpoulpi` со ссылками в `/usr/bin`. Пакет зависит от `git` и рекомендует `git-lfs`. Без репозитория установите `.deb` выпуска командой `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: тогда GitPoulpi скажет вам самому скачать следующий `.deb`, потому что команды `apt` он предлагает, только если находит репозиторий в `/etc/apt/sources.list.d/gitpoulpi.*`.

## Из исходников

Нужен свежий стабильный набор инструментов Rust (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Исполняемые файлы окажутся в `target/release/`. Профиль `release` собирается
быстро. Для самых маленьких и быстрых исполняемых файлов используйте профиль
`dist`. Он собирается дольше:

```
cargo build --profile dist
```

Скрипты `build-opti` в корне репозитория запускают эту сборку для трёх
исполняемых файлов и выводят их размеры:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd или PowerShell)
```

Чтобы собрать только нужные вам языки и функции, см.
[Пользовательские сборки](custom-builds.md).

## Собрать архив релиза самостоятельно

```
cargo run -p xtask -- dist
```

Это собирает профиль `dist` и записывает
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) или `.tar.gz` (другие
системы). В архиве три исполняемых файла, `INSTALL.txt`, `README.md`, `LICENSE`
и `CHANGELOG.md`. Рядом записывается файл `.sha256`.

## Обновления

Строка заголовка показывает запущенную версию (`v1.0.0`) между названием приложения и путём к репозиторию. Когда известен более новый выпуск (или идёт обновление), щелчок по ней открывает панель обновлений с версией, способом установки GitPoulpi и тем, что делать с новым выпуском; в остальных случаях щелчок открывает вкладку «Заметки о выпуске». `Ctrl+N`, затем `u`, или **Обновления: показать обновление** в палитре команд (`Ctrl+P`) всегда открывает панель обновлений, а **Справка: заметки о выпуске** всегда открывает список изменений.

Раз в сутки в фоне GitPoulpi проверяет место, где публикуются его выпуски (по умолчанию GitHub), на наличие нового (`updates.mode`, [Настройки > Синхронизация](settings.md#синхронизация)). Если он есть, чип показывает `доступна v0.2.0` акцентным цветом.

Что делает панель, зависит от способа установки GitPoulpi: архив выпуска или установщик Windows заменяют собственные файлы, менеджер пакетов выполняет свою команду, управляемая установка или контейнер сообщают, как до них доходят обновления. Сборка, выполненная через `cargo build`, никогда не обновляется. Полное описание, с возвратом, `auto` против `notify` и тем, что может заблокировать администратор, находится в [Обновлениях](updates.md).

## Проверка работоспособности

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` выводит версию и commit, из которого сделана сборка. Затем
следуйте разделу [Начало работы](getting-started.md).

## Удаление

`gitpoulpi uninstall` (его запускают оба установщика) удаляет псевдонимы команд,
записи PATH и ярлыки, созданные GitPoulpi, а затем файлы программы при установке
из выпуска; он отказывается работать в сборке без маркера `.gitpoulpi-dist`
(сборка через cargo), а `--keep-files` оставляет файлы программы. Агентам MCP он
недоступен. Без установщика удалите три исполняемых файла. GitPoulpi также хранит
следующие файлы:

| Что | Где |
|---|---|
| Настройки (`config.toml`) и сеансы | Папка конфигурации вашей ОС. Её меняет `GITPOULPI_CONFIG_DIR`. |
| Состояние каждого репозитория | По умолчанию `gitpoulpi/` внутри каталога `.git` репозитория (см. `session.per_repo_state_dir` в [Справочнике по конфигурации](config-reference.md)). |
| Кэши (аватары) | Папка кэша вашей ОС. |
| Токены и ключи API | Ваш credential helper. **Настройки > SSH и учётные данные > Забыть все имена пользователей и пароли** удаляет их. |

Точные пути см. в [Справочнике по конфигурации](config-reference.md).
