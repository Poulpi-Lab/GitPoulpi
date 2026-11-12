[English](../multiplexers.md) · **Svenska**

# Multiplexers och paneler

En multiplexer sitter mellan GitPoulpi och din terminal: den ritar om cellerna själv och
vidarebefordrar bara de escape-sekvenser den förstår. Det som fungerar i terminalen ensam
([Välja terminal](terminals.md)) fungerar därför kanske inte inuti tmux, zellij, screen eller herdr.
Den här sidan säger vad som släpps igenom och vilken konfiguration du ska lägga till. Resultaten kommer från samma
testomgång i november 2026 som terminaltabellerna: tmux 3.8 (MSYS2: 3.7c), zellij 0.45.1 och
0.46.0, GNU screen 5.0 (macOS levererar 4.00.03), herdr 0.9.5, psmux 3.4.0.

GitPoulpi identifierar en multiplexer utifrån `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` eller `tmux*`
och `HERDR_*`. Inuti en är bilder som standard tecken, utom där tabellen nedan
säger att GitPoulpi upptäcker en fungerande väg. `GITPOULPI_TERMINAL_IMAGE=off` stänger av alla bilder,
och `ui.image_protocol` tvingar fram ett protokoll.

## I korthet

| | Bilder | Pekarform (OSC 22) | Urklipp (OSC 52) | Tangenter |
|---|---|---|---|---|
| **tmux** | sixel när terminalen bakom tar det (upptäcks); kitty-bilder med `allow-passthrough on` och `ui.image_protocol = "kitty"` | med `allow-passthrough on`, i kitty och foot; inte i Ghostty | med `set-clipboard on` | inget kitty-tangentbord; `Ctrl+Enter` och `Ctrl+,` kommer aldrig fram |
| **zellij** | tecken | nej | ja | kitty-tangentbordet fungerar |
| **GNU screen** | tecken | nej | nej | gamla tangenter |
| **herdr** | kitty-bilder (upptäcks med `kitty_graphics = true`) | nej | ja | inte uppmätt |
| **psmux** (Windows) | tecken (med avsikt) | nej (Windows Terminal har ingen) | systemets urklipp fungerar | inte uppmätt |
| **MSYS2 tmux** (Windows) | tecken (med avsikt) | nej | systemets urklipp fungerar | inte uppmätt; skärmen blir förvrängd |
| **WezTerm-paneler**, **Windows Terminal-paneler**, **kitty- och Ghostty-delningar**, **iTerm2:s delade paneler** | som terminalen själv | som terminalen | som terminalen | som terminalen |
| **iTerm2 `tmux -CC`** | tecken | nej | nej (iTerm2 nekar det som standard) | inte uppmätt |

## tmux

**Bilder.**

- **Sixel** ritas av tmux själv, omkodad för en klient som stöder det. GitPoulpi
  frågar tmux en gång vid start (`tmux display-message -p '#{client_termfeatures}'`,
  begränsat till 400 ms) och använder sixel när klienten listar det. Så är fallet bakom foot,
  WezTerm, Rio och Tabby; det är inte fallet bakom kitty och Ghostty (tmux skriver då
  ut texten ”SIXEL IMAGE”). Kräver en tmux byggd med sixel (Archs och Homebrews är det;
  kontrollera med `tmux -V` och ett sixel-test). Ingen inställning. tmux eget svar på en
  fråga om enhetsattribut påstår sixel oavsett vad som sitter bakom, och därför frågar GitPoulpi
  efter klientens funktioner i stället.
- **iTerm2 bakom tmux** annonserar sixel men målar oinställda pixlar ogenomskinliga, så den mestadels
  genomskinliga grafen skulle bli en enfärgad rektangel. GitPoulpi känner igen iTerm2
  (`LC_TERMINAL`, `ITERM_SESSION_ID`, klienttypen) och ritar tecken.
- **kitty-bilder** går igenom tmux DCS-genomsläpp (kitty, Ghostty; sågs också i
  iTerm2 och Rio i ett rått test). De kräver `set -g allow-passthrough on` (tmux 3.3a eller
  nyare) **och** `ui.image_protocol = "kitty"` i GitPoulpi, som då packar in uppladdningarna
  och raderingarna. Utan tmux-alternativet kastas omslaget och grafen förblir tom.
- iTerm2 inline-bilder går igenom tmux men tmux kan inte placera dem: GitPoulpi använder dem aldrig
  där.

**Pekarform.** GitPoulpi packar in `OSC 22` för genomsläppet. Den når fönstret i
kitty och foot med `allow-passthrough on`. Ghostty visar den inte när den är inpackad (pilen
ligger kvar). WezTerm, Rio, Alacritty, Tabby, iTerm2 har ingen `OSC 22` att visa.

**Urklipp.** `set -g set-clipboard on`. GitPoulpi skickar också den genomsläppsinpackade formen
(den fungerar med `allow-passthrough on`). Bakom kitty, Ghostty, Alacritty och Tabby
kommer den fram; iTerm2 nekar den tills dess urklippsbehörighet är på.

**Tangenter.** tmux svarar inte på kitty-tangentbordsfrågan, och `extended-keys` för inte
fram `Ctrl+Enter` eller `Ctrl+,`: uppmätt med tmux 3.8 i kitty, båda kommer fram
som vanliga `Enter` och `,` med `extended-keys` av, på eller always. Bokstavssekvenser som
`Ctrl+Shift+Z` kommer bara fram med `extended-keys` på eller always **och**
`extended-keys-format csi-u` (tmux 3.5 eller nyare) **och** ett program som ber om
utökade tangenter; GitPoulpi gör det aldrig, så inställningen ändrar ingenting här. Använd `Alt`-varianterna
inuti tmux
([Kortkommandon](keybindings.md#tangenter-som-terminalen-inte-kan-skicka-och-kitty-tangentbordsprotokollet)).

**Konfiguration** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a eller senare: pekarformer, påtvingade kitty-bilder
```

**tmux inuti WSL.** Inte uppmätt.

## zellij

Bilder: zellij tar emot kitty-protokollet men inte dess Unicode-platshållare, och bär
ingen sixel eller iTerm2-bilder, så GitPoulpi ritar tecken (ett medvetet veto). Pekarform:
sväljs, så GitPoulpi skickar den inte. Urklipp: `OSC 52` vidarebefordras (fungerar bakom
kitty, Ghostty, Alacritty, Tabby; iTerm2 och Terminal.app nekar eller ignorerar det). Tangenter: zellij
svarar på kitty-tangentbordsfrågan, så hela tangentbordet fungerar.

zellij 0.46 öppnar en popup ”Welcome to Zellij” vid en ny start: tryck `Esc`. En session som
avslutats behåller sitt namn och nästa start med samma namn frågar vad som ska göras.
Ingen konfiguration behövs.

## GNU screen

Ingenting släpps igenom: bilder och `OSC 22` skrivs ut som tecken eller kastas, `OSC 52` ignoreras
och tangentbordet är det gamla. GitPoulpi ritar tecken. screen 4.00.03 (den
version macOS levererar) förvränger också färger och breda tecken i gränssnittet; screen 5 testades
bara på Linux. Använd tmux eller zellij för en användbar session.

## herdr

herdr bäddar in en terminal som skickar ut **kitty-bilder** igen. GitPoulpi använder dem när herdrs
`config.toml` säger `kitty_graphics = true` (under `[terminal]`, eller `[experimental]` i äldre
versioner) och terminalen runt herdr ritar dem (kitty, Ghostty, WezTerm): upptäcks,
ingen inställning. Om konfigurationen ligger någon annanstans (till exempel under `HERDR_HOME`, som GitPoulpi
inte läser) sätt `ui.image_protocol = "kitty"`. Över SSH (`herdr --remote`) kan GitPoulpi
inte se den lokala klienten, så det ritar tecken; se
[Felsökning](troubleshooting.md#grafen-ritas-med-tecken-inte-som-en-bild-herdr-tmux-andra-terminaler).
`OSC 22` vidarebefordras inte av herdr 0.9.5, så GitPoulpi skickar den inte. `OSC 52` kommer fram
bakom kitty, Ghostty, Alacritty och Tabby. I det råa testet ritade herdr kitty-bilder i
kitty, Ghostty och iTerm2, också med standardkonfigurationen. Tangenter: inte uppmätt.

## psmux och MSYS2 tmux (Windows)

- **psmux** (Rust, ConPTY, tmux kommandospråk; `winget install psmux`) sätter `TMUX`, så
  GitPoulpi ritar tecken där med avsikt. Inuti Windows Terminal körs det normalt
  och kopiering fungerar via systemets urklipp; pekaren ändras inte (Windows
  Terminal har ingen `OSC 22`). Andra värdterminaler mättes inte.
- **tmux från MSYS2 eller Cygwin** sätter också `TMUX` (tecken med avsikt), och i testet blev
  skärmen förvrängd: gamla bildrutor ritade över varandra i mintty och i Windows Terminal
  (med och utan `-u`; utan det skrevs ramteckningen ut som `qqqq`). Ett inbyggt Windows-program
  inuti en MSYS2-pseudoterminal ritas inte om korrekt av tmux. Det går inte att åtgärda i GitPoulpi:
  använd psmux, Windows Terminal-paneler eller WezTerm-paneler.
- zellij och screen har ingen Windows-release. herdr 0.9.5 installerades men kördes inte.

## Terminalpaneler

Paneler som hör till själva terminalen är inte multiplexers för GitPoulpi:

- **WezTerm-paneler** (`wezterm cli split-pane`, `wezterm connect`, SSH-domäner): WezTerm ritar
  allt, så bilder fungerar (graf och avatarer i panelen) och pekaren beter sig som i
  WezTerm själv (former fungerar i ett bygge med uppströms WezTerm PR #8092; panelavdelaren
  ger storleksändringspekaren). Inte uppmätt på Linux eller macOS.
- **Windows Terminal-paneler** (`wt ; split-pane`): sixel-graf och avatarer fungerar i panelen,
  kopiering fungerar, inga pekarformer.
- **kitty-, Ghostty- och iTerm2-delningar**: inbyggda, som terminalen. Inte uppmätt på macOS för
  kitty och Ghostty (de kräver en GPU).

## iTerm2 `tmux -CC` (kontrolläge)

iTerm2 öppnar ett eget fönster för varje tmux-fönster och behåller ett ”[tmux bash]”-gatewayfönster.
`TMUX` är satt, så GitPoulpi ritar tecken i det egna fönstret. Bilder når iTerm2
genom kontrollanslutningen (dialogen ”Allow Terminal-Initiated Display?” dök upp
för ett `OSC 1337`-test), men GitPoulpi använder dem inte där. Kitty-nyttolasten från ett
påtvingat `ui.image_protocol = "kitty"` skrivs ut i gatewayfönstret, så lämna det på
`auto`. `OSC 22` och `OSC 52` kommer inte fram.

## Inställningar att känna till

| Inställning | Användning |
|---|---|
| `ui.image_protocol` | `kitty` tvingar fram kitty-bilder genom en multiplexer som vidarebefordrar dem (tmux med `allow-passthrough on`, herdr med en konfiguration GitPoulpi inte läser); `none` ger tecken. |
| `ui.graph_renderer` | `text` är alltid rätt. |
| `ui.kitty_keyboard` | `off` om en multiplexer beter sig fel med tangentbordsfrågan. |

Se [Välja terminal](terminals.md) för terminalen bakom multiplexern, och
[Felsökning](troubleshooting.md) när grafen ritas med tecken.
