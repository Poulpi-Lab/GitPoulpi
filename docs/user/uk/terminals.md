[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · **Українська** · [简体中文](../zh-CN/terminals.md)

# Вибір термінала

GitPoulpi працює в будь-якому терміналі з підтримкою миші, але найкраще він
виглядає й відчувається в тому, що малює зображення, показує truecolor і надсилає
клавіатуру повністю. Ця сторінка впорядковує термінали за операційними системами,
каже, що саме дає кожен із них і яке налаштування змінити. Факти перевірено за
документацією, списком змін або трекером помилок кожного проєкту в жовтні 2026
року (див. [Джерела](#джерела)); не все тут перевірено в кожному терміналі.
Усе, що позначено **(не перевірено)**, не вдалося підтвердити за першоджерелом.

## Найкращий вибір

| ОС | Найкращий вибір, за порядком | Чому |
|---|---|---|
| **Windows** | **WezTerm**, потім **Windows Terminal** | WezTerm малює граф і аватари як зображення, з прозорістю. Windows Terminal має найкращу інтеграцію з ОС (це типовий термінал лаунчера); він малює граф і аватари як зображення sixel (без прозорості). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, потім **foot** | kitty і Ghostty дають зображення й повну клавіатуру. WezTerm теж дає зображення. foot малює граф і аватари як зображення sixel, швидкий і має повну клавіатуру. На Omarchy для зображень обирайте Ghostty або kitty. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, потім **iTerm2** | Ті самі міркування. Terminal.app працює, але не має ні зображень, ні truecolor. |

У будь-якій ОС **tmux, zellij і screen** ховають від GitPoulpi протокол зображень
термінала, тож він малює символами (див. [Мультиплексори](#мультиплексори)).

## Що використовує GitPoulpi

- **Зображення** (стовпець графа, аватари авторів, diff зображень і Перегляд
  файлу): **графічний протокол kitty** з Unicode-заповнювачами, **вбудовані
  зображення iTerm2** (`OSC 1337`) або **sixel**, у такому порядку переваги.
  Виявлення працює за принципом «заборонено, доки не доведено протилежне» за
  `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`,
  `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` і змінними мультиплексорів та
  ніколи не читає клавіатуру (тож не питає термінал і про його device attributes).
  kitty і Ghostty використовують протокол kitty. WezTerm використовує протокол
  iTerm2 (навмисно) і змішує прозорість PNG. iTerm2, Rio і mintty використовують
  протокол iTerm2 з непрозорим тлом. Windows Terminal, foot, Konsole і mlterm
  використовують **sixel**. Якщо немає жодного з них, аватари - це значок з
  ініціалами.
- **Sixel** малюється для **стовпця графа** (доріжки, вузли та, з `ui.avatars = "graph"`, аватари всередині кружків commit'ів), а в інших місцях для **аватарів** (Log, відомості про commit, Історія файлу, Blame, подання зображення). У sixel немає альфа-каналу, ідентифікаторів і масштабування, тому зображення має точно `стовпці x ширина клітинки` на `рядки x висота клітинки` пікселів, малюється лише там, де щось є, поверх порожніх клітинок, що зберігають тло; закодоване зображення кешується, перекодовується лише коли змінюються доріжки, прокручування чи масштаб, і надсилається знову лише коли клітинку під ним перемальовано. У sixel немає видалення: GitPoulpi пише текст поверх його клітинок, щоб його прибрати. Sixel потрібен справжній розмір клітинки в пікселях: у Windows Terminal його питають один раз під час запуску та після зміни масштабу (`CSI 16 t`, відповідь приблизно за 10 мс, за мовчання нічого не блокується); в інших місцях він береться з розміру вікна або з `GITPOULPI_TERMINAL_CELL_PX=10x20`. Термінал, що малює sixel, але не розпізнаний (термінал VS Code з `terminal.integrated.enableImages`, xterm, запущений з `-ti vt340`), потребує `ui.image_protocol = "sixel"`. Windows Terminal до 1.22 не малює sixel; задайте там `ui.image_protocol = "none"` (граф лишиться символьним).
- **Truecolor**: теми - це RGB. `ui.color_depth = "auto"` знижується до 256
  кольорів на `*-256color` `TERM` без `COLORTERM=truecolor` (і в Terminal.app) та до
  16 кольорів на консолі Linux. kitty, WezTerm і Windows Terminal вважаються
  truecolor.
- **Миша**: протокол миші SGR з кнопками, перетягуванням і рухом (відгук під час
  наведення, drag and drop, праве клацання, подвійне клацання).
- **Клавіатура**: **протокол клавіатури kitty** (лише прапорець disambiguate)
  запитується під час запуску з тайм-аутом 100 мс. Завдяки йому доходять
  `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` і `Ctrl+Shift+Z`. Без нього лишаються
  застарілі клавіші (`Ctrl+Y` повторює, варіанти з `Alt` замінюють деякі
  комбінації). Див.
  [Скорочення клавіатури](keybindings.md#клавіші-яких-термінал-не-може-надіслати-і-протокол-клавіатури-kitty).
- **Буфер обміну**: дії копіювання пишуть `OSC 52`.
- **Форма вказівника миші** (вказівник зміни розміру над роздільниками, рука над
  елементами, що клацаються): `OSC 22`, бонус там, де його реалізовано.
- **Події фокуса** ввімкнено (підказки закриваються, коли вікно втрачає фокус).
  GitPoulpi не запитує bracketed paste, гіперпосилання OSC 8 чи undercurl.
  Лише на шляху із зображеннями кожен кадр обгорнуто в синхронізоване оновлення
  (режим 2026, його ігнорують термінали, що його не знають), щоб зображення ніколи
  не блимало над спливним вікном.
- **Емодзі та символи**: GitPoulpi використовує лише гліфи зі стандартним
  емодзі-поданням або однокомірчасті фігури BMP, бо термінали й шрифти по-різному
  розуміють селектори варіантів.

## Таблиця можливостей

«Зображення» - це те, що GitPoulpi малює в цьому терміналі сьогодні (протокол
**kitty**, **iTerm2** чи **sixel**, або **текст**), а не те, що міг би термінал.

| Термінал | ОС | Зображення (граф, аватари, diff зображень) | Truecolor | Клавіатура kitty (повні комбінації) | Копіювання OSC 52 | Вказівник OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | так | так | так | так |
| **Ghostty** (1.3) | Linux, macOS (збірки для Windows немає) | kitty | так | так | так | так, починаючи з 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, з прозорістю | так | лише з `enable_kitty_keyboard = true` (типово `false`) | так (лише запис) | ні (pull request ще відкритий) |
| **foot** (1.28) | Linux (Wayland) | sixel (граф і аватари; без графіки kitty) | так | так, починаючи з 1.10.3 | так (`security.osc52`) | так, починаючи з 1.12 |
| **Alacritty** (0.17) | усі | текст (оригінальний проєкт не підтримує зображень) | так | так, починаючи з 0.13 | так (запис; вставлення типово вимкнене) | ні (не реалізовано) |
| **Konsole** | Linux | sixel (граф і аватари) | так | додається в 2026 (merge request !1188); випуск (не перевірено) | лише запис, злито в середині 2024 | (не перевірено) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | текст | так | ні (немає в жодному випущеному VTE) | ні (VTE це ігнорує) | ні |
| **iTerm2** (3.7) | macOS | iTerm2, непрозоре тло | так | так, починаючи з 3.6.9 (перевірте Profiles > Keys) | типово вимкнено: увімкніть «Applications in terminal may access clipboard» | (не перевірено) |
| **Terminal.app** | macOS | текст | ні (256 кольорів) | ні | (не перевірено) | (не перевірено) |
| **Windows Terminal** (1.25) | Windows | sixel (граф і аватари) | так | так, стабільно з 1.25 (жовтень 2026); Preview з 1.25 (березень 2026) | так (запис; лише коли у фокусі) | ні (не реалізовано) |
| **Rio** | Windows, Linux, macOS | iTerm2, непрозоре тло (розуміє й графіку kitty) | так | так (типово ввімкнено) | так | ні (не реалізовано) |
| **Warp** | macOS, Linux, Windows | текст (не виявляється) | так | так | так (шанує цю послідовність) | (не перевірено) |
| **термінал VS Code** (xterm.js) | усі | текст (sixel і зображення iTerm2 є за `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` використовує sixel) | так | (не перевірено) | (не перевірено) | ні |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, непрозоре тло | так | (не перевірено; не задокументовано) | (не перевірено; не задокументовано) | так |
| **Консоль Windows** (conhost, cmd, вікно PowerShell) | Windows | текст | лише нещодавні Windows 10/11 | не потрібна: сама повідомляє модифікатори | ні | ні |
| **ConEmu** | Windows | текст | (не перевірено) | (не перевірено) | (не перевірено) | (не перевірено) |
| **tmux, zellij, screen** | усі | текст, завжди | потрібні налаштування | потрібні налаштування | потрібні налаштування | ні |

Клітинки «ні (не реалізовано)» для OSC 22 походять із відсутності в списках змін і
документації проєкту та зі звітів користувачів, а не з тесту; сприймайте їх як «не
задокументовано». Сторінка не складає таблиці ширини емодзі й селекторів
варіантів: вона різна для кожного термінала й шрифту, тому GitPoulpi уникає гліфів
VS16 (див. [Що використовує GitPoulpi](#що-використовує-gitpoulpi)).

## По терміналах

### Windows

- **WezTerm.** Найкраща підтримка зображень у Windows. Залиште `ui.image_protocol`
  на `auto`. У `wezterm.lua` задайте `config.enable_kitty_keyboard = true` для
  повних комбінацій і шрифт із запасним шрифтом емодзі, наприклад
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Останній позначений стабільний випуск WezTerm - 20240203-110809-5046fc22
  (лютий 2024); новіші виправлення виходять у nightly-збірках. Форми вказівника
  OSC 22 у WezTerm ще немає.
- **Windows Terminal.** Truecolor, миша, OSC 52 і добрі емодзі. GitPoulpi малює
  там граф і аватари як зображення sixel (Windows Terminal 1.22 або новіший). Виберіть Nerd Font у *Settings > Profiles >
  Defaults > Appearance > Font face*. Утримуйте `Shift`, щоб виділяти текст, поки
  мишу захоплено. Форма вказівника не змінюється. У збірці старшій за 1.25 (без
  протоколу клавіатури kitty) користуйтеся варіантами з `Alt`.
- **Rio** працює з непрозорим тлом зображень; протестований менше.
- **mintty** (Git Bash, MSYS2, Cygwin) малює зображення через протокол iTerm2 з
  непрозорим тлом; протестований менше.
- **Консоль Windows, ConEmu, cmd**: уникайте. Зображення малюються погано або
  взагалі не малюються. Подвійне клацання по exe знову відкриває GitPoulpi у
  вибраному вами терміналі (*Налаштування > Лаунчер*).

### Linux

- **kitty.** Усе працює без жодних налаштувань. `Shift+клацання` належить самому
  kitty, тож для розширення виділення користуйтеся `Alt+клацанням`
  ([Миша та жести](mouse-and-gestures.md)).
- **Ghostty.** Усе працює без жодних налаштувань. Якщо замість значків дірки,
  додайте шрифт емодзі або Nerd Font у `font-family` (повторіть ключ, щоб додати
  запасні шрифти).
- **WezTerm.** Зображення через протокол iTerm2, прозорість змішується. Та сама
  примітка про `enable_kitty_keyboard`, що й у Windows.
- **foot.** Граф і аватари - sixel, повна клавіатура, швидко. Задайте
  `font=JetBrainsMono Nerd Font:size=11` у `foot.ini`.
- **Alacritty.** Символьний граф, клавіатура kitty з 0.13. Alacritty не має
  підтримки зображень.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Працюють із символами й застарілими
  клавішами: користуйтеся варіантами з `Alt` і `Ctrl+Y` для повторення (свіжий
  Konsole може додати протокол клавіатури kitty). Дії копіювання VTE (`OSC 52`)
  нічого не кладуть у буфер обміну. Метод введення (IBus, Fcitx) може перехопити
  перемикач stage `Ctrl+Space` - **(не перевірено)**; перепризначте його в
  *Налаштування > Клавіатура й миша*.
- **Wayland, Hyprland, Omarchy.** Підказка вказівника миші (`OSC 22`) працює в
  kitty, Ghostty і foot. Запис лаунчера Omarchy іде через `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: як у Linux.
- **iTerm2.** Зображення з непрозорим тлом (видимий прямокутник, якщо ваше тло
  відрізняється від теми; `ui.graph_renderer = "text"` його прибирає). Увімкніть
  *Settings > General > Selection > Applications in terminal may access clipboard*
  для копіювання та *Profiles > Keys > Left Option key: Esc+* для комбінацій з
  `Alt`.
- **Terminal.app.** Лише 256 кольорів: GitPoulpi сам вибирає `ui.color_depth = 256`,
  а теми виглядають пласкішими. Зображень немає. Позначте *Use Option as Meta key*
  для комбінацій з `Alt`.

### VS Code і Warp

Термінал VS Code (`TERM_PROGRAM=vscode`) і Warp не розпізнаються як термінали із
зображеннями, тож GitPoulpi малює символами. Обидва придатні до роботи. У VS Code увімкніть
`terminal.integrated.enableImages` і задайте `ui.image_protocol = "sixel"`, щоб
отримати аватари-зображення. Warp
документує підтримку протоколу клавіатури kitty; підтримка в терміналі VS Code
**(не перевірено)**.

## Мультиплексори

GitPoulpi виявляє tmux, zellij, screen і herdr (`TMUX`, `ZELLIJ`, `STY`,
`TERM=screen*` або `tmux*`, `HERDR_*`) і малює **символами**, бо мультиплексор
передає клітинки, а не керувальні послідовності зображень. Для решти можливостей
додайте в `~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

Щоб примусово вивести зображення через мультиплексор, що їх передає, задайте
`ui.image_protocol` як `kitty` або `iterm2`. herdr з
`kitty_graphics = true` усередині kitty, Ghostty чи WezTerm виявляється
автоматично; див.
[Усунення несправностей](troubleshooting.md#граф-намальовано-символами-а-не-картинкою-herdr-tmux-інші-термінали).
`GITPOULPI_TERMINAL_IMAGE=off` вимикає всі зображення.

tmux сам не передає протокол клавіатури kitty: `extended-keys` змушує його
надсилати клавіші з модифікаторами як послідовності `CSI u` або `xterm`. zellij
0.41 і новіший розуміє протокол клавіатури kitty і передає OSC 52, згідно з його
списком змін.

## Що деградує і як

| Чого бракує | Що ви отримуєте натомість |
|---|---|
| Протоколу зображень | Символьний граф. Аватар стає кольоровим значком з ініціалами (одна літера), що б не казав `ui.avatar_fallback`: identicon, стиснутий у текстові клітинки, не впізнати. Diff зображень і Перегляд файлу використовують кольорові напівблоки або інформаційну картку. |
| Truecolor | Кольори відображаються на палітру з 256 або 16 кольорів (`ui.color_depth`). |
| Протоколу клавіатури kitty | Застарілі клавіші; деякі комбінації доходять як інша клавіша. Налаштування попереджають, а ви перепризначаєте або користуєтеся варіантами з `Alt`. |
| OSC 52 | Дії копіювання нічого не кладуть у буфер обміну: виділіть мишею (утримуючи `Shift`) і скористайтеся власним копіюванням термінала. |
| OSC 22 | Вказівник не змінюється; роздільник чи кнопка під мишею все одно підсвічується. |
| Шрифту емодзі | Квадратики чи дірки замість значків; граф і текст усе одно працюють. Встановіть Nerd Font або шрифт емодзі й додайте його як запасний. |

## Налаштування, які варто знати

| Налаштування | Для чого |
|---|---|
| `ui.image_protocol` | `auto` (типово), `kitty`, `iterm2`, `sixel`, `none`. Задайте `none`, якщо замість зображень бачите сміття. Потрібен перезапуск. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` завжди правильний. Потрібен перезапуск. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` або `off`, якщо термінал чи мультиплексор поводиться хибно. Потрібен перезапуск. |
| `ui.mouse_enabled` | Вимкнено залишає мишу терміналу. Потрібен перезапуск. |
| `ui.launch_terminal` | Термінал, який відкривають ярлики ОС ([Налаштування](settings.md)). |

Див. також [Зображення](image-view.md), [Скорочення клавіатури](keybindings.md) та
[Усунення несправностей](troubleshooting.md).

## Джерела

Перевірено в жовтні 2026 року. Першоджерела, якщо не зазначено інше.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); клавіатура kitty за [матрицею terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (вторинне джерело)
- Windows Terminal: [оголошення 1.22, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (вторинне джерело), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 за [розбором попередження безпеки про OSC 52 у Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (вторинне джерело)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (без truecolor): [обговорення Apple](https://discussions.apple.com/thread/255315820) (вторинне джерело)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Сайти порівняння (вторинні, лише для перехресної перевірки): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Досі не перевірено: sixel у Konsole і mlterm (узято зі списків можливостей самих проєктів, не перевірено за випуском і не випробувано у справжньому вікні в жодному терміналі); OSC 22 у Konsole, iTerm2, Terminal.app і Warp; OSC 52 у
Terminal.app, VS Code і mintty; протокол клавіатури kitty у VS Code, mintty і
ConEmu; усі клітинки ConEmu; і ширина емодзі в кожному терміналі. Випуск, у якому
Konsole постачає протокол клавіатури kitty, не підтверджено.
