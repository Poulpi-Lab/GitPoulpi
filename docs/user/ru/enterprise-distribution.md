[English](../enterprise-distribution.md) · [Čeština](../cs/enterprise-distribution.md) · [Deutsch](../de/enterprise-distribution.md) · [Español](../es/enterprise-distribution.md) · [Français](../fr/enterprise-distribution.md) · [Italiano](../it/enterprise-distribution.md) · [日本語](../ja/enterprise-distribution.md) · [한국어](../ko/enterprise-distribution.md) · [Nederlands](../nl/enterprise-distribution.md) · [Polski](../pl/enterprise-distribution.md) · [Português (Brasil)](../pt-BR/enterprise-distribution.md) · [Português (Portugal)](../pt-PT/enterprise-distribution.md) · **Русский** · [Svenska](../sv/enterprise-distribution.md) · [Türkçe](../tr/enterprise-distribution.md) · [Українська](../uk/enterprise-distribution.md) · [简体中文](../zh-CN/enterprise-distribution.md)

# Корпоративное распространение

Для администраторов, которые развёртывают GitPoulpi на многих компьютерах со своих серверов: внутренние репозитории пакетов, зеркала, политика обновлений для всего компьютера и конфигурация по умолчанию. Сначала прочитайте [Установку](installation.md) и [Обновления](updates.md).

Всё здесь определяют два факта:

- Каждая установка записывает **способ своей установки** в маркер `.gitpoulpi-dist` рядом с программами (`channel=...`). GitPoulpi обновляет себя только так, как позволяет этот канал: он никогда не заменяет файлы, которыми владеет менеджер пакетов, и никогда не трогает MSI.
- **Политика** (файл, а в Windows ещё и реестр) принадлежит вам. Собственный `config.toml` пользователя никогда не может её ослабить.

## Chocolatey с частным репозиторием

Общедоступный пакет скачивает установщик с GitHub. Для закрытой сети его нужно интернализировать:

1. Получите пакет (`choco download gitpoulpi --internalize` с лицензированными инструментами Chocolatey либо скачайте `.nupkg` и распакуйте его).
2. В `tools\chocolateyinstall.ps1` направьте `url64bit` на свою копию `gitpoulpi-<версия>-x86_64-pc-windows-msvc-setup.exe` и оставьте `checksum64` равным первому полю файла `.sha256` выпуска (или контрольной сумме вашей копии).
3. Выполните `choco pack`, затем отправьте `.nupkg` в свой репозиторий (Nexus, Artifactory, ProGet, общая папка).

Установите его с параметрами пакета:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Параметр | Действие |
|---|---|
| `/Channel:choco-internal` | Записывает `channel=choco-internal` в маркер, чтобы GitPoulpi знал, что репозиторий ваш, а не общий репозиторий Chocolatey. |
| `/Feed:<url>` | Записывает адрес репозитория (`feed=` в маркере). GitPoulpi проверяет по нему наличие новых версий и выполняет `choco upgrade gitpoulpi -y --source=<feed>`. |

Без параметров канал - `choco`, а проверка идёт в общий репозиторий Chocolatey. Пакет устанавливается для всех пользователей в Program Files, без псевдонима `gp`.

Обновление GitPoulpi выполняет `choco upgrade` без `--params`, чтобы ничего из развёрнутого вами не затрагивалось. Поэтому Chocolatey должен помнить параметры пакета первой установки (`useRememberedArgumentsForUpgrades`, включено по умолчанию начиная с Chocolatey 2.0; в более старых версиях `choco feature enable -n useRememberedArgumentsForUpgrades`). Иначе при обновлении запишется обычный канал `choco`. Параметр политики `check_url` (ниже) удерживает проверку и обновление на вашем репозитории независимо от маркера и имеет приоритет над репозиторием из маркера.

**Ничто в пакете не записывает `policy.toml` или `config.toml`.** Всё, что вы развернули, остаётся и при установке, и при каждом обновлении. См. [Развёртывание config.toml по умолчанию](#развёртывание-configtoml-по-умолчанию).

## Scoop с частным bucket

Положите `gitpoulpi.json` (манифест из общего bucket) в `bucket/` репозитория Git, который вы размещаете, замените `url` на свою копию zip выпуска и задайте `hash` равным её sha256 (`autoupdate` можно оставить, если ваша копия следует именам выпусков). Затем:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

`post_install` манифеста перезаписывает маркер с `channel=scoop`; тогда GitPoulpi сам выполняет `scoop update gitpoulpi` и никогда не подменяет файлы за спиной у Scoop. Scoop устанавливает для пользователя и не требует прав администратора.

## apt с зеркалом

Репозиторий - это обычное дерево каталогов (`dists/`, `pool/`, `gitpoulpi-archive-keyring.gpg` и `.asc` с ключом подписи и образец `gitpoulpi.sources`), раздаваемое по HTTPS. Сделайте его зеркало через `wget --mirror`, `rsync` или ваш менеджер артефактов, раздавайте внутри сети и установите ключ и источник на каждом компьютере:

```
sudo install -d /etc/apt/keyrings
sudo install -m 644 gitpoulpi-archive-keyring.gpg /etc/apt/keyrings/gitpoulpi.gpg
```

`/etc/apt/sources.list.d/gitpoulpi.sources`:

```
Types: deb
URIs: https://mirror.example/gitpoulpi/apt
Suites: stable
Components: main
Architectures: amd64 arm64
Signed-By: /etc/apt/keyrings/gitpoulpi.gpg
```

Оставьте имя файла `gitpoulpi.sources` (или `gitpoulpi.list`): GitPoulpi показывает команды `apt`, только если находит `/etc/apt/sources.list.d/gitpoulpi.*`. Используйте ключ проекта: зеркало остаётся подписанным им, а `Signed-By` закрепляет именно этот ключ только для этого репозитория. Репозиторий хранит последние пять версий каждого пакета.

**`.deb` без сети:** к каждому выпуску прилагается `gitpoulpi_<версия>_<arch>.deb`. Скопируйте его и выполните `sudo apt install ./gitpoulpi_<версия>_<arch>.deb`. Маркер гласит `channel=apt`; без файла источника GitPoulpi предлагает пользователю скачать следующий `.deb` вместо показа команды `apt`.

## MSI с Intune или групповой политикой

`gitpoulpi-<версия>-x86_64-pc-windows-msvc.msi` устанавливается для всех пользователей и предназначен для управляемой установки: панель GitPoulpi лишь сообщает «управляется вашим администратором» (с вашим `message`), MSI-установку она никогда не обновляет. Обновляйте, развёртывая новый MSI; это основное обновление, поэтому старая версия заменяется.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Свойство | Действие |
|---|---|
| `INSTALLFOLDER` | Папка установки (по умолчанию `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (по умолчанию) добавляет папку в PATH компьютера, `0` не добавляет. |
| `UPDATES_MODE` | `mode` политики: `off`, `notify` или `auto`. |
| `UPDATES_PIN` | `pin` политики, например `0.2` или `0.2.4`. |
| `UPDATES_CHECK` | `check` политики: `github`, `nuget`, `manifest` или `none`. |
| `UPDATES_CHECK_URL` | `check_url` политики. |
| `UPDATES_MESSAGE` | `message` политики, показывается в панели обновлений. |

Свойство записывает своё значение в `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, только если вы его задали, поэтому незаданное оставляет реестр (и файл политики) в покое. **Intune:** разверните MSI как бизнес-приложение или приложение Win32 с командой выше. **Групповая политика:** установка программ не может передавать свойства, поэтому используйте преобразование (`.mst`) либо задайте значения реестра через Group Policy Preferences, см. ниже.

## Homebrew с внутренним tap

Скопируйте `gitpoulpi.rb` из общего tap в `Formula/` репозитория Git, который вы размещаете, замените строки `url` на свои копии архивов выпуска и обновите каждый `sha256` (первое поле файла `.sha256` рядом с каждым архивом). Затем:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

Формула устанавливает маркер с `channel=homebrew`: GitPoulpi сам выполняет `brew upgrade gitpoulpi`, и тот читает ваш tap.

## Зеркало реестра Docker

Образ называется `poulpilab/gitpoulpi`. Панель всегда показывает это имя, поэтому зеркало должно быть прозрачным.

- **Кэширующий прокси:** запустите реестр как прокси Docker Hub (образ `registry` с `proxy.remoteurl: https://registry-1.docker.io`) и задайте `"registry-mirrors": ["https://registry.example"]` в `daemon.json` демона Docker. Тогда `docker pull poulpilab/gitpoulpi` идёт через ваш кэш.
- **Копия:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, затем запускайте это имя. Панель по-прежнему показывает общедоступное, и это нормально как указание на версию.

О том, как его запускать, см. [Запуск в Docker](docker.md).

## Файл политики и реестр

Политика задаёт поведение обновлений на компьютере, и пользователи не могут её менять. Она читается из:

| ОС | Файл политики |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, также | ключ реестра `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, по одному значению `REG_SZ` на каждый ключ ниже, с теми же именами (для групповой политики, Intune и свойств MSI) |

В файле одна таблица:

```toml
[updates]
mode = "notify"
release_channel = "stable"
pin = "1.4"
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
repo = "Poulpi-Lab/GitPoulpi"
download_base = "https://mirror.example/gitpoulpi/"
allow_self_update = false
message = "Updates are deployed by IT."
```

| Ключ | Значения | Действие |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Потолок: `off` < `notify` < `auto`. Пользователи могут выбрать меньшее значение, но не большее. `off` скрывает проверку и не делает запросов. |
| `release_channel` | `stable`, `prerelease` | Принудительно задаёт канал обновлений. |
| `pin` | `1.4` или `1.4.2` | Самая высокая предлагаемая версия: `1.4` допускает любую `1.4.x`, `1.4.2` допускает не выше `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Где спрашивают «какой выпуск последний». По умолчанию для установки: GitHub, для каналов Chocolatey - репозиторий Chocolatey. `none` никогда не проверяет. |
| `check_url` | URL | С `nuget`: репозиторий. С `manifest`: адрес `latest.json`. С `github`: база API GitHub, для GitHub Enterprise Server (добавьте `repo`). |
| `repo` | `owner/name` | Репозиторий, проверяемый при `check = "github"`. |
| `download_base` | URL | Зеркало файлов выпуска (с теми же именами файлов) для каналов архива и установщика. Загрузки с этого хоста разрешены в дополнение к хостам GitHub. |
| `allow_self_update` | `true`, `false` | `false` превращает каждое действие «заменить файлы GitPoulpi» и «запустить менеджер пакетов» в примечание или команду для копирования. По умолчанию `true`. |
| `message` | текст | Показывается в панели обновлений («От вашего администратора: ...»). |

**Приоритет**, от сильнейшего: реестр, файл политики, `[updates]` в `config.toml` пользователя, `feed=` маркера, встроенные значения по умолчанию. Ключ, заданный политикой, в настройках отображается как **Задано администратором** и не редактируется там.

**Безопасный отказ:** файл политики или значение реестра, которые невозможно прочитать (синтаксическая ошибка TOML, `mode = "sometimes"`, `pin`, не являющийся версией), рассматриваются как `mode = notify` и `allow_self_update = false`, а в журнал приложения пишется одна строка. Опечатка никогда не оставит парк компьютеров обновляться самостоятельно. Неизвестный ключ игнорируется с предупреждением. В Windows шаг с правами администратора читает только политику компьютера, но не конфигурацию пользователя.

### Примеры

Выключить обновления (заблокированный образ, изолированная сеть):

```toml
[updates]
mode = "off"
```

Закрепить парк на линии 1.4, только уведомления, и указать, к кому обращаться:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Проверять внутренний репозиторий Chocolatey (NuGet):

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Такая же политика в Windows через реестр (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json и download_base: зеркало выпуска

Скопируйте файлы выпуска (архивы, программу установки, файлы `.sha256`, `latest.json`, **каждый файл подписи `.minisig`** и `security.json`) в одну папку внутреннего веб-сервера, сохранив имена. `latest.json` прилагается к каждому выпуску:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Тогда политика такая:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Каналы архива и установщика проверяют манифест и скачивают с `download_base`, с той же проверкой, что и с GitHub. Зеркалу достаточно копировать файлы; API ему не нужен.

Подписи путешествуют вместе с файлами, поэтому зеркало не может их ослабить:

- `latest.json` считается доверенным, только когда рядом лежит `latest.json.minisig`, проверяемый по ключу проекта, встроенному в GitPoulpi. Манифест без подписи или изменённый после подписания отклоняется (хэши внутри него - это то, по чему проверяются загрузки).
- Каждый архив скачивается вместе с `<archive>.minisig` с того же хоста и устанавливается, только когда эта подпись проверяется и называет этот файл и эту версию. Зеркало, скопировавшее архив, но не его `.minisig`, приведёт к ошибке обновления «нет подписи», и ничего не изменится.
- `security.json` (уведомления о безопасности, см. [Обновления](updates.md#уведомления-о-безопасности)) и `security.json.minisig` ищутся рядом с манифестом. Зеркало, которое их не копирует, просто не показывает уведомлений.

Копируйте файлы без изменений: подпись покрывает точные байты. Уведомления о безопасности переиздаются для последнего выпуска без новой сборки, поэтому при каждом обновлении зеркала заново берите `latest.json`, `security.json` и их файлы `.minisig` со страницы выпуска.

## Развёртывание config.toml по умолчанию

У GitPoulpi нет `config.toml` для всего компьютера: настройки хранятся в папке каждого пользователя (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Чтобы дать новым пользователям ваши значения по умолчанию, копируйте разреженный `config.toml` (только нужные вам ключи, см. [Справочник по конфигурации](config-reference.md)) в эту папку, когда у пользователя его нет (значения интерфейса по умолчанию, такие как `ui.theme` или `ui.language`, помещайте в разреженный `tui.toml` рядом с ним, который терминальный UI читает сам; см. [Два файла](config-reference.md#два-файла-configtoml-и-tuitoml)): скрипт входа в систему или Intune в Windows, `/etc/skel/.config/gitpoulpi/` в Linux, скрипт первого входа в macOS. Ничто в пакетах GitPoulpi никогда не перезаписывает конфигурацию, так что она в вашем распоряжении. Не помещайте туда правила обновления, если они должны соблюдаться: используйте политику, которую пользователь не может переопределить. Переносимую или отдельную для задачи конфигурацию можно также выбрать через `GITPOULPI_CONFIG_DIR`.
