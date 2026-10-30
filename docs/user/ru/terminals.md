[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · **Русский** · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Выбор терминала

GitPoulpi работает в любом терминале с поддержкой мыши, но лучше всего выглядит и
ощущается в том, который рисует изображения, показывает true color и передаёт
клавиатуру целиком. Эта страница ранжирует терминалы по ОС, говорит, что даёт
каждый из них и какую настройку менять. Факты сверены с документацией,
списком изменений или трекером задач каждого проекта в октябре 2026 года (см.
[Источники](#источники)); ничто здесь не проверялось на каждом терминале.
Всё, что помечено **(не проверено)**, не удалось подтвердить по первоисточнику.

## Лучший выбор

| ОС | Лучший выбор по порядку | Почему |
|---|---|---|
| **Windows** | **WezTerm**, затем **Windows Terminal** | WezTerm рисует граф и аватары как изображения, с прозрачностью. У Windows Terminal лучшая интеграция с ОС (это терминал лаунчера по умолчанию), граф и аватары он рисует как изображения sixel (без прозрачности). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, затем **foot** | kitty и Ghostty дают изображения и полную клавиатуру. WezTerm тоже даёт изображения. foot рисует граф и аватары как изображения sixel, быстрый и передаёт всю клавиатуру. В Omarchy для изображений выбирайте Ghostty или kitty. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, затем **iTerm2** | Те же соображения. Terminal.app работает, но не поддерживает ни изображения, ни true color. |

В любой ОС **tmux, zellij и screen** скрывают от GitPoulpi протокол изображений
терминала, поэтому он рисует символами (см. [Мультиплексоры](#мультиплексоры)).

## Что использует GitPoulpi

- **Изображения** (столбец графа, аватары авторов, diff изображений и просмотр
  файла): **протокол графики kitty** с юникодными заполнителями, **встроенные
  изображения iTerm2** (`OSC 1337`) или **sixel**, именно в таком порядке
  предпочтения. Определение работает по принципу «запрещено, пока не
  доказано обратное», по `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`,
  `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` и
  переменным мультиплексоров, и никогда не читает клавиатуру (поэтому не
  запрашивает у терминала и его атрибуты устройства). kitty и Ghostty используют
  протокол kitty. WezTerm использует протокол iTerm2 (намеренно) и подмешивает
  прозрачность PNG. iTerm2, Rio и mintty используют протокол iTerm2 с
  непрозрачной подложкой. Windows Terminal, foot, Konsole и mlterm используют
  **sixel**. Если не подходит ни один, аватары показываются значком с инициалами.
- **Sixel** рисуется для **столбца графа** (полосы, узлы и, при `ui.avatars = "graph"`, аватары внутри кружков commit'ов), а в остальных местах для **аватаров** (Log, сведения о commit'е, История файла, Blame, просмотр изображений). У sixel нет альфа-канала, идентификаторов и масштабирования, поэтому картинка имеет ровно `столбцы x ширина ячейки` на `строки x высота ячейки` пикселей, рисуется только там, где что-то есть, поверх пустых ячеек, сохраняющих фон; закодированная картинка кэшируется, перекодируется только при смене полос, прокрутки или масштаба и отправляется заново, только если ячейка под ней была перерисована. У sixel нет удаления: чтобы убрать картинку, GitPoulpi пишет поверх её ячеек текст. Sixel нужен настоящий размер ячейки в пикселях: у Windows Terminal его спрашивают один раз при запуске и после изменения масштаба (`CSI 16 t`, ответ примерно за 10 мс, при молчании ничего не блокируется); в других местах он берётся из размера окна или из `GITPOULPI_TERMINAL_CELL_PX=10x20`. Терминалу, который рисует sixel, но не распознан (терминал VS Code с `terminal.integrated.enableImages`, xterm, запущенный с `-ti vt340`), нужно `ui.image_protocol = "sixel"`. Windows Terminal до 1.22 sixel не рисует: задайте там `ui.image_protocol = "none"` (граф останется символьным).
- **True color**: темы заданы в RGB. `ui.color_depth = "auto"` переключается на 256
  цветов при `TERM` вида `*-256color` без `COLORTERM=truecolor` (и в Terminal.app) и
  на 16 цветов в консоли Linux. kitty, WezTerm и Windows Terminal считаются
  truecolor.
- **Мышь**: отчёты мыши SGR с кнопками, перетаскиванием и движением (отклик при
  наведении, drag and drop, правый щелчок, двойной щелчок).
- **Клавиатура**: **клавиатурный протокол kitty** (только флаг disambiguate)
  запрашивается при запуске с тайм-аутом 100 мс. Благодаря ему доходят `Ctrl+,`,
  `Ctrl+Enter`, `Ctrl+Shift+N` и `Ctrl+Shift+Z`. Без него остаются прежние
  клавиши (`Ctrl+Y` повторяет, варианты с `Alt` заменяют некоторые сочетания). См.
  [Сочетания клавиш](keybindings.md#клавиши-которые-терминал-не-может-отправить-и-клавиатурный-протокол-kitty).
- **Буфер обмена**: действия копирования пишут `OSC 52`.
- **Форма указателя мыши** (указатель изменения размера над разделителями, рука над
  интерактивными элементами): `OSC 22`, приятное дополнение там, где это реализовано.
- **События фокуса** включены (подсказки закрываются, когда окно теряет фокус).
  GitPoulpi не запрашивает bracketed paste, гиперссылки OSC 8 и волнистое
  подчёркивание. Только на пути с изображениями каждый кадр оборачивается в
  синхронизированное обновление (режим 2026, терминалы, которые его не знают,
  его игнорируют), чтобы изображение не мигало поверх всплывающего окна.
- **Эмодзи и символы**: GitPoulpi использует только глифы с эмодзи-отображением по
  умолчанию или фигуры BMP шириной в одну ячейку, потому что терминалы и шрифты
  расходятся в обращении с селекторами вариантов.

## Таблица возможностей

«Изображения» - это то, что GitPoulpi рисует в этом терминале сегодня (протокол
**kitty**, **iTerm2** или **sixel** либо **текст**), а не то, на что способен терминал.

| Терминал | ОС | Изображения (граф, аватары, diff изображений) | Truecolor | Клавиатура kitty (полные сочетания) | Копирование OSC 52 | Указатель OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | да | да | да | да |
| **Ghostty** (1.3) | Linux, macOS (сборки для Windows нет) | kitty | да | да | да | да, с 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, с прозрачностью | да | только с `enable_kitty_keyboard = true` (по умолчанию `false`) | да (только запись) | нет (pull request всё ещё открыт) |
| **foot** (1.28) | Linux (Wayland) | sixel (граф и аватары; графики kitty нет) | да | да, с 1.10.3 | да (`security.osc52`) | да, с 1.12 |
| **Alacritty** (0.17) | все | текст (в самом проекте изображений нет) | да | да, с 0.13 | да (запись; вставка по умолчанию выключена) | нет (не реализовано) |
| **Konsole** | Linux | sixel (граф и аватары) | да | добавляется в 2026 (merge request !1188); релиз (не проверено) | только запись, влито в середине 2024 | (не проверено) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | текст | да | нет (нет в выпущенном VTE) | нет (VTE игнорирует) | нет |
| **iTerm2** (3.7) | macOS | iTerm2, непрозрачная подложка | да | да, с 3.6.9 (проверьте Profiles > Keys) | по умолчанию выключено: включите «Applications in terminal may access clipboard» | (не проверено) |
| **Terminal.app** | macOS | текст | нет (256 цветов) | нет | (не проверено) | (не проверено) |
| **Windows Terminal** (1.25) | Windows | sixel (граф и аватары) | да | да, стабильно с 1.25 (окт. 2026); Preview с 1.25 (март 2026) | да (запись; только пока окно в фокусе) | нет (не реализовано) |
| **Rio** | Windows, Linux, macOS | iTerm2, непрозрачная подложка (графику kitty он тоже понимает) | да | да (включено по умолчанию) | да | нет (не реализовано) |
| **Warp** | macOS, Linux, Windows | текст (не определяется) | да | да | да (последовательность обрабатывается) | (не проверено) |
| **Терминал VS Code** (xterm.js) | все | текст (sixel и изображения iTerm2 есть за `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` использует sixel) | да | (не проверено) | (не проверено) | нет |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, непрозрачная подложка | да | (не проверено; не описано) | (не проверено; не описано) | да |
| **Консоль Windows** (conhost, cmd, окно PowerShell) | Windows | текст | только свежие Windows 10/11 | не нужна: сама сообщает модификаторы | нет | нет |
| **ConEmu** | Windows | текст | (не проверено) | (не проверено) | (не проверено) | (не проверено) |
| **tmux, zellij, screen** | все | всегда текст | нужны настройки | нужны настройки | нужны настройки | нет |

Ячейки «нет (не реализовано)» для OSC 22 основаны на отсутствии упоминаний в
списке изменений и документации проекта, а также на отчётах пользователей, а не на
проверке; считайте их «не описано». Ширина эмодзи и селекторов вариантов в таблицу
не вынесена: она различается у терминалов и шрифтов, поэтому GitPoulpi избегает
глифов VS16 (см. [Что использует GitPoulpi](#что-использует-gitpoulpi)).

## По терминалам

### Windows

- **WezTerm.** Лучшая поддержка изображений в Windows. Оставьте `ui.image_protocol`
  в значении `auto`. В `wezterm.lua` задайте `config.enable_kitty_keyboard = true`
  для полных сочетаний и шрифт с запасным шрифтом эмодзи, например
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Последний размеченный стабильный релиз WezTerm - 20240203-110809-5046fc22 (февраль
  2024); более новые исправления выходят в nightly-сборках. Формы указателя OSC 22
  в WezTerm пока нет.
- **Windows Terminal.** Truecolor, мышь, OSC 52 и хорошие эмодзи. Граф и аватары
  GitPoulpi рисует в нём изображениями sixel (нужен Windows Terminal 1.22 или
  новее). Выберите Nerd Font в *Settings >
  Profiles > Defaults > Appearance > Font face*. Удерживайте `Shift`, чтобы
  выделять текст, пока мышь захвачена. Форма указателя не меняется. В сборке старше
  1.25 (без клавиатурного протокола kitty) используйте варианты с `Alt`.
- **Rio** работает с непрозрачной подложкой изображений; проверен меньше.
- **mintty** (Git Bash, MSYS2, Cygwin) рисует изображения через протокол iTerm2 с
  непрозрачной подложкой; проверен меньше.
- **Консоль Windows, ConEmu, cmd**: лучше избегать. Изображения рисуются плохо или
  не рисуются совсем. Двойной щелчок по exe снова открывает GitPoulpi в выбранном
  вами терминале (**Настройки > Общие > Лаунчер**).

### Linux

- **kitty.** Всё работает без настроек. `Shift+щелчок` принадлежит самому kitty,
  поэтому для расширения выделения используйте `Alt+щелчок` ([Мышь](mouse-and-gestures.md)).
- **Ghostty.** Всё работает без настроек. Если вместо значков видны дыры, добавьте
  шрифт эмодзи или Nerd Font в `font-family` (повторите ключ, чтобы добавить запасные
  шрифты).
- **WezTerm.** Изображения через протокол iTerm2, прозрачность подмешивается. То же
  замечание про `enable_kitty_keyboard`, что и в Windows.
- **foot.** Граф и аватары - изображения sixel, полная клавиатура, быстрый. Задайте
  `font=JetBrainsMono Nerd Font:size=11` в `foot.ini`.
- **Alacritty.** Символьный граф, клавиатура kitty с 0.13. Поддержки изображений в
  Alacritty нет.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Работают с символами и прежними
  клавишами: используйте варианты с `Alt` и `Ctrl+Y` для повтора (свежий Konsole
  может добавить клавиатурный протокол kitty). Действия копирования VTE (`OSC 52`)
  ничего не кладут в буфер обмена. Метод ввода (IBus, Fcitx) может перехватывать
  переключение «в индекс / из индекса» по `Ctrl+Space` - **(не проверено)**; переназначьте его в
  разделе **Настройки > Клавиатура и мышь**.
- **Wayland, Hyprland, Omarchy.** Подсказка формы указателя (`OSC 22`) работает в
  kitty, Ghostty и foot. Запись лаунчера Omarchy идёт через `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: как в Linux.
- **iTerm2.** Изображения с непрозрачной подложкой (видимый прямоугольник, если ваш
  фон отличается от фона темы; `ui.graph_renderer = "text"` его убирает). Для
  копирования включите *Settings > General > Selection > Applications in terminal
  may access clipboard*, а для сочетаний с `Alt` - *Profiles > Keys > Left Option
  key: Esc+*.
- **Terminal.app.** Только 256 цветов: GitPoulpi сам выбирает `ui.color_depth = 256`,
  и темы выглядят бледнее. Изображений нет. Включите *Use Option as Meta key* для
  сочетаний с `Alt`.

### VS Code и Warp

Терминал VS Code (`TERM_PROGRAM=vscode`) и Warp не распознаются как терминалы с
изображениями, поэтому GitPoulpi рисует символами. Оба пригодны для работы. В VS Code включите
`terminal.integrated.enableImages` и задайте `ui.image_protocol = "sixel"`, чтобы
получить аватары-изображения. Warp документирует поддержку клавиатурного протокола
kitty; поддержка в терминале VS Code **(не проверена)**.

## Мультиплексоры

GitPoulpi определяет tmux, zellij, screen и herdr (`TMUX`, `ZELLIJ`, `STY`,
`TERM=screen*` или `tmux*`, `HERDR_*`) и рисует **символами**, потому что
мультиплексор пересылает ячейки, а не управляющие последовательности изображений.
Для остальных возможностей добавьте в `~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # полные сочетания (tmux 3.2 или новее)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a или новее; только если вы принудительно включаете изображения, см. ниже
```

Чтобы принудительно включить изображения через мультиплексор, который их
пересылает, задайте `ui.image_protocol` равным `kitty` или `iterm2`. herdr с `kitty_graphics = true` внутри kitty, Ghostty или
WezTerm определяется автоматически; см.
[Устранение неполадок](troubleshooting.md#граф-нарисован-символами-а-не-картинкой-herdr-tmux-другие-терминалы).
`GITPOULPI_TERMINAL_IMAGE=off` выключает все изображения.

tmux сам не пересылает клавиатурный протокол kitty: `extended-keys` заставляет его
отправлять клавиши с модификаторами как последовательности `CSI u` или `xterm`.
zellij 0.41 и новее поддерживает клавиатурный протокол kitty и пересылает OSC 52,
согласно его списку изменений.

## Что пропадает и чем заменяется

| Чего нет | Что вы получите вместо этого |
|---|---|
| Протокол изображений | Символьный граф. Аватар становится цветным значком с инициалами (одна буква), что бы ни говорил `ui.avatar_fallback`: identicon, сжатый до символьных ячеек, не узнать. Diff изображений и просмотр файла используют цветные полублоки или информационную карточку. |
| Truecolor | Цвета подбираются из палитры на 256 или 16 цветов (`ui.color_depth`). |
| Клавиатурный протокол kitty | Прежние клавиши; некоторые сочетания приходят как другая клавиша. Настройки предупреждают, а вы переназначаете клавиши или используете варианты с `Alt`. |
| OSC 52 | Действия копирования ничего не кладут в буфер обмена: выделите мышью (удерживая `Shift`) и скопируйте средствами самого терминала. |
| OSC 22 | Указатель не меняется; разделитель или кнопка под мышью по-прежнему подсвечивается. |
| Шрифт эмодзи | Квадратики или дыры вместо значков; граф и текст по-прежнему работают. Установите Nerd Font или шрифт эмодзи и укажите его как запасной. |

## Настройки, которые стоит знать

| Настройка | Назначение |
|---|---|
| `ui.image_protocol` | `auto` (по умолчанию), `kitty`, `iterm2`, `sixel`, `none`. Задайте `none`, если на месте изображений видите мусор. Нужен перезапуск. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` всегда корректен. Нужен перезапуск. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` или `off`, если терминал или мультиплексор ведёт себя неправильно. Нужен перезапуск. |
| `ui.mouse_enabled` | В выключенном состоянии оставляет мышь терминалу. Нужен перезапуск. |
| `ui.launch_terminal` | Терминал, который открывают ярлыки ОС ([Настройки](settings.md)). |

См. также [Изображения](image-view.md), [Сочетания клавиш](keybindings.md) и
[Устранение неполадок](troubleshooting.md).

## Источники

Проверено в октябре 2026 года. Первоисточники, если не указано иное.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); клавиатура kitty по [матрице terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (вторичный источник)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (вторичный источник), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 по [разбору уязвимости OSC 52 в Warp](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (вторичный источник)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (нет truecolor): [обсуждение Apple](https://discussions.apple.com/thread/255315820) (вторичный источник)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Сайты сравнения (вторичные, использовались только для перекрёстной проверки): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Всё ещё не проверено: sixel в Konsole и mlterm (взято из собственных списков
возможностей проектов, не сверено с релизом и не опробовано в реальном окне ни
в одном терминале); OSC 22 в Konsole, iTerm2, Terminal.app и Warp; OSC 52 в
Terminal.app, VS Code и mintty; клавиатурный протокол kitty в VS Code, mintty и
ConEmu; каждая ячейка ConEmu; ширина эмодзи в каждом терминале. Релиз, в котором
Konsole получит клавиатурный протокол kitty, подтвердить не удалось.
