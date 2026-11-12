[English](../multiplexers.md) · **Nederlands**

# Multiplexers en panes

Een multiplexer staat tussen GitPoulpi en je terminal: hij tekent de cellen zelf opnieuw en stuurt
alleen de escapereeksen door die hij begrijpt. Wat in de terminal alleen werkt
([Een terminal kiezen](terminals.md)), werkt dus misschien niet binnen tmux, zellij, screen of herdr.
Deze pagina zegt wat er doorkomt en welke configuratie je moet toevoegen. De resultaten komen uit dezelfde
testronde van november 2026 als de terminaltabellen: tmux 3.8 (MSYS2: 3.7c), zellij 0.45.1 en
0.46.0, GNU screen 5.0 (macOS levert 4.00.03), herdr 0.9.5, psmux 3.4.0.

GitPoulpi herkent een multiplexer aan `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` of `tmux*`
en `HERDR_*`. Binnen een multiplexer zijn afbeeldingen standaard tekens, behalve waar de tabel hieronder
zegt dat GitPoulpi een werkend pad detecteert. `GITPOULPI_TERMINAL_IMAGE=off` zet alle afbeeldingen
uit, en `ui.image_protocol` forceert een protocol.

## In één oogopslag

| | Afbeeldingen | Aanwijzervorm (OSC 22) | Klembord (OSC 52) | Toetsen |
|---|---|---|---|---|
| **tmux** | sixel als de terminal erachter het aankan (gedetecteerd); kitty-afbeeldingen met `allow-passthrough on` en `ui.image_protocol = "kitty"` | met `allow-passthrough on`, in kitty en foot; niet in Ghostty | met `set-clipboard on` | geen kitty-toetsenbord; `Ctrl+Enter` en `Ctrl+,` komen nooit aan |
| **zellij** | tekens | nee | ja | kitty-toetsenbord werkt |
| **GNU screen** | tekens | nee | nee | klassieke toetsen |
| **herdr** | kitty-afbeeldingen (gedetecteerd met `kitty_graphics = true`) | nee | ja | niet gemeten |
| **psmux** (Windows) | tekens (met opzet) | nee (Windows Terminal heeft er geen) | systeemklembord werkt | niet gemeten |
| **MSYS2-tmux** (Windows) | tekens (met opzet) | nee | systeemklembord werkt | niet gemeten; het scherm is vervormd |
| **WezTerm-panes**, **Windows Terminal-panes**, **kitty- en Ghostty-splits**, **iTerm2-splitpanes** | zoals de terminal zelf | zoals de terminal | zoals de terminal | zoals de terminal |
| **iTerm2 `tmux -CC`** | tekens | nee | nee (iTerm2 weigert het standaard) | niet gemeten |

## tmux

**Afbeeldingen.**

- **Sixel** wordt door tmux zelf getekend, opnieuw gecodeerd voor een client die het ondersteunt. GitPoulpi
  vraagt tmux één keer bij het starten (`tmux display-message -p '#{client_termfeatures}'`,
  begrensd op 400 ms) en gebruikt sixel als de client het vermeldt. Dat is het geval achter foot,
  WezTerm, Rio en Tabby; het is niet het geval achter kitty en Ghostty (tmux drukt dan de tekst
  "SIXEL IMAGE" af). Vereist een tmux die met sixel is gebouwd (die van Arch en Homebrew zijn dat;
  controleer met `tmux -V` en een sixel-test). Geen instelling. Het eigen antwoord van tmux op een
  vraag naar apparaatattributen claimt sixel, wat er ook achter zit, en daarom vraagt GitPoulpi
  in plaats daarvan naar de functies van de client.
- **iTerm2 achter tmux** adverteert sixel maar verft niet-gezette pixels ondoorzichtig, dus de grotendeels
  transparante grafiek zou een effen rechthoek worden. GitPoulpi herkent iTerm2
  (`LC_TERMINAL`, `ITERM_SESSION_ID`, het clienttype) en tekent tekens.
- **kitty-afbeeldingen** gaan door de DCS-passthrough van tmux (kitty, Ghostty; ook gezien in
  iTerm2 en Rio in een ruwe probe). Ze hebben `set -g allow-passthrough on` nodig (tmux 3.3a of
  nieuwer) **en** `ui.image_protocol = "kitty"` in GitPoulpi, dat dan de uploads en
  verwijderingen inpakt. Zonder de tmux-optie wordt de verpakking weggegooid en blijft de grafiek leeg.
- iTerm2-inlineafbeeldingen gaan door tmux, maar tmux kan ze niet positioneren: GitPoulpi gebruikt ze
  daar nooit.

**Aanwijzervorm.** GitPoulpi pakt `OSC 22` in voor de passthrough. Hij bereikt het venster in
kitty en foot met `allow-passthrough on`. Ghostty toont hem niet als hij is ingepakt (de
pijl blijft). WezTerm, Rio, Alacritty, Tabby, iTerm2 hebben geen `OSC 22` om te tonen.

**Klembord.** `set -g set-clipboard on`. GitPoulpi stuurt ook de voor passthrough ingepakte vorm
(die werkt met `allow-passthrough on`). Achter kitty, Ghostty, Alacritty en Tabby komt hij
aan; iTerm2 weigert hem tot zijn klembordrechten aan staan.

**Toetsen.** tmux beantwoordt de kitty-toetsenbordquery niet, en `extended-keys` brengt
`Ctrl+Enter` of `Ctrl+,` niet door: gemeten met tmux 3.8 in kitty, beide komen aan
als gewone `Enter` en `,` met `extended-keys` uit, aan of always. Lettercombinaties zoals
`Ctrl+Shift+Z` komen alleen aan met `extended-keys` aan of always **en**
`extended-keys-format csi-u` (tmux 3.5 of nieuwer) **en** een applicatie die om
uitgebreide toetsen vraagt; GitPoulpi doet dat nooit, dus de instelling verandert hier niets. Gebruik de `Alt`-varianten
binnen tmux
([Sneltoetsen](keybindings.md#toetsen-die-de-terminal-niet-kan-sturen-en-het-kitty-toetsenbordprotocol)).

**Config** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later: pointer shapes, forced kitty pictures
```

**tmux binnen WSL.** Niet gemeten. Het is de Linux-build, dus verwacht hetzelfde als hierboven met
Windows Terminal als buitenste terminal.

## zellij

Afbeeldingen: zellij accepteert het kitty-protocol maar niet de Unicode-plaatshouders, en draagt
geen sixel- of iTerm2-afbeeldingen, dus GitPoulpi tekent tekens (een bewust veto). Aanwijzervorm:
ingeslikt, dus GitPoulpi stuurt hem niet. Klembord: `OSC 52` wordt doorgestuurd (werkt achter
kitty, Ghostty, Alacritty, Tabby; iTerm2 en Terminal.app weigeren of negeren hem). Toetsen: zellij
beantwoordt de kitty-toetsenbordquery, dus het volledige toetsenbord werkt.

zellij 0.46 opent bij een verse start een pop-up "Welcome to Zellij": druk op `Esc`. Een sessie die
is afgesloten, houdt haar naam en de volgende start met dezelfde naam vraagt wat te doen.
Er is geen configuratie nodig.

## GNU screen

Niets komt door: afbeeldingen en `OSC 22` worden als tekens afgedrukt of weggegooid, `OSC 52` wordt
genegeerd, en het toetsenbord is het klassieke. GitPoulpi tekent tekens. screen 4.00.03 (de
versie die macOS levert) vervormt ook kleuren en brede glyphs in de interface; screen 5 is
alleen op Linux getest. Gebruik tmux of zellij voor een bruikbare sessie.

## herdr

herdr bevat een terminal die **kitty-afbeeldingen** opnieuw uitzendt. GitPoulpi gebruikt ze als de
`config.toml` van herdr `kitty_graphics = true` zegt (onder `[terminal]`, of `[experimental]` bij oudere
versies) en de terminal rond herdr ze tekent (kitty, Ghostty, WezTerm): gedetecteerd,
geen instelling. Als de config ergens anders staat (bijvoorbeeld onder `HERDR_HOME`, dat GitPoulpi
niet leest), zet dan `ui.image_protocol = "kitty"`. Via SSH (`herdr --remote`) kan GitPoulpi
de lokale client niet zien, dus tekent het tekens; zie
[Probleemoplossing](troubleshooting.md#de-grafiek-wordt-met-tekens-getekend-niet-als-afbeelding-herdr-tmux-andere-terminals).
`OSC 22` wordt door herdr 0.9.5 niet doorgestuurd, dus GitPoulpi stuurt hem niet. `OSC 52` komt aan
achter kitty, Ghostty, Alacritty en Tabby. In de ruwe probe tekende herdr kitty-afbeeldingen in
kitty, Ghostty en iTerm2, ook met de standaardconfig. Toetsen: niet gemeten.

## psmux en MSYS2-tmux (Windows)

- **psmux** (Rust, ConPTY, tmux-opdrachttaal; `winget install psmux`) zet `TMUX`, dus
  GitPoulpi tekent daar met opzet tekens. Binnen Windows Terminal draait het normaal
  en kopiëren werkt via het systeemklembord; de aanwijzer verandert niet (Windows
  Terminal heeft geen `OSC 22`). Andere hostterminals zijn niet gemeten.
- **tmux uit MSYS2 of Cygwin** zet ook `TMUX` (met opzet tekens), en in de test was het
  scherm vervormd: verouderde frames over elkaar getekend in mintty en in Windows Terminal
  (met en zonder `-u`; zonder werden kadertekens als `qqqq` afgedrukt). Een native Windows-programma
  in een MSYS2-pseudoterminal wordt door tmux niet correct opnieuw getekend. Dat is niet
  op te lossen in GitPoulpi: gebruik psmux, Windows Terminal-panes of WezTerm-panes.
- zellij en screen hebben geen Windows-release. herdr 0.9.5 is geïnstalleerd maar niet gedraaid.

## Terminalpanes

Panes die bij de terminal zelf horen, zijn voor GitPoulpi geen multiplexers:

- **WezTerm-panes** (`wezterm cli split-pane`, `wezterm connect`, SSH-domeinen): WezTerm tekent
  alles, dus afbeeldingen werken (grafiek en avatars in de pane) en de aanwijzer gedraagt zich als in
  WezTerm zelf (vormen werken in een build met upstream-WezTerm-PR #8092; de scheidingslijn van de pane
  geeft de formaataanwijzer). Niet gemeten op Linux of macOS.
- **Windows Terminal-panes** (`wt ; split-pane`): sixel-grafiek en -avatars werken in de pane,
  kopiëren werkt, geen aanwijzervormen.
- **kitty-, Ghostty- en iTerm2-splits**: native, zoals de terminal. Niet gemeten op macOS voor
  kitty en Ghostty (ze hebben een GPU nodig).

## iTerm2 `tmux -CC` (besturingsmodus)

iTerm2 opent een eigen venster voor elk tmux-venster en houdt een gatewayvenster "[tmux bash]" aan.
`TMUX` is gezet, dus GitPoulpi tekent tekens in het eigen venster. Afbeeldingen bereiken iTerm2
wel via de besturingsverbinding (het dialoogvenster "Allow Terminal-Initiated Display?" verscheen
bij een `OSC 1337`-test), maar GitPoulpi gebruikt ze daar niet. De kitty-payload van een
geforceerde `ui.image_protocol = "kitty"` wordt in het gatewayvenster afgedrukt, dus laat hem op
`auto`. `OSC 22` en `OSC 52` komen niet aan.

## Instellingen om te kennen

| Instelling | Gebruik |
|---|---|
| `ui.image_protocol` | `kitty` forceert kitty-afbeeldingen via een multiplexer die ze doorstuurt (tmux met `allow-passthrough on`, herdr met een config die GitPoulpi niet leest); `none` voor tekens. |
| `ui.graph_renderer` | `text` is altijd correct. |
| `ui.kitty_keyboard` | `off` als een multiplexer zich misdraagt met de toetsenbordquery. |

Zie [Een terminal kiezen](terminals.md) voor de terminal achter de multiplexer, en
[Probleemoplossing](troubleshooting.md) als de grafiek met tekens wordt getekend.
