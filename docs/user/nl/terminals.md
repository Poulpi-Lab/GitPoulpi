[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · **Nederlands** · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · [Türkçe](../tr/terminals.md) · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Een terminal kiezen

GitPoulpi draait in elke terminal met muisondersteuning, maar het ziet er het best uit en voelt het best aan in
een terminal die afbeeldingen tekent, true color toont en het volledige toetsenbord doorgeeft. Deze pagina
zegt wat elke terminal je op elk besturingssysteem geeft en welke instelling je moet wijzigen.

**Hoe je haar leest.** De tabellen komen uit een testronde in **november 2026**: elke terminal is aangestuurd
met ruwe escapereeksen en, waar hij GitPoulpi kon draaien, met GitPoulpi zelf, op Linux, Windows 11 en macOS 15.
De versies staan [hieronder](#geteste-versies). Elke cel is een van deze:

- **werkt**;
- **gedeeltelijk** (en wat ontbreekt);
- **werkt niet** (en waarom: de *terminal*, de *OS-laag* of *GitPoulpi*);
- **n.v.t.** (de terminal bestaat niet op dat besturingssysteem);
- **niet gemeten** (en waarom).

Multiplexers (tmux, zellij, screen, herdr, psmux, panes) hebben hun eigen pagina:
[Multiplexers](multiplexers.md).

## Beste keuzes

| Besturingssysteem | Beste keuzes, in volgorde | Waarom |
|---|---|---|
| **Windows** | **Windows Terminal**, daarna **WezTerm** en **Rio** | Windows Terminal tekent de grafiek en de avatars als sixel-afbeeldingen, rechts verankerd, en is de standaard van de starter. WezTerm en Rio tekenen ze als iTerm2-afbeeldingen, rechts verankerd sinds de ConPTY-fix van november 2026 (zie [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty en Ghostty geven afbeeldingen, de aanwijzervormen en het volledige toetsenbord. foot tekent sixel-afbeeldingen en heeft het volledige toetsenbord en de aanwijzervormen. |
| **macOS** | **iTerm2**, daarna **kitty** en **Ghostty** | iTerm2 3.7.7 tekent afbeeldingen via het kitty-protocol. Terminal.app werkt, maar heeft geen afbeeldingen en geen true color. kitty en Ghostty gedragen zich als op Linux (niet gemeten op macOS). |

Op elk besturingssysteem verbergen **screen** en **zellij** het afbeeldingsprotocol van de terminal voor
GitPoulpi, dus tekent het tekens; tmux laat sixel door. Zie [Multiplexers](multiplexers.md).

## Wat GitPoulpi gebruikt

- **Afbeeldingen** (grafiekkolom, avatars van auteurs, afbeeldingsverschil en Bestandsweergave): het
  **kitty-grafiekprotocol** met Unicode-plaatshouders, **iTerm2-inlineafbeeldingen**
  (`OSC 1337`) of **sixel**. De detectie weigert standaard en kijkt naar `TERM`,
  `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION` en de variabelen van multiplexers. Als de
  omgeving niemand noemt (een xterm gestart met `-ti vt340`, een VS Code-terminal
  met afbeeldingen aan, elke terminal die via SSH bereikt wordt, waar `TERM_PROGRAM` niet
  wordt doorgegeven), beslissen de antwoorden van de terminal zelf: GitPoulpi vraagt één keer bij het starten
  (zie "De opstartquery" hieronder) en gebruikt de XTVERSION-naam van de terminal (kitty,
  Ghostty, WezTerm, iTerm2, mintty, foot) of sixel in zijn apparaatattributen.
  Binnen een multiplexer worden de antwoorden genegeerd, omdat tmux sixel claimt, wat er
  ook achter zit.
- **Welk protocol waar.** kitty, Ghostty en iTerm2 3.7.7 of nieuwer: het kitty-protocol.
  WezTerm en Rio: het iTerm2-protocol, met doormenging van de transparantie van de PNG. mintty en iTerm2
  ouder dan 3.7.7: het iTerm2-protocol met een ondoorzichtige achtergrond. Windows Terminal, foot,
  Tabby, Konsole en mlterm: **sixel**. Al het andere: tekens, en de avatar wordt het
  initialenbadge. `ui.image_protocol` forceert er een.
- **iTerm2 3.7.7.** Elke `OSC 1337`-inlineafbeelding opent een modaal dialoogvenster "Allow
  Terminal-Initiated Display?" dat het venster blokkeert tot je "Remember my choice" aanvinkt. Dezelfde
  afbeeldingen via het kitty-protocol, plaatshouders inbegrepen, worden zonder vraag getekend, dus GitPoulpi gebruikt
  daar het kitty-protocol. `ui.image_protocol = "iterm2"` geeft nog steeds de inlineafbeeldingen.
- **Sixel** wordt getekend voor de **grafiekkolom** (lanes, knooppunten en, met
  `ui.avatars = "graph"`, de avatars in de commitcirkels) en elders voor de
  **avatars** (Log, commit-details, Bestandsgeschiedenis, Blame, de afbeeldingsweergave). Een
  sixel heeft geen alfakanaal, geen id's en geen schaling, dus de afbeelding is precies `kolommen x
  celbreedte` bij `rijen x celhoogte` pixels, alleen getekend waar iets is, over
  lege cellen die hun achtergrond houden; hij wordt alleen opnieuw gecodeerd als de lanes, de
  scrollpositie of de zoom veranderen. Een sixel kan niet worden verwijderd: GitPoulpi schrijft tekst
  over zijn cellen om hem te wissen. Sixel heeft de echte celgrootte in pixels nodig: Windows
  Terminal, Tabby en mintty beantwoorden `CSI 16 t` één keer bij het starten en na een zoom
  (blokkeert nooit als ze zwijgen); elders komt hij uit de venstergrootte van de terminal,
  of uit `GITPOULPI_TERMINAL_CELL_PX=10x20`. Een terminal die sixel tekent zonder het te zeggen,
  heeft `ui.image_protocol = "sixel"` nodig. Windows Terminal
  ouder dan 1.22 tekent geen sixel; zet daar `ui.image_protocol = "none"`.
- **True color**: thema's zijn RGB. `ui.color_depth = "auto"` zakt naar 256 kleuren bij een
  `*-256color`-`TERM` zonder `COLORTERM=truecolor` (en in Terminal.app, dat
  geen true color heeft), en naar 16 kleuren op de Linux-console.
- **Muis**: SGR-muisrapportage met knoppen, slepen en beweging (terugkoppeling onder de aanwijzer, slepen
  en neerzetten, rechtsklik, dubbelklik).
- **De opstartquery.** Vier vragen gaan in één schrijfactie de deur uit: de vlaggen van het kitty-toetsenbord
  (`CSI ? u`), de achtergrondkleur (`OSC 11`, alleen gebruikt voor `ui.theme = system`), de
  naam van de terminal (XTVERSION, `CSI > 0 q`) en zijn apparaatattributen (`CSI c`, als laatste: elke
  terminal beantwoordt hem, dus hij sluit het lezen af). Het lezen is één `poll`-lus met een deadline
  van 100 ms, afgerond voordat de toetsenbordlezer start. Een terminal die niets antwoordt
  kost eenmalig die 100 ms en alles blijft zoals de omgeving zegt. Alleen Unix: op
  Windows meldt de console zelf de toetsen en beslist de omgeving. De antwoorden staan in het
  omgevingsrapport van het activiteitenlogboek (regel `start-up query:`).
- **Toetsenbord**: het **kitty-toetsenbordprotocol** (alleen de vlag disambiguate) wordt bij het
  starten aangevraagd. Daardoor komen `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` en `Ctrl+Shift+Z` aan.
  Zonder blijven de klassieke toetsen (`Ctrl+Y` doet opnieuw, de `Alt`-varianten vervangen sommige
  toetscombinaties). Zie
  [Sneltoetsen](keybindings.md#toetsen-die-de-terminal-niet-kan-sturen-en-het-kitty-toetsenbordprotocol).
- **Klembord**: elke kopieeractie volgt één beleid. Lokaal het **systeemklembord**
  van de machine waarop GitPoulpi draait, plus `OSC 52` naar de terminal als de terminal
  bekend staat als iemand die hem honoreert (kitty, WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio,
  Tabby, iTerm2); `OSC 52` is ook de terugval als het systeemklembord faalt. **Via
  SSH, of zonder beeldscherm** (een Linux-console, een container) wordt alleen `OSC 52` verstuurd, en
  binnen tmux wordt hij zowel ingepakt voor de passthrough als kaal verstuurd. De statusbalk zegt
  "gekopieerd" alleen als het systeemklembord de tekst nam; wat naar de terminal ging zegt
  "naar het klembord van de terminal gestuurd (OSC 52, niet bevestigd)", omdat niets bevestigt
  dat de terminal gehoorzaamde. Het omgevingsrapport van het activiteitenlogboek noemt de route in zijn
  regel `clipboard:`.
- **Vorm van de muisaanwijzer** (formaataanwijzer boven scheidingslijnen, hand boven klikbare items, I-balk
  boven tekstvelden): `OSC 22`, een extraatje waar de terminal het implementeert. GitPoulpi
  stuurt hem altijd (behalve op de Linux-console en achter screen, zellij en herdr, die hem inslikken
  of afdrukken; achter tmux wordt hij ingepakt voor de passthrough). Onder `TERM_PROGRAM=mintty`
  stuurt het de eigen cursornamen van mintty (`hand`, `ibeam`, `sizewe`, ...), de enige die
  mintty leest. **Rio implementeert `OSC 22` op geen enkel besturingssysteem**: de broncode parseert de
  reeks en gooit hem weg, dus geen instelling aan welke kant dan ook verandert dat.
- **Tekstcursor**: met `ui.text_cursor = "terminal"` (de standaard) wordt de eigen cursor van de terminal
  op de invoegpositie van het actieve tekstveld gezet, als knipperend streepje (`DECSCUSR`), en elders
  verborgen; je eigen cursorstijl komt terug bij het afsluiten. `drawn` tekent in plaats daarvan de eigen invoegpositie van GitPoulpi.
- **Focusgebeurtenissen** staan aan (tooltips sluiten als het venster de focus verliest).
  **Bracketed paste** wordt bij het starten aangevraagd en bij elke uitgang losgelaten (Unix), zodat een
  plakactie van meerdere regels als één plakactie aankomt: de regeleinden dienen nooit een veld in en de
  letters activeren nooit sneltoetsen. Op Windows heeft de console geen plakhaken; zie
  [Windows](#windows) voor wat GitPoulpi in de plaats doet.
  GitPoulpi vraagt geen OSC 8-hyperlinks of undercurl aan.
  Alleen op het afbeeldingspad wordt elk frame in een gesynchroniseerde update gewikkeld
  (modus 2026, genegeerd door terminals die hem niet kennen), zodat een afbeelding nooit
  over een pop-up flitst.
- **Emoji en symbolen**: GitPoulpi gebruikt alleen glyphs met standaard emoji-presentatie
  of BMP-vormen met enkele breedte, omdat terminals en lettertypen het oneens zijn over variatiekiezers.

## De matrix

Rijen zijn terminals, kolommen zijn het besturingssysteem. "GitPoulpi" in een cel betekent dat GitPoulpi zelf
daar is gedraaid; "probe" betekent dat alleen ruwe escapereeksen zijn gedraaid. **n.v.t.**-cellen: kitty heeft geen
native Windows-build (de releases bevatten Linux en macOS); Ghostty heeft geen Windows-build
(alleen de bibliotheek libghostty is platformonafhankelijk); foot is een Wayland-terminal (alleen Linux);
xterm is een X11-client (Linux, en macOS via XQuartz); iTerm2 en Terminal.app zijn alleen voor macOS;
Windows Terminal, conhost en mintty zijn alleen voor Windows (mintty ook onder Cygwin en
MSYS2).

**Niet gemeten op macOS: kitty, Ghostty, WezTerm, Rio en Warp.** Ze hebben een GPU nodig
(Metal of OpenGL 3.3). Hun Linux-resultaten zijn de referentie voor wat hetzelfde programma doet;
draai ze opnieuw op een Mac met GPU voordat je op een macOS-cel vertrouwt.

### Afbeeldingen: welk protocol GitPoulpi kiest, en of het zichtbaar is

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **werkt**: kitty-protocol (GitPoulpi) | n.v.t. | niet gemeten (heeft een GPU nodig) |
| **Ghostty** | **werkt**: kitty-protocol (GitPoulpi) | n.v.t. | niet gemeten (heeft een GPU nodig) |
| **WezTerm** | **werkt**: iTerm2-protocol met transparantie (GitPoulpi). Plaatshouders worden samengedrukt getekend, dus het kitty-protocol wordt niet gebruikt | **werkt**: iTerm2-protocol met transparantie, grafiek en avatars op hun rijen (officiële 20240203 en de build van upstream-PR #8092). Officiële 20240203 zette de afbeeldingen op de verkeerde rijen voordat GitPoulpi de cursor eromheen toonde, zie "De ConPTY-beperking". | niet gemeten (heeft een GPU nodig) |
| **foot** | **werkt**: sixel met transparantie (GitPoulpi) | n.v.t. | n.v.t. |
| **Rio** | **werkt**: iTerm2-protocol met transparantie (GitPoulpi) | **werkt**: iTerm2-protocol met transparantie, celnauwkeurig bij elke geprobeerde lettergrootte (0.5.28; GitPoulpi). Sixel en kitty komen nooit aan (ConPTY gooit ze weg), dus er valt niets te kiezen | niet gemeten (heeft een GPU nodig) |
| **Alacritty** | tekst: **werkt** (de terminal heeft geen afbeeldingsondersteuning) | tekst: **gedeeltelijk** (geen afbeeldingen; afbeeldingsverschil als blokken) | tekst: **werkt** |
| **Tabby** | **werkt**: sixel met transparantie (probe; GitPoulpi niet gedraaid in Tabby op Linux) | **werkt**: sixel, uitgelijnd (GitPoulpi) | **werkt**: sixel, transparantie behouden (GitPoulpi) |
| **Warp** | niet gemeten (hij heeft XWayland nodig en decodeert synthetische toetsaanslagen verkeerd, dus er kon niets worden getypt) | **werkt niet** (terminal: geen sixel-, iTerm2- of kitty-afbeeldingen); GitPoulpi niet gemeten (kon niet vanuit het invoervak van Warp worden gestart) | niet gemeten (heeft een GPU nodig) |
| **xterm** | probe: sixel **werkt** met `-ti vt340`; GitPoulpi kiest het uit de apparaatattributen (de opstartquery kwam na de testronde), of zet `ui.image_protocol = "sixel"` | n.v.t. | tekst: **werkt**; sixel tekent (probe) en heeft `ui.image_protocol = "sixel"` nodig |
| **iTerm2** | n.v.t. | n.v.t. | **werkt**: kitty-protocol vanaf 3.7.7, geen dialoogvenster (GitPoulpi). Oudere versies krijgen de inlineafbeeldingen met een ondoorzichtige achtergrond. Sixel tekent maar verft niet-gezette pixels ondoorzichtig, dus `ui.image_protocol = "sixel"` geeft een effen rechthoek |
| **Terminal.app** | n.v.t. | n.v.t. | tekst: **werkt** (geen afbeeldingen) |
| **Windows Terminal** | n.v.t. | **werkt**: sixel (GitPoulpi, 1.25) | n.v.t. |
| **conhost** | n.v.t. | tekst: **gedeeltelijk** (tekengrafiek, afbeeldingsverschil met halve blokken) | n.v.t. |
| **mintty** | n.v.t. | **werkt niet** (terminal achter ConPTY: er wordt helemaal geen afbeelding getekend, zelfs geen gewone `OSC 1337` met zichtbare cursor geplaatst; de grafiekkolom blijft leeg; tekst en palet zijn in orde) | n.v.t. |

### Vorm van de muisaanwijzer (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **werkt** | n.v.t. | niet gemeten (heeft een GPU nodig) |
| **Ghostty** | **werkt** | n.v.t. | niet gemeten (heeft een GPU nodig) |
| **WezTerm** | **werkt niet** (terminal: niet geïmplementeerd in de geteste build) | officieel: **werkt niet** (terminal). Build met upstream-WezTerm-PR #8092, die aanwijzervormen toevoegt: **werkt** (hand, I-balk, formaatpijlen) | niet gemeten (heeft een GPU nodig) |
| **foot** | **werkt** | n.v.t. | n.v.t. |
| **Rio** | **werkt niet** (terminal: geparseerd en weggegooid) | **werkt niet** (terminal, dezelfde reden; niet ConPTY) | **werkt niet** (terminal, dezelfde broncode; alleen op Linux en Windows gedraaid) |
| **Alacritty** | **werkt niet** (terminal) | **werkt niet** (terminal) | **werkt niet** (terminal) |
| **Tabby** | **werkt niet** (terminal: xterm.js) | **werkt niet** (terminal) | **werkt niet** (terminal) |
| **Warp** | niet gemeten | **werkt niet** (terminal) | niet gemeten |
| **xterm** | **werkt niet** (terminal: CSS-namen onbekend) | n.v.t. | niet gemeten (de aanwijzer stond buiten het venster) |
| **iTerm2** | n.v.t. | n.v.t. | **werkt niet** (terminal) |
| **Terminal.app** | n.v.t. | n.v.t. | **werkt niet** (terminal) |
| **Windows Terminal** | n.v.t. | **werkt niet** (terminal) | n.v.t. |
| **conhost** | n.v.t. | **werkt niet** (terminal) | n.v.t. |
| **mintty** | n.v.t. | **werkt** met de eigen namen van mintty; GitPoulpi stuurt ze | n.v.t. |

Waar de cel "werkt niet" zegt, licht de scheidingslijn of knop onder de muis nog steeds op.

### Toetsenbordprotocol

GitPoulpi vraagt op Linux en macOS om het kitty-toetsenbordprotocol. Op Windows vraagt het dat niet:
de console meldt modifiers zelf, en de toetscombinaties hieronder worden beperkt door wat de
terminal voor zichzelf houdt.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **werkt** | n.v.t. | niet gemeten (heeft een GPU nodig) |
| **Ghostty** | **werkt** | n.v.t. | niet gemeten (heeft een GPU nodig) |
| **WezTerm** | **gedeeltelijk**: geen antwoord tot `enable_kitty_keyboard = true` in `wezterm.lua` (standaard `false`) | **gedeeltelijk**: WezTerm houdt `Ctrl+Shift+N` (opent zijn venster); `Ctrl+,` wordt niet afgeleverd (officiële build). De PR #8092-build houdt geen van beide, maar `Ctrl+,` opent nog steeds geen Instellingen (geen kitty-toetsenbordantwoord via ConPTY) | niet gemeten (heeft een GPU nodig) |
| **foot** | **werkt** | n.v.t. | n.v.t. |
| **Rio** | **werkt** (standaard aan) | **gedeeltelijk**: Rio houdt `Ctrl+Shift+N` (opent zijn venster) | niet gemeten (heeft een GPU nodig) |
| **Alacritty** | **werkt** (0.13 en later) | **gedeeltelijk**: `Ctrl+Shift+N` komt aan als `Ctrl+N`, `Ctrl+,` wordt niet afgeleverd | **werkt** |
| **Tabby** | **werkt niet** (terminal: geen antwoord; hij houdt `Ctrl+Shift+N` en `Ctrl+,` voor zichzelf) | **gedeeltelijk**: Tabby houdt `Ctrl+Shift+N` en `Ctrl+,` | **werkt niet** (terminal: geen antwoord); sommige toetscombinaties kwamen niet aan |
| **Warp** | niet gemeten | niet gemeten | niet gemeten (heeft een GPU nodig) |
| **xterm** | **werkt niet** (terminal) | n.v.t. | **werkt niet** (terminal) |
| **iTerm2** | n.v.t. | n.v.t. | **werkt** (3.7.7; controleer *Profiles > Keys* als het niet werkt) |
| **Terminal.app** | n.v.t. | n.v.t. | **werkt niet** (terminal: geen antwoord; klassieke toetsen) |
| **Windows Terminal** | n.v.t. | **gedeeltelijk**: WT houdt `Ctrl+Shift+N` (nieuw venster) en `Ctrl+,` (zijn instellingen) | n.v.t. |
| **conhost** | n.v.t. | niet gemeten | n.v.t. |
| **mintty** | n.v.t. | **gedeeltelijk**: geen van beide combinaties had zichtbaar effect | n.v.t. |

Waar een toetscombinatie niet aankomt, gebruik je de `Alt`-varianten
([Sneltoetsen](keybindings.md#toetsen-die-de-terminal-niet-kan-sturen-en-het-kitty-toetsenbordprotocol)),
of wijs je de combinatie opnieuw toe in *Instellingen > Toetsenbord & muis*.

### Klembord (kopieeracties)

Een kopieeractie gebruikt altijd het systeemklembord als er een lokaal beeldscherm is, dus deze cellen
beschrijven het `OSC 52`-pad van de terminal, dat telt via SSH, in containers en als
terugval. Elke GitPoulpi-kopieertest die het systeemklembord gebruikte, werkte.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **werkt** | n.v.t. | niet gemeten (heeft een GPU nodig) |
| **Ghostty** | **werkt** (gezien achter tmux, zellij, herdr); kaal: niet gemeten (een klembordschrijfactie heeft toetsenbordfocus nodig, die de test niet gaf) | n.v.t. | niet gemeten (heeft een GPU nodig) |
| **WezTerm** | **werkt** | **werkt** | niet gemeten (heeft een GPU nodig) |
| **foot** | niet gemeten (een schrijfactie heeft toetsenbordfocus nodig, die de test niet gaf) | n.v.t. | n.v.t. |
| **Rio** | niet gemeten (dezelfde reden) | **werkt** | niet gemeten (heeft een GPU nodig) |
| **Alacritty** | niet gemeten (dezelfde reden) | **werkt niet** (klembord ongewijzigd met de standaardconfig; niet onderzocht) | **werkt** |
| **Tabby** | **werkt** | **werkt** | **werkt** |
| **Warp** | niet gemeten | **werkt niet** (klembord ongewijzigd) | niet gemeten (heeft een GPU nodig) |
| **xterm** | niet gemeten | n.v.t. | **werkt niet** (klembord ongewijzigd) |
| **iTerm2** | n.v.t. | n.v.t. | **werkt niet** tot *Settings > General > Selection > Applications in terminal may access clipboard* aan staat |
| **Terminal.app** | n.v.t. | n.v.t. | **werkt niet** (terminal) |
| **Windows Terminal** | n.v.t. | **werkt** (met focus) | n.v.t. |
| **conhost** | n.v.t. | niet gemeten (geen probe gedraaid) | n.v.t. |
| **mintty** | n.v.t. | **werkt niet** (klembord ongewijzigd met zijn standaardinstellingen) | n.v.t. |

### Plakken

Op Linux en macOS vraagt GitPoulpi om bracketed paste, dus een plakactie van meerdere regels is één plakactie.
Dat is niet per terminal gedraaid (**niet gemeten** per terminal). Op Windows heeft de console
geen plakhaken, dus een terminal die niet eerst vraagt, levert de tekst als toetsaanslagen af
en elk regeleinde zou een `Enter` zijn. GitPoulpi behandelt daarom toetsaanslagen
die binnen 30 ms op elkaar volgen (een `Enter` ertussen, of acht of meer tekens) als één
plakactie. Ingehouden toetsen en "abc, `Enter`" met de hand getypt zijn geen plakacties.

| Terminal | Windows (plakactie van twee regels in de prompt voor een nieuwe branch) |
|---|---|
| **Windows Terminal** | **werkt**: WT vraagt eerst "meerdere regels plakken?" (zijn eigen waarschuwing), daarna wordt er geen branch gemaakt |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **werkt** (geen branch gemaakt) |
| **Warp** | niet gemeten (GitPoulpi kon er niet in worden gestart) |
| **conhost** | niet gemeten (`Ctrl+V` en `Shift+Insert` door de test geplakt in de klassieke console deden niets) |

### Breedte van emoji en pictogrammen

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **werkt** (brede cellen uitgelijnd; kitty tekent het tandwiel U+2699 leeg, een bekende lettertypekwestie) | WezTerm, Rio, Tabby, Windows Terminal: **werkt**. Alacritty: **gedeeltelijk** (emoji als monochrome omtrekken) | Alacritty, Tabby: **werkt**. Andere: niet gemeten (heeft een GPU nodig) |
| **Tabby** | **werkt** | **werkt** | **werkt** |
| **Warp** | niet gemeten | **werkt** | niet gemeten |
| **xterm** | **gedeeltelijk** (vakjes: het lettertype) | n.v.t. | **gedeeltelijk** (vakjes: het lettertype) |
| **iTerm2**, **Terminal.app** | n.v.t. | n.v.t. | **werkt** |
| **conhost** | n.v.t. | **gedeeltelijk**: kleuren-emoji getekend, brede cellen uitgelijnd | n.v.t. |
| **mintty** | n.v.t. | **werkt** | n.v.t. |

### Geteste versies

kitty 0.49.2 (macOS 0.50.0 startte niet), Ghostty 1.4.0, WezTerm 20260716 op Linux en
20240203 (de laatste getagde release) op Windows, foot 1.28.0, Rio 0.5.28 (Linux, Windows) en
0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) en 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. Terminals veranderen snel: een nieuwere versie dan deze kan afwijken.

## Per terminal

### Windows

- **Windows Terminal.** De standaard van de starter en de beste afbeeldingsondersteuning op Windows:
  sixel-grafiek en -avatars, OSC 52, goede emoji, en hij beantwoordt de vraag naar de celgrootte. Kies
  een Nerd Font in *Settings > Profiles > Defaults > Appearance > Font face*. Houd `Shift` ingedrukt om
  tekst te selecteren terwijl de muis is vastgelegd. Hij houdt `Ctrl+Shift+N` en `Ctrl+,`: gebruik de
  `Alt`-varianten of wijs opnieuw toe. De aanwijzervorm verandert niet.
- **WezTerm.** Zet in `wezterm.lua` `config.enable_kitty_keyboard = true` (builds voor Linux en
  macOS) en een lettertype met emoji-fallback, bijvoorbeeld
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Afbeeldingen gebruiken het iTerm2-protocol en niets hoeft te worden ingesteld: de laatste getagde release,
  20240203-110809 (feb 2024), verankert ze op de juiste rijen (opnieuw gecontroleerd in november 2026 met
  de huidige GitPoulpi; vóór de hieronder beschreven ConPTY-fix deed hij dat niet). Upstream-WezTerm-PR #8092 voegt aanwijzervormen toe.
- **Rio** (MSI, vraagt UAC). Afbeeldingen (grafiek, avatars, afbeeldingsverschil) gebruiken het iTerm2-protocol
  met transparantie en landen precies op hun cellen: gemeten op 0.5.28 met zijn standaardlettertype
  en op 18 en 22,5 px (cellen van 8x16, 11x21 en 13x26 pixels, altijd hele pixels), een afbeelding van 20x10 cellen en
  een van 4x27 cellen kwamen beide celnauwkeurig uit. Niets hoeft te worden ingesteld. De pseudoconsole
  slikt de vraag naar de celgrootte in, dus GitPoulpi gaat ervan uit dat de cellen van Rio twee keer zo hoog zijn als breed
  (waar bij alle drie de groottes); stel bij een lettertype met een andere verhouding
  `GITPOULPI_TERMINAL_CELL_PX=WxH` in (bijvoorbeeld `10x24`) voor ronde commitmarkeringen. Rio
  heeft op geen enkel besturingssysteem aanwijzervormen.
- **Tabby.** Sixel-grafiek en -avatars, kopiëren en plakken werken. Tabby houdt `Ctrl+Shift+N` en
  `Ctrl+,` voor zichzelf.
- **Alacritty.** Heeft OpenGL 3.3 nodig. Tekengrafiek; kopiëren via `OSC 52` stond niet aan met de standaardconfig.
- **mintty** (Git Bash, MSYS2, Cygwin). Aanwijzervormen werken (de eigen namen van mintty). Afbeeldingen
  worden niet getoond (de grafiekkolom blijft leeg; mintty 3.8.3 tekent achter
  ConPTY geen `OSC 1337`-afbeelding, ook niet als een gewone probe hem plaatst) en hij beantwoordt daar geen vraag naar de celgrootte: zet
  `ui.graph_renderer = "text"`.
- **Warp.** Niet gemeten met GitPoulpi: het kon niet vanuit het invoervak van Warp worden gestart. De
  terminal heeft geen sixel-, iTerm2- of kitty-afbeeldingen.
- **conhost, cmd, een PowerShell-venster.** Tekengrafiek, afbeeldingsverschil met halve blokken. Standaard
  heropent GitPoulpi, gestart vanuit zo'n venster, zichzelf in de terminal van je keuze; zie
  "De herstartregel".

**De ConPTY-beperking.** Windows draait het programma van elke terminal achter een pseudoconsole
(ConPTY). Builds van Windows Terminal en WezTerm die een recente bundelen, laten de afbeeldingsreeksen
door; die in Windows zelf (gebruikt door Rio en mintty), en die gebundeld
met WezTerm 20240203, niet. Wat verloren gaat: de DCS-reeksen (sixel) en APC-reeksen (kitty-graphics)
worden weggegooid, dus alleen de iTerm2-afbeeldingen (`OSC 1337`) komen aan, en daarom gebruiken Rio en
WezTerm die; `CSI 16 t` (celgrootte) wordt alleen in Windows Terminal en Tabby beantwoord.
Wat is opgelost: een `OSC 1337` steekt ConPTY meteen over en landt op de cursor die ConPTY het laatst
heeft geschilderd, en ConPTY schildert de cursor alleen terwijl hij zichtbaar is. GitPoulpi hield de cursor
verborgen, dus zijn afbeeldingen landden op verouderde rijen (de grafiek over de voettekst, een gescheurd scherm na
het scrollen). Op Windows toont het nu de cursor rond elke afbeelding die het schrijft, binnen dezelfde
schrijfactie, en verbergt hem weer; Rio en WezTerm 20240203 verankeren elke afbeelding op zijn cel
(`pty_conpty_picture_anchor` leest de cursorverplaatsing achter ConPTY terug). Buiten Windows of voor sixel
verandert er niets.

**Plakken op Windows.** Zie de tabel "Plakken" hierboven: GitPoulpi vouwt een salvo toetsaanslagen
samen tot één plakactie, dus een plakactie van meerdere regels dient in de geteste terminals nooit een prompt in.

**De herstartregel.** Een dubbelklik op het exe-bestand start het in een eigen klassiek consolevenster
(conhost). Als `ui.launch_terminal` een andere terminal noemt (standaard: Windows
Terminal), heropent GitPoulpi zichzelf daar en sluit het eerste venster
(`ui.relaunch_in_terminal`). Het doet dat **alleen** als het alleen is in een klassiek consolevenster.
Een terminal die het exe-bestand rechtstreeks draait (`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty
`-e`, Tabby `run`) is waar je wilt zijn, en er wordt niets heropend; vanuit een shell is het
niet alleen, dus er gebeurt ook niets. Zet het uit met `ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Alles werkt zonder instelling. `Shift+klik` is van kitty zelf, dus gebruik
  `Alt+klik` om een selectie uit te breiden ([Muis](mouse-and-gestures.md)).
- **Ghostty.** Alles werkt zonder instelling. Als pictogrammen als gaten verschijnen, voeg dan een emoji-
  of Nerd Font toe aan `font-family` (herhaal de sleutel om fallbacks toe te voegen).
- **WezTerm.** Afbeeldingen via het iTerm2-protocol, transparantie doorgemengd. Zet
  `enable_kitty_keyboard = true` voor alle toetscombinaties. Geen aanwijzervormen.
- **foot.** Sixel-grafiek en -avatars, aanwijzervormen, volledig toetsenbord, snel. Zet
  `font=JetBrainsMono Nerd Font:size=11` in `foot.ini`. Kopiëren via `OSC 52` wordt bepaald door
  `security.osc52`.
- **Rio.** Afbeeldingen via het iTerm2-protocol met transparantie, volledig toetsenbord. Geen
  aanwijzervormen (Rio gooit `OSC 22` weg).
- **Alacritty.** Tekengrafiek, kitty-toetsenbord sinds 0.13. Geen afbeeldingsondersteuning.
- **Tabby.** Sixel-afbeeldingen; geen kitty-toetsenbord, geen aanwijzervormen (Electron, xterm.js). Hij
  houdt `Ctrl+Shift+N` en `Ctrl+,`.
- **xterm.** Start hem als `xterm -u8 -ti vt340` voor sixel (UTF-8 is nodig voor de kadertekens).
  Vakjes in plaats van emoji: het lettertype.
- **Warp.** Niet gemeten: hij heeft XWayland nodig en decodeerde in de test synthetische toetsaanslagen verkeerd.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Geen onderdeel van de ronde van november 2026. Werken
  met tekens en klassieke toetsen: gebruik de `Alt`-varianten en `Ctrl+Y` voor opnieuw. VTE
  negeert `OSC 52`, dus kopiëren via SSH komt niet op het klembord. Een invoermethode
  (IBus, Fcitx) kan de `Ctrl+Space`-schakelaar voor stagen overnemen; wijs hem opnieuw toe in *Instellingen >
  Toetsenbord & muis*. Konsole wordt voor sixel herkend uit zijn eigen documentatie.
- **Wayland, Hyprland, Omarchy.** De aanwijzerhint (`OSC 22`) werkt in kitty, Ghostty en
  foot. Het startermenu-item van Omarchy gaat via `xdg-terminal-exec`.

### macOS

- **iTerm2.** 3.7.7 en nieuwer tekenen afbeeldingen via het kitty-protocol, zonder vraag.
  Instellingen die helpen:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    nodig voor kopiëren via `OSC 52` (anders toont iTerm2 "the terminal attempted to
    access the clipboard but it was denied").
  - *Profiles > Keys > Left Option key: Esc+*: nodig voor de `Alt`-combinaties.
  - Als je `ui.image_protocol = "iterm2"` zet (of een versie ouder dan 3.7.7 gebruikt), vink dan
    "Remember my choice" aan in het dialoogvenster "Allow Terminal-Initiated Display?". De afbeeldingen
    hebben dan een ondoorzichtige achtergrond (een zichtbare rechthoek als je achtergrond afwijkt van die van het
    thema; `ui.graph_renderer = "text"` haalt hem weg).
  - Het kitty-toetsenbordprotocol staat standaard aan (3.6.9 en nieuwer).
- **Terminal.app.** Alleen 256 kleuren: GitPoulpi kiest zelf `ui.color_depth = 256` en
  thema's zien er vlakker uit. Geen afbeeldingen, geen kitty-toetsenbord, geen klembord via `OSC 52`. Vink
  *Use Option as Meta key* aan voor de `Alt`-combinaties.
- **Alacritty, Tabby.** Alacritty: tekengrafiek, kitty-toetsenbord, kopiëren via `OSC 52`; zet
  `option_as_alt = "Both"` voor de `Alt`-combinaties. Tabby: sixel-grafiek en -avatars.
- **Option als Meta.** Option stelt tekens samen (Option+B typt een integraalteken) tenzij
  de terminal het in Meta verandert: iTerm2 *Left Option key: Esc+*, Terminal.app *Use Option as
  Meta key*, Alacritty `option_as_alt`. De `Alt`-combinaties van GitPoulpi hebben dat nodig, en nergens staat het
  standaard aan.
- **kitty, Ghostty, WezTerm, Rio, Warp**: niet gemeten op macOS (ze hebben een GPU nodig). Verwacht het
  gedrag van Linux.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; sixel heeft `ui.image_protocol = "sixel"` nodig.

### VS Code en Warp

De VS Code-terminal (`TERM_PROGRAM=vscode`) maakt geen deel uit van de ronde. Hij wordt herkend
aan zijn apparaatattributen als `terminal.integrated.enableImages` aan staat, anders tekent GitPoulpi
tekens; `ui.image_protocol = "sixel"` forceert de afbeeldingen. Warp: zie de notities per besturingssysteem hierboven.

## Multiplexers

tmux, zellij, screen, herdr, psmux en de eigen panes van de terminals staan tussen GitPoulpi en de
terminal en veranderen wat er doorkomt. De korte versie: tmux laat sixel door en, met
`allow-passthrough on`, aanwijzervormen en kitty-afbeeldingen; zellij en screen laten
geen afbeeldingen en geen aanwijzervormen door; herdr laat kitty-afbeeldingen door. Alle details en de
toe te voegen configuratie staan in [Multiplexers](multiplexers.md).

## Wat wegvalt, en wat er dan komt

| Ontbreekt | Wat je in de plaats krijgt |
|---|---|
| Afbeeldingsprotocol | Tekengrafiek. De avatar wordt de gekleurde initialenbadge (één letter), wat `ui.avatar_fallback` ook zegt: een identicon in tekstcellen geperst is onherkenbaar. Het afbeeldingsverschil en de Bestandsweergave gebruiken gekleurde halve blokken, of een infokaart. |
| Truecolor | Kleuren worden op het palet van 256 of 16 kleuren afgebeeld (`ui.color_depth`). |
| Kitty-toetsenbordprotocol | Klassieke toetsen; sommige toetscombinaties komen als een andere toets aan. Instellingen waarschuwt, en je wijst opnieuw toe of gebruikt de `Alt`-varianten. |
| OSC 52 | Lokaal verandert er niets (het systeemklembord wordt gebruikt). Via SSH of zonder beeldscherm zetten kopieeracties niets op het klembord: selecteer met de muis (houd `Shift` ingedrukt) en gebruik de eigen kopieeropdracht van de terminal. |
| OSC 22 | De aanwijzer verandert niet; de scheidingslijn of knop onder de muis licht nog steeds op. |
| Emoji-lettertype | Vakjes of gaten in plaats van pictogrammen; de grafiek en de tekst werken nog. Installeer een Nerd Font of een emoji-lettertype en vermeld het als fallback. |

## Instellingen om te kennen

| Instelling | Gebruik |
|---|---|
| `ui.image_protocol` | `auto` (standaard), `kitty`, `iterm2`, `sixel`, `none`. Zet `none` als je rommel ziet waar afbeeldingen horen. Herstart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` is altijd correct (gebruik het waar de ConPTY-beperking geldt). Herstart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` of `off`, als een terminal of multiplexer zich misdraagt. Herstart. |
| `ui.text_cursor` | `terminal` (standaard) of `drawn`. |
| `ui.mouse_enabled` | Uit laat de muis aan de terminal. Herstart. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | De terminal die de snelkoppelingen van het besturingssysteem openen, en of een dubbelklik GitPoulpi daar heropent ([Instellingen](settings.md)). |

Zie ook [Multiplexers](multiplexers.md), [Afbeeldingen](image-view.md),
[Sneltoetsen](keybindings.md) en [Probleemoplossing](troubleshooting.md).

## Bronnen

Documentatie gecontroleerd in oktober 2026, gedrag getest in november 2026. Primaire bronnen, tenzij anders vermeld.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); kitty-toetsenbord uit de [terminfo.dev-matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secundair)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secundair), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [escape sequence support](https://rioterm.com/docs/escape-sequence-support) (geen `OSC 22`-rij), [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 uit een [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secundair)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (geen true color): [Apple discussion](https://discussions.apple.com/thread/255315820) (secundair)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Vergelijkingssites (secundair, alleen gebruikt om te kruiscontroleren): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
