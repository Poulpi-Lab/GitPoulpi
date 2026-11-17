# Корпоративне розповсюдження

Для адміністраторів, що розгортають GitPoulpi на багатьох комп'ютерах зі своїх серверів: внутрішні стрічки пакунків, дзеркала, загальномашинна політика оновлення та типова конфігурація. Спершу прочитайте [Встановлення](installation.md) і [Оновлення](updates.md).

Усе тут тримається на двох фактах:

- Кожне встановлення записує, **як його встановлено**, у маркері `.gitpoulpi-dist` поруч із програмами (`channel=...`). GitPoulpi оновлює себе лише так, як дозволяє цей канал: він ніколи не замінює файли, що належать менеджеру пакунків, а MSI не чіпає ніколи.
- **Політика** (файл, а в Windows - реєстр) належить вам. Власний `config.toml` користувача ніколи не може її послабити.

## Chocolatey, із приватною стрічкою

Публічний пакунок завантажує інсталятор із GitHub. Для закритої мережі інтерналізуйте його:

1. Отримайте пакунок (`choco download gitpoulpi --internalize` з ліцензованими інструментами Chocolatey або завантажте `.nupkg` і розпакуйте його).
2. У `tools\chocolateyinstall.ps1` вкажіть `url64bit` на власну копію `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe` і тримайте `checksum64` рівним першому полю файлу `.sha256` випуску (або контрольній сумі вашої копії).
3. `choco pack`, потім надішліть `.nupkg` у свою стрічку (Nexus, Artifactory, ProGet, спільна тека).

Встановіть його з параметрами пакунка:

```
choco install gitpoulpi --source=https://nexus.example/repository/choco-internal/ ^
  --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco-internal/"
```

| Параметр | Дія |
|---|---|
| `/Channel:choco-internal` | Записує `channel=choco-internal` у маркер, тож GitPoulpi знає, що стрічка ваша, а не спільнотна стрічка Chocolatey. |
| `/Feed:<url>` | Записує адресу стрічки (`feed=` у маркері). GitPoulpi перевіряє її на нові версії та виконує `choco upgrade gitpoulpi -y --source=<feed>`. |

Без параметрів канал - `choco`, а перевірка йде до спільнотної стрічки Chocolatey. Пакунок встановлюється для всіх користувачів у Program Files, без псевдоніма `gp`.

Оновлення GitPoulpi виконує `choco upgrade` без `--params`, щоб нічого з розгорнутого вами не чіпалося. Тому Chocolatey має пам'ятати параметри пакунка з першого встановлення (`useRememberedArgumentsForUpgrades`, типово ввімкнено з Chocolatey 2.0; у старіших версіях `choco feature enable -n useRememberedArgumentsForUpgrades`). Інакше оновлення записало б простий канал `choco`. `check_url` політики (нижче) тримає перевірку й оновлення спрямованими на вашу стрічку, що б не казав маркер, і має перевагу над стрічкою з маркера.

**Ніщо в пакунку не пише `policy.toml` чи `config.toml`.** Усе, що ви розгорнули, лишається, і при встановленні, і при кожному оновленні. Див. [Розгортання типового config.toml](#розгортання-типового-configtoml).

## Scoop, із приватним bucket

Покладіть `gitpoulpi.json` (маніфест із публічного bucket) у `bucket/` репозиторію Git, який ви розміщуєте, змініть `url` на вашу копію zip випуску і задайте `hash` як його sha256 (`autoupdate` можна лишити, якщо ваша копія слідує іменам випуску). Потім:

```
scoop bucket add corp https://git.example/scoop-corp.git
scoop install corp/gitpoulpi
```

`post_install` маніфесту переписує маркер із `channel=scoop`; тоді GitPoulpi сам виконує `scoop update gitpoulpi` і ніколи не міняє файли за спиною Scoop. Scoop встановлює для користувача й не потребує прав адміністратора.

## apt, із дзеркалом

Репозиторій - це звичайне дерево тек (`dists/`, `pool/`, `gitpoulpi-archive-keyring.gpg` і `.asc` ключа підпису та зразок `gitpoulpi.sources`), що віддається через HTTPS. Дзеркальте його через `wget --mirror`, `rsync` чи ваш менеджер артефактів, віддавайте внутрішньо й встановіть ключ та джерело на кожному комп'ютері:

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

Лишайте ім'я файлу `gitpoulpi.sources` (або `gitpoulpi.list`): GitPoulpi показує команди `apt` лише коли знаходить `/etc/apt/sources.list.d/gitpoulpi.*`. Використовуйте ключ проєкту: дзеркало лишається підписаним ним, а `Signed-By` закріплює саме цей ключ лише для цього репозиторію. Репозиторій тримає останні п'ять версій кожного пакунка.

**`.deb` офлайн:** кожен випуск додає `gitpoulpi_<version>_<arch>.deb`. Скопіюйте його й виконайте `sudo apt install ./gitpoulpi_<version>_<arch>.deb`. Маркер каже `channel=apt`; без файлу джерела GitPoulpi каже користувачеві завантажити наступний `.deb` замість показу команди `apt`.

## MSI, з Intune чи груповою політикою

`gitpoulpi-<version>-x86_64-pc-windows-msvc.msi` встановлює для всіх користувачів і призначений для керування: панель GitPoulpi лише каже «керується вашим адміністратором» (з вашим `message`), вона ніколи не оновлює встановлення MSI. Оновлюйте розгортанням нового MSI; це мажорне оновлення, тож стару версію замінюють.

```
msiexec /i gitpoulpi-0.2.0-x86_64-pc-windows-msvc.msi /qn ^
  ADDTOPATH=1 UPDATES_MODE=notify UPDATES_PIN=0.2 ^
  UPDATES_MESSAGE="Updates are deployed by IT."
```

| Властивість | Дія |
|---|---|
| `INSTALLFOLDER` | Тека встановлення (типово `Program Files\GitPoulpi`). |
| `ADDTOPATH` | `1` (типово) додає теку до PATH машини, `0` не додає. |
| `UPDATES_MODE` | `mode` політики: `off`, `notify` або `auto`. |
| `UPDATES_PIN` | `pin` політики, наприклад `0.2` або `0.2.4`. |
| `UPDATES_CHECK` | `check` політики: `github`, `nuget`, `manifest` або `none`. |
| `UPDATES_CHECK_URL` | `check_url` політики. |
| `UPDATES_MESSAGE` | `message` політики, що показується в панелі оновлення. |

Властивість записує своє значення в `HKLM\SOFTWARE\Policies\GitPoulpi\Updates` лише коли ви її задали, тож незадана лишає реєстр (і файл політики) у спокої. **Intune:** розгорніть MSI як застосунок line-of-business або Win32 з наведеною вище командою. **Групова політика:** встановлення програм не може передавати властивості, тож або використайте трансформацію (`.mst`), або задайте значення реєстру через Group Policy Preferences, див. нижче.

## Homebrew, із внутрішнім tap

Скопіюйте `gitpoulpi.rb` із публічного tap у `Formula/` репозиторію Git, який ви розміщуєте, змініть рядки `url` на ваші копії архівів випуску й оновіть кожен `sha256` (перше поле файлу `.sha256` поруч із кожним архівом). Потім:

```
brew tap corp/gitpoulpi https://git.example/homebrew-gitpoulpi.git
brew install corp/gitpoulpi/gitpoulpi
```

Формула встановлює маркер із `channel=homebrew`: GitPoulpi сам виконує `brew upgrade gitpoulpi`, що читає ваш tap.

## Дзеркало реєстру Docker

Назва образу - `poulpilab/gitpoulpi`. Панель завжди показує цю назву, тож дзеркало має бути прозорим.

- **Кеш наскрізної передачі:** запустіть реєстр як проксі Docker Hub (образ `registry` з `proxy.remoteurl: https://registry-1.docker.io`) і задайте `"registry-mirrors": ["https://registry.example"]` у `daemon.json` демона Docker. Тоді `docker pull poulpilab/gitpoulpi` іде через ваш кеш.
- **Копія:** `docker pull poulpilab/gitpoulpi:<v>`, `docker tag poulpilab/gitpoulpi:<v> registry.example/tools/gitpoulpi:<v>`, `docker push registry.example/tools/gitpoulpi:<v>`, потім запустіть цю назву. Панель усе одно показує публічну, що годиться як покажчик на версію.

Як його запускати, див. [Запуск у Docker](docker.md).

## Файл політики і реєстр

Політика задає, як поводяться оновлення на комп'ютері, і користувачі не можуть її змінити. Її читають із:

| ОС | Файл політики |
|---|---|
| Windows | `%ProgramData%\GitPoulpi\policy.toml` |
| Linux | `/etc/gitpoulpi/policy.toml` |
| macOS | `/Library/Application Support/GitPoulpi/policy.toml` |
| Windows, також | ключ реєстру `HKLM\SOFTWARE\Policies\GitPoulpi\Updates`, по одному значенню `REG_SZ` на кожен ключ нижче, з тими самими іменами (для Group Policy, Intune і властивостей MSI) |

Файл має одну таблицю:

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

| Ключ | Значення | Дія |
|---|---|---|
| `mode` | `off`, `notify`, `auto` | Стеля: `off` < `notify` < `auto`. Користувачі можуть вибрати нижче значення, ніколи вище. `off` ховає перевірку й не робить жодного запиту. |
| `release_channel` | `stable`, `prerelease` | Примусово задає канал оновлення. |
| `pin` | `1.4` або `1.4.2` | Найвища запропонована версія: `1.4` дозволяє будь-яку `1.4.x`, `1.4.2` дозволяє щонайбільше `1.4.2`. |
| `check` | `github`, `nuget`, `manifest`, `none` | Де питають «який останній випуск». Типово для встановлення: GitHub, стрічка Chocolatey для каналів Chocolatey. `none` ніколи не перевіряє. |
| `check_url` | URL | З `nuget`: стрічка. З `manifest`: адреса `latest.json`. З `github`: база GitHub API, для GitHub Enterprise Server (додайте `repo`). |
| `repo` | `owner/name` | Репозиторій, що перевіряється з `check = "github"`. |
| `download_base` | URL | Дзеркало файлів випуску (ті самі імена файлів) для каналів архіву й інсталятора. Завантаження з цього хоста дозволені на додачу до GitHub. |
| `allow_self_update` | `true`, `false` | `false` перетворює кожну дію «замінити файли GitPoulpi» і «виконати менеджер пакунків» на примітку чи команду для копіювання. Типово `true`. |
| `message` | текст | Показується в панелі оновлення («Від вашого адміністратора: ...»). |

**Пріоритет**, від найсильнішого: реєстр, файл політики, `[updates]` у `config.toml` користувача, `feed=` з маркера, вбудовані типові значення. Ключ, який задає політика, у Налаштуваннях має позначку **Задано вашим адміністратором** і не редагується там.

**Безпечний збій:** файл політики чи значення реєстру, що не читається (синтаксична помилка TOML, `mode = "sometimes"`, `pin`, що не є версією), трактується як `mode = notify` і `allow_self_update = false`, а один рядок іде в журнал застосунку. Одруківка ніколи не лишає парк машин, що оновлюють себе самі. Невідомий ключ ігнорується з попередженням. У Windows крок, що тримає права адміністратора, читає лише політику машини, ніколи конфігурацію користувача.

### Приклади

Вимкнути оновлення (заблокований образ, ізольована мережа):

```toml
[updates]
mode = "off"
```

Закріпити парк машин на лінії 1.4, лише сповіщати й сказати, до кого звертатися:

```toml
[updates]
mode = "notify"
pin = "1.4"
allow_self_update = false
message = "Updates go through the software center. Ask the service desk."
```

Перевіряти внутрішню стрічку Chocolatey (NuGet):

```toml
[updates]
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
```

Така сама політика в Windows через реєстр (PowerShell):

```
New-Item -Path HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Force
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name mode -Value notify
Set-ItemProperty HKLM:\SOFTWARE\Policies\GitPoulpi\Updates -Name pin -Value 1.4
```

### latest.json і download_base: дзеркало випуску

Скопіюйте файли випуску (архіви, setup, файли `.sha256`, `latest.json`, **кожен файл підпису `.minisig`** і `security.json`) в одну теку внутрішнього вебсервера, зберігши імена. `latest.json` додається до кожного випуску:

```json
{
  "version": "1.4.2",
  "notes": "Markdown release notes",
  "assets": { "gitpoulpi-1.4.2-x86_64-pc-windows-msvc.zip": "<sha256>" }
}
```

Тоді політика така:

```toml
[updates]
check = "manifest"
check_url = "https://mirror.example/gitpoulpi/latest.json"
download_base = "https://mirror.example/gitpoulpi/"
```

Канали архіву й інсталятора перевіряють маніфест і завантажують із `download_base`, з тією самою перевіркою, що й з GitHub. Дзеркалу достатньо копіювати файли; API йому не потрібен.

Підписи подорожують разом із файлами, тож дзеркало не може їх послабити:

- `latest.json` вважається довіреним, лише коли поруч лежить `latest.json.minisig` і перевіряється за ключем проєкту, вбудованим у GitPoulpi. Маніфест без підпису чи відредагований після підписання відхиляється (хеші в ньому - це те, із чим звіряють завантаження).
- Кожен архів завантажується разом із `<archive>.minisig` з того самого хоста і встановлюється, лише коли цей підпис перевіряється й називає цей файл та цю версію. Дзеркало, що скопіювало архів, але не його `.minisig`, змушує оновлення зазнати невдачі з «no signature», і нічого не змінюється.
- `security.json` (сповіщення безпеки, див. [Оновлення](updates.md#сповіщення-безпеки)) і `security.json.minisig` шукають поруч із маніфестом. Дзеркало, що їх не копіює, просто не показує сповіщень.

Копіюйте файли без змін: підпис охоплює точні байти. Сповіщення безпеки перевидаються для останнього випуску без нової збірки, тож оновлюйте `latest.json`, `security.json` та їхні файли `.minisig` зі сторінки випуску щоразу, коли оновлюєте дзеркало.

## Розгортання типового config.toml

GitPoulpi не має загальномашинного `config.toml`: налаштування лежать у теці кожного користувача (`%APPDATA%\GitPoulpi\config.toml`, `~/.config/gitpoulpi/config.toml`, `~/Library/Application Support/GitPoulpi/config.toml`). Щоб дати новим користувачам ваші типові значення, скопіюйте розріджений `config.toml` (лише потрібні вам ключі, див. [Довідник конфігурації](config-reference.md)) у цю теку, коли користувач його не має: скрипт входу чи Intune у Windows, `/etc/skel/.config/gitpoulpi/` у Linux, скрипт першого входу в macOS. Ніщо в пакунках GitPoulpi ніколи не перезаписує конфігурацію, тож нею керуєте ви. Не кладіть туди правила оновлення, коли вони мають діяти обов'язково: використовуйте політику, яку користувач не може перевизначити. Портативну чи окрему для завдання конфігурацію також можна вибрати через `GITPOULPI_CONFIG_DIR`.
