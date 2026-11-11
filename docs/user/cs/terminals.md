[English](../terminals.md) · **Čeština** · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Výběr terminálu

GitPoulpi běží v každém terminálu s podporou myši, ale nejlépe vypadá a funguje v
takovém, který kreslí obrázky, umí true color a posílá celou klávesnici. Tato
stránka řadí terminály podle operačního systému, říká, co která volba nabízí a jaké
nastavení změnit. Údaje byly v říjnu 2026 ověřeny v dokumentaci, changelogu nebo
issue trackeru každého projektu (viz [Zdroje](#zdroje)); nic z toho nebylo vyzkoušeno
v každém terminálu. Vše označené **(neověřeno)** se nepodařilo potvrdit z primárního
zdroje.

## Nejlepší volby

| OS | Nejlepší volby v pořadí | Proč |
|---|---|---|
| **Windows** | **WezTerm**, poté **Windows Terminal** | WezTerm kreslí graf a avatary jako obrázky, včetně průhlednosti. Windows Terminal má nejlepší integraci s OS (je výchozí ve spouštěči), graf i avatary v něm kreslí jako obrázky sixel (bez průhlednosti). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, poté **foot** | kitty a Ghostty dávají obrázky i celou klávesnici. WezTerm dává také obrázky. foot kreslí graf i avatary jako obrázky sixel, je rychlý a má celou klávesnici. V Omarchy zvolte kvůli obrázkům Ghostty nebo kitty. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, poté **iTerm2** | Stejná úvaha. Terminal.app funguje, ale nemá obrázky ani true color. |

V libovolném OS **tmux, zellij a screen** skrývají před GitPoulpi protokol obrázků
terminálu, takže kreslí znaky (viz [Multiplexery](#multiplexery)).

## Co GitPoulpi používá

- **Obrázky** (sloupec grafu, avatary autorů, obrázkový diff a zobrazení Soubor):
  **grafický protokol kitty** s Unicode placeholdery, **vložené obrázky iTerm2**
  (`OSC 1337`), nebo **sixel**, v tomto pořadí preference. Detekce
  je „zakázáno, dokud není prokázáno“, vychází z `TERM`, `TERM_PROGRAM`,
  `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`,
  `KONSOLE_VERSION` a proměnných multiplexerů a klávesnici nikdy nečte (takže se
  terminálu neptá ani na jeho atributy zařízení). kitty a Ghostty používají protokol
  kitty. WezTerm používá protokol iTerm2 (záměrně) a mísí průhlednost PNG. iTerm2, Rio
  a mintty používají protokol iTerm2 s neprůhledným pozadím. Windows Terminal, foot,
  Konsole a mlterm používají **sixel**. Bez žádného z nich jsou avatary odznak s
  iniciálami.
- **Sixel** se kreslí pro **sloupec grafu** (dráhy, uzly a s `ui.avatars = "graph"` i avatary uvnitř kroužků commitů) a jinde pro **avatary** (Log, detail commitu, Historie souboru, Blame, zobrazení obrázku). Sixel nemá alfa kanál, id ani změnu měřítka, proto má obrázek přesně `sloupce x šířka buňky` krát `řádky x výška buňky` pixelů, kreslí se jen tam, kde něco je, přes prázdné buňky se zachovaným pozadím; zakódovaný obrázek se ukládá do mezipaměti a překóduje se jen při změně drah, posunu nebo přiblížení a posílá se znovu jen po překreslení buňky pod ním. Sixel nelze smazat: GitPoulpi ho odstraní zapsáním textu přes jeho buňky. Sixel potřebuje skutečnou velikost buňky v pixelech: Windows Terminal se na ni ptá jednou při startu a po změně přiblížení (`CSI 16 t`, odpověď do ~10 ms, při mlčení nikdy neblokuje); jinde se bere z velikosti okna terminálu nebo z `GITPOULPI_TERMINAL_CELL_PX=10x20`. Terminál, který sixel kreslí, ale nepozná se (terminál VS Code s `terminal.integrated.enableImages`, xterm spuštěný s `-ti vt340`), potřebuje `ui.image_protocol = "sixel"`. Windows Terminal před 1.22 sixel nekreslí; nastavte tam `ui.image_protocol = "none"` (graf pak zůstane znakový).
- **True color**: motivy jsou RGB. `ui.color_depth = "auto"` sníží na 256 barev při
  `TERM` typu `*-256color` bez `COLORTERM=truecolor` (a v Terminal.app) a na 16
  barev v konzoli Linuxu. kitty, WezTerm a Windows Terminal se počítají jako truecolor.
- **Myš**: hlášení myši SGR s tlačítky, tažením a pohybem (zpětná vazba při najetí,
  drag and drop, pravé tlačítko, dvojklik).
- **Klávesnice**: **klávesnicový protokol kitty** (jen příznak disambiguate) se
  vyžádá při startu s limitem 100 ms. Díky němu dorazí `Ctrl+,`, `Ctrl+Enter`,
  `Ctrl+Shift+N` a `Ctrl+Shift+Z`. Bez něj zůstanou starší klávesy (`Ctrl+Y`
  provede znovu, varianty s `Alt` nahrazují některé akordy). Viz
  [Klávesové zkratky](keybindings.md#klávesy-které-terminál-neumí-poslat-a-klávesnicový-protokol-kitty).
- **Schránka**: akce kopírování zapisují `OSC 52`.
- **Tvar ukazatele myši** (ukazatel změny velikosti nad děliči, ruka nad klikatelnými
  položkami): `OSC 22`, bonus tam, kde je implementován.
- **Události fokusu** jsou zapnuty (tooltipy se zavřou, když okno ztratí fokus).
  GitPoulpi si nevyžádá bracketed paste, hypertextové odkazy OSC 8 ani undercurl.
  Jen na cestě s obrázky se každý snímek obalí synchronizovanou aktualizací
  (režim 2026, terminály, které ho neznají, ho ignorují), aby obrázek neblikl
  přes vyskakovací okno.
- **Emoji a symboly**: GitPoulpi používá jen glyfy s výchozí emoji prezentací nebo
  jednoznakové tvary z BMP, protože terminály a písma se v selektorech variant
  neshodují.

## Tabulka schopností

„Obrázky“ je to, co GitPoulpi v daném terminálu dnes kreslí (protokol **kitty** nebo
**iTerm2**, nebo **text**), ne to, co by terminál uměl.

| Terminál | OS | Obrázky (graf, avatary, obrázkový diff) | Truecolor | Klávesnice kitty (celé akordy) | Kopírování OSC 52 | Ukazatel OSC 22 |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | ano | ano | ano | ano |
| **Ghostty** (1.3) | Linux, macOS (bez sestavení pro Windows) | kitty | ano | ano | ano | ano, od 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, s průhledností | ano | jen s `enable_kitty_keyboard = true` (výchozí `false`) | ano (jen zápis) | ne (pull request je stále otevřený) |
| **foot** (1.28) | Linux (Wayland) | sixel (graf i avatary; bez grafiky kitty) | ano | ano, od 1.10.3 | ano (`security.osc52`) | ano, od 1.12 |
| **Alacritty** (0.17) | všechny | text (upstream nemá podporu obrázků) | ano | ano, od 0.13 | ano (zápis; vkládání je ve výchozím stavu vypnuté) | ne (neimplementováno) |
| **Konsole** | Linux | sixel (graf i avatary) | ano | přidává se v roce 2026 (merge request !1188); vydání (neověřeno) | jen zápis, začleněno v polovině 2024 | (neověřeno) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | text | ano | ne (není ve vydaném VTE) | ne (VTE ho ignoruje) | ne |
| **iTerm2** (3.7) | macOS | iTerm2, neprůhledné pozadí | ano | ano, od 3.6.9 (zkontrolujte Profiles > Keys) | ve výchozím stavu vypnuto: zapněte „Applications in terminal may access clipboard“ | (neověřeno) |
| **Terminal.app** | macOS | text | ne (256 barev) | ne | (neověřeno) | (neověřeno) |
| **Windows Terminal** (1.25) | Windows | sixel (graf i avatary) | ano | ano, stabilní od 1.25 (říj 2026); Preview od 1.25 (bře 2026) | ano (zápis; jen při fokusu) | ne (neimplementováno) |
| **Rio** | Windows, Linux, macOS | iTerm2, neprůhledné pozadí (mluví i grafikou kitty) | ano | ano (ve výchozím stavu zapnuto) | ano | ne (neimplementováno) |
| **Warp** | macOS, Linux, Windows | text (nerozpoznán) | ano | ano | ano (sekvenci respektuje) | (neověřeno) |
| **Terminál VS Code** (xterm.js) | všechny | text (sixel a obrázky iTerm2 existují za `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` použije sixel) | ano | (neověřeno) | (neověřeno) | ne |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, neprůhledné pozadí | ano | (neověřeno; nezdokumentováno) | (neověřeno; nezdokumentováno) | ano |
| **Konzole Windows** (conhost, cmd, okno PowerShellu) | Windows | text | jen nedávné Windows 10/11 | není potřeba: modifikátory hlásí sama | ne | ne |
| **ConEmu** | Windows | text | (neověřeno) | (neověřeno) | (neověřeno) | (neověřeno) |
| **tmux, zellij, screen** | všechny | vždy text | potřebuje nastavení | potřebuje nastavení | potřebuje nastavení | ne |

Buňky „ne (neimplementováno)“ u OSC 22 vycházejí z absence v changelogu a
dokumentaci projektu a z hlášení uživatelů, ne z testu; berte je jako
„nezdokumentováno“. Stránka netabuluje šířku emoji a selektorů variant: liší se
podle terminálu a písma, a proto se GitPoulpi glyfům s VS16 vyhýbá (viz
[Co GitPoulpi používá](#co-gitpoulpi-používá)).

## Podle terminálu

### Windows

- **WezTerm.** Nejlepší podpora obrázků ve Windows. Ponechte `ui.image_protocol` na
  `auto`. V `wezterm.lua` nastavte `config.enable_kitty_keyboard = true` pro celé
  akordy a písmo se záložními emoji, například
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Poslední označené stabilní vydání WezTerm je 20240203-110809-5046fc22 (únor 2024);
  novější opravy vycházejí v nightly sestaveních. WezTerm zatím nemá tvar ukazatele OSC 22.
- **Windows Terminal.** Truecolor, myš, OSC 52 a dobré emoji. GitPoulpi v něm
  kreslí graf i avatary jako obrázky sixel (Windows Terminal 1.22 nebo novější).
  Nerd Font zvolte v *Settings > Profiles > Defaults > Appearance > Font face*.
  Text vyberete tak, že při zachycené myši podržíte `Shift`. Tvar ukazatele se
  nemění. V sestavení starším než 1.25 (bez klávesnicového protokolu kitty) použijte
  varianty s `Alt`.
- **Rio** funguje s neprůhledným pozadím obrázků; méně otestováno.
- **mintty** (Git Bash, MSYS2, Cygwin) kreslí obrázky přes protokol iTerm2 s
  neprůhledným pozadím; méně otestováno.
- **Konzole Windows, ConEmu, cmd**: vyhněte se. Obrázky se kreslí špatně nebo vůbec.
  Dvojklik na exe znovu otevře GitPoulpi ve vámi zvoleném terminálu
  (*Nastavení > Spouštěč*).

### Linux

- **kitty.** Vše funguje bez nastavení. `Shift+klik` patří samotnému kitty, takže
  pro rozšíření výběru použijte `Alt+klik` ([Myš](mouse-and-gestures.md)).
- **Ghostty.** Vše funguje bez nastavení. Pokud se ikony zobrazují jako díry, přidejte
  do `font-family` písmo s emoji nebo Nerd Font (klíč zopakujte pro záložní písma).
- **WezTerm.** Obrázky přes protokol iTerm2, průhlednost se mísí. Stejná poznámka
  k `enable_kitty_keyboard` jako ve Windows.
- **foot.** Graf i avatary jako sixel, celá klávesnice, rychlý. Nastavte `font=JetBrainsMono Nerd Font:size=11` v `foot.ini`.
- **Alacritty.** Znakový graf, klávesnice kitty od 0.13. Alacritty nemá podporu obrázků.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Fungují se znaky a staršími klávesami:
  použijte varianty s `Alt` a `Ctrl+Y` pro znovu (nedávná Konsole může přidat
  klávesnicový protokol kitty). Akce kopírování ve VTE (`OSC 52`) nedají do schránky nic.
  Metoda zadávání (IBus, Fcitx) může zachytit přepínač stage na `Ctrl+Space` - **(neověřeno)**;
  přeřaďte ho v *Nastavení > Klávesnice*.
- **Wayland, Hyprland, Omarchy.** Nápověda tvaru ukazatele myši (`OSC 22`) funguje v
  kitty, Ghostty a foot. Položka spouštěče v Omarchy jde přes `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: jako v Linuxu.
- **iTerm2.** Obrázky s neprůhledným pozadím (viditelný obdélník, pokud se vaše
  pozadí liší od motivu; `ui.graph_renderer = "text"` ho odstraní). Pro kopírování
  zapněte *Settings > General > Selection > Applications in terminal may access clipboard*
  a pro akordy s `Alt` *Profiles > Keys > Left Option key: Esc+*.
- **Terminal.app.** Jen 256 barev: GitPoulpi si zvolí `ui.color_depth = 256` samo a
  motivy vypadají plošeji. Žádné obrázky. Pro akordy s `Alt` zaškrtněte *Use Option as Meta key*.

### VS Code a Warp

Terminál VS Code (`TERM_PROGRAM=vscode`) a Warp se nerozpoznávají jako terminály s
obrázky, takže GitPoulpi kreslí znaky. Oba jsou použitelné. Ve VS Code zapněte `terminal.integrated.enableImages` a nastavte
`ui.image_protocol = "sixel"` pro obrázky avatarů. Warp dokumentuje podporu
klávesnicového protokolu kitty; podpora v terminálu VS Code je **(neověřeno)**.

## Multiplexery

GitPoulpi rozpozná tmux, zellij, screen a herdr (`TMUX`, `ZELLIJ`, `STY`, `TERM=screen*`
nebo `tmux*`, `HERDR_*`) a kreslí **znaky**, protože multiplexer předává buňky, ne
escape sekvence obrázků. Pro ostatní funkce v `~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

Chcete-li vynutit obrázky přes multiplexer, který je předává, nastavte
`ui.image_protocol` na `kitty` nebo `iterm2`. herdr s
`kitty_graphics = true` uvnitř kitty, Ghostty nebo WezTerm se rozpozná automaticky;
viz [Řešení potíží](troubleshooting.md#graf-je-vykreslen-znaky-ne-jako-obrázek-herdr-tmux-jiné-terminály).
`GITPOULPI_TERMINAL_IMAGE=off` vypne všechny obrázky.

tmux klávesnicový protokol kitty sám nepředává: `extended-keys` způsobí, že posílá
klávesy s modifikátory jako sekvence `CSI u` nebo `xterm`. zellij 0.41 a novější
klávesnicový protokol kitty mluví a podle svého changelogu předává OSC 52.

## Co se zhorší a jak

| Chybí | Co dostanete místo toho |
|---|---|
| Protokol obrázků | Znakový graf. Avatar se změní na barevný odznak s iniciálami (jedno písmeno) bez ohledu na `ui.avatar_fallback`: identicon vtěsnaný do textových buněk je nerozpoznatelný. Obrázkový diff a zobrazení Soubor použijí barevné půlbloky nebo informační kartu. |
| Truecolor | Barvy se převedou na paletu 256 nebo 16 barev (`ui.color_depth`). |
| Klávesnicový protokol kitty | Starší klávesy; některé akordy dorazí jako jiná klávesa. Nastavení varuje a vy přeřadíte klávesy nebo použijete varianty s `Alt`. |
| OSC 52 | Akce kopírování nedají do schránky nic: vyberte myší (podržte `Shift`) a použijte vlastní kopírování terminálu. |
| OSC 22 | Ukazatel se nemění; dělič nebo tlačítko pod myší se stále rozsvítí. |
| Písmo s emoji | Čtverečky nebo díry místo ikon; graf a text stále fungují. Nainstalujte Nerd Font nebo písmo s emoji a uveďte ho jako záložní. |

## Nastavení, která stojí za to znát

| Nastavení | K čemu |
|---|---|
| `ui.image_protocol` | `auto` (výchozí), `kitty`, `iterm2`, `sixel`, `none`. Nastavte `none`, pokud vidíte změť znaků tam, kde mají být obrázky. Vyžaduje restart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` je vždy správně. Vyžaduje restart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` nebo `off`, pokud se terminál nebo multiplexer chová špatně. Vyžaduje restart. |
| `ui.mouse_enabled` | Vypnuto nechá myš terminálu. Vyžaduje restart. |
| `ui.launch_terminal` | Terminál, který otevírají zástupci OS ([Nastavení](settings.md)). |

Viz také [Obrázky](image-view.md), [Klávesové zkratky](keybindings.md) a
[Řešení potíží](troubleshooting.md).

## Zdroje

Ověřeno v říjnu 2026. Pokud není uvedeno jinak, jde o primární zdroje.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [tvary ukazatele](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [stažení](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sekvence](https://wezterm.org/escape-sequences.html), [pull request pro OSC 22](https://github.com/wezterm/wezterm/pull/6292), [vydání](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [vydání](https://github.com/alacritty/alacritty/releases)
- Konsole: [merge request pro OSC 52](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [merge request pro klávesnici kitty](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [issue k OSC 52](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [issue ke klávesnici kitty](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [stažení](https://iterm2.com/downloads.html); klávesnice kitty z [matice terminfo.dev](https://terminfo.dev/extensions/kitty-keyboard-protocol) (sekundární)
- Windows Terminal: [oznámení 1.22, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (sekundární), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [funkce](https://rioterm.com/docs/features), [klávesnice kitty](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [aplikace na celou obrazovku](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 z [rozboru bezpečnostního upozornění k OSC 52 ve Warpu](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (sekundární)
- VS Code: [pokročilý terminál](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [řídicí sekvence](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tipy](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (bez truecolor): [diskuse Apple](https://discussions.apple.com/thread/255315820) (sekundární)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Srovnávací weby (sekundární, použité jen k ověření): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Stále neověřeno: sixel v Konsole a mlterm (převzato z vlastních seznamů funkcí projektů, neověřeno proti vydání a v žádném terminálu nevyzkoušeno v reálném okně); OSC 22 v Konsole, iTerm2, Terminal.app a Warpu; OSC 52 v Terminal.app,
VS Code a mintty; klávesnicový protokol kitty ve VS Code, mintty a ConEmu; každá buňka
ConEmu; a šířka emoji podle terminálu. Vydání, ve kterém Konsole začne dodávat
klávesnicový protokol kitty, se nepodařilo potvrdit.
