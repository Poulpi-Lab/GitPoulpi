[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · **Nederlands** · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Een terminal kiezen

GitPoulpi draait in elke terminal met muisondersteuning, maar het ziet er het best uit en voelt het best aan in een terminal die afbeeldingen tekent, true color toont en het volledige toetsenbord doorgeeft. Deze pagina rangschikt de terminals per besturingssysteem, zegt wat elke terminal je geeft en welke instelling je moet wijzigen. De feiten zijn in oktober 2026 gecontroleerd aan de hand van de documentatie, de changelog of de issuetracker van elk project (zie [Bronnen](#bronnen)); niets hiervan is op elke terminal getest. Alles wat als **(niet geverifieerd)** staat gemarkeerd, kon niet in een primaire bron worden bevestigd.

## Beste keuzes

| Besturingssysteem | Beste keuzes, in volgorde | Waarom |
|---|---|---|
| **Windows** | **WezTerm**, daarna **Windows Terminal** | WezTerm tekent de grafiek en de avatars als afbeeldingen, met transparantie. Windows Terminal heeft de beste integratie met het besturingssysteem (de standaard van de starter); hij tekent de grafiek en de avatars als sixel-afbeeldingen (zonder transparantie). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, daarna **foot** | kitty en Ghostty geven afbeeldingen en het volledige toetsenbord. WezTerm geeft ook afbeeldingen. foot tekent de grafiek en de avatars als sixel-afbeeldingen, is snel en heeft het volledige toetsenbord. Kies op Omarchy Ghostty of kitty voor afbeeldingen. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, daarna **iTerm2** | Dezelfde redenering. Terminal.app werkt, maar heeft geen afbeeldingen en geen true color. |

Op elk besturingssysteem verbergen **tmux, zellij en screen** het afbeeldingsprotocol van de terminal voor GitPoulpi, dus tekent het tekens (zie [Multiplexers](#multiplexers)).

## Wat GitPoulpi gebruikt

- **Afbeeldingen** (grafiekkolom, avatars van auteurs, afbeeldingsverschil en Bestandsweergave): het **kitty-grafiekprotocol** met Unicode-plaatshouders, **iTerm2-inlineafbeeldingen** (`OSC 1337`) of **sixel**, in die volgorde van voorkeur. De detectie weigert standaard en kijkt naar `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` en de variabelen van multiplexers; het toetsenbord wordt er nooit voor gelezen (de terminal wordt dus ook niet naar zijn apparaatattributen gevraagd). kitty en Ghostty gebruiken het kitty-protocol. WezTerm gebruikt het iTerm2-protocol (met opzet) en mengt de transparantie van de PNG door. iTerm2, Rio en mintty gebruiken het iTerm2-protocol met een ondoorzichtige achtergrond. Windows Terminal, foot, Konsole en mlterm gebruiken **sixel**. Met geen van die protocollen zijn avatars het initialenbadge.
- **Sixel** wordt getekend voor de **grafiekkolom** (lanes, knooppunten en, met `ui.avatars = "graph"`, de avatars in de commitcirkels) en elders voor de **avatars** (Log, commit-details, Bestandsgeschiedenis, Blame, de afbeeldingsweergave). Een sixel heeft geen alfakanaal, geen id's en geen schaling, dus de afbeelding is precies `kolommen x celbreedte` bij `rijen x celhoogte` pixels, alleen getekend waar iets is, over lege cellen die hun achtergrond houden; de gecodeerde afbeelding wordt gecachet, alleen opnieuw gecodeerd als lanes, scrollpositie of zoom veranderen en alleen opnieuw verstuurd als een cel eronder is herschilderd. Een sixel kan niet worden verwijderd: GitPoulpi schrijft tekst over zijn cellen om hem te wissen. Sixel heeft de echte celgrootte in pixels nodig: aan Windows Terminal wordt die één keer gevraagd bij het starten en na een zoom (`CSI 16 t`, antwoord in ongeveer 10 ms, blokkeert nooit als het zwijgt); elders komt hij uit de venstergrootte of uit `GITPOULPI_TERMINAL_CELL_PX=10x20`. Een terminal die sixel tekent maar niet wordt herkend (een VS Code-terminal met `terminal.integrated.enableImages`, een xterm gestart met `-ti vt340`) heeft `ui.image_protocol = "sixel"` nodig. Windows Terminal ouder dan 1.22 tekent geen sixel; zet daar `ui.image_protocol = "none"` (de grafiek blijft dan tekens).
- **True color**: thema's zijn RGB. `ui.color_depth = "auto"` zakt naar 256 kleuren bij een `*-256color`-`TERM` zonder `COLORTERM=truecolor` (en in Terminal.app), en naar 16 kleuren op de Linux-console. kitty, WezTerm en Windows Terminal tellen als true color.
- **Muis**: SGR-muisrapportage met knoppen, slepen en beweging (terugkoppeling onder de aanwijzer, slepen en neerzetten, rechtsklik, dubbelklik).
- **Toetsenbord**: het **kitty-toetsenbordprotocol** (alleen de vlag disambiguate) wordt bij het starten aangevraagd met een time-out van 100 ms. Daardoor komen `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` en `Ctrl+Shift+Z` aan. Zonder blijven de klassieke toetsen (`Ctrl+Y` doet opnieuw, `Alt`-varianten vervangen sommige toetscombinaties). Zie [Sneltoetsen](keybindings.md#toetsen-die-de-terminal-niet-kan-sturen-en-het-kitty-toetsenbordprotocol).
- **Klembord**: kopieeracties schrijven `OSC 52`.
- **Vorm van de muisaanwijzer** (formaataanwijzer boven scheidingslijnen, hand boven klikbare items): `OSC 22`, een extraatje waar het is geïmplementeerd.
- **Focusgebeurtenissen** staan aan (tooltips sluiten als het venster de focus verliest). GitPoulpi vraagt geen bracketed paste, OSC 8-hyperlinks of undercurl aan. Alleen op het afbeeldingspad wordt elk frame in een gesynchroniseerde update gewikkeld (modus 2026, genegeerd door terminals die hem niet kennen), zodat een afbeelding nooit over een pop-up flitst.
- **Emoji en symbolen**: GitPoulpi gebruikt alleen glyphs met standaard emoji-presentatie of BMP-vormen met enkele breedte, omdat terminals en lettertypen het oneens zijn over variatiekiezers.

## Overzicht van mogelijkheden

"Afbeeldingen" is wat GitPoulpi vandaag in die terminal tekent (protocol **kitty**, **iTerm2** of **sixel**, of **tekst**), niet wat de terminal zou kunnen.

| Terminal | Besturingssysteem | Afbeeldingen (grafiek, avatars, afbeeldingsverschil) | True color | Kitty-toetsenbord (volledige toetsen) | OSC 52-kopiëren | OSC 22-aanwijzer |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | ja | ja | ja | ja |
| **Ghostty** (1.3) | Linux, macOS (geen Windows-build) | kitty | ja | ja | ja | ja, sinds 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, met transparantie | ja | alleen met `enable_kitty_keyboard = true` (standaard `false`) | ja (alleen schrijven) | nee (de pull request staat nog open) |
| **foot** (1.28) | Linux (Wayland) | sixel (grafiek en avatars; geen kitty-graphics) | ja | ja, sinds 1.10.3 | ja (`security.osc52`) | ja, sinds 1.12 |
| **Alacritty** (0.17) | alle | tekst (upstream geen afbeeldingsondersteuning) | ja | ja, sinds 0.13 | ja (schrijven; plakken standaard uit) | nee (niet geïmplementeerd) |
| **Konsole** | Linux | sixel (grafiek en avatars) | ja | wordt in 2026 toegevoegd (merge request !1188); release (niet geverifieerd) | alleen schrijven, medio 2024 samengevoegd | (niet geverifieerd) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | tekst | ja | nee (niet in een uitgebrachte VTE) | nee (VTE negeert het) | nee |
| **iTerm2** (3.7) | macOS | iTerm2, ondoorzichtige achtergrond | ja | ja, sinds 3.6.9 (controleer Profiles > Keys) | standaard uit: schakel "Applications in terminal may access clipboard" in | (niet geverifieerd) |
| **Terminal.app** | macOS | tekst | nee (256 kleuren) | nee | (niet geverifieerd) | (niet geverifieerd) |
| **Windows Terminal** (1.25) | Windows | sixel (grafiek en avatars) | ja | ja, stabiel sinds 1.25 (okt 2026); Preview vanaf 1.25 (mrt 2026) | ja (schrijven; alleen met focus) | nee (niet geïmplementeerd) |
| **Rio** | Windows, Linux, macOS | iTerm2, ondoorzichtige achtergrond (spreekt ook kitty-graphics) | ja | ja (standaard aan) | ja | nee (niet geïmplementeerd) |
| **Warp** | macOS, Linux, Windows | tekst (niet gedetecteerd) | ja | ja | ja (honoreert de reeks) | (niet geverifieerd) |
| **VS Code-terminal** (xterm.js) | alle | tekst (sixel en iTerm2-afbeeldingen bestaan achter `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` gebruikt de sixel) | ja | (niet geverifieerd) | (niet geverifieerd) | nee |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, ondoorzichtige achtergrond | ja | (niet geverifieerd; niet gedocumenteerd) | (niet geverifieerd; niet gedocumenteerd) | ja |
| **Windows-console** (conhost, cmd, PowerShell-venster) | Windows | tekst | alleen recente Windows 10/11 | niet nodig: hij meldt modifiers zelf | nee | nee |
| **ConEmu** | Windows | tekst | (niet geverifieerd) | (niet geverifieerd) | (niet geverifieerd) | (niet geverifieerd) |
| **tmux, zellij, screen** | alle | altijd tekst | vraagt instellingen | vraagt instellingen | vraagt instellingen | nee |

De cellen "nee (niet geïmplementeerd)" voor OSC 22 komen voort uit het ontbreken in de changelog en documentatie van een project plus meldingen van gebruikers, niet uit een test; lees ze als "niet gedocumenteerd". De pagina zet de breedte van emoji en variatiekiezers niet in een tabel: die verschilt per terminal en lettertype, en daarom vermijdt GitPoulpi VS16-glyphs (zie [Wat GitPoulpi gebruikt](#wat-gitpoulpi-gebruikt)).

## Per terminal

### Windows

- **WezTerm.** De beste afbeeldingsondersteuning op Windows. Laat `ui.image_protocol` op `auto`. Zet in `wezterm.lua` `config.enable_kitty_keyboard = true` voor de volledige toetsen, en een lettertype met emoji-fallback, bijvoorbeeld `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`. De laatst getagde stabiele release van WezTerm is 20240203-110809-5046fc22 (feb 2024); nieuwere fixes komen in nightlies. WezTerm heeft nog geen OSC 22-aanwijzervorm.
- **Windows Terminal.** True color, muis, OSC 52 en goede emoji. GitPoulpi tekent daar de avatars als sixel-afbeeldingen (Windows Terminal 1.22 of nieuwer); de grafiek bestaat uit tekens. Kies een Nerd Font in *Settings > Profiles > Defaults > Appearance > Font face*. Houd `Shift` ingedrukt om tekst te selecteren terwijl de muis is vastgelegd. De aanwijzervorm verandert niet. Gebruik op een build ouder dan 1.25 (zonder kitty-toetsenbordprotocol) de `Alt`-varianten.
- **Rio** werkt met een ondoorzichtige afbeeldingsachtergrond; minder getest.
- **mintty** (Git Bash, MSYS2, Cygwin) tekent afbeeldingen via het iTerm2-protocol met een ondoorzichtige achtergrond; minder getest.
- **Windows-console, ConEmu, cmd**: vermijden. Afbeeldingen worden slecht of helemaal niet getekend. Dubbelklikken op het exe-bestand opent GitPoulpi opnieuw in de terminal van je keuze (*Instellingen > Algemeen > Starter*).

### Linux

- **kitty.** Alles werkt zonder instelling. `Shift+klik` is van kitty zelf, dus gebruik `Alt+klik` om een selectie uit te breiden ([Muis](mouse-and-gestures.md)).
- **Ghostty.** Alles werkt zonder instelling. Als pictogrammen als gaten verschijnen, voeg dan een emoji- of Nerd Font toe aan `font-family` (herhaal de sleutel om fallbacks toe te voegen).
- **WezTerm.** Afbeeldingen via het iTerm2-protocol, transparantie doorgemengd. Dezelfde opmerking over `enable_kitty_keyboard` als op Windows.
- **foot.** Tekengrafiek, volledig toetsenbord, snel. Avatars zijn sixel-afbeeldingen, de grafiek bestaat uit tekens. Zet `font=JetBrainsMono Nerd Font:size=11` in `foot.ini`.
- **Alacritty.** Tekengrafiek, kitty-toetsenbord sinds 0.13. Alacritty heeft geen afbeeldingsondersteuning.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Werken met tekens en klassieke toetsen: gebruik de `Alt`-varianten en `Ctrl+Y` voor opnieuw (een recente Konsole voegt misschien het kitty-toetsenbordprotocol toe). Kopieeracties via VTE (`OSC 52`) zetten niets op het klembord. Een invoermethode (IBus, Fcitx) kan de `Ctrl+Space`-schakelaar voor stagen overnemen - **(niet geverifieerd)**; wijs hem opnieuw toe in *Instellingen > Toetsenbord & muis*.
- **Wayland, Hyprland, Omarchy.** De muisaanwijzerhint (`OSC 22`) werkt in kitty, Ghostty en foot. Het startermenu-item van Omarchy gaat via `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: zoals op Linux.
- **iTerm2.** Afbeeldingen met een ondoorzichtige achtergrond (een zichtbaar rechthoekje als je achtergrond afwijkt van die van het thema; `ui.graph_renderer = "text"` haalt het weg). Schakel *Settings > General > Selection > Applications in terminal may access clipboard* in voor kopiëren, en *Profiles > Keys > Left Option key: Esc+* voor de `Alt`-toetsen.
- **Terminal.app.** Alleen 256 kleuren: GitPoulpi kiest zelf `ui.color_depth = 256` en thema's zien er vlakker uit. Geen afbeeldingen. Vink *Use Option as Meta key* aan voor `Alt`-toetsen.

### VS Code en Warp

De VS Code-terminal (`TERM_PROGRAM=vscode`) en Warp worden niet als afbeeldingsterminals herkend, dus GitPoulpi tekent tekens. Beide zijn bruikbaar. Zet in VS Code `terminal.integrated.enableImages` aan en `ui.image_protocol = "sixel"` voor avatarafbeeldingen. Warp documenteert ondersteuning voor het kitty-toetsenbordprotocol; die van de VS Code-terminal is **(niet geverifieerd)**.

## Multiplexers

GitPoulpi detecteert tmux, zellij, screen en herdr (`TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` of `tmux*`, `HERDR_*`) en tekent **tekens**, omdat een multiplexer cellen doorstuurt en geen afbeeldingsescapes. Voor de overige functies, in `~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # full chords (tmux 3.2 or later)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later; only if you force pictures, see below
```

Om afbeeldingen af te dwingen via een multiplexer die ze doorstuurt, zet je `ui.image_protocol` op `kitty` of `iterm2`. herdr met `kitty_graphics = true` in kitty, Ghostty of WezTerm wordt automatisch gedetecteerd; zie [Probleemoplossing](troubleshooting.md#de-grafiek-wordt-met-tekens-getekend-niet-als-afbeelding-herdr-tmux-andere-terminals). `GITPOULPI_TERMINAL_IMAGE=off` zet alle afbeeldingen uit.

tmux stuurt het kitty-toetsenbordprotocol zelf niet door: `extended-keys` laat hem gewijzigde toetsen als `CSI u`- of `xterm`-reeksen sturen. zellij 0.41 en later spreekt het kitty-toetsenbordprotocol en stuurt OSC 52 door, volgens zijn changelog.

## Wat wegvalt, en wat er dan komt

| Ontbreekt | Wat je in de plaats krijgt |
|---|---|
| Afbeeldingsprotocol | Tekengrafiek. De avatar wordt de gekleurde initialenbadge (één letter), wat `ui.avatar_fallback` ook zegt: een identicon in tekstcellen geperst is onherkenbaar. Het afbeeldingsverschil en de Bestandsweergave gebruiken gekleurde halve blokken, of een infokaart. |
| True color | Kleuren worden op het palet van 256 of 16 kleuren afgebeeld (`ui.color_depth`). |
| Kitty-toetsenbordprotocol | Klassieke toetsen; sommige toetscombinaties komen als een andere toets aan. Instellingen waarschuwt, en je wijst opnieuw toe of gebruikt de `Alt`-varianten. |
| OSC 52 | Kopieeracties zetten niets op het klembord: selecteer met de muis (houd `Shift` ingedrukt) en gebruik de eigen kopieeropdracht van de terminal. |
| OSC 22 | De aanwijzer verandert niet; de scheidingslijn of knop onder de muis licht nog steeds op. |
| Emoji-lettertype | Vakjes of gaten in plaats van pictogrammen; de grafiek en de tekst werken nog. Installeer een Nerd Font of een emoji-lettertype en vermeld het als fallback. |

## Instellingen om te kennen

| Instelling | Gebruik |
|---|---|
| `ui.image_protocol` | `auto` (standaard), `kitty`, `iterm2`, `sixel`, `none`. Zet `none` als je rommel ziet waar afbeeldingen horen. Herstart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` is altijd correct. Herstart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` of `off`, als een terminal of multiplexer zich misdraagt. Herstart. |
| `ui.mouse_enabled` | Uit laat de muis aan de terminal. Herstart. |
| `ui.launch_terminal` | De terminal die de snelkoppelingen van het besturingssysteem openen ([Instellingen](settings.md)). |

Zie ook [Afbeeldingen](image-view.md), [Sneltoetsen](keybindings.md) en [Probleemoplossing](troubleshooting.md).

## Bronnen

Gecontroleerd in oktober 2026. Primaire bronnen, tenzij anders vermeld.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); kitty-toetsenbord uit de [terminfo.dev-matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secundair)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secundair), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 uit een [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secundair)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (geen true color): [Apple discussion](https://discussions.apple.com/thread/255315820) (secundair)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Vergelijkingssites (secundair, alleen gebruikt om te kruiscontroleren): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Nog steeds niet geverifieerd: sixel in Konsole en mlterm (overgenomen uit de eigen functielijsten van de projecten, niet opnieuw gecontroleerd tegen een release, en in geen enkele terminal op een echt venster geprobeerd); OSC 22 in Konsole, iTerm2, Terminal.app en Warp; OSC 52 in Terminal.app, VS Code en mintty; het kitty-toetsenbordprotocol in VS Code, mintty en ConEmu; elke cel van ConEmu; en de breedte van emoji per terminal. De release waarin Konsole het kitty-toetsenbordprotocol meelevert, is niet bevestigd.
