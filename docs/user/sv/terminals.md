[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · **Svenska** · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Välja terminal

GitPoulpi fungerar i vilken terminal som helst med musstöd, men ser och känns bäst
i en som ritar bilder, visar true color och skickar hela tangentbordet. Den här
sidan rangordnar terminalerna per operativsystem, säger vad var och en ger dig och
vilken inställning du ska ändra. Uppgifterna kontrollerades mot varje projekts
dokumentation, ändringslogg eller ärendehantering i oktober 2026 (se
[Källor](#källor)); inget av detta har testats i varje terminal.
Allt som är markerat **(overifierat)** kunde inte bekräftas från en primär källa.

## Bästa val

| OS | Bästa val, i ordning | Varför |
|---|---|---|
| **Windows** | **WezTerm**, sedan **Windows Terminal** | WezTerm ritar grafen och avatarerna som bilder, med genomskinlighet. Windows Terminal har den bästa OS-integrationen (startarens standard); den ritar avatarerna som sixel-bilder och grafen som tecken. |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, sedan **foot** | kitty och Ghostty ger bilder och hela tangentbordet. WezTerm ger också bilder. foot ritar avatarerna som sixel-bilder, är snabb och har hela tangentbordet. På Omarchy: välj Ghostty eller kitty för bilder. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, sedan **iTerm2** | Samma resonemang. Terminal.app fungerar men har varken bilder eller true color. |

På alla OS döljer **tmux, zellij och screen** terminalens bildprotokoll för
GitPoulpi, så det ritar tecken (se [Multiplexers](#multiplexers)).

## Vad GitPoulpi använder

- **Bilder** (grafkolumnen, författaravatarer, bilddiff och filvy): **kitty
  graphics protocol** med Unicode-platshållare, eller **iTerm2 inline images**
  (`OSC 1337`) eller **sixel**, i den ordningen. Identifieringen är
  nej-som-standard, utifrån `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`,
  `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION`
  och multiplexerns variabler, och läser aldrig tangentbordet (så den frågar inte
  heller terminalen efter dess enhetsattribut). kitty och Ghostty använder
  kitty-protokollet. WezTerm använder iTerm2-protokollet (med avsikt) och blandar
  in PNG-bildens genomskinlighet. iTerm2, Rio och mintty använder
  iTerm2-protokollet med ogenomskinlig bakgrund. Windows Terminal, foot, Konsole
  och mlterm använder **sixel**. Med inget av dem är avatarerna initialmärket.
- **Sixel** ritas för **grafkolumnen** (lanes, noder och, med `ui.avatars = "graph"`, avatarerna i commit-cirklarna) och annars för **avatarerna** (Log, commit-detaljer, Filhistorik, Blame, bildvyn). En sixel har ingen alfakanal, inga id och ingen skalning, så bilden är exakt `kolumner x cellbredd` gånger `rader x cellhöjd` pixlar, ritad bara där något finns, över tomma celler som behåller sin bakgrund; den kodade bilden cachas, kodas om bara när lanes, rullning eller zoom ändras och skickas om bara när en cell under den har målats om. En sixel går inte att radera: GitPoulpi skriver text över dess celler för att ta bort den. Sixel behöver cellens verkliga storlek i pixlar: Windows Terminal tillfrågas en gång vid start och efter en zoom (`CSI 16 t`, svar på ungefär 10 ms, blockerar aldrig om det förblir tyst); på andra håll kommer den från fönsterstorleken eller från `GITPOULPI_TERMINAL_CELL_PX=10x20`. En terminal som ritar sixel men inte känns igen (en VS Code-terminal med `terminal.integrated.enableImages`, en xterm som startats med `-ti vt340`) behöver `ui.image_protocol = "sixel"`. Windows Terminal före 1.22 ritar inte sixel; sätt `ui.image_protocol = "none"` där (grafen förblir tecken).
- **True color**: teman är RGB. `ui.color_depth = "auto"` går ned till 256 färger
  vid ett `*-256color` `TERM` utan `COLORTERM=truecolor` (och i Terminal.app), och
  till 16 färger i Linux-konsolen. kitty, WezTerm och Windows Terminal räknas som
  true color.
- **Mus**: SGR-musrapportering med knappar, drag och rörelse (hover-respons, dra
  och släpp, högerklick, dubbelklick).
- **Tangentbord**: **kitty keyboard protocol** (bara flaggan disambiguate) begärs
  vid start med 100 ms tidsgräns. Det gör att `Ctrl+,`, `Ctrl+Enter`,
  `Ctrl+Shift+N` och `Ctrl+Shift+Z` kommer fram. Utan det finns de gamla
  tangenterna kvar (`Ctrl+Y` gör om, `Alt`-varianter ersätter vissa sekvenser). Se
  [Kortkommandon](keybindings.md#tangenter-som-terminalen-inte-kan-skicka-och-kitty-tangentbordsprotokollet).
- **Urklipp**: kopieringsåtgärder skriver `OSC 52`.
- **Musmarkörens form** (storleksmarkör över avdelare, hand över klickbara
  objekt): `OSC 22`, en bonus där den är implementerad.
- **Fokushändelser** är på (verktygstips stängs när fönstret tappar fokus).
  GitPoulpi begär inte bracketed paste, OSC 8-hyperlänkar eller undercurl.
  Bara på bildvägen omsluts varje bildruta av en synkroniserad uppdatering
  (läge 2026, ignoreras av terminaler som inte känner till det) så att en bild
  aldrig blinkar över en popup.
- **Emoji och symboler**: GitPoulpi använder bara tecken med emoji som
  standardpresentation eller enkelbreda BMP-former, eftersom terminaler och
  typsnitt är oeniga om variationsväljare.

## Förmågetabell

”Bilder” är vad GitPoulpi ritar i den terminalen idag (protokollet **kitty**,
**iTerm2** eller **sixel**, eller **text**), inte vad terminalen skulle kunna göra.

| Terminal | OS | Bilder (graf, avatarer, bilddiff) | Truecolor | Kitty-tangentbord (hela tangentsekvenser) | OSC 52-kopiering | OSC 22-pekare |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | ja | ja | ja | ja |
| **Ghostty** (1.3) | Linux, macOS (ingen Windows-version) | kitty | ja | ja | ja | ja, sedan 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, med genomskinlighet | ja | bara med `enable_kitty_keyboard = true` (standard `false`) | ja (bara skriva) | nej (pull requesten är fortfarande öppen) |
| **foot** (1.28) | Linux (Wayland) | sixel (graf och avatarer; ingen kitty-grafik) | ja | ja, sedan 1.10.3 | ja (`security.osc52`) | ja, sedan 1.12 |
| **Alacritty** (0.17) | alla | text (inget bildstöd uppströms) | ja | ja, sedan 0.13 | ja (skriva; klistra in av som standard) | nej (inte implementerat) |
| **Konsole** | Linux | sixel (graf och avatarer) | ja | läggs till under 2026 (merge request !1188); release (overifierat) | bara skriva, infört mitten av 2024 | (overifierat) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | text | ja | nej (inte i en utgiven VTE) | nej (VTE ignorerar det) | nej |
| **iTerm2** (3.7) | macOS | iTerm2, ogenomskinlig bakgrund | ja | ja, sedan 3.6.9 (kontrollera Profiles > Keys) | av som standard: slå på ”Applications in terminal may access clipboard” | (overifierat) |
| **Terminal.app** | macOS | text | nej (256 färger) | nej | (overifierat) | (overifierat) |
| **Windows Terminal** (1.25) | Windows | sixel (graf och avatarer) | ja | ja, stabilt sedan 1.25 (okt 2026); Preview från 1.25 (mars 2026) | ja (skriva; bara medan fönstret har fokus) | nej (inte implementerat) |
| **Rio** | Windows, Linux, macOS | iTerm2, ogenomskinlig bakgrund (talar också kitty-grafik) | ja | ja (på som standard) | ja | nej (inte implementerat) |
| **Warp** | macOS, Linux, Windows | text (identifieras inte) | ja | ja | ja (respekterar sekvensen) | (overifierat) |
| **VS Code-terminalen** (xterm.js) | alla | text (sixel och iTerm2-bilder finns bakom `terminal.integrated.enableImages`, `ui.image_protocol = "sixel"` använder sixel) | ja | (overifierat) | (overifierat) | nej |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, ogenomskinlig bakgrund | ja | (overifierat; inte dokumenterat) | (overifierat; inte dokumenterat) | ja |
| **Windows-konsolen** (conhost, cmd, PowerShell-fönster) | Windows | text | bara nyare Windows 10/11 | behövs inte: den rapporterar modifierare själv | nej | nej |
| **ConEmu** | Windows | text | (overifierat) | (overifierat) | (overifierat) | (overifierat) |
| **tmux, zellij, screen** | alla | text, alltid | kräver inställningar | kräver inställningar | kräver inställningar | nej |

Cellerna ”nej (inte implementerat)” för OSC 22 bygger på att det saknas i ett
projekts ändringslogg och dokumentation samt användarrapporter, inte på ett test;
läs dem som ”inte dokumenterat”. Sidan tabellerar inte bredden på emoji och
variationsväljare: den skiljer sig per terminal och typsnitt, och därför undviker
GitPoulpi VS16-tecken (se [Vad GitPoulpi använder](#vad-gitpoulpi-använder)).

## Per terminal

### Windows

- **WezTerm.** Det bästa bildstödet på Windows. Låt `ui.image_protocol` stå på
  `auto`. Sätt `config.enable_kitty_keyboard = true` i `wezterm.lua` för hela
  tangentsekvenserna, och ett typsnitt med emoji-reserv, till exempel
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  WezTerms senast taggade stabila release är 20240203-110809-5046fc22 (feb 2024);
  nyare rättningar kommer i nightlies. WezTerm har ännu ingen OSC 22-pekarform.
- **Windows Terminal.** Truecolor, mus, OSC 52 och bra emoji. GitPoulpi ritar
  avatarerna som sixel-bilder (Windows Terminal 1.22 eller senare); grafen är
  tecken. Välj ett Nerd Font i *Settings > Profiles > Defaults > Appearance >
  Font face*. Håll `Shift` för att markera text medan musen är fångad. Pekarens
  form ändras inte. På en version äldre än 1.25 (inget kitty keyboard protocol)
  använder du `Alt`-varianterna.
- **Rio** fungerar med ogenomskinlig bildbakgrund; mindre testat.
- **mintty** (Git Bash, MSYS2, Cygwin) ritar bilder via iTerm2-protokollet med
  ogenomskinlig bakgrund; mindre testat.
- **Windows-konsolen, ConEmu, cmd**: undvik. Bilder ritas dåligt eller inte alls.
  Ett dubbelklick på exe-filen öppnar GitPoulpi igen i den terminal du har valt
  (*Inställningar > Allmänt > Startare*).

### Linux

- **kitty.** Allt fungerar utan någon inställning. `Shift+klick` är kittys egen,
  så använd `Alt+klick` för att utöka en markering ([Mus](mouse-and-gestures.md)).
- **Ghostty.** Allt fungerar utan någon inställning. Om ikoner visas som hål, lägg
  till ett emoji- eller Nerd Font i `font-family` (upprepa nyckeln för att lägga
  till reserver).
- **WezTerm.** Bilder via iTerm2-protokollet, genomskinlighet inblandad. Samma
  anmärkning om `enable_kitty_keyboard` som på Windows.
- **foot.** Teckengraf, hela tangentbordet, snabb. Avatarerna är sixel-bilder,
  grafen är tecken. Sätt `font=JetBrainsMono Nerd Font:size=11` i
  `foot.ini`.
- **Alacritty.** Teckengraf, kitty-tangentbord sedan 0.13. Alacritty har inget
  bildstöd.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Fungerar med tecken och gamla
  tangenter: använd `Alt`-varianterna och `Ctrl+Y` för att göra om (en ny Konsole
  kan lägga till kitty keyboard protocol). VTE:s kopieringsåtgärder (`OSC 52`)
  lägger ingenting i urklipp. En inmatningsmetod (IBus, Fcitx) kan ta
  `Ctrl+Space` för stage-växlingen - **(overifierat)**; bind om den i
  *Inställningar > Tangentbord*.
- **Wayland, Hyprland, Omarchy.** Musmarkörens antydan (`OSC 22`) fungerar i
  kitty, Ghostty och foot. Omarchys startarpost går via `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: som på Linux.
- **iTerm2.** Bilder med ogenomskinlig bakgrund (en synlig rektangel om din
  bakgrund skiljer sig från temats; `ui.graph_renderer = "text"` tar bort den).
  Slå på *Settings > General > Selection > Applications in terminal may access
  clipboard* för kopiering, och *Profiles > Keys > Left Option key: Esc+* för
  `Alt`-sekvenserna.
- **Terminal.app.** Bara 256 färger: GitPoulpi väljer `ui.color_depth = 256` själv
  och teman ser plattare ut. Inga bilder. Bocka i *Use Option as Meta key* för
  `Alt`-sekvenser.

### VS Code och Warp

VS Code-terminalen (`TERM_PROGRAM=vscode`) och Warp känns inte igen som
bildterminaler, så GitPoulpi ritar tecken. Båda går att använda. I VS Code: slå på `terminal.integrated.enableImages` och sätt
`ui.image_protocol = "sixel"` för avatarbilder. Warp
dokumenterar stöd för kitty keyboard protocol; VS Code-terminalens stöd är
**(overifierat)**.

## Multiplexers

GitPoulpi identifierar tmux, zellij, screen och herdr (`TMUX`, `ZELLIJ`, `STY`,
`TERM=screen*` eller `tmux*`, `HERDR_*`) och ritar **tecken**, eftersom en
multiplexer vidarebefordrar celler, inte bildsekvenser. För de andra funktionerna,
i `~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g extended-keys on                # hela tangentsekvenser (tmux 3.2 eller senare)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a eller senare; bara om du tvingar fram bilder, se nedan
```

För att tvinga fram bilder genom en multiplexer som vidarebefordrar dem, sätt
`ui.image_protocol` till `kitty` eller `iterm2`.
herdr med `kitty_graphics = true` i kitty, Ghostty eller WezTerm identifieras
automatiskt; se
[Felsökning](troubleshooting.md#grafen-ritas-med-tecken-inte-som-en-bild-herdr-tmux-andra-terminaler).
`GITPOULPI_TERMINAL_IMAGE=off` stänger av alla bilder.

tmux vidarebefordrar inte kitty keyboard protocol självt: `extended-keys` får det
att skicka modifierade tangenter som `CSI u`- eller `xterm`-sekvenser. zellij 0.41
och senare talar kitty keyboard protocol och vidarebefordrar OSC 52, enligt dess
ändringslogg.

## Vad som försämras, och hur

| Saknas | Vad du får i stället |
|---|---|
| Bildprotokoll | Teckengraf. Avataren blir det färgade initialmärket (en bokstav) oavsett vad `ui.avatar_fallback` säger: en identicon pressad in i teckenceller går inte att känna igen. Bilddiffen och filvyn använder färgade halvblock, eller ett infokort. |
| Truecolor | Färgerna mappas till 256- eller 16-färgspaletten (`ui.color_depth`). |
| Kitty keyboard protocol | Gamla tangenter; vissa sekvenser kommer fram som en annan tangent. Inställningar varnar, och du binder om eller använder `Alt`-varianterna. |
| OSC 52 | Kopieringsåtgärder lägger ingenting i urklipp: markera med musen (håll `Shift`) och använd terminalens egen kopiering. |
| OSC 22 | Pekaren ändras inte; avdelaren eller knappen under musen lyser ändå upp. |
| Emoji-typsnitt | Rutor eller hål i stället för ikoner; grafen och texten fungerar ändå. Installera ett Nerd Font eller ett emoji-typsnitt och ange det som reserv. |

## Inställningar att känna till

| Inställning | Användning |
|---|---|
| `ui.image_protocol` | `auto` (standard), `kitty`, `iterm2`, `sixel`, `none`. Sätt `none` om du ser skräp där bilder ska vara. Kräver omstart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` är alltid rätt. Kräver omstart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` eller `off`, om en terminal eller multiplexer beter sig fel. Kräver omstart. |
| `ui.mouse_enabled` | Av lämnar musen åt terminalen. Kräver omstart. |
| `ui.launch_terminal` | Terminalen som OS-genvägarna öppnar ([Inställningar](settings.md)). |

Se även [Bilder](image-view.md), [Kortkommandon](keybindings.md) och
[Felsökning](troubleshooting.md).

## Källor

Kontrollerade i oktober 2026. Primära källor om inget annat anges.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); kitty keyboard från [terminfo.dev matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) (sekundär)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (sekundär), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 från en [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (sekundär)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (ingen truecolor): [Apple discussion](https://discussions.apple.com/thread/255315820) (sekundär)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Jämförelsesajter (sekundära, används bara för korskontroll): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Fortfarande overifierat: sixel i Konsole och mlterm (hämtat från projektens egna funktionslistor, inte återkontrollerat mot en release, och inte prövat i ett riktigt fönster i någon terminal); OSC 22 i Konsole, iTerm2, Terminal.app och Warp; OSC 52 i
Terminal.app, VS Code och mintty; kitty keyboard protocol i VS Code, mintty och
ConEmu; varje cell för ConEmu; och emojibredd per terminal. Releasen där Konsole
levererar kitty keyboard protocol bekräftades inte.
