[English](../installation.md) · [Čeština](../cs/installation.md) · [Deutsch](../de/installation.md) · [Español](../es/installation.md) · [Français](../fr/installation.md) · [Italiano](../it/installation.md) · [日本語](../ja/installation.md) · [한국어](../ko/installation.md) · [Nederlands](../nl/installation.md) · [Polski](../pl/installation.md) · [Português (Brasil)](../pt-BR/installation.md) · [Português (Portugal)](../pt-PT/installation.md) · [Русский](../ru/installation.md) · [Svenska](../sv/installation.md) · [Türkçe](../tr/installation.md) · **Українська** · [简体中文](../zh-CN/installation.md)

# Встановлення

GitPoulpi має три виконувані файли:

| Виконуваний файл | Що це |
|---|---|
| `gitpoulpi-tui` | Інтерфейс користувача в терміналі. |
| `gitpoulpi` | Інтерфейс командного рядка (`--json` для скриптів). Без підкоманди відкриває інтерфейс. |
| `gitpoulpi-mcp` | Сервер MCP для ШІ-агентів. Див. [MCP та агенти](mcp-and-agents.md). |

Поряд із ними в архіві лежить `gitpoulpi-launch`, крихітна програма, яку запускають ярлики, щоб показати вже відкрите вікно GitPoulpi (`gitpoulpi launch`). У Windows вона не має вікна консолі, тож клацання ніколи не викликає його блимання. Вона встановлюється, оновлюється й видаляється разом з іншими.

## Що вам потрібно

- **Системний `git`** у вашому `PATH`. GitPoulpi використовує libgit2 для
  більшості операцій, а ваш `git` - для решти.
  Потрібна версія **2.34 або новіша**. У macOS підходить `git` з Apple Command Line Tools, а також з Homebrew.
- **Термінал** з підтримкою миші. kitty, Ghostty, WezTerm, Rio і iTerm2 (а також Windows Terminal, foot і Tabby через sixel) малюють граф
  commit'ів як зображення. Інші термінали (консоль Windows, zellij, screen і
  більшість решти) показують символьний граф. Див.
  [Вибір термінала](terminals.md) і [Мультиплексори](multiplexers.md).
- **Nerd Font** або шрифт з емодзі для значків на панелі інструментів і в
  бічній панелі.
- За бажання: **`git-lfs`** 3.0 чи новіший для репозиторіїв Git LFS. Без нього
  файли LFS показуються як маленькі файли-вказівники, а GitPoulpi пояснює
  чому. Як його встановити, див. [Git LFS](lfs.md).
- За бажання: **credential helper** (Git Credential Manager, libsecret, `pass`,
  Keychain) для зберігання токенів хостингів і API-ключів ШІ. Див.
  [Хостинги й трекери issues](hosting-integration.md).

## З архіву релізу

Архів релізу - це `gitpoulpi-<version>-<target>.zip` (Windows) або `.tar.gz`
(Linux, macOS), з файлом `.sha256` поруч. Звірте архів із файлом `.sha256`,
розпакуйте його й покладіть три виконувані файли в папку з вашого `PATH`
(тримайте `gitpoulpi-updater` і маркер `.gitpoulpi-dist` поруч із ними: маркер каже
`channel=archive`, тож збірка може оновлювати себе, див. [Оновлення](updates.md)).
`INSTALL.txt` в архіві дає ті самі кроки.

### Перевірка завантаження

Кожен файл випуску підписано ключем проєкту [minisign](https://jedisct1.github.io/minisign/), і GitPoulpi сам перевіряє цей підпис, перш ніж встановити оновлення. Для файлу, завантаженого вручну, підписом є `.minisig` поруч із ним. Відкритий ключ (id ключа `B461FFCBF6F6A4F2`) закомічено як `packaging/minisign.pub` у репозиторії:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Прочитайте довірений коментар, який виводить команда: він має бути `file:<ім'я файлу> version:<очікувана вами версія>`. Підпис, що чинний, але називає інший файл чи іншу версію, не є підписом для вашого.

Якщо випуск зібрано з атестаціями походження збірки (змінна `ATTESTATIONS` репозиторію), GitHub CLI перевіряє, де зібрано файл:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

Файл `.sha256` лише виявляє пошкоджене завантаження. Підпис - це те, що показує, що файл належить проєкту.

## З інсталятором

До кожного випуску додано інсталятор для кожної ОС. Обидва роблять однакові вибори
(ярлики, PATH, псевдонім команди на кшталт `gp`), викликаючи сам встановлений
`gitpoulpi` - той самий код, що стоїть за Налаштування > Лаунчер, тож те,
що налаштував інсталятор, і те, що показують Налаштування, завжди збігаються.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

Перший діалог питає, для кого встановлення:

| Вибір | Де | Права адміністратора | Що це означає для оновлень |
|---|---|---|---|
| **Лише для мене** (типово) | `%LOCALAPPDATA%\Programs\GitPoulpi` | немає | GitPoulpi замінює власні файли: **Оновити й перезапустити** на [панелі оновлення](updates.md), без запиту. |
| **Для всіх користувачів** | `Program Files\GitPoulpi` | один запит під час встановлення | Кожне оновлення просить підтвердження адміністратора (запит Windows), а процеси GitPoulpi інших користувачів на комп'ютері спершу закриваються. |

Той самий вибір працює тихо: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` або `/ALLUSERS`.

Прапорці: ярлик у меню «Пуск», ярлик на робочому столі, додавання до PATH. Окрема сторінка питає **псевдонім команди**: типово `gp`, можна будь-яке ім'я, порожньо - без псевдоніма (`/ALIAS=name` або `/ALIAS=` у командному рядку). У PowerShell `gp` уже означає `Get-ItemProperty`, тож там оберіть інше ім'я або користуйтеся `gitpoulpi`.

Інсталятор копіює виконувані файли (зокрема `gitpoulpi-updater`, невелику програму, що міняє файли під час оновлення) і маркер `.gitpoulpi-dist`. Маркер записує, як встановлено GitPoulpi (`inno-user` або `inno-machine`), і саме це панель оновлення показує як **Встановлено через**. Повторний запуск setup зберігає ваші вибори (область, ярлики, псевдонім).

Видалення - у Параметри Windows > Застосунки: воно запускає `gitpoulpi uninstall` (псевдонім, записи PATH, ярлики) і видаляє файли, зокрема залишки `.old` від оновлення. Ваші налаштування й репозиторії лишаються.

### Linux і macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

Скрипт вибирає архів для вашої ОС і процесора, перевіряє його `.sha256`, встановлює (канал `archive`: GitPoulpi оновлює себе на місці) в
`~/.local/opt/gitpoulpi` і лінкує `gitpoulpi` та `gitpoulpi-mcp` у `~/.local/bin`.
Права root не потрібні, і нічого поза вашою домашньою папкою не чіпається. Без `--yes`
він питає про псевдонім (типово `gp`), PATH і ярлики.

| Прапорець | Дія |
|---|---|
| `--prefix DIR` | Встановити файли програми в `DIR` (абсолютний шлях). |
| `--version X.Y.Z` | Встановити цей випуск замість останнього. |
| `--archive FILE` | Взяти локальний архів; поруч має лежати `FILE.sha256` (`--no-verify` пропускає перевірку). Мережа не використовується. |
| `--alias NAME` / `--no-alias` | Зробити так, щоб `NAME` запускало GitPoulpi (`gitpoulpi alias create`), або без псевдоніма. |
| `--no-path` | Не додавати GitPoulpi до PATH нових оболонок (інакше викликається `gitpoulpi path add`). |
| `--desktop` | Також створити ярлики в меню застосунків і на робочому столі. |
| `--yes` | Нічого не питати: діють прапорці та типові значення (псевдонім `gp`, PATH - так, ярлики - ні). |
| `--uninstall` | Виконати `gitpoulpi uninstall`, потім прибрати посилання й папку. |

Пошук останнього випуску вимагає, щоб репозиторій був публічним; для приватного
завантажте архів у браузері й скористайтеся `--archive`.

## Через менеджер пакунків

Кожен канал нижче записує поруч із програмами той самий маркер `.gitpoulpi-dist` зі своєю назвою каналу. GitPoulpi читає його, щоб знати, кому належать оновлення: належать менеджеру пакунків, тож GitPoulpi ніколи не міняє файли за його спиною. Панель оновлення (клацніть по версії в рядку заголовка) тоді виконує власну команду менеджера пакунків або показує її, щоб ви її скопіювали. Див. [Оновлення](updates.md).

| Канал | Встановлення | Примітки |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Встановлює для всіх користувачів, тож оновлення просять підтвердження адміністратора. |
| **winget** (Windows) | `winget install --id GitPoulpi.GitPoulpi` | Додайте `--scope user` або `--scope machine` для вибору; область machine просить підтвердження адміністратора під час встановлення й кожного оновлення. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi`, потім `scoop install gitpoulpi` | Для користувача, без прав адміністратора. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | Tap - це `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | див. [репозиторій apt](#apt-debian-і-ubuntu) | Підписаний репозиторій; `.deb` також додано до кожного випуску. |
| **MSI** (Windows, кероване) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | Для адміністраторів: усі користувачі, без оновлень у застосунку. Див. [Корпоративне розповсюдження](enterprise-distribution.md#msi-з-intune-чи-груповою-політикою). |
| **AUR** (Arch Linux) | `yay -S gitpoulpi-bin` (або `paru -S gitpoulpi-bin`) | Без помічника: `git clone https://aur.archlinux.org/gitpoulpi-bin.git`, потім `cd gitpoulpi-bin && makepkg -si`. |
| **cargo** | `cargo binstall gitpoulpi-cli` | Завантажує архів випуску замість компіляції. |
| **cargo, з вихідного коду** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Компілює; потрібен тулчейн Rust. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | Див. [Запуск у Docker](docker.md). Той самий образ є на `ghcr.io/poulpi-lab/gitpoulpi`. |

Наведені назви - це ті, у які публікує процес випуску (див. [Випуск](../../developer/releasing.md)): канал, що ще не діє, відповідає «не знайдено», а архів випуску чи інсталятор працюють завжди.

Діє власна обробка PATH менеджера пакунків: Chocolatey, winget, Scoop і Homebrew самі кладуть програми в PATH, і жоден із них не створює псевдонім `gp` (виконайте `gitpoulpi alias create gp` або Налаштування > Лаунчер).

### apt (Debian і Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt-gitpoulpi/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

Репозиторій підтримує `amd64` і `arm64` і підписаний ключем, що належить проєкту. Програми потрапляють у `/usr/lib/gitpoulpi` із посиланнями в `/usr/bin`. Пакунок залежить від `git` і рекомендує `git-lfs`. Без репозиторію встановіть `.deb` випуску через `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: тоді GitPoulpi каже вам завантажити наступний `.deb` самому, бо пропонує команди `apt` лише коли знаходить репозиторій у `/etc/apt/sources.list.d/gitpoulpi.*`.

## З вихідного коду

Потрібен свіжий стабільний тулчейн Rust (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

Виконувані файли лежать у `target/release/`. Профіль `release` збирається
швидко. Для найменших і найшвидших виконуваних файлів використайте профіль
`dist`. Він збирається довше:

```
cargo build --profile dist
```

Скрипти `build-opti` у корені репозиторію виконують цю збірку для трьох
виконуваних файлів і виводять їхні розміри:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd or PowerShell)
```

Щоб зібрати лише потрібні вам мови та функції, див.
[Власні збірки](custom-builds.md).

## Власноручна збірка архіву релізу

```
cargo run -p xtask -- dist
```

Це збирає профіль `dist` і записує
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) або `.tar.gz` (інші
системи). В архіві три виконувані файли, `INSTALL.txt`, `README.md`, `LICENSE`
і `CHANGELOG.md`. Поруч записується файл `.sha256`.

## Оновлення

Рядок заголовка показує запущену версію (`v1.0.0`) між назвою застосунку та шляхом до репозиторію. Коли відомо про новіший випуск (або оновлення вже виконується), клацання по ній відкриває панель оновлення з версією, способом встановлення GitPoulpi і тим, що робити з новішим випуском; інакше клацання відкриває вкладку «Нотатки до випуску». `Ctrl+N`, потім `u`, або **Оновлення: показати панель оновлення** в палітрі команд (`Ctrl+P`) завжди відкриває панель оновлення, а **Довідка: нотатки до випуску** завжди відкриває журнал змін.

Раз на добу, у фоні, GitPoulpi питає там, де публікуються його випуски (типово GitHub), чи є новіший (`updates.mode`, [Налаштування > Синхронізація](settings.md#синхронізація)). Коли є, чип показує `доступна v0.2.0` кольором акценту.

Що робить панель, залежить від того, як встановлено GitPoulpi: архів випуску чи інсталятор Windows замінює власні файли, менеджер пакунків виконує власну команду, кероване чи контейнерне встановлення каже, як до нього доходять оновлення. Збірка, зроблена через `cargo build`, ніколи не оновлюється. Повний опис, із поверненням, `auto` проти `notify` і тим, що може заблокувати адміністратор, є в розділі [Оновлення](updates.md).

## Перевірка роботи

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` виводить версію та commit, з якого зібрано. Далі див.
[Початок роботи](getting-started.md).

## Видалення програми

`gitpoulpi uninstall` (його запускають обидва інсталятори) прибирає псевдоніми команди, записи
PATH і ярлики, створені GitPoulpi, а для встановлення з випуску - ще й файли програми; він
відмовляється працювати зі збіркою без маркера `.gitpoulpi-dist` (збірка cargo), а `--keep-files`
лишає файли програми. Агентам MCP він недоступний. Без інсталятора видаліть три виконувані файли.
GitPoulpi також зберігає такі файли:

| Що | Де |
|---|---|
| Налаштування (`config.toml`) і сеанси | Папка конфігурації вашої ОС. `GITPOULPI_CONFIG_DIR` її змінює. |
| Стан окремих репозиторіїв | `.gitpoulpi/` усередині кожного репозиторію. |
| Кеші (аватари) | Папка кешу вашої ОС. |
| Токени й API-ключі | Ваш credential helper. **Налаштування > SSH і облікові дані > Забути всі імена користувачів і паролі** їх видаляє. |

Точні шляхи див. у [Довіднику конфігурації](config-reference.md).
